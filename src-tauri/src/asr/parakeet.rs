//! NVIDIA Parakeet TDT, via sherpa-onnx.
//!
//! Parakeet is a FastConformer encoder with a token-and-duration transducer
//! decoder. The decoder predicts how long each token lasts and skips frames
//! rather than stepping through every one, which is the property that makes a
//! sub-300 ms budget achievable rather than aspirational. It also restores
//! punctuation and capitalisation itself, so there is no second model in the
//! path — requirement ASR-3 is satisfied by the model, not by us.

use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};

use super::Recogniser;

/// The four artefacts a sherpa-onnx transducer needs.
const REQUIRED: &[&str] = &["encoder.onnx", "decoder.onnx", "joiner.onnx", "tokens.txt"];

pub struct Parakeet {
    inner: sherpa_rs::transducer::TransducerRecognizer,
    provider: String,
}

impl Parakeet {
    /// Load a model directory. Fails loudly if any artefact is missing rather
    /// than half-initialising.
    ///
    /// `hotwords` biases recognition toward names, acronyms and jargon the user
    /// has told Kivo about. This is a different mechanism from the cleanup
    /// layer's replacements: replacements rewrite text after the fact, whereas
    /// this makes the model more likely to hear the word in the first place.
    pub fn load_with_hotwords(dir: &Path, threads: i32, hotwords: &[String]) -> Result<Self> {
        let file = if hotwords.is_empty() {
            String::new()
        } else {
            match write_hotwords(dir, hotwords) {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!("could not write the hotwords file, continuing without: {e:#}");
                    String::new()
                }
            }
        };
        Self::load_inner(dir, threads, file)
    }

    pub fn load(dir: &Path, threads: i32) -> Result<Self> {
        Self::load_inner(dir, threads, String::new())
    }

    fn load_inner(dir: &Path, threads: i32, hotwords_file: String) -> Result<Self> {
        for f in REQUIRED {
            let p = dir.join(f);
            if !p.exists() {
                return Err(anyhow!("model file missing: {}", p.display()));
            }
        }

        // CoreML puts the encoder on the Neural Engine where it exists; the CPU
        // path is a supported configuration, not a courtesy, because a real
        // share of machines have nothing better.
        let provider = if cfg!(target_os = "macos") {
            "coreml"
        } else {
            "cpu"
        };

        let cfg = sherpa_rs::transducer::TransducerConfig {
            encoder: path(dir, "encoder.onnx"),
            decoder: path(dir, "decoder.onnx"),
            joiner: path(dir, "joiner.onnx"),
            tokens: path(dir, "tokens.txt"),
            num_threads: threads.max(1),
            sample_rate: crate::audio::TARGET_RATE as i32,
            feature_dim: 80,
            // Hotword biasing requires the modified beam search decoder;
            // greedy search ignores it entirely.
            decoding_method: if hotwords_file.is_empty() {
                "greedy_search".into()
            } else {
                "modified_beam_search".into()
            },
            hotwords_file,
            hotwords_score: 1.5,
            model_type: "nemo_transducer".into(),
            provider: Some(provider.into()),
            debug: false,
            ..Default::default()
        };

        // sherpa-rs reports through eyre, so bridge it rather than leaking a
        // second error type into the rest of the crate.
        let inner = sherpa_rs::transducer::TransducerRecognizer::new(cfg)
            .map_err(|e| anyhow!("sherpa-onnx refused the Parakeet model: {e}"))?;

        Ok(Self {
            inner,
            provider: provider.into(),
        })
    }

    /// Where Kivo keeps models. One directory, inspectable, no magic.
    pub fn default_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Kivo")
            .join("models")
            .join("parakeet-tdt-0.6b")
    }

    /// True when every artefact is present.
    pub fn is_installed(dir: &Path) -> bool {
        REQUIRED.iter().all(|f| dir.join(f).exists())
    }
}

fn path(dir: &Path, name: &str) -> String {
    dir.join(name).to_string_lossy().into_owned()
}

/// sherpa-onnx wants hotwords as a file, one phrase per line.
fn write_hotwords(dir: &Path, words: &[String]) -> Result<String> {
    let p = dir.join("hotwords.txt");
    let body: String = words
        .iter()
        .map(|w| w.trim())
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&p, body).context("could not write hotwords")?;
    Ok(p.to_string_lossy().into_owned())
}

impl Recogniser for Parakeet {
    fn transcribe(&mut self, samples: &[f32], sample_rate: u32) -> Result<String> {
        if samples.is_empty() {
            return Ok(String::new());
        }
        Ok(self
            .inner
            .transcribe(sample_rate, samples)
            .trim()
            .to_string())
    }

    fn describe(&self) -> String {
        format!("Parakeet TDT 0.6B · {}", self.provider)
    }
}
