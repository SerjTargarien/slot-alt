use slot_store::games::{
    game_file, read_game, write_game, GameSettings, ScreenOverrides, ScreenSetting,
};
use slot_store::{
    BezelPick, Binding, Device, GbColors, Platform, Scale, Shader, Shortcut, SlotState,
};

#[test]
fn a_game_without_its_own_values_is_its_systems_screen() {
    let s = SlotState::default();
    let system = s.screen(Platform::Gb);
    assert_eq!(ScreenOverrides::default().apply(&system), system);
}

#[test]
fn a_screen_set_on_one_system_leaves_the_others_alone() {
    let mut s = SlotState::default();
    let before = s.screen(Platform::Ws);
    let mut gb = s.screen(Platform::Gb);
    gb.shader = Shader::Off;
    gb.scale = Scale::Fit;
    gb.stretched = !gb.stretched;
    s.set_screen(Platform::Gb, &gb);
    assert_eq!(s.screen(Platform::Gb), gb);
    assert_eq!(s.screen(Platform::Ws), before);
    assert_eq!(s.screen(Platform::Gbav), s.screen(Platform::Gba));
}

#[test]
fn only_what_differs_from_the_system_is_the_games_own() {
    let system = SlotState::default().screen(Platform::Gb);
    let mut now = system.clone();
    now.shader = Shader::Off;
    now.bezel = BezelPick::None;
    let mut own = ScreenOverrides::default();
    own.record(Platform::Gb, &system, &now);
    assert_eq!(own.shader, Some(Shader::Off));
    assert_eq!(own.bezel, Some(BezelPick::None));
    assert_eq!((own.scale, own.palette, own.stretched), (None, None, None));
    assert_eq!(own.apply(&system), now);
    own.record(Platform::Gb, &system, &system);
    assert_eq!(own, ScreenOverrides::default());
}

#[test]
fn a_games_file_reads_back_and_goes_once_it_holds_nothing() {
    let d = tempfile::tempdir().unwrap();
    let mut game = GameSettings::new(Platform::Gbav, "Emerald");
    game.screen.shader = Some(Shader::Lcd3x);
    game.screen.palette = Some(GbColors::Auto);
    game.screen.stretched = Some(true);
    write_game(d.path(), &game).unwrap();
    let file = game_file(d.path(), Platform::Gba, "Emerald");
    assert!(file.ends_with("Config/games/gba/Emerald.txt"), "{file:?}");
    let back = read_game(d.path(), Platform::Gba, "Emerald");
    assert_eq!(back.screen, game.screen);
    for s in [
        ScreenSetting::Shader,
        ScreenSetting::Palette,
        ScreenSetting::Stretched,
    ] {
        game.screen.clear(s);
    }
    write_game(d.path(), &game).unwrap();
    assert!(!file.exists());
    write_game(d.path(), &game).unwrap();
}

#[test]
fn a_games_gameplay_and_keys_read_back() {
    let d = tempfile::tempdir().unwrap();
    let mut game = GameSettings::new(Platform::Gb, "Tetris");
    game.play.ff_speed = Some(12);
    game.play.eject_save = Some(false);
    game.shortcuts.set(Shortcut::SaveState, None);
    game.shortcuts
        .set(Shortcut::PaletteNext, Binding::plain(Device::R3));
    write_game(d.path(), &game).unwrap();
    let text = std::fs::read_to_string(game_file(d.path(), Platform::Gb, "Tetris")).unwrap();
    assert!(text.contains("shortcut.save_state=none\n"), "{text}");
    let back = read_game(d.path(), Platform::Gb, "Tetris");
    assert_eq!(back.play, game.play);
    assert_eq!(back.shortcuts, game.shortcuts);
    assert_eq!(
        back.shortcuts.get(Shortcut::LoadState),
        None,
        "follows the shelf"
    );
}

#[test]
fn a_games_own_controls_rows_read_back_and_others_are_dropped() {
    use slot_store::controls::{Input, SystemControls};
    let d = tempfile::tempdir().unwrap();
    let system = SystemControls::default_for(Platform::Gb);
    let mut now = system;
    now.add(Input::A, Binding::plain(Device::Y).unwrap());
    now.set_row(Input::Up, [None; 3]);
    let mut game = GameSettings::new(Platform::Gb, "Tetris");
    game.controls.record(Platform::Gb, &system, &now);
    assert!(game.controls.has(Input::A) && game.controls.has(Input::Up));
    assert!(!game.controls.has(Input::B));
    write_game(d.path(), &game).unwrap();
    let file = game_file(d.path(), Platform::Gb, "Tetris");
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("control.up=none\n"), "{text}");
    // A Game Boy has no L: a line for it means nothing here.
    std::fs::write(&file, format!("{text}control.l=r1\n")).unwrap();
    let back = read_game(d.path(), Platform::Gb, "Tetris");
    assert_eq!(back.controls, game.controls);
    assert_eq!(back.controls.apply(&system), now);
}

#[test]
fn a_wonderswan_games_orientation_reads_back() {
    let d = tempfile::tempdir().unwrap();
    let mut game = GameSettings::new(Platform::Ws, "Klonoa");
    game.ws_portrait = Some(true);
    write_game(d.path(), &game).unwrap();
    let file = game_file(d.path(), Platform::Ws, "Klonoa");
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "ws_orientation=portrait\n"
    );
    assert_eq!(
        read_game(d.path(), Platform::Ws, "Klonoa").ws_portrait,
        Some(true)
    );
    std::fs::write(&file, "ws_orientation=sideways\n").unwrap();
    assert_eq!(
        read_game(d.path(), Platform::Ws, "Klonoa").ws_portrait,
        None
    );
}
