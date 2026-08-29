//! QMC decryption (task book §9).
//!
//! Reference implementations studied (all public, used for algorithm knowledge
//! only — this is a clean-room reimplementation, no upstream source copied):
//! - `presburger/qmc-decoder` (MIT): the original static seedMap XOR cipher
//!   used by old `.qmc0`/`.qmcflac` files (no ekey required).
//! - `bczhc/qmc-decrypt` → `third_party/qmc2-rust` (MIT/Apache): the QMC2
//!   format with ekey-driven ciphers (RC4-variant and Map-variant), plus the
//!   `QTag` tail + metadata/ekey layout.
//!
//! QMC is NOT a single algorithm. Two generations:
//! 1. **Static v1**: whole file XOR'd with a fixed 8x7 lookup-table keystream
//!    (`QmcSeed`). No per-file key. Used by legacy `.qmc0`/`.qmcflac`.
//! 2. **QMC2 (v2)**: cipher key derived from a Base64 `ekey` stored at the
//!    file tail (after a `QTag` magic). The key selects RC4-variant (key>300B)
//!    or Map-variant (key<=300B).

use musicglass_core::AppError;
use std::io::{Read, Seek, SeekFrom};
use base64::Engine;

// ───────────────────────────── v1: static seedMap ───────────────────────────

/// Fixed keystream lookup table (from presburger/qmc-decoder seed.hpp).
const SEED_MAP: [[u8; 7]; 8] = [
    [0x4a, 0xd6, 0xca, 0x90, 0x67, 0xf7, 0x52],
    [0x5e, 0x95, 0x23, 0x9f, 0x13, 0x11, 0x7e],
    [0x47, 0x74, 0x3d, 0x90, 0xaa, 0x3f, 0x51],
    [0xc6, 0x09, 0xd5, 0x9f, 0xfa, 0x66, 0xf9],
    [0xf3, 0xd6, 0xa1, 0x90, 0xa0, 0xf7, 0xf0],
    [0x1d, 0x95, 0xde, 0x9f, 0x84, 0x11, 0xf4],
    [0x0e, 0x74, 0xbb, 0x90, 0xbc, 0x3f, 0x92],
    [0x00, 0x09, 0x5b, 0x9f, 0x62, 0x66, 0xa1],
];

/// QMC v1 keystream generator, 1:1 with `qmc_decoder::seed` (seed.hpp).
pub struct QmcSeed {
    x: i32,
    y: i32,
    dx: i32,
    index: i32,
}

impl QmcSeed {
    pub fn new() -> Self {
        QmcSeed {
            x: -1,
            y: 8,
            dx: 1,
            index: -1,
        }
    }

    /// Produce the next keystream byte.
    pub fn next_mask(&mut self) -> u8 {
        self.index += 1;
        let ret = if self.x < 0 {
            self.dx = 1;
            self.y = (8 - self.y) % 8;
            0xc3
        } else if self.x > 6 {
            self.dx = -1;
            self.y = 7 - self.y;
            0xd8
        } else {
            SEED_MAP[self.y as usize][self.x as usize]
        };

        self.x += self.dx;

        // Every 0x8000 bytes, skip one position (consume it recursively).
        if self.index == 0x8000 || (self.index > 0x8000 && (self.index + 1) % 0x8000 == 0) {
            return self.next_mask();
        }
        ret
    }

    /// Decrypt `data` in place (XOR stream).
    pub fn apply(&mut self, data: &mut [u8]) {
        for b in data.iter_mut() {
            *b ^= self.next_mask();
        }
    }
}

// ──────────────────────── v2: ekey + RC4/Map ciphers ───────────────────────

const QMC2_PREFIX: &[u8] = b"QQMusic EncV2,Key:";
const STAGE1_KEY: &[u8] = b"386ZJY!@#*$%^&)(";
const STAGE2_KEY: &[u8] = b"**#!(#$%&^a1cZ,T";

