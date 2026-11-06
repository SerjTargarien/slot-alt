use slot::input::HostInput;
use slot_input::{Btn, InputSource, RawEvent};
use winit::event::WindowEvent;
use winit::keyboard::KeyCode;

fn edge(h: &mut HostInput, code: KeyCode, pressed: bool) -> Vec<RawEvent> {
    h.key(code, pressed, false);
    h.poll(0)
}

#[test]
fn the_host_keymap_matches_the_documented_layout() {
    let map = [
        (KeyCode::ArrowUp, Btn::Up),
        (KeyCode::ArrowDown, Btn::Down),
        (KeyCode::ArrowLeft, Btn::Left),
        (KeyCode::ArrowRight, Btn::Right),
        (KeyCode::KeyZ, Btn::A),
        (KeyCode::KeyX, Btn::B),
        (KeyCode::KeyC, Btn::X),
        (KeyCode::KeyA, Btn::L1),
        (KeyCode::KeyS, Btn::R1),
        (KeyCode::KeyQ, Btn::L2),
        (KeyCode::KeyW, Btn::R2),
        (KeyCode::Enter, Btn::Start),
        (KeyCode::ShiftRight, Btn::Select),
        (KeyCode::Tab, Btn::Menu),
        (KeyCode::Equal, Btn::VolUp),
        (KeyCode::Minus, Btn::VolDown),
        (KeyCode::Backslash, Btn::Power),
    ];
    let mut h = HostInput::new();
    for (code, btn) in map {
        assert_eq!(edge(&mut h, code, true), vec![RawEvent::Down(btn)]);
        assert_eq!(edge(&mut h, code, false), vec![RawEvent::Up(btn)]);
    }
    assert!(edge(&mut h, KeyCode::KeyP, true).is_empty());
}

#[test]
fn key_repeat_does_not_re_press_the_button() {
    let mut h = HostInput::new();
    assert_eq!(
        edge(&mut h, KeyCode::Tab, true),
        vec![RawEvent::Down(Btn::Menu)]
    );
    h.key(KeyCode::Tab, true, true);
    h.key(KeyCode::Tab, true, true);
    assert!(
        h.poll(0).is_empty(),
        "autorepeat would re-arm the menu hold and double tap windows"
    );
}

#[test]
fn the_lid_key_toggles_because_the_host_has_no_hinge() {
    let mut h = HostInput::new();
    assert_eq!(
        edge(&mut h, KeyCode::BracketRight, true),
        vec![RawEvent::Down(Btn::Lid)]
    );
    assert!(edge(&mut h, KeyCode::BracketRight, false).is_empty());
    assert_eq!(
        edge(&mut h, KeyCode::BracketRight, true),
        vec![RawEvent::Up(Btn::Lid)]
    );
    assert_eq!(
        edge(&mut h, KeyCode::BracketRight, true),
        vec![RawEvent::Down(Btn::Lid)]
    );
}

#[test]
fn the_two_reflex_keys_no_longer_end_a_session() {
    let mut h = HostInput::new();
    for reflex in [KeyCode::Escape, KeyCode::KeyL] {
        assert!(
            edge(&mut h, reflex, true).is_empty(),
            "{reflex:?} is bound again, and it can end a live link"
        );
    }
}

#[test]
fn a_window_that_loses_focus_lets_go_of_the_keys_held_in_it() {
    let mut h = HostInput::new();
    assert_eq!(
        edge(&mut h, KeyCode::KeyZ, true),
        vec![RawEvent::Down(Btn::A)]
    );
    assert_eq!(
        edge(&mut h, KeyCode::ArrowRight, true),
        vec![RawEvent::Down(Btn::Right)]
    );
    assert_eq!(
        edge(&mut h, KeyCode::KeyZ, false),
        vec![RawEvent::Up(Btn::A)]
    );
    h.on_window_event(&WindowEvent::Focused(false));
    assert_eq!(
        h.poll(0),
        vec![RawEvent::Up(Btn::Right)],
        "the key still down when the window went away was never released"
    );
    h.on_window_event(&WindowEvent::Focused(false));
    assert!(h.poll(0).is_empty());
    h.on_window_event(&WindowEvent::Focused(true));
    assert!(h.poll(0).is_empty());
}

