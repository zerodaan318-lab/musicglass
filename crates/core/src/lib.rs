//! MusicGlass core types and utilities.
//!
//! This crate holds the shared domain model used across all other crates:
//! - [`error::AppError`] — unified error type
//! - [`metadata::Metadata`] — the Unified Metadata Model (task book §20)
//! - [`audio::AudioInfo`] — decoded audio properties
//! - [`path`] — safe path handling (no traversal, no injection)

pub mod audio;
pub mod container;
pub mod error;
pub mod format;
pub mod logger;
pub mod metadata;
pub mod path;
pub mod plugin;

pub use audio::AudioInfo;
pub use container::{AudioStream, FileInfo};
pub use error::{AppError, Result};
pub use format::Format;
pub use metadata::Metadata;
pub use plugin::MusicContainer;