/// TEA (Tiny Encryption Algorithm) decrypt, 32 rounds, little-endian u32.
/// Used by QMC2 to unwrap the ekey.
fn tea_decrypt(block: &[u8], key: &[u8]) -> Option<Vec<u8>> {
    if block.len() % 8 != 0 || key.len() < 16 {
        return None;
    }
    let k = |i: usize| u32::from_le_bytes([key[4 * i], key[4 * i + 1], key[4 * i + 2], key[4 * i + 3]]);
    let k0 = k(0);
    let k1 = k(1);
    let k2 = k(2);
    let k3 = k(3);
    let delta: u32 = 0x9e3779b9;
    let mut out = Vec::with_capacity(block.len());
    let mut chunks = block.chunks_exact(8);
    for c in &mut chunks {
        let mut v0 = u32::from_le_bytes([c[0], c[1], c[2], c[3]]);
        let mut v1 = u32::from_le_bytes([c[4], c[5], c[6], c[7]]);
        let mut sum: u32 = delta.wrapping_mul(32);
        for _ in 0..32 {
            v1 = v1.wrapping_sub(
                (v0 << 4)
                    .wrapping_add(k2)
                    .wrapping_mul(v0.wrapping_add(sum))
                    .wrapping_add((v0 >> 5).wrapping_add(k3)),
            );
            v0 = v0.wrapping_sub(
                (v1 << 4)
                    .wrapping_add(k0)
                    .wrapping_mul(v1.wrapping_add(sum))
                    .wrapping_add((v1 >> 5).wrapping_add(k1)),
            );
            sum = sum.wrapping_sub(delta);
        }
        out.extend_from_slice(&v0.to_le_bytes());
        out.extend_from_slice(&v1.to_le_bytes());
    }
    // strip PKCS#7-like padding (last byte = pad count, all equal)
    if let Some(&pad) = out.last() {
        let pad = pad as usize;
        if pad > 0 && pad <= 8 && out.len() >= pad {
            let all_pad = out[out.len() - pad..].iter().all(|&b| b == pad as u8);
            if all_pad {
                out.truncate(out.len() - pad);
            }
        }
    }
    Some(out)
}

/// Derive the 16-byte TEA key from the 8-byte ekey header.
fn derive_tea_key(ekey_header: &[u8]) -> [u8; 16] {
    // simple_make_key(106, 8) → fixed table
    let simple = [
        0x69u8, 0x56, 0x46, 0x38, 0x2b, 0x20, 0x15, 0x0b,
    ];
    let mut tea_key = [0u8; 16];
    for i in (0..16).step_by(2) {
        tea_key[i] = simple[i / 2];
        tea_key[i + 1] = ekey_header[i / 2];
    }
    tea_key
}

/// Parse the ekey string into the raw cipher key (QMC2 v2 aware).
fn parse_ekey(ekey: &str) -> Result<Vec<u8>, AppError> {
    let ekey = ekey.trim_matches('\0');
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(ekey.as_bytes())
        .map_err(|_| AppError::KeyRequired {
            reason: "ekey is not valid Base64".into(),
        })?;

    if decoded.len() < 8 {
        return Err(AppError::KeyRequired {
            reason: "ekey too short".into(),
        });
    }

    let decoded = if decoded.starts_with(QMC2_PREFIX) {
        // EncV2: two-stage TEA.
        let blob = &decoded[QMC2_PREFIX.len()..];
        let s1 = tea_decrypt(blob, STAGE1_KEY)
            .ok_or_else(|| AppError::KeyRequired {
                reason: "EncV2 stage1 TEA failed".into(),
            })?;
        let s2 = tea_decrypt(&s1, STAGE2_KEY)
            .ok_or_else(|| AppError::KeyRequired {
                reason: "EncV2 stage2 TEA failed".into(),
            })?;
        base64::engine::general_purpose::STANDARD
            .decode(&s2)
            .map_err(|_| AppError::KeyRequired {
                reason: "EncV2 inner base64 failed".into(),
            })?
    } else {
        decoded
    };

    let (header, body) = decoded.split_at(8);
    let tea_key = derive_tea_key(header);
    let body = tea_decrypt(body, &tea_key).ok_or_else(|| AppError::KeyRequired {
        reason: "ekey body TEA failed".into(),
    })?;
    Ok([header.to_vec(), body].concat())
}

/// QMC2 Map cipher (key length <= 300). `map_l` keystream.
pub struct MapCipher {
    key: Vec<u8>,
}

impl MapCipher {
    pub fn new(key: Vec<u8>) -> Self {
        MapCipher { key }
    }

    #[inline]
    fn scramble_by_index(value: u8, index: usize) -> u8 {
        let rot = (index as u32).wrapping_add(4) & 0b111;
        let left = value.wrapping_shl(rot);
        let right = value.wrapping_shr(rot);
        left | right
    }

