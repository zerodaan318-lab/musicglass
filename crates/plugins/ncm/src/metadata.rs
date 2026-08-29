//! Map the decrypted NCM metadata JSON into the unified [`Metadata`] model.

use musicglass_core::Metadata;
use serde_json::Value;

/// Parse NCM metadata JSON (`music:{...}`) into the unified model.
///
/// NCM JSON fields (documented in `docs/FORMAT_RESEARCH_NCM.md`):
/// `musicName`, `artist` (array of [name, id]), `album`, `albumId`,
/// `albumPic` (cover URL), `bitrate`, `duration`, `alias`, `transNames`,
/// `format`, `comment` (sometimes), `lyric` (sometimes).
pub fn parse_metadata(json: &[u8]) -> Result<Metadata, musicglass_core::AppError> {
    let v: Value = serde_json::from_slice(json)
        .map_err(|e| musicglass_core::AppError::Metadata {
            reason: format!("NCM metadata JSON parse failed: {e}"),
        })?;

    let mut m = Metadata::default();

    m.title = v["musicName"].as_str().map(|s| s.to_string());

    // artist: array of [name, id] pairs → join names with " / "
    if let Some(arr) = v["artist"].as_array() {
        let names: Vec<String> = arr
            .iter()
            .filter_map(|a| a.as_array())
            .filter_map(|inner| inner.first())
            .filter_map(|n| n.as_str())
            .map(|s| s.to_string())
            .collect();
        if !names.is_empty() {
            m.artist = Some(names.join(" / "));
        }
    }

    m.album = v["album"].as_str().map(|s| s.to_string());
    // albumId kept as extra since Metadata has no album_id field.
    if let Some(id) = v["albumId"].as_u64() {
        m.extra.insert("album_id".into(), id.to_string());
    }

    // Cover URL is useful even though binary cover is extracted separately.
    if let Some(url) = v["albumPic"].as_str() {
        m.extra.insert("cover_url".into(), url.to_string());
    }

    if let Some(b) = v["bitrate"].as_u64() {
        m.extra.insert("bitrate".into(), format!("{} kbps", b / 1000));
    }
    if let Some(d) = v["duration"].as_u64() {
        m.extra.insert("duration_ms".into(), d.to_string());
    }
    if let Some(f) = v["format"].as_str() {
        m.extra.insert("inner_format".into(), f.to_string());
    }

    // Lyrics if present in metadata.
    if let Some(lyric) = v["lyric"].as_str() {
        if !lyric.is_empty() {
            m.lyrics = Some(lyric.to_string());
        }
    }
    // Comment if present.
    if let Some(c) = v["comment"].as_str() {
        if !c.is_empty() {
            m.comment = Some(c.to_string());
        }
    }

    Ok(m)
}

/// Detect the inner codec/format from the metadata `format` field or the
/// audio magic bytes. Returns a [`musicglass_core::Format`] as_str string.
pub fn detect_inner_format(json: &[u8], audio: &[u8]) -> String {
    if let Ok(v) = serde_json::from_slice::<Value>(json) {
        if let Some(f) = v["format"].as_str() {
            return f.to_string();
        }
    }
    detect_by_magic(audio)
}

/// Sniff the inner audio codec from its leading magic bytes.
pub fn detect_by_magic(audio: &[u8]) -> String {
    if audio.len() >= 4 {
        if &audio[0..4] == b"fLaC" {
            return "flac".into();
        }
        if &audio[0..4] == b"ID3 " || audio[0] == 0xFF && (audio[1] & 0xE0) == 0xE0 {
            return "mp3".into();
        }
        if &audio[4..8] == b"ftyp" {
            return "m4a".into();
        }
        if &audio[0..4] == b"OggS" {
            return "ogg".into();
        }
        if &audio[0..4] == b"RIFF" {
            return "wav".into();
        }
    }
    "unknown".into()
}
