//! Slot²: a game's own screen settings over its system's (design/game-overrides.md).

use crate::common;

use std::path::Path;

use common::{app_playing_in, tmp_root_with_carts, StubSnapshot};
use slot::app::App;
use slot_input::{Action, Btn, RawEvent};
use slot_store::games::{game_file, read_game};
use slot_store::{
    read_slot_state, write_slot_state, Core, Platform, Shader, Shortcut, SlotState, StateRing,
};
use slot_ui::QuickRow;

fn press(a: &mut App, btn: Btn) {
    a.apply(Action::GbaDown(btn));
    a.apply(Action::GbaUp(btn));
}

fn to_screen_row(a: &mut App, row: QuickRow) {
    a.apply(Action::QuickMenu);
    for step in [QuickRow::Settings, QuickRow::PlayScreen, row] {
        for _ in 0..16 {
            if a.in_game_menu() == Some(step) {
                break;
            }
            press(a, Btn::Down);
        }
        assert_eq!(a.in_game_menu(), Some(step), "never reached {step:?}");
        if step != row {
            press(a, Btn::A);
        }
    }
}

fn own_shader(root: &Path, stem: &str) -> Option<Shader> {
    read_game(root, Platform::Gba, stem).screen.shader
}

#[test]
fn an_in_game_screen_change_is_the_games_own() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let system = read_slot_state(d.path()).shader_gba;
    let mut a = app_playing_in(d.path(), "Emerald");
    assert_eq!(a.screen_shader(), system);
    to_screen_row(&mut a, QuickRow::Shader);
    assert!(!a.game_owns_row(QuickRow::Shader));
    press(&mut a, Btn::Right);
    let mine = a.screen_shader();
    assert_ne!(mine, system);
    assert!(a.game_owns_row(QuickRow::Shader));
    assert_eq!(own_shader(d.path(), "Emerald"), Some(mine));
    assert_eq!(
        read_slot_state(d.path()).shader_gba,
        system,
        "slot.state keeps the GBA's"
    );
    assert!(game_file(d.path(), Platform::Gba, "Emerald").exists());
    assert_eq!(app_playing_in(d.path(), "Fusion").screen_shader(), system);
    assert_eq!(app_playing_in(d.path(), "Emerald").screen_shader(), mine);
}

#[test]
fn y_puts_an_in_game_row_back_to_the_systems_value() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let system = read_slot_state(d.path()).shader_gba;
    let mut a = app_playing_in(d.path(), "Emerald");
    to_screen_row(&mut a, QuickRow::Shader);
    press(&mut a, Btn::Right);
    assert!(a.game_owns_row(QuickRow::Shader));
    press(&mut a, Btn::Y);
    assert!(!a.game_owns_row(QuickRow::Shader));
    assert_eq!(a.screen_shader(), system);
    assert!(
        !game_file(d.path(), Platform::Gba, "Emerald").exists(),
        "a game that follows its system in everything has no file"
    );
}

#[test]
fn a_value_set_back_to_the_systems_follows_the_system_again() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    to_screen_row(&mut a, QuickRow::Shader);
    press(&mut a, Btn::Right);
    press(&mut a, Btn::Left);
    assert!(!a.game_owns_row(QuickRow::Shader));
    assert_eq!(own_shader(d.path(), "Emerald"), None);
}

#[test]
fn a_shelf_change_reaches_every_game_without_its_own_value() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    to_screen_row(&mut a, QuickRow::Shader);
    press(&mut a, Btn::Right);
    let mine = a.screen_shader();
    // The shelf's GBA page sets the system's shader to another one.
    let system = Shader::GBA
        .into_iter()
        .find(|x| *x != mine && *x != read_slot_state(d.path()).shader_gba)
        .unwrap();
    let with = |stem: &str| {
        let mut s = SlotState {
            cart: Some(stem.to_string()),
            clock_set: true,
            ..Default::default()
        };
        s.shader_gba = system;
        write_slot_state(d.path(), &s).unwrap();
        let mut a = App::boot(d.path());
        a.set_snapshot(StubSnapshot::boxed());
        a.on_core_ready();
        for _ in 0..120 {
            a.update(1.0 / 60.0);
        }
        a.screen_shader()
    };
    assert_eq!(with("Fusion"), system);
    assert_eq!(with("Emerald"), mine);
}

