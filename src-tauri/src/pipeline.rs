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
use crate::history::{self, History};
use crate::inject;
use crate::models::NetworkEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Idle,
    Recording,
    Working,
}

pub use history::Entry;

pub struct Kivo {
    pub config: Mutex<Config>,
    recogniser: Mutex<Box<dyn Recogniser>>,
    capture: Mutex<Option<Capture>>,
    pub state: Mutex<State>,
    pub history: Mutex<History>,
    /// Every outbound request Kivo has attempted, so the network panel in
    /// Settings reports what happened rather than asserting what should.
    pub network: Mutex<Vec<NetworkEvent>>,
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
            match Parakeet::load_with_hotwords(&dir, num_threads(), &config.vocabulary) {
                Ok(p) => Box::new(p),
                Err(e) => {
                    tracing::error!("could not load Parakeet: {e:#}");
                    Box::new(Unavailable)
                }
            }
        } else {
            Box::new(Unavailable)
        };

        let history = History::open(config.keep_history);

        Arc::new(Self {
            config: Mutex::new(config),
            recogniser: Mutex::new(recogniser),
            capture: Mutex::new(None),
            state: Mutex::new(State::Idle),
            history: Mutex::new(history),
            network: Mutex::new(Vec::new()),
            last_error: Mutex::new(None),
        })
    }

    /// Apply a new configuration, reopening whatever it changes.
    pub fn apply_config(&self, next: Config) {
        let previous = self.config.lock().clone();
        *self.config.lock() = next.clone();

        if previous.keep_history != next.keep_history {
            *self.history.lock() = History::open(next.keep_history);
            if !next.keep_history {
                // Off means gone, not merely ignored.
                History::destroy();
            }
        }
        if previous.vocabulary != next.vocabulary {
            if let Err(e) = self.reload_model() {
                tracing::warn!("could not reload the model for new vocabulary: {e:#}");
            }
        }
    }

    pub fn describe_model(&self) -> String {
        self.recogniser.lock().describe()
    }

    /// Try to (re)load the model — after a download, or a vocabulary change.
    pub fn reload_model(&self) -> Result<String> {
        let dir = Parakeet::default_dir();
        let vocab = self.config.lock().vocabulary.clone();
        let p = Parakeet::load_with_hotwords(&dir, num_threads(), &vocab)?;
        let desc = p.describe();
        *self.recogniser.lock() = Box::new(p);
        Ok(desc)
    }

    pub fn recent_history(&self, limit: usize, query: &str) -> Vec<Entry> {
        self.history.lock().recent(limit, query)
    }

    /// Key down: open the microphone.
    pub fn begin(&self) {
        if *self.state.lock() != State::Idle {
            return;
        }
        let device = self.config.lock().input_device.clone();
        tracing::info!(
            "begin: opening input device {:?}",
            if device.is_empty() {
                "<system default>"
            } else {
                &device
            }
        );
        match Capture::start(&device) {
            Ok(c) => {
                tracing::info!(
                    "begin: device open — {} @ {} Hz, {} ch",
                    c.device_name,
                    c.device_rate,
                    c.channels
                );
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
        tracing::info!(
            "captured {} samples ({:.1}s at 16 kHz)",
            samples.len(),
            samples.len() as f32 / 16_000.0
        );
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

        tracing::info!(
            "recognised {} chars, cleaned to {} chars",
            raw.chars().count(),
            text.chars().count()
        );
        tracing::debug!("raw: {raw:?}");
        tracing::debug!("clean: {text:?}");

        let millis = started.elapsed().as_millis() as u64;
        let mode = cfg.mode.label().to_string();

        let id = self
            .history
            .lock()
            .insert(&raw, &text, &mode, millis, cfg.history_limit)
            .unwrap_or_else(|e| {
                tracing::error!("could not record history: {e:#}");
                0
            });

        *self.state.lock() = State::Idle;
        Some(Entry {
            id,
            raw,
            text,
            mode,
            millis,
            at: now_secs(),
        })
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

fn now_secs() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
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
