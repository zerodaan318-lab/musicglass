//! Audio conversion engine (task book §6, §15, §26).
//!
//! Wraps FFmpeg as an external binary invoked with an **argument array**
//! (never a shell string) so user paths cannot inject commands (§49).

use musicglass_core::{AppError, AudioInfo, Format, Metadata, Result};
use std::path::Path;
use std::process::Command;

/// Locate the bundled FFmpeg executable.
///
/// Resolution order:
/// 1. env `MUSICGLASS_FFMPEG`
/// 2. `resources/ffmpeg/bin/ffmpeg.exe` relative to the current exe
/// 3. `ffmpeg` on PATH
pub fn ffmpeg_path() -> Result<std::path::PathBuf> {
    if let Ok(p) = std::env::var("MUSICGLASS_FFMPEG") {
        return Ok(std::path::PathBuf::from(p));
    }
    if let Some(exe) = std::env::current_exe().ok() {
        // cargo test binaries live in target/debug/deps/, ffmpeg is copied to
        // target/debug/resources/ by build.rs
        let candidates = [
            exe.parent().map(|p| p.join("resources/ffmpeg/bin/ffmpeg.exe")),
            exe.parent()
                .and_then(|p| p.parent())
                .map(|p| p.join("resources/ffmpeg/bin/ffmpeg.exe")),
        ];
        for c in candidates.into_iter().flatten() {
            if c.exists() {
                return Ok(c);
            }
        }
    }
    Ok(std::path::PathBuf::from("ffmpeg"))
}

/// Run `ffmpeg -version` to confirm availability (used by `doctor`).
pub fn ffmpeg_available() -> bool {
    Command::new(ffmpeg_path().unwrap_or_default())
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Probe a file and return its [`AudioInfo`] via ffprobe JSON.
pub fn inspect(path: &Path) -> Result<AudioInfo> {
    let probe = ffmpeg_path()?
        .parent()
        .map(|p| p.join("ffprobe.exe"))
        .filter(|p| p.exists())
        .unwrap_or_else(|| std::path::PathBuf::from("ffprobe"));

    let out = Command::new(&probe)
        .args(["-v", "quiet", "-print_format", "json", "-show_format", "-show_streams"])
        .arg(path)
        .output()
        .map_err(|e| AppError::FfmpegFailed(e.to_string()))?;
    if !out.status.success() {
        return Err(AppError::FfmpegFailed(
            String::from_utf8_lossy(&out.stderr).into_owned(),
        ));
    }
    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).map_err(|e| AppError::Other(e.to_string()))?;

    let duration = json["format"]["duration"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let file_size = json["format"]["size"]
        .as_str()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or_else(|| path.metadata().map(|m| m.len()).unwrap_or(0));

    // Find the first audio stream.
    let streams = json["streams"].as_array();
    let audio = streams
        .as_ref()
        .and_then(|s| s.iter().find(|s| s["codec_type"] == "audio"))
        .ok_or_else(|| AppError::Other("no audio stream found".into()))?;

    let codec = audio["codec_name"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();
    let sample_rate = audio["sample_rate"]
        .as_str()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    let channels = audio["channels"].as_u64().unwrap_or(0) as u16;
    let bit_depth = audio["bits_per_raw_sample"]
        .as_str()
        .and_then(|s| s.parse::<u16>().ok());
    let bitrate = json["format"]["bit_rate"]
        .as_str()
        .and_then(|s| s.parse::<u64>().ok());

    let fmt = Format::from_codec(&codec);
    let lossless = fmt.is_lossless() || audio["profile"].as_str() == Some("FLAC");

    Ok(AudioInfo {
        codec,
        bit_depth,
        sample_rate,
        channels,
        duration_secs: duration,
        lossless,
        bitrate,
        file_size,
    })
}

/// Describes a conversion target.
#[derive(Debug, Clone, Default)]
pub struct ConversionRequest {
    pub target_format: Format,
    pub bitrate: Option<u64>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u16>,
    pub metadata: Option<Metadata>,
}

/// Convert `input` to `output` per the given request.
///
/// Uses `-acodec copy` when the source codec already matches the target
/// (lossless→lossless, e.g. FLAC→FLAC) to avoid needless re-encoding (§3).
pub fn convert(input: &Path, output: &Path, req: &ConversionRequest) -> Result<()> {
    let ff = ffmpeg_path()?;
    let mut cmd = Command::new(&ff);
    cmd.arg("-y").arg("-i").arg(input);

    let target = req.target_format;
    // Pick the encoder; prefer stream copy when codec matches.
    let src_info = inspect(input)?;
    let copy_ok = src_info.codec == target.as_str() && req.bitrate.is_none();

    if copy_ok {
        cmd.arg("-acodec").arg("copy");
    } else {
        match target {
            Format::Mp3 => {
                cmd.arg("-acodec").arg("libmp3lame");
                if let Some(br) = req.bitrate {
                    cmd.arg("-b:a").arg(format!("{}k", br / 1000));
                }
            }
            Format::Flac => {
                cmd.arg("-acodec").arg("flac");
            }
            Format::Wav => {
                cmd.arg("-acodec").arg("pcm_s16le");
            }
            Format::M4a => {
                cmd.arg("-acodec").arg("aac");
                if let Some(br) = req.bitrate {
                    cmd.arg("-b:a").arg(format!("{}k", br / 1000));
                }
            }
            Format::Aac => {
                cmd.arg("-acodec").arg("aac");
            }
            Format::Ogg => {
                cmd.arg("-acodec").arg("libvorbis");
            }
            Format::Opus => {
                cmd.arg("-acodec").arg("libopus");
            }
            _ => {
                return Err(AppError::UnsupportedFormat(target.as_str().to_string()));
            }
        }
    }

    if let Some(sr) = req.sample_rate {
        cmd.arg("-ar").arg(sr.to_string());
    }

    cmd.arg(output);
    let out = cmd
        .output()
        .map_err(|e| AppError::FfmpegFailed(e.to_string()))?;
    if !out.status.success() {
        return Err(AppError::FfmpegFailed(
            String::from_utf8_lossy(&out.stderr).into_owned(),
        ));
    }
    Ok(())
}

pub mod verify;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffmpeg_path_resolves() {
        let _ = ffmpeg_path();
    }
}
