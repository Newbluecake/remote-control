# remote-control

Synchronized movie watching — sync mpv playback with your partner over the network.

Each person runs [mpv](https://mpv.io/) locally with the same video file. **remote-control** syncs playback controls (pause, seek, speed, subtitles) through a lightweight WebSocket relay server, so you watch in perfect sync without video streaming lag.

## Features

- **Play/pause sync** — one person pauses, both pause
- **Seek sync** — skip forward/backward together
- **Speed & subtitle track sync** — change playback speed or subtitle track, synced instantly
- **Drift correction** — automatic periodic position correction via heartbeats
- **Anti-loop guard** — prevents command echo (no infinite feedback loops)
- **Auto-reconnect** — exponential backoff on server disconnect, seamless recovery
- **Room system** — simple room codes to find each other; auto-generated if not provided
- **Terminal chat** — type messages directly in the terminal while watching
- **Cross-platform** — Linux, macOS, Windows (Unix sockets + Windows named pipes)
- **Single binary** — one 3MB executable acts as both server and client

## Quick Start

### Prerequisites

- [mpv](https://mpv.io/) installed on both machines
- Same video file on both machines

### 1. Start the relay server

Run on any machine reachable by both people (a VPS, or one of your own machines):

```bash
remote-control serve
```

Default bind: `0.0.0.0:9090`. Customize with `--bind`.

### 2. Start mpv with IPC enabled

```bash
# Linux / macOS
mpv --input-ipc-server=/tmp/mpvsocket movie.mkv

# Windows
mpv --input-ipc-server=\\.\pipe\mpvsocket movie.mkv
```

### 3. Join a room

**Person A** (creates the room):

```bash
remote-control join --server ws://your-server:9090 --nickname alice
```

The terminal prints the auto-generated room code:

```
--- Room: X7KP | Peers: 1 | Type to chat, Enter to send ---
```

**Person B** (joins with the code):

```bash
remote-control join --server ws://your-server:9090 --room X7KP --nickname bob
```

Now any playback action on either side is synced to the other.

## CLI Reference

```
remote-control <COMMAND>

Commands:
  serve    Start the WebSocket relay server
  join     Join a room and sync with a partner
```

### `serve`

| Flag | Default | Description |
|------|---------|-------------|
| `-b, --bind` | `0.0.0.0:9090` | Address to bind the server to |

### `join`

| Flag | Default | Description |
|------|---------|-------------|
| `-S, --server` | `ws://localhost:9090` | WebSocket server URL |
| `-r, --room` | *(auto-generated)* | Room code to join |
| `-n, --nickname` | `anon` | Your display name |
| `-s, --mpv-socket` | `/tmp/mpvsocket` | Path to mpv IPC socket |
| `--drift-threshold` | `0.5` | Drift correction threshold (seconds) |

## Architecture

```
Person A                          Relay Server                       Person B
┌──────────┐    WebSocket    ┌──────────────────┐    WebSocket    ┌──────────┐
│ mpv ←IPC→ client ──────────→ room forwarding ←────────── client ←IPC→ mpv │
└──────────┘                 └──────────────────┘                 └──────────┘
```

See [docs/architecture.md](docs/architecture.md) for detailed design documentation.

## Building from Source

```bash
git clone https://github.com/Newbluecake/remote-control.git
cd remote-control
cargo build --release
# Binary at target/release/remote-control
```

## Documentation

| Document | Description |
|----------|-------------|
| [Getting Started](docs/getting-started.md) | Detailed setup guide |
| [Architecture](docs/architecture.md) | System design and protocol spec |
| [Development](docs/development.md) | Contributing and development guide |
| [Deployment](docs/deployment.md) | Server deployment guide |
| [CONTRIBUTING](CONTRIBUTING.md) | Contribution guidelines |
| [CHANGELOG](CHANGELOG.md) | Version history |

## License

MIT
