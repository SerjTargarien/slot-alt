//! Slot²: the game in the slot's own settings over its system's (design/game-overrides.md):
//! loading a game's file, the view the app reads while a game is in the slot, and making
//! a change the game's own. Moved out of app.rs (the structure pass), so Slot²'s code isn't
//! interleaved with upstream's.

use super::*;

impl App {
    /// Slot²: reads the starting game's own settings, before its core opens.
    pub fn load_game(&mut self, platform: Platform, stem: &str) {
        self.game = Some(match &self.root {
            Some(root) => read_game(root, platform, stem),
            None => GameSettings::new(platform, stem),
        });
        self.build_game_view();
    }

    pub(super) fn build_game_view(&mut self) {
        self.game_controls = self
            .game
            .as_ref()
            .map(|g| g.controls.apply(self.controls.get(g.platform)));
        self.game_view = self.game.as_ref().map(|g| {
            let mut view = self.state.clone();
            view.set_screen(g.platform, &g.screen.apply(&self.state.screen(g.platform)));
            view.set_play(&g.play.apply(&self.state.play()));
            Box::new(view)
        });
    }

    /// Slot²: whether the cart about to go in loads its last state: its own Load State on
    /// Insert, read from its file before it is in the slot, or the shelf's.
    pub(super) fn loads_on_insert(&self) -> bool {
        let cart = self.shelf().carts.get(self.shelf().index);
        let own = match (&self.root, cart) {
            (Some(root), Some(c)) => read_game(root, c.platform, &c.stem).play.auto_load,
            _ => None,
        };
        own.unwrap_or(self.state.auto_load)
    }

    /// The stem of the game in the slot.
    pub(super) fn slot_stem(&self) -> Option<&str> {
        match &self.phase {
            Phase::Inserting { cart, .. }
            | Phase::Playing { cart }
            | Phase::Ejecting { cart, .. }
            | Phase::Polaroids { cart } => Some(cart),
            Phase::Doze { cart: Some(cart) } => Some(cart),
            _ => None,
        }
    }

    /// The game in the slot, once its settings are loaded. A game of another system with
    /// the same name is loaded in its place when it starts (`load_game`).
    pub(super) fn game_in_slot(&self) -> Option<&GameSettings> {
        let stem = self.slot_stem()?;
        self.game.as_ref().filter(|g| g.stem == stem)
    }

    /// Loads the settings of a game that came into the slot without a core starting for
    /// it here (a session resumed at boot).
    pub(super) fn follow_game(&mut self) {
        if self.game_in_slot().is_some() || self.slot_stem().is_none() {
            return;
        }
        if let Some((platform, stem)) = self.seated_cart().map(|c| (c.platform, c.stem.clone())) {
            self.load_game(platform, &stem);
        }
    }

    /// The settings a screen value is read from: the game's view while a game is in the
    /// slot, the systems' own otherwise.
    pub(super) fn view(&self) -> &SlotState {
        match (
            &self.game_view,
            self.editing_game || self.game_in_slot().is_none(),
        ) {
            (Some(view), false) => view,
            _ => &self.state,
        }
    }

    /// Makes a change (`edit`, written as for a system) to the game in the slot's own
    /// values instead: `edit` runs with `state` holding the game's view, and what it
    /// changed on the game's system is kept in the game's file. With no game in the
    /// slot, `edit` changes the systems' settings as usual.
    pub(super) fn edit_game(&mut self, edit: impl FnOnce(&mut Self)) {
        let Some(platform) = self.game_in_slot().map(|g| g.platform) else {
            return edit(self);
        };
        if self.editing_game {
            return edit(self);
        }
        let system = self.state.screen(platform);
        let shelf = self.state.play();
        let view = match self.game_view.take() {
            Some(view) => *view,
            None => self.state.clone(),
        };
        let systems = std::mem::replace(&mut self.state, view);
        self.editing_game = true;
        edit(self);
        self.editing_game = false;
        let (now, play) = (self.state.screen(platform), self.state.play());
        self.state = systems;
        if let Some(game) = &mut self.game {
            game.screen.record(&system, &now);
            game.play.record(&shelf, &play);
        }
        self.keep_game();
    }

    /// The game in the slot follows its system (or the shelf) again in `setting`.
    pub(super) fn clear_game_setting(&mut self, setting: GameSetting) {
        if self.game_in_slot().is_none() {
            return;
        }
        if let Some(game) = &mut self.game {
            match setting {
                GameSetting::Screen(s) => game.screen.clear(s),
                GameSetting::Play(s) => game.play.clear(s),
                GameSetting::Key(s) => game.shortcuts.clear(s),
                GameSetting::Control(i) => game.controls.clear(i),
            }
        }
        self.keep_game();
        match setting {
            GameSetting::Screen(ScreenSetting::ColourCorrection) => {
                self.colour_pending = Some(self.colour_correction());
            }
            GameSetting::Screen(ScreenSetting::Palette) => {
                match self.core_palette_now(self.playing_platform()) {
                    Some((_, value)) => self.core_palette_pending = Some(value),
                    None => self.palette_pending = Some(self.gb_colors()),
                }
            }
            GameSetting::Play(PlaySetting::SpriteLimit) => {
                self.sprite_pending = Some(self.view().sprite_limit)
            }
            GameSetting::Screen(ScreenSetting::Blending) => {
                self.blending_pending = Some(self.blending_in_play())
            }
            _ => {}
        }
    }

