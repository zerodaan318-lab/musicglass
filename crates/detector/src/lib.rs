//! Format detection (task book §10).
//!
//! Detects audio format from extension, magic bytes, and container structure.
//! Returns a confidence score so the UI can warn on low-confidence guesses.

use musicglass_core::{AppError, Format, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    pub format: Format,
    pub confidence: f32,
}

impl Detection {
    pub fn is_proprietary(&self) -> bool {
        self.format.is_proprietary()
    }
    pub fn is_supported_input(&self) -> bool {
        !matches!(self.format, Format::Unknown)
    }
}

/// Inspect a file path and return the best-guess format with confidence.
pub fn detect(path: &Path) -> Result<Detection> {
    let data = read_head(path)?;
    // 1) magic bytes win over extension
    if let Some(f) = detect_by_magic(&data) {
        return Ok(Detection { format: f, confidence: 0.99 });
    }
    // 2) fall back to extension
    if let Some(f) = detect_by_ext(path) {
        return Ok(Detection { format: f, confidence: 0.7 });
    }
    Ok(Detection { format: Format::Unknown, confidence: 0.0 })
}

fn read_head(path: &Path) -> Result<Vec<u8>> {
    use std::fs::File;
    use std::io::Read;
    let mut buf = vec![0u8; 64];
    let mut f = File::open(path).map_err(|e| AppError::Io { path: path.to_path_buf(), source: e })?;
    let n = f.read(&mut buf).map_err(|e| AppError::Io { path: path.to_path_buf(), source: e })?;
    buf.truncate(n);
    Ok(buf)
}

fn detect_by_magic(head: &[u8]) -> Option<Format> {
    if head.len() < 4 {
        return None;
    }
    let four = &head[0..4];
    if four == b"fLaC" {
        return Some(Format::Flac);
    }
    if four == b"OggS" {
        if head.windows(5).any(|w| w == b"Opus") {
            return Some(Format::Opus);
        }
        return Some(Format::Ogg);
    }
    if head.len() >= 12 && &head[4..8] == b"ftyp" {
        return Some(Format::M4a); // MP4-family container (m4a/aac)
    }
    if head.len() >= 16 && &head[0..16] == b"RIFF\0\0\0\0WAVE" {
        return Some(Format::Wav);
    }
    if head.len() >= 4 && head[0] == 0xFF && (head[1] & 0xE0) == 0xE0 {
        return Some(Format::Mp3); // MPEG audio frame sync
    }
    // NCM magic: "CTENFDAM" at offset 0 (unencrypted header marker)
    if head.len() >= 8 && &head[0..8] == b"CTENFDAM" {
        return Some(Format::Ncm);
    }
    None
}

fn detect_by_ext(path: &Path) -> Option<Format> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "mp3" => Some(Format::Mp3),
        "flac" => Some(Format::Flac),
        "wav" => Some(Format::Wav),
        "m4a" | "mp4" => Some(Format::M4a),
        "aac" => Some(Format::Aac),
        "ogg" => Some(Format::Ogg),
        "opus" => Some(Format::Opus),
        "ape" => Some(Format::Ape),
        "wma" => Some(Format::Wma),
        "ncm" => Some(Format::Ncm),
        "qmc" | "qmc0" | "qmc2" | "qmc3" | "mgg" | "mgg0" | "mgg1" | "mflac" | "mflac0" => Some(Format::Qmc),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_flac() {
        let head = b"fLaCxxxx";
        assert_eq!(detect_by_magic(head), Some(Format::Flac));
    }

    #[test]
    fn magic_ncm() {
        let head = b"CTENFDAMxxxx";
        assert_eq!(detect_by_magic(head), Some(Format::Ncm));
    }

    #[test]
    fn ext_qmc() {
        assert_eq!(detect_by_ext(std::path::Path::new("a.qmc3")), Some(Format::Qmc));
        assert_eq!(detect_by_ext(std::path::Path::new("a.mflac0")), Some(Format::Qmc));
    }

    #[test]
    fn unknown_by_ext() {
        assert_eq!(detect_by_ext(std::path::Path::new("a.txt")), None);
    }
}
