//! Slot²: what a device has beyond the buttons every handheld here has: the sticks, the
//! clicks under them, a lid (design/two-devices.md, phase 3). Detected at boot: the lid
//! from the signals slot already closes on, the sticks and their clicks from what the
//! kernel reports. Anbernic's H700 kernel reports stick axes on models without sticks, so
//! with its button layout the sticks come from upstream's list of models instead.
//! `/etc/slot-device` keys override any of it, for a device whose kernel says wrong.

use std::path::Path;

use slot_store::{Binding, Device, Modifier};

use crate::hardware::{Buttons, Hardware};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Parts {
    pub left_stick: bool,
    pub right_stick: bool,
    /// L3 and R3.
    pub stick_clicks: bool,
    pub lid: bool,
}

impl Parts {
    /// Everything: what slot assumes until the device says (and in tests).
    pub const ALL: Parts = Parts {
        left_stick: true,
        right_stick: true,
        stick_clicks: true,
        lid: true,
    };

    /// Whether the device has the button or stick direction.
    pub fn has(&self, device: Device) -> bool {
        match device {
            Device::LsUp | Device::LsDown | Device::LsLeft | Device::LsRight => self.left_stick,
            Device::RsUp | Device::RsDown | Device::RsLeft | Device::RsRight => self.right_stick,
            Device::L3 | Device::R3 => self.stick_clicks,
            _ => true,
        }
    }

    /// Whether every button a binding needs is on the device.
    pub fn can_press(&self, binding: &Binding) -> bool {
        let modifier = match binding.modifier {
            Some(Modifier::Button(m)) => self.has(m),
            _ => true,
        };
        modifier && self.has(binding.button)
    }

    /// For the log: what was found.
    pub fn report(&self) -> String {
        let yes = |b: bool| if b { "yes" } else { "no" };
        format!(
            "left stick {}, right stick {}, stick clicks {}, lid {}",
            yes(self.left_stick),
            yes(self.right_stick),
            yes(self.stick_clicks),
            yes(self.lid)
        )
    }
}

/// Anbernic's H700 handhelds with a left stick, by BaseOS's `BASEOS_TARGET` (upstream
/// slot's list, `LEFT_STICK_TARGETS`, before f5f1586 took it out). None has a right stick
/// or stick clicks its layout reads.
const ANBERNIC_LEFT_STICK: [&str; 6] = [
    "rg34xxsp",
    "rg35xxh",
    "rg35xxpro",
    "rg40xxh",
    "rg40xxv",
    "rgcubexx",
];

const ABS_X: u16 = 0x00;
const ABS_Y: u16 = 0x01;
const ABS_RX: u16 = 0x03;
const ABS_RY: u16 = 0x04;
const BTN_THUMBL: u16 = 0x13d;
const BTN_THUMBR: u16 = 0x13e;
const SW_LID: u16 = 0x00;

/// What the device has, from `sys` (normally `/sys`), its button layout and, on BaseOS,
/// its model.
pub fn detect(sys: &Path, buttons: Buttons, target: Option<&str>) -> Parts {
    let caps: Vec<[String; 3]> = std::fs::read_dir(sys.join("class/input"))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("event"))
        .map(|e| {
            let read = |f: &str| {
                std::fs::read_to_string(e.path().join("device/capabilities").join(f))
                    .unwrap_or_default()
            };
            [read("abs"), read("key"), read("sw")]
        })
        .collect();
    let any = |at: usize, codes: &[u16]| {
        caps.iter()
            .any(|c| codes.iter().all(|code| slot_power::has_bit(&c[at], *code)))
    };
    let hall = std::fs::read_dir(sys.join("class/power_supply"))
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| e.path().join("hallkey").is_file());
    let lid = any(2, &[SW_LID]) || hall;
    match buttons {
        Buttons::Gamepad => Parts {
            left_stick: any(0, &[ABS_X, ABS_Y]),
            right_stick: any(0, &[ABS_RX, ABS_RY]),
            stick_clicks: any(1, &[BTN_THUMBL]) || any(1, &[BTN_THUMBR]),
            lid,
        },
        Buttons::Anbernic => Parts {
            left_stick: target.is_some_and(|t| ANBERNIC_LEFT_STICK.contains(&t)),
            right_stick: false,
            stick_clicks: false,
            lid,
        },
    }
}

