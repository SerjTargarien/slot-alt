use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};

use slot_input::{Btn, RawEvent};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Layout {
    Anbernic,
    Gamepad,
}

static LAYOUT: AtomicU8 = AtomicU8::new(0);

pub fn set_layout(layout: Layout) {
    LAYOUT.store(layout as u8, Ordering::Relaxed);
}

pub fn layout() -> Layout {
    match LAYOUT.load(Ordering::Relaxed) {
        1 => Layout::Gamepad,
        _ => Layout::Anbernic,
    }
}

/// Slot²: the sticks the device has (bit 0 the left, bit 1 the right; `parts.rs`). Axes of
/// a stick it doesn't have are ignored, as upstream ignored them on models without one.
static STICKS: AtomicU8 = AtomicU8::new(0b11);

pub fn set_sticks(left: bool, right: bool) {
    STICKS.store(u8::from(left) | u8::from(right) << 1, Ordering::Relaxed);
}

fn sticks() -> [bool; 2] {
    let bits = STICKS.load(Ordering::Relaxed);
    [bits & 1 != 0, bits & 2 != 0]
}

pub const EVENT_BYTES: usize = 24;

pub const EV_SYN: u16 = 0x00;
pub const EV_KEY: u16 = 0x01;
pub const EV_SW: u16 = 0x05;
pub const SW_LID: u16 = 0x00;

pub const EV_ABS: u16 = 0x03;
pub const ABS_X: u16 = 0x00;
pub const ABS_Y: u16 = 0x01;
pub const ABS_RX: u16 = 0x03;
pub const ABS_RY: u16 = 0x04;
pub const ABS_HAT0X: u16 = 0x10;
pub const ABS_HAT0Y: u16 = 0x11;
/// The left stick on the Anbernic H700 handhelds.
pub const ABS_LSX: u16 = 0x02;
pub const ABS_LSY: u16 = 0x03;

/// How far a stick goes before it presses a direction, and how far back before it lets
/// go, as fractions of its travel from the centre. The gap stops a direction chattering
/// when the stick rests near the line.
const STICK_PRESS: f32 = 0.5;
const STICK_RELEASE: f32 = 0.35;

/// A stick's travel when the kernel can't say (the Anbernic H700's).
const STICK_TRAVEL: (i32, i32) = (-4096, 4096);

/// The stick (0 left, 1 right) and the axis (0 across, 1 down) an axis code reports.
fn stick_axis(layout: Layout, code: u16) -> Option<(usize, usize)> {
    match (layout, code) {
        (Layout::Gamepad, ABS_X) => Some((0, 0)),
        (Layout::Gamepad, ABS_Y) => Some((0, 1)),
        (Layout::Gamepad, ABS_RX) => Some((1, 0)),
        (Layout::Gamepad, ABS_RY) => Some((1, 1)),
        (Layout::Anbernic, ABS_LSX) => Some((0, 0)),
        (Layout::Anbernic, ABS_LSY) => Some((0, 1)),
        _ => None,
    }
}

/// The axis codes that are sticks in a layout.
pub fn stick_codes(layout: Layout) -> &'static [u16] {
    match layout {
        Layout::Gamepad => &[ABS_X, ABS_Y, ABS_RX, ABS_RY],
        Layout::Anbernic => &[ABS_LSX, ABS_LSY],
    }
}

/// The D-pad hat and both sticks, as buttons. The D-pad presses Up, Down, Left and
/// Right; each stick presses its own directions (LS Up..., RS Up...) past half travel.
/// While `steers` (outside a game) the left stick moves like the D-pad instead, so the
/// menus and the shelf follow it.
pub struct Hat {
    layout: Layout,
    sticks: [bool; 2],
    hat: [i32; 2],
    stick: [[i32; 2]; 2],
    travel: [[(i32, i32); 2]; 2],
    steers: bool,
    down: Vec<Btn>,
}

impl Default for Hat {
    fn default() -> Self {
        Hat::new(layout())
    }
}

impl Hat {
    pub fn new(layout: Layout) -> Self {
        Hat {
            layout,
            sticks: sticks(),
            hat: [0; 2],
            stick: [[0; 2]; 2],
            travel: [[STICK_TRAVEL; 2]; 2],
            steers: false,
            down: Vec::new(),
        }
    }

    /// Slot²: which sticks this reader takes (the device's, `set_sticks`, unless told).
    pub fn with_sticks(mut self, sticks: [bool; 2]) -> Self {
        self.sticks = sticks;
        self
    }

