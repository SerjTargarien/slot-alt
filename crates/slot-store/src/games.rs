//! Slot²: a game's own settings over its system's (design/game-overrides.md). Each game
//! that changed something has a file, `Config/games/<system>/<file name>.txt`, holding
//! only what it changed; a game without one follows its system.

use std::path::{Path, PathBuf};

use crate::bindings::{Binding, Shortcut};
use crate::controls::{inputs, parse_row, row_text, system_key, Input, Row, SystemControls};
use crate::core_palette::core_palettes;
use crate::gb_palette::{GbColors, GbPalette};
use crate::platform::Platform;
use crate::slot_state::{
    ff_speed_text, ff_speed_value, BezelPick, Blending, Scale, Shader, SlotState, Stretch,
};

const DIR: &str = "games";

/// One system's screen settings, as `slot.state` keeps them. A system without one of them
/// (colour correction on the WonderSwan) reads its stock value and ignores a change.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Screen {
    pub shader: Shader,
    pub colour_correction: bool,
    /// The Game Boy's colours: one value for every system in `slot.state`.
    pub palette: GbColors,
    /// The palette the WonderSwan's or the TurboGrafx-16's core draws in
    /// (`core_palette`); empty for other systems.
    pub core_palette: String,
    pub bezel: BezelPick,
    pub scale: Scale,
    pub stretch: Stretch,
    /// Stretch On or Off on a panel taller than 4:3.
    pub stretched: bool,
    pub blending: Blending,
}

impl SlotState {
    pub fn screen(&self, platform: Platform) -> Screen {
        use Platform::*;
        let s = self;
        Screen {
            shader: match platform {
                Gba | Gbav => s.shader_gba,
                Gb => s.shader_gb,
                Gbc => s.shader_gbc,
                Ws => s.shader_ws,
                Lynx => s.shader_lynx,
                Gg => s.shader_gg,
                Ngp => s.shader_ngp,
                Tg16 => s.shader_tg16,
                Nds => s.shader_nds,
            },
            colour_correction: match platform {
                Gba | Gbav => s.colour_correction_gba,
                Gb => s.colour_correction_gb,
                Gbc => s.colour_correction_gbc,
                _ => false,
            },
            palette: s.gb_colors(),
            core_palette: match platform {
                Ws => s.palette_ws.clone(),
                Tg16 => s.palette_tg16.clone(),
                _ => String::new(),
            },
            bezel: match platform {
                Gba | Gbav => s.bezel_gba.clone(),
                Gb => s.bezel_gb.clone(),
                Gbc => s.bezel_gbc.clone(),
                Ws => s.bezel_ws.clone(),
                Lynx => s.bezel_lynx.clone(),
                Gg => s.bezel_gg.clone(),
                Ngp => s.bezel_ngp.clone(),
                Tg16 => s.bezel_tg16.clone(),
                Nds => s.bezel_nds.clone(),
            },
            scale: match platform {
                Gba | Gbav => s.scale_gba,
                Gb => s.scale_gb,
                Gbc => s.scale_gbc,
                Ws => s.scale_ws,
                Lynx => s.scale_lynx,
                Gg => s.scale_gg,
                Ngp => s.scale_ngp,
                Tg16 => s.scale_tg16,
                Nds => s.scale_nds,
            },
            stretch: match platform {
                Gb => s.stretch_gb,
                Gbc => s.stretch_gbc,
                _ => Stretch::default(),
            },
            stretched: match platform {
                Gba | Gbav => false,
                Gb => s.stretched_gb,
                Gbc => s.stretched_gbc,
                Ws => s.stretched_ws,
                Lynx => s.stretched_lynx,
                Gg => s.stretched_gg,
                Ngp => s.stretched_ngp,
                Tg16 => s.stretched_tg16,
                Nds => s.stretched_nds,
            },
            blending: match platform {
                Gba | Gbav => s.blending_gba,
                Gb => s.blending_gb,
                Gbc => s.blending_gbc,
                _ => Blending::default(),
            },
        }
    }

