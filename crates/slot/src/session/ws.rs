//! Slot²: WonderSwan orientation (design/ws-orientation.md): Rotate turns a game by hand,
//! and the LCD's orientation icons turn it by themselves. Moved out of session.rs (the
//! structure pass), so Slot²'s code isn't interleaved with upstream's.

use super::*;

impl Session {
    /// Slot²: Rotate turns the WonderSwan game between upright and wide, and the game
    /// keeps the way it was left.
    pub(super) fn sync_ws(&mut self, controls: &SystemControls, platform: Platform) {
        if platform != Platform::Ws {
            self.ws_rotate_held = false;
            return;
        }
        let down = self
            .pad
            .pressed(controls, platform, false)
            .contains(&Input::Rotate);
        let fresh = down && !self.ws_rotate_held;
        self.ws_rotate_held = down;
        // The game says how it is held by lighting the LCD's vertical or horizontal
        // icon; when it says something new, the picture follows. Rotate turns it by hand
        // in between.
        let said = self
            .emu
            .as_ref()
            .and_then(|e| e.lcd_icons())
            .and_then(ws_icon_says);
        let told = said.is_some() && said != self.ws_icon_said;
        if said.is_some() {
            self.ws_icon_said = said;
        }
        let to = match (fresh && self.playing(), told) {
            (true, _) => !self.ws_portrait,
            (false, true) => said.unwrap_or(self.ws_portrait),
            (false, false) => return,
        };
        if to == self.ws_portrait {
            return;
        }
        self.ws_portrait = to;
        if let Some(emu) = &self.emu {
            emu.set_option(WS_ROTATE, ws_rotation(to));
        }
        self.app.keep_ws_portrait(to);
    }
}

/// The core's own turn: Portrait draws the WonderSwan held upright, Landscape wide.
pub const WS_ROTATE: &str = "wswan_rotate_display";

/// What the WonderSwan's LCD icons say about how the game is held: upright (bit 1, the
/// vertical icon) or wide (bit 2), or nothing when neither or both are lit.
pub(super) fn ws_icon_says(icons: u8) -> Option<bool> {
    match (icons & 0b010 != 0, icons & 0b100 != 0) {
        (true, false) => Some(true),
        (false, true) => Some(false),
        _ => None,
    }
}

pub fn ws_rotation(portrait: bool) -> &'static str {
    match portrait {
        true => "portrait",
        false => "landscape",
    }
}

#[cfg(test)]
mod ws_icon_tests {
    use super::ws_icon_says;

    #[test]
    fn the_lcds_icons_say_upright_or_wide_or_nothing() {
        assert_eq!(ws_icon_says(0b010), Some(true), "the vertical icon");
        assert_eq!(ws_icon_says(0b100), Some(false), "the horizontal icon");
        assert_eq!(
            ws_icon_says(0b111_011),
            Some(true),
            "the dots and sleep don't count"
        );
        assert_eq!(ws_icon_says(0), None);
        assert_eq!(ws_icon_says(0b110), None, "both lit says nothing");
    }
}
