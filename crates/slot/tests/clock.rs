use std::time::Duration;

use slot::clock::{parse_steps, CpuCap, Governor};

const RK3566: &str = "408000 600000 816000 1104000 1416000 1608000 1800000 \n";

fn budget() -> Duration {
    Duration::from_micros(16_680 - 4_000)
}

fn governor() -> Governor {
    Governor::new(parse_steps(RK3566)).unwrap()
}

fn run(g: &mut Governor, frames: u32, work_ms: f64, missed: bool) -> Vec<u32> {
    (0..frames)
        .filter_map(|_| g.frame(Duration::from_secs_f64(work_ms / 1000.0), missed, budget()))
        .collect()
}

#[test]
fn the_cap_starts_at_the_top_step() {
    assert_eq!(governor().khz(), 1_800_000);
    assert!(Governor::new(Vec::new()).is_none());
}

#[test]
fn calm_frames_step_the_cap_down_one_step_every_three_seconds() {
    let mut g = governor();
    assert_eq!(run(&mut g, 179, 2.0, false), Vec::<u32>::new());
    assert_eq!(run(&mut g, 1, 2.0, false), vec![1_608_000]);
    assert_eq!(run(&mut g, 180, 2.0, false), vec![1_416_000]);
}

#[test]
fn light_work_settles_at_the_lowest_step_that_still_fits() {
    let mut g = governor();
    run(&mut g, 180 * 20, 1.0, false);
    assert_eq!(g.khz(), 408_000);
}

#[test]
fn work_that_grows_as_the_clock_drops_settles_where_it_still_fits() {
    let mut g = governor();
    for _ in 0..180 * 20 {
        let work_ms = 4.0 * 1_800_000.0 / f64::from(g.khz());
        g.frame(Duration::from_secs_f64(work_ms / 1000.0), false, budget());
    }
    assert_eq!(g.khz(), 1_104_000);
}

#[test]
fn a_missed_frame_steps_up_at_the_next_check() {
    let mut g = governor();
    run(&mut g, 180 * 3, 2.0, false);
    let before = g.khz();
    let mut ups = run(&mut g, 1, 2.0, true);
    ups.extend(run(&mut g, 29, 2.0, false));
    assert_eq!(ups.len(), 1);
    assert!(ups[0] > before);
}

#[test]
fn several_misses_jump_two_steps() {
    let mut g = governor();
    run(&mut g, 180 * 4, 2.0, false);
    assert_eq!(g.khz(), 816_000);
    run(&mut g, 3, 2.0, true);
    run(&mut g, 27, 2.0, false);
    assert_eq!(g.khz(), 1_416_000);
}

#[test]
fn work_near_the_budget_steps_up_without_a_miss() {
    let mut g = governor();
    run(&mut g, 180 * 3, 2.0, false);
    let before = g.khz();
    run(&mut g, 30, 11.5, false);
    assert!(g.khz() > before);
}

#[test]
fn a_step_up_holds_off_the_next_step_down_for_three_seconds() {
    let mut g = governor();
    run(&mut g, 180 * 3, 2.0, false);
    run(&mut g, 30, 2.0, true);
    let after_up = g.khz();
    assert_eq!(run(&mut g, 150, 2.0, false), Vec::<u32>::new());
    assert_eq!(g.khz(), after_up);
}

#[test]
fn the_top_step_is_never_exceeded() {
    let mut g = governor();
    assert_eq!(run(&mut g, 300, 15.0, true), Vec::<u32>::new());
    assert_eq!(g.khz(), 1_800_000);
}

#[test]
fn the_cap_is_written_to_every_policy_and_restored_on_drop() {
    let d = tempfile::tempdir().unwrap();
    let policy = d.path().join("devices/system/cpu/cpufreq/policy0");
    std::fs::create_dir_all(&policy).unwrap();
    std::fs::write(policy.join("scaling_available_frequencies"), RK3566).unwrap();
    std::fs::write(policy.join("scaling_max_freq"), "1800000\n").unwrap();
    let (cap, steps) = CpuCap::open(d.path()).unwrap();
    assert_eq!(steps.len(), 7);
    cap.set(816_000);
    let read = || std::fs::read_to_string(policy.join("scaling_max_freq")).unwrap();
    assert_eq!(read().trim(), "816000");
    drop(cap);
    assert_eq!(read().trim(), "1800000");
}

#[test]
fn no_cpufreq_means_no_cap() {
    let d = tempfile::tempdir().unwrap();
    assert!(CpuCap::open(d.path()).is_none());
}
