//! Decoded audio properties (task book §15).
//!
//! Values come from FFprobe / container inspection, never inferred from
//! the file extension alone (task book §16).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioInfo {
    /// Container/codec name, e.g. "flac", "mp3", "aac".
    pub codec: String,
    /// Bits per sample (None for lossy/compressed formats).
    pub bit_depth: Option<u16>,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Number of channels.
    pub channels: u16,
    /// Duration in seconds.
    pub duration_secs: f64,
    /// Whether the source is mathematically lossless.
    pub lossless: bool,
    /// Bitrate in bits/sec (meaningful for lossy formats).
    pub bitrate: Option<u64>,
    /// File size in bytes.
    pub file_size: u64,
}

impl AudioInfo {
    /// Lossless codecs as recognized by this project.
    pub fn is_lossless_codec(codec: &str) -> bool {
        matches!(codec.to_ascii_lowercase().as_str(), "flac" | "wav" | "alac" | "ape" | "pcm")
    }

    /// Build a one-line quality summary for the UI.
    pub fn quality_summary(&self) -> String {
        let depth = self
            .bit_depth
            .map(|d| format!("{} bit", d))
            .unwrap_or_else(|| self.bitrate.map(|b| format!("{} kbps", b / 1000)).unwrap_or_default());
        let khz = format!("{:.0} kHz", self.sample_rate as f64 / 1000.0);
        let lossy = if self.lossless { "Lossless" } else { "Lossy" };
        format!("{} · {} · {} · {}", self.codec.to_uppercase(), depth, khz, lossy)
    }
}
