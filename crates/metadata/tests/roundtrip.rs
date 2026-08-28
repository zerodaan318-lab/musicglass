//! Integration test: metadata round-trip via the bundled FFmpeg + lofty.
//!
//! Generates a tiny silent FLAC with ffmpeg, writes Unified Metadata through
//! `musicglass_metadata::write_metadata`, then reads it back and asserts the
//! fields survived. Requires the bundled ffmpeg (see audio::build.rs).

use musicglass_core::{Format, Metadata};
use musicglass_metadata::{read_metadata, write_metadata};
use std::process::Command;

fn ffmpeg() -> std::path::PathBuf {
    // mirror audio::ffmpeg_path resolution: env, then next to exe, then PATH
    if let Ok(p) = std::env::var("MUSICGLASS_FFMPEG") {
        return std::path::PathBuf::from(p);
    }
    if let Some(exe) = std::env::current_exe().ok() {
        // cargo test binaries live in target/debug/deps/, ffmpeg is copied to
        // target/debug/resources/ by audio/build.rs
        let candidates = [
            exe.parent().map(|p| p.join("resources/ffmpeg/bin/ffmpeg.exe")),
            exe.parent().and_then(|p| p.parent()).map(|p| p.join("resources/ffmpeg/bin/ffmpeg.exe")),
        ];
        for c in candidates.into_iter().flatten() {
            if c.exists() {
                return c;
            }
        }
    }
    std::path::PathBuf::from("ffmpeg")
}

fn make_silent_flac(path: &std::path::Path) {
    let ff = ffmpeg();
    let status = Command::new(&ff)
        .args(["-y", "-f", "lavfi", "-i", "anullsrc=r=44100:cl=stereo", "-t", "1", "-c:a", "flac", path.to_str().unwrap()])
        .status()
        .expect("ffmpeg spawn");
    assert!(status.success(), "ffmpeg failed to generate test flac");
}

#[test]
fn metadata_roundtrip_flac() {
    let tmp = std::env::temp_dir().join("mg_test_meta.flac");
    make_silent_flac(&tmp);

    let mut meta = Metadata::default();
    meta.title = Some("夜曲".into());
    meta.artist = Some("周杰伦".into());
    meta.album = Some("十一月的萧邦".into());
    meta.album_artist = Some("周杰伦".into());
    meta.genre = Some("Pop".into());
    meta.track_number = Some(1);
    meta.track_total = Some(12);
    meta.disc_number = Some(1);
    meta.year = Some("2005".into());
    meta.lyrics = Some("lyrics here".into());

    write_metadata(&tmp, &meta, Format::Flac).expect("write_metadata");

    let read = read_metadata(&tmp).expect("read_metadata");
    assert_eq!(read.title.as_deref(), Some("夜曲"));
    assert_eq!(read.artist.as_deref(), Some("周杰伦"));
    assert_eq!(read.album.as_deref(), Some("十一月的萧邦"));
    assert_eq!(read.album_artist.as_deref(), Some("周杰伦"));
    assert_eq!(read.genre.as_deref(), Some("Pop"));
    assert_eq!(read.track_number, Some(1));
    assert_eq!(read.track_total, Some(12));
    assert_eq!(read.disc_number, Some(1));
    assert_eq!(read.year.as_deref(), Some("2005"));
    assert_eq!(read.lyrics.as_deref(), Some("lyrics here"));

    let _ = std::fs::remove_file(&tmp);
}
