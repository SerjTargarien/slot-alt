//! Slot²: every menu on both devices' panels, walked the way a player does (design/two-devices.md,
//! "Menus on both devices"). Each device gets a map, `tests/menu-map/<device>.txt`: every page,
//! the rows it shows and what each shows. `differences.txt` lists what one device's menus
//! have that the other's don't. A change to the menus fails here until the maps are written
//! again, so it shows up as a readable diff:
//!
//!     SLOT_MENU_MAP=write cargo test -p slot --test tests menu_map
//!
//! The walk checks rules on every page as it goes (`rules_hold_on_every_page_of_both_devices`).

use crate::common::{app_booting_at, tmp_root_with_carts, CLOCK_IS_SET};
use crate::scale::playing;
use slot::app::{App, OWN_FACE};
use slot::parts::Parts;
use slot_input::{Action, Btn};
use slot_store::{Platform, SlotState};
use slot_ui::{QuickRow, QUICK_VALUE_MIN_SCALE};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// The two devices: the RG34XX's 3:2 panel and no sticks, the RGB20SX's square one with
/// two sticks and their clicks; neither has a lid (as `parts.rs` detects them).
const DEVICES: [(&str, (u32, u32), Parts); 2] = [
    (
        "rg34xx",
        (720, 480),
        Parts {
            left_stick: false,
            right_stick: false,
            stick_clicks: false,
            lid: false,
        },
    ),
    (
        "rgb20sx",
        (720, 720),
        Parts {
            left_stick: true,
            right_stick: true,
            stick_clicks: true,
            lid: false,
        },
    ),
];

/// Rows that show nothing at their right by design: actions, and read-outs drawn elsewhere.
const SHOW_NOTHING: [QuickRow; 0] = [];

/// Rows whose value isn't the player's to change with Left and Right.
const READ_ONLY: [QuickRow; 3] = [QuickRow::DateTime, QuickRow::About, QuickRow::RommStatus];

struct Line {
    depth: usize,
    row: QuickRow,
    label: &'static str,
    text: String,
    greyed: bool,
}

struct Walk {
    in_game: bool,
    lines: Vec<Line>,
    broken: Vec<String>,
}

fn press(a: &mut App, btn: Btn) {
    a.apply(Action::GbaDown(btn));
    a.apply(Action::GbaUp(btn));
}

impl Walk {
    fn at(&self, a: &App) -> Option<QuickRow> {
        match self.in_game {
            true => a.in_game_menu(),
            false => a.quick_menu(),
        }
    }

    fn hidden(&self, a: &App) -> Vec<QuickRow> {
        match self.in_game {
            true => a.in_game_hidden(),
            false => a.hidden_rows(),
        }
    }

    fn go(&self, a: &mut App, row: QuickRow) {
        for _ in 0..64 {
            if self.at(a) == Some(row) {
                return;
            }
            press(a, Btn::Down);
        }
        panic!(
            "the bar never reached {row:?}: at {:?}, {:?}",
            self.at(a),
            a.phase()
        );
    }

    /// Back to `row`, from wherever a press left the menu: down its page, or from the top.
    fn back_to(&self, a: &mut App, row: QuickRow) {
        if self.at(a).is_some_and(|now| now.page() == row.page()) {
            return self.go(a, row);
        }
        if self.at(a).is_none() {
            a.apply(Action::QuickMenu);
        }
        while self.at(a).is_some_and(|now| now.page() != row.page()) {
            let was = self.at(a);
            press(a, Btn::B);
            if self.at(a).is_none() {
                a.apply(Action::QuickMenu);
            }
            if self.at(a) == was {
                break;
            }
        }
        if self.at(a).is_some_and(|now| now.page() != row.page()) {
            panic!("lost on the way back to {row:?}: at {:?}", self.at(a));
        }
        self.go(a, row);
    }