    pub fn set_screen(&mut self, platform: Platform, to: &Screen) {
        use Platform::*;
        let s = self;
        let shader = match platform {
            Gba | Gbav => &mut s.shader_gba,
            Gb => &mut s.shader_gb,
            Gbc => &mut s.shader_gbc,
            Ws => &mut s.shader_ws,
            Lynx => &mut s.shader_lynx,
            Gg => &mut s.shader_gg,
            Ngp => &mut s.shader_ngp,
            Tg16 => &mut s.shader_tg16,
            Nds => &mut s.shader_nds,
        };
        *shader = to.shader;
        match platform {
            Gba | Gbav => s.colour_correction_gba = to.colour_correction,
            Gb => s.colour_correction_gb = to.colour_correction,
            Gbc => s.colour_correction_gbc = to.colour_correction,
            _ => {}
        }
        s.set_gb_colors(to.palette);
        match platform {
            Ws => s.palette_ws = to.core_palette.clone(),
            Tg16 => s.palette_tg16 = to.core_palette.clone(),
            _ => {}
        }
        let bezel = match platform {
            Gba | Gbav => &mut s.bezel_gba,
            Gb => &mut s.bezel_gb,
            Gbc => &mut s.bezel_gbc,
            Ws => &mut s.bezel_ws,
            Lynx => &mut s.bezel_lynx,
            Gg => &mut s.bezel_gg,
            Ngp => &mut s.bezel_ngp,
            Tg16 => &mut s.bezel_tg16,
            Nds => &mut s.bezel_nds,
        };
        *bezel = to.bezel.clone();
        let scale = match platform {
            Gba | Gbav => &mut s.scale_gba,
            Gb => &mut s.scale_gb,
            Gbc => &mut s.scale_gbc,
            Ws => &mut s.scale_ws,
            Lynx => &mut s.scale_lynx,
            Gg => &mut s.scale_gg,
            Ngp => &mut s.scale_ngp,
            Tg16 => &mut s.scale_tg16,
            Nds => &mut s.scale_nds,
        };
        *scale = to.scale;
        match platform {
            Gb => s.stretch_gb = to.stretch,
            Gbc => s.stretch_gbc = to.stretch,
            _ => {}
        }
        match platform {
            Gba | Gbav => {}
            Gb => s.stretched_gb = to.stretched,
            Gbc => s.stretched_gbc = to.stretched,
            Ws => s.stretched_ws = to.stretched,
            Lynx => s.stretched_lynx = to.stretched,
            Gg => s.stretched_gg = to.stretched,
            Ngp => s.stretched_ngp = to.stretched,
            Tg16 => s.stretched_tg16 = to.stretched,
            Nds => s.stretched_nds = to.stretched,
        }
        match platform {
            Gba | Gbav => s.blending_gba = to.blending,
            Gb => s.blending_gb = to.blending,
            Gbc => s.blending_gbc = to.blending,
            _ => {}
        }
    }
}

/// A screen setting a game can have its own value for.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ScreenSetting {
    Shader,
    ColourCorrection,
    Palette,
    Bezel,
    Scale,
    Stretch,
    Stretched,
    Blending,
}

impl ScreenSetting {
    pub const ALL: [ScreenSetting; 8] = [
        ScreenSetting::Shader,
        ScreenSetting::ColourCorrection,
        ScreenSetting::Palette,
        ScreenSetting::Bezel,
        ScreenSetting::Scale,
        ScreenSetting::Stretch,
        ScreenSetting::Stretched,
        ScreenSetting::Blending,
    ];

