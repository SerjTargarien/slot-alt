use slot_store::controls::{
    drives, inputs, label, read_system_controls, write_system_controls, Input, SystemControls,
    SLOTS,
};
use slot_store::{write_controls, Binding, Controls, Device, GameButton, Modifier, Platform};

fn plain(d: Device) -> Binding {
    Binding::plain(d).unwrap()
}

#[test]
fn every_system_moves_with_the_d_pad_and_the_left_stick() {
    for platform in Platform::ALL {
        let c = SystemControls::default_for(platform);
        let moving = |to: Input| {
            let input = inputs(platform)
                .iter()
                .copied()
                .find(|i| drives(platform, *i) == to)
                .expect("an input for each direction");
            c.row(input)
        };
        assert_eq!(
            moving(Input::Up),
            [Some(plain(Device::Up)), Some(plain(Device::LsUp)), None],
            "{platform:?}"
        );
        assert_eq!(
            moving(Input::Left),
            [Some(plain(Device::Left)), Some(plain(Device::LsLeft)), None],
            "{platform:?}"
        );
    }
}

#[test]
fn the_defaults_bind_what_upstream_bound() {
    let gb = SystemControls::default_for(Platform::Gb);
    assert_eq!(gb.row(Input::A), [Some(plain(Device::A)), None, None]);
    assert_eq!(gb.row(Input::L), [None; SLOTS], "a Game Boy has no L");
    let gba = SystemControls::default_for(Platform::Gba);
    assert_eq!(gba.row(Input::L), [Some(plain(Device::L1)), None, None]);
}

#[test]
fn the_wonderswan_second_pad_is_the_right_stick_and_the_shoulders() {
    let ws = SystemControls::default_for(Platform::Ws);
    // The shoulders left to right (L1, L2, R2, R1) are its left, up, down and right.
    for (input, stick, shoulder) in [
        (Input::Y1, Device::RsUp, Device::L2),
        (Input::Y2, Device::RsRight, Device::R1),
        (Input::Y3, Device::RsDown, Device::R2),
        (Input::Y4, Device::RsLeft, Device::L1),
    ] {
        assert_eq!(
            ws.row(input),
            [Some(plain(stick)), Some(plain(shoulder)), None],
            "{input:?}"
        );
    }
    assert!(
        !inputs(Platform::Ws).contains(&Input::L),
        "L1 is the Y pad's"
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
    ws.add(Input::Y1, plain(Device::R3));
    ws.remove_last(Input::B);
    write_system_controls(d.path(), Platform::Ws, &ws).unwrap();
    let text = std::fs::read_to_string(d.path().join("Config/controls/ws.txt")).unwrap();
    assert!(text.lines().any(|l| l == "y1=rs_up,l2,r3"), "{text}");
    assert!(text.lines().any(|l| l == "b=none"), "{text}");
    assert_eq!(
        text.lines().count(),
        2,
        "only the rows that differ from the defaults: {text}"
    );
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
        let a = match platform {
            // The Pocket's B Button row is the core's A.
            Platform::Ngp => Input::B,
            _ => Input::A,
        };
        assert_eq!(
            c.row(a),
            [Some(plain(Device::R2)), None, None],
            "{platform:?}"
        );
        assert_eq!(
            c.row(Input::Up),
            SystemControls::default_for(platform).row(Input::Up),
            "unchanged rows keep the new defaults"
        );
    }
}

#[test]
fn rows_read_in_each_systems_own_order_and_keep_todays_buttons() {
    // The Game Gear's Button 1 is the core's B, still on the B button.
    let gg = SystemControls::default_for(Platform::Gg);
    assert_eq!(label(Platform::Gg, Input::A), "Button 1");
    assert_eq!(drives(Platform::Gg, Input::A), Input::B);
    assert_eq!(gg.row(Input::A), [Some(plain(Device::B)), None, None]);
    assert!(
        !inputs(Platform::Gg).contains(&Input::Select),
        "Select resets it"
    );
    // The WonderSwan's X pad runs up, right, down, left down its page.
    let ws = SystemControls::default_for(Platform::Ws);
    let pad = [Input::Up, Input::Down, Input::Left, Input::Right];
    assert_eq!(
        pad.map(|i| label(Platform::Ws, i)),
        ["X1", "X2", "X3", "X4"]
    );
    assert_eq!(
        ws.row(Input::Down),
        [
            Some(plain(Device::Right)),
            Some(plain(Device::LsRight)),
            None
        ],
        "X2 is right"
    );
}

#[test]
fn the_wonderswan_turns_with_select_and_nothing_else_has_rotate() {
    let ws = SystemControls::default_for(Platform::Ws);
    assert!(inputs(Platform::Ws).contains(&Input::Rotate));
    assert_eq!(
        ws.row(Input::Rotate),
        [Some(plain(Device::Select)), None, None]
    );
    assert_eq!(label(Platform::Ws, Input::Rotate), "Rotate");
    for platform in Platform::ALL.into_iter().filter(|p| *p != Platform::Ws) {
        assert!(!inputs(platform).contains(&Input::Rotate), "{platform:?}");
    }
}
