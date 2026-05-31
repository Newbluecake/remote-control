# Deployment Guide

## Relay Server Deployment

The relay server is a single binary that listens on a TCP port. It's stateless (no database, no disk) and uses minimal resources.

### Requirements

- Any machine reachable by both clients
- One open TCP port (default: 9090)
- ~5MB RAM, negligible CPU

### Option 1: Direct binary on a VPS

```bash
# On your VPS (e.g., DigitalOcean, Vultr, Hetzner)
scp target/release/remote-control user@your-server:~/

# SSH into the server
ssh user@your-server

# Run in background
nohup ./remote-control serve --bind 0.0.0.0:9090 &

# Or with systemd (see below)
```

### Option 2: systemd service

Create `/etc/systemd/system/remote-control.service`:

```ini
[Unit]
Description=remote-control relay server
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/remote-control serve --bind 0.0.0.0:9090
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

```bash
sudo cp remote-control /usr/local/bin/
sudo systemctl daemon-reload
sudo systemctl enable --now remote-control
sudo systemctl status remote-control
```

### Option 3: Docker

```dockerfile
FROM rust:1.75-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/remote-control /usr/local/bin/
EXPOSE 9090
CMD ["remote-control", "serve", "--bind", "0.0.0.0:9090"]
```

```bash
docker build -t remote-control .
docker run -d -p 9090:9090 --name remote-control remote-control
```

### Option 4: Fly.io (free tier)

```bash
# fly.toml
flyctl launch --name remote-control
flyctl deploy
```

## Firewall Configuration

Ensure port 9090 (or your chosen port) is open for TCP traffic:

```bash
# UFW (Ubuntu)
sudo ufw allow 9090/tcp

# firewalld (CentOS/Fedora)
sudo firewall-cmd --permanent --add-port=9090/tcp
sudo firewall-cmd --reload

# iptables
sudo iptables -A INPUT -p tcp --dport 9090 -j ACCEPT
```

For cloud providers, also configure the security group / firewall rules in the web console.

## TLS (Future)

Currently the relay uses plain WebSocket (`ws://`). For production use over the internet, consider putting it behind a reverse proxy with TLS:

### nginx reverse proxy

```nginx
server {
    listen 443 ssl;
    server_name sync.example.com;

    ssl_certificate /etc/letsencrypt/live/sync.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/sync.example.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:9090;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
}
```

Then clients connect with:
```bash
remote-control join --server wss://sync.example.com ...
```

## Monitoring

The server logs connection events via tracing:

```
INFO Relay server listening on 0.0.0.0:9090
INFO addr=1.2.3.4:54321 nickname=alice room_code=X7KP Joined room (1 peers)
INFO addr=5.6.7.8:12345 nickname=bob room_code=X7KP Joined room (2 peers)
```

For production, pipe logs to journald or a log aggregator.

## Resource Usage

The relay server is extremely lightweight:
- **Memory**: ~5MB base + ~1KB per active connection
- **CPU**: negligible (just JSON parsing and forwarding)
- **Bandwidth**: ~1KB/s per active room (heartbeats + occasional commands)

A $4/month VPS can handle hundreds of concurrent rooms.
