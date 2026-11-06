use slot_input::{Action, Btn};
use slot_retro::ButtonMask;
use slot_store::controls::{drives, inputs, Input, SystemControls};
use slot_store::{Binding, Device, Modifier, Platform};

use super::bind::device_btn;

#[derive(Default)]
pub struct Pad {
    held: Vec<Btn>,
    chorded: Vec<Btn>,
}

impl Pad {
    pub fn apply(&mut self, action: Action) {
        let (list, b, down) = match action {
            Action::GbaDown(b) => (&mut self.held, b, true),
            Action::GbaUp(b) => (&mut self.held, b, false),
            Action::MenuChordDown(b) => (&mut self.chorded, b, true),
            Action::MenuChordUp(b) => (&mut self.chorded, b, false),
            _ => return,
        };
        list.retain(|x| *x != b);
        if down {
            list.push(b);
        }
    }

    /// The core buttons held, from a system's controls. An input is pressed when any of
    /// its bindings is held. A held chord (L1+Up) wins over its buttons' own bindings, so
    /// holding L1 and Up presses what L1+Up is bound to and not Up's input too.
    pub fn mask(&self, controls: &SystemControls, platform: Platform) -> ButtonMask {
        let held = |d: Device| self.held.contains(&device_btn(d));
        let chord_held = |b: &Binding| match b.modifier {
            Some(Modifier::Button(m)) => held(m) && held(b.button),
            _ => false,
        };
        let mut taken: Vec<Device> = Vec::new();
        for &input in inputs(platform) {
            for b in controls.bindings(input).filter(|b| chord_held(b)) {
                if let Some(Modifier::Button(m)) = b.modifier {
                    taken.extend([m, b.button]);
                }
            }
        }
        let active = |b: Binding| match b.modifier {
            None => held(b.button) && !taken.contains(&b.button),
            Some(Modifier::Menu) => self.chorded.contains(&device_btn(b.button)),
            Some(Modifier::Button(_)) => chord_held(&b),
            Some(Modifier::Select) => false,
        };
        let bits = inputs(platform)
            .iter()
            .filter(|&&input| controls.bindings(input).any(|b| active(b)))
            .fold(0, |bits, &input| bits | bit(drives(platform, input)));
        ButtonMask(bits)
    }

    /// The core buttons held, each physical button as itself, for what still reads the
    /// buttons directly (a WonderSwan game played upright).
    pub fn raw(&self) -> ButtonMask {
        let pairs = [
            (Btn::Up, ButtonMask::UP),
            (Btn::Down, ButtonMask::DOWN),
            (Btn::Left, ButtonMask::LEFT),
            (Btn::Right, ButtonMask::RIGHT),
            (Btn::A, ButtonMask::A),
            (Btn::B, ButtonMask::B),
            (Btn::X, ButtonMask::X),
            (Btn::Y, ButtonMask::Y),
            (Btn::L1, ButtonMask::L),
            (Btn::R1, ButtonMask::R),
            (Btn::L2, ButtonMask::L2),
            (Btn::R2, ButtonMask::R2),
            (Btn::Select, ButtonMask::SELECT),
            (Btn::Start, ButtonMask::START),
        ];
        let bits = pairs
            .iter()
            .filter(|(b, _)| self.held.contains(b))
            .fold(0, |bits, (_, bit)| bits | bit);
        ButtonMask(bits)
    }

    pub fn clear(&mut self) {
        self.held.clear();
        self.chorded.clear();
    }
}

/// The core button each input drives. Turbo A and B ride on X and Y, which the
/// emulator turns into auto-fire; the WonderSwan's second pad is R2, R, L2 and L to Cygne.
fn bit(input: Input) -> u16 {
    match input {
        Input::Up => ButtonMask::UP,
        Input::Down => ButtonMask::DOWN,
        Input::Left => ButtonMask::LEFT,
        Input::Right => ButtonMask::RIGHT,
        Input::A => ButtonMask::A,
        Input::B => ButtonMask::B,
        Input::X | Input::TurboA => ButtonMask::X,
        Input::Y | Input::TurboB => ButtonMask::Y,
        Input::L => ButtonMask::L,
        Input::R => ButtonMask::R,
        Input::Select => ButtonMask::SELECT,
        Input::Start => ButtonMask::START,
        Input::Y1 => ButtonMask::R2,
        Input::Y2 => ButtonMask::R,
        Input::Y3 => ButtonMask::L2,
        Input::Y4 => ButtonMask::L,
    }
}
