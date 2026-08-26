use std::path::{Path, PathBuf};

/// sherpa-onnx and ONNX Runtime are linked dynamically and referenced as
/// @rpath/..., and sherpa-rs drops them into the *profile* directory
/// (target/debug or target/release). Pointing tauri.conf.json straight at one
/// of those paths breaks every other profile — clippy and `cargo test` build
/// under debug and would fail validation on a release path that does not exist.
///
/// So they are staged into a fixed `frameworks/` directory that the bundle
/// config can name once, regardless of profile.
#[cfg(target_os = "macos")]
fn stage_frameworks() {
    const LIBS: &[&str] = &[
        "libonnxruntime.1.17.1.dylib",
        "libsherpa-onnx-c-api.dylib",
        "libsherpa-onnx-cxx-api.dylib",
    ];

    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let dest = manifest.join("frameworks");

    // OUT_DIR is target/<profile>/build/<crate>-<hash>/out — the profile
    // directory sherpa-rs writes into is three levels up.
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let profile_dir = out.ancestors().nth(3).map(Path::to_path_buf);

    let Some(profile_dir) = profile_dir else {
        return;
    };
    let _ = std::fs::create_dir_all(&dest);

    for lib in LIBS {
        let src = profile_dir.join(lib);
        let dst = dest.join(lib);
        if src.exists() {
            // Copy only when it changed, so we do not thrash rebuilds.
            let stale = std::fs::metadata(&dst)
                .and_then(|m| m.modified())
                .ok()
                .zip(std::fs::metadata(&src).and_then(|m| m.modified()).ok())
                .map(|(d, s)| d < s)
                .unwrap_or(true);
            if stale {
                let _ = std::fs::copy(&src, &dst);
            }
        } else if !dst.exists() {
            // First build of a clean tree: the dylib has not been produced yet.
            // Leave a placeholder so bundle validation has something to point
            // at; the real file lands on the build that follows.
            let _ = std::fs::write(&dst, b"");
            println!(
                "cargo:warning=staged placeholder for {lib}; rebuild to bundle the real library"
            );
        }
    }
    println!("cargo:rerun-if-changed=frameworks");
}

fn main() {
    #[cfg(target_os = "macos")]
    {
        stage_frameworks();
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
    }

    tauri_build::build()
}
