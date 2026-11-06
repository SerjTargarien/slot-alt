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
    c.set_row(Input::TurboA, [None; 3]);
    c.set_row(Input::TurboB, [None; 3]);
    let mut pad = Pad::default();
    pad.apply(Action::GbaDown(Btn::X));
    assert_eq!(
        pad.mask(&c, gb).0,
        0,
        "X reached the game with nothing bound to it"
    );
    c.set_row(Input::A, [Binding::plain(Device::X), None, None]);
    assert_eq!(pad.mask(&c, gb).0, ButtonMask::A);
    pad.apply(Action::GbaDown(Btn::A));
    assert_eq!(
        pad.mask(&c, gb).0,
        ButtonMask::A,
        "A still pressed A after A moved to X"
    );
    pad.apply(Action::GbaUp(Btn::X));
    assert_eq!(pad.mask(&c, gb).0, 0);
    c.set_row(
        Input::B,
        [Binding::with(Modifier::Menu, Device::Y), None, None],
    );
    pad.apply(Action::GbaDown(Btn::Y));
    assert_eq!(pad.mask(&c, gb).0, 0, "plain Y pressed a MENU+Y binding");
    pad.apply(Action::MenuChordDown(Btn::Y));
    assert_eq!(pad.mask(&c, gb).0, ButtonMask::B);
}

#[test]
fn turbo_a_and_b_fire_from_whatever_they_are_bound_to() {
    use slot::input::Pad;
    use slot_input::Action;
    use slot_retro::ButtonMask;
    use slot_store::controls::{Input, SystemControls};
    use slot_store::{Binding, Device, Platform};
    let gb = Platform::Gb;
    let mut c = SystemControls::default_for(gb);
    let mut pad = Pad::default();
    pad.apply(Action::GbaDown(Btn::X));
    assert_eq!(pad.mask(&c, gb).turbo(0).0, ButtonMask::A);
    assert_eq!(pad.mask(&c, gb).turbo(3).0, 0);
    assert_eq!(pad.mask(&c, gb).without_turbo().0, 0);
    pad.apply(Action::GbaUp(Btn::X));
    pad.apply(Action::GbaDown(Btn::Y));
    assert_eq!(pad.mask(&c, gb).turbo(0).0, ButtonMask::B);
    pad.apply(Action::GbaUp(Btn::Y));
    c.set_row(Input::TurboA, [Binding::plain(Device::L2), None, None]);
    pad.apply(Action::GbaDown(Btn::X));
    assert_eq!(
        pad.mask(&c, gb).turbo(0).0,
        0,
        "X kept turbo A after it moved"
    );
    pad.apply(Action::GbaDown(Btn::L2));
    assert_eq!(pad.mask(&c, gb).turbo(0).0, ButtonMask::A);
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
        assert_eq!(pad.mask(&c, gb).0, ButtonMask::A, "{b:?}");
    }
}

#[test]
fn a_held_chord_wins_over_its_buttons_own_inputs() {
    use slot::input::Pad;
    use slot_input::Action;
    use slot_retro::ButtonMask;
    use slot_store::controls::SystemControls;
    use slot_store::Platform;
    // The WonderSwan's Y1 is L1+Up by default, and its X1 is Up.
    let ws = Platform::Ws;
    let c = SystemControls::default_for(ws);
    let mut pad = Pad::default();
    pad.apply(Action::GbaDown(Btn::Up));
    assert_eq!(pad.mask(&c, ws).0, ButtonMask::UP);
    pad.apply(Action::GbaDown(Btn::L1));
    assert_eq!(pad.mask(&c, ws).0, ButtonMask::R2, "Y1, and not X1 too");
    pad.apply(Action::GbaUp(Btn::L1));
    pad.apply(Action::GbaUp(Btn::Up));
    pad.apply(Action::GbaDown(Btn::R1));
    pad.apply(Action::GbaDown(Btn::A));
    assert_eq!(pad.mask(&c, ws).0, ButtonMask::R, "R1+A is Y2");
    pad.apply(Action::GbaUp(Btn::R1));
    assert_eq!(pad.mask(&c, ws).0, ButtonMask::A);
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
        pad.mask(&SystemControls::default_for(platform), platform).0
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
    assert_eq!(press(Platform::Gg, Btn::X), ButtonMask::X, "turbo 2");
    assert_eq!(press(Platform::Nds, Btn::Y), ButtonMask::Y);
}
