//! Proprietary format plugin abstraction (task book §7).
//!
//! Every proprietary container (NCM, QMC, ...) implements [`MusicContainer`]
//! with the static-dispatch signature from the task book (no `self`, so the
//! trait is not `dyn`-compatible). Plugin selection happens in the
//! `musicglass-plugins` crate via a [`Plugin`](musicglass_plugins::Plugin)
//! enum, avoiding `Box<dyn>` and keeping the main program free of
//! `if ncm { ... } else if qmc { ... }`.

use crate::error::Result;
use crate::{AudioStream, FileInfo, Metadata};
use std::path::Path;

/// Unified interface every proprietary format plugin must implement.
///
/// Methods take `path` directly (no `self`) so each plugin is a zero-sized
/// marker type; the `musicglass-plugins` crate wraps them in an enum for
/// dynamic selection.
pub trait MusicContainer {
    /// Whether this plugin can handle the given file (by magic bytes).
    fn can_handle(path: &Path) -> bool;

    /// Inspect container structure without full extraction.
    fn inspect(path: &Path) -> Result<FileInfo>;

    /// Extract the raw inner audio stream.
    fn extract_audio(path: &Path) -> Result<AudioStream>;

    /// Extract the unified metadata.
    fn extract_metadata(path: &Path) -> Result<Metadata>;

    /// Extract cover art, if present.
    fn extract_cover(path: &Path) -> Result<Option<Vec<u8>>>;
}
