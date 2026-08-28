//! Metadata reading/writing (task book §20, §21, §22).
//!
//! - `reader` — read any supported format into Unified [`Metadata`]
//! - `writer` — write Unified [`Metadata`] into a target format
//! - `cover`  — read/embed cover art (JPEG/PNG/WEBP)
//! - `mapper` — format capability checks & loss tracking
//!
//! Phase 3 implements these on top of [`lofty`], which covers all Phase-1
//! output formats and carries cover/lyrics.

pub mod cover;
pub mod mapper;
pub mod reader;
pub mod writer;

pub use cover::{CoverArt, read_cover, write_cover};
pub use mapper::{unsupported_fields};
pub use reader::read_metadata;
pub use writer::{tag_type_for, write_metadata};
