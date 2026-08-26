//! Transcribe a wav file through the real pipeline and report the timings.
//!
//! This is the harness PRD 9.5 calls for: it measures recognition and cleanup
//! on actual audio rather than trusting a leaderboard number measured on read
//! speech with a datacentre GPU.
//!
//!   cargo run --release --example transcribe -- path/to/audio.wav

use std::path::PathBuf;
use std::time::Instant;

use kivo_lib::asr::{parakeet::Parakeet, Recogniser};
use kivo_lib::audio;
use kivo_lib::clean::{self, CleanConfig};

fn main() -> anyhow::Result<()> {
    let path: PathBuf = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("usage: transcribe <file.wav>"))?;

    let dir = Parakeet::default_dir();
    if !Parakeet::is_installed(&dir) {
        anyhow::bail!(
            "no model at {} — run ./scripts/fetch-model.sh",
            dir.display()
        );
    }

    // ---- read the wav -------------------------------------------------------
    let mut reader = hound::WavReader::open(&path)?;
    let spec = reader.spec();
    let raw: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().filter_map(Result::ok).collect(),
        hound::SampleFormat::Int => {
            let max = (1i64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .filter_map(Result::ok)
                .map(|s| s as f32 / max)
                .collect()
        }
    };
    let seconds = raw.len() as f32 / spec.channels as f32 / spec.sample_rate as f32;
    println!(
        "audio     {} · {:.2}s · {} Hz · {}ch",
        path.display(),
        seconds,
        spec.sample_rate,
        spec.channels
    );

    // ---- stage 2: segment ---------------------------------------------------
    let t = Instant::now();
    let mono = audio::to_mono(&raw, spec.channels);
    let resampled = audio::resample(&mono, spec.sample_rate, audio::TARGET_RATE);
    let speech = audio::trim_silence(&resampled, audio::TARGET_RATE);
    let segment_ms = t.elapsed().as_secs_f64() * 1000.0;

    // ---- stage 3: recognise -------------------------------------------------
    let t = Instant::now();
    let mut model = Parakeet::load(&dir, 4)?;
    let load_ms = t.elapsed().as_secs_f64() * 1000.0;
    println!(
        "model     {} · loaded in {:.0} ms",
        model.describe(),
        load_ms
    );

    // The first inference on CoreML includes one-time graph compilation, so a
    // single cold run measures the wrong thing. Warm up, then take the median
    // of several — that is what a user actually experiences from the second
    // dictation onward.
    let cold = Instant::now();
    let text = model.transcribe(&speech, audio::TARGET_RATE)?;
    let cold_ms = cold.elapsed().as_secs_f64() * 1000.0;

    let runs: usize = std::env::var("KIVO_RUNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);
    let mut times: Vec<f64> = Vec::with_capacity(runs);
    for _ in 0..runs {
        let t = Instant::now();
        let _ = model.transcribe(&speech, audio::TARGET_RATE)?;
        times.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let asr_ms = times[times.len() / 2];
    println!("recognise cold {cold_ms:.0} ms · warm median {asr_ms:.0} ms over {runs} runs");

    // ---- stage 4: clean -----------------------------------------------------
    let t = Instant::now();
    let cleaned = clean::clean(&text, &CleanConfig::default());
    let clean_ms = t.elapsed().as_secs_f64() * 1000.0;

    println!();
    println!("raw       {text}");
    println!("cleaned   {cleaned}");
    println!();
    println!("segment   {segment_ms:7.1} ms");
    println!(
        "recognise {asr_ms:7.1} ms   ({:.1}x real time, warm)",
        seconds as f64 * 1000.0 / asr_ms
    );
    println!("clean     {clean_ms:7.2} ms");
    println!("─────────────────────");
    println!(
        "pipeline  {:7.1} ms   (budget: 300 ms, excluding injection)",
        segment_ms + asr_ms + clean_ms
    );
    Ok(())
}
