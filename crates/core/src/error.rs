//! Unified error type for MusicGlass.
//!
//! All crates return [`AppError`]. The Tauri/CLI layer maps it into a
//! human-readable message plus an optional `technical` field (task book §32).

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("文件读写错误（路径 {path}）：{source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("无法识别的文件格式：{0}")]
    UnsupportedFormat(String),

    #[error("文件过大，超出安全处理上限（{bytes} 字节）")]
    FileTooLarge { bytes: u64 },

    #[error("路径不安全（可能为越界访问或非法字符）：{0}")]
    UnsafePath(String),

    #[error("FFmpeg 执行失败：{0}")]
    FfmpegFailed(String),

    #[error("元数据错误：{reason}")]
    Metadata { reason: String },

    #[error("元数据字段无法映射：{field}")]
    MetadataMapping { field: String },

    #[error("插件错误（{plugin}）：{reason}")]
    Plugin { plugin: String, reason: String },

    #[error("处理此文件需要额外的密钥：{reason}")]
    KeyRequired { reason: String },

    #[error("校验失败：{reason}")]
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
            AppError::Metadata { .. } => "METADATA_ERROR",
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
            AppError::Io { .. } => "请确认文件可读且未被其他程序占用。",
            AppError::UnsupportedFormat(_) => "文件可能已损坏，或采用了不支持的结构。",
            AppError::FileTooLarge { .. } => "建议分批处理较大文件，或清理临时空间后重试。",
            AppError::UnsafePath(_) => "文件路径包含非法字符或越界访问尝试。",
            AppError::FfmpegFailed(_) => "请重新安装或更新 FFmpeg 后重试转换。",
            AppError::Metadata { .. } => "无法读取或写入元数据，文件可能已损坏。",
            AppError::MetadataMapping { .. } => "部分元数据无法写入目标格式。",
            AppError::Plugin { .. } => "格式插件解析容器失败，该文件可能不被支持。",
            AppError::KeyRequired { .. } => "请提供该专有格式所需的密钥以继续。",
            AppError::Verification { .. } => "输出结果与预期不符，源文件可能已损坏。",
            AppError::Other(_) => "发生未预期的错误，请查看日志获取技术详情。",
        }
    }
}
