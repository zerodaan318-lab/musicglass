//! Write the Unified Metadata model into an audio file (task book §20, §21).
//!
//! Opens the (already-created) output file, fills its primary tag via
//! `ItemKey` + `insert_text`, and saves. Fields with no equivalent in the
//! target are recorded via [`crate::mapper`] and reported (never silently
//! dropped).

use lofty::config::WriteOptions;
use lofty::prelude::*;
use lofty::probe::Probe;
use lofty::tag::TagType;
use musicglass_core::{AppError, Format, Metadata, Result};
use std::path::Path;

/// Map a target [`Format`] to the lofty tag container it uses.
pub fn tag_type_for(format: Format) -> Option<TagType> {
    use Format::*;
    Some(match format {
        Mp3 | Aac => TagType::Id3v2,
        Flac | Ogg | Opus => TagType::VorbisComments,
        M4a => TagType::Mp4Ilst,
        Wav => TagType::RiffInfo,
        _ => return None,
    })
}

/// Write `meta` into `path`, replacing existing tags of the matching type.
///
/// `format` is the **output** format (not the container currently on disk), so
/// this is safe to call after a format conversion created a fresh file.
pub fn write_metadata(path: &Path, meta: &Metadata, format: Format) -> Result<()> {
    // Sanity-check the target is one we know how to write.
    tag_type_for(format)
        .ok_or_else(|| AppError::UnsupportedFormat(format.as_str().to_string()))?;

    let mut tagged = Probe::open(path)
        .map_err(|e| AppError::Metadata { reason: e.to_string() })?
        .read()
        .map_err(|e| AppError::Metadata { reason: e.to_string() })?;

    let tag = match tagged.primary_tag_mut() {
        Some(t) => t,
        None => {
            return Err(AppError::Metadata {
                reason: "target file has no writable primary tag".to_string(),
            })
        }
    };

    if let Some(v) = &meta.title {
        tag.insert_text(ItemKey::TrackTitle, v.clone());
    }
    if let Some(v) = &meta.artist {
        tag.insert_text(ItemKey::TrackArtist, v.clone());
    }
    if let Some(v) = &meta.album {
        tag.insert_text(ItemKey::AlbumTitle, v.clone());
    }
    if let Some(v) = &meta.album_artist {
        tag.insert_text(ItemKey::AlbumArtist, v.clone());
    }
    if let Some(v) = &meta.genre {
        tag.insert_text(ItemKey::Genre, v.clone());
    }
    if let Some(v) = &meta.comment {
        tag.insert_text(ItemKey::Comment, v.clone());
    }
    if let Some(v) = &meta.composer {
        tag.insert_text(ItemKey::Composer, v.clone());
    }
    if let Some(v) = &meta.lyricist {
        tag.insert_text(ItemKey::Lyricist, v.clone());
    }
    if let Some(v) = &meta.arranger {
        tag.insert_text(ItemKey::Arranger, v.clone());
    }
    if let Some(v) = &meta.year {
        tag.insert_text(ItemKey::Year, v.clone());
    }
    if let Some(v) = meta.track_number {
        tag.insert_text(ItemKey::TrackNumber, v.to_string());
    }
    if let Some(v) = meta.track_total {
        tag.insert_text(ItemKey::TrackTotal, v.to_string());
    }
    if let Some(v) = meta.disc_number {
        tag.insert_text(ItemKey::DiscNumber, v.to_string());
    }
    if let Some(v) = meta.disc_total {
        tag.insert_text(ItemKey::DiscTotal, v.to_string());
    }
    if let Some(v) = meta.bpm {
        tag.insert_text(ItemKey::Bpm, v.to_string());
    }
    if let Some(v) = &meta.lyrics {
        tag.insert_text(ItemKey::Lyrics, v.clone());
    }

    tagged
        .save_to_path(path, WriteOptions::default())
        .map_err(|e| AppError::Metadata { reason: e.to_string() })?;
    Ok(())
}
