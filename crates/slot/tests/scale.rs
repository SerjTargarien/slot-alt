mod common;

use common::{tmp_root_with_carts, tmp_root_with_gb_carts, tmp_root_with_gbc_carts, StubSnapshot};
use slot::app::{App, Phase};
use slot::video_mode::VideoMode;
use slot_gfx::{GameLayout, TexId, WHOLE_TEXTURE};
use slot_input::{Action, Btn};
use slot_store::{write_slot_state, BezelPick, Platform, Scale, SlotState, Stretch};
use slot_ui::{BezelArt, QuickRow, QuickValue};
use tempfile::TempDir;

const GB_WINDOW: [f32; 4] = [40.0 / 240.0, 0.0, 160.0 / 240.0, 1.0];
const GB_CROP: [f32; 4] = [40.0 / 240.0, 8.0 / 160.0, 160.0 / 240.0, 144.0 / 160.0];

fn root_for(platform: Platform) -> TempDir {
    match platform {
        Platform::Gba => tmp_root_with_carts(&["Game", "Zzz"]),
        Platform::Gb => tmp_root_with_gb_carts(&["Game", "Zzz"]),
        Platform::Gbc => tmp_root_with_gbc_carts(&["Game", "Zzz"]),
        Platform::Gg | Platform::Ngp | Platform::Tg16 => {
            let d = tmp_root_with_gb_carts(&[]);
            let (dir, ext) = match platform {
                Platform::Gg => ("GG", "gg"),
                Platform::Tg16 => ("TG16", "pce"),
                _ => ("NGP", "ngc"),
            };
            for stem in ["Game", "Zzz"] {
                let rom = d
                    .path()
                    .join("Games")
                    .join(dir)
                    .join(format!("{stem}.{ext}"));
                std::fs::write(rom, vec![0u8; 0x10000]).unwrap();
            }
            d
        }
        Platform::Gbav => {
            let d = tmp_root_with_carts(&["Zzz"]);
            let dir = d.path().join("Games").join(Platform::Gbav.dir_name());
            std::fs::create_dir_all(&dir).unwrap();
            let mut rom = vec![0u8; 0x100];
            rom[0xa0..0xa4].copy_from_slice(b"GAME");
            std::fs::write(dir.join("Game.gba"), rom).unwrap();
            d
        }
        other => panic!("Scale is the Game Boy family's, not {other:?}'s"),
    }
}

fn playing(platform: Platform, panel: (u32, u32), state: SlotState) -> (TempDir, App) {
    let d = root_for(platform);
    write_slot_state(
        d.path(),
        &SlotState {
            cart: Some("Game".into()),
            cart_platform: Some(platform),
            clock_set: true,
            ..state
        },
    )
    .unwrap();
    let mut a = App::boot(d.path());
    a.set_panel(panel);
    a.set_snapshot(StubSnapshot::boxed());
    a.on_core_ready();
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    assert!(
        matches!(a.phase(), Phase::Playing { .. }),
        "not playing: {:?}",
        a.phase()
    );
    assert_eq!(a.playing_platform(), platform);
    (d, a)
}

#[test]
fn defaults_place_every_platform_as_before() {
    for panel in [(720, 480), (640, 480)] {
        for platform in [Platform::Gba, Platform::Gb, Platform::Gbc] {
            let (_d, mut a) = playing(platform, panel, SlotState::default());
            for mode in [VideoMode::Actual, VideoMode::Stretch] {
                a.set_video_mode(mode);
                let want = match (platform, mode) {
                    (Platform::Gba, _) | (_, VideoMode::Actual) => WHOLE_TEXTURE,
                    _ => GB_WINDOW,
                };
                assert_eq!(
                    a.game_layout(),
                    GameLayout::BAND,
                    "{platform:?} {mode:?} {panel:?}"
                );
                assert_eq!(a.source_rect(), want, "{platform:?} {mode:?} {panel:?}");
            }
        }
    }
}

