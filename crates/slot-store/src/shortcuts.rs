//! Slot²: shortcuts that belong to systems (design/shortcuts.md). General shortcuts keep
//! upstream's `Config/shortcuts.txt`; a system's own (its colour correction, palettes,
//! stretch, sunlight) have keys per system, in `Config/shortcuts/<system>.txt`, starting
//! from the keys `shortcuts.txt` has for them and holding only the rows changed since.

use std::path::{Path, PathBuf};

use crate::bindings::{Binding, Shortcut, ShortcutKeys};
use crate::controls::{system_key, SYSTEMS};
use crate::platform::Platform;

const DIR: &str = "shortcuts";

/// The systems a shortcut belongs to, or `None` for a general one, which every system has.
pub fn systems_of(shortcut: Shortcut) -> Option<&'static [Platform]> {
    use Platform::*;
    Some(match shortcut {
        Shortcut::ColorCorrection => &[Gb, Gbc, Gba, Gbav],
        Shortcut::PaletteBack | Shortcut::PaletteNext => &[Gb],
        Shortcut::SunlightUp | Shortcut::SunlightDown => &[Gba, Gbav],
        Shortcut::StretchOn | Shortcut::StretchOff => &[Gb, Gbc, Gg, Ngp, Tg16, Ws, Lynx],
        _ => return None,
    })
}

/// Whether a system has a shortcut: every general one, and its own.
pub fn has(platform: Platform, shortcut: Shortcut) -> bool {
    systems_of(shortcut).is_none_or(|on| on.contains(&platform))
}

/// The shortcuts of a system's own, in the order its page shows them.
pub fn own(platform: Platform) -> Vec<Shortcut> {
    Shortcut::ALL
        .into_iter()
        .filter(|s| systems_of(*s).is_some_and(|on| on.contains(&platform)))
        .collect()
}

/// One system's keys for its own shortcuts.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct SystemShortcuts {
    keys: ShortcutKeys,
}

impl SystemShortcuts {
    /// A system's keys before any of its own are changed: those `shortcuts.txt` has.
    pub fn from(general: &ShortcutKeys) -> SystemShortcuts {
        SystemShortcuts { keys: *general }
    }

    pub fn get(&self, shortcut: Shortcut) -> Option<Binding> {
        self.keys.get(shortcut)
    }

    pub fn set(&mut self, shortcut: Shortcut, binding: Option<Binding>) {
        self.keys.set(shortcut, binding);
    }

    fn text(&self, platform: Platform, base: &SystemShortcuts) -> String {
        own(platform)
            .into_iter()
            .filter(|s| self.get(*s) != base.get(*s))
            .map(|s| format!("{}={}\n", s.key(), Binding::key(self.get(s))))
            .collect()
    }
}

fn file(root: &Path, platform: Platform) -> PathBuf {
    root.join("Config")
        .join(DIR)
        .join(format!("{}.txt", system_key(platform)))
}

/// A system's keys for its own shortcuts: `general`'s, with its own file's on top.
pub fn read_system_shortcuts(
    root: &Path,
    platform: Platform,
    general: &ShortcutKeys,
) -> SystemShortcuts {
    let mut keys = SystemShortcuts::from(general);
    let Ok(text) = std::fs::read_to_string(file(root, platform)) else {
        return keys;
    };
    for (key, value) in text.lines().filter_map(|l| l.split_once('=')) {
        let shortcut = own(platform)
            .into_iter()
            .find(|s| s.key().eq_ignore_ascii_case(key.trim()));
        if let (Some(shortcut), Some(binding)) = (shortcut, Binding::parse(value)) {
            keys.set(shortcut, binding);
        }
    }
    keys
}

pub fn write_system_shortcuts(
    root: &Path,
    platform: Platform,
    keys: &SystemShortcuts,
    general: &ShortcutKeys,
) -> std::io::Result<()> {
    let path = file(root, platform);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let text = keys.text(platform, &SystemShortcuts::from(general));
    crate::atomic::atomic_write(&path, text.as_bytes())
}

/// Every system's keys for its own shortcuts (`Gbav` shares the GBA's).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct AllShortcuts {
    sets: [SystemShortcuts; SYSTEMS.len()],
}

fn at(platform: Platform) -> usize {
    let platform = match platform {
        Platform::Gbav => Platform::Gba,
        p => p,
    };
    SYSTEMS.iter().position(|p| *p == platform).unwrap_or(0)
}

impl AllShortcuts {
    pub fn from(general: &ShortcutKeys) -> AllShortcuts {
        AllShortcuts {
            sets: SYSTEMS.map(|_| SystemShortcuts::from(general)),
        }
    }

    pub fn read(root: &Path, general: &ShortcutKeys) -> AllShortcuts {
        AllShortcuts {
            sets: SYSTEMS.map(|p| read_system_shortcuts(root, p, general)),
        }
    }

    pub fn get(&self, platform: Platform) -> &SystemShortcuts {
        &self.sets[at(platform)]
    }

    pub fn get_mut(&mut self, platform: Platform) -> &mut SystemShortcuts {
        &mut self.sets[at(platform)]
    }
}
