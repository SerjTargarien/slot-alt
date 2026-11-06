//! Slot²: each system's own controls (design/controls.md). Every system lists its inputs,
//! each input takes up to three bindings, and each system keeps its own file,
//! `Config/controls/<system>.txt`. Upstream's single `Config/controls.txt` seeds them.

use std::path::{Path, PathBuf};

use crate::bindings::{read_controls, Binding, Controls, Device, GameButton, Modifier};
use crate::platform::Platform;

/// The bindings one input can take.
pub const SLOTS: usize = 3;

/// One input's bindings, oldest first, filled from the front.
pub type Row = [Option<Binding>; SLOTS];

const DIR: &str = "controls";

/// An emulated input. Which ones a system has, and what each is called on its page, is up
/// to the system (`inputs`, `label`); which core button each drives is up to the frontend.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Input {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    X,
    Y,
    L,
    R,
    Select,
    Start,
    TurboA,
    TurboB,
    /// The WonderSwan's second four-way pad (its first is Up, Down, Left and Right).
    Y1,
    Y2,
    Y3,
    Y4,
    /// The DS's slot controls, which drive slot rather than the core: swap the main
    /// screen, step through the layouts, hold the stylus out, and touch with it. While the
    /// stylus is out, the D-pad's inputs move it and Touch's buttons touch.
    SwapScreens,
    NextLayout,
    Stylus,
    Touch,
}

impl Input {
    pub const COUNT: usize = 22;

    pub const ALL: [Input; Input::COUNT] = [
        Input::Up,
        Input::Down,
        Input::Left,
        Input::Right,
        Input::A,
        Input::B,
        Input::X,
        Input::Y,
        Input::L,
        Input::R,
        Input::Select,
        Input::Start,
        Input::TurboA,
        Input::TurboB,
        Input::Y1,
        Input::Y2,
        Input::Y3,
        Input::Y4,
        Input::SwapScreens,
        Input::NextLayout,
        Input::Stylus,
        Input::Touch,
    ];

    fn index(self) -> usize {
        self as usize
    }

    pub fn key(self) -> &'static str {
        match self {
            Input::Up => "up",
            Input::Down => "down",
            Input::Left => "left",
            Input::Right => "right",
            Input::A => "a",
            Input::B => "b",
            Input::X => "x",
            Input::Y => "y",
            Input::L => "l",
            Input::R => "r",
            Input::Select => "select",
            Input::Start => "start",
            Input::TurboA => "turbo_a",
            Input::TurboB => "turbo_b",
            Input::Y1 => "y1",
            Input::Y2 => "y2",
            Input::Y3 => "y3",
            Input::Y4 => "y4",
            Input::SwapScreens => "swap_screens",
            Input::NextLayout => "next_layout",
            Input::Stylus => "stylus",
            Input::Touch => "touch",
        }
    }

    /// Whether the input drives slot rather than the core (the DS's screens and stylus).
    pub fn is_slots(self) -> bool {
        matches!(
            self,
            Input::SwapScreens | Input::NextLayout | Input::Stylus | Input::Touch
        )
    }

    fn parse(key: &str) -> Option<Input> {
        Input::ALL
            .into_iter()
            .find(|i| i.key().eq_ignore_ascii_case(key.trim()))
    }

    /// The upstream game button this input was, for seeding from `controls.txt`.
    fn upstream(self) -> Option<GameButton> {
        Some(match self {
            Input::Up => GameButton::Up,
            Input::Down => GameButton::Down,
            Input::Left => GameButton::Left,
            Input::Right => GameButton::Right,
            Input::A => GameButton::A,
            Input::B => GameButton::B,
            Input::Select => GameButton::Select,
            Input::Start => GameButton::Start,
            Input::L => GameButton::L,
            Input::R => GameButton::R,
            // The DS's X and Y were the Turbo A and B rows' buttons.
            Input::TurboA | Input::X => GameButton::TurboA,
            Input::TurboB | Input::Y => GameButton::TurboB,
            Input::Y1
            | Input::Y2
            | Input::Y3
            | Input::Y4
            | Input::SwapScreens
            | Input::NextLayout
            | Input::Stylus
            | Input::Touch => return None,
        })
    }
}

