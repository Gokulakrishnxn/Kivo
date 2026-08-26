//! Stage 1-2 — capture and segment.

pub mod ring;

use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::Arc;

pub use ring::Ring;

/// What Parakeet expects. Everything is resampled to this before recognition.
pub const TARGET_RATE: u32 = 16_000;
/// Longer than this is recording, not dictation.
pub const MAX_SECONDS: usize = 30;

pub struct Capture {
    stream: cpal::Stream,
    pub ring: Arc<Ring>,
    pub device_rate: u32,
    pub channels: u16,
    pub device_name: String,
}

/// Every input device the OS is offering, newest listing each call.
pub fn input_devices() -> Vec<String> {
    let host = cpal::default_host();
    host.input_devices()
        .map(|ds| {
            ds.filter_map(|d| d.description().ok().map(|x| x.name().to_string()))
                .collect()
        })
        .unwrap_or_default()
}

impl Capture {
    /// Open an input device and start filling the ring.
    ///
    /// `preferred` names a device; an empty string, or a name that is no longer
    /// present, falls back to the system default. Devices get unplugged, and a
    /// dictation tool that simply stops working when a headset is removed is
    /// worse than one that quietly follows the default.
    ///
    /// The device is opened here and dropped in `stop`, so the OS microphone
    /// indicator reflects reality — a dictation tool that holds the mic open
    /// while idle has lost the privacy argument whatever its source says.
    pub fn start(preferred: &str) -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .input_devices()
            .ok()
            .and_then(|mut ds| {
                if preferred.is_empty() {
                    return None;
                }
                ds.find(|d| {
                    d.description()
                        .map(|x| x.name() == preferred)
                        .unwrap_or(false)
                })
            })
            .or_else(|| host.default_input_device())
            .ok_or_else(|| anyhow!("no input device available"))?;
        let device_name = device
            .description()
            .map(|d| d.name().to_string())
            .unwrap_or_else(|_| "unknown".into());
        let config = device.default_input_config()?;
        let device_rate = config.sample_rate();
        let channels = config.channels();

        let ring = Ring::new(device_rate as usize * channels as usize, MAX_SECONDS);
        let producer = ring.clone();

        let err_fn = |e| tracing::error!("audio stream error: {e}");
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                config.into(),
                move |data: &[f32], _: &_| producer.push(data),
                err_fn,
                None,
            )?,
            cpal::SampleFormat::I16 => device.build_input_stream(
                config.into(),
                move |data: &[i16], _: &_| {
                    // Scratch lives on the stack; the callback still allocates
                    // nothing on the heap.
                    let mut buf = [0.0f32; 2048];
                    for chunk in data.chunks(buf.len()) {
                        for (i, s) in chunk.iter().enumerate() {
                            buf[i] = *s as f32 / i16::MAX as f32;
                        }
                        producer.push(&buf[..chunk.len()]);
                    }
                },
                err_fn,
                None,
            )?,
            other => return Err(anyhow!("unsupported sample format: {other:?}")),
        };

        stream.play()?;
        Ok(Self {
            stream,
            ring,
            device_rate,
            channels,
            device_name,
        })
    }

    /// Stop the stream, take the audio, and zero the buffer behind us.
    pub fn stop(self) -> Vec<f32> {
        drop(self.stream); // closes the device; the mic indicator goes out
        let raw = self.ring.drain();
        let mono = to_mono(&raw, self.channels);
        let out = resample(&mono, self.device_rate, TARGET_RATE);
        self.ring.reset(); // audio is never kept after use
        out
    }
}

/// Average interleaved channels down to one.
pub fn to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    if channels <= 1 {
        return samples.to_vec();
    }
    let n = channels as usize;
    samples
        .chunks(n)
        .map(|frame| frame.iter().sum::<f32>() / frame.len() as f32)
        .collect()
}

/// Linear resample to the target rate.
///
/// Deliberately simple: dictation audio is near-field speech and the band we
/// care about is well under the Nyquist limit of every device rate we see in
/// practice. If measurement shows this costing accuracy, `rubato`'s sinc
/// resampler is a drop-in here.
pub fn resample(samples: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == to || samples.is_empty() {
        return samples.to_vec();
    }
    let ratio = to as f64 / from as f64;
    let out_len = (samples.len() as f64 * ratio).round() as usize;
    let mut out = Vec::with_capacity(out_len);

    for i in 0..out_len {
        let src = i as f64 / ratio;
        let idx = src.floor() as usize;
        let frac = (src - idx as f64) as f32;
        let a = samples.get(idx).copied().unwrap_or(0.0);
        let b = samples.get(idx + 1).copied().unwrap_or(a);
        out.push(a + (b - a) * frac);
    }
    out
}

/// Trim leading and trailing near-silence.
///
/// Users press the key half a second before speaking and release half a second
/// after. Cutting that dead air is close to free and removes frames the
/// expensive model would otherwise process. The pad matters: trimming tight
/// clips plosives and costs accuracy on the first word, which is exactly the
/// word people notice.
pub fn trim_silence(samples: &[f32], rate: u32) -> Vec<f32> {
    const THRESHOLD: f32 = 0.006;
    let pad = (rate as usize) / 10; // 100 ms either side

    let first = samples.iter().position(|s| s.abs() > THRESHOLD);
    let last = samples.iter().rposition(|s| s.abs() > THRESHOLD);

    match (first, last) {
        (Some(f), Some(l)) => {
            let start = f.saturating_sub(pad);
            let end = (l + pad).min(samples.len());
            samples[start..end].to_vec()
        }
        _ => Vec::new(), // nothing above the floor: silence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stereo_averages_to_mono() {
        assert_eq!(to_mono(&[1.0, 0.0, 0.5, 0.5], 2), vec![0.5, 0.5]);
    }

    #[test]
    fn mono_passes_through_untouched() {
        assert_eq!(to_mono(&[1.0, 2.0], 1), vec![1.0, 2.0]);
    }

    #[test]
    fn halving_the_rate_halves_the_sample_count() {
        let input: Vec<f32> = (0..100).map(|i| i as f32).collect();
        assert_eq!(resample(&input, 32_000, 16_000).len(), 50);
    }

    #[test]
    fn resampling_to_the_same_rate_is_a_no_op() {
        let input = vec![1.0, 2.0, 3.0];
        assert_eq!(resample(&input, 16_000, 16_000), input);
    }

    #[test]
    fn trim_keeps_speech_and_pads_it() {
        let mut s = vec![0.0; 16_000];
        s[8_000] = 0.9;
        let out = trim_silence(&s, 16_000);
        // 100 ms either side of a single loud sample.
        assert!(
            out.len() <= 3_300 && out.len() >= 3_100,
            "got {}",
            out.len()
        );
    }

    #[test]
    fn pure_silence_trims_to_nothing() {
        assert!(trim_silence(&vec![0.0; 1000], 16_000).is_empty());
    }
}