    #[inline]
    fn map_l(&self, offset: usize) -> u8 {
        let mut off = offset;
        if off > 0x7FFF {
            off %= 0x7FFF;
        }
        let idx = (off * off + 71214) % self.key.len();
        MapCipher::scramble_by_index(self.key[idx], idx)
    }

    pub fn apply(&self, data: &mut [u8], base_offset: usize) {
        for (i, b) in data.iter_mut().enumerate() {
            *b ^= self.map_l(base_offset + i);
        }
    }
}

/// QMC2 RC4-variant cipher (key length > 300).
pub struct Rc4Cipher {
    s: Vec<u8>,
    hash: u32,
    rc4_key: Vec<u8>,
}

const FIRST_SEGMENT_SIZE: usize = 0x80;
const OTHER_SEGMENT_SIZE: usize = 0x1400;

impl Rc4Cipher {
    pub fn new(rc4_key: &[u8]) -> Self {
        let n = rc4_key.len();
        let mut s = vec![0u8; n];
        for (i, b) in s.iter_mut().enumerate() {
            *b = i as u8;
        }
        let mut j = 0usize;
        for (i, &key) in rc4_key.iter().enumerate() {
            j = j.wrapping_add(s[i] as usize).wrapping_add(key as usize) % n;
            s.swap(i, j);
        }
        Rc4Cipher {
            s,
            hash: Self::calc_hash_base(rc4_key),
            rc4_key: rc4_key.to_vec(),
        }
    }

    fn calc_hash_base(data: &[u8]) -> u32 {
        let mut hash: u32 = 1;
        for &value in data.iter() {
            let value = u32::from(value);
            if value == 0 {
                continue;
            }
            let next = hash.wrapping_mul(value);
            if next == 0 || next <= hash {
                break;
            }
            hash = next;
        }
        hash
    }

    fn calc_segment_key(&self, id: usize, seed: u8) -> usize {
        let dividend = f64::from(self.hash);
        let divisor = ((id + 1) * usize::from(seed)) as f64;
        let key = dividend / divisor * 100.0;
        key as u64 as usize
    }

    fn rc4_derive(n: usize, s: &mut [u8], j: &mut usize, k: &mut usize) -> u8 {
        *j = (*j + 1) % n;
        *k = (usize::from(s[*j]) + *k) % n;
        s.swap(*j, *k);
        let index = usize::from(s[*j]) + usize::from(s[*k]);
        s[index % n]
    }

    fn encode_first_segment(&self, offset: usize, buf: &mut [u8]) {
        let n = self.rc4_key.len();
        let mut offset = offset;
        for b in buf.iter_mut() {
            let key1 = self.rc4_key[offset % n];
            let key2 = self.calc_segment_key(offset, key1);
            *b ^= self.rc4_key[key2 % n];
            offset += 1;
        }
    }

    fn encode_other_segment(&self, offset: usize, buf: &mut [u8]) {
        let seg_id = offset / OTHER_SEGMENT_SIZE;
        let seg_id_small = seg_id & 0x1FF;
        let mut discard = self.calc_segment_key(seg_id, self.rc4_key[seg_id_small]) & 0x1FF;
        discard += offset % OTHER_SEGMENT_SIZE;

        let n = self.rc4_key.len();
        let mut s = self.s.clone();
        let mut j = 0usize;
        let mut k = 0usize;
        for _ in 0..discard {
            Self::rc4_derive(n, &mut s, &mut j, &mut k);
        }
        for b in buf.iter_mut() {
            *b ^= Self::rc4_derive(n, &mut s, &mut j, &mut k);
        }
    }

    pub fn apply(&self, data: &mut [u8], base_offset: usize) {
        let mut offset = base_offset;
        let mut len = data.len();
        let mut i = 0usize;

        if offset < FIRST_SEGMENT_SIZE {
            let n = std::cmp::min(len, FIRST_SEGMENT_SIZE - offset);
            self.encode_first_segment(offset, &mut data[i..i + n]);
            i += n;
            len -= n;
            offset += n;
        }

        let to_align = offset % OTHER_SEGMENT_SIZE;
        if to_align != 0 {
            let n = std::cmp::min(len, OTHER_SEGMENT_SIZE - to_align);
            self.encode_other_segment(offset, &mut data[i..i + n]);
            i += n;
            len -= n;
            offset += n;
        }

        while len > OTHER_SEGMENT_SIZE {
            self.encode_other_segment(offset, &mut data[i..i + OTHER_SEGMENT_SIZE]);
            i += OTHER_SEGMENT_SIZE;
            len -= OTHER_SEGMENT_SIZE;
            offset += OTHER_SEGMENT_SIZE;
        }
        if len > 0 {
            self.encode_other_segment(offset, &mut data[i..i + len]);
        }
    }
}