/// The inputs a system has.
pub fn inputs(platform: Platform) -> &'static [Input] {
    use Input::*;
    match platform {
        Platform::Gb | Platform::Gbc | Platform::Tg16 => {
            &[Up, Down, Left, Right, A, B, Select, Start, TurboA, TurboB]
        }
        // Select resets the Game Gear, and the Pocket has none.
        Platform::Gg | Platform::Ngp => &[Up, Down, Left, Right, A, B, Start, TurboA, TurboB],
        Platform::Gba | Platform::Gbav | Platform::Lynx => &[
            Up, Down, Left, Right, A, B, Select, Start, L, R, TurboA, TurboB,
        ],
        Platform::Ws => &[
            Up, Down, Left, Right, Y1, Y2, Y3, Y4, A, B, Start, TurboA, TurboB,
        ],
        Platform::Nds => &[
            Up,
            Down,
            Left,
            Right,
            A,
            B,
            X,
            Y,
            L,
            R,
            Select,
            Start,
            SwapScreens,
            NextLayout,
            Stylus,
            Touch,
        ],
    }
}

/// The standard input whose core button a system's input drives. Pages list inputs in
/// one order for every system, so where a system's own names would read out of order
/// there, its rows drive other buttons: the Game Gear's Button 1 and the Pocket's A are
/// the core's B, and the WonderSwan's X1-X4 run round its pad (up, right, down, left).
pub fn drives(platform: Platform, input: Input) -> Input {
    match (platform, input) {
        (Platform::Gg | Platform::Ngp, Input::A) => Input::B,
        (Platform::Gg | Platform::Ngp, Input::B) => Input::A,
        (Platform::Gg | Platform::Ngp, Input::TurboA) => Input::TurboB,
        (Platform::Gg | Platform::Ngp, Input::TurboB) => Input::TurboA,
        (Platform::Ws, Input::Down) => Input::Right,
        (Platform::Ws, Input::Left) => Input::Down,
        (Platform::Ws, Input::Right) => Input::Left,
        (_, input) => input,
    }
}

/// The file a system's controls live in, under `Config/controls/`.
pub fn system_key(platform: Platform) -> &'static str {
    match platform {
        Platform::Gb => "gb",
        Platform::Gbc => "gbc",
        Platform::Gba | Platform::Gbav => "gba",
        Platform::Ws => "ws",
        Platform::Lynx => "lynx",
        Platform::Gg => "gg",
        Platform::Ngp => "ngp",
        Platform::Tg16 => "tg16",
        Platform::Nds => "nds",
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct SystemControls {
    rows: [Row; Input::COUNT],
}

impl SystemControls {
    /// What a system does out of the box: today's controls, with the left stick moving too.
    pub fn default_for(platform: Platform) -> SystemControls {
        let mut c = SystemControls {
            rows: [[None; SLOTS]; Input::COUNT],
        };
        for &input in inputs(platform) {
            c.rows[input.index()] = default_row(platform, input);
        }
        c
    }

    pub fn row(&self, input: Input) -> Row {
        self.rows[input.index()]
    }

    /// The bindings that are set, oldest first.
    pub fn bindings(&self, input: Input) -> impl Iterator<Item = Binding> + '_ {
        self.rows[input.index()].into_iter().flatten()
    }

    pub fn set_row(&mut self, input: Input, row: Row) {
        let mut packed = [None; SLOTS];
        for (to, b) in packed.iter_mut().zip(row.into_iter().flatten()) {
            *to = Some(b);
        }
        self.rows[input.index()] = packed;
    }

    /// Adds a binding, replacing the oldest when the row is full. A binding the row
    /// already has moves to the newest place instead of appearing twice.
    pub fn add(&mut self, input: Input, binding: Binding) {
        let mut kept: Vec<Binding> = self.bindings(input).filter(|b| *b != binding).collect();
        if kept.len() == SLOTS {
            kept.remove(0);
        }
        kept.push(binding);
        let mut row = [None; SLOTS];
        for (to, b) in row.iter_mut().zip(kept) {
            *to = Some(b);
        }
        self.rows[input.index()] = row;
    }

    pub fn remove_last(&mut self, input: Input) {
        let row = &mut self.rows[input.index()];
        if let Some(last) = row.iter_mut().rev().find(|b| b.is_some()) {
            *last = None;
        }
    }

    pub fn reset(&mut self, platform: Platform, input: Input) {
        self.rows[input.index()] = default_row(platform, input);
    }

    fn text(&self, platform: Platform) -> String {
        inputs(platform)
            .iter()
            .map(|&input| {
                let row = self.rows[input.index()];
                let values: Vec<String> = row
                    .into_iter()
                    .flatten()
                    .map(|b| Binding::key(Some(b)))
                    .collect();
                let value = match values.is_empty() {
                    true => "none".to_string(),
                    false => values.join(","),
                };
                format!("{}={value}\n", input.key())
            })
            .collect()
    }

    fn parse(platform: Platform, text: &str, mut base: SystemControls) -> SystemControls {
        for (key, value) in text.lines().filter_map(|l| l.split_once('=')) {
            let Some(input) = Input::parse(key).filter(|i| inputs(platform).contains(i)) else {
                continue;
            };
            let mut row = [None; SLOTS];
            let mut at = 0;
            for part in value.split(',') {
                match Binding::parse(part) {
                    Some(Some(b)) if at < SLOTS => {
                        row[at] = Some(b);
                        at += 1;
                    }
                    Some(_) => {}
                    None => {}
                }
            }
            base.rows[input.index()] = row;
        }
        base
    }
}

