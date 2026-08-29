//! QMC plugin: implements [`MusicContainer`] for QQ Music encrypted containers
//! (task book §9). Handles the `.qmc` / `.qmc0` / `.qmc2` / `.qmc3` / `.mgg` /
//! `.mgg0` / `.mgg1` / `.mflac` / `.mflac0` family.
//!
//! Unlike NCM, QMC has NO separate metadata/cover segment — the whole file is
//! a single XOR stream cipher, and the decrypted output IS the inner audio
//! (MP3/FLAC/OGG). Tags and cover are read from that audio via lofty, exactly
//! like any standard file.

mod decrypt;
pub mod metadata;

pub use decrypt::QmcSeed;

use musicglass_core::{
    AppError, AudioInfo, AudioStream, FileInfo, Metadata, MusicContainer, Result,
};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct QmcPlugin;

impl MusicContainer for QmcPlugin {
    /// Identify by extension (QMC is not magic-byte tagged).
    fn can_handle(path: &Path) -> bool {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        matches!(
            ext.as_str(),
            "qmc" | "qmc0" | "qmc2" | "qmc3" | "mgg" | "mgg0" | "mgg1" | "mflac" | "mflac0" | "qmcogg"
        )
    }

    /// Decrypt and report the inner audio info (sniffed from magic bytes).
    fn inspect(path: &Path) -> Result<FileInfo> {
        let mut file = File::open(path).map_err(|e| AppError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        let audio = decrypt::decrypt_qmc(&mut file)?;
        let codec = metadata::detect_by_magic(&audio);
        let lossless = matches!(codec, "flac" | "wav" | "ape" | "alac" | "aiff");

        Ok(FileInfo {
            format: "qmc".into(),
            inner_codec: codec.into(),
            audio_info: AudioInfo {
                codec: codec.into(),
                bit_depth: None,
                sample_rate: 0,
                channels: 0,
                duration_secs: 0.0,
                lossless,
                bitrate: None,
                file_size: audio.len() as u64,
            },
        })
    }

    /// Decrypt and return the raw inner audio bytes.
    fn extract_audio(path: &Path) -> Result<AudioStream> {
        let mut file = File::open(path).map_err(|e| AppError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        let audio = decrypt::decrypt_qmc(&mut file)?;
        let codec = metadata::detect_by_magic(&audio);
        Ok(AudioStream {
            data: audio,
            codec: codec.into(),
        })
    }

    /// QMC stores tags inside the decrypted audio, not in a side segment.
    /// Decrypt to a temp file and read tags via lofty.
    fn extract_metadata(path: &Path) -> Result<Metadata> {
        let mut file = File::open(path).map_err(|e| AppError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        let audio = decrypt::decrypt_qmc(&mut file)?;

        let ext = metadata::format_from_ext(path);
        let tmp = write_temp(&audio, ext)?;
        let res = musicglass_metadata::read_metadata(&tmp);
        let _ = std::fs::remove_file(&tmp);
        res.map_err(|e| AppError::Metadata {
            reason: e.to_string(),
        })
    }

    /// Same approach as metadata: decrypt to temp, read cover via lofty.
    fn extract_cover(path: &Path) -> Result<Option<Vec<u8>>> {
        let mut file = File::open(path).map_err(|e| AppError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        let audio = decrypt::decrypt_qmc(&mut file)?;

        let ext = metadata::format_from_ext(path);
        let tmp = write_temp(&audio, ext)?;
        let res = musicglass_metadata::read_cover(&tmp);
        let _ = std::fs::remove_file(&tmp);
        res.map(|opt| opt.map(|c| c.data))
            .map_err(|e| AppError::Metadata {
                reason: e.to_string(),
            })
    }
}

/// Write decrypted audio to a per-run temp file so lofty can probe it.
fn write_temp(audio: &[u8], ext: &str) -> Result<PathBuf> {
    let mut path = std::env::temp_dir();
    // unique enough for a transient probe; not security-sensitive
    let name = format!("musicglass_qmc_{}_{}.{}", std::process::id(), ext, ext);
    path.push(name);
    let mut f = File::create(&path).map_err(|e| AppError::Io {
        path: path.clone(),
        source: e,
    })?;
    f.write_all(audio).map_err(|e| AppError::Io {
        path: path.clone(),
        source: e,
    })?;
    Ok(path)
}
