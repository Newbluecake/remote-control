use std::collections::VecDeque;
use std::time::Instant;

use rdev::Key;

const SUPPRESS_WINDOW_MS: u64 = 100;

pub struct SyncGuard {
    recent_simulations: VecDeque<(Key, bool, Instant)>,
}

impl SyncGuard {
    pub fn new() -> Self {
        Self {
            recent_simulations: VecDeque::new(),
        }
    }

    pub fn mark_simulated(&mut self, key: Key, pressed: bool) {
        self.recent_simulations
            .push_back((key, pressed, Instant::now()));
        self.gc();
    }

    pub fn should_broadcast(&mut self, key: Key, pressed: bool) -> bool {
        self.gc();
        if let Some(idx) = self
            .recent_simulations
            .iter()
            .position(|(k, p, _)| *k == key && *p == pressed)
        {
            self.recent_simulations.remove(idx);
            return false;
        }
        true
    }

    fn gc(&mut self) {
        let cutoff = Instant::now() - std::time::Duration::from_millis(SUPPRESS_WINDOW_MS);
        while let Some((_, _, t)) = self.recent_simulations.front() {
            if *t < cutoff {
                self.recent_simulations.pop_front();
            } else {
                break;
            }
        }
    }
}