#[test]
fn the_shader_shortcut_in_a_game_sets_the_games_own() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let system = read_slot_state(d.path()).shader_gba;
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply(Action::ShaderNext);
    assert_ne!(a.screen_shader(), system);
    assert_eq!(own_shader(d.path(), "Emerald"), Some(a.screen_shader()));
    assert_eq!(read_slot_state(d.path()).shader_gba, system);
}

#[test]
fn a_games_file_keeps_lines_it_does_not_know() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let file = game_file(d.path(), Platform::Gba, "Emerald");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, "later=kept\n").unwrap();
    let mut a = app_playing_in(d.path(), "Emerald");
    to_screen_row(&mut a, QuickRow::Shader);
    press(&mut a, Btn::Right);
    press(&mut a, Btn::Y);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "later=kept\n");
}

fn own_file(root: &Path, stem: &str, text: &str) {
    let file = game_file(root, Platform::Gba, stem);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, text).unwrap();
}

fn to_gameplay_row(a: &mut App, row: QuickRow) {
    a.apply(Action::QuickMenu);
    for step in [QuickRow::Settings, QuickRow::PlayGameplay, row] {
        for _ in 0..16 {
            if a.in_game_menu() == Some(step) {
                break;
            }
            press(a, Btn::Down);
        }
        assert_eq!(a.in_game_menu(), Some(step), "never reached {step:?}");
        if step != row {
            press(a, Btn::A);
        }
    }
}

#[test]
fn an_in_game_gameplay_change_is_the_games_own() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let shelf = read_slot_state(d.path()).ff_speed;
    let mut a = app_playing_in(d.path(), "Emerald");
    to_gameplay_row(&mut a, QuickRow::FastForward);
    press(&mut a, Btn::Right);
    let mine = a.ff_speed();
    assert_ne!(mine, shelf);
    assert!(a.game_owns_row(QuickRow::FastForward));
    assert_eq!(
        read_game(d.path(), Platform::Gba, "Emerald").play.ff_speed,
        Some(mine)
    );
    assert_eq!(
        read_slot_state(d.path()).ff_speed,
        shelf,
        "the shelf keeps its own"
    );
    assert_eq!(app_playing_in(d.path(), "Fusion").ff_speed(), shelf);
    press(&mut a, Btn::Y);
    assert_eq!(a.ff_speed(), shelf);
    assert!(!game_file(d.path(), Platform::Gba, "Emerald").exists());
}

#[test]
fn a_games_own_load_state_on_insert_decides_before_it_is_in_the_slot() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    common::clocked(d.path());
    assert!(read_slot_state(d.path()).auto_load);
    own_file(d.path(), "Emerald", "auto_load=false\n");
    let mut a = App::boot(d.path());
    a.apply_at(Action::GbaDown(Btn::A), 0);
    a.apply_at(Action::GbaUp(Btn::A), 120);
    assert!(
        a.starting_clean(),
        "Emerald starts clean, as its own setting says"
    );
}

#[test]
fn a_games_own_save_state_on_eject_holds_after_the_slot_is_cleared() {
    let d = tmp_root_with_carts(&["Emerald"]);
    own_file(d.path(), "Emerald", "eject_save=false\n");
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply(Action::Eject);
    let r = StateRing::new(d.path(), Platform::Gba, Core::Mgba, "Emerald");
    assert_eq!(r.read_resume().unwrap(), None, "a state was kept on eject");
}

fn to_shortcut_row(a: &mut App, row: QuickRow) {
    a.apply(Action::QuickMenu);
    for step in [QuickRow::Settings, QuickRow::PlayShortcuts, row] {
        for _ in 0..24 {
            if a.in_game_menu() == Some(step) {
                break;
            }
            press(a, Btn::Down);
        }
        assert_eq!(a.in_game_menu(), Some(step), "never reached {step:?}");
        if step != row {
            press(a, Btn::A);
        }
    }
}

