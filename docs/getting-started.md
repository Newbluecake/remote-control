# Getting Started

## Prerequisites

1. **mpv** — a free, open-source media player
   - Linux: `sudo apt install mpv` or `sudo pacman -S mpv`
   - macOS: `brew install mpv`
   - Windows: [download from mpv.io](https://mpv.io/installation/)

2. **remote-control binary** — download from [GitHub Releases](https://github.com/Newbluecake/remote-control/releases) or build from source.

3. **Same video file** on both machines (filename doesn't matter, content does).

## Installation

### From source

```bash
git clone https://github.com/Newbluecake/remote-control.git
cd remote-control
cargo build --release
cp target/release/remote-control ~/.local/bin/  # or anywhere in PATH
```

### Verify installation

```bash
remote-control --version
remote-control --help
```

## Step-by-step Usage

### Step 1: Run the relay server

Pick a machine that both people can reach (a VPS, cloud server, or one of your own machines with port forwarding).

```bash
remote-control serve --bind 0.0.0.0:9090
```

The server is completely stateless — it just forwards messages. It can handle many rooms concurrently.

### Step 2: Start mpv with IPC

On **both** machines, start mpv with the IPC server enabled:

```bash
# Linux / macOS
mpv --input-ipc-server=/tmp/mpvsocket "My Movie.mkv"

# Windows (PowerShell)
mpv --input-ipc-server=\\.\pipe\mpvsocket "My Movie.mkv"
```

> **Tip:** You can add `input-ipc-server=/tmp/mpvsocket` to your `~/.config/mpv/mpv.conf` so it's always enabled.

### Step 3: Connect the first person

```bash
remote-control join \
  --server ws://your-server-ip:9090 \
  --nickname alice
```

Output:
```
2024-01-15T20:00:00 INFO Connecting to mpv at /tmp/mpvsocket
2024-01-15T20:00:00 INFO Connecting to relay server at ws://your-server-ip:9090
2024-01-15T20:00:00 INFO Joined room X7KP (1 peers)
--- Room: X7KP | Peers: 1 | Type to chat, Enter to send ---
```

Share the room code **X7KP** with your partner.

### Step 4: Connect the second person

```bash
remote-control join \
  --server ws://your-server-ip:9090 \
  --room X7KP \
  --nickname bob
```

Output:
```
--- Room: X7KP | Peers: 2 | Type to chat, Enter to send ---
>>> alice joined
```

### Step 5: Watch together!

Now any action on either mpv is synced:

- **Space** (pause/play) — synced
- **Arrow keys** (seek) — synced
- **[** / **]** (speed change) — synced
- **j** (cycle subtitles) — synced

Type text in the terminal and press Enter to chat:
```
hi, ready to start?
[bob] yeah let's go!
```

## Troubleshooting

### "Failed to connect to mpv"

mpv must be running **before** you run `remote-control join`. Make sure the IPC socket path matches:

```bash
# Check if mpv socket exists
ls /tmp/mpvsocket
# or on Windows:
# The named pipe is ephemeral, it exists while mpv runs
```

### "Failed to connect to relay server"

- Check the server is running (`remote-control serve`)
- Check the port is open (firewall, security group)
- Check the URL is correct (`ws://` not `wss://`)

### Video is out of sync

The drift threshold defaults to 0.5 seconds. If you're on a high-latency connection, try increasing it:

```bash
remote-control join --drift-threshold 1.0 ...
```

### Reconnection

If the server goes down briefly, the client auto-reconnects with exponential backoff (1s → 30s). You'll see:

```
WARN Server disconnected, reconnecting in 1s...
INFO Joined room X7KP (2 peers)
```
