use slot_input::{anywhere, Action, Contexts, Entry};
use slot_store::controls::{inputs, SystemControls};
use slot_store::shortcuts::{has, systems_of, SystemShortcuts};
use slot_store::{Binding, CarouselAction, CarouselKeys, Modifier, Platform, ShortcutKeys};
use slot_ui::QuickRow;

use super::bind::{chord, collides, device_btn, shortcut_action};

#[derive(Copy, Clone, Debug)]
struct Row {
    row: QuickRow,
    binding: Binding,
    action: Option<Action>,
    contexts: Contexts,
}

pub struct Table {
    rows: Vec<Row>,
}

impl Table {
    /// The bindings in play: one system's controls, the general shortcuts and that
    /// system's own (`system`; another system's aren't in play), and the shelf's keys.
    pub fn new(
        controls: &SystemControls,
        platform: Platform,
        shortcuts: &ShortcutKeys,
        system: &SystemShortcuts,
        carousel: &CarouselKeys,
    ) -> Table {
        let game = inputs(platform).iter().flat_map(|&input| {
            controls.bindings(input).map(move |binding| Row {
                row: QuickRow::of_input(input),
                binding,
                // A SELECT chord is told apart from SELECT by the gesture layer, which
                // holds SELECT back while it might be a chord.
                action: match binding.modifier {
                    Some(Modifier::Select) => {
                        Some(Action::SelectChordDown(device_btn(binding.button)))
                    }
                    _ => None,
                },
                contexts: Contexts::GAME,
            })
        });
        let rest = QuickRow::ALL.into_iter().filter_map(|row| {
            if let Some(s) = row.shortcut() {
                if !has(platform, s) {
                    return None;
                }
                let action = shortcut_action(s);
                let binding = match systems_of(s) {
                    Some(_) => system.get(s),
                    None => shortcuts.get(s),
                };
                return binding.map(|binding| Row {
                    row,
                    binding,
                    action: Some(action),
                    contexts: match (anywhere(action), binding.modifier) {
                        (false, _) => Contexts::GAME,
                        (true, None) => Contexts::GAME.with(Contexts::SHELF),
                        (true, Some(_)) => Contexts::ALL,
                    },
                });
            }
            let a = row.carousel_action()?;
            carousel.get(a).map(|binding| Row {
                row,
                binding,
                action: Some(shelf_action(a)),
                contexts: Contexts::SHELF,
            })
        });
        Table {
            rows: game.chain(rest).collect(),
        }
    }

    pub fn entries(&self) -> Vec<Entry> {
        self.rows
            .iter()
            .filter_map(|r| {
                r.action.map(|action| Entry {
                    chord: chord(r.binding),
                    action,
                    contexts: r.contexts,
                })
            })
            .collect()
    }

    /// Another row sharing one of this row's bindings. A system's inputs are checked
    /// against its own inputs and the shortcuts it has. A shortcut that needs a button an
    /// input uses clashes with it (in a game the input wins and the shortcut is left out).
    pub fn clash(&self, row: QuickRow) -> Option<QuickRow> {
        let console = |r: &Row| r.action.is_none();
        let game = |r: &Row| r.row.input().is_some();
        self.rows.iter().filter(|m| m.row == row).find_map(|mine| {
            self.rows
                .iter()
                .filter(|r| r.row != row && r.contexts.meets(mine.contexts))
                .filter(|r| !self.apart(mine.row, r.row))
                .find(|r| {
                    r.binding == mine.binding
                        || (game(mine) != game(r)
                            && collides(chord(mine.binding), chord(r.binding)))
                        || (!console(mine)
                            && !console(r)
                            && (overlaps(mine.binding, r.binding)
                                || overlaps(r.binding, mine.binding)))
                })
                .map(|r| r.row)
        })
    }
}

impl Table {
    /// Rows that never apply at the same moment: the DS's Touch, which only acts while the
    /// stylus is out, and the game's other inputs.
    fn apart(&self, a: QuickRow, b: QuickRow) -> bool {
        let input = |r: QuickRow| r.input().is_some();
        let touch = |r: QuickRow| r == QuickRow::GameTouch;
        (touch(a) && input(b)) || (input(a) && touch(b))
    }
}

fn shelf_action(a: CarouselAction) -> Action {
    match a {
        CarouselAction::PreviousShelf => Action::PreviousShelf,
        CarouselAction::NextShelf => Action::NextShelf,
        CarouselAction::Favorite => Action::Favorite,
        CarouselAction::ForgetRecent => Action::ForgetRecent,
    }
}

fn overlaps(pair: Binding, single: Binding) -> bool {
    let Some(Modifier::Button(first)) = pair.modifier else {
        return false;
    };
    single.modifier.is_none()
        && (single.button == first || (!first.leads() && single.button == pair.button))
}
