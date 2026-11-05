use std::path::PathBuf;
use std::time::{Duration, Instant};

use slot::frontend::Frontend;
use slot::input::DeviceInput;
use slot::panel_turn::{panel_turn, PanelTurn};
use slot_gfx::{Compositor, FbdevSurface, GfxError, Surface};
use slot_power::{trace_first_frame, DevicePlatform};

const CARD: &str = "/mnt/sdcard";

const MIN_FRAME: Duration = Duration::from_millis(12);

const STEP_TIMEOUT: Duration = Duration::from_millis(12);

const MARGIN: Duration = Duration::from_micros(1500);

const LOG_SYNC: Duration = Duration::from_secs(1);

struct Pacer {
    last_return: Option<Instant>,
    margin: Duration,
    period: Duration,
    work: Duration,
    works: [Duration; WORK_WINDOW],
    next: usize,
}

const WORK_WINDOW: usize = 32;

impl Pacer {
    fn new(surface_margin: Option<Duration>) -> Self {
        let margin = std::env::var("SLOT_PACE_MARGIN_US")
            .ok()
            .and_then(|v| v.trim().parse().ok())
            .map(Duration::from_micros)
            .or(surface_margin)
            .unwrap_or(MARGIN);
        if margin != MARGIN {
            eprintln!("slot: pace margin {} us", margin.as_micros());
        }
        Pacer {
            last_return: None,
            margin,
            period: Duration::from_micros(16_760),
            work: Duration::from_millis(4),
            works: [Duration::from_millis(4); WORK_WINDOW],
            next: 0,
        }
    }

    fn wait(&self) {
        let Some(last) = self.last_return else {
            return;
        };
        let delay = self.period.saturating_sub(self.work + self.margin);
        if let Some(left) = (last + delay).checked_duration_since(Instant::now()) {
            std::thread::sleep(left);
        }
    }

    fn swapped(&mut self, work: Duration, blocked: bool) -> bool {
        let now = Instant::now();
        let mut missed = false;
        if let Some(last) = self.last_return {
            let seen = now - last;
            missed = seen > self.period.mul_f64(1.5);
            if blocked && seen > Duration::from_millis(12) && seen < Duration::from_millis(22) {
                self.period = self.period.mul_f64(0.95) + seen.mul_f64(0.05);
            }
        }
        self.works[self.next] = work;
        self.next = (self.next + 1) % WORK_WINDOW;
        self.work = self.works.iter().copied().max().unwrap_or(work);
        self.last_return = Some(now);
        missed
    }
}

