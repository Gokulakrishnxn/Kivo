//! Stage 3 — recognition. Audio in, text out. Nothing else.
//!
//! Everything above and below this module knows only the `Recogniser` trait.
//! That boundary was drawn before the model was chosen, so if Parakeet misses
//! its latency budget on low-end hardware the replacement is a contained change
//! rather than a rewrite.

pub mod parakeet;

use anyhow::Result;

pub trait Recogniser: Send {
    /// Transcribe 16 kHz mono f32 samples.
    fn transcribe(&mut self, samples: &[f32], sample_rate: u32) -> Result<String>;
    /// What is actually running, for the About panel.
    fn describe(&self) -> String;
}

/// Stands in when no model is installed yet, so the rest of the pipeline —
/// hotkey, capture, cleanup, injection — can be exercised end to end.
/// It never pretends to have heard anything.
pub struct Unavailable;

impl Recogniser for Unavailable {
    fn transcribe(&mut self, _: &[f32], _: u32) -> Result<String> {
        anyhow::bail!("No speech model installed. Open Settings to download Parakeet.")
    }
    fn describe(&self) -> String {
        "none installed".into()
    }
}
