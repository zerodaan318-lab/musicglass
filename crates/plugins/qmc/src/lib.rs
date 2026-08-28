//! QMC plugin (task book §9).
//!
//! Phase 6 implements per-version parsing (qmc0/2/3, mgg, mflac...).
//! Different QMC versions are NOT treated as one format — each is handled
//! separately. Key-required cases return [`AppError::KeyRequired`].

use musicglass_core::{AppError, AudioInfo, AudioStream, FileInfo, Metadata, Result};
use musicglass_core::MusicContainer;
use std::path::Path;

pub struct QmcPlugin;

impl MusicContainer for QmcPlugin {
    fn can_handle(path: &Path) -> bool {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        matches!(
            ext.as_str(),
            "qmc" | "qmc0" | "qmc2" | "qmc3" | "mgg" | "mgg0" | "mgg1" | "mflac" | "mflac0"
        )
    }

    fn inspect(_path: &Path) -> Result<FileInfo> {
        Err(AppError::Plugin {
            plugin: "qmc".into(),
            reason: "QMC parsing implemented in Phase 6 (per-version)".into(),
        })
    }
    fn extract_audio(_path: &Path) -> Result<AudioStream> {
        Err(AppError::Plugin { plugin: "qmc".into(), reason: "Phase 6".into() })
    }
    fn extract_metadata(_path: &Path) -> Result<Metadata> {
        Err(AppError::Plugin { plugin: "qmc".into(), reason: "Phase 6".into() })
    }
    fn extract_cover(_path: &Path) -> Result<Option<Vec<u8>>> {
        Err(AppError::Plugin { plugin: "qmc".into(), reason: "Phase 6".into() })
    }
}

#[allow(unused)]
fn _unused(_: AudioInfo) {}
