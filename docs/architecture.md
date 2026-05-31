# Architecture

## Overview

remote-control is a single Rust binary with two modes:

- **`serve`** — stateless WebSocket relay server that forwards messages between room peers
- **`join`** — client that bridges local mpv IPC events with the relay network

## System Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Relay Server                                     │
│                                                                               │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │  RoomManager                                                          │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐                           │   │
│  │  │ Room "A" │  │ Room "B" │  │ Room "C" │  ...                       │   │
│  │  │ peers: 2 │  │ peers: 2 │  │ peers: 1 │                           │   │
│  │  └──────────┘  └──────────┘  └──────────┘                           │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                        ↕ WebSocket (per connection)                           │
└─────────────────────────────────────────────────────────────────────────────┘
          ↕                                                    ↕
┌─────────────────────┐                          ┌─────────────────────┐
│ Client A             │                          │ Client B             │
│                      │                          │                      │
│ ┌──────────────────┐│                          │┌──────────────────┐ │
│ │ SyncGuard        ││                          ││ SyncGuard        │ │
│ │ (anti-loop)      ││                          ││ (anti-loop)      │ │
│ └──────────────────┘│                          │└──────────────────┘ │
│         ↕            │                          │         ↕           │
│ ┌──────────────────┐│                          │┌──────────────────┐ │
│ │ MpvController    ││                          ││ MpvController    │ │
│ │ (observe + cmd)  ││                          ││ (observe + cmd)  │ │
│ └──────────────────┘│                          │└──────────────────┘ │
│         ↕ IPC        │                          │         ↕ IPC       │
│ ┌──────────────────┐│                          │┌──────────────────┐ │
│ │ mpv player       ││                          ││ mpv player       │ │
│ └──────────────────┘│                          │└──────────────────┘ │
└─────────────────────┘                          └─────────────────────┘
```

## Module Structure

```
src/
├── main.rs              # Entry point, CLI dispatch
├── cli.rs               # clap command definitions
├── protocol.rs          # Wire protocol types (PeerMessage, RelayMessage)
├── mpv/
│   ├── ipc.rs           # Low-level mpv JSON IPC (platform-gated)
│   └── controller.rs    # High-level mpv control (observe properties, send commands)
├── relay/
│   ├── mod.rs           # WebSocket server, connection handling
│   └── room.rs          # Room state, peer membership, message broadcast
└── client/
    ├── mod.rs           # Client session: mpv ↔ WebSocket bridge, reconnection
    └── sync.rs          # Anti-loop suppression guard, drift correction logic
```

## Wire Protocol

All messages are JSON over WebSocket, using serde's internally-tagged representation (`"type"` field).

### PeerMessage (between clients, forwarded by relay)

| Type | Fields | Description |
|------|--------|-------------|
| `SetPause` | `paused`, `position`, `timestamp` | Play/pause with current position |
| `Seek` | `position`, `timestamp` | Absolute seek |
| `SetSpeed` | `speed`, `timestamp` | Playback speed change |
| `SetSubTrack` | `track_id`, `timestamp` | Subtitle track switch |
| `Heartbeat` | `position`, `paused`, `speed`, `timestamp` | Periodic state sync |
| `Chat` | `text`, `timestamp` | Text chat message |

### RelayMessage (client ↔ server envelope)

| Type | Fields | Description |
|------|--------|-------------|
| `JoinRoom` | `room_code`, `nickname` | Client requests to join a room |
| `RoomJoined` | `room_code`, `peer_count` | Server confirms join |
| `PeerJoined` | `nickname` | Another peer joined |
| `PeerLeft` | `nickname` | A peer left |
| `Peer` | `from`, `message` | Forwarded PeerMessage |
| `Error` | `message` | Server error |

### Example wire message

```json
{
  "type": "Peer",
  "from": "alice",
  "message": {
    "type": "SetPause",
    "paused": true,
    "position": 123.45,
    "timestamp": 1622505600000
  }
}
```

## Sync Protocol

### Anti-Loop Guard

When a remote command is applied to local mpv, it triggers a property-change event. Without protection, this event would be re-broadcast, creating an infinite loop.

**Solution: Suppression Window**

1. Before applying a remote command, insert the property name and expected value into a suppression map with a 500ms expiry.
2. When a property-change event arrives from mpv, check the suppression map.
3. If the property+value match within tolerance and the window hasn't expired, discard the event (it was our own echo).
4. Otherwise, broadcast as a genuine local user action.

Tolerances:
- Position: 1.0 second
- Speed: 0.01x
- Pause/SubTrack: exact match

### Drift Correction

Both clients send a `Heartbeat` every 5 seconds. When a heartbeat arrives:

1. If either side is paused, no correction needed.
2. Compare local position with remote position.
3. If drift exceeds the threshold (default 0.5s), seek to the remote position.
4. The correction seek goes through the suppression guard (won't be re-broadcast).

### Reconnection

On WebSocket disconnect:
- The client retries with exponential backoff (1s → 2s → 4s → ... → 30s max).
- The mpv connection is preserved across relay reconnects.
- If mpv itself disconnects, the client exits cleanly.

## mpv IPC Protocol

remote-control communicates with mpv via its JSON IPC interface:
- **Unix**: Unix domain socket (default: `/tmp/mpvsocket`)
- **Windows**: Named pipe (default: `\\.\pipe\mpvsocket`)

The protocol is newline-delimited JSON. Each command includes a `request_id` for matching responses. Property observations generate unsolicited events that are demultiplexed by a reader task.

Observed properties:
| ID | Property | Purpose |
|----|----------|---------|
| 1 | `pause` | Detect play/pause |
| 2 | `playback-time` | Detect seeks, measure drift |
| 3 | `speed` | Detect speed changes |
| 4 | `sid` | Detect subtitle track switches |

## Design Decisions

| Decision | Rationale |
|----------|-----------|
| Single binary for server + client | Simplifies distribution, one artifact |
| Custom mpv IPC instead of crate | Existing crates are sync-only or spawn mpv; JSON IPC is ~150 lines |
| WebSocket relay over P2P | NAT traversal is trivial; P2P adds STUN/TURN complexity |
| `Mutex<HashMap>` over actors | Only 2 peers per room; actor framework is overkill |
| JSON over protobuf | Human-readable for debugging; message volume is tiny |
| No latency compensation | 100ms delay is imperceptible for movies; drift correction handles the rest |
