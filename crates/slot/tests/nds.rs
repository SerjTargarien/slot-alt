//! A DS game's views of its two screens, and its stylus.

use crate::common;

use slot::app::App;
use slot::video_mode::{DsLayout, DsScreen, DsView};
use slot_input::Action;
use slot_store::{Platform, Shader};

fn playing_ds() -> (tempfile::TempDir, App) {
    playing_ds_with(Shader::Off)
}

fn playing_ds_with(shader: Shader) -> (tempfile::TempDir, App) {
    playing_ds_in(slot_store::SlotState {
        shader_nds: shader,
        ..Default::default()
    })
}

fn playing_ds_in(state: slot_store::SlotState) -> (tempfile::TempDir, App) {
    let d = common::tmp_root_with_carts(&["Emerald"]);
    let nds = d.path().join("Games/NDS");
    std::fs::create_dir_all(&nds).unwrap();
    std::fs::write(nds.join("Kart.nds"), vec![0u8; 0x10000]).unwrap();
    slot_store::write_slot_state(
        d.path(),
        &slot_store::SlotState {
            cart: Some("Kart".into()),
            cart_platform: Some(Platform::Nds),
            clock_set: true,
            ..state
        },
    )
    .unwrap();
    let mut a = App::boot(d.path());
    a.set_snapshot(common::StubSnapshot::boxed());
    a.set_core(slot_store::Core::Dsperate);
    a.on_core_ready();
    a.set_game_ready(true);
    for _ in 0..120 {
        a.update(1.0 / 60.0);
    }
    assert_eq!(a.playing_platform(), Platform::Nds);
    assert!(
        a.ds_controls(),
        "{:?} visible {}",
        a.phase(),
        a.game_visible()
    );
    (d, a)
}

#[test]
fn a_ds_game_starts_on_both_screens_and_l2_swaps_which_is_on_top() {
    let (d, mut a) = playing_ds();
    assert_eq!(a.ds_view().layout, DsLayout::Stacked);
    let (main, other) = a.ds_parts();
    assert_eq!(main.screen, DsScreen::Top);
    assert_eq!(main.dst, [200.0, 0.0, 320.0, 240.0], "1.25x, on top");
    assert_eq!(other.unwrap().dst, [200.0, 240.0, 320.0, 240.0]);
    assert_eq!(a.source_rect(), DsScreen::Top.source_rect());
    assert_eq!(a.game_layout().rect, main.dst);
    a.swap_ds_screens();
    let (main, other) = a.ds_parts();
    assert_eq!(main.screen, DsScreen::Bottom);
    assert_eq!(main.dst[1], 0.0, "the bottom screen now on top");
    assert_eq!(other.unwrap().screen, DsScreen::Top);
    assert_eq!(
        slot::video_mode::ds_view_for(d.path(), "Kart", DsLayout::Stacked).main,
        DsScreen::Bottom,
        "the game remembers its main screen"
    );
}

#[test]
fn select_l2_steps_through_the_layouts_at_fit_and_shaders_show_from_2x() {
    let (d, mut a) = playing_ds_with(Shader::Lcd3x);
    assert!(a.game_sharp(), "a DS game is drawn sharp bilinear");
    let mut seen = Vec::new();
    for _ in 0..DsLayout::ALL.len() {
        let (main, other) = a.ds_parts();
        seen.push((
            a.ds_view().layout,
            main.dst,
            other.map(|o| o.dst),
            a.drawn_shader(),
        ));
        a.next_ds_layout();
    }
    assert_eq!(a.ds_view().layout, DsLayout::Stacked, "and round again");
    let plain = Shader::Off;
    let lcd = Shader::Lcd3x;
    assert_eq!(
        seen,
        vec![
            (
                DsLayout::Stacked,
                [200.0, 0.0, 320.0, 240.0],
                Some([200.0, 240.0, 320.0, 240.0]),
                plain
            ),
            (DsLayout::Single, [40.0, 0.0, 640.0, 480.0], None, lcd),
            (
                DsLayout::Below,
                [120.0, 0.0, 480.0, 360.0],
                Some([280.0, 360.0, 160.0, 120.0]),
                plain
            ),
            (
                DsLayout::Corner,
                [40.0, 0.0, 640.0, 480.0],
                Some([424.0, 288.0, 256.0, 192.0]),
                lcd
            ),
        ]
    );
    a.next_ds_layout();
    assert_eq!(
        slot::video_mode::ds_view_for(d.path(), "Kart", DsLayout::Stacked).layout,
        DsLayout::Single,
        "the game remembers its layout"
    );
}

