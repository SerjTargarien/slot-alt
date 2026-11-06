use std::fs;
use std::path::Path;

use slot::input::adc::{parse, AdcKey};
use slot::input::evdev::{code_to_btn_in, pick_devices_in, Layout};
use slot_input::{Btn, RawEvent};

#[test]
fn the_rgb20sx_buttons_map_by_their_standard_gamepad_codes() {
    let probed = [
        (0x131, Btn::A),
        (0x130, Btn::B),
        (0x133, Btn::X),
        (0x134, Btn::Y),
        (0x136, Btn::L1),
        (0x137, Btn::R1),
        (0x138, Btn::L2),
        (0x139, Btn::R2),
        (0x13a, Btn::Select),
        (0x13b, Btn::Start),
        (0x13c, Btn::Menu),
        (0x220, Btn::Up),
        (0x221, Btn::Down),
        (0x222, Btn::Left),
        (0x223, Btn::Right),
        (115, Btn::VolUp),
        (114, Btn::VolDown),
        (116, Btn::Power),
        (0x13d, Btn::L3),
        (0x13e, Btn::R3),
    ];
    for (code, btn) in probed {
        assert_eq!(
            code_to_btn_in(Layout::Gamepad, code),
            Some(btn),
            "{code:#x}"
        );
    }
}

#[test]
fn the_same_code_means_different_buttons_on_each_layout() {
    assert_eq!(code_to_btn_in(Layout::Anbernic, 0x138), Some(Btn::Menu));
    assert_eq!(code_to_btn_in(Layout::Gamepad, 0x138), Some(Btn::L2));
    assert_eq!(code_to_btn_in(Layout::Anbernic, 0x13a), Some(Btn::L2));
    assert_eq!(code_to_btn_in(Layout::Gamepad, 0x13a), Some(Btn::Select));
}

fn write(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

fn rgb20sx_tree() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    let (dev, sys) = (d.path().join("dev/input"), d.path().join("sys/class/input"));
    let nodes = [
        ("event0", "rk805 pwrkey", "10000000000000 0"),
        ("event1", "adc-keys", "1000000000000000 0 0 0 0"),
        ("event2", "adc-joystick", "0"),
        (
            "event3",
            "gpio-keys-control",
            "f00000000 0 0 0 6fdb000000000000 0 0 0 0",
        ),
        ("event4", "gpio-keys-vol", "c000000000000 0"),
    ];
    for (node, name, keys) in nodes {
        write(&dev.join(node), "");
        write(&sys.join(node).join("device/name"), &format!("{name}\n"));
        write(
            &sys.join(node).join("device/capabilities/key"),
            &format!("{keys}\n"),
        );
    }
    d
}

fn picked(d: &tempfile::TempDir, layout: Layout) -> Vec<String> {
    pick_devices_in(
        &d.path().join("dev/input"),
        &d.path().join("sys/class/input"),
        layout,
    )
    .iter()
    .filter_map(|p| p.file_name())
    .map(|n| n.to_string_lossy().into_owned())
    .collect()
}

#[test]
fn the_gamepad_layout_opens_every_rgb20sx_button_device() {
    let d = rgb20sx_tree();
    assert_eq!(
        picked(&d, Layout::Gamepad),
        ["event0", "event1", "event3", "event4"]
    );
}

#[test]
fn the_anbernic_layout_never_opens_a_device_holding_only_btn_mode() {
    let d = rgb20sx_tree();
    assert_eq!(picked(&d, Layout::Anbernic), ["event0", "event3", "event4"]);
}

#[test]
fn an_adc_key_presses_low_and_releases_high() {
    let mut fn_key = AdcKey::new(Btn::Menu);
    assert_eq!(fn_key.feed(1023), None);
    assert_eq!(fn_key.feed(15), Some(RawEvent::Down(Btn::Menu)));
    assert_eq!(fn_key.feed(20), None);
    assert_eq!(fn_key.feed(1023), Some(RawEvent::Up(Btn::Menu)));
    assert_eq!(fn_key.feed(1023), None);
}

#[test]
fn a_reading_between_the_thresholds_changes_nothing() {
    let mut fn_key = AdcKey::new(Btn::Menu);
    assert_eq!(fn_key.feed(500), None);
    assert_eq!(fn_key.feed(15), Some(RawEvent::Down(Btn::Menu)));
    assert_eq!(fn_key.feed(500), None);
    assert_eq!(fn_key.feed(900), Some(RawEvent::Up(Btn::Menu)));
}

#[test]
fn losing_the_adc_releases_a_held_key_once() {
    let mut fn_key = AdcKey::new(Btn::Menu);
    assert_eq!(fn_key.lost(), None);
    fn_key.feed(15);
    assert_eq!(fn_key.lost(), Some(RawEvent::Up(Btn::Menu)));
    assert_eq!(fn_key.lost(), None);
}

#[test]
fn a_sysfs_reading_parses_with_its_newline() {
    assert_eq!(parse("1023\n"), Some(1023));
    assert_eq!(parse(" 15 "), Some(15));
    assert_eq!(parse(""), None);
    assert_eq!(parse("-1"), None);
}
