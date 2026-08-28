//! Metadata reading/writing (task book §20, §21).
//!
//! Phase 3 will implement readers/writers per format and the MetadataMapper.
//! This crate currently defines the module boundaries and re-exports the
//! Unified Metadata model from `musicglass-core`.

pub use musicglass_core::Metadata;

/// Maps a [`Metadata`] into a target format's tag set.
///
/// Implemented in Phase 3. Returns the list of fields that could not be
/// represented in the target (so the UI can warn, never silently drop).
pub fn map_to_target(_meta: &Metadata, _target: &str) -> Vec<String> {
    Vec::new()
}
