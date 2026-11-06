use slot_input::{Action, Btn};
use slot_retro::ButtonMask;
use slot_store::controls::{drives, inputs, Input, SystemControls};
use slot_store::{Binding, Device, Modifier, Platform};

use super::bind::device_btn;

#[derive(Default)]
pub struct Pad {
    held: Vec<Btn>,
    chorded: Vec<Btn>,
    select_chorded: Vec<Btn>,
}

impl Pad {
    pub fn apply(&mut self, action: Action) {
        let (list, b, down) = match action {
            Action::GbaDown(b) => (&mut self.held, b, true),
            Action::GbaUp(b) => (&mut self.held, b, false),
            Action::MenuChordDown(b) => (&mut self.chorded, b, true),
            Action::MenuChordUp(b) => (&mut self.chorded, b, false),
            Action::SelectChordDown(b) => (&mut self.select_chorded, b, true),
            Action::SelectChordUp(b) => (&mut self.select_chorded, b, false),
            _ => return,
        };
        list.retain(|x| *x != b);
        if down {
            list.push(b);
        }
    }

    /// The inputs pressed now, from a system's controls. An input is pressed when any of
    /// its bindings is held. A held chord (L1+Up) wins over its buttons' own bindings, so
    /// holding L1 and Up presses what L1+Up is bound to and not Up's input too. While the
    /// DS's stylus is out, Touch's buttons press only Touch.
    pub fn pressed(
        &self,
        controls: &SystemControls,
        platform: Platform,
        stylus_out: bool,
    ) -> Vec<Input> {
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
        let touching: Vec<Device> = match stylus_out {
            true => controls.bindings(Input::Touch).map(|b| b.button).collect(),
            false => Vec::new(),
        };
        let active = |input: Input, b: Binding| match b.modifier {
            None => {
                held(b.button)
                    && !taken.contains(&b.button)
                    && (input == Input::Touch || !touching.contains(&b.button))
            }
            Some(Modifier::Menu) => self.chorded.contains(&device_btn(b.button)),
            Some(Modifier::Button(_)) => chord_held(&b),
            // The gesture layer sends a SELECT chord as itself (see the binding table).
            Some(Modifier::Select) => self.select_chorded.contains(&device_btn(b.button)),
        };
        inputs(platform)
            .iter()
            .copied()
            .filter(|&input| controls.bindings(input).any(|b| active(input, b)))
            .collect()
    }

    /// The core buttons held: the pressed inputs that drive the core. While the DS's
    /// Stylus Hold is held, the D-pad's inputs move the stylus instead.
    pub fn mask(
        &self,
        controls: &SystemControls,
        platform: Platform,
        stylus_out: bool,
    ) -> ButtonMask {
        let pressed = self.pressed(controls, platform, stylus_out);
        let steering = pressed.contains(&Input::Stylus);
        let moving = |i: Input| {
            steering && matches!(i, Input::Up | Input::Down | Input::Left | Input::Right)
        };
        let bits = pressed
            .into_iter()
            .filter(|&i| !i.is_slots() && !moving(i))
            .fold(0, |bits, i| bits | bit(drives(platform, i)));
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
        self.select_chorded.clear();
    }
}

/// The core button each input drives. The WonderSwan's second pad is R2, R, L2 and L to
/// Cygne.
fn bit(input: Input) -> u16 {
    match input {
        Input::Up => ButtonMask::UP,
        Input::Down => ButtonMask::DOWN,
        Input::Left => ButtonMask::LEFT,
        Input::Right => ButtonMask::RIGHT,
        Input::A => ButtonMask::A,
        Input::B => ButtonMask::B,
        Input::X => ButtonMask::X,
        Input::Y => ButtonMask::Y,
        Input::L => ButtonMask::L,
        Input::R => ButtonMask::R,
        Input::Select => ButtonMask::SELECT,
        Input::Start => ButtonMask::START,
        Input::Y1 => ButtonMask::R2,
        Input::Y2 => ButtonMask::R,
        Input::Y3 => ButtonMask::L2,
        Input::Y4 => ButtonMask::L,
        Input::SwapScreens
        | Input::NextLayout
        | Input::Stylus
        | Input::Touch
        | Input::StylusUp
        | Input::StylusDown
        | Input::StylusLeft
        | Input::StylusRight => 0,
    }
}
