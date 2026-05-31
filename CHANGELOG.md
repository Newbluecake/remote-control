# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

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
