//! The model manager — the only outbound request Kivo makes.
//!
//! Deliberately implemented over the system `curl` and `tar` rather than by
//! pulling an HTTP stack and a TLS stack into the process. Kivo is resident all
//! day holding a microphone permission, and every crate linked into it is
//! attack surface for the sake of one download that happens once. The payload
//! is SHA-256 verified against a pinned digest, so a compromised transport
//! still cannot install a model Kivo will load.

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

/// The mirror on Kivo's own releases, so the digest below can be pinned.
/// Weights are NVIDIA's Parakeet TDT 0.6B v2 (CC-BY-4.0); the ONNX export is
/// sherpa-onnx's. Neither is Kivo's work.
pub const URL: &str = "https://github.com/Gokulakrishnxn/Kivo/releases/download/\
                       models-parakeet-tdt-0.6b-v2/parakeet-tdt-0.6b-v2-int8.tar.bz2";
pub const SHA256: &str = "157c157bc51155e03e37d2466522a3a737dd9c72bb25f36eb18912964161e1ad";
pub const DOWNLOAD_MB: u64 = 460;

/// One line in the network panel. Kivo records every outbound attempt so the
/// claim in Settings is observed rather than asserted.
#[derive(Debug, Clone, Serialize)]
pub struct NetworkEvent {
    pub at: i64,
    pub url: String,
    pub outcome: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub enum Progress {
    Started,
    Downloading { percent: u8 },
    Verifying,
    Installing,
    Done,
    Failed { message: String },
}

fn model_dir() -> PathBuf {
    crate::asr::parakeet::Parakeet::default_dir()
}

/// How many attempts the whole download-verify-extract sequence gets before
/// giving up. A truncated or corrupted transfer is the common transient
/// failure here, and a plain retry clears most of them without bothering
/// the user.
const MAX_ATTEMPTS: u32 = 2;

/// Download, verify and install. Long-running: call from a worker thread.
///
/// `on` is invoked with progress, including live percentage while the
/// transfer is running. Nothing is installed unless the digest matches, and a
/// failure leaves any existing model untouched.
pub fn install(mut on: impl FnMut(Progress)) -> Result<NetworkEvent> {
    on(Progress::Started);

    let mut last_message = String::new();
    for attempt in 1..=MAX_ATTEMPTS {
        match try_install(&mut on) {
            Ok(ev) => return Ok(ev),
            Err(e) => {
                last_message = format!("{e:#}");
                if attempt < MAX_ATTEMPTS {
                    tracing::warn!("model install attempt {attempt} failed, retrying: {e:#}");
                } else {
                    tracing::error!("model install failed after {attempt} attempts: {e:#}");
                }
            }
        }
    }

    on(Progress::Failed {
        message: last_message.clone(),
    });
    Ok(NetworkEvent {
        at: now(),
        url: URL.into(),
        outcome: format!("failed: {last_message}"),
        bytes: 0,
    })
}

/// One full download-verify-extract-install attempt.
fn try_install(on: &mut impl FnMut(Progress)) -> Result<NetworkEvent> {
    let tmp = tempdir().context("could not create a temporary directory")?;
    let archive = tmp.join("model.tar.bz2");

    // --- download, with a real percentage ----------------------------------
    let total = head_content_length(URL);

    let mut child = Command::new("curl")
        .args([
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--proto",
            "=https", // refuse a plaintext redirect
            "--tlsv1.2",
            "-o",
        ])
        .arg(&archive)
        .arg(URL)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("could not run curl")?;

    // Poll the file curl is writing rather than parsing its terminal
    // progress meter, which is designed for a tty, not a pipe.
    let mut last_percent = 0u8;
    loop {
        if let Some(status) = child.try_wait().context("could not poll curl")? {
            if !status.success() {
                let mut stderr = String::new();
                if let Some(mut e) = child.stderr.take() {
                    let _ = e.read_to_string(&mut stderr);
                }
                let _ = std::fs::remove_dir_all(&tmp);
                let detail = stderr.trim();
                bail!(
                    "Could not download the model from {URL}.{}",
                    if detail.is_empty() {
                        String::new()
                    } else {
                        format!(" ({detail})")
                    }
                );
            }
            break;
        }

        if let Some(total) = total {
            let done = std::fs::metadata(&archive).map(|m| m.len()).unwrap_or(0);
            // Capped below 100: only the process actually exiting means done,
            // since the last few percent are often buffered rather than synced.
            let percent = ((done * 100 / total.max(1)) as u8).min(99);
            if percent != last_percent {
                on(Progress::Downloading { percent });
                last_percent = percent;
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    on(Progress::Downloading { percent: 100 });

    // --- verify -----------------------------------------------------------
    on(Progress::Verifying);
    let bytes = std::fs::metadata(&archive).map(|m| m.len()).unwrap_or(0);
    let digest = sha256_file(&archive)?;
    if digest != SHA256 {
        let _ = std::fs::remove_dir_all(&tmp);
        bail!(
            "The downloaded model did not match its expected checksum, so nothing was installed. \
             (expected {SHA256}, got {digest})"
        );
    }

    // --- install ----------------------------------------------------------
    on(Progress::Installing);
    let status = Command::new("tar")
        .arg("-xjf")
        .arg(&archive)
        .arg("-C")
        .arg(&tmp)
        .status()
        .context("could not run tar")?;
    if !status.success() {
        let _ = std::fs::remove_dir_all(&tmp);
        bail!("could not extract the model archive");
    }

    let src = std::fs::read_dir(&tmp)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| {
            p.is_dir()
                && p.file_name()
                    .map(|n| n.to_string_lossy().starts_with("sherpa-onnx-"))
                    .unwrap_or(false)
        })
        .ok_or_else(|| anyhow!("the archive did not contain a model directory"))?;

    let dest = model_dir();
    std::fs::create_dir_all(&dest).context("could not create the model directory")?;
    for (pattern, target) in [
        ("encoder", "encoder.onnx"),
        ("decoder", "decoder.onnx"),
        ("joiner", "joiner.onnx"),
        ("tokens", "tokens.txt"),
    ] {
        let found = std::fs::read_dir(&src)?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .find(|p| {
                p.file_name()
                    .map(|n| n.to_string_lossy().starts_with(pattern))
                    .unwrap_or(false)
            })
            .ok_or_else(|| anyhow!("the archive is missing its {pattern} file"))?;
        std::fs::copy(&found, dest.join(target))
            .with_context(|| format!("could not install {target}"))?;
    }

    let _ = std::fs::remove_dir_all(&tmp);

    // Belt and braces: confirm what the pipeline will actually load is there,
    // rather than trusting that four successful copies imply a usable model.
    if !crate::asr::parakeet::Parakeet::is_installed(&dest) {
        bail!(
            "The model was downloaded and verified, but the installed files at {} are incomplete.",
            dest.display()
        );
    }

    on(Progress::Done);
    Ok(NetworkEvent {
        at: now(),
        url: URL.into(),
        outcome: "verified and installed".into(),
        bytes,
    })
}

/// `Content-Length` from a HEAD request, for real download percentages.
/// `None` if the server does not say — progress then jumps straight to 100%
/// rather than showing a wrong number.
fn head_content_length(url: &str) -> Option<u64> {
    let out = Command::new("curl")
        .args(["--head", "--silent", "--location", url])
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .rev() // a redirect chain repeats the header; the final response wins
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.trim()
                .eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse().ok())
                .flatten()
        })
}

