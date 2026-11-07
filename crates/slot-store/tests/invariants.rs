//! Slot²: facts the store keeps in more than one place, checked against each other (the
//! structure pass, HANDOFF.md). A setting is a field, a line written and a line read; a
//! system's setting is a field per system; a key is spelled once for writing and once
//! for reading. These tests fail when one of them is left out.

use slot_store::controls::{Input, SystemControls};
use slot_store::games::{read_game, write_game, GameSettings, Screen};
use slot_store::{
    read_slot_state, write_slot_state, Appearance, BezelPick, Binding, Blending, Core, Device,
    GbColors, GbPalette, Platform, Recent, Scale, Shader, Shortcut, SlotState, Stretch,
    WallpaperPick, Zone, FF_SPEEDS, LID_OFF_MIN,
};

/// The first of `choices` that isn't `now`.
fn other<T: PartialEq + Copy>(now: T, choices: &[T]) -> T {
    *choices.iter().find(|c| **c != now).expect("another value")
}

/// A state with every field away from its default, written out in full: a field added to
/// `SlotState` doesn't compile here until it is given a value, and then the round trip
/// below fails until it is written and read.
fn every_field_changed() -> SlotState {
    let d = SlotState::default();
    let recent = |p, stem: &str| Recent {
        platform: p,
        stem: stem.into(),
    };
    SlotState {
        cart: Some("Tetris".into()),
        cart_platform: Some(Platform::Gb),
        brightness: other(d.brightness, &[1, 2]),
        dim: other(d.dim, &[1, 2]),
        blue_light: other(d.blue_light, &[1, 2]),
        saturation: other(d.saturation, &[1, 2]),
        contrast: other(d.contrast, &[1, 2]),
        volume: other(d.volume, &[3, 4]),
        volume_hp: other(d.volume_hp, &[5, 6]),
        muted: !d.muted,
        muted_hp: !d.muted_hp,
        clock_set: !d.clock_set,
        zone: other(d.zone, &Zone::ALL),
        clock_24h: !d.clock_24h,
        network_time: !d.network_time,
        rumble: !d.rumble,
        sprite_limit: !d.sprite_limit,
        ff_speed: other(d.ff_speed, &FF_SPEEDS),
        ff_sound: !d.ff_sound,
        colour_correction_gba: !d.colour_correction_gba,
        colour_correction_gb: !d.colour_correction_gb,
        colour_correction_gbc: !d.colour_correction_gbc,
        shader_gba: other(d.shader_gba, &Shader::GBA),
        shader_gb: other(d.shader_gb, &Shader::GB),
        shader_gbc: other(d.shader_gbc, &Shader::GB),
        shader_ws: other(d.shader_ws, &Shader::WS),
        shader_lynx: other(d.shader_lynx, &Shader::LYNX),
        shader_gg: other(d.shader_gg, &Shader::GG),
        shader_ngp: other(d.shader_ngp, &Shader::NGP),
        shader_tg16: other(d.shader_tg16, &Shader::TG16),
        auto_load: !d.auto_load,
        eject_save: !d.eject_save,
        rewind: !d.rewind,
        gb_palettes: !d.gb_palettes,
        gb_palette: other(d.gb_palette, &GbPalette::all().collect::<Vec<_>>()),
        appearance: other(d.appearance, &[Appearance::Dark, Appearance::Light]),
        wallpaper: WallpaperPick::Random,
        wallpaper_tint: d.wallpaper_tint.step(true),
        show_recents: !d.show_recents,
        show_titles: !d.show_titles,
        bespoke_slot: !d.bespoke_slot,
        wifi_off: !d.wifi_off,
        gb_core: other(d.gb_core, &[Core::Mgba, Core::Gambatte]),
        lid_off_min: Some(other(d.lid_off_min.unwrap_or(0), &LID_OFF_MIN)),
        bezel_gba: BezelPick::None,
        bezel_gb: BezelPick::None,
        bezel_gbc: BezelPick::None,
        bezel_ws: BezelPick::Default,
        bezel_lynx: BezelPick::Default,
        bezel_gg: BezelPick::Default,
        bezel_ngp: BezelPick::Default,
        bezel_tg16: BezelPick::Default,
        scale_gba: other(d.scale_gba, &[Scale::Integer, Scale::Fit]),
        scale_gb: other(d.scale_gb, &[Scale::Integer, Scale::Fit]),
        scale_gbc: other(d.scale_gbc, &[Scale::Integer, Scale::Fit]),
        stretch_gb: other(d.stretch_gb, &[Stretch::Classic, Stretch::FullScreen]),
        stretch_gbc: other(d.stretch_gbc, &[Stretch::Classic, Stretch::FullScreen]),
        stretched_gb: !d.stretched_gb,
        stretched_gbc: !d.stretched_gbc,
        scale_gg: other(d.scale_gg, &[Scale::Integer, Scale::Fit]),
        scale_ngp: other(d.scale_ngp, &[Scale::Integer, Scale::Fit]),
        stretched_gg: !d.stretched_gg,
        stretched_ngp: !d.stretched_ngp,
        scale_tg16: other(d.scale_tg16, &[Scale::Integer, Scale::Fit]),
        stretched_tg16: !d.stretched_tg16,
        scale_ws: other(d.scale_ws, &[Scale::Integer, Scale::Fit]),
        scale_lynx: other(d.scale_lynx, &[Scale::Integer, Scale::Fit]),
        stretched_ws: !d.stretched_ws,
        stretched_lynx: !d.stretched_lynx,
        palette_ws: "swancrystal".into(),
        palette_tg16: "Composite".into(),
        blending_gba: other(d.blending_gba, &Blending::MGBA),
        blending_gb: other(d.blending_gb, &Blending::MGBA),
        blending_gbc: other(d.blending_gbc, &Blending::MGBA),
        recents: vec![
            recent(Platform::Gba, "Emerald"),
            recent(Platform::Ws, "Klonoa"),
        ],
        favorites: vec![recent(Platform::Gb, "Tetris")],
        shelf: Some("gba".into()),
        shelf_cart: Some(recent(Platform::Gba, "Emerald")),
    }
}

