//! Slot²: the game in the slot's own shortcut keys over its system's and the general ones
//! (design/shortcuts.md). Moved out of app.rs (the structure pass).

use super::*;

impl App {
    /// Slot²: the general shortcuts' keys and `platform`'s own, with the game in the
    /// slot's own keys over both.
    pub(super) fn shortcut_keys(&self, platform: Platform) -> (ShortcutKeys, SystemShortcuts) {
        let (mut general, mut system) = (self.shortcuts, *self.system_shortcuts.get(platform));
        if let Some(game) = self.game_in_slot() {
            for s in Shortcut::ALL {
                match (game.shortcuts.get(s), feature_of(s)) {
                    (Some(key), None) => general.set(s, key),
                    (Some(key), Some(_)) => system.set(s, key),
                    (None, _) => {}
                }
            }
        }
        (general, system)
    }

    /// A key of the game in the slot's own. A key set back to the shelf's follows the
    /// shelf again, as a Screen value does. As on the shelf, Fast Forward and its toggle
    /// never share one: the other is left unbound in this game.
    pub(super) fn set_game_key(&mut self, shortcut: Shortcut, binding: Option<Binding>) {
        let rival = match shortcut {
            Shortcut::FastForward => Some(Shortcut::FastForwardToggle),
            Shortcut::FastForwardToggle => Some(Shortcut::FastForward),
            _ => None,
        };
        let platform = self.playing_platform();
        let (general, _) = self.shortcut_keys(platform);
        let shelf = |s: Shortcut| match feature_of(s) {
            Some(_) => self.system_shortcuts.get(platform).get(s),
            None => self.shortcuts.get(s),
        };
        let keys = [shortcut]
            .into_iter()
            .map(|s| (s, binding, shelf(s)))
            .chain(
                rival
                    .filter(|r| binding.is_some() && general.get(*r) == binding)
                    .map(|r| (r, None, shelf(r))),
            )
            .collect::<Vec<_>>();
        let Some(game) = &mut self.game else {
            return;
        };
        for (s, key, shelf) in keys {
            match key == shelf {
                true => game.shortcuts.clear(s),
                false => game.shortcuts.set(s, key),
            }
        }
        self.keep_game();
    }
}