    fn key(self) -> &'static str {
        match self {
            ScreenSetting::Shader => "shader",
            ScreenSetting::ColourCorrection => "colour_correction",
            ScreenSetting::Palette => "palette",
            ScreenSetting::Bezel => "bezel",
            ScreenSetting::Scale => "scale",
            ScreenSetting::Stretch => "stretch",
            ScreenSetting::Stretched => "stretched",
            ScreenSetting::Blending => "blending",
        }
    }
}

/// A game's own screen values: each `None` follows its system.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ScreenOverrides {
    pub shader: Option<Shader>,
    pub colour_correction: Option<bool>,
    pub palette: Option<GbColors>,
    pub core_palette: Option<String>,
    pub bezel: Option<BezelPick>,
    pub scale: Option<Scale>,
    pub stretch: Option<Stretch>,
    pub stretched: Option<bool>,
    pub blending: Option<Blending>,
}

impl ScreenOverrides {
    /// The game's screen: its own values over `system`.
    pub fn apply(&self, system: &Screen) -> Screen {
        Screen {
            shader: self.shader.unwrap_or(system.shader),
            colour_correction: self.colour_correction.unwrap_or(system.colour_correction),
            palette: self.palette.unwrap_or(system.palette),
            core_palette: self
                .core_palette
                .clone()
                .unwrap_or_else(|| system.core_palette.clone()),
            bezel: self.bezel.clone().unwrap_or_else(|| system.bezel.clone()),
            scale: self.scale.unwrap_or(system.scale),
            stretch: self.stretch.unwrap_or(system.stretch),
            stretched: self.stretched.unwrap_or(system.stretched),
            blending: self.blending.unwrap_or(system.blending),
        }
    }

    /// Keeps as the game's own every value of `now` that isn't `system`'s. A value set
    /// back to the system's follows the system again.
    pub fn record(&mut self, system: &Screen, now: &Screen) {
        fn own<T: PartialEq + Clone>(now: &T, system: &T) -> Option<T> {
            (now != system).then(|| now.clone())
        }
        *self = ScreenOverrides {
            shader: own(&now.shader, &system.shader),
            colour_correction: own(&now.colour_correction, &system.colour_correction),
            palette: own(&now.palette, &system.palette),
            core_palette: own(&now.core_palette, &system.core_palette),
            bezel: own(&now.bezel, &system.bezel),
            scale: own(&now.scale, &system.scale),
            stretch: own(&now.stretch, &system.stretch),
            stretched: own(&now.stretched, &system.stretched),
            blending: own(&now.blending, &system.blending),
        };
    }

    pub fn has(&self, setting: ScreenSetting) -> bool {
        match setting {
            ScreenSetting::Shader => self.shader.is_some(),
            ScreenSetting::ColourCorrection => self.colour_correction.is_some(),
            ScreenSetting::Palette => self.palette.is_some() || self.core_palette.is_some(),
            ScreenSetting::Bezel => self.bezel.is_some(),
            ScreenSetting::Scale => self.scale.is_some(),
            ScreenSetting::Stretch => self.stretch.is_some(),
            ScreenSetting::Stretched => self.stretched.is_some(),
            ScreenSetting::Blending => self.blending.is_some(),
        }
    }

    /// The setting follows the system again.
    pub fn clear(&mut self, setting: ScreenSetting) {
        match setting {
            ScreenSetting::Shader => self.shader = None,
            ScreenSetting::ColourCorrection => self.colour_correction = None,
            ScreenSetting::Palette => {
                self.palette = None;
                self.core_palette = None;
            }
            ScreenSetting::Bezel => self.bezel = None,
            ScreenSetting::Scale => self.scale = None,
            ScreenSetting::Stretch => self.stretch = None,
            ScreenSetting::Stretched => self.stretched = None,
            ScreenSetting::Blending => self.blending = None,
        }
    }