#[test]
fn gb_integer_on_4_by_3_takes_the_panel_and_drops_the_strip() {
    let (_d, mut a) = playing(
        Platform::Gb,
        (640, 480),
        SlotState {
            bezel_gb: BezelPick::Ags102,
            scale_gb: Scale::Integer,
            ..SlotState::default()
        },
    );
    assert!(a.game_layout().panel);
    assert_eq!(a.source_rect(), GB_CROP);
    assert!(!a.sp_frame());
    a.apply(Action::Eject);
    for _ in 0..600 {
        assert!(!a.sp_frame(), "the strip came back during {:?}", a.phase());
        a.update(1.0 / 60.0);
        if matches!(a.phase(), Phase::Shelf) {
            break;
        }
    }
    assert!(matches!(a.phase(), Phase::Shelf), "{:?}", a.phase());
}

#[test]
fn stretch_full_screen_crops_the_border_and_hides_bezels() {
    for panel in [(720, 480), (640, 480)] {
        let (_d, mut a) = playing(
            Platform::Gb,
            panel,
            SlotState {
                bezel_gb: BezelPick::Default,
                stretch_gb: Stretch::FullScreen,
                ..SlotState::default()
            },
        );
        a.set_game_bezel(Some(TexId::from_raw(77)));
        a.set_video_mode(VideoMode::Stretch);
        assert_eq!(a.source_rect(), GB_CROP, "{panel:?}");
        assert_eq!(a.game_layout().panel, panel == (640, 480));
        assert_eq!(a.shown_game_bezel(), None, "{panel:?}");
    }
}

#[test]
fn stretch_fills_a_square_panel_whatever_the_stretch_style() {
    for stretch in [Stretch::Classic, Stretch::FullScreen] {
        let (_d, a) = playing(
            Platform::Gb,
            (720, 720),
            SlotState {
                stretch_gb: stretch,
                stretched_gb: true,
                ..SlotState::default()
            },
        );
        assert_eq!(a.source_rect(), GB_CROP, "{stretch:?}");
        assert!(a.game_layout().panel, "{stretch:?}");
        assert_eq!(a.stretch_choices(Platform::Gb), [Stretch::Classic]);
    }
}

fn press(a: &mut App, btn: Btn) {
    a.apply(Action::GbaDown(btn));
    a.apply(Action::GbaUp(btn));
}

fn to_in_game_row(a: &mut App, row: QuickRow) {
    a.apply(Action::QuickMenu);
    for step in [QuickRow::Settings, QuickRow::PlayScreen, row] {
        for _ in 0..16 {
            if a.in_game_menu() == Some(step) {
                break;
            }
            press(a, Btn::Down);
        }
        assert_eq!(a.in_game_menu(), Some(step), "never reached {step:?}");
        if step != row {
            press(a, Btn::A);
        }
    }
}

#[test]
fn on_a_square_panel_the_stretch_row_switches_stretch_like_l1_and_r1() {
    let (_d, mut a) = playing(Platform::Gb, (720, 720), SlotState::default());
    assert!(!a.in_game_hidden().contains(&QuickRow::Stretch));
    assert_eq!(a.quick_value(QuickRow::Stretch), Some(QuickValue::Off));
    to_in_game_row(&mut a, QuickRow::Stretch);
    press(&mut a, Btn::Right);
    assert_eq!(a.quick_value(QuickRow::Stretch), Some(QuickValue::On));
    assert!(a.game_layout().panel);
    assert_eq!(a.source_rect(), GB_CROP);
    press(&mut a, Btn::Left);
    assert_eq!(a.quick_value(QuickRow::Stretch), Some(QuickValue::Off));
    assert!(!a.picture_fills());
}

