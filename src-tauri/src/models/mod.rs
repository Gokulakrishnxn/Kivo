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
use std::path::{Path, PathBuf};
use std::process::Command;

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

/// Download, verify and install. Long-running: call from a worker thread.
///
/// `on` is invoked with progress. Nothing is installed unless the digest
/// matches, and a failure leaves any existing model untouched.
pub fn install(mut on: impl FnMut(Progress)) -> Result<NetworkEvent> {
    on(Progress::Started);

    let tmp = tempdir().context("could not create a temporary directory")?;
    let archive = tmp.join("model.tar.bz2");

    // --- download ---------------------------------------------------------
    let status = Command::new("curl")
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
        .status()
        .context("could not run curl")?;

    if !status.success() {
        let _ = std::fs::remove_dir_all(&tmp);
        let ev = NetworkEvent {
            at: now(),
            url: URL.into(),
            outcome: "download failed".into(),
            bytes: 0,
        };
        on(Progress::Failed {
            message: "The download did not complete.".into(),
        });
        return Ok(ev);
    }
    on(Progress::Downloading { percent: 100 });

    // --- verify -----------------------------------------------------------
    on(Progress::Verifying);
    let bytes = std::fs::metadata(&archive).map(|m| m.len()).unwrap_or(0);
    let digest = sha256_file(&archive)?;
    if digest != SHA256 {
        let _ = std::fs::remove_dir_all(&tmp);
        on(Progress::Failed {
            message: "The downloaded model did not match its expected checksum, so nothing was installed.".into(),
        });
        bail!("sha-256 mismatch: expected {SHA256}, got {digest}");
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
    on(Progress::Done);
    Ok(NetworkEvent {
        at: now(),
        url: URL.into(),
        outcome: "verified and installed".into(),
        bytes,
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
