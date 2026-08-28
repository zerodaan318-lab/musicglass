//! Proprietary format plugin registry (task book §7).
//!
//! The [`MusicContainer`](musicglass_core::MusicContainer) trait is defined in
//! `musicglass_core`. This crate wires the concrete plugin crates into a
//! [`Plugin`] enum so the desktop app selects a handler without writing
//! `if ncm { ... } else if qmc { ... }`.

pub use musicglass_core::MusicContainer;
pub use musicglass_plugins_ncm::NcmPlugin;
pub use musicglass_plugins_qmc::QmcPlugin;

/// Enum wrapper enabling dynamic plugin selection without `dyn` (the trait has
/// no `self`, so it is not `dyn`-compatible — selection is by enum match).
#[derive(Clone, Copy, Debug)]
pub enum Plugin {
    Ncm,
    Qmc,
}

impl Plugin {
    /// All built-in plugins in priority order.
    pub fn all() -> &'static [Plugin] {
        &[Plugin::Ncm, Plugin::Qmc]
    }

    /// Find the first plugin that claims the file.
    pub fn find(path: &std::path::Path) -> Option<Plugin> {
        Plugin::all().iter().find(|p| p.can_handle(path)).copied()
    }

    pub fn can_handle(&self, path: &std::path::Path) -> bool {
        match self {
            Plugin::Ncm => NcmPlugin::can_handle(path),
            Plugin::Qmc => QmcPlugin::can_handle(path),
        }
    }

    pub fn inspect(&self, path: &std::path::Path) -> musicglass_core::Result<musicglass_core::FileInfo> {
        match self {
            Plugin::Ncm => NcmPlugin::inspect(path),
            Plugin::Qmc => QmcPlugin::inspect(path),
        }
    }

    pub fn extract_audio(&self, path: &std::path::Path) -> musicglass_core::Result<musicglass_core::AudioStream> {
        match self {
            Plugin::Ncm => NcmPlugin::extract_audio(path),
            Plugin::Qmc => QmcPlugin::extract_audio(path),
        }
    }

    pub fn extract_metadata(&self, path: &std::path::Path) -> musicglass_core::Result<musicglass_core::Metadata> {
        match self {
            Plugin::Ncm => NcmPlugin::extract_metadata(path),
            Plugin::Qmc => QmcPlugin::extract_metadata(path),
        }
    }

    pub fn extract_cover(&self, path: &std::path::Path) -> musicglass_core::Result<Option<Vec<u8>>> {
        match self {
            Plugin::Ncm => NcmPlugin::extract_cover(path),
            Plugin::Qmc => QmcPlugin::extract_cover(path),
        }
    }
}