#[test]
fn on_a_square_panel_stretch_is_one_value_per_platform_shared_with_the_shelf() {
    let (d, mut a) = playing(Platform::Gb, (720, 720), SlotState::default());
    a.set_video_mode(VideoMode::Stretch);
    assert!(
        !a.picture_fills(),
        "a game's own video_mode.txt entry is ignored here"
    );
    to_in_game_row(&mut a, QuickRow::Stretch);
    press(&mut a, Btn::Right);
    assert_eq!(a.quick_value(QuickRow::GbStretch), Some(QuickValue::On));
    assert_eq!(a.quick_value(QuickRow::GbcStretch), Some(QuickValue::Off));
    assert!(a.picture_fills());
    let saved = slot_store::read_slot_state(d.path());
    assert!(saved.stretched_gb && !saved.stretched_gbc);
    a.apply(Action::QuickMenu);
    assert_eq!(a.in_game_menu(), None);
    a.apply(Action::StretchOff);
    assert_eq!(a.quick_value(QuickRow::Stretch), Some(QuickValue::Off));
    assert_eq!(a.quick_value(QuickRow::GbStretch), Some(QuickValue::Off));
}

#[test]
fn the_shelf_shows_the_stretch_switch_on_a_square_panel_and_the_style_elsewhere() {
    let (_d, a) = playing(Platform::Gb, (720, 720), SlotState::default());
    let hidden = a.hidden_rows();
    assert!(!hidden.contains(&QuickRow::GbStretch));
    assert!(!hidden.contains(&QuickRow::GbcStretch));
    assert_eq!(a.quick_value(QuickRow::GbStretch), Some(QuickValue::Off));
    let (_d, a) = playing(Platform::Gb, (720, 480), SlotState::default());
    assert_eq!(
        a.quick_value(QuickRow::GbStretch),
        Some(QuickValue::Classic)
    );
}

#[test]
fn the_stretch_shortcuts_switch_a_game_boy_game_and_leave_gba_alone() {
    let (_d, mut a) = playing(Platform::Gb, (720, 480), SlotState::default());
    a.apply(Action::StretchOn);
    assert!(a.picture_fills());
    a.apply(Action::StretchOff);
    assert!(!a.picture_fills());
    let (_d, a) = playing(Platform::Gba, (720, 480), SlotState::default());
    assert!(a
        .input_entries()
        .iter()
        .all(|e| !matches!(e.action, Action::StretchOn | Action::StretchOff)));
}

#[test]
fn a_gba_game_on_a_square_panel_has_no_stretch_row() {
    let (_d, a) = playing(Platform::Gba, (720, 720), SlotState::default());
    assert!(a.in_game_hidden().contains(&QuickRow::Stretch));
}

#[test]
fn a_3_by_2_panel_keeps_both_stretch_styles() {
    let (_d, a) = playing(Platform::Gb, (720, 480), SlotState::default());
    assert_eq!(
        a.stretch_choices(Platform::Gb),
        [Stretch::Classic, Stretch::FullScreen]
    );
}

#[test]
fn a_gba_cart_ignores_a_stale_stretch() {
    let (_d, mut a) = playing(Platform::Gba, (640, 480), SlotState::default());
    a.set_game_bezel(Some(TexId::from_raw(77)));
    a.set_video_mode(VideoMode::Stretch);
    assert_eq!(a.source_rect(), WHOLE_TEXTURE);
    assert_eq!(a.game_layout(), GameLayout::BAND);
    assert_eq!(a.shown_game_bezel(), Some(TexId::from_raw(77)));
}

#[test]
fn gba_video_follows_the_gba_scale() {
    let (_d, a) = playing(
        Platform::Gbav,
        (640, 480),
        SlotState {
            scale_gba: Scale::FullScreen,
            ..SlotState::default()
        },
    );
    assert!(a.game_layout().panel);
    let (_d, a) = playing(
        Platform::Gbav,
        (640, 480),
        SlotState {
            scale_gb: Scale::Integer,
            ..SlotState::default()
        },
    );
    assert_eq!(a.game_layout(), GameLayout::BAND);
}

