//! Shared plugin/extraction types (task book §7, §8, §9).
//!
//! `FileInfo` and `AudioStream` describe what a proprietary-container plugin
//! extracts. They live in `core` (not `plugins`) so that the standalone
//! `ncm`/`qmc` crates can use them while only depending on `core`.

use crate::audio::AudioInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub format: String,
    pub inner_codec: String,
    pub audio_info: AudioInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStream {
    /// Decoded bytes (or container bytes for passthrough).
    pub data: Vec<u8>,
    pub codec: String,
}
