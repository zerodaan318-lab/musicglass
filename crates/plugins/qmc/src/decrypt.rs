//! QMC decryption (task book §9).
//!
//! Reference implementation: `presburger/qmc-decoder` (MIT License).
//! The QMC container is a whole-file XOR stream cipher driven by a fixed
//! 8x7 lookup table (`seedMap`) and a zig-zag index walker. There is no
//! separate metadata/cover segment — the decrypted output IS the inner audio
//! (MP3 / FLAC / OGG), whose tags are read afterwards via lofty.
//!
//! This is a clean-room translation of the published algorithm; no upstream
//! source is copied.

use musicglass_core::AppError;
use std::io::{Read, Seek, SeekFrom};

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

/// QMC keystream generator, 1:1 with `qmc_decoder::seed`.
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

        // Every 0x8000 bytes, skip one position (recurse to consume it).
        if self.index == 0x8000 || (self.index > 0x8000 && (self.index + 1) % 0x8000 == 0) {
            return self.next_mask();
        }
        ret
    }

    /// Decrypt `data` in place (XOR stream).
    fn apply(&mut self, data: &mut [u8]) {
        for b in data.iter_mut() {
            *b ^= self.next_mask();
        }
    }
}

/// Decrypt a whole QMC-file buffer in place, returning the inner audio bytes.
pub(crate) fn decrypt_qmc<R: Read + Seek>(reader: &mut R) -> Result<Vec<u8>, AppError> {
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

    let mut seed = QmcSeed::new();
    seed.apply(&mut data);
    Ok(data)
}
