//! Slot²: what a menu row says, as text: its name and what it shows at its right. The
//! menu map and the menu's rule tests read it, on both devices' panels
//! (design/two-devices.md, "Menus on both devices").

use super::*;

/// What the map writes for a row that draws live state as a picture of its own (the
/// clock, the wallpaper, Wi-Fi, RomM, RetroAchievements): it changes as slot runs.
pub const OWN_FACE: &str = "(its own face)";

impl App {
    /// The name a row shows: a controls row takes its system's own (the Game Gear's
    /// Button 1, the WonderSwan's X1).
    pub fn row_label(&self, row: QuickRow) -> &'static str {
        match row.input() {
            Some(input) => slot_store::controls::label(self.controls_page(), input),
            None => row.label(),
        }
    }

    /// What a row shows at its right: its value, its bindings, the name of its bezel or
    /// palette, `OWN_FACE`, or nothing.
    pub fn row_text(&self, row: QuickRow) -> String {
        if row.binds() {
            return self.bound_text(row);
        }
        match row {
            QuickRow::DateTime
            | QuickRow::Wallpaper
            | QuickRow::Wifi
            | QuickRow::Networks
            | QuickRow::Romm
            | QuickRow::RommServer
            | QuickRow::RommPairedServer
            | QuickRow::RommPort
            | QuickRow::RommStatus
            | QuickRow::RaUsername
            | QuickRow::RaPassword
            | QuickRow::RaMode => return OWN_FACE.into(),
            QuickRow::Palette
            | QuickRow::GbPalette
            | QuickRow::WsPalette
            | QuickRow::Tg16Palette => return self.palette_label().into(),
            _ => {}
        }
        let bezel = match row {
            QuickRow::GbaBezel => Some(Platform::Gba),
            QuickRow::GbBezel => Some(Platform::Gb),
            QuickRow::GbcBezel => Some(Platform::Gbc),
            QuickRow::WsBezel => Some(Platform::Ws),
            QuickRow::LynxBezel => Some(Platform::Lynx),
            QuickRow::GgBezel => Some(Platform::Gg),
            QuickRow::NgpBezel => Some(Platform::Ngp),
            QuickRow::Tg16Bezel => Some(Platform::Tg16),
            QuickRow::Bezel => Some(self.playing_platform()),
            _ => None,
        };
        if let Some(platform) = bezel {
            return self.resolved_bezel(platform).label();
        }
        self.quick_value(row)
            .map_or(String::new(), |v| v.text().to_string())
    }
}