#[test]
fn the_in_game_shortcuts_page_lists_the_systems_own_first_then_the_general() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    to_shortcut_row(&mut a, QuickRow::ScColour);
    let shown = QuickRow::ScColour.shown(&a.in_game_hidden());
    assert_eq!(
        shown[..3],
        [
            QuickRow::ScColour,
            QuickRow::ScSunlightDown,
            QuickRow::ScSunlightUp
        ],
        "the GBA's own"
    );
    assert_eq!(shown[3..], QuickRow::SHORTCUTS);
    press(&mut a, Btn::B);
    assert_eq!(a.in_game_menu(), Some(QuickRow::PlayShortcuts));
}

#[test]
fn a_shortcut_set_in_a_game_is_the_games_own_key() {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    let general = slot_store::read_shortcuts(d.path());
    let mut a = app_playing_in(d.path(), "Emerald");
    to_shortcut_row(&mut a, QuickRow::ScSave);
    press(&mut a, Btn::X);
    assert!(a.game_owns_row(QuickRow::ScSave));
    assert_eq!(
        read_game(d.path(), Platform::Gba, "Emerald")
            .shortcuts
            .get(Shortcut::SaveState),
        Some(None),
        "unbound in this game"
    );
    assert_eq!(
        slot_store::read_shortcuts(d.path()),
        general,
        "shortcuts.txt is untouched"
    );
    let saves = |a: &App| {
        a.binding_table()
            .entries()
            .iter()
            .any(|e| e.action == Action::SaveState)
    };
    assert!(!saves(&a), "Save State isn't listened for in Emerald");
    assert!(saves(&app_playing_in(d.path(), "Fusion")));
    press(&mut a, Btn::Y);
    assert!(!a.game_owns_row(QuickRow::ScSave));
    assert!(saves(&a));
}

#[test]
fn a_key_set_back_to_the_shelfs_follows_the_shelf_again() {
    let d = tmp_root_with_carts(&["Emerald"]);
    assert_eq!(
        slot_store::read_shortcuts(d.path()).get(Shortcut::SaveState),
        slot_store::Binding::pair(slot_store::Device::Select, slot_store::Device::R1),
        "the shelf's Save State is SELECT+R1"
    );
    let mut a = app_playing_in(d.path(), "Emerald");
    to_shortcut_row(&mut a, QuickRow::ScSave);
    press(&mut a, Btn::X);
    assert!(a.game_owns_row(QuickRow::ScSave));
    press(&mut a, Btn::A);
    assert!(a.capturing());
    for ev in [
        RawEvent::Down(Btn::Select),
        RawEvent::Down(Btn::R1),
        RawEvent::Up(Btn::R1),
        RawEvent::Up(Btn::Select),
    ] {
        a.capture_input(ev);
    }
    assert!(!a.capturing());
    assert!(
        !a.game_owns_row(QuickRow::ScSave),
        "SELECT+R1 is the shelf's key, so the game follows the shelf"
    );
    assert!(!game_file(d.path(), Platform::Gba, "Emerald").exists());
}

#[test]
fn a_shortcut_line_that_means_nothing_leaves_the_file() {
    let d = tmp_root_with_carts(&["Emerald"]);
    own_file(
        d.path(),
        "Emerald",
        "shortcut.save_state=sideways\nshortcut.no_such=r3\nlater=kept\n",
    );
    let mut a = app_playing_in(d.path(), "Emerald");
    assert!(!a.game_owns_row(QuickRow::ScSave), "the shelf's key holds");
    to_screen_row(&mut a, QuickRow::Shader);
    press(&mut a, Btn::Right);
    press(&mut a, Btn::Y);
    assert_eq!(
        std::fs::read_to_string(game_file(d.path(), Platform::Gba, "Emerald")).unwrap(),
        "later=kept\n"
    );
}