    fn value(&self, setting: ScreenSetting) -> Option<String> {
        let on = |b: bool| if b { "true" } else { "false" }.to_string();
        match setting {
            ScreenSetting::Shader => self.shader.map(|s| s.name().into()),
            ScreenSetting::ColourCorrection => self.colour_correction.map(on),
            ScreenSetting::Palette => match &self.core_palette {
                Some(key) => Some(key.clone()),
                None => self.palette.map(|p| p.core_name().into()),
            },
            ScreenSetting::Bezel => self.bezel.as_ref().map(BezelPick::key),
            ScreenSetting::Scale => self.scale.map(|s| s.name().into()),
            ScreenSetting::Stretch => self.stretch.map(|s| s.name().into()),
            ScreenSetting::Stretched => self.stretched.map(on),
            ScreenSetting::Blending => self.blending.map(|b| b.key().into()),
        }
    }

    /// Takes `key=value` if it is one of these settings; whether it was. A WonderSwan's
    /// or TurboGrafx-16's `palette` is its core's, a Game Boy's its `GbColors`.
    fn read(&mut self, platform: Platform, key: &str, value: &str) -> bool {
        if key == "palette" && !core_palettes(platform).is_empty() {
            self.core_palette = core_palettes(platform)
                .iter()
                .find(|(k, _)| *k == value)
                .map(|(k, _)| k.to_string());
            return true;
        }
        let Some(setting) = ScreenSetting::ALL.into_iter().find(|s| s.key() == key) else {
            return false;
        };
        let on = || match value {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        };
        match setting {
            ScreenSetting::Shader => self.shader = Shader::parse(value),
            ScreenSetting::ColourCorrection => self.colour_correction = on(),
            ScreenSetting::Palette => {
                self.palette = match value {
                    "Auto" => Some(GbColors::Auto),
                    name => GbPalette::parse(name).map(GbColors::Palette),
                }
            }
            ScreenSetting::Bezel => self.bezel = Some(BezelPick::parse(value)),
            ScreenSetting::Scale => self.scale = Scale::parse(value),
            ScreenSetting::Stretch => self.stretch = Stretch::parse(value),
            ScreenSetting::Stretched => self.stretched = on(),
            ScreenSetting::Blending => self.blending = Blending::parse(value),
        }
        true
    }
}

/// The Gameplay page's settings: one value for every game in `slot.state`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Play {
    pub ff_speed: u8,
    pub ff_sound: bool,
    pub rewind: bool,
    pub rumble: bool,
    pub auto_load: bool,
    pub eject_save: bool,
}

impl SlotState {
    pub fn play(&self) -> Play {
        Play {
            ff_speed: self.ff_speed,
            ff_sound: self.ff_sound,
            rewind: self.rewind,
            rumble: self.rumble,
            auto_load: self.auto_load,
            eject_save: self.eject_save,
        }
    }

    pub fn set_play(&mut self, to: &Play) {
        self.ff_speed = to.ff_speed;
        self.ff_sound = to.ff_sound;
        self.rewind = to.rewind;
        self.rumble = to.rumble;
        self.auto_load = to.auto_load;
        self.eject_save = to.eject_save;
    }
}

/// A Gameplay setting a game can have its own value for.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PlaySetting {
    FfSpeed,
    FfSound,
    Rewind,
    Rumble,
    AutoLoad,
    EjectSave,
}

impl PlaySetting {
    pub const ALL: [PlaySetting; 6] = [
        PlaySetting::FfSpeed,
        PlaySetting::FfSound,
        PlaySetting::Rewind,
        PlaySetting::Rumble,
        PlaySetting::AutoLoad,
        PlaySetting::EjectSave,
    ];

    fn key(self) -> &'static str {
        match self {
            PlaySetting::FfSpeed => "ff_speed",
            PlaySetting::FfSound => "ff_sound",
            PlaySetting::Rewind => "rewind",
            PlaySetting::Rumble => "rumble",
            PlaySetting::AutoLoad => "auto_load",
            PlaySetting::EjectSave => "eject_save",
        }
    }
}

