//! Slot²: what each system can do that a shortcut drives (design/shortcuts.md), in one
//! place, so a system that gains a feature gains its shortcuts with it.

use crate::core_palette::core_palettes;
use crate::platform::Platform;

/// A system feature with shortcuts of its own.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Feature {
    /// The core's colour correction (the Color Correction row).
    ColourCorrection,
    /// Palettes to step through: the Game Boy's (`GbColors`), the WonderSwan's and the
    /// TurboGrafx-16's (their cores', `core_palette`).
    Palette,
    /// Stretch On or Off (the Stretch row on a square panel).
    Stretch,
    /// The GBA cartridges' sunlight sensor (Boktai).
    Sunlight,
}

pub fn has_feature(platform: Platform, feature: Feature) -> bool {
    use Platform::*;
    match feature {
        Feature::ColourCorrection => matches!(platform.hardware(), Gb | Gbc | Gba),
        Feature::Palette => platform == Gb || !core_palettes(platform).is_empty(),
        Feature::Stretch => !matches!(platform.hardware(), Gba | Nds),
        Feature::Sunlight => platform.hardware() == Gba,
    }
}

/// Whether a system's keys for a feature's shortcuts start from the general keys in
/// `shortcuts.txt`. They do on the systems that always had the feature; a system that
/// gains one later starts with its shortcuts unbound.
pub fn starts_bound(platform: Platform, feature: Feature) -> bool {
    match feature {
        Feature::Palette => platform == Platform::Gb,
        _ => true,
    }
}
