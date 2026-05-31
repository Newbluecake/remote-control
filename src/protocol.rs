use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PeerMessage {
    SetPause {
        paused: bool,
        position: f64,
        timestamp: u64,
    },
    Seek {
        position: f64,
        timestamp: u64,
    },
    SetSpeed {
        speed: f64,
        timestamp: u64,
    },
    SetSubTrack {
        track_id: i64,
        timestamp: u64,
    },
    Heartbeat {
        position: f64,
        paused: bool,
        speed: f64,
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
