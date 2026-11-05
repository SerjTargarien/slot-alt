use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const CHECK_EVERY: u32 = 30;
const CALM_FRAMES: u32 = 180;
const UP_ABOVE: f64 = 0.85;
const DOWN_BELOW: f64 = 0.65;
const MANY_MISSES: u32 = 3;

pub struct Governor {
    steps: Vec<u32>,
    index: usize,
    checked: u32,
    misses: u32,
    window_max: Duration,
    calm: u32,
    calm_max: Duration,
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
            window_max: Duration::ZERO,
            calm: 0,
            calm_max: Duration::ZERO,
        })
    }

    pub fn khz(&self) -> u32 {
        self.steps[self.index]
    }

    pub fn reset_to_top(&mut self) -> Option<u32> {
        self.calm = 0;
        self.calm_max = Duration::ZERO;
        self.set(self.steps.len() - 1)
    }

    pub fn frame(&mut self, work: Duration, missed: bool, budget: Duration) -> Option<u32> {
        self.checked += 1;
        self.misses += u32::from(missed);
        self.window_max = self.window_max.max(work);
        if self.checked < CHECK_EVERY {
            return None;
        }
        let (misses, peak) = (self.misses, self.window_max);
        self.checked = 0;
        self.misses = 0;
        self.window_max = Duration::ZERO;
        if misses > 0 || peak > budget.mul_f64(UP_ABOVE) {
            self.calm = 0;
            self.calm_max = Duration::ZERO;
            let jump = if misses >= MANY_MISSES { 2 } else { 1 };
            return self.set((self.index + jump).min(self.steps.len() - 1));
        }
        self.calm += CHECK_EVERY;
        self.calm_max = self.calm_max.max(peak);
        if self.calm < CALM_FRAMES || self.index == 0 {
            return None;
        }
        let lower = self.steps[self.index - 1];
        let scaled = self
            .calm_max
            .mul_f64(f64::from(self.khz()) / f64::from(lower));
        self.calm = 0;
        self.calm_max = Duration::ZERO;
        match scaled < budget.mul_f64(DOWN_BELOW) {
            true => self.set(self.index - 1),
            false => None,
        }
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
