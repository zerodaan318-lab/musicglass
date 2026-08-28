//! Audio container/codec identifiers (shared across all crates).
//!
//! Lives in `core` because every crate (detector, audio, metadata, plugins)
//! needs it — keeping it here avoids a dependency cycle.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Format {
    Mp3,
    Flac,
    Wav,
    M4a,
    Aac,
    Ogg,
    Opus,
    Ape,
    Wma,
    Alac,
    Aiff,
    /// NetEase Cloud Music encrypted container (task book §8).
    Ncm,
    /// QQ Music encrypted container family (task book §9).
    Qmc,
    Unknown,
}

impl Format {
    /// Human-readable lowercase name, e.g. `"flac"`. Used for FFmpeg codec args
    /// and error messages.
    pub fn as_str(&self) -> &'static str {
        match self {
            Format::Mp3 => "mp3",
            Format::Flac => "flac",
            Format::Wav => "wav",
            Format::M4a => "m4a",
            Format::Aac => "aac",
            Format::Ogg => "ogg",
            Format::Opus => "opus",
            Format::Ape => "ape",
            Format::Wma => "wma",
            Format::Alac => "alac",
            Format::Aiff => "aiff",
            Format::Ncm => "ncm",
            Format::Qmc => "qmc",
            Format::Unknown => "unknown",
        }
    }

    /// Whether this is a mathematically lossless audio codec.
    pub fn is_lossless(&self) -> bool {
        matches!(
            self,
            Format::Flac | Format::Wav | Format::Ape | Format::Alac | Format::Aiff
        )
    }

    /// Whether this format is a proprietary container requiring plugin handling.
    pub fn is_proprietary(&self) -> bool {
        matches!(self, Format::Ncm | Format::Qmc)
    }
}
