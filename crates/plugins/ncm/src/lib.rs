//! NCM plugin: implements [`MusicContainer`] for NetEase Cloud Music `.ncm`
//! encrypted containers (task book §8).
//!
//! The decryption primitives live in `decrypt.rs` (a clean-room
//! reimplementation of the publicly documented NCM format). Metadata mapping
//! lives in `metadata.rs`. This file wires them into the plugin trait.

mod decrypt;
pub mod metadata;

use musicglass_core::{
    AppError, AudioInfo, AudioStream, FileInfo, Metadata, MusicContainer,
};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// NCM plugin marker type (zero-sized, static dispatch).
pub struct NcmPlugin;

impl MusicContainer for NcmPlugin {
    /// Identify by magic bytes `CTENFDAM` at offset 0.
    fn can_handle(path: &Path) -> bool {
        match File::open(path) {
            Ok(f) => {
                let mut reader = BufReader::new(f);
                let mut head = [0u8; 8];
                use std::io::Read;
                reader.read_exact(&mut head).is_ok() && &head == b"CTENFDAM"
            }
            Err(_) => false,
        }
    }

    /// Parse the container enough to report structure + inner audio info.
    /// This fully decrypts (needed to sniff the inner codec), but only builds
    /// a lightweight `AudioInfo` without delegating to FFprobe (NCM inner is a
    /// standard codec the detector can read directly later).
    fn inspect(path: &Path) -> Result<FileInfo, AppError> {
        let mut file = File::open(path).map_err(|e| AppError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        let parsed = decrypt::parse_ncm(&mut file)?;

        let inner = metadata::detect_inner_format(&parsed.metadata_json, &parsed.audio);
        let lossless = matches!(inner.as_str(), "flac" | "wav" | "ape" | "alac" | "aiff");

        // Best-effort duration/bitrate from JSON metadata (no probe needed).
        let (duration_secs, bitrate) = {
            let v: serde_json::Value = match serde_json::from_slice(&parsed.metadata_json) {
                Ok(v) => v,
                Err(_) => serde_json::Value::Null,
            };
            let dur = v["duration"].as_u64().map(|ms| ms as f64 / 1000.0).unwrap_or(0.0);
            let br = v["bitrate"].as_u64(); // bps in NCM JSON
            (dur, br)
        };

        Ok(FileInfo {
            format: "ncm".into(),
            inner_codec: inner.clone(),
            audio_info: AudioInfo {
                codec: inner,
                bit_depth: None,
                sample_rate: 0,
                channels: 0,
                duration_secs,
                lossless,
                bitrate,
                file_size: parsed.audio.len() as u64,
            },
        })
    }

    /// Decrypt and return the raw inner audio bytes.
    fn extract_audio(path: &Path) -> Result<AudioStream, AppError> {
        let mut file = File::open(path).map_err(|e| AppError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        let parsed = decrypt::parse_ncm(&mut file)?;
        let codec = metadata::detect_inner_format(&parsed.metadata_json, &parsed.audio);
        Ok(AudioStream {
            data: parsed.audio,
            codec,
        })
    }

    /// Extract the unified metadata model.
    fn extract_metadata(path: &Path) -> Result<Metadata, AppError> {
        let mut file = File::open(path).map_err(|e| AppError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        let parsed = decrypt::parse_ncm(&mut file)?;
        metadata::parse_metadata(&parsed.metadata_json)
    }

    /// Extract the embedded cover art (JPEG/PNG), if present.
    fn extract_cover(path: &Path) -> Result<Option<Vec<u8>>, AppError> {
        let mut file = File::open(path).map_err(|e| AppError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        let parsed = decrypt::parse_ncm(&mut file)?;
        Ok(parsed.cover)
    }
}
