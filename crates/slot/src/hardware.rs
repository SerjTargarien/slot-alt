//! What slot needs to know about the device under it: how to open the display, how the
//! buttons are numbered, where FN comes from, and which features the OS supports.
//!
//! An OS describes its device in `/etc/slot-device`, a `KEY=value` file like BaseOS's
//! release file (Base² ships one; see CONTRACT.md). Without one, slot treats the OS as
//! BaseOS, as upstream does.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub const DESCRIPTION: &str = "/etc/slot-device";

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Display {
    /// The Mali fbdev build BaseOS ships.
    Fbdev,
    /// DRM/KMS with the Mali GBM build.
    Kms,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Buttons {
    Anbernic,
    Gamepad,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hardware {
    pub name: String,
    pub display: Display,
    pub buttons: Buttons,
    /// Read FN from this ADC level, when the kernel doesn't report FN as a key.
    pub fn_adc: Option<PathBuf>,
    /// Cap the CPU clock to the frame budget (`clock.rs`).
    pub cpu_cap: bool,
    /// slot may run Wi-Fi (the network menu, link play, art, updates).
    pub wifi: bool,
    /// slot refreshes BaseOS's boot logo from the card.
    pub boot_logo: bool,
    /// slot may offer upstream's updates. Off where an upstream build couldn't drive the
    /// device.
    pub self_update: bool,
    /// Slot²: what the device has, where detection gets it wrong (`parts.rs`): unset,
    /// slot detects it.
    pub left_stick: Option<bool>,
    pub right_stick: Option<bool>,
    pub stick_clicks: Option<bool>,
    pub lid: Option<bool>,
}

impl Hardware {
    /// What upstream slot assumes: BaseOS.
    pub fn baseos() -> Hardware {
        Hardware {
            name: "BaseOS".into(),
            display: Display::Fbdev,
            buttons: Buttons::Anbernic,
            fn_adc: None,
            cpu_cap: false,
            wifi: true,
            boot_logo: true,
            // Slot²: off on BaseOS too. The channel is slotPlus's (and BaseOS's): an update
            // from it replaces Slot² and deletes the files of ours it doesn't list
            // (design/two-devices.md). A description can still turn it on.
            self_update: false,
            left_stick: None,
            right_stick: None,
            stick_clicks: None,
            lid: None,
        }
    }

    /// A description file. Missing keys keep the BaseOS value.
    pub fn parse(text: &str) -> Hardware {
        let base = Hardware::baseos();
        let get = |key| crate::baseos::lookup(text, key);
        let flag = |key, default| match get(key).as_deref() {
            Some("1" | "yes" | "true") => true,
            Some("0" | "no" | "false") => false,
            _ => default,
        };
        Hardware {
            name: get("NAME").unwrap_or(base.name),
            display: match get("DISPLAY").as_deref() {
                Some("kms") => Display::Kms,
                _ => base.display,
            },
            buttons: match get("BUTTONS").as_deref() {
                Some("gamepad") => Buttons::Gamepad,
                _ => base.buttons,
            },
            fn_adc: get("FN_ADC").filter(|p| !p.is_empty()).map(PathBuf::from),
            cpu_cap: flag("CPU_CAP", base.cpu_cap),
            wifi: flag("WIFI", base.wifi),
            boot_logo: flag("BOOT_LOGO", base.boot_logo),
            self_update: flag("SELF_UPDATE", base.self_update),
            left_stick: get("LEFT_STICK").and_then(|v| parse_flag(&v)),
            right_stick: get("RIGHT_STICK").and_then(|v| parse_flag(&v)),
            stick_clicks: get("STICK_CLICKS").and_then(|v| parse_flag(&v)),
            lid: get("LID").and_then(|v| parse_flag(&v)),
        }
    }

    /// `SLOT_FN_ADC` overrides where FN comes from: `0` turns the ADC off, a path reads
    /// that one. An OS sets it when the running kernel differs from its usual one.
    pub fn with_fn_adc(mut self, env: Option<&str>) -> Hardware {
        match env {
            Some("0") => self.fn_adc = None,
            Some(path) if !path.is_empty() => self.fn_adc = Some(PathBuf::from(path)),
            _ => {}
        }
        self
    }
}

fn parse_flag(value: &str) -> Option<bool> {
    match value {
        "1" | "yes" | "true" => Some(true),
        "0" | "no" | "false" => Some(false),
        _ => None,
    }
}

/// The device slot is running on, worked out once.
pub fn current() -> &'static Hardware {
    static CURRENT: OnceLock<Hardware> = OnceLock::new();
    CURRENT.get_or_init(|| {
        detect(Path::new(DESCRIPTION)).with_fn_adc(std::env::var("SLOT_FN_ADC").ok().as_deref())
    })
}

/// The OS's description file, or BaseOS, which ships none.
pub fn detect(description: &Path) -> Hardware {
    match std::fs::read_to_string(description) {
        Ok(text) => Hardware::parse(&text),
        Err(_) => Hardware::baseos(),
    }
}
