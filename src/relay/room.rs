use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

struct Peer {
    #[allow(dead_code)]
    nickname: String,
    tx: mpsc::UnboundedSender<String>,
}

struct Room {
    peers: HashMap<SocketAddr, Peer>,
}

#[derive(Clone)]
pub struct RoomManager {
    rooms: Arc<Mutex<HashMap<String, Room>>>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn join(
        &self,
        room_code: &str,
        nickname: &str,
        addr: SocketAddr,
        tx: mpsc::UnboundedSender<String>,
    ) -> usize {
        let mut rooms = self.rooms.lock().unwrap();
        let room = rooms.entry(room_code.to_string()).or_insert_with(|| Room {
            peers: HashMap::new(),
        });

        room.peers.insert(
            addr,
            Peer {
                nickname: nickname.to_string(),
                tx,
            },
        );
        room.peers.len()
    }

    pub fn leave(&self, room_code: &str, addr: SocketAddr) {
        let mut rooms = self.rooms.lock().unwrap();
        if let Some(room) = rooms.get_mut(room_code) {
            room.peers.remove(&addr);
            if room.peers.is_empty() {
                rooms.remove(room_code);
            }
        }
    }

    pub fn broadcast(&self, room_code: &str, from: SocketAddr, message: &str) {
        let rooms = self.rooms.lock().unwrap();
        if let Some(room) = rooms.get(room_code) {
            for (addr, peer) in &room.peers {
                if *addr != from {
                    let _ = peer.tx.send(message.to_string());
                }
            }
        }
    }
}
