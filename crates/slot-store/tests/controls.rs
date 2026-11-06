use slot_store::controls::{
    inputs, read_system_controls, write_system_controls, Input, SystemControls, SLOTS,
};
use slot_store::{write_controls, Binding, Controls, Device, GameButton, Modifier, Platform};

fn plain(d: Device) -> Binding {
    Binding::plain(d).unwrap()
}

fn chord(a: Device, b: Device) -> Binding {
    Binding::pair(a, b).unwrap()
}

#[test]
fn every_system_moves_with_the_d_pad_and_the_left_stick() {
    for platform in Platform::ALL {
        let c = SystemControls::default_for(platform);
        assert_eq!(
            c.row(Input::Up),
            [Some(plain(Device::Up)), Some(plain(Device::LsUp)), None],
            "{platform:?}"
        );
        assert_eq!(
            c.row(Input::Left),
            [Some(plain(Device::Left)), Some(plain(Device::LsLeft)), None]
        );
    }
}

#[test]
fn the_defaults_bind_what_upstream_bound() {
    let gb = SystemControls::default_for(Platform::Gb);
    assert_eq!(gb.row(Input::A), [Some(plain(Device::A)), None, None]);
    assert_eq!(gb.row(Input::TurboB), [Some(plain(Device::Y)), None, None]);
    assert_eq!(gb.row(Input::L), [None; SLOTS], "a Game Boy has no L");
    let gba = SystemControls::default_for(Platform::Gba);
    assert_eq!(gba.row(Input::L), [Some(plain(Device::L1)), None, None]);
    let ds = SystemControls::default_for(Platform::Nds);
    assert_eq!(ds.row(Input::X), [Some(plain(Device::X)), None, None]);
    assert_eq!(
        ds.row(Input::TurboA),
        [None; SLOTS],
        "the DS's X is its own"
    );
}

#[test]
fn the_wonderswan_second_pad_is_held_l1_with_the_d_pad_or_held_r1_with_the_face() {
    let ws = SystemControls::default_for(Platform::Ws);
    assert_eq!(
        ws.row(Input::Y1),
        [
            Some(chord(Device::L1, Device::Up)),
            Some(chord(Device::R1, Device::X)),
            None
        ]
    );
    assert_eq!(
        ws.row(Input::Y4),
        [
            Some(chord(Device::L1, Device::Left)),
            Some(chord(Device::R1, Device::Y)),
            None
        ]
    );
    assert!(
        !inputs(Platform::Ws).contains(&Input::L),
        "L1 is only a modifier"
    );
}

#[test]
fn adding_a_fourth_binding_replaces_the_oldest() {
    let mut c = SystemControls::default_for(Platform::Gb);
    c.add(Input::A, plain(Device::R2));
    assert_eq!(
        c.row(Input::A),
        [Some(plain(Device::A)), Some(plain(Device::R2)), None]
    );
    c.add(Input::A, plain(Device::L2));
    c.add(Input::A, plain(Device::RsUp));
    assert_eq!(
        c.row(Input::A),
        [
            Some(plain(Device::R2)),
            Some(plain(Device::L2)),
            Some(plain(Device::RsUp))
        ]
    );
    // One already there moves to the newest place rather than appearing twice.
    c.add(Input::A, plain(Device::R2));
    assert_eq!(
        c.row(Input::A),
        [
            Some(plain(Device::L2)),
            Some(plain(Device::RsUp)),
            Some(plain(Device::R2))
        ]
    );
}

#[test]
fn x_removes_the_last_binding_and_y_puts_the_row_back() {
    let mut c = SystemControls::default_for(Platform::Gb);
    c.remove_last(Input::Up);
    assert_eq!(c.row(Input::Up), [Some(plain(Device::Up)), None, None]);
    c.remove_last(Input::Up);
    c.remove_last(Input::Up);
    assert_eq!(c.row(Input::Up), [None; SLOTS]);
    c.reset(Platform::Gb, Input::Up);
    assert_eq!(c, SystemControls::default_for(Platform::Gb));
}

#[test]
fn each_system_keeps_its_own_file() {
    let d = tempfile::tempdir().unwrap();
    let mut ws = SystemControls::default_for(Platform::Ws);
    ws.add(Input::Y1, plain(Device::RsUp));
    ws.remove_last(Input::B);
    write_system_controls(d.path(), Platform::Ws, &ws).unwrap();
    let text = std::fs::read_to_string(d.path().join("Config/controls/ws.txt")).unwrap();
    assert!(text.lines().any(|l| l == "y1=up+l1,x+r1,rs_up"), "{text}");
    assert!(text.lines().any(|l| l == "b=none"), "{text}");
    assert_eq!(read_system_controls(d.path(), Platform::Ws), ws);
    assert_eq!(
        read_system_controls(d.path(), Platform::Gb),
        SystemControls::default_for(Platform::Gb),
        "the Game Boy's controls didn't change"
    );
}

#[test]
fn upstream_controls_seed_every_system_where_the_user_changed_them() {
    let d = tempfile::tempdir().unwrap();
    let mut upstream = Controls::default();
    upstream.set_game(GameButton::A, Binding::plain(Device::R2));
    upstream.set_game(GameButton::TurboA, Binding::with(Modifier::Menu, Device::X));
    write_controls(d.path(), &upstream).unwrap();
    for platform in [Platform::Gb, Platform::Gba, Platform::Ws, Platform::Ngp] {
        let c = read_system_controls(d.path(), platform);
        assert_eq!(c.row(Input::A), [Some(plain(Device::R2)), None, None]);
        assert_eq!(
            c.row(Input::Up),
            SystemControls::default_for(platform).row(Input::Up),
            "unchanged rows keep the new defaults"
        );
    }
    let ds = read_system_controls(d.path(), Platform::Nds);
    assert_eq!(
        ds.row(Input::X),
        [Binding::with(Modifier::Menu, Device::X), None, None],
        "the DS's X was the Turbo A row"
    );
}
