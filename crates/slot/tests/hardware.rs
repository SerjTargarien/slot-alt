use slot::hardware::{detect, Buttons, Display, Hardware};
use std::path::PathBuf;

const BASESQ: &str = "# Base² on the RGB20SX
NAME=\"Base² RGB20SX\"
DISPLAY=kms
BUTTONS=gamepad
FN_ADC=
CPU_CAP=1
WIFI=0
LEFT_STICK=1
BOOT_LOGO=0
SELF_UPDATE=0
";

#[test]
fn a_description_file_sets_every_field() {
    let hw = Hardware::parse(BASESQ);
    assert_eq!(hw.name, "Base² RGB20SX");
    assert_eq!(hw.display, Display::Kms);
    assert_eq!(hw.buttons, Buttons::Gamepad);
    assert_eq!(hw.fn_adc, None);
    assert!(hw.cpu_cap);
    assert!(!hw.wifi);
    assert_eq!(hw.left_stick, Some(true));
    assert!(!hw.boot_logo);
    assert!(!hw.self_update);
}

#[test]
fn missing_keys_keep_what_baseos_does() {
    assert_eq!(Hardware::parse(""), Hardware::baseos());
    let hw = Hardware::parse("DISPLAY=kms\nFN_ADC=/sys/fn\n");
    assert_eq!(hw.display, Display::Kms);
    assert_eq!(hw.buttons, Buttons::Anbernic);
    assert_eq!(hw.fn_adc, Some(PathBuf::from("/sys/fn")));
    assert!(hw.wifi);
    assert_eq!(hw.left_stick, None);
}

#[test]
fn slot_fn_adc_turns_the_adc_off_or_moves_it() {
    let adc = || Hardware::parse("FN_ADC=/sys/fn\n");
    assert_eq!(adc().with_fn_adc(Some("0")).fn_adc, None);
    assert_eq!(
        Hardware::baseos().with_fn_adc(Some("/sys/other")).fn_adc,
        Some(PathBuf::from("/sys/other"))
    );
    assert_eq!(adc().with_fn_adc(None), adc());
}

#[test]
fn the_description_file_or_else_baseos() {
    let dir = tempfile::tempdir().unwrap();
    let desc = dir.path().join("slot-device");
    assert_eq!(detect(&desc), Hardware::baseos());
    std::fs::write(&desc, BASESQ).unwrap();
    assert_eq!(detect(&desc), Hardware::parse(BASESQ));
}
