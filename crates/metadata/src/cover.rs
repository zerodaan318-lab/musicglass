//! Cover art read/write (task book §22).
//!
//! Supports JPEG/PNG/WEBP (what lofty can embed). Read returns raw bytes and
//! MIME so the UI can display and the writer can re-embed.

use lofty::file::TaggedFileExt;
use lofty::picture::MimeType;
use lofty::picture::Picture;
use lofty::prelude::*;
use lofty::probe::Probe;
use musicglass_core::Result;
use std::path::Path;

/// Extracted cover art.
#[derive(Debug, Clone)]
pub struct CoverArt {
    pub data: Vec<u8>,
    pub mime: String,
}

/// Read the first embedded cover from a file.
pub fn read_cover(path: &Path) -> Result<Option<CoverArt>> {
    let tagged = Probe::open(path)
        .map_err(|e| musicglass_core::AppError::Metadata { reason: e.to_string() })?
        .read()
        .map_err(|e| musicglass_core::AppError::Metadata { reason: e.to_string() })?;

    let tag = match tagged.primary_tag() {
        Some(t) => t,
        None => return Ok(None),
    };

    let pic = match tag.pictures().first() {
        Some(p) => p,
        None => return Ok(None),
    };
    Ok(Some(CoverArt {
        data: pic.data().to_vec(),
        mime: pic
            .mime_type()
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "image/jpeg".to_string()),
    }))
}

/// Embed `cover` into `path`, replacing any existing picture.
pub fn write_cover(path: &Path, cover: &CoverArt) -> Result<()> {
    let mut tagged = Probe::open(path)
        .map_err(|e| musicglass_core::AppError::Metadata { reason: e.to_string() })?
        .read()
        .map_err(|e| musicglass_core::AppError::Metadata { reason: e.to_string() })?;

    let tag = match tagged.primary_tag_mut() {
        Some(t) => t,
        None => {
            return Err(musicglass_core::AppError::Metadata {
                reason: "no writable primary tag found".to_string(),
            })
        }
    };

    let mime = MimeType::from_str(&cover.mime);
    let pic = Picture::unchecked(cover.data.clone())
        .pic_type(lofty::picture::PictureType::CoverFront)
        .mime_type(mime)
        .build();
    tag.set_picture(0, pic);

    tagged
        .save_to_path(path, lofty::config::WriteOptions::default())
        .map_err(|e| musicglass_core::AppError::Metadata { reason: e.to_string() })?;
    Ok(())
}
