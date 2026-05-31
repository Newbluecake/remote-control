use anyhow::Result;
use serde_json::Value;
use tracing::debug;

use super::ipc::MpvIpc;

const OBS_PAUSE: u64 = 1;
const OBS_POSITION: u64 = 2;
const OBS_SPEED: u64 = 3;
const OBS_SID: u64 = 4;

#[derive(Debug, Clone)]
pub enum MpvEvent {
    Pause(bool),
    Position(f64),
    Speed(f64),
    SubTrack(i64),
}

pub struct MpvController {
    ipc: MpvIpc,
}

impl MpvController {
    pub async fn connect(socket_path: &str) -> Result<Self> {
        let mut ipc = MpvIpc::connect(socket_path).await?;

        ipc.observe_property(OBS_PAUSE, "pause").await?;
        ipc.observe_property(OBS_POSITION, "playback-time").await?;
        ipc.observe_property(OBS_SPEED, "speed").await?;
        ipc.observe_property(OBS_SID, "sid").await?;

        Ok(Self { ipc })
    }

    pub async fn set_pause(&mut self, paused: bool) -> Result<()> {
        self.ipc.set_property("pause", Value::Bool(paused)).await
    }

    pub async fn seek(&mut self, position: f64) -> Result<()> {
        self.ipc
            .set_property("playback-time", serde_json::json!(position))
            .await
    }

    pub async fn set_speed(&mut self, speed: f64) -> Result<()> {
        self.ipc
            .set_property("speed", serde_json::json!(speed))
            .await
    }

    pub async fn set_sub_track(&mut self, track_id: i64) -> Result<()> {
        self.ipc
            .set_property("sid", serde_json::json!(track_id))
            .await
    }

    pub async fn get_position(&mut self) -> Result<f64> {
        let val = self.ipc.get_property("playback-time").await?;
        Ok(val.as_f64().unwrap_or(0.0))
    }

    pub async fn get_pause(&mut self) -> Result<bool> {
        let val = self.ipc.get_property("pause").await?;
        Ok(val.as_bool().unwrap_or(false))
    }

    pub async fn get_speed(&mut self) -> Result<f64> {
        let val = self.ipc.get_property("speed").await?;
        Ok(val.as_f64().unwrap_or(1.0))
    }

    pub async fn recv_event(&mut self) -> Option<MpvEvent> {
        loop {
            let raw = self.ipc.recv_event().await?;

            if raw.get("event").and_then(|e| e.as_str()) != Some("property-change") {
                continue;
            }

            let id = raw.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
            let data = raw.get("data").cloned().unwrap_or(Value::Null);

            let event = match id {
                OBS_PAUSE => data.as_bool().map(MpvEvent::Pause),
                OBS_POSITION => data.as_f64().map(MpvEvent::Position),
                OBS_SPEED => data.as_f64().map(MpvEvent::Speed),
                OBS_SID => data.as_i64().map(MpvEvent::SubTrack),
                _ => None,
            };

            if let Some(event) = event {
                debug!(?event, "mpv property change");
                return Some(event);
            }
        }
    }
}
