//! NCM proprietary container decryption (task book §8).
//!
//! Reference open-source implementations that established the format
//! (research recorded in `docs/FORMAT_RESEARCH_NCM.md`):
//! - `anonymous5l/ncmdump` (C++)
//! - `taurusxin/ncmdump` (Go)
//! - `nondanee/ncmdump` (C)
//! - `iqiziqi/ncmdump.rs` (Rust)
//! - `Johnserf-Seed/ncm2mp3` (Rust, Apache-2.0) — provides the canonical
//!   description of the custom RC4 PRGA used here.
//!
//! This module is a clean-room reimplementation: it parses the binary layout
//! and reimplements the decryption primitives from the publicly documented
//! algorithm, but does not copy any upstream source.
//!
//! # Binary layout (in order)
//! ```text
//! Magic 'CTENFDAM' (8B) + Gap (2B)
//! → RC4 key length (4B LE) + RC4 key data  [XOR 0x64
//!     → AES-128-ECB(CORE_KEY) → strip "neteasecloudmusic"]
//! → Metadata length (4B LE) + Metadata data [XOR 0x63
//!     → strip "163 key(Don't modify):" → Base64
//!     → AES-128-ECB(META_KEY) → strip "music:" → JSON]
//! → CRC32 (4B) + Gap (5B)
//! → Cover length (4B LE) + Cover raw bytes (JPEG or PNG)
//! → Audio data (to EOF) [custom RC4 stream cipher]
//! ```

use aes::cipher::KeyInit;
use aes::Aes128Dec;
use base64::engine::general_purpose::STANDARD as BASE64_STD;
use base64::Engine;
use cipher::block_padding::Pkcs7;
use cipher::BlockDecrypt;
use block_padding::UnpadError;
use musicglass_core::AppError;
use std::io::{Read, Seek, SeekFrom};

// CORE_KEY / META_KEY are the fixed AES-128 keys embedded in the NetEase
// client, documented by every open-source ncmdump implementation.
const CORE_KEY: &[u8; 16] = b"hzHRAmso5kInbaxW";
const META_KEY: &[u8; 16] = b"#14ljk_!\\]&0U<'(";
const NCM_MAGIC: &[u8; 8] = b"CTENFDAM";

/// Error raised when the input is not a valid NCM container.
fn invalid(reason: &str) -> AppError {
    AppError::Plugin {
        plugin: "ncm".into(),
        reason: reason.into(),
    }
}

/// Read a 4-byte little-endian length prefix followed by that many bytes,
/// each XORed with `salt` (the NCM segment obfuscation step).
fn read_xor_segment<R: Read>(reader: &mut R, salt: u8) -> Result<Vec<u8>, AppError> {
    let mut len_buf = [0u8; 4];
    reader
        .read_exact(&mut len_buf)
        .map_err(|_| invalid("truncated: missing segment length"))?;
    let len = u32::from_le_bytes(len_buf) as usize;
    if len == 0 || len > 64 * 1024 * 1024 {
        return Err(invalid("segment length out of range"));
    }
    let mut data = vec![0u8; len];
    reader
        .read_exact(&mut data)
        .map_err(|_| invalid("truncated segment"))?;
    for b in data.iter_mut() {
        *b ^= salt;
    }
    Ok(data)
}

/// AES-128-ECB decrypt with PKCS7 unpadding.
fn aes128_ecb_decrypt(key: &[u8; 16], data: &mut [u8]) -> Result<Vec<u8>, AppError> {
    let cipher = Aes128Dec::new(key.into());
    cipher
        .decrypt_padded::<Pkcs7>(data)
        .map(|d| d.to_vec())
        .map_err(|_: UnpadError| invalid("AES decryption failed (wrong key or corrupt block)"))
}

/// Decrypt the RC4 key: XOR 0x64 → AES-128-ECB(CORE_KEY) → strip prefix.
/// Returns the raw RC4 keystream seed bytes (after "neteasecloudmusic").
fn decrypt_rc4_key(segment: &[u8]) -> Result<Vec<u8>, AppError> {
    let mut buf = segment.to_vec();
    let decrypted = aes128_ecb_decrypt(CORE_KEY, &mut buf)?;
    const PREFIX: &[u8] = b"neteasecloudmusic";
    if !decrypted.starts_with(PREFIX) {
        return Err(invalid("RC4 key segment did not decrypt to expected prefix"));
    }
    Ok(decrypted[PREFIX.len()..].to_vec())
}

