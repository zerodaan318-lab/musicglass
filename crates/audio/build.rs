//! Build script: copy the bundled FFmpeg binaries next to the compiled binary
//! so that `ffmpeg_path()` (which looks in `current_exe()/../resources/ffmpeg/bin`)
//! resolves during `cargo run` / `cargo test`, not just after Tauri packaging.

use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"));
    // OUT_DIR: <workspace>/target/<profile>/build/<crate>-<hash>/out
    // nth(0)=out, nth(1)=<crate>-<hash>, nth(2)=build, nth(3)=<profile>(debug/release)
    let profile_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("cannot locate profile dir from OUT_DIR");

    let dest = profile_dir.join("resources/ffmpeg/bin");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../resources/ffmpeg/bin");

    if src.exists() {
        let _ = fs::create_dir_all(&dest);
        for exe in ["ffmpeg.exe", "ffprobe.exe", "ffplay.exe"] {
            let from = src.join(exe);
            if from.exists() {
                let _ = fs::copy(&from, dest.join(exe));
            }
        }
        println!("cargo:rerun-if-changed=../../resources/ffmpeg/bin");
    }
}
