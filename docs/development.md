# Development Guide

## Setup

```bash
git clone https://github.com/Newbluecake/remote-control.git
cd remote-control
cargo build
```

Requirements:
- Rust 1.70+ (edition 2021)
- mpv (for testing)

## Project Structure

```
src/
├── main.rs              # Entry point
├── cli.rs               # CLI definitions (clap derive)
├── protocol.rs          # Message types shared between client and server
├── mpv/
│   ├── ipc.rs           # Async mpv JSON IPC connection
│   └── controller.rs    # High-level mpv property observation and control
├── relay/
│   ├── mod.rs           # WebSocket server
│   └── room.rs          # Room state management
└── client/
    ├── mod.rs           # Client session loop
    └── sync.rs          # Anti-loop guard and drift correction
```

## Local Testing

Open 4 terminal windows:

```bash
# Terminal 1: Relay server
cargo run -- serve

# Terminal 2: First mpv instance
mpv --input-ipc-server=/tmp/mpvsocket1 test-video.mp4

# Terminal 3: First client
cargo run -- join --mpv-socket /tmp/mpvsocket1 --room TEST --nickname alice

# Terminal 4: Second mpv + client
mpv --input-ipc-server=/tmp/mpvsocket2 test-video.mp4
cargo run -- join --mpv-socket /tmp/mpvsocket2 --room TEST --nickname bob
```

Pause/seek in one mpv window and observe the other syncing.

## Key Design Patterns

### Async Event Loop (tokio::select!)

The client runs a single async loop with four branches:
1. mpv events (property changes)
2. WebSocket messages (from relay server)
3. Heartbeat timer (every 5s)
4. stdin (chat input)

### Anti-Loop Suppression

When applying a remote command, we "suppress" the expected mpv echo:
```rust
guard.suppress("pause", Value::Bool(true));
mpv.set_pause(true).await;
// When mpv fires PauseChanged(true), guard prevents re-broadcast
```

### Platform Gating

Windows vs Unix differences are isolated in `mpv/ipc.rs` with `#[cfg]`:
```rust
#[cfg(unix)]
pub async fn connect(path: &str) -> Result<Self> { /* UnixStream */ }

#[cfg(windows)]
pub async fn connect(path: &str) -> Result<Self> { /* NamedPipeClient */ }
```

## Adding a New Synced Property

1. Add the property to `PeerMessage` in `protocol.rs`
2. Add an `MpvEvent` variant in `mpv/controller.rs`
3. Add `observe_property()` call in `MpvController::connect()`
4. Add match arm in `MpvController::recv_event()`
5. Add `should_broadcast_*()` method to `SyncGuard`
6. Handle the new event in `client/mod.rs` (both send and receive paths)

## Code Quality

```bash
cargo clippy           # Lint
cargo fmt              # Format
cargo build --release  # Release build (3.2MB binary)
```

## Debug Logging

Set `RUST_LOG` for detailed tracing output:

```bash
RUST_LOG=debug cargo run -- join ...
RUST_LOG=remote_control=trace cargo run -- serve
```