    /// An axis's travel as the kernel reports it, from its lowest to its highest value.
    pub fn set_travel(&mut self, code: u16, min: i32, max: i32) {
        if let Some((s, a)) = stick_axis(self.layout, code).filter(|_| max > min) {
            self.travel[s][a] = (min, max);
        }
    }

    pub fn steer_with_stick(&mut self, on: bool) {
        self.steers = on;
    }

    pub fn feed(&mut self, ev: Ev) -> Vec<RawEvent> {
        match ev.code {
            ABS_HAT0X => self.hat[0] = ev.value.signum(),
            ABS_HAT0Y => self.hat[1] = ev.value.signum(),
            code => {
                let Some((s, a)) = stick_axis(self.layout, code).filter(|(s, _)| self.sticks[*s])
                else {
                    return Vec::new();
                };
                self.stick[s][a] = tilt(ev.value, self.travel[s][a], self.stick[s][a]);
            }
        }
        let want = self.wanted();
        let mut out: Vec<RawEvent> = self
            .down
            .iter()
            .filter(|b| !want.contains(b))
            .map(|b| RawEvent::Up(*b))
            .collect();
        out.extend(
            want.iter()
                .filter(|b| !self.down.contains(b))
                .map(|b| RawEvent::Down(*b)),
        );
        self.down = want;
        out
    }

    /// The directions held now. The D-pad wins over a steering stick on the same axis.
    fn wanted(&self) -> Vec<Btn> {
        let ends = |at: usize, v: i32, pad: [[Btn; 2]; 2]| pad[at][usize::from(v > 0)];
        const DPAD: [[Btn; 2]; 2] = [[Btn::Left, Btn::Right], [Btn::Up, Btn::Down]];
        const LS: [[Btn; 2]; 2] = [[Btn::LsLeft, Btn::LsRight], [Btn::LsUp, Btn::LsDown]];
        const RS: [[Btn; 2]; 2] = [[Btn::RsLeft, Btn::RsRight], [Btn::RsUp, Btn::RsDown]];
        let mut want = Vec::new();
        for axis in 0..2 {
            let (hat, left, right) = (self.hat[axis], self.stick[0][axis], self.stick[1][axis]);
            if hat != 0 {
                want.push(ends(axis, hat, DPAD));
            }
            match (left, self.steers) {
                (0, _) => {}
                (v, true) if hat == 0 => want.push(ends(axis, v, DPAD)),
                (_, true) => {}
                (v, false) => want.push(ends(axis, v, LS)),
            }
            if right != 0 {
                want.push(ends(axis, right, RS));
            }
        }
        want
    }
}

/// Where a stick axis points (-1, 0 or 1), from its value, its travel and where it
/// pointed before.
fn tilt(value: i32, (min, max): (i32, i32), was: i32) -> i32 {
    let half = (max - min) as f32 / 2.0;
    let at = (value as f32 - (min as f32 + half)) / half;
    let past = |v: f32| at * v.signum() > 0.0;
    match was {
        w if w != 0 && past(w as f32) && at.abs() >= STICK_RELEASE => w,
        _ if at >= STICK_PRESS => 1,
        _ if at <= -STICK_PRESS => -1,
        _ => 0,
    }
}