#[test]
fn the_views_saved_before_layouts_still_read() {
    let view = |layout, main| {
        Some(DsView {
            main,
            layout,
            stylus_screen: None,
        })
    };
    assert_eq!(
        DsView::parse("both"),
        view(DsLayout::Stacked, DsScreen::Top)
    );
    assert_eq!(DsView::parse("top"), view(DsLayout::Single, DsScreen::Top));
    assert_eq!(
        DsView::parse("bottom"),
        view(DsLayout::Single, DsScreen::Bottom)
    );
    for layout in DsLayout::ALL {
        for main in [DsScreen::Top, DsScreen::Bottom] {
            for stylus_screen in [None, Some(true), Some(false)] {
                let v = DsView {
                    main,
                    layout,
                    stylus_screen,
                };
                assert_eq!(DsView::parse(&v.as_str()), Some(v));
            }
        }
    }
    assert_eq!(DsView::parse("sideways,top"), None);
}

#[test]
fn the_stylus_moves_and_touches_and_leaves_the_screens_where_they_are() {
    let (_d, mut a) = playing_ds();
    a.next_ds_layout();
    assert_eq!(a.ds_view().layout, DsLayout::Single);
    assert_eq!(a.source_rect(), DsScreen::Top.source_rect());
    assert_eq!(a.pointer(), None, "no stylus until R2");
    a.stylus_out(true);
    assert_eq!(
        a.source_rect(),
        DsScreen::Top.source_rect(),
        "the stylus never swaps the screens: L2 does"
    );
    let at = a.pointer().unwrap();
    assert!(!at.pressed);
    a.stylus_steer([false, false, false, true], false);
    for _ in 0..30 {
        a.update(1.0 / 60.0);
    }
    a.stylus_steer([false; 4], true);
    let moved = a.pointer().unwrap();
    assert!(moved.x > at.x, "moved right: {} to {}", at.x, moved.x);
    assert_eq!(moved.y, at.y);
    assert!(moved.pressed, "it touches");
    a.stylus_out(false);
    assert_eq!(a.pointer(), None);
    a.stylus_out(true);
    let back = a.pointer().unwrap();
    assert_eq!(
        (back.x, back.y),
        (moved.x, moved.y),
        "it comes back where it was left"
    );
    assert!(!back.pressed);
}

#[test]
fn opening_the_menu_puts_the_stylus_away() {
    let (_d, mut a) = playing_ds();
    a.stylus_out(true);
    a.apply(Action::QuickMenu);
    for _ in 0..10 {
        a.update(1.0 / 60.0);
    }
    assert!(!a.ds_controls());
    assert_eq!(a.pointer(), None);
}

#[test]
fn a_ds_game_records_no_rewind_and_keeps_x_and_y_as_buttons() {
    let (_d, a) = playing_ds();
    assert!(!a.records_rewind());
    let hidden = a.in_game_hidden();
    assert!(
        !hidden.contains(&slot_ui::QuickRow::GameX) && !hidden.contains(&slot_ui::QuickRow::GameY),
        "X and Y are the DS's own"
    );
}

#[test]
fn a_ds_game_offers_no_rewind() {
    let (_d, a) = playing_ds();
    let hidden = a.in_game_hidden();
    assert!(hidden.contains(&slot_ui::QuickRow::Rewind));
    assert!(hidden.contains(&slot_ui::QuickRow::ScRewind));
}

#[test]
fn a_ds_bezel_set_follows_the_layout() {
    let (_d, mut a) = playing_ds();
    let mut seen = Vec::new();
    for _ in DsLayout::ALL {
        seen.push(a.ds_bezel_file());
        a.next_ds_layout();
    }
    // At Fit, as on the band before: One Screen at Fit is upstream's Full Screen, and
    // Main and Small has no art of its own.
    assert_eq!(
        seen,
        [
            Some("both-screens.png"),
            Some("full-screen.png"),
            None,
            Some("picture-in-picture.png")
        ]
    );
    a.swap_ds_screens();
    assert_eq!(
        a.ds_bezel_file(),
        Some("both-screens.png"),
        "swapping the screens keeps the layout's bezel"
    );
}

/// A DS game on the RGB20SX's square panel, its screens on the whole 720x720 canvas.
fn playing_ds_square() -> (tempfile::TempDir, App) {
    let (d, mut a) = playing_ds();
    a.set_panel((720, 720));
    (d, a)
}