fn default_row(platform: Platform, input: Input) -> Row {
    let plain = |d| Binding::plain(d).expect("a plain binding");
    let chord = |m, d| Binding::pair(m, d).expect("a chord");
    let one = |b| [Some(b), None, None];
    let two = |a, b| [Some(a), Some(b), None];
    match drives(platform, input) {
        // Movement: the D-pad and the left stick.
        Input::Up => two(plain(Device::Up), plain(Device::LsUp)),
        Input::Down => two(plain(Device::Down), plain(Device::LsDown)),
        Input::Left => two(plain(Device::Left), plain(Device::LsLeft)),
        Input::Right => two(plain(Device::Right), plain(Device::LsRight)),
        Input::A => one(plain(Device::A)),
        Input::B => one(plain(Device::B)),
        Input::X => one(plain(Device::X)),
        Input::Y => one(plain(Device::Y)),
        Input::L => one(plain(Device::L1)),
        Input::R => one(plain(Device::R1)),
        Input::Select => one(plain(Device::Select)),
        Input::Start => one(plain(Device::Start)),
        Input::TurboA => one(plain(Device::X)),
        Input::TurboB => one(plain(Device::Y)),
        // The WonderSwan's second pad, as slotPlus placed it on the RG SP: hold L1 and
        // the D-pad drives it, or hold R1 and the face diamond (X up, A right, B down,
        // Y left) does.
        Input::Y1 => two(chord(Device::L1, Device::Up), chord(Device::R1, Device::X)),
        Input::Y2 => two(
            chord(Device::L1, Device::Right),
            chord(Device::R1, Device::A),
        ),
        Input::Y3 => two(
            chord(Device::L1, Device::Down),
            chord(Device::R1, Device::B),
        ),
        Input::Y4 => two(
            chord(Device::L1, Device::Left),
            chord(Device::R1, Device::Y),
        ),
        // The DS's controls, where slot has always had them.
        Input::SwapScreens => one(plain(Device::L2)),
        Input::NextLayout => one(chord(Device::Select, Device::L2)),
        Input::Stylus => one(plain(Device::R2)),
        Input::Touch => one(plain(Device::A)),
    }
    .map(|b: Option<Binding>| b.filter(|_| inputs(platform).contains(&input)))
}

fn file(root: &Path, platform: Platform) -> PathBuf {
    root.join("Config")
        .join(DIR)
        .join(format!("{}.txt", system_key(platform)))
}

/// A system's controls: its own file if it has one, else its defaults with whatever the
/// user had changed in upstream's shared `controls.txt`.
pub fn read_system_controls(root: &Path, platform: Platform) -> SystemControls {
    let seeded = seeded(platform, &read_controls(root));
    match std::fs::read_to_string(file(root, platform)) {
        Ok(text) => SystemControls::parse(platform, &text, seeded),
        Err(_) => seeded,
    }
}

pub fn write_system_controls(
    root: &Path,
    platform: Platform,
    c: &SystemControls,
) -> std::io::Result<()> {
    let path = file(root, platform);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    crate::atomic::atomic_write(&path, c.text(platform).as_bytes())
}