/// Decrypt the metadata blob: XOR 0x63 → strip "163 key(Don't modify):"
/// → Base64 → AES-128-ECB(META_KEY) → strip "music:" → JSON bytes.
fn decrypt_metadata(segment: &[u8]) -> Result<Vec<u8>, AppError> {
    const PREFIX: &[u8] = b"163 key(Don't modify):";
    if !segment.starts_with(PREFIX) {
        return Err(invalid("metadata segment did not decrypt to expected prefix"));
    }
    let b64 = &segment[PREFIX.len()..];
    let mut raw = BASE64_STD
        .decode(b64)
        .map_err(|_| invalid("metadata base64 decode failed"))?;
    let decrypted = aes128_ecb_decrypt(META_KEY, &mut raw)?;
    const MUSIC_PREFIX: &[u8] = b"music:";
    if !decrypted.starts_with(MUSIC_PREFIX) {
        return Err(invalid("metadata did not decrypt to expected 'music:' prefix"));
    }
    Ok(decrypted[MUSIC_PREFIX.len()..].to_vec())
}

/// Build the NCM custom RC4 keystream generator.
///
/// KSA is standard RC4. PRGA is the NCM variant:
/// `keystream[t] = sbox[(sbox[t+1] + sbox[(sbox[t+1] + t+1) & 0xff]) & 0xff]`
/// It is stateless — each byte depends only on its index — so audio can be
/// decrypted in chunks without holding the whole song in memory.
struct NcmRc4 {
    sbox: [u8; 256],
}

impl NcmRc4 {
    fn new(key: &[u8]) -> Self {
        let mut sbox = [0u8; 256];
        for (i, b) in sbox.iter_mut().enumerate() {
            *b = i as u8;
        }
        let mut j: u8 = 0;
        for i in 0..256usize {
            let ki = key[i % key.len()];
            j = j.wrapping_add(sbox[i]).wrapping_add(ki);
            sbox.swap(i, j as usize);
        }
        NcmRc4 { sbox }
    }

    /// Keystream byte at position `t` (0-based).
    fn byte_at(&self, t: usize) -> u8 {
        let i = (t + 1) & 0xff;
        let s_i = self.sbox[i] as usize;
        let s_sum = self.sbox[(s_i + i) & 0xff] as usize;
        self.sbox[(s_i + s_sum) & 0xff]
    }

    /// XOR `data` in place starting at absolute offset `start`.
    fn apply(&self, data: &mut [u8], start: usize) {
        for (t, b) in data.iter_mut().enumerate() {
            *b ^= self.byte_at(start + t);
        }
    }
}

/// Parsed NCM container contents.
pub(crate) struct NcmContainer {
    pub(crate) audio: Vec<u8>,
    pub(crate) cover: Option<Vec<u8>>,
    pub(crate) metadata_json: Vec<u8>,
}

/// Parse and fully decrypt an NCM file from a reader.
pub(crate) fn parse_ncm<R: Read + Seek>(reader: &mut R) -> Result<NcmContainer, AppError> {
    // Magic + gap
    let mut magic = [0u8; 10];
    reader
        .read_exact(&mut magic)
        .map_err(|_| invalid("not an NCM file (too short)"))?;
    if &magic[0..8] != NCM_MAGIC {
        return Err(invalid("magic bytes do not match 'CTENFDAM'"));
    }

    let rc4_key_seg = read_xor_segment(reader, 0x64)?;
    let meta_seg = read_xor_segment(reader, 0x63)?;

    // CRC32 (4B) + gap (5B) — skip, not validated here.
    reader
        .seek(SeekFrom::Current(9))
        .map_err(|_| invalid("truncated before cover"))?;

    let cover_seg = read_xor_segment(reader, 0)?;
    let cover = if cover_seg.is_empty() { None } else { Some(cover_seg) };

    // Remaining bytes = encrypted audio.
    let mut audio = Vec::new();
    reader
        .read_to_end(&mut audio)
        .map_err(|_| invalid("failed reading audio body"))?;

    let rc4_key = decrypt_rc4_key(&rc4_key_seg)?;
    let metadata_json = decrypt_metadata(&meta_seg)?;

    let rc4 = NcmRc4::new(&rc4_key);
    rc4.apply(&mut audio, 0);

    Ok(NcmContainer {
        audio,
        cover,
        metadata_json,
    })
}