/// A game's own Gameplay values: each `None` follows the shelf's.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct PlayOverrides {
    pub ff_speed: Option<u8>,
    pub ff_sound: Option<bool>,
    pub rewind: Option<bool>,
    pub rumble: Option<bool>,
    pub auto_load: Option<bool>,
    pub eject_save: Option<bool>,
}

impl PlayOverrides {
    pub fn apply(&self, shelf: &Play) -> Play {
        Play {
            ff_speed: self.ff_speed.unwrap_or(shelf.ff_speed),
            ff_sound: self.ff_sound.unwrap_or(shelf.ff_sound),
            rewind: self.rewind.unwrap_or(shelf.rewind),
            rumble: self.rumble.unwrap_or(shelf.rumble),
            auto_load: self.auto_load.unwrap_or(shelf.auto_load),
            eject_save: self.eject_save.unwrap_or(shelf.eject_save),
        }
    }

    /// As `ScreenOverrides::record`.
    pub fn record(&mut self, shelf: &Play, now: &Play) {
        fn own<T: PartialEq + Copy>(now: T, shelf: T) -> Option<T> {
            (now != shelf).then_some(now)
        }
        *self = PlayOverrides {
            ff_speed: own(now.ff_speed, shelf.ff_speed),
            ff_sound: own(now.ff_sound, shelf.ff_sound),
            rewind: own(now.rewind, shelf.rewind),
            rumble: own(now.rumble, shelf.rumble),
            auto_load: own(now.auto_load, shelf.auto_load),
            eject_save: own(now.eject_save, shelf.eject_save),
        };
    }

    pub fn has(&self, setting: PlaySetting) -> bool {
        match setting {
            PlaySetting::FfSpeed => self.ff_speed.is_some(),
            PlaySetting::FfSound => self.ff_sound.is_some(),
            PlaySetting::Rewind => self.rewind.is_some(),
            PlaySetting::Rumble => self.rumble.is_some(),
            PlaySetting::AutoLoad => self.auto_load.is_some(),
            PlaySetting::EjectSave => self.eject_save.is_some(),
        }
    }

    pub fn clear(&mut self, setting: PlaySetting) {
        match setting {
            PlaySetting::FfSpeed => self.ff_speed = None,
            PlaySetting::FfSound => self.ff_sound = None,
            PlaySetting::Rewind => self.rewind = None,
            PlaySetting::Rumble => self.rumble = None,
            PlaySetting::AutoLoad => self.auto_load = None,
            PlaySetting::EjectSave => self.eject_save = None,
        }
    }

    fn value(&self, setting: PlaySetting) -> Option<String> {
        let on = |b: bool| if b { "true" } else { "false" }.to_string();
        match setting {
            PlaySetting::FfSpeed => self.ff_speed.map(|s| ff_speed_text(s).into()),
            PlaySetting::FfSound => self.ff_sound.map(on),
            PlaySetting::Rewind => self.rewind.map(on),
            PlaySetting::Rumble => self.rumble.map(on),
            PlaySetting::AutoLoad => self.auto_load.map(on),
            PlaySetting::EjectSave => self.eject_save.map(on),
        }
    }

    fn read(&mut self, key: &str, value: &str) -> bool {
        let Some(setting) = PlaySetting::ALL.into_iter().find(|s| s.key() == key) else {
            return false;
        };
        let on = match value {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        };
        match setting {
            PlaySetting::FfSpeed => self.ff_speed = ff_speed_value(value),
            PlaySetting::FfSound => self.ff_sound = on,
            PlaySetting::Rewind => self.rewind = on,
            PlaySetting::Rumble => self.rumble = on,
            PlaySetting::AutoLoad => self.auto_load = on,
            PlaySetting::EjectSave => self.eject_save = on,
        }
        true
    }
}

/// A game's own shortcut keys, over the system's and the general ones. `Some(None)`
/// leaves a shortcut unbound in this game.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct GameShortcuts {
    keys: [Option<Option<Binding>>; Shortcut::ALL.len()],
}

