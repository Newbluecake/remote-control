pub mod sync;

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::io::AsyncBufReadExt;
use tokio::time::{self, Duration};
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};

use crate::cli::JoinArgs;
use crate::mpv::controller::{MpvController, MpvEvent};
use crate::protocol::{self, PeerMessage, RelayMessage};

use self::sync::SyncGuard;

pub async fn run_client(args: JoinArgs) -> Result<()> {
    info!("Connecting to mpv at {}", args.mpv_socket);
    let mut mpv = MpvController::connect(&args.mpv_socket)
        .await
        .context("Failed to connect to mpv. Is mpv running with --input-ipc-server?")?;

    let mut backoff = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(30);

    loop {
        match run_session(&args, &mut mpv).await {
            Ok(SessionEnd::MpvClosed) => {
                info!("mpv closed, exiting");
                return Ok(());
            }
            Ok(SessionEnd::ServerDisconnected) => {
                warn!("Server disconnected, reconnecting in {}s...", backoff.as_secs());
                time::sleep(backoff).await;
                backoff = (backoff * 2).min(max_backoff);
            }
            Err(e) => {
                warn!("Session error: {e:#}, reconnecting in {}s...", backoff.as_secs());
                time::sleep(backoff).await;
                backoff = (backoff * 2).min(max_backoff);
            }
        }
    }
}

enum SessionEnd {
    MpvClosed,
    ServerDisconnected,
}

async fn run_session(
    args: &JoinArgs,
    mpv: &mut MpvController,
) -> Result<SessionEnd> {
    info!("Connecting to relay server at {}", args.server);
    let (ws, _) = tokio_tungstenite::connect_async(&args.server)
        .await
        .context("Failed to connect to relay server")?;
    let (mut ws_tx, mut ws_rx) = ws.split();

    let join_msg = serde_json::to_string(&RelayMessage::JoinRoom {
        room_code: args.room.clone().unwrap_or_default(),
        nickname: args.nickname.clone(),
    })?;
    ws_tx.send(Message::Text(join_msg.into())).await?;

    if let Some(Ok(Message::Text(text))) = ws_rx.next().await {
        if let Ok(RelayMessage::RoomJoined {
            room_code,
            peer_count,
        }) = serde_json::from_str(&text)
        {
            info!("Joined room {room_code} ({peer_count} peers)");
            println!("--- Room: {room_code} | Peers: {peer_count} | Type to chat, Enter to send ---");
        }
    }

    let mut guard = SyncGuard::new();
    let mut heartbeat_interval = time::interval(Duration::from_secs(5));
    let drift_threshold = args.drift_threshold;
    let nickname = args.nickname.clone();

    let stdin = tokio::io::BufReader::new(tokio::io::stdin());
    let mut stdin_lines = stdin.lines();

    loop {
        tokio::select! {
            event = mpv.recv_event() => {
                let event = match event {
                    Some(e) => e,
                    None => return Ok(SessionEnd::MpvClosed),
                };

                let peer_msg = match event {
                    MpvEvent::Pause(paused) => {
                        if !guard.should_broadcast_pause(paused) {
                            continue;
                        }
                        let pos = mpv.get_position().await.unwrap_or(0.0);
                        info!("Local → pause={paused} pos={pos:.1}");
                        Some(PeerMessage::SetPause {
                            paused,
                            position: pos,
                            timestamp: protocol::now_ms(),
                        })
                    }
                    MpvEvent::Position(pos) => {
                        if !guard.should_broadcast_position(pos) {
                            continue;
                        }
                        info!("Local → seek {pos:.1}s");
                        Some(PeerMessage::Seek {
                            position: pos,
                            timestamp: protocol::now_ms(),
                        })
                    }
                    MpvEvent::Speed(speed) => {
                        if !guard.should_broadcast_speed(speed) {
                            continue;
                        }
                        info!("Local → speed {speed:.2}x");
                        Some(PeerMessage::SetSpeed {
                            speed,
                            timestamp: protocol::now_ms(),
                        })
                    }
                    MpvEvent::SubTrack(track_id) => {
                        if !guard.should_broadcast_sub_track(track_id) {
                            continue;
                        }
                        info!("Local → subtitle track {track_id}");
                        Some(PeerMessage::SetSubTrack {
                            track_id,
                            timestamp: protocol::now_ms(),
                        })
                    }
                };

                if let Some(msg) = peer_msg {
                    let relay = RelayMessage::Peer {
                        from: nickname.clone(),
                        message: msg,
                    };
                    let text = serde_json::to_string(&relay)?;
                    if ws_tx.send(Message::Text(text.into())).await.is_err() {
                        return Ok(SessionEnd::ServerDisconnected);
                    }
                }
            }

            msg = ws_rx.next() => {
                let msg = match msg {
                    Some(Ok(msg)) => msg,
                    _ => return Ok(SessionEnd::ServerDisconnected),
                };

                if let Message::Text(text) = msg {
                    handle_server_message(
                        &text, mpv, &mut guard, drift_threshold,
                    ).await;
                }
            }

            _ = heartbeat_interval.tick() => {
                let position = mpv.get_position().await.unwrap_or(0.0);
                let paused = mpv.get_pause().await.unwrap_or(false);
                let speed = mpv.get_speed().await.unwrap_or(1.0);

                let msg = RelayMessage::Peer {
                    from: nickname.clone(),
                    message: PeerMessage::Heartbeat {
                        position,
                        paused,
                        speed,
                        timestamp: protocol::now_ms(),
                    },
                };
                let text = serde_json::to_string(&msg)?;
                if ws_tx.send(Message::Text(text.into())).await.is_err() {
                    return Ok(SessionEnd::ServerDisconnected);
                }
            }

            line = stdin_lines.next_line() => {
                if let Ok(Some(text)) = line {
                    let text = text.trim().to_string();
                    if text.is_empty() {
                        continue;
                    }
                    let msg = RelayMessage::Peer {
                        from: nickname.clone(),
                        message: PeerMessage::Chat {
                            text,
                            timestamp: protocol::now_ms(),
                        },
                    };
                    let json = serde_json::to_string(&msg)?;
                    if ws_tx.send(Message::Text(json.into())).await.is_err() {
                        return Ok(SessionEnd::ServerDisconnected);
                    }
                }
            }
        }
    }
}

