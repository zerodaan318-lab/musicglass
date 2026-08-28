//! NCM plugin (task book §8).
//!
//! Phase 5 implements full parsing. For now it declares the plugin and its
//! magic-byte detection so the registry can route `.ncm` files to it.

use musicglass_core::{AppError, AudioInfo, AudioStream, FileInfo, Metadata, Result};
use musicglass_core::MusicContainer;
use std::path::Path;

pub struct NcmPlugin;

impl MusicContainer for NcmPlugin {
    fn can_handle(path: &Path) -> bool {
        // NCM unencrypted header marker "CTENFDAM" at offset 0.
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext.eq_ignore_ascii_case("ncm") {
                return true;
            }
        }
        std::fs::read(path)
            .map(|b| b.len() >= 8 && &b[0..8] == b"CTENFDAM")
            .unwrap_or(false)
    }

    fn inspect(_path: &Path) -> Result<FileInfo> {
        Err(AppError::Plugin {
            plugin: "ncm".into(),
            reason: "NCM parsing implemented in Phase 5".into(),
        })
    }
    fn extract_audio(_path: &Path) -> Result<AudioStream> {
        Err(AppError::Plugin { plugin: "ncm".into(), reason: "Phase 5".into() })
    }
    fn extract_metadata(_path: &Path) -> Result<Metadata> {
        Err(AppError::Plugin { plugin: "ncm".into(), reason: "Phase 5".into() })
    }
    fn extract_cover(_path: &Path) -> Result<Option<Vec<u8>>> {
        Err(AppError::Plugin { plugin: "ncm".into(), reason: "Phase 5".into() })
    }
}

#[allow(unused)]
fn _unused(_: AudioInfo) {}
