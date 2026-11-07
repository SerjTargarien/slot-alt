//! Slot²: slot-ui's integration tests as one program, so the crate is linked once
//! rather than once per file. A new test file must be listed here (the test at the
//! end checks); one that changes state the whole process shares (environment variables,
//! the Wi-Fi worker) gets a `[[test]]` of its own in Cargo.toml instead.

mod barcode;
mod barcode_scan;
mod bare_face;
mod battery;
mod board;
mod bundled_bezels;
mod cart;
mod chrome;
mod clock;
mod contrast;
mod extra_carts;
mod guide;
mod house_label;
mod hud;
mod invariants;
mod icon;
mod keyboard;
mod label;
mod legend;
mod letters;
mod link_art;
mod native_menus;
mod native_shelf;
mod network_icons;
mod orientation;
mod pair_screen;
mod palette;
mod plate;
mod polaroids;
mod quick_menu;
mod rebuild;
mod receipt;
mod refusal;
mod render_about;
mod render_board;
mod render_gb_cart;
mod render_keyboard;
mod render_link_art;
mod render_row_edges;
mod render_sticker;
mod shelf;
mod shell;
mod shot_viewer;
mod sticker;
mod sticker_curl;
mod switcher;
mod title_line;
mod toast;
mod wifi_icon;

/// Every file in `tests/` is listed above, or is one of the programs of its own.
#[test]
fn every_test_file_is_in_a_program() {
    let separate = [];
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let listed = include_str!("all.rs");
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if path.extension().is_none_or(|e| e != "rs") || stem == "all" || separate.contains(&stem) {
            continue;
        }
        assert!(
            listed.contains(&format!("mod {stem};")),
            "tests/{stem}.rs is not listed in tests/all.rs"
        );
    }
}
