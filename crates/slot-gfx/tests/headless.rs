//! Slot²: the render tests skip themselves when no test surface starts, so this one
//! fails instead: on the machines Slot² is built on (macOS, or Linux with Mesa's EGL),
//! a skipped render test would otherwise pass unseen.

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[test]
fn the_test_surface_starts_so_the_render_tests_run() {
    use slot_gfx::{Compositor, HeadlessSurface};
    let surface = HeadlessSurface::new().expect("the test surface");
    Compositor::new(&surface).expect("a compositor on it");
}