    /// The page the bar is on, row by row, opening each page a row opens.
    fn page(&mut self, a: &mut App, device: &str, depth: usize) {
        let first = self.at(a).expect("a menu is open");
        for row in first.shown(&self.hidden(a)) {
            self.go(a, row);
            let text = a.row_text(row);
            let label = a.row_label(row);
            self.check(a, device, row, label, &text);
            self.lines.push(Line {
                depth,
                row,
                label,
                text,
                greyed: a.disabled_rows().contains(&row),
            });
            if row.child().is_some() && !a.disabled_rows().contains(&row) {
                press(a, Btn::A);
                if self.at(a).is_some_and(|now| now.page() != row.page()) {
                    self.page(a, device, depth + 1);
                    press(a, Btn::B);
                    if self.at(a) != Some(row) {
                        self.broken.push(format!(
                            "{device}: back from {row:?}'s page lands on {:?}",
                            self.at(a)
                        ));
                    }
                } else {
                    // It opened something else here (Tools' notice without a network):
                    // B closes it.
                    self.lines.last_mut().unwrap().text += "(opens no page here)";
                    press(a, Btn::B);
                }
                self.back_to(a, row);
            }
        }
    }

    /// The rules every shown row keeps.
    fn check(&mut self, a: &mut App, device: &str, row: QuickRow, label: &str, text: &str) {
        let setting = !row.opens() && !row.centred();
        // A setting shows something at its right.
        if setting && text.is_empty() && !SHOW_NOTHING.contains(&row) {
            self.broken.push(format!(
                "{device}: {row:?} ({label}) is shown with nothing beside it"
            ));
        }
        // What it shows fits beside its name.
        if !text.is_empty() && text != OWN_FACE {
            let bare = text.trim_start_matches(slot::app::CLASH_MARK);
            let squeeze = slot_ui::quick_value_squeeze(a.canvas(), label, bare);
            if squeeze < QUICK_VALUE_MIN_SCALE {
                self.broken.push(format!(
                    "{device}: {row:?}'s \"{bare}\" runs into \"{label}\" (squeezed to {squeeze:.2})"
                ));
            }
        }
        // A value the player can change changes with Left or Right, and goes back.
        if setting && !row.binds() && a.quick_value(row).is_some() && !READ_ONLY.contains(&row) {
            let before = a.row_text(row);
            press(a, Btn::Right);
            let (undo, changed) = match a.row_text(row) {
                after if after != before => (Btn::Left, true),
                _ => {
                    press(a, Btn::Left);
                    (Btn::Right, a.row_text(row) != before)
                }
            };
            if !changed {
                self.broken.push(format!(
                    "{device}: {row:?} shows \"{before}\" but neither Left nor Right changes it"
                ));
            } else {
                press(a, undo);
                if a.row_text(row) != before {
                    self.broken.push(format!(
                        "{device}: {row:?} doesn't go back to \"{before}\" with {undo:?}"
                    ));
                }
            }
            if self.at(a) != Some(row) {
                self.go(a, row);
            }
        }
    }

    fn text(&self) -> String {
        self.lines
            .iter()
            .map(|l| {
                let mut s = format!("{}{}", "  ".repeat(l.depth), l.label);
                if !l.text.is_empty() {
                    s += &format!(": {}", l.text);
                }
                if l.row.child().is_some() {
                    s += " >";
                }
                if l.greyed {
                    s += " (greyed)";
                }
                s + "\n"
            })
            .collect()
    }
}

/// One device's menus: the shelf's, then each system's in a game.
fn walk(device: &str, panel: (u32, u32), parts: Parts) -> (Vec<(String, Walk)>, Vec<String>) {
    let mut sections = Vec::new();
    let mut broken = Vec::new();

    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    slot_store::write_slot_state(
        d.path(),
        &SlotState {
            clock_set: true,
            ..SlotState::default()
        },
    )
    .unwrap();
    let (mut a, _clock) = app_booting_at(d.path(), CLOCK_IS_SET);
    a.set_panel(panel);
    a.set_parts(parts);
    a.apply(Action::QuickMenu);
    let mut shelf = Walk {
        in_game: false,
        lines: Vec::new(),
        broken: Vec::new(),
    };
    shelf.page(&mut a, device, 0);
    broken.append(&mut shelf.broken);
    sections.push(("Shelf".to_string(), shelf));

    for platform in Platform::ALL {
        let (d, mut a) = playing(platform, panel, SlotState::default());
        // The core the session opens the game on (the app alone assumes mGBA).
        a.set_core(slot_store::core_for_platform(
            d.path(),
            "Game",
            platform,
            a.gb_core(),
        ));
        a.set_parts(parts);
        a.apply(Action::QuickMenu);
        let mut game = Walk {
            in_game: true,
            lines: Vec::new(),
            broken: Vec::new(),
        };
        game.page(&mut a, &format!("{device} {platform:?}"), 0);
        broken.append(&mut game.broken);
        sections.push((format!("In game: {}", platform.name()), game));
    }
    (sections, broken)
}

