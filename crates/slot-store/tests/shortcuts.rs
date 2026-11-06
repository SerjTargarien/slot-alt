use slot_store::shortcuts::{
    has, own, read_system_shortcuts, write_system_shortcuts, AllShortcuts, SystemShortcuts,
};
use slot_store::{Binding, Device, Modifier, Platform, Shortcut, ShortcutKeys};

#[test]
fn each_system_has_the_general_shortcuts_and_only_its_own() {
    use Shortcut::*;
    assert_eq!(
        own(Platform::Gb),
        [
            ColorCorrection,
            PaletteBack,
            PaletteNext,
            StretchOn,
            StretchOff
        ]
    );
    for platform in Platform::ALL {
        assert!(has(platform, SaveState), "{platform:?}");
    }
    assert!(has(Platform::Gbav, SunlightUp));
    assert!(!has(Platform::Gbc, PaletteNext));
    assert!(!has(Platform::Nds, StretchOn));
    assert!(own(Platform::Nds).is_empty());
    assert!(own(Platform::Gba).contains(&SunlightDown));
}

#[test]
fn sunlight_ships_unbound() {
    let keys = ShortcutKeys::default();
    assert_eq!(keys.get(Shortcut::SunlightUp), None);
    assert_eq!(keys.get(Shortcut::SunlightDown), None);
}

#[test]
fn a_system_file_holds_only_the_rows_that_differ_and_reads_back() {
    let d = tempfile::tempdir().unwrap();
    let general = ShortcutKeys::default();
    let mut keys = SystemShortcuts::from(&general);
    let file = d.path().join("Config/shortcuts/gb.txt");
    write_system_shortcuts(d.path(), Platform::Gb, &keys, &general).unwrap();
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "");
    keys.set(
        Shortcut::PaletteNext,
        Binding::with(Modifier::Menu, Device::R3),
    );
    keys.set(Shortcut::StretchOn, None);
    write_system_shortcuts(d.path(), Platform::Gb, &keys, &general).unwrap();
    let text = std::fs::read_to_string(&file).unwrap();
    assert_eq!(text.lines().count(), 2, "{text}");
    assert_eq!(
        read_system_shortcuts(d.path(), Platform::Gb, &general),
        keys
    );
    assert_eq!(
        read_system_shortcuts(d.path(), Platform::Gbc, &general),
        SystemShortcuts::from(&general)
    );
}

#[test]
fn a_system_file_ignores_shortcuts_that_are_not_its_own() {
    let d = tempfile::tempdir().unwrap();
    let dir = d.path().join("Config/shortcuts");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("gba.txt"), "palette_next=none\nsave_state=none\n").unwrap();
    let general = ShortcutKeys::default();
    assert_eq!(
        read_system_shortcuts(d.path(), Platform::Gba, &general),
        SystemShortcuts::from(&general)
    );
}

#[test]
fn the_gba_on_a_frame_shares_the_gbas_keys() {
    let general = ShortcutKeys::default();
    let mut all = AllShortcuts::from(&general);
    all.get_mut(Platform::Gbav)
        .set(Shortcut::SunlightUp, Binding::plain(Device::R3));
    assert_eq!(
        all.get(Platform::Gba).get(Shortcut::SunlightUp),
        Binding::plain(Device::R3)
    );
    assert_eq!(all.get(Platform::Gb).get(Shortcut::SunlightUp), None);
}
