//! Slot²: the palettes a core of its own draws a system in (design/core-options.md): the
//! WonderSwan's mono games (Beetle Cygne's `wswan_mono_palette`) and the TurboGrafx-16
//! (Beetle PCE Fast's `pce_fast_palette`). The Game Boy's are `GbColors`.

use crate::platform::Platform;

/// Each as the core spells it, and as the menu shows it.
pub const WS_PALETTES: [(&str, &str); 27] = [
    ("default", "Grayscale"),
    ("wonderswan", "WonderSwan"),
    ("wondeswan_color", "WonderSwan Color"),
    ("swancrystal", "SwanCrystal"),
    ("gb_dmg", "Game Boy DMG"),
    ("gb_pocket", "Game Boy Pocket"),
    ("gb_light", "Game Boy Light"),
    ("blossom_pink", "Blossom Pink"),
    ("bubbles_blue", "Bubbles Blue"),
    ("buttercup_green", "Buttercup Green"),
    ("digivice", "Digivice"),
    ("game_com", "Game.com"),
    ("gameking", "GameKing"),
    ("game_master", "Game Master"),
    ("golden_wild", "Golden Wild"),
    ("greenscale", "Greenscale"),
    ("hokage_orange", "Hokage Orange"),
    ("labo_fawn", "Labo Fawn"),
    // The core's "Legendary Super Saiyan" is too long for a toast.
    ("legendary_super_saiyan", "Super Saiyan"),
    ("microvision", "Microvision"),
    ("million_live_gold", "Million Live Gold"),
    ("odyssey_gold", "Odyssey Gold"),
    ("shiny_sky_blue", "Shiny Sky Blue"),
    ("slime_blue", "Slime Blue"),
    ("ti_83", "TI-83"),
    ("travel_wood", "Travel Wood"),
    ("virtual_boy", "Virtual Boy"),
];

pub const TG16_PALETTES: [(&str, &str); 2] = [("RGB", "RGB"), ("Composite", "Composite")];

/// The palettes `platform`'s core draws it in, the first its default; none for a system
/// without (the Game Boy's are `GbColors`).
pub fn core_palettes(platform: Platform) -> &'static [(&'static str, &'static str)] {
    match platform {
        Platform::Ws => &WS_PALETTES,
        Platform::Tg16 => &TG16_PALETTES,
        _ => &[],
    }
}

/// The core option a system's palette is set with.
pub fn core_palette_option(platform: Platform) -> Option<&'static str> {
    match platform {
        Platform::Ws => Some("wswan_mono_palette"),
        Platform::Tg16 => Some("pce_fast_palette"),
        _ => None,
    }
}

/// `key` as one of `platform`'s palettes, or its default when it isn't one.
pub fn core_palette(platform: Platform, key: &str) -> &'static str {
    let palettes = core_palettes(platform);
    palettes
        .iter()
        .find(|(k, _)| *k == key)
        .or(palettes.first())
        .map_or("", |(k, _)| k)
}

/// The menu's name for one of `platform`'s palettes.
pub fn core_palette_label(platform: Platform, key: &str) -> &'static str {
    core_palettes(platform)
        .iter()
        .find(|(k, _)| *k == key)
        .map_or("", |(_, label)| label)
}

/// The palette after (or before) `key` in `platform`'s, wrapping round.
pub fn step_core_palette(platform: Platform, key: &str, right: bool) -> &'static str {
    let palettes = core_palettes(platform);
    let n = palettes.len();
    if n == 0 {
        return "";
    }
    let at = palettes.iter().position(|(k, _)| *k == key).unwrap_or(0);
    palettes[if right {
        (at + 1) % n
    } else {
        (at + n - 1) % n
    }]
    .0
}