async fn handle_server_message(
    text: &str,
    mpv: &mut MpvController,
    guard: &mut SyncGuard,
    drift_threshold: f64,
) {
    match serde_json::from_str::<RelayMessage>(text) {
        Ok(RelayMessage::Peer { from, message }) => match message {
            PeerMessage::SetPause {
                paused, position, ..
            } => {
                info!("{from} → pause={paused} pos={position:.1}");
                guard.suppress("pause", serde_json::Value::Bool(paused));
                guard.suppress("playback-time", serde_json::json!(position));
                let _ = mpv.set_pause(paused).await;
                let _ = mpv.seek(position).await;
            }
            PeerMessage::Seek { position, .. } => {
                info!("{from} → seek {position:.1}s");
                guard.suppress("playback-time", serde_json::json!(position));
                let _ = mpv.seek(position).await;
            }
            PeerMessage::SetSpeed { speed, .. } => {
                info!("{from} → speed {speed:.2}x");
                guard.suppress("speed", serde_json::json!(speed));
                let _ = mpv.set_speed(speed).await;
            }
            PeerMessage::SetSubTrack { track_id, .. } => {
                info!("{from} → subtitle track {track_id}");
                guard.suppress_sub_track(track_id);
                let _ = mpv.set_sub_track(track_id).await;
            }
            PeerMessage::Heartbeat {
                position, paused, ..
            } => {
                let local_paused = mpv.get_pause().await.unwrap_or(false);
                if !paused && !local_paused {
                    let local_pos = mpv.get_position().await.unwrap_or(0.0);
                    let drift = (local_pos - position).abs();
                    if drift > drift_threshold {
                        info!(
                            "Drift correction: {drift:.2}s → seeking to {position:.1}s"
                        );
                        guard.suppress(
                            "playback-time",
                            serde_json::json!(position),
                        );
                        let _ = mpv.seek(position).await;
                    }
                }
            }
            PeerMessage::Chat { text, .. } => {
                println!("[{from}] {text}");
            }
        },
        Ok(RelayMessage::PeerJoined { nickname }) => {
            info!("{nickname} joined the room");
            println!(">>> {nickname} joined");
        }
        Ok(RelayMessage::PeerLeft { nickname }) => {
            info!("{nickname} left the room");
            println!(">>> {nickname} left");
        }
        Ok(RelayMessage::Error { message }) => {
            error!("Server error: {message}");
        }
        _ => {}
    }
}
