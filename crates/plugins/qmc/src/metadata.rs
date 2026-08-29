//! Inner-format detection for decrypted QMC audio.

/// Map a QMC family extension to the expected inner codec string.
/// References (task book §9): qmc0/qmc2/qmc3 = MP3, qmcflac/mflac = FLAC,
/// qmcogg/mgg = OGG/other. Returns the lofty/ffmpeg-friendly codec name.
pub fn format_from_ext(path: &std::path::Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "qmcflac" | "mflac" | "mflac0" => "flac",
        "mgg" | "mgg0" | "mgg1" | "qmcogg" => "ogg",
        // qmc / qmc0 / qmc2 / qmc3 are MP3 containers
        _ => "mp3",
    }
}

/// Sniff the inner audio codec from its leading magic bytes after decryption.
pub fn detect_by_magic(audio: &[u8]) -> &'static str {
    if audio.len() >= 4 {
        if &audio[0..4] == b"fLaC" {
            return "flac";
        }
        if &audio[0..4] == b"ID3 " || (audio[0] == 0xFF && (audio[1] & 0xE0) == 0xE0) {
            return "mp3";
        }
        if &audio[4..8] == b"ftyp" {
            return "m4a";
        }
        if &audio[0..4] == b"OggS" {
            return "ogg";
        }
        if &audio[0..4] == b"RIFF" {
            return "wav";
        }
    }
    "unknown"
}
