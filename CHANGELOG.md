# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [0.2.0] - 2026-06-01

### Added

- Global keyboard synchronization — any key pressed on one machine is forwarded and replayed on remote peers, enabling sync with any application ([960d25f](https://github.com/Newbluecake/remote-control/commit/960d25f))
- Local key capture logging with buffered output and run-length compression (`Key xN` format) ([cf74c3a](https://github.com/Newbluecake/remote-control/commit/cf74c3a), [e83595b](https://github.com/Newbluecake/remote-control/commit/e83595b))
- Version display with git commit hash on startup and `--version` flag ([d3fb7b1](https://github.com/Newbluecake/remote-control/commit/d3fb7b1))
- macOS ad-hoc signing script (`sign-mac.sh`) included in release archives ([cf74c3a](https://github.com/Newbluecake/remote-control/commit/cf74c3a))
- `Ctrl+Shift+F12` hotkey to toggle keyboard sync on/off
- `--no-sync` CLI flag to start with sync disabled

### Removed

- mpv IPC module — replaced by generic keyboard synchronization

### Changed

- Protocol messages changed from mpv-specific commands to generic `KeyEvent`
- SyncGuard rewritten for keyboard echo suppression (FIFO deque, 100ms window)

Stats: 3 feat, 1 refactor

## [0.1.0] - 2025-05-31

### Added

- mpv JSON IPC controller with async Unix socket and Windows named pipe support
- WebSocket relay server with room-based message forwarding
- Play/pause, seek, speed, and subtitle track synchronization
- Anti-loop guard to prevent command echo feedback loops
- Heartbeat-based drift correction (configurable threshold)
- Auto-reconnect with exponential backoff on server disconnect
- Terminal text chat between room peers
- Auto-generated room codes (4-character alphanumeric)
- Cross-platform support (Linux, macOS, Windows)
- CLI with `serve` and `join` subcommands