pub fn run() {
    slot::boot_time::mark("start");
    let _ = std::thread::Builder::new()
        .name("slot-fonts".into())
        .spawn(slot_ui::warm_fonts);
    let audio = std::thread::Builder::new()
        .name("slot-audio-open".into())
        .spawn(slot::audio::opened_sink);
    let root = std::env::var_os("SLOT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(CARD));
    let rocknix = slot::rocknix::running();
    if rocknix {
        eprintln!("slot: running on ROCKNIX, model {}", slot::rocknix::model());
    }
    let mut surface = match open_surface(rocknix) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("slot: {e}");
            return;
        }
    };
    slot::boot_time::mark("surface");
    let mut compositor = match Compositor::new(&*surface) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("slot: {e}");
            return;
        }
    };
    slot::boot_time::mark("compositor");
    let panel = surface.window_size();
    let env = |key| std::env::var(key).ok();
    let turn = panel_turn(
        panel,
        slot::baseos::var("BASEOS_PANEL_ROTATION_CCW").as_deref(),
        env("SLOT_ROTATE").as_deref(),
    );
    let turned = turn != PanelTurn::None;
    let window = if turned { (panel.1, panel.0) } else { panel };
    eprintln!(
        "slot: baseos target {}, model {}, rotation ccw {}",
        slot::baseos::var("BASEOS_TARGET").unwrap_or_default(),
        slot::baseos::var("BASEOS_MODEL").unwrap_or_default(),
        slot::baseos::var("BASEOS_PANEL_ROTATION_CCW").unwrap_or_default()
    );
    eprintln!(
        "slot: panel {}x{}, canvas {}x{}, turn {turn:?}",
        panel.0, panel.1, window.0, window.1
    );
    if turned {
        if let Err(e) = compositor.set_panel_turn(panel, turn == PanelTurn::Ccw) {
            eprintln!("slot: {e}");
            return;
        }
    }
    let platform = DevicePlatform::new(root.clone());
    eprintln!("slot: {}", platform.report());
    platform.trace_boot();
    slot::boot_time::mark("platform");
    let sink = match audio.map(|h| h.join()) {
        Ok(Ok(sink)) => sink,
        _ => slot::audio::opened_sink(),
    };
    slot::boot_time::mark("audio");
    let mut frontend = Frontend::boot_with(Box::new(platform), sink);
    slot::boot_time::mark("frontend");
    let target = slot::baseos::var("BASEOS_TARGET").unwrap_or_default();
    frontend.set_left_stick(slot::input::evdev::has_left_stick(&target));
    frontend.upload_boot_faces(&mut compositor);
    slot::boot_time::mark("faces");
    frontend.upload_bezel(&mut compositor, window);
    slot::boot_time::mark("bezel");
    if rocknix {
        slot::input::evdev::set_layout(slot::input::evdev::Layout::Gamepad);
    }
    let mut input = DeviceInput::open(&root);
    let fn_adc = std::path::Path::new(slot::input::adc::RGB20SX_FN);
    if rocknix && slot::rocknix::model() == slot::rocknix::RGB20SX && fn_adc.exists() {
        input.watch_adc(fn_adc.to_path_buf(), slot_input::Btn::Menu);
    }
    slot::boot_time::mark("input");
    let card = root.clone();
    let _ = std::thread::Builder::new()
        .name("slot-bootlogo".into())
        .spawn(move || {
            if !rocknix {
                slot::bootlogo::refresh(&card);
            }
            slot::root::sweep_temps(&card);
        });
    let _ = std::thread::Builder::new()
        .name("slot-logsync".into())
        .spawn(sync_log_writes);
    let mut drawn = false;
    frontend.drive_emulator();
    let mut pacer = Pacer::new(surface.pace_margin());
    let mut clock = frame_clock(rocknix);
    let (mut frames, mut missed, mut early, mut stale) = (0u32, 0u32, 0u32, 0u32);
    let (mut work_sum, mut work_max) = (Duration::ZERO, Duration::ZERO);
    let (mut gpu_sum, mut gpu_max) = (Duration::ZERO, Duration::ZERO);
    let (mut cpu_sum, mut cpu_max) = (Duration::ZERO, Duration::ZERO);
    let (mut cpu_misses, mut gpu_misses) = (0u32, 0u32);
    loop {
        pacer.wait();
        let began = Instant::now();
        input.steer_with_stick(frontend.app().left_stick_dpad());
        frontend.advance(&mut input);
        if frontend.restarting() {
            frontend.restart();
        }
        if frontend.relaunching() {
            frontend.relaunch();
            return;
        }
        if frontend.repairing() {
            frontend.repair();
        }
        if frontend.powering_off() {
            frontend.poweroff();
            return;
        }
        if slot::crash::stopping() {
            eprintln!("slot: told to stop, saving first");
            frontend.stop();
            return;
        }
        let late = frontend.step_emulator(pacer.period, STEP_TIMEOUT);
        frontend.render(&mut compositor, window);
        let settling = Instant::now();
        surface.settle();
        let swap = Instant::now();
        let gpu = swap - settling;
        let work = swap - began;
        if let Err(e) = surface.swap() {
            eprintln!("slot: {e}");
            frontend.stop();
            return;
        }
        let swap_took = swap.elapsed();
        slot::latency::swapped(swap_took.as_secs_f64() * 1000.0);
        let dropped = pacer.swapped(work, swap_took > Duration::from_millis(1));
        if let Some((cap, governor)) = &mut clock {
            let budget = pacer.period.saturating_sub(pacer.margin);
            let (khz, why) = if frontend.launching() {
                (governor.launching(), " for the launch")
            } else if frontend.leaving() {
                governor.leaving();
                (None, "")
            } else {
                (governor.frame(work, gpu, dropped || late, budget), "")
            };
            if let Some(khz) = khz {
                cap.set(khz);
                eprintln!("slot: clock: {} MHz{why}", khz / 1000);
            }
        }
        if slot::latency::tracing() {
            frames += 1;
            missed += u32::from(dropped);
            early += u32::from(swap_took > pacer.period / 2);
            stale += u32::from(late);
            (work_sum, work_max) = (work_sum + work, work_max.max(work));
            (gpu_sum, gpu_max) = (gpu_sum + gpu, gpu_max.max(gpu));
            let cpu = work.saturating_sub(gpu);
            (cpu_sum, cpu_max) = (cpu_sum + cpu, cpu_max.max(cpu));
            if dropped {
                match slot::clock::cpu_bound(work, gpu) {
                    true => cpu_misses += 1,
                    false => gpu_misses += 1,
                }
            }
            if frames == 600 {
                let ms = |d: Duration| d.as_secs_f64() * 1e3;
                let cap = match &clock {
                    Some((_, governor)) => format!("{} MHz", governor.khz() / 1000),
                    None => "off".to_string(),
                };
                eprintln!(
                    "slot: pace: 600 frames, {missed} missed a latch ({cpu_misses} cpu-bound, {gpu_misses} gpu-bound), {early} drawn early, {stale} stale, work {:.1} ms, period {:.2} ms, frame avg {:.1} max {:.1} ms, cpu part avg {:.1} max {:.1} ms, gpu wait avg {:.1} max {:.1} ms, cpu cap {cap}",
                    ms(pacer.work),
                    ms(pacer.period),
                    ms(work_sum / 600),
                    ms(work_max),
                    ms(cpu_sum / 600),
                    ms(cpu_max),
                    ms(gpu_sum / 600),
                    ms(gpu_max)
                );
                (frames, missed, early, stale) = (0, 0, 0, 0);
                (work_sum, work_max) = (Duration::ZERO, Duration::ZERO);
                (gpu_sum, gpu_max) = (Duration::ZERO, Duration::ZERO);
                (cpu_sum, cpu_max) = (Duration::ZERO, Duration::ZERO);
                (cpu_misses, gpu_misses) = (0, 0);
            }
        }
        if !drawn {
            drawn = true;
            slot::boot_time::mark("first frame");
            trace_first_frame();
        }
        if let Some(left) = MIN_FRAME.checked_sub(began.elapsed()) {
            std::thread::sleep(left);
        }
    }
}

