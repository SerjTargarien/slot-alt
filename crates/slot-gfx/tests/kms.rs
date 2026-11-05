#![cfg(target_os = "linux")]

use std::mem::{offset_of, size_of};

use slot_gfx::kms::{
    crtc_for, pick_connector, pick_mode, Connector, DrmConnector, DrmCrtc, DrmEncoder, DrmRes,
    EventContext, ModeInfo, CONNECTOR_DSI,
};

const CONNECTOR_HDMI_A: u32 = 11;

#[test]
fn the_structs_match_libdrms_layout() {
    assert_eq!(size_of::<ModeInfo>(), 68);
    assert_eq!(size_of::<DrmRes>(), 80);
    assert_eq!(size_of::<DrmConnector>(), 88);
    assert_eq!(size_of::<DrmEncoder>(), 20);
    assert_eq!(size_of::<DrmCrtc>(), 100);
    assert_eq!(size_of::<EventContext>(), 40);
    assert_eq!(offset_of!(DrmRes, connectors), 40);
    assert_eq!(offset_of!(DrmConnector, modes), 40);
    assert_eq!(offset_of!(DrmConnector, count_encoders), 72);
    assert_eq!(offset_of!(DrmConnector, encoders), 80);
    assert_eq!(offset_of!(DrmCrtc, mode_valid), 24);
    assert_eq!(offset_of!(DrmCrtc, mode), 28);
}

fn rgb20sx_mode() -> ModeInfo {
    let mut m = ModeInfo::blank();
    m.clock = 37000;
    m.hdisplay = 720;
    m.htotal = 814;
    m.vdisplay = 720;
    m.vtotal = 749;
    m.vrefresh = 61;
    m.kind = 1 << 3;
    m
}

#[test]
fn the_refresh_rate_comes_from_the_timings_not_the_rounded_field() {
    let hz = rgb20sx_mode().refresh_hz();
    assert!((hz - 60.69).abs() < 0.01, "{hz}");
    let mut untimed = ModeInfo::blank();
    untimed.vrefresh = 60;
    assert_eq!(untimed.refresh_hz(), 60.0);
}

#[test]
fn the_preferred_mode_wins_over_the_first() {
    let mut other = rgb20sx_mode();
    other.kind = 0;
    other.hdisplay = 640;
    assert_eq!(pick_mode(&[other, rgb20sx_mode()]), Some(1));
    assert_eq!(pick_mode(&[other]), Some(0));
    assert_eq!(pick_mode(&[]), None);
}

fn connector(id: u32, kind: u32, connected: bool) -> Connector {
    Connector {
        id,
        kind,
        connected,
        modes: vec![rgb20sx_mode()],
    }
}

#[test]
fn the_built_in_panel_is_chosen_even_with_hdmi_plugged_in() {
    let both = [
        connector(56, CONNECTOR_HDMI_A, true),
        connector(59, CONNECTOR_DSI, true),
    ];
    assert_eq!(pick_connector(&both), Some(1));
}

#[test]
fn an_unplugged_connector_is_never_chosen() {
    let only_hdmi_free = [
        connector(56, CONNECTOR_HDMI_A, false),
        connector(59, CONNECTOR_DSI, true),
    ];
    assert_eq!(pick_connector(&only_hdmi_free), Some(1));
    let hdmi_only = [
        connector(56, CONNECTOR_HDMI_A, true),
        connector(59, CONNECTOR_DSI, false),
    ];
    assert_eq!(pick_connector(&hdmi_only), Some(0));
    let mut no_modes = connector(59, CONNECTOR_DSI, true);
    no_modes.modes.clear();
    assert_eq!(pick_connector(&[no_modes]), None);
}

#[test]
fn a_crtc_is_found_from_the_encoders_bitmask() {
    assert_eq!(crtc_for(0b10, &[53, 54]), Some(54));
    assert_eq!(crtc_for(0b11, &[53, 54]), Some(53));
    assert_eq!(crtc_for(0b100, &[53, 54]), None);
}

#[test]
fn every_library_function_the_backend_calls_resolves() {
    match slot_gfx::kms::libraries_resolve() {
        Ok(()) => {}
        Err(e) if e.to_string().contains("cannot open shared object") => {
            eprintln!("skipped, the graphics libraries are not installed: {e}");
        }
        Err(e) => panic!("{e}"),
    }
}

mod flip_stats {
    use slot_gfx::kms::FlipStats;

    const REFRESH_HZ: f64 = 60.69;
    const FRAME: u64 = 16_477;

    fn run(stats: &mut FlipStats, start: u64, frames: u64) -> (u64, Option<String>) {
        let mut vblank = start;
        let mut last = None;
        for _ in 0..frames {
            vblank += FRAME;
            last = stats.record(800, vblank - 2_000, vblank).or(last);
        }
        (vblank, last)
    }

    #[test]
    fn on_time_flips_skip_nothing_and_report_every_600() {
        let mut stats = FlipStats::new(REFRESH_HZ);
        assert_eq!(stats.record(800, 0, 1_000_000), None);
        let (_, line) = run(&mut stats, 1_000_000, 600);
        let line = line.expect("a report after 600 flips");
        assert!(line.contains("600 flips, 0 skipped a vblank"), "{line}");
        assert!(line.contains("submit to scanout avg 2.00"), "{line}");
        assert!(line.contains("eglSwapBuffers avg 0.80"), "{line}");
    }

    #[test]
    fn a_frame_submitted_after_its_vblank_is_late() {
        let mut stats = FlipStats::new(REFRESH_HZ);
        stats.record(800, 0, 1_000_000);
        let missed = 1_000_000 + 2 * FRAME;
        stats.record(800, 1_000_000 + FRAME + 500, missed);
        let (_, line) = run(&mut stats, missed, 599);
        let line = line.unwrap();
        assert!(
            line.contains("1 skipped a vblank (1 submitted after"),
            "{line}"
        );
    }

    #[test]
    fn a_frame_submitted_in_time_but_shown_late_is_counted_apart() {
        let mut stats = FlipStats::new(REFRESH_HZ);
        stats.record(800, 0, 1_000_000);
        let missed = 1_000_000 + 2 * FRAME;
        stats.record(800, 1_000_000 + FRAME - 2_000, missed);
        let (_, line) = run(&mut stats, missed, 599);
        let line = line.unwrap();
        assert!(
            line.contains("1 skipped a vblank (0 submitted after"),
            "{line}"
        );
        assert!(line.contains("1 submitted in time"), "{line}");
    }
}