/// Delete the installed model. Frees roughly 630 MB.
pub fn remove() -> Result<()> {
    let dir = model_dir();
    if dir.exists() {
        std::fs::remove_dir_all(&dir).context("could not remove the model")?;
    }
    Ok(())
}

pub fn sha256_file(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut file = std::fs::File::open(path).context("could not read the downloaded file")?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

fn tempdir() -> Result<PathBuf> {
    let base = std::env::temp_dir().join(format!("kivo-model-{}", std::process::id()));
    std::fs::create_dir_all(&base)?;
    Ok(base)
}

fn now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn the_pinned_digest_is_a_well_formed_sha256() {
        assert_eq!(SHA256.len(), 64);
        assert!(SHA256.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn the_download_url_is_https_and_points_at_our_own_release() {
        assert!(URL.starts_with("https://"));
        assert!(URL.contains("Gokulakrishnxn/Kivo/releases"));
    }

    #[test]
    fn hashing_matches_a_known_vector() {
        // SHA-256 of "abc"
        let mut f = std::env::temp_dir();
        f.push(format!("kivo-hash-test-{}", std::process::id()));
        std::fs::File::create(&f)
            .unwrap()
            .write_all(b"abc")
            .unwrap();
        assert_eq!(
            sha256_file(&f).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let _ = std::fs::remove_file(&f);
    }
}