    /// Slot²: the WonderSwan game in the slot was turned; it starts that way next time.
    pub fn keep_ws_portrait(&mut self, portrait: bool) {
        if self.game_in_slot().is_none() {
            return;
        }
        if let Some(game) = &mut self.game {
            game.ws_portrait = Some(portrait);
        }
        self.keep_game();
    }

    /// Whether the game in the slot has its own `setting`.
    pub(super) fn game_owns(&self, setting: GameSetting) -> bool {
        self.game_in_slot().is_some_and(|g| match setting {
            GameSetting::Screen(s) => g.screen.has(s),
            GameSetting::Play(s) => g.play.has(s),
            GameSetting::Key(s) => g.shortcuts.has(s),
            GameSetting::Control(i) => g.controls.has(i),
        })
    }

    /// Whether an in-game row shows the game's own value. Stretch is Classic or Full
    /// Screen on a 4:3 panel, On or Off on a taller one.
    pub fn game_owns_row(&self, row: QuickRow) -> bool {
        match game_setting(row) {
            Some(GameSetting::Screen(ScreenSetting::Stretch)) => {
                self.game_owns(GameSetting::Screen(ScreenSetting::Stretch))
                    || self.game_owns(GameSetting::Screen(ScreenSetting::Stretched))
            }
            Some(setting) => self.game_owns(setting),
            None => false,
        }
    }

    /// Y on an in-game row with the game's own value: it follows its system again.
    pub(super) fn clear_game_row(&mut self, row: QuickRow) {
        match game_setting(row) {
            Some(GameSetting::Screen(ScreenSetting::Stretch)) => {
                self.clear_game_setting(GameSetting::Screen(ScreenSetting::Stretch));
                self.clear_game_setting(GameSetting::Screen(ScreenSetting::Stretched));
            }
            Some(setting) => self.clear_game_setting(setting),
            None => {}
        }
    }

    pub(super) fn keep_game(&mut self) {
        self.build_game_view();
        let (Some(root), Some(game)) = (&self.root, &self.game) else {
            return;
        };
        let written = write_game(root, game);
        if let Err(e) = &written {
            eprintln!("slot: games: {e}");
        }
        self.card_writes.wrote("games", &written);
    }
}

/// Whether a game of this platform can be stretched (the Stretch shortcuts): not the GBA,
/// whose Scale has its own full screen, nor the DS, which has screen layouts.
/// Slot²: a setting a game can have its own value for (design/game-overrides.md).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(super) enum GameSetting {
    Screen(ScreenSetting),
    Play(PlaySetting),
    Key(Shortcut),
    Control(Input),
}

/// The setting an in-game row sets as the game's own.
pub(super) fn game_setting(row: QuickRow) -> Option<GameSetting> {
    use GameSetting::{Play, Screen};
    Some(match row {
        QuickRow::Shader => Screen(ScreenSetting::Shader),
        QuickRow::ColourCorrection => Screen(ScreenSetting::ColourCorrection),
        QuickRow::Palette => Screen(ScreenSetting::Palette),
        QuickRow::Bezel => Screen(ScreenSetting::Bezel),
        QuickRow::Scale => Screen(ScreenSetting::Scale),
        QuickRow::Stretch => Screen(ScreenSetting::Stretch),
        QuickRow::Blending => Screen(ScreenSetting::Blending),
        QuickRow::FastForward => Play(PlaySetting::FfSpeed),
        QuickRow::FastForwardSound => Play(PlaySetting::FfSound),
        QuickRow::Rewind => Play(PlaySetting::Rewind),
        QuickRow::Rumble => Play(PlaySetting::Rumble),
        QuickRow::SpriteLimit => Play(PlaySetting::SpriteLimit),
        QuickRow::AutoLoad => Play(PlaySetting::AutoLoad),
        QuickRow::EjectSave => Play(PlaySetting::EjectSave),
        r => match r.input() {
            Some(input) => GameSetting::Control(input),
            None => GameSetting::Key(r.shortcut()?),
        },
    })
}

/// An in-game row whose value (not key) can be the game's own: changed with Left and
/// Right, put back with Y.
pub(super) fn game_value_row(row: QuickRow) -> bool {
    matches!(
        game_setting(row),
        Some(GameSetting::Screen(_) | GameSetting::Play(_))
    )
}