const SHORTCUT_PREFIX: &str = "shortcut.";

impl GameShortcuts {
    fn at(shortcut: Shortcut) -> usize {
        Shortcut::ALL
            .iter()
            .position(|s| *s == shortcut)
            .unwrap_or(0)
    }

    /// The game's own key for `shortcut`, if it has one (which may be none at all).
    pub fn get(&self, shortcut: Shortcut) -> Option<Option<Binding>> {
        self.keys[Self::at(shortcut)]
    }

    pub fn set(&mut self, shortcut: Shortcut, binding: Option<Binding>) {
        self.keys[Self::at(shortcut)] = Some(binding);
    }

    /// The shortcut follows the system's or the general key again.
    pub fn clear(&mut self, shortcut: Shortcut) {
        self.keys[Self::at(shortcut)] = None;
    }

    pub fn has(&self, shortcut: Shortcut) -> bool {
        self.get(shortcut).is_some()
    }

    fn text(&self) -> String {
        Shortcut::ALL
            .into_iter()
            .filter_map(|s| {
                let key = self.get(s)?;
                Some(format!(
                    "{SHORTCUT_PREFIX}{}={}\n",
                    s.key(),
                    Binding::key(key)
                ))
            })
            .collect()
    }

    /// Takes a `shortcut.` line; whether it was one. A line naming no shortcut, or a key
    /// that isn't one, is dropped (the game follows the shelf's key), so it leaves the
    /// file the next time it is written.
    fn read(&mut self, key: &str, value: &str) -> bool {
        let Some(name) = key.strip_prefix(SHORTCUT_PREFIX) else {
            return false;
        };
        let shortcut = Shortcut::ALL.into_iter().find(|s| s.key() == name);
        if let (Some(shortcut), Some(binding)) = (shortcut, Binding::parse(value)) {
            self.set(shortcut, binding);
        }
        true
    }
}

/// A game's own controls rows, over its system's: each `None` follows the system.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct GameControls {
    rows: [Option<Row>; Input::COUNT],
}

impl Default for GameControls {
    fn default() -> Self {
        GameControls {
            rows: [None; Input::COUNT],
        }
    }
}

const CONTROL_PREFIX: &str = "control.";

impl GameControls {
    fn at(input: Input) -> usize {
        Input::ALL.iter().position(|i| *i == input).unwrap_or(0)
    }

    /// The game's controls: its own rows over `system`'s.
    pub fn apply(&self, system: &SystemControls) -> SystemControls {
        let mut c = *system;
        for input in Input::ALL {
            if let Some(row) = self.rows[Self::at(input)] {
                c.set_row(input, row);
            }
        }
        c
    }

    /// Keeps as the game's own every row of `now` that isn't `system`'s.
    pub fn record(&mut self, platform: Platform, system: &SystemControls, now: &SystemControls) {
        for input in Input::ALL {
            let own = inputs(platform).contains(&input) && now.row(input) != system.row(input);
            self.rows[Self::at(input)] = own.then(|| now.row(input));
        }
    }

    pub fn has(&self, input: Input) -> bool {
        self.rows[Self::at(input)].is_some()
    }

    /// The row follows the system again.
    pub fn clear(&mut self, input: Input) {
        self.rows[Self::at(input)] = None;
    }

    fn text(&self) -> String {
        Input::ALL
            .into_iter()
            .filter_map(|i| {
                let row = self.rows[Self::at(i)]?;
                Some(format!("{CONTROL_PREFIX}{}={}\n", i.key(), row_text(row)))
            })
            .collect()
    }

    /// Takes a `control.` line; whether it was one. One naming an input the game's system
    /// hasn't is dropped, as a shortcut line is.
    fn read(&mut self, platform: Platform, key: &str, value: &str) -> bool {
        let Some(name) = key.strip_prefix(CONTROL_PREFIX) else {
            return false;
        };
        if let Some(input) = Input::parse(name).filter(|i| inputs(platform).contains(i)) {
            self.rows[Self::at(input)] = Some(parse_row(value));
        }
        true
    }
}

