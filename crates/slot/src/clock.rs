use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const CHECK_EVERY: u32 = 30;
const CALM_FRAMES: u32 = 180;
const UP_ABOVE: f64 = 0.85;
const LONE_MISS_ABOVE: f64 = 0.70;
const DOWN_BELOW: f64 = 0.75;
const MANY_MISSES: u32 = 3;

/// True when the CPU part of a frame (everything but the GPU wait) is at least as
/// long as the GPU wait. Only such frames can ask for a faster CPU. With no GPU wait,
/// as on BaseOS, every frame counts.
pub fn cpu_bound(work: Duration, gpu: Duration) -> bool {
    let gpu = gpu.min(work);
    work - gpu >= gpu
}

#[derive(Copy, Clone, Default)]
struct Peak {
    total: Duration,
    gpu: Duration,
}

impl Peak {
    fn take(&mut self, work: Duration, gpu: Duration) {
        self.total = self.total.max(work);
        self.gpu = self.gpu.max(gpu.min(work));
    }

    fn merge(&mut self, other: Peak) {
        self.total = self.total.max(other.total);
        self.gpu = self.gpu.max(other.gpu);
    }

    fn at(&self, ratio: f64) -> Duration {
        let cpu = self.total.saturating_sub(self.gpu);
        cpu.mul_f64(ratio) + self.gpu
    }
}

pub struct Governor {
    steps: Vec<u32>,
    index: usize,
    checked: u32,
    misses: u32,
    window: Peak,
    cpu_bound_peak: Duration,
    calm: u32,
    calm_peak: Peak,
}

impl Governor {
    pub fn new(mut steps: Vec<u32>) -> Option<Self> {
        steps.sort_unstable();
        steps.dedup();
        let index = steps.len().checked_sub(1)?;
        Some(Governor {
            steps,
            index,
            checked: 0,
            misses: 0,
            window: Peak::default(),
            cpu_bound_peak: Duration::ZERO,
            calm: 0,
            calm_peak: Peak::default(),
        })
    }

    pub fn khz(&self) -> u32 {
        self.steps[self.index]
    }

    pub fn frame(
        &mut self,
        work: Duration,
        gpu: Duration,
        missed: bool,
        budget: Duration,
    ) -> Option<u32> {
        let cpu_bound = cpu_bound(work, gpu);
        self.checked += 1;
        self.misses += u32::from(missed && cpu_bound);
        if cpu_bound {
            self.cpu_bound_peak = self.cpu_bound_peak.max(work);
        }
        self.window.take(work, gpu);
        if self.checked < CHECK_EVERY {
            return None;
        }
        let (misses, peak) = (self.misses, std::mem::take(&mut self.window));
        let cpu_peak = std::mem::take(&mut self.cpu_bound_peak);
        self.checked = 0;
        self.misses = 0;
        let cpu_bound_miss =
            misses > 1 || (misses == 1 && cpu_peak > budget.mul_f64(LONE_MISS_ABOVE));
        if cpu_bound_miss || cpu_peak > budget.mul_f64(UP_ABOVE) {
            self.calm = 0;
            self.calm_peak = Peak::default();
            let jump = if misses >= MANY_MISSES { 2 } else { 1 };
            return self.set((self.index + jump).min(self.steps.len() - 1));
        }
        self.calm += CHECK_EVERY;
        self.calm_peak.merge(peak);
        if self.calm < CALM_FRAMES || self.index == 0 {
            return None;
        }
        let ratio = f64::from(self.khz()) / f64::from(self.steps[self.index - 1]);
        let predicted = self.calm_peak.at(ratio);
        self.calm = 0;
        self.calm_peak = Peak::default();
        match predicted < budget.mul_f64(DOWN_BELOW) {
            true => self.set(self.index - 1),
            false => None,
        }
    }

    /// Called every frame while a game launches: the cap goes to the top step so the
    /// game never starts at the menu's clock, and the calm count restarts, so it steps
    /// down again only after calm seconds of play.
    pub fn launching(&mut self) -> Option<u32> {
        self.forget();
        self.set(self.steps.len() - 1)
    }

    /// Called every frame while a game is put away. Its save takes one long frame that
    /// a faster clock would not shorten, so those frames are left out.
    pub fn leaving(&mut self) {
        self.forget();
    }

    fn forget(&mut self) {
        self.checked = 0;
        self.misses = 0;
        self.window = Peak::default();
        self.cpu_bound_peak = Duration::ZERO;
        self.calm = 0;
        self.calm_peak = Peak::default();
    }

    fn set(&mut self, index: usize) -> Option<u32> {
        let changed = index != self.index;
        self.index = index;
        changed.then(|| self.khz())
    }
}

pub fn parse_steps(text: &str) -> Vec<u32> {
    text.split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect()
}

pub struct CpuCap {
    caps: Vec<(PathBuf, String)>,
}

impl CpuCap {
    pub fn open(sysfs: &Path) -> Option<(CpuCap, Vec<u32>)> {
        let mut policies: Vec<PathBuf> = fs::read_dir(sysfs.join("devices/system/cpu/cpufreq"))
            .ok()?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with("policy"))
            })
            .collect();
        policies.sort();
        let steps = policies
            .first()
            .and_then(|p| fs::read_to_string(p.join("scaling_available_frequencies")).ok())
            .map(|t| parse_steps(&t))
            .filter(|s| !s.is_empty())?;
        let caps = policies
            .into_iter()
            .map(|p| p.join("scaling_max_freq"))
            .filter_map(|f| {
                let original = fs::read_to_string(&f).ok()?.trim().to_string();
                Some((f, original))
            })
            .collect::<Vec<_>>();
        (!caps.is_empty()).then_some((CpuCap { caps }, steps))
    }

    pub fn set(&self, khz: u32) {
        for (f, _) in &self.caps {
            if let Err(e) = fs::write(f, format!("{khz}\n")) {
                eprintln!("slot: clock: {}: {e}", f.display());
            }
        }
    }
}

impl Drop for CpuCap {
    fn drop(&mut self) {
        for (f, original) in &self.caps {
            let _ = fs::write(f, format!("{original}\n"));
        }
    }
}