fn map_text(sections: &[(String, Walk)]) -> String {
    sections
        .iter()
        .map(|(name, w)| format!("== {name}\n{}", w.text()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Each row by its place in its section (the names of the pages above it), for comparing
/// the two devices.
fn rows_by_place(sections: &[(String, Walk)]) -> BTreeMap<(String, String), String> {
    let mut rows = BTreeMap::new();
    for (name, w) in sections {
        let mut path: Vec<&str> = Vec::new();
        for l in &w.lines {
            path.truncate(l.depth);
            let place = path
                .iter()
                .chain([&l.label])
                .copied()
                .collect::<Vec<_>>()
                .join(" > ");
            rows.insert((name.clone(), place), l.text.clone());
            path.push(l.label);
        }
    }
    rows
}

fn differences(a: &[(String, Walk)], b: &[(String, Walk)]) -> String {
    let (ra, rb) = (rows_by_place(a), rows_by_place(b));
    let (na, nb) = (DEVICES[0].0, DEVICES[1].0);
    let mut out = format!("# What the {na}'s menus and the {nb}'s differ in (menu_map.rs)\n");
    let mut section = String::new();
    let keys: std::collections::BTreeSet<_> = ra.keys().chain(rb.keys()).cloned().collect();
    for key in keys {
        let line = match (ra.get(&key), rb.get(&key)) {
            (Some(x), Some(y)) if x == y => continue,
            (Some(x), Some(y)) => format!("{}: {na} \"{x}\", {nb} \"{y}\"", key.1),
            (Some(_), None) => format!("{}: {na} only", key.1),
            (None, Some(_)) => format!("{}: {nb} only", key.1),
            (None, None) => unreachable!(),
        };
        if key.0 != section {
            section = key.0.clone();
            out += &format!("\n== {section}\n");
        }
        out += &format!("{line}\n");
    }
    out
}

fn map_file(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/menu-map")
        .join(format!("{name}.txt"))
}

/// Compares `text` with its file, or writes it with SLOT_MENU_MAP=write.
fn matches_file(name: &str, text: &str) -> Result<(), String> {
    let path = map_file(name);
    if std::env::var("SLOT_MENU_MAP").as_deref() == Ok("write") {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, text).unwrap();
        return Ok(());
    }
    let kept = std::fs::read_to_string(&path).unwrap_or_default();
    if kept == text {
        return Ok(());
    }
    let first = kept
        .lines()
        .zip(text.lines())
        .position(|(k, t)| k != t)
        .unwrap_or_else(|| kept.lines().count().min(text.lines().count()));
    let show = |s: &str| {
        s.lines()
            .skip(first)
            .take(4)
            .collect::<Vec<_>>()
            .join("\n    ")
    };
    Err(format!(
        "tests/menu-map/{name}.txt differs from line {}:\n  was:\n    {}\n  now:\n    {}",
        first + 1,
        show(&kept),
        show(text)
    ))
}

#[test]
fn the_menu_maps_of_both_devices_are_as_kept() {
    let walks: Vec<_> = DEVICES
        .iter()
        .map(|(name, panel, parts)| (*name, walk(name, *panel, *parts).0))
        .collect();
    let mut wrong = Vec::new();
    for (name, sections) in &walks {
        if let Err(e) = matches_file(name, &map_text(sections)) {
            wrong.push(e);
        }
    }
    if let Err(e) = matches_file("differences", &differences(&walks[0].1, &walks[1].1)) {
        wrong.push(e);
    }
    assert!(
        wrong.is_empty(),
        "{}\n\nIf the change is meant: SLOT_MENU_MAP=write cargo test -p slot --test tests menu_map",
        wrong.join("\n\n")
    );
}

#[test]
fn rules_hold_on_every_page_of_both_devices() {
    let mut broken = Vec::new();
    for (name, panel, parts) in DEVICES {
        broken.extend(walk(name, panel, parts).1);
    }
    broken.sort();
    broken.dedup();
    assert!(
        broken.is_empty(),
        "{} broken:\n{}",
        broken.len(),
        broken.join("\n")
    );
}