/// Detect the QMC tail and extract the ekey string.
/// Returns the audio region length and the ekey (if v2).
fn detect_tail(buf: &[u8]) -> Result<(usize, Option<String>), AppError> {
    if buf.len() < 8 {
        return Err(AppError::Plugin {
            plugin: "qmc".into(),
            reason: "file too small".into(),
        });
    }
    // QMC2 v2: last 4 bytes == "QTag" (LE u32 0x67615451)
    let eof_magic = u32::from_le_bytes([buf[buf.len() - 4], buf[buf.len() - 3], buf[buf.len() - 2], buf[buf.len() - 1]]);
    if eof_magic == 0x67615451 {
        // meta_size is big-endian u32 at buf.len()-8
        let meta_size =
            u32::from_be_bytes([buf[buf.len() - 8], buf[buf.len() - 7], buf[buf.len() - 6], buf[buf.len() - 5]]) as usize;
        let ekey_loc = buf.len() - 8 - meta_size;
        let search_start = if ekey_loc > 0 { ekey_loc } else { 0 };
        // find comma that ends the ekey
        let end = buf[search_start..buf.len() - 8]
            .iter()
            .position(|&b| b == b',')
            .map(|p| search_start + p)
            .ok_or_else(|| AppError::Plugin {
                plugin: "qmc".into(),
                reason: "v2 ekey comma not found".into(),
            })?;
        let ekey = String::from_utf8_lossy(&buf[ekey_loc..end]).to_string();
        return Ok((ekey_loc, Some(ekey)));
    }

    // QMC v1: last 4 bytes == ekey length (LE), range 1..=0x400
    let key_size = u32::from_le_bytes([buf[buf.len() - 4], buf[buf.len() - 3], buf[buf.len() - 2], buf[buf.len() - 1]]) as usize;
    if key_size > 0 && key_size <= 0x400 {
        let ekey_loc = buf.len() - 4 - key_size;
        let ekey = String::from_utf8_lossy(&buf[ekey_loc..buf.len() - 4]).to_string();
        return Ok((ekey_loc, Some(ekey)));
    }

    // No v1/v2 tail → static v1 whole-file cipher (legacy .qmc0/.qmcflac)
    Ok((buf.len(), None))
}

/// Decrypt a whole QMC file. Returns (inner_audio_bytes, detected_codec_hint).
/// `codec_hint` is the extension-derived expected format for logging.
pub(crate) fn decrypt_qmc<R: Read + Seek>(
    reader: &mut R,
    ext: &str,
) -> Result<(Vec<u8>, String), AppError> {
    let mut data = Vec::new();
    reader
        .seek(SeekFrom::Start(0))
        .map_err(|e| AppError::Io {
            path: std::path::PathBuf::from("<qmc>"),
            source: e,
        })?;
    reader
        .read_to_end(&mut data)
        .map_err(|e| AppError::Io {
            path: std::path::PathBuf::from("<qmc>"),
            source: e,
        })?;

    if data.is_empty() {
        return Err(AppError::Plugin {
            plugin: "qmc".into(),
            reason: "empty QMC file".into(),
        });
    }

    let (audio_len, ekey) = detect_tail(&data)?;
    let mut audio = data[..audio_len].to_vec();

    match ekey {
        Some(ekey) => {
            let key = parse_ekey(&ekey).map_err(|e| AppError::KeyRequired {
                reason: format!("ekey derive failed: {e}"),
            })?;
            if key.len() > 300 {
                let c = Rc4Cipher::new(&key);
                c.apply(&mut audio, 0);
            } else {
                let c = MapCipher::new(key);
                c.apply(&mut audio, 0);
            }
        }
        None => {
            // legacy static XOR
            let mut seed = QmcSeed::new();
            seed.apply(&mut audio);
        }
    }

    let codec = crate::metadata::detect_by_magic(&audio);
    let _ = ext;
    Ok((audio, codec.into()))
}