#[test]
fn on_the_square_panel_the_layouts_use_all_of_it() {
    let (_d, mut a) = playing_ds_square();
    let (main, other) = a.ds_parts();
    assert_eq!(
        main.dst,
        [120.0, 0.0, 480.0, 360.0],
        "both screens at 1.875x"
    );
    assert_eq!(other.unwrap().dst, [120.0, 360.0, 480.0, 360.0]);
    let layout = a.game_layout();
    assert!(layout.panel, "placed over the whole panel");
    assert_eq!(
        layout.rect,
        [120.0, 0.0, 480.0, 240.0],
        "in 720x480 units of the panel"
    );
    a.next_ds_layout();
    assert_eq!(
        a.ds_parts().0.dst,
        [0.0, 90.0, 720.0, 540.0],
        "one screen at Fit"
    );
    a.next_ds_layout();
    let (main, other) = a.ds_parts();
    assert_eq!(main.dst, [0.0, 0.0, 720.0, 540.0], "main and small");
    assert_eq!(other.unwrap().dst, [240.0, 540.0, 240.0, 180.0]);
    a.next_ds_layout();
    let (main, other) = a.ds_parts();
    assert_eq!(main.dst, [0.0, 90.0, 720.0, 540.0]);
    assert_eq!(
        other.unwrap().dst,
        [464.0, 438.0, 256.0, 192.0],
        "1x in the corner"
    );
}

#[test]
fn integer_scale_draws_whole_multiples_and_stretch_fills_with_one_screen() {
    use slot_store::{Scale, SlotState};
    let (_d, mut a) = playing_ds_in(SlotState {
        scale_nds: Scale::Integer,
        ..Default::default()
    });
    a.set_panel((720, 720));
    assert_eq!(
        a.ds_parts().0.dst,
        [232.0, 168.0, 256.0, 192.0],
        "both at 1x"
    );
    a.next_ds_layout();
    assert_eq!(
        a.ds_parts().0.dst,
        [104.0, 168.0, 512.0, 384.0],
        "one at 2x"
    );
    let (_d, mut a) = playing_ds_in(SlotState {
        stretched_nds: true,
        nds_layout: "single".into(),
        ..Default::default()
    });
    a.set_panel((720, 720));
    assert_eq!(a.ds_starting_layout(), DsLayout::Single);
    let d = tempfile::tempdir().unwrap();
    assert_eq!(
        slot::video_mode::ds_view_for(d.path(), "New", a.ds_starting_layout()).layout,
        DsLayout::Single,
        "a game never played starts in the shelf's layout"
    );
    a.set_ds_view(DsView {
        layout: DsLayout::Single,
        ..DsView::default()
    });
    assert_eq!(a.ds_parts().0.dst, [0.0, 0.0, 720.0, 720.0], "stretched");
    a.next_ds_layout();
    assert_eq!(
        a.ds_parts().0.dst,
        [0.0, 0.0, 720.0, 540.0],
        "Stretch is One Screen's only"
    );
}

#[test]
fn the_stylus_shows_the_touch_screen_when_the_shelf_or_the_game_says_so() {
    use slot_store::SlotState;
    use slot_ui::{QuickRow, QuickValue};
    let one_screen = |a: &mut App| {
        a.set_ds_view(DsView {
            layout: DsLayout::Single,
            ..a.ds_view()
        })
    };
    let (_d, mut a) = playing_ds();
    one_screen(&mut a);
    a.stylus_out(true);
    assert_eq!(
        a.source_rect(),
        DsScreen::Top.source_rect(),
        "off by default"
    );
    a.stylus_out(false);
    let (d, mut a) = playing_ds_in(SlotState {
        nds_stylus_screen: true,
        ..Default::default()
    });
    one_screen(&mut a);
    assert_eq!(a.quick_value(QuickRow::StylusScreen), Some(QuickValue::On));
    a.stylus_out(true);
    assert_eq!(
        a.source_rect(),
        DsScreen::Bottom.source_rect(),
        "the shelf's On brings the touch screen up"
    );
    a.stylus_out(false);
    assert_eq!(a.source_rect(), DsScreen::Top.source_rect(), "and back");
    // The game's own choice wins over the shelf's, and is kept with its view.
    let mut view = a.ds_view();
    view.stylus_screen = Some(false);
    a.set_ds_view(view);
    a.stylus_out(true);
    assert_eq!(a.source_rect(), DsScreen::Top.source_rect());
    assert_eq!(
        slot::video_mode::DsView::parse(&view.as_str())
            .unwrap()
            .stylus_screen,
        Some(false)
    );
    let _ = d;
}
