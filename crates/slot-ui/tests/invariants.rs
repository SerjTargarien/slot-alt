//! Slot²: facts the menu keeps in more than one place, checked against each other (the
//! structure pass, HANDOFF.md). A row's place, its page, its label and the way in and out of
//! a page are each written by hand in several lists; these tests fail when one drifts.

use slot_store::Platform;
use slot_ui::QuickRow;

/// Every list a row's `page()` can name, and its name for the messages.
fn pages() -> Vec<(&'static str, &'static [QuickRow])> {
    vec![
        ("MAIN", &QuickRow::MAIN[..]),
        ("ACHIEVEMENTS", &QuickRow::ACHIEVEMENTS),
        ("GENERAL", &QuickRow::GENERAL),
        ("WIFI", &QuickRow::WIFI),
        ("NETWORK", &QuickRow::NETWORK),
        ("ROMM", &QuickRow::ROMM),
        ("CONTROL_PAGES", &QuickRow::CONTROL_PAGES),
        ("SHELF_KEYS", &QuickRow::SHELF_KEYS),
        ("CONTROL_SYSTEMS", &QuickRow::CONTROL_SYSTEMS),
        ("CONTROLS", &QuickRow::CONTROLS),
        ("SHORTCUT_PAGES", &QuickRow::SHORTCUT_PAGES),
        ("PLAY_SHORTCUTS", &QuickRow::PLAY_SHORTCUTS),
        ("SCREEN", &QuickRow::SCREEN),
        ("SCREEN_GENERAL", &QuickRow::SCREEN_GENERAL),
        ("SCREEN_GBA", &QuickRow::SCREEN_GBA),
        ("SCREEN_GBC", &QuickRow::SCREEN_GBC),
        ("SCREEN_GB", &QuickRow::SCREEN_GB),
        ("SCREEN_WS", &QuickRow::SCREEN_WS),
        ("SCREEN_LYNX", &QuickRow::SCREEN_LYNX),
        ("SCREEN_GG", &QuickRow::SCREEN_GG),
        ("SCREEN_NGP", &QuickRow::SCREEN_NGP),
        ("SCREEN_TG16", &QuickRow::SCREEN_TG16),
        ("GAME", &QuickRow::GAME),
        ("PLAY", &QuickRow::PLAY),
        ("SETTINGS", &QuickRow::SETTINGS),
        ("PLAY_SCREEN", &QuickRow::PLAY_SCREEN),
    ]
}

#[test]
fn every_row_is_on_the_page_it_names() {
    for row in QuickRow::ALL {
        assert!(row.page().contains(&row), "{row:?} isn't on its own page");
    }
}

#[test]
fn every_page_list_is_the_page_of_each_of_its_rows() {
    for (name, page) in pages() {
        for row in page {
            assert!(
                row.page() == page,
                "{row:?} is listed on {name} but its page() is another list"
            );
        }
    }
}

#[test]
fn every_row_is_on_exactly_one_page_list() {
    for row in QuickRow::ALL {
        let on: Vec<&str> = pages()
            .into_iter()
            .filter(|(_, page)| page.contains(&row))
            .map(|(name, _)| name)
            .collect();
        assert_eq!(on.len(), 1, "{row:?} is on {on:?}");
    }
}

#[test]
fn every_row_has_a_label() {
    for row in QuickRow::ALL {
        assert!(!row.label().is_empty(), "{row:?} has no label");
    }
}

#[test]
fn a_rows_child_is_the_first_row_of_a_page() {
    for row in QuickRow::ALL {
        let Some(child) = row.child() else {
            continue;
        };
        // The one exception, by design: every shortcut is on one list, which shows the
        // general ones, a system's own, or (in game) both; each way in starts at the first
        // of its own group.
        let first = match child.page() == QuickRow::PLAY_SHORTCUTS {
            true => [QuickRow::PLAY_SHORTCUTS[0], QuickRow::SHORTCUTS[0]].contains(&child),
            false => child.page()[0] == child,
        };
        assert!(
            first,
            "{row:?} opens {child:?}, which isn't its page's first row"
        );
    }
}

