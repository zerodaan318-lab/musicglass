//! Unit tests for NCM metadata mapping and inner-format detection.
//! These run without needing a real encrypted sample.

use musicglass_plugins_ncm::metadata::{detect_by_magic, parse_metadata};

#[test]
fn detect_flac_by_magic() {
    let audio = b"fLaC\x00\x00\x00\x00some data here";
    assert_eq!(detect_by_magic(audio), "flac");
}

#[test]
fn detect_mp3_by_sync() {
    // MP3 frame sync 0xFFFB
    let audio = [0xFF, 0xFB, 0x90, 0x00, 0x01, 0x02];
    assert_eq!(detect_by_magic(&audio), "mp3");
}

#[test]
fn detect_m4a_by_ftyp() {
    let mut audio = vec![0u8; 4];
    audio.extend_from_slice(b"ftyp");
    audio.extend_from_slice(b"mp42");
    assert_eq!(detect_by_magic(&audio), "m4a");
}

#[test]
fn detect_wav_by_riff() {
    let audio = b"RIFF\x00\x00\x00\x00WAVE";
    assert_eq!(detect_by_magic(audio), "wav");
}

#[test]
fn parse_metadata_maps_fields() {
    let json = br#"{
        "musicName": "Test Song",
        "artist": [["Artist A", 1001], ["Artist B", 1002]],
        "album": "Test Album",
        "albumId": 5001,
        "albumPic": "http://example.com/cover.jpg",
        "bitrate": 320000,
        "duration": 210000,
        "format": "flac"
    }"#;
    let m = parse_metadata(json).expect("parse ok");
    assert_eq!(m.title.as_deref(), Some("Test Song"));
    assert_eq!(m.artist.as_deref(), Some("Artist A / Artist B"));
    assert_eq!(m.album.as_deref(), Some("Test Album"));
    assert_eq!(m.extra.get("album_id").map(|s| s.as_str()), Some("5001"));
    assert_eq!(m.extra.get("cover_url").map(|s| s.as_str()), Some("http://example.com/cover.jpg"));
    assert_eq!(m.extra.get("bitrate").map(|s| s.as_str()), Some("320 kbps"));
    assert_eq!(m.extra.get("inner_format").map(|s| s.as_str()), Some("flac"));
}

#[test]
fn parse_metadata_single_artist() {
    let json = br#"{"musicName":"Solo","artist":[["Only One", 7]],"album":"A"}"#;
    let m = parse_metadata(json).expect("parse ok");
    assert_eq!(m.artist.as_deref(), Some("Only One"));
}
