//! Unified Metadata Model (task book §20).
//!
//! Every format reader maps into this struct; every writer maps out of it.
//! Fields are `Option` because not all formats carry all tags. Lossy mapping
//! (a field with no target) must be recorded, never silently dropped (§21).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub composer: Option<String>,
    pub lyricist: Option<String>,
    pub arranger: Option<String>,
    pub genre: Option<String>,
    pub year: Option<String>,
    pub track_number: Option<u32>,
    pub track_total: Option<u32>,
    pub disc_number: Option<u32>,
    pub disc_total: Option<u32>,
    pub comment: Option<String>,
    pub lyrics: Option<String>,
    pub bpm: Option<f32>,
    /// Extended/unmapped tags kept verbatim (key = original tag name).
    #[serde(default)]
    pub extra: BTreeMap<String, String>,
}

impl Metadata {
    /// Fields that could not be written to a target format.
    /// The caller records these so the UI can warn the user (§21).
    pub fn lost_fields(&self, supported: &[&str]) -> Vec<String> {
        let mut lost = Vec::new();
        let has = |f: &Option<String>| f.is_some();
        if has(&self.title) && !supported.contains(&"title") { lost.push("title".into()); }
        if has(&self.artist) && !supported.contains(&"artist") { lost.push("artist".into()); }
        if has(&self.album) && !supported.contains(&"album") { lost.push("album".into()); }
        if has(&self.album_artist) && !supported.contains(&"album_artist") { lost.push("album_artist".into()); }
        if has(&self.composer) && !supported.contains(&"composer") { lost.push("composer".into()); }
        if has(&self.lyricist) && !supported.contains(&"lyricist") { lost.push("lyricist".into()); }
        if has(&self.arranger) && !supported.contains(&"arranger") { lost.push("arranger".into()); }
        if has(&self.genre) && !supported.contains(&"genre") { lost.push("genre".into()); }
        if self.year.is_some() && !supported.contains(&"year") { lost.push("year".into()); }
        if self.track_number.is_some() && !supported.contains(&"track_number") { lost.push("track_number".into()); }
        if self.lyrics.is_some() && !supported.contains(&"lyrics") { lost.push("lyrics".into()); }
        for k in self.extra.keys() {
            if !supported.contains(&k.as_str()) {
                lost.push(format!("extra:{}", k));
            }
        }
        lost
    }

    /// Number of populated core fields (for UI display).
    pub fn populated_count(&self) -> usize {
        [
            &self.title, &self.artist, &self.album, &self.album_artist,
            &self.composer, &self.lyricist, &self.arranger, &self.genre,
            &self.year,
        ]
        .iter()
        .filter(|f| f.is_some())
        .count()
            + [self.track_number, self.track_total, self.disc_number, self.disc_total]
                .iter()
                .filter(|f| f.is_some())
                .count()
            + if self.lyrics.is_some() { 1 } else { 0 }
            + self.extra.len()
    }
}
