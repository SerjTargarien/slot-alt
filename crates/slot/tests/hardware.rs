use slot::hardware::{detect, Buttons, Display, Hardware};
use std::path::PathBuf;

const SLOTSQ: &str = "# slot² on the RGB20SX
NAME=\"slot² RGB20SX\"
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
    let hw = Hardware::parse(SLOTSQ);
    assert_eq!(hw.name, "slot² RGB20SX");
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
fn rocknix_reads_fn_from_the_adc_only_on_the_rgb20sx() {
    let rgb = Hardware::rocknix(slot::rocknix::RGB20SX);
    assert_eq!(rgb.display, Display::Kms);
    assert_eq!(rgb.buttons, Buttons::Gamepad);
    assert_eq!(
        rgb.fn_adc,
        Some(PathBuf::from(slot::input::adc::RGB20SX_FN))
    );
    assert!(rgb.cpu_cap);
    assert!(!rgb.wifi);
    assert!(!rgb.self_update);
    assert_eq!(Hardware::rocknix("Anbernic RG353V").fn_adc, None);
}

#[test]
fn slot_fn_adc_turns_the_adc_off_or_moves_it() {
    let rgb = || Hardware::rocknix(slot::rocknix::RGB20SX);
    assert_eq!(rgb().with_fn_adc(Some("0")).fn_adc, None);
    assert_eq!(
        Hardware::baseos().with_fn_adc(Some("/sys/fn")).fn_adc,
        Some(PathBuf::from("/sys/fn"))
    );
    assert_eq!(rgb().with_fn_adc(None), rgb());
}

#[test]
fn the_description_file_wins_over_os_release() {
    let dir = tempfile::tempdir().unwrap();
    let (desc, release) = (
        dir.path().join("slot-device"),
        dir.path().join("os-release"),
    );
    std::fs::write(&release, "OS_NAME=\"ROCKNIX\"\n").unwrap();
    assert_eq!(detect(None, &desc, &release, "x").name, "ROCKNIX, x");
    std::fs::write(&desc, SLOTSQ).unwrap();
    assert_eq!(detect(None, &desc, &release, "x"), Hardware::parse(SLOTSQ));
}

#[test]
fn slot_platform_wins_over_everything() {
    let dir = tempfile::tempdir().unwrap();
    let desc = dir.path().join("slot-device");
    std::fs::write(&desc, SLOTSQ).unwrap();
    let missing = dir.path().join("missing");
    assert_eq!(
        detect(Some("rocknix"), &desc, &missing, slot::rocknix::RGB20SX),
        Hardware::rocknix(slot::rocknix::RGB20SX)
    );
    assert_eq!(
        detect(Some("baseos"), &desc, &missing, ""),
        Hardware::baseos()
    );
    assert_eq!(detect(None, &missing, &missing, ""), Hardware::baseos());
}