#[test]
fn the_pad_presses_the_console_buttons_bound_to_what_is_held() {
    use slot::input::Pad;
    use slot_input::Action;
    use slot_retro::ButtonMask;
    use slot_store::controls::{Input, SystemControls};
    use slot_store::{Binding, Device, Modifier, Platform};
    let gb = Platform::Gb;
    let mut c = SystemControls::default_for(gb);
    let mut pad = Pad::default();
    pad.apply(Action::GbaDown(Btn::X));
    assert_eq!(
        pad.mask(&c, gb, false).0,
        0,
        "X reached the game with nothing bound to it"
    );
    c.set_row(Input::A, [Binding::plain(Device::X), None, None]);
    assert_eq!(pad.mask(&c, gb, false).0, ButtonMask::A);
    pad.apply(Action::GbaDown(Btn::A));
    assert_eq!(
        pad.mask(&c, gb, false).0,
        ButtonMask::A,
        "A still pressed A after A moved to X"
    );
    pad.apply(Action::GbaUp(Btn::X));
    assert_eq!(pad.mask(&c, gb, false).0, 0);
    c.set_row(
        Input::B,
        [Binding::with(Modifier::Menu, Device::Y), None, None],
    );
    pad.apply(Action::GbaDown(Btn::Y));
    assert_eq!(
        pad.mask(&c, gb, false).0,
        0,
        "plain Y pressed a MENU+Y binding"
    );
    pad.apply(Action::MenuChordDown(Btn::Y));
    assert_eq!(pad.mask(&c, gb, false).0, ButtonMask::B);
}

#[test]
fn any_of_an_inputs_bindings_presses_it() {
    use slot::input::Pad;
    use slot_input::Action;
    use slot_retro::ButtonMask;
    use slot_store::controls::{Input, SystemControls};
    use slot_store::{Binding, Device, Platform};
    let gb = Platform::Gb;
    let mut c = SystemControls::default_for(gb);
    c.add(Input::A, Binding::plain(Device::R2).unwrap());
    for b in [Btn::A, Btn::R2] {
        let mut pad = Pad::default();
        pad.apply(Action::GbaDown(b));
        assert_eq!(pad.mask(&c, gb, false).0, ButtonMask::A, "{b:?}");
    }
}

#[test]
fn a_held_chord_wins_over_its_buttons_own_inputs() {
    use slot::input::Pad;
    use slot_input::Action;
    use slot_retro::ButtonMask;
    use slot_store::controls::{Input, SystemControls};
    use slot_store::{Binding, Device, Platform};
    // On the WonderSwan, X1 is Up and Y4 is L1; bind Y1 to L1+Up as well.
    let ws = Platform::Ws;
    let mut c = SystemControls::default_for(ws);
    c.add(Input::Y1, Binding::pair(Device::L1, Device::Up).unwrap());
    let mut pad = Pad::default();
    pad.apply(Action::GbaDown(Btn::Up));
    assert_eq!(pad.mask(&c, ws, false).0, ButtonMask::UP);
    pad.apply(Action::GbaDown(Btn::L1));
    assert_eq!(
        pad.mask(&c, ws, false).0,
        ButtonMask::R2,
        "Y1, and neither X1 nor Y4 too"
    );
    pad.apply(Action::GbaUp(Btn::Up));
    assert_eq!(
        pad.mask(&c, ws, false).0,
        ButtonMask::L,
        "L1 alone is Y4 again"
    );
}

#[test]
fn every_system_presses_todays_core_buttons_for_todays_presses() {
    use slot::input::Pad;
    use slot_input::Action;
    use slot_retro::ButtonMask;
    use slot_store::controls::SystemControls;
    use slot_store::Platform;
    let press = |platform: Platform, b: Btn| {
        let mut pad = Pad::default();
        pad.apply(Action::GbaDown(b));
        pad.mask(&SystemControls::default_for(platform), platform, false)
            .0
    };
    for platform in Platform::ALL {
        for (b, bit) in [
            (Btn::Up, ButtonMask::UP),
            (Btn::Down, ButtonMask::DOWN),
            (Btn::Left, ButtonMask::LEFT),
            (Btn::Right, ButtonMask::RIGHT),
            (Btn::A, ButtonMask::A),
            (Btn::B, ButtonMask::B),
            (Btn::Start, ButtonMask::START),
        ] {
            assert_eq!(press(platform, b), bit, "{platform:?} {b:?}");
        }
    }
    assert_eq!(press(Platform::Gb, Btn::L1), 0, "a Game Boy has no L");
    assert_eq!(press(Platform::Gba, Btn::L1), ButtonMask::L);
    assert_eq!(press(Platform::Gg, Btn::Select), 0, "it would reset");
    assert_eq!(press(Platform::Gg, Btn::X), 0, "X is free: turbo is gone");
    assert_eq!(press(Platform::Nds, Btn::Y), ButtonMask::Y);
}

