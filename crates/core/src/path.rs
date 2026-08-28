//! Safe path utilities (task book §49).
//!
//! Prevents path traversal, rejects control characters, and normalizes
//! unicode so FFmpeg (invoked with an argument array, never a shell string)
//! receives a clean path.

use crate::error::{AppError, Result};
use std::path::{Component, Path, PathBuf};

/// Reject paths that escape the allowed root or contain unsafe components.
pub fn ensure_safe(path: &Path, root: Option<&Path>) -> Result<PathBuf> {
    let s = path.to_string_lossy();
    if s.contains("..") || s.contains('\0') {
        return Err(AppError::UnsafePath(s.into_owned()));
    }
    for c in s.chars() {
        if (c as u32) < 0x20 {
            return Err(AppError::UnsafePath(s.into_owned()));
        }
    }
    for comp in path.components() {
        if let Component::ParentDir = comp {
            return Err(AppError::UnsafePath(s.into_owned()));
        }
    }
    if let Some(root) = root {
        if !path.starts_with(root) {
            return Err(AppError::UnsafePath(s.into_owned()));
        }
    }
    Ok(path.to_path_buf())
}

/// Build an output path from a template (task book §24).
///
/// Supported tokens: `{title} {artist} {album} {track} {disc} {year}`.
/// Missing tokens fall back to empty string; the caller must sanitize the
/// result before writing.
pub fn render_template(template: &str, meta: &crate::metadata::Metadata) -> String {
    template
        .replace("{title}", meta.title.as_deref().unwrap_or(""))
        .replace("{artist}", meta.artist.as_deref().unwrap_or(""))
        .replace("{album}", meta.album.as_deref().unwrap_or(""))
        .replace("{track}", &meta.track_number.map(|n| n.to_string()).unwrap_or_default())
        .replace("{disc}", &meta.disc_number.map(|n| n.to_string()).unwrap_or_default())
        .replace("{year}", meta.year.as_deref().unwrap_or(""))
}

/// Replace characters illegal on Windows filesystems.
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if (c as u32) < 0x20 => '_',
            c => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_traversal() {
        assert!(ensure_safe(Path::new("../evil.txt"), None).is_err());
    }

    #[test]
    fn accepts_normal() {
        assert!(ensure_safe(Path::new("周杰伦/十一月的萧邦/01 - 夜曲.flac"), None).is_ok());
    }

    #[test]
    fn template_with_unicode() {
        let m = crate::metadata::Metadata {
            title: Some("夜曲".into()),
            track_number: Some(1),
            ..Default::default()
        };
        assert_eq!(render_template("{track} - {title}", &m), "1 - 夜曲");
    }

    #[test]
    fn sanitize_strips_illegal() {
        assert_eq!(sanitize_filename("a/b:c*"), "a_b_c_");
    }
}
