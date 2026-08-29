//! QMC plugin: implements [`MusicContainer`] for QQ Music encrypted containers
//! (task book §9). Handles the `.qmc` / `.qmc0` / `.qmc2` / `.qmc3` / `.mgg` /
//! `.mgg0` / `.mgg1` / `.mflac` / `.mflac0` / `.qmcogg` family.
//!
//! QMC has two generations (see `docs/FORMAT_RESEARCH_QMC.md`):
//! - **v1 static**: whole-file XOR with a fixed keystream (no ekey).
//! - **QMC2 (v2)**: ekey stored at the file tail drives an RC4-variant or
//!   Map-variant cipher.
//!
//! QMC has NO separate metadata/cover segment — the decrypted output IS the
//! inner audio (MP3/FLAC/OGG). Tags and cover are read from that audio via
//! lofty, exactly like any standard file.

mod decrypt;
pub mod metadata;

pub use decrypt::{MapCipher, QmcSeed, Rc4Cipher};

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
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let (audio, codec) = decrypt::decrypt_qmc(&mut file, &ext)?;
        let lossless = matches!(codec.as_str(), "flac" | "wav" | "ape" | "alac" | "aiff");

        Ok(FileInfo {
            format: "qmc".into(),
            inner_codec: codec.clone(),
            audio_info: AudioInfo {
                codec,
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
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let (audio, codec) = decrypt::decrypt_qmc(&mut file, &ext)?;
        Ok(AudioStream {
            data: audio,
            codec,
        })
    }

    /// QMC stores tags inside the decrypted audio, not in a side segment.
    /// Decrypt to a temp file and read tags via lofty.
    fn extract_metadata(path: &Path) -> Result<Metadata> {
        let mut file = File::open(path).map_err(|e| AppError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let (audio, _) = decrypt::decrypt_qmc(&mut file, &ext)?;

        let tmp_ext = metadata::format_from_ext(path);
        let tmp = write_temp(&audio, tmp_ext)?;
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
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let (audio, _) = decrypt::decrypt_qmc(&mut file, &ext)?;

        let tmp_ext = metadata::format_from_ext(path);
        let tmp = write_temp(&audio, tmp_ext)?;
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
