//! Slot²: what Slot² tells a core beyond upstream's options: the WonderSwan's and the
//! TurboGrafx-16's palettes, and the sprite limit (design/core-options.md). Moved out of
//! app.rs (the structure pass).

use super::*;

impl App {
    /// The Palette row's value: the playing system's in a game; on the shelf, the open
    /// Screen page's system's.
    pub fn palette_label(&self) -> &'static str {
        let platform = match (self.in_game_menu(), self.quick_menu()) {
            (Some(_), _) => self.playing_platform(),
            (None, Some(row)) if row.page().contains(&QuickRow::WsPalette) => Platform::Ws,
            (None, Some(row)) if row.page().contains(&QuickRow::Tg16Palette) => Platform::Tg16,
            _ => Platform::Gb,
        };
        match core_palette::core_palettes(platform).is_empty() {
            true => self.view().gb_colors().label(),
            false => core_palette::core_palette_label(
                platform,
                &self.view().screen(platform).core_palette,
            ),
        }
    }

    /// Slot²: the playing system's core palette, as its option and value, if it has one.
    pub fn core_palette_now(&self, platform: Platform) -> Option<(&'static str, String)> {
        let key = core_palette::core_palette_option(platform)?;
        Some((key, self.view().screen(platform).core_palette))
    }

    /// Slot²: the sprite limit the running game plays with: its own, or the shelf's.
    pub fn sprite_limit(&self) -> bool {
        self.view().sprite_limit
    }

    /// A sprite limit change for the running core, once, as its option.
    pub fn take_sprite_limit(&mut self) -> Option<(&'static str, &'static str)> {
        let on = self.sprite_pending.take()?;
        crate::core::sprite_option(self.core, on)
    }

    /// A palette change for the running core, once.
    pub fn take_core_palette(&mut self) -> Option<(&'static str, String)> {
        let value = self.core_palette_pending.take()?;
        let key = core_palette::core_palette_option(self.playing_platform())?;
        Some((key, value))
    }
}
