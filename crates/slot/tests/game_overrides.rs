//! Slot²: a game's own screen settings over its system's (design/game-overrides.md).

use crate::common;

use std::path::Path;

use common::{app_playing_in, tmp_root_with_carts, StubSnapshot};
use slot::app::App;
use slot_input::{Action, Btn};
use slot_store::games::{game_file, read_game};
use slot_store::{read_slot_state, write_slot_state, Platform, Shader, SlotState};
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
