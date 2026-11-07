use slot_store::core_palette::{
    core_palette, core_palette_label, core_palette_option, core_palettes, step_core_palette,
};
use slot_store::games::{read_game, write_game, GameSettings};
use slot_store::{read_slot_state, write_slot_state, Platform, SlotState};

#[test]
fn the_wonderswan_and_turbografx_have_palettes_of_their_cores() {
    assert_eq!(core_palettes(Platform::Ws).len(), 27);
    assert_eq!(
        core_palettes(Platform::Tg16),
        [("RGB", "RGB"), ("Composite", "Composite")]
    );
    assert!(
        core_palettes(Platform::Gb).is_empty(),
        "the Game Boy's are GbColors"
    );
    assert_eq!(
        core_palette_option(Platform::Ws),
        Some("wswan_mono_palette")
    );
    assert_eq!(
        core_palette_option(Platform::Tg16),
        Some("pce_fast_palette")
    );
    assert_eq!(
        core_palette_label(Platform::Ws, "swancrystal"),
        "SwanCrystal"
    );
    assert_eq!(step_core_palette(Platform::Tg16, "Composite", true), "RGB");
    assert_eq!(
        step_core_palette(Platform::Ws, "default", false),
        "virtual_boy"
    );
    assert_eq!(core_palette(Platform::Ws, "sideways"), "default");
}

#[test]
fn slot_state_keeps_each_systems_palette() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("Config")).unwrap();
    let s = SlotState::default();
    assert_eq!(
        (s.palette_ws.as_str(), s.palette_tg16.as_str()),
        ("default", "RGB")
    );
    let s = SlotState {
        palette_ws: "virtual_boy".into(),
        palette_tg16: "Composite".into(),
        ..SlotState::default()
    };
    write_slot_state(d.path(), &s).unwrap();
    let back = read_slot_state(d.path());
    assert_eq!(back.palette_ws, "virtual_boy");
    assert_eq!(back.palette_tg16, "Composite");
}

#[test]
fn a_games_own_palette_is_its_cores_on_a_wonderswan_and_gb_colors_on_a_game_boy() {
    let d = tempfile::tempdir().unwrap();
    let mut ws = GameSettings::new(Platform::Ws, "Gunpey");
    ws.screen.core_palette = Some("swancrystal".into());
    write_game(d.path(), &ws).unwrap();
    assert_eq!(
        read_game(d.path(), Platform::Ws, "Gunpey")
            .screen
            .core_palette
            .as_deref(),
        Some("swancrystal")
    );
    let file = slot_store::games::game_file(d.path(), Platform::Ws, "Gunpey");
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "palette=swancrystal\n"
    );
    std::fs::write(&file, "palette=GBC Dark Green →A\n").unwrap();
    assert_eq!(
        read_game(d.path(), Platform::Ws, "Gunpey")
            .screen
            .core_palette,
        None
    );
}
