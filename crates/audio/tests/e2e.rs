//! End-to-end test: generate FLAC -> convert to MP3 with metadata -> verify -> history.

use musicglass_audio::verify::verify;
use musicglass_audio::{convert, inspect, ConversionRequest};
use musicglass_core::{Format, Metadata};
use musicglass_metadata::write_metadata;
use musicglass_task_manager::history::History;
use std::path::PathBuf;
use std::process::Command;

fn ffmpeg() -> PathBuf {
    if let Ok(p) = std::env::var("MUSICGLASS_FFMPEG") {
        return PathBuf::from(p);
    }
    if let Some(exe) = std::env::current_exe().ok() {
        let cands = [
            exe.parent().map(|p| p.join("resources/ffmpeg/bin/ffmpeg.exe")),
            exe.parent().and_then(|p| p.parent()).map(|p| p.join("resources/ffmpeg/bin/ffmpeg.exe")),
        ];
        for c in cands.into_iter().flatten() {
            if c.exists() {
                return c;
            }
        }
    }
    PathBuf::from("ffmpeg")
}

#[test]
fn e2e_flac_to_mp3_with_verify_and_history() {
    let tmp = std::env::temp_dir();
    let src = tmp.join("mg_e2e_src.flac");
    let dst = tmp.join("mg_e2e_dst.mp3");

    // 1) generate silent FLAC
    let ff = ffmpeg();
    let st = Command::new(&ff)
        .args(["-y", "-f", "lavfi", "-i", "anullsrc=r=44100:cl=stereo", "-t", "2", "-c:a", "flac", src.to_str().unwrap()])
        .status()
        .expect("ffmpeg");
    assert!(st.success());

    // 2) write metadata into source
    let mut meta = Metadata::default();
    meta.title = Some("Test Song".into());
    meta.artist = Some("Test Artist".into());
    meta.album = Some("Test Album".into());
    musicglass_metadata::write_metadata(&src, &meta, Format::Flac).unwrap();

    // 3) inspect source
    let info = inspect(&src).expect("inspect src");

    // 4) convert to MP3 320k
    let req = ConversionRequest {
        target_format: Format::Mp3,
        bitrate: Some(320_000),
        sample_rate: None,
        bit_depth: None,
        metadata: None,
    };
    convert(&src, &dst, &req).expect("convert");

    // 5) verify output
    let report = verify(&dst, &info, &meta).expect("verify");
    assert!(report.audio_ok, "audio verify failed: {:?}", report.details);
    assert!(report.metadata_ok, "metadata verify failed: {:?}", report.details);

    // 6) record into history
    let db = tmp.join("mg_e2e_history.db");
    let _ = std::fs::remove_file(&db);
    let h = History::open(&db).unwrap();
    h.record(&src, &dst, Format::Flac, Format::Mp3, "Completed", 12345, 6789, Some(1500), None)
        .unwrap();
    let rows = h.query_all().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].status, "Completed");
    assert_eq!(rows[0].input_format, "flac");
    assert_eq!(rows[0].output_format, "mp3");

    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&dst);
    let _ = std::fs::remove_file(&db);
}
