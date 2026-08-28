//! Unified error type for MusicGlass.
//!
//! All crates return [`AppError`]. The Tauri/CLI layer maps it into a
//! human-readable message plus an optional `technical` field (task book §32).

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("io error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("format not recognized: {0}")]
    UnsupportedFormat(String),

    #[error("file too large to process safely: {bytes} bytes")]
    FileTooLarge { bytes: u64 },

    #[error("path is not safe (traversal or invalid): {0}")]
    UnsafePath(String),

    #[error("ffmpeg execution failed: {0}")]
    FfmpegFailed(String),

    #[error("metadata field could not be mapped: {field}")]
    MetadataMapping { field: String },

    #[error("plugin error ({plugin}): {reason}")]
    Plugin { plugin: String, reason: String },

    #[error("additional key required to process this file: {reason}")]
    KeyRequired { reason: String },

    #[error("verification failed: {reason}")]
    Verification { reason: String },

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

impl AppError {
    /// Short code used by the UI to pick a recovery suggestion.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Io { .. } => "IO_ERROR",
            AppError::UnsupportedFormat(_) => "UNSUPPORTED_FORMAT",
            AppError::FileTooLarge { .. } => "FILE_TOO_LARGE",
            AppError::UnsafePath(_) => "UNSAFE_PATH",
            AppError::FfmpegFailed(_) => "FFMPEG_FAILED",
            AppError::MetadataMapping { .. } => "METADATA_MAPPING",
            AppError::Plugin { .. } => "PLUGIN_ERROR",
            AppError::KeyRequired { .. } => "KEY_REQUIRED",
            AppError::Verification { .. } => "VERIFICATION_FAILED",
            AppError::Other(_) => "OTHER",
        }
    }

    /// Human-readable suggestion shown in the UI error panel.
    pub fn suggestion(&self) -> &'static str {
        match self {
            AppError::Io { .. } => "Check the file is readable and not locked by another program.",
            AppError::UnsupportedFormat(_) => "The file may be corrupted or use an unsupported structure.",
            AppError::FileTooLarge { .. } => "Try processing fewer/larger files separately, or increase the temp space.",
            AppError::UnsafePath(_) => "The file path contains invalid characters or traversal attempts.",
            AppError::FfmpegFailed(_) => "Reinstall or update FFmpeg, then retry the conversion.",
            AppError::MetadataMapping { .. } => "Some metadata could not be written to the target format.",
            AppError::Plugin { .. } => "The format plugin failed to parse the container. The file may be unsupported.",
            AppError::KeyRequired { .. } => "Provide the required key for this proprietary format to continue.",
            AppError::Verification { .. } => "The output did not match expectations; the source may be corrupted.",
            AppError::Other(_) => "Unexpected error. Check the logs for technical details.",
        }
    }
}