#[test]
fn in_game_rows_follow_the_playing_platform() {
    let (_d, a) = playing(Platform::Gba, (640, 480), SlotState::default());
    let hidden = a.in_game_hidden();
    assert!(!hidden.contains(&QuickRow::Scale));
    assert!(hidden.contains(&QuickRow::Stretch));
    assert_eq!(a.quick_value(QuickRow::Scale), Some(QuickValue::Fit));
    let (_d, a) = playing(Platform::Gb, (720, 480), SlotState::default());
    let hidden = a.in_game_hidden();
    assert!(!hidden.contains(&QuickRow::Scale));
    assert!(!hidden.contains(&QuickRow::Stretch));
    assert_eq!(a.quick_value(QuickRow::Stretch), Some(QuickValue::Classic));
}

#[test]
fn bezel_art_follows_the_layout() {
    for (panel, scale, want) in [
        ((720, 480), Scale::SpFrame, Some(BezelArt::Default)),
        ((640, 480), Scale::SpFrame, Some(BezelArt::Default)),
        ((720, 480), Scale::Fit, Some(BezelArt::Fit3x2)),
        ((640, 480), Scale::Fit, Some(BezelArt::Fit4x3)),
        ((640, 480), Scale::Integer, Some(BezelArt::Integer4x3)),
        ((720, 480), Scale::Integer, Some(BezelArt::Default)),
    ] {
        let (_d, a) = playing(
            Platform::Gb,
            panel,
            SlotState {
                bezel_gb: BezelPick::Default,
                scale_gb: scale,
                ..SlotState::default()
            },
        );
        assert_eq!(a.bezel_art(), want, "{panel:?} {scale:?}");
    }
    let (_d, mut a) = playing(
        Platform::Gb,
        (720, 480),
        SlotState {
            bezel_gb: BezelPick::Default,
            ..SlotState::default()
        },
    );
    a.set_video_mode(VideoMode::Stretch);
    assert_eq!(a.bezel_art(), None);
}

#[test]
fn a_saved_integer_on_3_by_2_draws_exactly_as_default() {
    let (_d, a) = playing(
        Platform::Gb,
        (720, 480),
        SlotState {
            scale_gb: Scale::Integer,
            ..SlotState::default()
        },
    );
    assert_eq!(a.game_layout(), GameLayout::BAND);
    assert_eq!(a.source_rect(), WHOLE_TEXTURE);
}

fn bezels_in(
    platform: Platform,
    panel: (u32, u32),
    scale: Scale,
    picked: &str,
    files: &[&str],
) -> (Vec<String>, BezelPick) {
    let pick = BezelPick::File(picked.into());
    let state = match platform {
        Platform::Gba => SlotState {
            bezel_gba: pick,
            scale_gba: scale,
            ..SlotState::default()
        },
        _ => SlotState {
            bezel_gb: pick,
            scale_gb: scale,
            ..SlotState::default()
        },
    };
    let d = root_for(platform);
    let dir = d.path().join("Bezels").join(platform.dir_name());
    std::fs::create_dir_all(&dir).unwrap();
    for f in files {
        std::fs::write(dir.join(f), b"").unwrap();
    }
    write_slot_state(
        d.path(),
        &SlotState {
            cart: Some("Game".into()),
            cart_platform: Some(platform),
            clock_set: true,
            ..state
        },
    )
    .unwrap();
    let mut a = App::boot(d.path());
    a.set_panel(panel);
    let mut shown: Vec<String> = a
        .bezel_files(platform)
        .into_iter()
        .filter(|name| {
            slot_store::bezel_scale(name).is_none_or(|s| s == a.resolved_scale(platform))
        })
        .collect();
    shown.sort();
    (shown, a.resolved_bezel(platform))
}

const ALL: [&str; 4] = [
    "Shell_fit.png",
    "Shell_integer.png",
    "Shell_spscreen.png",
    "Wood.png",
];

