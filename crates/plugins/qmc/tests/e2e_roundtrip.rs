//! Self-contained QMC e2e verification against public reference test vectors.
//!
//! Two kinds of checks:
//! 1. **Algorithm vectors** copied from the public `qmc2-rust` unit tests
//!    (bczhc/qmc-decrypt, MIT/Apache) — prove our RC4/Map ciphers compute the
//!    exact same bytes as the community reference.
//! 2. **Round-trip**: encrypt known audio with the same keystream, wrap it in a
//!    valid QMC container (v1 static / v2 with tail), then decrypt via the real
//!    `QmcPlugin` and assert byte-for-byte equality.
//!
//! This is NOT a Tencent-downloaded song (we don't fetch copyrighted files).
//! The keystream/cipher themselves are unit-verified against the public
//! reverse-engineered reference, so a successful round-trip proves our
//! implementation is internally consistent and complete. A real user-provided
//! `.qmc`/`.mgg` sample is still needed for end-to-end confirmation.

use musicglass_core::MusicContainer;
use musicglass_plugins_qmc::QmcPlugin;
use musicglass_plugins_qmc::{MapCipher, Rc4Cipher};
use std::path::Path;
use base64::Engine;

// ── v2 RC4 reference vectors (from qmc2-rust qmc2_rc4.rs tests) ──
#[test]
fn rc4_first_segment_matches_reference() {
    let mut key = [0u8; 255];
    for (i, p) in key.iter_mut().enumerate() {
        *p = i as u8;
    }
    let c = Rc4Cipher::new(&key);
    let mut data = [0u8; 16];
    c.apply(&mut data, 0);
    assert_eq!(data, [0, 50, 16, 8, 5, 3, 2, 1, 1, 1, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn rc4_second_segment_matches_reference() {
    let mut key = [0u8; 255];
    for (i, p) in key.iter_mut().enumerate() {
        *p = i as u8;
    }
    let c = Rc4Cipher::new(&key);
    let mut data = [0u8; 16];
    c.apply(&mut data, 0x1400); // OTHER_SEGMENT_SIZE
    assert_eq!(
        data,
        [
            151, 56, 198, 1, 226, 173, 127, 4, // beginning of 2nd "other" segment
            181, 165, 171, 21, 82, 152, 195, 210, // next 8 bytes
        ]
    );
}

// ── v2 Map reference vectors (from qmc2-rust qmc2_map.rs tests) ──
#[test]
fn map_l_matches_reference() {
    let key: [u8; 16] = [
        0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F, 0x50,
    ];
    let c = MapCipher::new(key.to_vec());
    let mut data = [0u8; 16];
    c.apply(&mut data, 0);
    assert_eq!(
        data,
        [0x3F, 0x8A, 0xC1, 0x49, 0x3F, 0x49, 0xC1, 0x8A, 0x3F, 0x8A, 0xC1, 0x49, 0x3F, 0x49, 0xC1, 0x8A]
    );
}

// ── v2 (QMC2) round-trip: encrypt known audio with a known ekey, wrap in a
//     valid QTag container, then decrypt via the real plugin. ──

/// Mirror of qmc2-rust `generate_ekey`: produce an ekey string from a raw key.
/// We use this to BUILD a valid QMC2 container for testing (symmetric with
/// parse_ekey). Key is `header(8) || body`.
fn build_ekey(raw_key: &[u8]) -> String {
    use base64::Engine;
    let (header, body) = raw_key.split_at(8);
    // derive_tea_key(header) then TEA-encrypt body
    let simple = [0x69u8, 0x56, 0x46, 0x38, 0x2b, 0x20, 0x15, 0x0b];
    let mut tea_key = [0u8; 16];
    for i in (0..16).step_by(2) {
        tea_key[i] = simple[i / 2];
        tea_key[i + 1] = header[i / 2];
    }
    let encrypted_body = tea_encrypt(body, &tea_key);
    let encoded = [header.to_vec(), encrypted_body].concat();
    base64::engine::general_purpose::STANDARD.encode(encoded)
}

/// TEA encrypt (32 rounds, LE u32) — inverse of decrypt in decrypt.rs.
fn tea_encrypt(block: &[u8], key: &[u8]) -> Vec<u8> {
    let k = |i: usize| u32::from_le_bytes([key[4 * i], key[4 * i + 1], key[4 * i + 2], key[4 * i + 3]]);
    let (k0, k1, k2, k3) = (k(0), k(1), k(2), k(3));
    let delta: u32 = 0x9e3779b9;
    let mut out = Vec::with_capacity(block.len());
    for c in block.chunks_exact(8) {
        let mut v0 = u32::from_le_bytes([c[0], c[1], c[2], c[3]]);
        let mut v1 = u32::from_le_bytes([c[4], c[5], c[6], c[7]]);
        let mut sum: u32 = 0;
        for _ in 0..32 {
            sum = sum.wrapping_add(delta);
            v0 = v0.wrapping_add(
                (v1 << 4)
                    .wrapping_add(k0)
                    .wrapping_mul(v1.wrapping_add(sum))
                    .wrapping_add((v1 >> 5).wrapping_add(k1)),
            );
            v1 = v1.wrapping_add(
                (v0 << 4)
                    .wrapping_add(k2)
                    .wrapping_mul(v0.wrapping_add(sum))
                    .wrapping_add((v0 >> 5).wrapping_add(k3)),
            );
        }
        out.extend_from_slice(&v0.to_le_bytes());
        out.extend_from_slice(&v1.to_le_bytes());
    }
    // PKCS#7-like padding to multiple of 8
    let pad = (8 - out.len() % 8) % 8;
    if pad > 0 {
        out.extend(std::iter::repeat(pad as u8).take(pad));
    }
    out
}

#[test]
fn v2_rc4_roundtrip() {
    // Use a > 300-byte key so the RC4-variant cipher is selected.
    let mut raw_key = vec![0u8; 312];
    for (i, b) in raw_key.iter_mut().enumerate() {
        *b = (i * 7 + 13) as u8;
    }
    let ekey = build_ekey(&raw_key);

    // Known audio.
    let audio = make_audio(300 * 1024); // > 0x1400, exercises segment logic

    // Encrypt with the same RC4 cipher the plugin uses.
    let cipher = Rc4Cipher::new(&raw_key);
    let mut enc = audio.clone();
    cipher.apply(&mut enc, 0);

    // Wrap in a valid QMC2 v2 container:
    //   [enc audio] + ekey + "," + "0" + "," + "2" + "," + meta_size(BE u32) + "QTag"
    let mut container = enc.clone();
    let tail = format!("{},0,2,", ekey);
    let meta = tail.as_bytes();
    container.extend_from_slice(meta);
    let meta_size = meta.len() as u32;
    container.extend_from_slice(&meta_size.to_be_bytes());
    container.extend_from_slice(b"QTag");

    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let path = Path::new(&manifest)
        .join("..")
        .join("..")
        .join("..")
        .join("target")
        .join("qmc_v2_rc4.qmc");
    std::fs::create_dir_all(path.parent().unwrap()).ok();
    std::fs::write(&path, &container).unwrap();

    let out = QmcPlugin::extract_audio(&path).expect("extract v2");
    assert_eq!(out.data, audio, "v2 RC4 decrypt must match original");

    let _ = std::fs::remove_file(&path);
}

#[test]
fn v2_map_roundtrip() {
    // Use a <= 300-byte key so the Map-variant cipher is selected.
    let mut raw_key = vec![0u8; 64];
    for (i, b) in raw_key.iter_mut().enumerate() {
        *b = (i * 5 + 3) as u8;
    }
    let ekey = build_ekey(&raw_key);

    let audio = make_audio(300 * 1024);
    let cipher = MapCipher::new(raw_key);
    let mut enc = audio.clone();
    cipher.apply(&mut enc, 0);

    let mut container = enc.clone();
    let tail = format!("{},0,2,", ekey);
    container.extend_from_slice(tail.as_bytes());
    let meta_size = tail.len() as u32;
    container.extend_from_slice(&meta_size.to_be_bytes());
    container.extend_from_slice(b"QTag");

    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let path = Path::new(&manifest)
        .join("..")
        .join("..")
        .join("..")
        .join("target")
        .join("qmc_v2_map.qmc");
    std::fs::create_dir_all(path.parent().unwrap()).ok();
    std::fs::write(&path, &container).unwrap();

    let out = QmcPlugin::extract_audio(&path).expect("extract v2 map");
    assert_eq!(out.data, audio, "v2 Map decrypt must match original");

    let _ = std::fs::remove_file(&path);
}
fn make_audio(size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];
    data[0..4].copy_from_slice(b"fLaC");
    let mut s: u64 = 0x1234_5678_9abc_def0;
    for i in 4..size {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        data[i] = (s & 0xff) as u8;
    }
    data
}

#[test]
fn v1_static_roundtrip() {
    // Build a v1 static .qmc: whole file XOR'd with the static keystream.
    let audio = make_audio(200 * 1024); // > 0x8000, exercises skip logic
    let mut seed = musicglass_plugins_qmc::QmcSeed::new();
    let mut enc = audio.clone();
    seed.apply(&mut enc);
    assert_ne!(enc, audio);

    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let path = Path::new(&manifest)
        .join("..")
        .join("..")
        .join("..")
        .join("target")
        .join("qmc_v1_rt.qmc");
    std::fs::create_dir_all(path.parent().unwrap()).ok();
    std::fs::write(&path, &enc).unwrap();

    let out = QmcPlugin::extract_audio(&path).expect("extract");
    assert_eq!(out.codec, "flac");
    assert_eq!(out.data, audio, "v1 decrypt must match original");

    let info = QmcPlugin::inspect(&path).expect("inspect");
    assert_eq!(info.inner_codec, "flac");
    let _ = std::fs::remove_file(&path);
}
