//! The pipeline, and the state the whole app hangs off.
//!
//! From the user's point of view this is synchronous: you release the key, you
//! get text. There is no queue and no background job, so every stage has a
//! latency budget it must not exceed (PRD 9.5).

use anyhow::Result;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

use crate::asr::{parakeet::Parakeet, Recogniser, Unavailable};
use crate::audio::{self, Capture};
use crate::clean;
use crate::config::Config;
use crate::inject;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Idle,
    Recording,
    Working,
}

#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    pub raw: String,
    pub text: String,
    pub mode: String,
    pub millis: u64,
    pub at: String,
}

pub struct Kivo {
    pub config: Mutex<Config>,
    recogniser: Mutex<Box<dyn Recogniser>>,
    capture: Mutex<Option<Capture>>,
    pub state: Mutex<State>,
    pub history: Mutex<Vec<Entry>>,
    /// Non-fatal problems worth showing in the interface.
    pub last_error: Mutex<Option<String>>,
}

impl Kivo {
    pub fn new() -> Arc<Self> {
        let config = Config::load();
        let dir = Parakeet::default_dir();

        // A missing model must not stop the app from starting; the user needs
        // the interface in order to fix it.
        let recogniser: Box<dyn Recogniser> = if Parakeet::is_installed(&dir) {
            match Parakeet::load(&dir, num_threads()) {
                Ok(p) => Box::new(p),
                Err(e) => {
                    tracing::error!("could not load Parakeet: {e:#}");
                    Box::new(Unavailable)
                }
            }
        } else {
            Box::new(Unavailable)
        };

        Arc::new(Self {
            config: Mutex::new(config),
            recogniser: Mutex::new(recogniser),
            capture: Mutex::new(None),
            state: Mutex::new(State::Idle),
            history: Mutex::new(Vec::new()),
            last_error: Mutex::new(None),
        })
    }

    pub fn describe_model(&self) -> String {
        self.recogniser.lock().describe()
    }

    /// Try to (re)load the model — used after a download completes.
    pub fn reload_model(&self) -> Result<String> {
        let dir = Parakeet::default_dir();
        let p = Parakeet::load(&dir, num_threads())?;
        let desc = p.describe();
        *self.recogniser.lock() = Box::new(p);
        Ok(desc)
    }

    /// Key down: open the microphone.
    pub fn begin(&self) {
        if *self.state.lock() != State::Idle {
            return;
        }
        match Capture::start() {
            Ok(c) => {
                *self.capture.lock() = Some(c);
                *self.state.lock() = State::Recording;
            }
            Err(e) => {
                tracing::error!("could not open the microphone: {e:#}");
                *self.last_error.lock() = Some(format!("Kivo could not open the microphone. {e}"));
            }
        }
    }

    /// Key up: close the microphone and run the rest of the pipeline.
    pub fn finish(&self) -> Option<Entry> {
        let Some(capture) = self.capture.lock().take() else {
            *self.state.lock() = State::Idle;
            return None;
        };
        *self.state.lock() = State::Working;
        let started = Instant::now();

        let samples = capture.stop();
        let speech = audio::trim_silence(&samples, audio::TARGET_RATE);

        if speech.len() < audio::TARGET_RATE as usize / 10 {
            // Under 100 ms of sound: a tap, not an utterance.
            *self.state.lock() = State::Idle;
            return None;
        }

        let raw = match self
            .recogniser
            .lock()
            .transcribe(&speech, audio::TARGET_RATE)
        {
            Ok(t) => t,
            Err(e) => {
                tracing::error!("recognition failed: {e:#}");
                *self.last_error.lock() = Some(format!("{e}"));
                *self.state.lock() = State::Idle;
                return None;
            }
        };

        if raw.trim().is_empty() {
            *self.state.lock() = State::Idle;
            return None;
        }

        let cfg = self.config.lock().clone();
        let text = clean::clean(&raw, &cfg.cleaning());

        if let Err(e) = inject::insert(&text, cfg.injection()) {
            tracing::error!("insertion failed: {e:#}");
            *self.last_error.lock() = Some(format!(
                "Kivo could not insert text into this app, so it was left on your clipboard. {e}"
            ));
        }

        let entry = Entry {
            raw,
            text,
            mode: format!("{:?}", cfg.mode),
            millis: started.elapsed().as_millis() as u64,
            at: timestamp(),
        };

        if cfg.keep_history {
            let mut h = self.history.lock();
            h.insert(0, entry.clone());
            h.truncate(cfg.history_limit);
        }

        *self.state.lock() = State::Idle;
        Some(entry)
    }

    /// Discard whatever is being recorded without inserting anything.
    pub fn cancel(&self) {
        if let Some(c) = self.capture.lock().take() {
            let _ = c.stop(); // drops the device and zeroes the ring
        }
        *self.state.lock() = State::Idle;
    }
}

fn num_threads() -> i32 {
    std::thread::available_parallelism()
        .map(|n| (n.get() as i32 / 2).clamp(1, 4))
        .unwrap_or(2)
}

fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    secs.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thread_count_is_sane_and_bounded() {
        let n = num_threads();
        assert!((1..=4).contains(&n), "got {n}");
    }

    #[test]
    fn finishing_without_starting_is_harmless() {
        let k = Kivo::new();
        assert!(k.finish().is_none());
        assert_eq!(*k.state.lock(), State::Idle);
    }

    #[test]
    fn cancel_from_idle_is_harmless() {
        let k = Kivo::new();
        k.cancel();
        assert_eq!(*k.state.lock(), State::Idle);
    }
}
