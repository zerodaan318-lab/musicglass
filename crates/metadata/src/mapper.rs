//! Metadata mapping & loss tracking (task book §21).
//!
//! When a target format cannot represent every field of the Unified Metadata,
//! we must record the loss rather than silently drop it. `lost_fields` lets the
//! UI warn the user exactly which tags were not carried over.

use musicglass_core::{Format, Metadata};
use std::collections::BTreeSet;

/// Fields that a given output format cannot store in its standard tag.
///
/// This is a conservative, format-knowledge-based list. WAV (RIFF INFO) is the
/// most limited; MP3/FLAC/M4A/OGG/OPUS cover the vast majority of fields.
pub fn unsupported_fields(format: Format, meta: &Metadata) -> Vec<String> {
    let mut lost = BTreeSet::new();

    // Fields present in the source but with no standard home in the target.
    if matches!(format, Format::Wav) {
        // RIFF INFO is sparse: no native lyrics, bpm, arranger, lyricist, disc.
        if meta.lyrics.is_some() {
            lost.insert("Lyrics".into());
        }
        if meta.bpm.is_some() {
            lost.insert("BPM".into());
        }
        if meta.arranger.is_some() {
            lost.insert("Arranger".into());
        }
        if meta.lyricist.is_some() {
            lost.insert("Lyricist".into());
        }
        if meta.disc_number.is_some() || meta.disc_total.is_some() {
            lost.insert("Disc".into());
        }
    }

    lost.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use musicglass_core::Metadata;

    #[test]
    fn wav_drops_lyrics_and_bpm() {
        let mut m = Metadata::default();
        m.lyrics = Some("la la".into());
        m.bpm = Some(120.0);
        m.title = Some("x".into());
        let lost = unsupported_fields(Format::Wav, &m);
        assert!(lost.contains(&"Lyrics".to_string()));
        assert!(lost.contains(&"BPM".to_string()));
        // title is supported everywhere
        assert!(!lost.contains(&"Title".to_string()));
    }

    #[test]
    fn flac_keeps_everything() {
        let m = Metadata::default();
        let lost = unsupported_fields(Format::Flac, &m);
        assert!(lost.is_empty());
    }
}
