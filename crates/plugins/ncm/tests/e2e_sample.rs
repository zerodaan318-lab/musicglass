//! End-to-end verification against a REAL `.ncm` sample (task book §8: 已验证的样本).
//!
//! Run with: cargo test -p musicglass-plugins-ncm --test e2e_sample -- --nocapture
//! It decrypts the real file, writes the inner audio + cover to disk, and
//! prints key facts so we can confirm the decrypt is correct (not garbage).

use musicglass_plugins_ncm::metadata::{detect_by_magic, parse_metadata};
use musicglass_plugins_ncm::NcmPlugin;
use musicglass_core::MusicContainer;
use std::path::Path;

const SAMPLE: &str = "XXXTENTACION - Everybody Dies In Their Nightmares.ncm";

#[test]
fn e2e_real_sample_decrypt() {
    // cargo sets CARGO_MANIFEST_DIR to the crate root (crates/plugins/ncm).
    // The sample lives in the workspace root, two levels up.
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let path = std::path::Path::new(&manifest)
        .join("..")
        .join("..")
        .join("..")
        .join(SAMPLE);
    // This test needs a real .ncm sample that is git-ignored (*.ncm) and NOT
    // committed. If absent, skip so CI without the sample doesn't fail.
    if !path.exists() {
        println!("SKIP: sample .ncm not present at {:?} (git-ignored, provide locally to run)", path);
        return;
    }

    // 1. can_handle by magic
    assert!(NcmPlugin::can_handle(&path), "can_handle should be true for .ncm");

    // 2. extract audio
    let audio = NcmPlugin::extract_audio(&path).expect("extract_audio");
    println!("[audio] codec={} bytes={}", audio.codec, audio.data.len());
    assert!(!audio.data.is_empty(), "audio bytes must not be empty");

    // 3. sniff inner format by magic
    let inner = detect_by_magic(&audio.data);
    println!("[audio] detected inner format = {inner}");
    assert!(
        matches!(inner.as_str(), "flac" | "mp3" | "m4a" | "wav" | "ogg"),
        "unexpected inner format: {inner}"
    );

    // 4. write audio to disk for ffprobe verification
    let audio_out = format!("target/ncm_e2e_audio.{inner}");
    std::fs::create_dir_all("target").ok();
    std::fs::write(&audio_out, &audio.data).expect("write audio");
    println!("[audio] written to {audio_out}");

    // 5. extract cover
    let cover = NcmPlugin::extract_cover(&path).expect("extract_cover");
    if let Some(c) = &cover {
        println!("[cover] {} bytes, magic={:?}", c.len(), &c[..4.min(c.len())]);
        let is_jpeg = c.len() >= 3 && &c[0..3] == b"\xFF\xD8\xFF";
        let is_png = c.len() >= 4 && &c[0..4] == b"\x89PNG";
        assert!(is_jpeg || is_png, "cover should be JPEG or PNG");
        let ext = if is_jpeg { "jpg" } else { "png" };
        std::fs::write(format!("target/ncm_e2e_cover.{ext}"), c).expect("write cover");
        println!("[cover] written to target/ncm_e2e_cover.{ext}");
    } else {
        println!("[cover] none");
    }

    // 6. extract metadata
    let meta = NcmPlugin::extract_metadata(&path).expect("extract_metadata");
    println!(
        "[meta] title={:?} artist={:?} album={:?}",
        meta.title, meta.artist, meta.album
    );
    assert!(meta.title.is_some() || meta.artist.is_some(), "metadata should have title or artist");

    println!("[OK] real sample decrypted successfully");
}
