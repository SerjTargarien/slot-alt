use slot_input::{anywhere, Action, Contexts, Entry};
use slot_store::controls::{inputs, Input, SystemControls};
use slot_store::{Binding, CarouselAction, CarouselKeys, Modifier, Platform, ShortcutKeys};
use slot_ui::QuickRow;

use super::bind::{chord, shortcut_action};

#[derive(Copy, Clone, Debug)]
struct Row {
    row: QuickRow,
    binding: Binding,
    action: Option<Action>,
    contexts: Contexts,
}

pub struct Table {
    rows: Vec<Row>,
    stretches: bool,
}

impl Table {
    /// The bindings in play: one system's controls (`stretches` says whether its games
    /// take the Stretch shortcuts), the shortcuts and the shelf's keys.
    pub fn new(
        controls: &SystemControls,
        platform: Platform,
        stretches: bool,
        shortcuts: &ShortcutKeys,
        carousel: &CarouselKeys,
        turbo: bool,
    ) -> Table {
        let game = inputs(platform)
            .iter()
            .filter(|i| turbo || !matches!(i, Input::TurboA | Input::TurboB))
            .flat_map(|&input| {
                controls.bindings(input).map(move |binding| Row {
                    row: QuickRow::of_input(input),
                    binding,
                    action: None,
                    contexts: Contexts::GAME,
                })
            });
        let rest = QuickRow::ALL.into_iter().filter_map(|row| {
            if let Some(s) = row.shortcut() {
                let action = shortcut_action(s);
                return shortcuts.get(s).map(|binding| Row {
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
            stretches,
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
    /// against its own inputs and the shortcuts; the Stretch shortcuts only where that
    /// system stretches.
    pub fn clash(&self, row: QuickRow) -> Option<QuickRow> {
        let console = |r: &Row| r.action.is_none();
        self.rows.iter().filter(|m| m.row == row).find_map(|mine| {
            self.rows
                .iter()
                .filter(|r| r.row != row && r.contexts.meets(mine.contexts))
                .filter(|r| !self.apart(mine.row, r.row))
                .find(|r| {
                    r.binding == mine.binding
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
    /// Rows that never apply in the same game: the Stretch shortcuts and the inputs of a
    /// system that doesn't stretch.
    fn apart(&self, a: QuickRow, b: QuickRow) -> bool {
        let stretch = |r: QuickRow| matches!(r, QuickRow::ScStretchOn | QuickRow::ScStretchOff);
        let input = |r: QuickRow| r.input().is_some();
        !self.stretches && ((stretch(a) && input(b)) || (input(a) && stretch(b)))
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