/// Reads each stick axis's travel from the kernel (`EVIOCGABS`), so a stick reads the same
/// whatever range its device reports.
pub fn read_travel(file: &std::fs::File, hat: &mut Hat) {
    use std::os::fd::AsRawFd;
    for &code in stick_codes(hat.layout) {
        // struct input_absinfo: value, minimum, maximum, fuzz, flat, resolution.
        let mut info = [0i32; 6];
        let request = (2u32 << 30) | (24 << 16) | (u32::from(b'E') << 8) | (0x40 + u32::from(code));
        // SAFETY: EVIOCGABS writes one input_absinfo (six i32s) into `info`.
        let read = unsafe { libc::ioctl(file.as_raw_fd(), request as _, info.as_mut_ptr()) };
        if read == 0 {
            hat.set_travel(code, info[1], info[2]);
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Ev {
    pub kind: u16,
    pub code: u16,
    pub value: i32,
}

pub fn decode(bytes: &[u8]) -> Option<Ev> {
    let packet: &[u8; EVENT_BYTES] = bytes.get(..EVENT_BYTES)?.try_into().ok()?;
    Some(Ev {
        kind: u16::from_le_bytes([packet[16], packet[17]]),
        code: u16::from_le_bytes([packet[18], packet[19]]),
        value: i32::from_le_bytes([packet[20], packet[21], packet[22], packet[23]]),
    })
}

pub fn to_raw(ev: Ev) -> Option<RawEvent> {
    to_raw_in(layout(), ev)
}

pub fn to_raw_in(layout: Layout, ev: Ev) -> Option<RawEvent> {
    let btn = match ev.kind {
        EV_KEY => code_to_btn_in(layout, ev.code)?,
        EV_SW if ev.code == SW_LID => Btn::Lid,
        _ => return None,
    };
    match ev.value {
        0 => Some(RawEvent::Up(btn)),
        1 => Some(RawEvent::Down(btn)),
        _ => None,
    }
}

pub fn code_to_btn(code: u16) -> Option<Btn> {
    code_to_btn_in(layout(), code)
}

pub fn code_to_btn_in(layout: Layout, code: u16) -> Option<Btn> {
    match layout {
        Layout::Anbernic => anbernic_btn(code),
        Layout::Gamepad => gamepad_btn(code),
    }
}

fn gamepad_btn(code: u16) -> Option<Btn> {
    Some(match code {
        0x131 => Btn::A,
        0x130 => Btn::B,
        0x133 => Btn::X,
        0x134 => Btn::Y,
        0x136 => Btn::L1,
        0x137 => Btn::R1,
        0x138 => Btn::L2,
        0x139 => Btn::R2,
        0x13a => Btn::Select,
        0x13b => Btn::Start,
        0x13c => Btn::Menu,
        0x13d => Btn::L3,
        0x13e => Btn::R3,
        0x220 => Btn::Up,
        0x221 => Btn::Down,
        0x222 => Btn::Left,
        0x223 => Btn::Right,
        115 => Btn::VolUp,
        114 => Btn::VolDown,
        116 => Btn::Power,
        _ => return None,
    })
}

fn anbernic_btn(code: u16) -> Option<Btn> {
    Some(match code {
        0x130 => Btn::A,
        0x131 => Btn::B,
        0x132 => Btn::Y,
        0x133 => Btn::X,
        0x134 => Btn::L1,
        0x135 => Btn::R1,
        0x136 => Btn::Select,
        0x137 => Btn::Start,
        0x138 => Btn::Menu,
        0x13a => Btn::L2,
        0x13b => Btn::R2,
        115 => Btn::VolUp,
        114 => Btn::VolDown,
        116 => Btn::Power,
        _ => return None,
    })
}

const WANTED: [u16; 15] = [
    0x130, 0x131, 0x132, 0x133, 0x134, 0x135, 0x136, 0x137, 0x138, 0x13a, 0x13b, 0x162, 115, 114,
    116,
];

const GAMEPAD_WANTED: [u16; 18] = [
    0x130, 0x131, 0x133, 0x134, 0x136, 0x137, 0x138, 0x139, 0x13a, 0x13b, 0x13c, 0x220, 0x221,
    0x222, 0x223, 115, 114, 116,
];

fn wanted(layout: Layout) -> &'static [u16] {
    match layout {
        Layout::Anbernic => &WANTED,
        Layout::Gamepad => &GAMEPAD_WANTED,
    }
}

pub use slot_power::has_bit;

pub fn pick_devices(dev: &Path, sys: &Path) -> Vec<PathBuf> {
    pick_devices_in(dev, sys, layout())
}

pub fn pick_devices_in(dev: &Path, sys: &Path, layout: Layout) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dev) else {
        return Vec::new();
    };
    let mut nodes: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("event"))
        })
        .collect();
    nodes.sort();
    nodes.retain(|node| {
        let Some(name) = node.file_name() else {
            return false;
        };
        wanted_node(&sys.join(name).join("device"), layout)
    });
    nodes
}

fn wanted_node(device: &Path, layout: Layout) -> bool {
    let cap = |file: &str| std::fs::read_to_string(device.join("capabilities").join(file));
    let keys = cap("key").unwrap_or_default();
    let lid = cap("sw").is_ok_and(|sw| has_bit(&sw, SW_LID));
    // A joystick with no buttons of its own (the RGB20SX's ADC sticks).
    let abs = cap("abs").unwrap_or_default();
    let present = sticks();
    let sticks = stick_codes(layout)
        .iter()
        .filter(|code| stick_axis(layout, **code).is_some_and(|(s, _)| present[s]))
        .any(|code| has_bit(&abs, *code));
    lid || sticks || wanted(layout).iter().any(|bit| has_bit(&keys, *bit))
}

pub fn device_name(sys: &Path, node: &Path) -> String {
    node.file_name()
        .and_then(|n| std::fs::read_to_string(sys.join(n).join("device/name")).ok())
        .map(|n| n.trim().to_string())
        .unwrap_or_else(|| node.display().to_string())
}