#[test]
fn the_bezel_list_holds_plain_bezels_and_those_for_the_scale() {
    for (platform, panel, scale, want) in [
        (
            Platform::Gb,
            (640, 480),
            Scale::SpFrame,
            ["Shell_spscreen.png", "Wood.png"],
        ),
        (
            Platform::Gb,
            (640, 480),
            Scale::Integer,
            ["Shell_integer.png", "Wood.png"],
        ),
        (
            Platform::Gb,
            (640, 480),
            Scale::Fit,
            ["Shell_fit.png", "Wood.png"],
        ),
        (
            Platform::Gb,
            (720, 480),
            Scale::Integer,
            ["Shell_integer.png", "Wood.png"],
        ),
        (
            Platform::Gba,
            (640, 480),
            Scale::Fit,
            ["Shell_fit.png", "Wood.png"],
        ),
    ] {
        let (shown, _) = bezels_in(platform, panel, scale, "Wood.png", &ALL);
        assert_eq!(shown, want, "{platform:?} {panel:?} {scale:?}");
    }
}

#[test]
fn a_plain_bezel_stays_picked_in_every_scale() {
    for scale in [Scale::SpFrame, Scale::Integer, Scale::Fit] {
        let (_, pick) = bezels_in(Platform::Gb, (640, 480), scale, "Wood.png", &ALL);
        assert_eq!(pick, BezelPick::File("Wood.png".into()), "{scale:?}");
    }
}

#[test]
fn a_suffixed_bezel_follows_the_scale_to_its_matching_file() {
    let (_, pick) = bezels_in(
        Platform::Gb,
        (640, 480),
        Scale::Integer,
        "Shell_fit.png",
        &ALL,
    );
    assert_eq!(pick, BezelPick::File("Shell_integer.png".into()));
    let only_fit = ["Shell_fit.png", "Wood.png"];
    let (_, pick) = bezels_in(
        Platform::Gb,
        (640, 480),
        Scale::Integer,
        "Shell_fit.png",
        &only_fit,
    );
    assert_eq!(pick, BezelPick::None);
}

/// The picture's size in pixels on a 720x720 panel. A layout that takes the whole panel
/// keeps its rectangle in band units (720x480), so its height scales by 720/480; one that
/// fits the band is drawn in the band at 1:1.
fn drawn(a: &App) -> (f32, f32) {
    let layout = a.game_layout();
    match layout.panel {
        true => (layout.rect[2], (layout.rect[3] * 720.0 / 480.0).round()),
        false => (layout.rect[2], layout.rect[3].round()),
    }
}

#[test]
fn the_game_gear_and_pocket_scale_to_integer_or_fit_on_a_square_panel() {
    for (platform, integer, fit, lines) in [
        (Platform::Gg, (640.0, 576.0), (720.0, 648.0), 144.0),
        (Platform::Ngp, (640.0, 608.0), (720.0, 684.0), 152.0),
    ] {
        let (_d, mut a) = playing(platform, (720, 720), SlotState::default());
        assert_eq!(a.scale_choices(platform), [Scale::Integer, Scale::Fit]);
        assert_eq!(drawn(&a), integer, "{platform:?} starts at Integer");
        // The picture itself is what's drawn into that box, not the whole 240x160 buffer.
        let top = (160.0 - lines) / 2.0 / 160.0;
        assert_eq!(
            a.source_rect(),
            [40.0 / 240.0, top, 160.0 / 240.0, lines / 160.0],
            "{platform:?}"
        );
        let row = match platform {
            Platform::Gg => QuickRow::GgScale,
            _ => QuickRow::NgpScale,
        };
        assert_eq!(a.quick_value(row), Some(QuickValue::Integer));
        to_in_game_row(&mut a, QuickRow::Scale);
        press(&mut a, Btn::Right);
        assert_eq!(a.quick_value(row), Some(QuickValue::Fit));
        assert_eq!(drawn(&a), fit, "{platform:?} at Fit");
    }
}

