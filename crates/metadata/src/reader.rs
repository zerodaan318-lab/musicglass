//! Read audio tags into the Unified Metadata model (task book §20).
//!
//! Uses `lofty`. NCM/QMC are handled by their plugins, which call into this
//! after extracting the inner standard audio.

use lofty::prelude::*;
use lofty::probe::Probe;
use musicglass_core::{Metadata, Result};
use std::path::Path;

/// Read all standard tags from a file into [`Metadata`].
pub fn read_metadata(path: &Path) -> Result<Metadata> {
    let tagged = Probe::open(path)
        .map_err(|e| musicglass_core::AppError::Metadata { reason: e.to_string() })?
        .read()
        .map_err(|e| musicglass_core::AppError::Metadata { reason: e.to_string() })?;

    let tag = match tagged.primary_tag() {
        Some(t) => t,
        None => return Ok(Metadata::default()),
    };

    let mut m = Metadata::default();

    m.title = tag.get_string(ItemKey::TrackTitle).map(String::from);
    m.artist = tag.get_string(ItemKey::TrackArtist).map(String::from);
    m.album = tag.get_string(ItemKey::AlbumTitle).map(String::from);
    m.album_artist = tag.get_string(ItemKey::AlbumArtist).map(String::from);
    m.genre = tag.get_string(ItemKey::Genre).map(String::from);
    m.comment = tag.get_string(ItemKey::Comment).map(String::from);
    m.composer = tag.get_string(ItemKey::Composer).map(String::from);
    m.lyricist = tag.get_string(ItemKey::Lyricist).map(String::from);
    m.arranger = tag.get_string(ItemKey::Arranger).map(String::from);
    m.year = tag
        .get_string(ItemKey::Year)
        .or_else(|| tag.get_string(ItemKey::ReleaseDate))
        .map(String::from);
    m.track_number = tag.get_string(ItemKey::TrackNumber).and_then(|s| s.parse::<u32>().ok());
    m.track_total = tag.get_string(ItemKey::TrackTotal).and_then(|s| s.parse::<u32>().ok());
    m.disc_number = tag.get_string(ItemKey::DiscNumber).and_then(|s| s.parse::<u32>().ok());
    m.disc_total = tag.get_string(ItemKey::DiscTotal).and_then(|s| s.parse::<u32>().ok());
    m.bpm = tag.get_string(ItemKey::Bpm).and_then(|s| s.parse::<f32>().ok());
    m.lyrics = tag.get_string(ItemKey::Lyrics).map(String::from);

    Ok(m)
}