impl Parts {
    /// The description's keys win over what was detected.
    pub fn with_overrides(self, hw: &Hardware) -> Parts {
        Parts {
            left_stick: hw.left_stick.unwrap_or(self.left_stick),
            right_stick: hw.right_stick.unwrap_or(self.right_stick),
            stick_clicks: hw.stick_clicks.unwrap_or(self.stick_clicks),
            lid: hw.lid.unwrap_or(self.lid),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn node(sys: &Path, n: usize, abs: &str, key: &str, sw: &str) {
        let caps = sys.join(format!("class/input/event{n}/device/capabilities"));
        std::fs::create_dir_all(&caps).unwrap();
        std::fs::write(caps.join("abs"), abs).unwrap();
        std::fs::write(caps.join("key"), key).unwrap();
        std::fs::write(caps.join("sw"), sw).unwrap();
    }

    fn fake_sys() -> (tempfile::TempDir, PathBuf) {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().to_path_buf();
        (d, p)
    }

    #[test]
    fn the_rgb20sx_has_two_sticks_and_their_clicks_and_no_lid() {
        // As its kernel reports them on 2026-11-07: the ADC sticks' X, Y, RX and RY, and
        // the clicks among gpio-keys' buttons.
        let (_d, sys) = fake_sys();
        node(&sys, 0, "0", "100000 0 0 0", "0");
        node(&sys, 2, "1b", "0", "0");
        node(&sys, 3, "0", "6000000000000000 0 0 0 0", "0");
        let p = detect(&sys, Buttons::Gamepad, None);
        assert_eq!(
            p,
            Parts {
                left_stick: true,
                right_stick: true,
                stick_clicks: true,
                lid: false
            }
        );
    }

    #[test]
    fn the_rg34xx_s_phantom_axes_are_no_stick_and_its_model_says_so() {
        // ANBERNIC-keys reports ABS_RX, RY, RZ and the hat (30038) on a stickless model.
        let (_d, sys) = fake_sys();
        node(
            &sys,
            1,
            "30038",
            "400000000 dff000000000000 0 0 c000000000000 2",
            "0",
        );
        let p = detect(&sys, Buttons::Anbernic, Some("rg34xx"));
        assert_eq!(
            p,
            Parts {
                left_stick: false,
                right_stick: false,
                stick_clicks: false,
                lid: false
            }
        );
        assert!(detect(&sys, Buttons::Anbernic, Some("rg35xxh")).left_stick);
        assert!(!detect(&sys, Buttons::Anbernic, None).left_stick);
    }

    #[test]
    fn a_lid_is_its_switch_or_a_hall_sensor() {
        let (_d, sys) = fake_sys();
        node(&sys, 0, "0", "0", "1");
        assert!(detect(&sys, Buttons::Anbernic, None).lid);
        let (_d, sys) = fake_sys();
        let psy = sys.join("class/power_supply/axp2202-battery");
        std::fs::create_dir_all(&psy).unwrap();
        assert!(!detect(&sys, Buttons::Anbernic, None).lid);
        std::fs::write(psy.join("hallkey"), "1").unwrap();
        assert!(detect(&sys, Buttons::Anbernic, None).lid);
    }

    #[test]
    fn the_descriptions_keys_win() {
        let hw = Hardware::parse("LEFT_STICK=1\nLID=0\n");
        let found = Parts {
            left_stick: false,
            right_stick: false,
            stick_clicks: false,
            lid: true,
        };
        assert_eq!(
            found.with_overrides(&hw),
            Parts {
                left_stick: true,
                right_stick: false,
                stick_clicks: false,
                lid: false
            }
        );
        assert_eq!(found.with_overrides(&Hardware::baseos()), found);
    }

    #[test]
    fn a_binding_is_pressable_when_its_buttons_are_there() {
        let none = Parts {
            left_stick: false,
            right_stick: false,
            stick_clicks: false,
            lid: false,
        };
        assert!(!none.can_press(&Binding::plain(Device::LsUp).unwrap()));
        assert!(!none.can_press(&Binding::pair(Device::R3, Device::A).unwrap()));
        assert!(none.can_press(&Binding::plain(Device::L2).unwrap()));
        assert!(Parts::ALL.can_press(&Binding::plain(Device::RsDown).unwrap()));
    }
}
