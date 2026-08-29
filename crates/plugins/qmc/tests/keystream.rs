//! Unit tests for QMC keystream + inner-format detection.
//! The keystream sequence is cross-checked against presburger/qmc-decoder's
//! seed.hpp logic (manually traced for the first 10 outputs).

use musicglass_plugins_qmc::QmcSeed;
use musicglass_plugins_qmc::metadata::{detect_by_magic, format_from_ext};
use std::path::Path;

#[test]
fn keystream_first_bytes_match_reference() {
    let mut s = QmcSeed::new();
    let expected = [0xc3, 0x4a, 0xd6, 0xca, 0x90, 0x67, 0xf7, 0x52, 0xd8, 0xa1];
    for &e in expected.iter() {
        assert_eq!(s.next_mask(), e, "keystream mismatch at expected {e:#x}");
    }
}

#[test]
fn detect_flac_magic() {
    assert_eq!(detect_by_magic(b"fLaCxxxx"), "flac");
}

#[test]
fn detect_mp3_sync() {
    assert_eq!(detect_by_magic(&[0xFF, 0xFB, 0x00, 0x00]), "mp3");
}

#[test]
fn detect_ogg_magic() {
    assert_eq!(detect_by_magic(b"OggSxxxx"), "ogg");
}

#[test]
fn format_from_extension() {
    assert_eq!(format_from_ext(Path::new("a.qmcflac")), "flac");
    assert_eq!(format_from_ext(Path::new("a.mgg")), "ogg");
    assert_eq!(format_from_ext(Path::new("a.qmc0")), "mp3");
    assert_eq!(format_from_ext(Path::new("a.qmc3")), "mp3");
}
