use std::collections::HashMap;
use std::time::Instant;

use serde_json::Value;

pub struct SyncGuard {
    suppressed: HashMap<String, (Value, Instant)>,
}

const SUPPRESS_DURATION_MS: u64 = 500;
const POSITION_TOLERANCE: f64 = 1.0;
const SPEED_TOLERANCE: f64 = 0.01;

impl SyncGuard {
    pub fn new() -> Self {
        Self {
            suppressed: HashMap::new(),
        }
    }

    pub fn suppress(&mut self, property: &str, value: Value) {
        let expiry = Instant::now() + std::time::Duration::from_millis(SUPPRESS_DURATION_MS);
        self.suppressed
            .insert(property.to_string(), (value, expiry));
    }

    pub fn should_broadcast_pause(&mut self, paused: bool) -> bool {
        self.check("pause", Value::Bool(paused))
    }

    pub fn should_broadcast_position(&mut self, position: f64) -> bool {
        self.check_float("playback-time", position, POSITION_TOLERANCE)
    }

    pub fn should_broadcast_speed(&mut self, speed: f64) -> bool {
        self.check_float("speed", speed, SPEED_TOLERANCE)
    }

    pub fn should_broadcast_sub_track(&mut self, track_id: i64) -> bool {
        self.check("sid", Value::Number(track_id.into()))
    }

    pub fn suppress_sub_track(&mut self, track_id: i64) {
        self.suppress("sid", Value::Number(track_id.into()));
    }

    fn check(&mut self, property: &str, value: Value) -> bool {
        if let Some((expected, expiry)) = self.suppressed.get(property) {
            if Instant::now() < *expiry && *expected == value {
                self.suppressed.remove(property);
                return false;
            }
            self.suppressed.remove(property);
        }
        true
    }

    fn check_float(&mut self, property: &str, value: f64, tolerance: f64) -> bool {
        if let Some((expected, expiry)) = self.suppressed.get(property) {
            if Instant::now() < *expiry {
                if let Some(expected_f) = expected.as_f64() {
                    if (expected_f - value).abs() < tolerance {
                        self.suppressed.remove(property);
                        return false;
                    }
                }
            }
            self.suppressed.remove(property);
        }
        true
    }
}
