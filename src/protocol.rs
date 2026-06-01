use rdev::Key;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PeerMessage {
    KeyEvent {
        key: Key,
        pressed: bool,
        timestamp: u64,
    },
    Chat {
        text: String,
        timestamp: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RelayMessage {
    JoinRoom {
        room_code: String,
        nickname: String,
    },
    RoomJoined {
        room_code: String,
        peer_count: usize,
    },
    PeerJoined {
        nickname: String,
    },
    PeerLeft {
        nickname: String,
    },
    Peer {
        from: String,
        message: PeerMessage,
    },
    Error {
        message: String,
    },
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