#[test]
fn a_select_chord_presses_its_input_and_not_select() {
    use slot::input::Pad;
    use slot_input::Action;
    use slot_retro::ButtonMask;
    use slot_store::controls::{Input, SystemControls};
    use slot_store::{Binding, Device, Platform};
    let gba = Platform::Gba;
    let mut c = SystemControls::default_for(gba);
    c.add(
        Input::Start,
        Binding::pair(Device::L1, Device::Select).unwrap(),
    );
    // The gesture layer sends the chord as itself, holding SELECT back.
    let mut pad = Pad::default();
    pad.apply(Action::SelectChordDown(Btn::L1));
    assert_eq!(pad.mask(&c, gba, false).0, ButtonMask::START);
    pad.apply(Action::SelectChordUp(Btn::L1));
    assert_eq!(pad.mask(&c, gba, false).0, 0);
}

#[test]
fn a_select_chord_on_a_game_row_reaches_the_gestures() {
    use slot_store::controls::{Input, SystemControls};
    use slot_store::{Binding, CarouselKeys, Device, Platform, ShortcutKeys};
    let mut c = SystemControls::default_for(Platform::Gba);
    c.add(
        Input::Start,
        Binding::pair(Device::L1, Device::Select).unwrap(),
    );
    let table = slot::input::Table::new(
        &c,
        Platform::Gba,
        &ShortcutKeys::default(),
        &slot_store::shortcuts::SystemShortcuts::from(&ShortcutKeys::default()),
        &CarouselKeys::default(),
    );
    assert!(table.entries().iter().any(|e| e.chord
        == slot_input::Chord {
            modifier: Some(slot_input::Mod::Select),
            button: Btn::L1,
        }
        && e.action == slot_input::Action::SelectChordDown(Btn::L1)));
}

#[test]
fn the_ds_stylus_is_the_right_stick_and_r2_and_leaves_the_game_its_buttons() {
    use slot::input::Pad;
    use slot_input::Action;
    use slot_retro::ButtonMask;
    use slot_store::controls::{Input, SystemControls};
    use slot_store::{Binding, Device, Platform};
    let ds = Platform::Nds;
    let mut c = SystemControls::default_for(ds);
    let mut pad = Pad::default();
    for b in [Btn::A, Btn::Up, Btn::RsLeft, Btn::R2] {
        pad.apply(Action::GbaDown(b));
    }
    let pressed = pad.pressed(&c, ds, true);
    assert!(pressed.contains(&Input::StylusLeft) && pressed.contains(&Input::Touch));
    assert_eq!(
        pad.mask(&c, ds, true).0,
        ButtonMask::A | ButtonMask::UP,
        "the D-pad and A stay the game's while the stick steers"
    );
    // Stylus Hold, bound for a stickless handheld, gives the stylus the D-pad instead.
    c.set_row(Input::Stylus, [Binding::plain(Device::L3), None, None]);
    pad.apply(Action::GbaDown(Btn::L3));
    assert_eq!(
        pad.mask(&c, ds, true).0,
        ButtonMask::A,
        "the D-pad steers it"
    );
}

#[test]
fn chords_collide_when_they_need_a_button_in_the_same_layer() {
    use slot::input::collides;
    use slot_input::{Chord, Mod};
    let plain = |button| Chord {
        modifier: None,
        button,
    };
    let with = |m, button| Chord {
        modifier: Some(m),
        button,
    };
    assert!(collides(plain(Btn::L1), plain(Btn::L1)));
    assert!(
        collides(plain(Btn::L1), with(Mod::Button(Btn::L1), Btn::Up)),
        "L1 alone takes the L1 of L1+Up"
    );
    assert!(
        !collides(plain(Btn::L1), with(Mod::Menu, Btn::L1)),
        "MENU+L1 is its own"
    );
    assert!(
        !collides(plain(Btn::X), with(Mod::Select, Btn::X)),
        "and SELECT+X"
    );
    assert!(collides(
        with(Mod::Select, Btn::X),
        with(Mod::Select, Btn::X)
    ));
    assert!(
        !collides(with(Mod::Button(Btn::Start), Btn::L2), plain(Btn::Start)),
        "START leads: it reaches the game, only L2 is taken"
    );
    assert!(collides(
        with(Mod::Button(Btn::Start), Btn::L2),
        plain(Btn::L2)
    ));
}
