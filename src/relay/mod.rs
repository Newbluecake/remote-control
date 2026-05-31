mod room;

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info};

use self::room::RoomManager;
use crate::protocol::RelayMessage;

pub async fn run_server(bind: &str) -> Result<()> {
    let listener = TcpListener::bind(bind)
        .await
        .with_context(|| format!("Failed to bind to {bind}"))?;
    info!("Relay server listening on {bind}");

    let rooms = RoomManager::new();

    loop {
        let (stream, addr) = listener.accept().await?;
        let rooms = rooms.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, addr, rooms).await {
                error!(%addr, "Connection error: {e}");
            }
        });
    }
}

async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    rooms: RoomManager,
) -> Result<()> {
    let ws = tokio_tungstenite::accept_async(stream).await?;
    let (mut ws_tx, mut ws_rx) = ws.split();

    info!(%addr, "New WebSocket connection");

    // Wait for JoinRoom message
    let (room_code, nickname) = loop {
        let msg = ws_rx
            .next()
            .await
            .ok_or_else(|| anyhow::anyhow!("Connection closed before joining"))??;

        if let Message::Text(text) = msg {
            if let Ok(RelayMessage::JoinRoom {
                room_code,
                nickname,
            }) = serde_json::from_str(&text)
            {
                let room_code = if room_code.is_empty() {
                    generate_room_code()
                } else {
                    room_code
                };
                break (room_code, nickname);
            }
        }
    };

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    let peer_count = rooms.join(&room_code, &nickname, addr, tx);

    // Send RoomJoined to this client
    let joined_msg = serde_json::to_string(&RelayMessage::RoomJoined {
        room_code: room_code.clone(),
        peer_count,
    })?;
    ws_tx.send(Message::Text(joined_msg.into())).await?;

    // Notify other peers
    rooms.broadcast(
        &room_code,
        addr,
        &serde_json::to_string(&RelayMessage::PeerJoined {
            nickname: nickname.clone(),
        })?,
    );

    info!(%addr, %nickname, %room_code, "Joined room ({peer_count} peers)");

    loop {
        tokio::select! {
            // Messages from this client -> forward to other peers
            msg = ws_rx.next() => {
                let msg = match msg {
                    Some(Ok(msg)) => msg,
                    _ => break,
                };

                if let Message::Text(text) = msg {
                    if let Ok(RelayMessage::Peer { message, .. }) = serde_json::from_str::<RelayMessage>(&text) {
                        let forwarded = serde_json::to_string(&RelayMessage::Peer {
                            from: nickname.clone(),
                            message,
                        })?;
                        rooms.broadcast(&room_code, addr, &forwarded);
                    }
                }
            }
            // Messages from other peers -> send to this client
            msg = rx.recv() => {
                match msg {
                    Some(text) => {
                        if ws_tx.send(Message::Text(text.into())).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
        }
    }

    rooms.leave(&room_code, addr);
    rooms.broadcast(
        &room_code,
        addr,
        &serde_json::to_string(&RelayMessage::PeerLeft {
            nickname: nickname.clone(),
        })?,
    );

    info!(%addr, %nickname, "Left room {room_code}");
    Ok(())
}

fn generate_room_code() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::rng();
    (0..4)
        .map(|_| CHARSET[rng.random_range(0..CHARSET.len())] as char)
        .collect()
}
