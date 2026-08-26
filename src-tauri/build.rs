fn main() {
    // sherpa-onnx and ONNX Runtime are linked dynamically and referenced as
    // @rpath/..., but nothing adds an LC_RPATH — so a bundled build dies at
    // launch with "Library not loaded". Two search paths cover both cases:
    // Contents/Frameworks inside a .app bundle, and the target directory that
    // the dylibs are copied into for `cargo run` and `cargo test`.
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Frameworks");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
    }

    tauri_build::build()
}
