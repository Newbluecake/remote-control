use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use rdev::{EventType, Key};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyAction {
    pub key: Key,
    pub pressed: bool,
}

pub struct KeyboardBridge {
    rx: mpsc::Receiver<KeyAction>,
    #[allow(dead_code)]
    enabled: Arc<AtomicBool>,
}

impl KeyboardBridge {
    pub fn new(start_enabled: bool) -> Self {
        let (tok_tx, tok_rx) = mpsc::channel::<KeyAction>(256);
        let enabled = Arc::new(AtomicBool::new(start_enabled));
        let enabled_clone = enabled.clone();

        let (std_tx, std_rx) = std::sync::mpsc::channel::<rdev::Event>();

        std::thread::spawn(move || {
            rdev::listen(move |event| {
                let _ = std_tx.send(event);
            })
            .expect("Failed to start keyboard listener");
        });

        tokio::task::spawn_blocking(move || {
            let mut held_keys: HashSet<Key> = HashSet::new();
            let mut key_buffer: Vec<String> = Vec::new();
            let mut last_flush = Instant::now();
            let flush_ms = 500;

            while let Ok(event) = std_rx.recv() {
                let (key, pressed) = match event.event_type {
                    EventType::KeyPress(k) => (k, true),
                    EventType::KeyRelease(k) => (k, false),
                    _ => continue,
                };

                if pressed {
                    held_keys.insert(key);
                    key_buffer.push(format!("{:?}", key));
                } else {
                    held_keys.remove(&key);
                }

                if last_flush.elapsed().as_millis() >= flush_ms {
                    if !key_buffer.is_empty() {
                        info!("Captured keys: {}", compress_keys(&key_buffer));
                        key_buffer.clear();
                    }
                    last_flush = Instant::now();
                }

                if pressed && key == Key::F12 && has_ctrl_shift(&held_keys) {
                    let was = enabled_clone.fetch_xor(true, Ordering::SeqCst);
                    let now = !was;
                    info!("Keyboard sync {}", if now { "ENABLED" } else { "DISABLED" });
                    continue;
                }

                if !enabled_clone.load(Ordering::Relaxed) {
                    continue;
                }

                let action = KeyAction { key, pressed };
                if tok_tx.blocking_send(action).is_err() {
                    break;
                }
            }
        });

        Self {
            rx: tok_rx,
            enabled,
        }
    }

    pub async fn recv_key_event(&mut self) -> Option<KeyAction> {
        self.rx.recv().await
    }

    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
}

pub fn simulate_key(action: &KeyAction) {
    let event_type = if action.pressed {
        EventType::KeyPress(action.key)
    } else {
        EventType::KeyRelease(action.key)
    };
    if let Err(e) = rdev::simulate(&event_type) {
        tracing::error!("Failed to simulate key: {:?}", e);
    }
    std::thread::sleep(std::time::Duration::from_millis(20));
}

fn has_ctrl_shift(held: &HashSet<Key>) -> bool {
    (held.contains(&Key::ControlLeft) || held.contains(&Key::ControlRight))
        && (held.contains(&Key::ShiftLeft) || held.contains(&Key::ShiftRight))
}

fn compress_keys(keys: &[String]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < keys.len() {
        let mut count = 1;
        while i + count < keys.len() && keys[i + count] == keys[i] {
            count += 1;
        }
        if count > 1 {
            parts.push(format!("{} x{}", keys[i], count));
        } else {
            parts.push(keys[i].clone());
        }
        i += count;
    }
    parts.join(" ")
}