#[test]
fn every_row_can_be_reached_from_the_main_or_the_in_game_menu() {
    let mut reached: Vec<&'static [QuickRow]> = vec![&QuickRow::MAIN, &QuickRow::PLAY];
    let mut at = 0;
    while at < reached.len() {
        for row in reached[at] {
            if let Some(page) = row.child().map(QuickRow::page) {
                if !reached.contains(&page) {
                    reached.push(page);
                }
            }
        }
        at += 1;
    }
    for row in QuickRow::ALL {
        assert!(
            reached.contains(&row.page()),
            "nothing opens {row:?}'s page"
        );
    }
}

#[test]
fn a_row_that_says_it_opens_a_page_has_one() {
    for row in QuickRow::ALL {
        if row.child().is_some() {
            assert!(row.opens(), "{row:?} opens a page but doesn't say so");
        }
    }
}

#[test]
fn the_system_pages_line_up_with_their_systems() {
    assert_eq!(
        QuickRow::SHORTCUT_PAGES.len(),
        QuickRow::SHORTCUT_SYSTEMS.len() + 1
    );
    for (row, platform) in QuickRow::SHORTCUT_PAGES[1..]
        .iter()
        .zip(QuickRow::SHORTCUT_SYSTEMS)
    {
        assert_eq!(row.shortcut_system(), Some(platform), "{row:?}");
        assert_eq!(QuickRow::shortcuts_of(platform), *row, "{platform:?}");
    }
    assert_eq!(
        QuickRow::CONTROL_SYSTEMS.len(),
        slot_store::controls::SYSTEMS.len()
    );
    for (row, platform) in QuickRow::CONTROL_SYSTEMS
        .iter()
        .zip(slot_store::controls::SYSTEMS)
    {
        assert_eq!(row.control_system(), Some(platform), "{row:?}");
        assert_eq!(QuickRow::controls_of(platform), *row, "{platform:?}");
    }
    for platform in Platform::ALL {
        let row = QuickRow::controls_of(platform);
        assert_eq!(
            row.control_system().map(Platform::hardware),
            Some(platform.hardware()),
            "{platform:?}'s controls page is another system's"
        );
    }
}

#[test]
fn back_from_a_page_returns_to_the_row_that_opened_it() {
    for row in QuickRow::ALL {
        let Some(child) = row.child() else {
            continue;
        };
        // The exception, by design: every system's controls page is one page, and so is
        // every system's shortcuts page; the app goes back to the system the page was
        // opened for (`QuickRow::controls_of`, `QuickRow::shortcuts_of`).
        if row.control_system().is_some() || row.shortcut_system().is_some() {
            continue;
        }
        assert!(
            child.parent() == Some(row) || child.parent_in_game() == Some(row),
            "Back from {row:?}'s page goes to {:?} (in game {:?})",
            child.parent(),
            child.parent_in_game()
        );
    }
}

#[test]
fn a_systems_features_are_the_rows_its_screen_page_has() {
    use slot_store::features::{has_feature, Feature};
    let pages = [
        (Platform::Gba, &QuickRow::SCREEN_GBA[..]),
        (Platform::Gbc, &QuickRow::SCREEN_GBC),
        (Platform::Gb, &QuickRow::SCREEN_GB),
        (Platform::Ws, &QuickRow::SCREEN_WS),
        (Platform::Lynx, &QuickRow::SCREEN_LYNX),
        (Platform::Gg, &QuickRow::SCREEN_GG),
        (Platform::Ngp, &QuickRow::SCREEN_NGP),
        (Platform::Tg16, &QuickRow::SCREEN_TG16),
    ];
    for (platform, page) in pages {
        let has_row = |label: &str| page.iter().any(|r| r.label() == label);
        for (feature, label) in [
            (Feature::Palette, "Palette"),
            (Feature::ColourCorrection, "Color Correction"),
            (Feature::Stretch, "Stretch"),
        ] {
            assert_eq!(
                has_feature(platform, feature),
                has_row(label),
                "{platform:?}: the feature {feature:?} and the {label} row disagree"
            );
        }
    }
}