#[test]
fn the_game_gear_and_pocket_stretch_to_the_whole_square_panel() {
    for platform in [Platform::Gg, Platform::Ngp] {
        let (d, mut a) = playing(platform, (720, 720), SlotState::default());
        assert!(!a.picture_fills());
        to_in_game_row(&mut a, QuickRow::Stretch);
        press(&mut a, Btn::Right);
        assert!(a.picture_fills(), "{platform:?}");
        assert!(a.game_layout().panel);
        assert_ne!(a.source_rect(), slot_gfx::WHOLE_TEXTURE, "{platform:?}");
        let saved = slot_store::read_slot_state(d.path());
        assert_eq!(
            (saved.stretched_gg, saved.stretched_ngp),
            (platform == Platform::Gg, platform == Platform::Ngp)
        );
        a.apply(Action::QuickMenu);
        a.apply(Action::StretchOff);
        assert!(!a.picture_fills(), "{platform:?}: Stretch Off");
    }
}

#[test]
fn the_game_gear_and_pocket_draw_no_bezel_and_hide_its_rows() {
    for platform in [Platform::Gg, Platform::Ngp] {
        let (_d, mut a) = playing(platform, (720, 720), SlotState::default());
        a.set_game_bezel(Some(TexId::from_raw(77)));
        assert_eq!(a.shown_game_bezel(), None, "{platform:?}");
        assert_eq!(a.bezel_art(), None);
        assert!(a.in_game_hidden().contains(&QuickRow::Bezel));
        assert!(a.hidden_rows().contains(&QuickRow::GgBezel));
        assert!(!a.in_game_hidden().contains(&QuickRow::Scale));
        assert!(!a.in_game_hidden().contains(&QuickRow::Stretch));
    }
}

#[test]
fn a_turbografx_picture_scales_by_the_width_its_game_draws() {
    // (frame the core drew, Integer, Fit): a 512-wide frame has half-width pixels, so
    // it takes the room of a 256-wide one.
    for (frame, integer, fit) in [
        ((256, 240), (512.0, 480.0), (720.0, 675.0)),
        ((320, 240), (640.0, 480.0), (720.0, 540.0)),
        ((336, 240), (672.0, 480.0), (720.0, 514.0)),
        ((512, 240), (512.0, 480.0), (720.0, 675.0)),
    ] {
        let (_d, mut a) = playing(Platform::Tg16, (720, 720), SlotState::default());
        a.set_frame_shape(frame);
        assert_eq!(
            a.scale_choices(Platform::Tg16),
            [Scale::Integer, Scale::Fit]
        );
        assert_eq!(drawn(&a), integer, "{frame:?} at Integer");
        let x = (720 - frame.0) as f32 / 2.0 / 720.0;
        assert_eq!(
            a.source_rect(),
            [x, 0.0, frame.0 as f32 / 720.0, 1.0],
            "{frame:?}"
        );
        to_in_game_row(&mut a, QuickRow::Scale);
        press(&mut a, Btn::Right);
        assert_eq!(a.quick_value(QuickRow::Tg16Scale), Some(QuickValue::Fit));
        let (w, h) = drawn(&a);
        assert_eq!((w, h.round()), fit, "{frame:?} at Fit");
    }
}

#[test]
fn a_turbografx_picture_stretches_to_the_whole_square_panel() {
    let (d, mut a) = playing(Platform::Tg16, (720, 720), SlotState::default());
    a.set_frame_shape((256, 240));
    to_in_game_row(&mut a, QuickRow::Stretch);
    press(&mut a, Btn::Right);
    assert!(a.picture_fills());
    assert!(a.game_layout().panel);
    assert!(slot_store::read_slot_state(d.path()).stretched_tg16);
    a.apply(Action::QuickMenu);
    a.apply(Action::StretchOff);
    assert!(!a.picture_fills());
    a.set_game_bezel(Some(TexId::from_raw(77)));
    assert_eq!(a.shown_game_bezel(), None);
    assert!(a.hidden_rows().contains(&QuickRow::Tg16Bezel));
}
