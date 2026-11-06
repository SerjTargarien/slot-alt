use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use slot_input::{Btn, RawEvent};

pub const PRESSED_BELOW: u32 = 300;

pub const RELEASED_ABOVE: u32 = 800;

const POLL: Duration = Duration::from_millis(16);

pub struct AdcKey {
    btn: Btn,
    held: bool,
}

impl AdcKey {
    pub fn new(btn: Btn) -> Self {
        AdcKey { btn, held: false }
    }

    pub fn feed(&mut self, raw: u32) -> Option<RawEvent> {
        match (self.held, raw) {
            (false, r) if r < PRESSED_BELOW => {
                self.held = true;
                Some(RawEvent::Down(self.btn))
            }
            (true, r) if r > RELEASED_ABOVE => {
                self.held = false;
                Some(RawEvent::Up(self.btn))
            }
            _ => None,
        }
    }

    pub fn lost(&mut self) -> Option<RawEvent> {
        std::mem::replace(&mut self.held, false).then_some(RawEvent::Up(self.btn))
    }
}

pub fn parse(text: &str) -> Option<u32> {
    text.trim().parse().ok()
}

pub fn spawn(path: PathBuf, btn: Btn, queue: Arc<Mutex<Vec<RawEvent>>>) {
    let spawned = std::thread::Builder::new()
        .name("slot-input-adc".into())
        .spawn(move || {
            let mut key = AdcKey::new(btn);
            loop {
                let event = match std::fs::read_to_string(&path)
                    .ok()
                    .as_deref()
                    .and_then(parse)
                {
                    Some(raw) => key.feed(raw),
                    None => key.lost(),
                };
                if let Some(event) = event {
                    if matches!(event, RawEvent::Down(_)) {
                        crate::latency::read();
                    }
                    queue.lock().unwrap_or_else(|e| e.into_inner()).push(event);
                }
                std::thread::sleep(POLL);
            }
        });
    if let Err(e) = spawned {
        eprintln!("slot: input adc {btn:?}: {e}");
    }
}
