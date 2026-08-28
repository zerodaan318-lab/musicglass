//! Audio conversion engine (task book §6, §15, §26).
//!
//! Wraps FFmpeg as an external binary invoked with an **argument array**
//! (never a shell string) so user paths cannot inject commands (§49).
//! This phase implements the orchestration skeleton; FFmpeg path resolution
//! and bit-perfect verification are filled in once the bundled binary is
//! confirmed present (see `doctor`).

use musicglass_core::{AppError, AudioInfo, Metadata, Result};
use std::path::Path;
use std::process::Command;

/// Locate the bundled FFmpeg executable.
///
/// Resolution order:
/// 1. env `MUSICGLASS_FFMPEG`
/// 2. `resources/ffmpeg/bin/ffmpeg.exe` relative to the executable
/// 3. `ffmpeg` on PATH
pub fn ffmpeg_path() -> Result<std::path::PathBuf> {
    if let Ok(p) = std::env::var("MUSICGLASS_FFMPEG") {
        return Ok(std::path::PathBuf::from(p));
    }
    let candidate = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.join("resources/ffmpeg/bin/ffmpeg.exe")))
        .filter(|p| p.exists());
    if let Some(p) = candidate {
        return Ok(p);
    }
    Ok(std::path::PathBuf::from("ffmpeg")) // rely on PATH
}

/// Run `ffmpeg -version` to confirm availability (used by `doctor`).
pub fn ffmpeg_available() -> bool {
    Command::new(ffmpeg_path().unwrap_or_default())
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Probe a file and return its [`AudioInfo`] using ffprobe-style JSON.
///
/// NOTE: full ffprobe JSON parsing is implemented in Phase 2 follow-up;
/// this skeleton returns an error until FFmpeg is confirmed bundled.
pub fn inspect(path: &Path) -> Result<AudioInfo> {
    let _ = path;
    Err(AppError::Other(
        "inspect() requires bundled FFmpeg; wire ffprobe JSON parsing here".into(),
    ))
}

/// Convert `input` to `output` per the given request.
///
/// Skeleton: builds the argument list and would spawn FFmpeg. Actual
/// encoding parameters (codec, bitrate, sample rate) are applied by the
/// caller via [`ConversionRequest`]. Verification is delegated to the
/// `verify` module (Phase 3).
pub fn convert(input: &Path, output: &Path, req: &ConversionRequest) -> Result<()> {
    let ff = ffmpeg_path()?;
    let mut cmd = Command::new(&ff);
    cmd.arg("-y").arg("-i").arg(input);
    // TODO(phase2): map req -> encoder/params
    let _ = req;
    cmd.arg(output);
    let out = cmd.output().map_err(|e| AppError::FfmpegFailed(e.to_string()))?;
    if !out.status.success() {
        return Err(AppError::FfmpegFailed(
            String::from_utf8_lossy(&out.stderr).into_owned(),
        ));
    }
    Ok(())
}

/// Describes a conversion target (Phase 2 skeleton; extended in Phase 3).
#[derive(Debug, Clone, Default)]
pub struct ConversionRequest {
    pub target_format: String,
    pub bitrate: Option<u64>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u16>,
    pub metadata: Option<Metadata>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffmpeg_path_resolves() {
        // Should not panic; may fall back to "ffmpeg" on PATH.
        let _ = ffmpeg_path();
    }
}
