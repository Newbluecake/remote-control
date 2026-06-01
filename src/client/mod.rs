pub mod sync;

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::io::AsyncBufReadExt;
use tokio::time::{self, Duration};
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};

use crate::cli::JoinArgs;
use crate::keyboard::listener::{simulate_key, KeyAction, KeyboardBridge};
use crate::protocol::{self, PeerMessage, RelayMessage};

use self::sync::SyncGuard;

pub async fn run_client(args: JoinArgs) -> Result<()> {
    info!(
        "Starting keyboard capture (Ctrl+Shift+F12 to toggle sync, currently {})",
        if args.no_sync { "DISABLED" } else { "ENABLED" }
    );
    let mut keyboard = KeyboardBridge::new(!args.no_sync);

    let mut backoff = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(30);

    loop {
        match run_session(&args, &mut keyboard).await {
            Ok(SessionEnd::KeyboardClosed) => {
                info!("Keyboard listener closed, exiting");
                return Ok(());
            }
            Ok(SessionEnd::ServerDisconnected) => {
                warn!(
                    "Server disconnected, reconnecting in {}s...",
                    backoff.as_secs()
                );
                time::sleep(backoff).await;
                backoff = (backoff * 2).min(max_backoff);
            }
            Err(e) => {
                warn!(
                    "Session error: {e:#}, reconnecting in {}s...",
                    backoff.as_secs()
                );
                time::sleep(backoff).await;
                backoff = (backoff * 2).min(max_backoff);
            }
        }
    }
}

enum SessionEnd {
    KeyboardClosed,
    ServerDisconnected,
}

async fn run_session(args: &JoinArgs, keyboard: &mut KeyboardBridge) -> Result<SessionEnd> {
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
            println!(
                "--- Room: {room_code} | Peers: {peer_count} | Ctrl+Shift+F12 to toggle sync | Type to chat ---"
            );
        }
    }

    let mut guard = SyncGuard::new();
    let nickname = args.nickname.clone();

    let stdin = tokio::io::BufReader::new(tokio::io::stdin());
    let mut stdin_lines = stdin.lines();

    loop {
        tokio::select! {
            key_event = keyboard.recv_key_event() => {
                let action = match key_event {
                    Some(a) => a,
                    None => return Ok(SessionEnd::KeyboardClosed),
                };

                if !guard.should_broadcast(action.key, action.pressed) {
                    continue;
                }

                let msg = RelayMessage::Peer {
                    from: nickname.clone(),
                    message: PeerMessage::KeyEvent {
                        key: action.key,
                        pressed: action.pressed,
                        timestamp: protocol::now_ms(),
                    },
                };
                let text = serde_json::to_string(&msg)?;
                if ws_tx.send(Message::Text(text.into())).await.is_err() {
                    return Ok(SessionEnd::ServerDisconnected);
                }
            }

            msg = ws_rx.next() => {
                let msg = match msg {
                    Some(Ok(msg)) => msg,
                    _ => return Ok(SessionEnd::ServerDisconnected),
                };

                if let Message::Text(text) = msg {
                    handle_server_message(&text, &mut guard).await;
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

async fn handle_server_message(text: &str, guard: &mut SyncGuard) {
    match serde_json::from_str::<RelayMessage>(text) {
        Ok(RelayMessage::Peer { from, message }) => match message {
            PeerMessage::KeyEvent { key, pressed, .. } => {
                let action_str = if pressed { "press" } else { "release" };
                info!("{from} -> {action_str} {key:?}");
                guard.mark_simulated(key, pressed);
                let action = KeyAction { key, pressed };
                tokio::task::spawn_blocking(move || {
                    simulate_key(&action);
                })
                .await
                .ok();
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