fn frame_clock(rocknix: bool) -> Option<(slot::clock::CpuCap, slot::clock::Governor)> {
    if !rocknix || std::env::var("SLOT_CLOCK").is_ok_and(|v| v == "max") {
        return None;
    }
    let (cap, steps) = slot::clock::CpuCap::open(std::path::Path::new("/sys"))?;
    let governor = slot::clock::Governor::new(steps)?;
    cap.set(governor.khz());
    eprintln!(
        "slot: clock: capping the CPU to the frame budget, starting at {} MHz",
        governor.khz() / 1000
    );
    Some((cap, governor))
}

fn open_surface(rocknix: bool) -> Result<Box<dyn Surface>, GfxError> {
    #[cfg(target_os = "linux")]
    if rocknix {
        return slot_gfx::KmsSurface::new().map(|s| Box::new(s) as Box<dyn Surface>);
    }
    let _ = rocknix;
    FbdevSurface::new().map(|s| Box::new(s) as Box<dyn Surface>)
}

fn sync_log_writes() {
    let mut synced = 0;
    loop {
        std::thread::sleep(LOG_SYNC);
        let mut st: libc::stat = unsafe { std::mem::zeroed() };
        if unsafe { libc::fstat(2, &mut st) } != 0 {
            return;
        }
        if st.st_size != synced {
            #[cfg(target_os = "linux")]
            unsafe {
                libc::syncfs(2);
            }
            synced = st.st_size;
        }
    }
}