/// Everything one game changed from its system.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GameSettings {
    pub platform: Platform,
    pub stem: String,
    pub screen: ScreenOverrides,
    pub play: PlayOverrides,
    pub shortcuts: GameShortcuts,
    pub controls: GameControls,
    /// A WonderSwan game's orientation as it was last left: held upright (`true`) or wide
    /// (design/ws-orientation.md). `None` starts it the way its cart says.
    pub ws_portrait: Option<bool>,
    /// Lines this build doesn't know, kept as they were.
    other: Vec<(String, String)>,
}

impl GameSettings {
    /// A game that changed nothing.
    pub fn new(platform: Platform, stem: &str) -> GameSettings {
        GameSettings {
            platform,
            stem: stem.to_string(),
            screen: ScreenOverrides::default(),
            play: PlayOverrides::default(),
            shortcuts: GameShortcuts::default(),
            controls: GameControls::default(),
            ws_portrait: None,
            other: Vec::new(),
        }
    }

    /// Whether this is the game `stem` of `platform`'s system.
    pub fn is(&self, platform: Platform, stem: &str) -> bool {
        system_key(self.platform) == system_key(platform) && self.stem == stem
    }

    fn text(&self) -> String {
        let mut text = String::new();
        for s in ScreenSetting::ALL {
            if let Some(v) = self.screen.value(s) {
                text.push_str(&format!("{}={v}\n", s.key()));
            }
        }
        for s in PlaySetting::ALL {
            if let Some(v) = self.play.value(s) {
                text.push_str(&format!("{}={v}\n", s.key()));
            }
        }
        text.push_str(&self.shortcuts.text());
        text.push_str(&self.controls.text());
        if let Some(portrait) = self.ws_portrait {
            let way = if portrait { "portrait" } else { "landscape" };
            text.push_str(&format!("{WS_ORIENTATION}={way}\n"));
        }
        for (k, v) in &self.other {
            text.push_str(&format!("{k}={v}\n"));
        }
        text
    }
}

const WS_ORIENTATION: &str = "ws_orientation";

fn read_ws_orientation(to: &mut Option<bool>, key: &str, value: &str) -> bool {
    if key != WS_ORIENTATION {
        return false;
    }
    *to = match value {
        "portrait" => Some(true),
        "landscape" => Some(false),
        _ => None,
    };
    true
}

pub fn game_file(root: &Path, platform: Platform, stem: &str) -> PathBuf {
    root.join("Config")
        .join(DIR)
        .join(system_key(platform))
        .join(format!("{stem}.txt"))
}

pub fn read_game(root: &Path, platform: Platform, stem: &str) -> GameSettings {
    let mut game = GameSettings::new(platform, stem);
    let Ok(text) = std::fs::read_to_string(game_file(root, platform, stem)) else {
        return game;
    };
    for (key, value) in text.lines().filter_map(|l| l.split_once('=')) {
        let (key, value) = (key.trim(), value.trim());
        let known = game.screen.read(platform, key, value)
            || game.play.read(key, value)
            || game.shortcuts.read(key, value)
            || game.controls.read(platform, key, value)
            || read_ws_orientation(&mut game.ws_portrait, key, value);
        if !known {
            game.other.push((key.to_string(), value.to_string()));
        }
    }
    game
}

/// Writes the game's file, or removes it once the game follows its system in everything.
pub fn write_game(root: &Path, game: &GameSettings) -> std::io::Result<()> {
    let path = game_file(root, game.platform, &game.stem);
    let text = game.text();
    if text.is_empty() {
        return match std::fs::remove_file(&path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        };
    }
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    crate::atomic::atomic_write(&path, text.as_bytes())
}