/// The defaults, with each input the user had moved in upstream's controls bound to just
/// that button instead.
fn seeded(platform: Platform, upstream: &Controls) -> SystemControls {
    let mut c = SystemControls::default_for(platform);
    let stock = Controls::default();
    for &input in inputs(platform) {
        let Some(g) = drives(platform, input).upstream() else {
            continue;
        };
        if upstream.game(g) != stock.game(g) {
            c.rows[input.index()] = [upstream.game(g), None, None];
        }
    }
    c
}

/// Whether a binding uses a modifier that's another button (a chord, such as L1+Up).
pub fn is_chord(binding: Binding) -> bool {
    matches!(binding.modifier, Some(Modifier::Button(_)))
}

/// What an input is called on a system's page, after that system's own buttons (as
/// slotPlus's SYSTEMS.md lays them out).
pub fn label(platform: Platform, input: Input) -> &'static str {
    match (platform, input) {
        (Platform::Ws, Input::Up) => "X1",
        (Platform::Ws, Input::Down) => "X2",
        (Platform::Ws, Input::Left) => "X3",
        (Platform::Ws, Input::Right) => "X4",
        (Platform::Gg, Input::A) => "Button 1",
        (Platform::Gg, Input::B) => "Button 2",
        (Platform::Gg, Input::TurboA) => "Turbo 1",
        (Platform::Gg, Input::TurboB) => "Turbo 2",
        (Platform::Ngp, Input::Start) => "Option",
        (Platform::Tg16, Input::B) => "Button II",
        (Platform::Tg16, Input::A) => "Button I",
        (Platform::Tg16, Input::Start) => "Run",
        (Platform::Tg16, Input::TurboB) => "Turbo II",
        (Platform::Tg16, Input::TurboA) => "Turbo I",
        (Platform::Lynx, Input::L) => "Option 1",
        (Platform::Lynx, Input::R) => "Option 2",
        (Platform::Lynx, Input::Start) => "Pause",
        (_, Input::Up) => "Up",
        (_, Input::Down) => "Down",
        (_, Input::Left) => "Left",
        (_, Input::Right) => "Right",
        (_, Input::A) => "A Button",
        (_, Input::B) => "B Button",
        (_, Input::X) => "X Button",
        (_, Input::Y) => "Y Button",
        (_, Input::L) => "L Button",
        (_, Input::R) => "R Button",
        (_, Input::Select) => "Select",
        (_, Input::Start) => "Start",
        (_, Input::TurboA) => "Turbo A",
        (_, Input::TurboB) => "Turbo B",
        (_, Input::Y1) => "Y1",
        (_, Input::Y2) => "Y2",
        (_, Input::Y3) => "Y3",
        (_, Input::Y4) => "Y4",
        (_, Input::SwapScreens) => "Swap Screens",
        (_, Input::NextLayout) => "Next Layout",
        (_, Input::Stylus) => "Stylus",
        (_, Input::Touch) => "Touch",
    }
}

/// The systems with a controls page, in the order the page lists them.
pub const SYSTEMS: [Platform; 9] = [
    Platform::Gb,
    Platform::Gbc,
    Platform::Gba,
    Platform::Ws,
    Platform::Lynx,
    Platform::Gg,
    Platform::Ngp,
    Platform::Tg16,
    Platform::Nds,
];

/// Every system's controls (`Gbav` shares the GBA's).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct AllControls {
    sets: [SystemControls; SYSTEMS.len()],
}

impl Default for AllControls {
    fn default() -> Self {
        AllControls {
            sets: SYSTEMS.map(SystemControls::default_for),
        }
    }
}

fn at(platform: Platform) -> usize {
    let platform = match platform {
        Platform::Gbav => Platform::Gba,
        p => p,
    };
    SYSTEMS.iter().position(|p| *p == platform).unwrap_or(0)
}

impl AllControls {
    pub fn read(root: &Path) -> AllControls {
        AllControls {
            sets: SYSTEMS.map(|p| read_system_controls(root, p)),
        }
    }

    pub fn get(&self, platform: Platform) -> &SystemControls {
        &self.sets[at(platform)]
    }

    pub fn get_mut(&mut self, platform: Platform) -> &mut SystemControls {
        &mut self.sets[at(platform)]
    }

    pub fn write(&self, root: &Path, platform: Platform) -> std::io::Result<()> {
        write_system_controls(root, platform, self.get(platform))
    }
}
