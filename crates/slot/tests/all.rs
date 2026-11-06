//! Slot²: slot's integration tests as one program, so the crate is linked once
//! rather than once per file. A new test file must be listed here (the test at the
//! end checks); one that changes state the whole process shares (environment variables,
//! the Wi-Fi worker, the ROM header index) gets a `[[test]]` of its own in Cargo.toml
//! instead.

mod common;

mod app;
mod art_index;
mod art_jobs;
mod art_make;
mod art_rom;
mod audio;
mod baseos;
mod bezel;
mod bindings;
mod bios;
mod boot;
mod bootlogo;
mod build_info;
mod cable;
mod cable_session;
mod card_log;
mod card_repair;
mod cart_faces;
mod cart_slot;
mod clock;
mod core_options;
mod core_picker;
mod cpu_clock;
mod crash;
mod delete;
mod dist;
mod drc;
mod eject;
mod emu;
mod evdev;
mod face_builder;
mod fastforward;
mod favorites;
mod fetch_art_ui;
mod flush;
mod frames;
mod gambatte_rumble;
mod gambatte_save;
mod game_overrides;
mod gamepad_input;
mod gb_palettes;
mod gg_core;
mod gpsp;
mod guide;
mod hardware;
mod headphones;
mod hold_picker;
mod in_game_menu;
mod in_game_menu_session;
mod input;
mod label_cache;
mod last_shelf;
mod layout;
mod led;
mod levels;
mod link_art_builder;
mod link_kind;
mod link_screen;
mod link_session;
mod link_start;
mod link_ui;
mod loading_mark;
mod lynx_video;
mod menu_repeat;
mod mgba_gb_link;
mod mgba_link;
mod nds;
mod network_ui;
mod ngp_core;
mod panel_turn;
mod pce_core;
mod platform;
mod play;
mod polaroids;
mod power;
mod prefetch;
mod print_scene;
mod quick_menu;
mod recents;
mod refusal;
mod render;
mod render_clock;
mod render_colour_correction;
mod render_fetch_art;
mod render_gallery_stress;
mod render_gb_palette;
mod render_guide;
mod render_in_game_menu;
mod render_link_screen;
mod render_native;
mod render_network;
mod render_picture_modes;
mod render_power_wake;
mod render_print;
mod render_quick_menu;
mod render_screenshot;
mod render_shelf;
mod render_shelves;
mod render_studio;
mod render_unwrap;
mod resample;
mod resume;
mod rewind;
mod romm_api;
mod romm_conf;
mod romm_gifts;
mod romm_install;
mod romm_jobs;
mod romm_pair;
mod romm_socket;
mod rumble;
mod scale;
mod screenshot;
mod server_carts;
mod server_files;
mod server_games;
mod server_http;
mod server_net;
mod server_states;
mod server_ui;
mod sfx;
mod shot_loader;
mod shot_writer;
mod show_titles;
mod single;
mod state_writer;
mod studio_faces;
mod studio_ui;
mod studio_view;
mod sunlight;
mod swap;
mod tg16_video;
mod thumb;
mod toast;
mod undo;
mod unwrap_scene;
mod update;
mod update_ui;
mod volume;
mod wallpaper;
mod wifi_sp;
mod wifi_ui;
mod wifi_wpa;
mod ws_video;

/// Every file in `tests/` is listed above, or is one of the programs of its own.
#[test]
fn every_test_file_is_in_a_program() {
    let separate = [
        "core",
        "studio_state",
        "e2e",
        "gb_palette_recolour",
        "romm_ui",
        "wifi_worker",
        "header_index_boot",
    ];
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