#[test]
fn every_slot_state_field_is_written_and_read() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("Config")).unwrap();
    let s = every_field_changed();
    write_slot_state(d.path(), &s).unwrap();
    assert_eq!(read_slot_state(d.path()), s);
}

#[test]
fn each_systems_screen_sets_and_reads_back_as_itself() {
    let changed = every_field_changed();
    for platform in Platform::ALL {
        let mut s = SlotState::default();
        let want: Screen = changed.screen(platform);
        s.set_screen(platform, &want);
        assert_eq!(s.screen(platform), want, "{platform:?}");
        // and touches no other system's
        for other in Platform::ALL {
            if other.hardware() != platform.hardware() {
                let mut alone = SlotState::default();
                alone.set_gb_colors(want.palette);
                assert_eq!(
                    s.screen(other),
                    alone.screen(other),
                    "setting {platform:?}'s screen moved {other:?}'s"
                );
            }
        }
    }
}

#[test]
fn a_game_with_its_own_everything_reads_back_as_itself() {
    let d = tempfile::tempdir().unwrap();
    for platform in [Platform::Gb, Platform::Gba, Platform::Ws, Platform::Tg16] {
        let mut game = GameSettings::new(platform, "Game");
        let changed = every_field_changed();
        let system = SlotState::default().screen(platform);
        game.screen
            .record(platform, &system, &changed.screen(platform));
        game.play
            .record(&SlotState::default().play(), &changed.play());
        game.shortcuts.set(Shortcut::SaveState, None);
        game.shortcuts
            .set(Shortcut::FastForward, Binding::plain(Device::R3));
        let mut controls = SystemControls::default_for(platform);
        controls.set_row(Input::Start, [None; 3]);
        game.controls
            .record(platform, &SystemControls::default_for(platform), &controls);
        if platform == Platform::Ws {
            game.ws_portrait = Some(true);
        }
        write_game(d.path(), &game).unwrap();
        assert_eq!(read_game(d.path(), platform, "Game"), game, "{platform:?}");
    }
}

#[test]
fn the_names_a_file_uses_are_each_one_thing() {
    fn unique(what: &str, keys: Vec<String>) {
        let mut seen = std::collections::HashSet::new();
        for k in &keys {
            assert!(seen.insert(k), "{what}: {k} is used twice");
        }
    }
    unique(
        "shortcuts",
        Shortcut::ALL.iter().map(|s| format!("{s:?}")).collect(),
    );
    unique(
        "inputs",
        Input::ALL.iter().map(|i| i.key().into()).collect(),
    );
    unique(
        "platforms",
        Platform::ALL
            .iter()
            .map(|p| slot_store::controls::system_key(*p).to_string() + &format!("{p:?}"))
            .collect(),
    );
    // A system's palette keys are its core's, and every Game Boy palette has its name.
    for platform in [Platform::Ws, Platform::Tg16] {
        unique(
            "core palettes",
            slot_store::core_palette::core_palettes(platform)
                .iter()
                .map(|(k, _)| k.to_string())
                .collect(),
        );
    }
    unique(
        "gb colours",
        GbColors::all().map(|c| c.core_name().to_string()).collect(),
    );
}

#[test]
fn every_feature_has_shortcuts_and_a_system_that_has_it() {
    use slot_store::features::{has_feature, Feature};
    use slot_store::shortcuts::feature_of;
    for feature in Feature::ALL {
        assert!(
            Shortcut::ALL
                .into_iter()
                .any(|s| feature_of(s) == Some(feature)),
            "{feature:?} has no shortcuts"
        );
        assert!(
            Platform::ALL.into_iter().any(|p| has_feature(p, feature)),
            "no system has {feature:?}"
        );
    }
}
