//! The DS's stylus, for a handheld with no touch screen: while its Stylus input is held
//! (R2 by default) the D-pad's inputs move a cursor over the DS's bottom screen, and its
//! Touch input (A) touches it.

use slot_retro::Pointer;

/// The DS's bottom screen, which the stylus touches.
pub const SCREEN_W: f32 = 256.0;
pub const SCREEN_H: f32 = 192.0;

/// How fast the cursor moves, in screen pixels a second: it starts slow
/// enough to place on a single button and speeds up while held.
const SLOW: f32 = 60.0;
const FAST: f32 = 240.0;
/// Seconds held before it reaches full speed.
const RAMP: f32 = 0.6;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stylus {
    pub x: f32,
    pub y: f32,
    pub touching: bool,
    held: [bool; 4],
    held_for: f32,
}

impl Stylus {
    /// The stylus out at the bottom screen's `at`, where it was last left.
    pub fn at(at: (f32, f32)) -> Stylus {
        Stylus {
            x: at.0,
            y: at.1,
            touching: false,
            held: [false; 4],
            held_for: 0.0,
        }
    }

    /// Which ways it is being moved (up, down, left, right) and whether it touches.
    pub fn steer(&mut self, held: [bool; 4], touching: bool) {
        self.held = held;
        self.touching = touching;
        if !held.iter().any(|&h| h) {
            self.held_for = 0.0;
        }
    }

    pub fn tick(&mut self, dt: f32) {
        let [up, down, left, right] = self.held;
        let dx = f32::from(i8::from(right) - i8::from(left));
        let dy = f32::from(i8::from(down) - i8::from(up));
        if dx == 0.0 && dy == 0.0 {
            return;
        }
        self.held_for += dt;
        let speed = SLOW + (FAST - SLOW) * (self.held_for / RAMP).min(1.0);
        self.x = (self.x + dx * speed * dt).clamp(0.0, SCREEN_W - 1.0);
        self.y = (self.y + dy * speed * dt).clamp(0.0, SCREEN_H - 1.0);
    }

    /// The pointer the DS core reads. Its frame is the two screens stacked,
    /// and libretro's pointer runs -0x7fff to 0x7fff over the whole frame,
    /// so the bottom screen is 0 to 0x7fff down.
    pub fn pointer(&self) -> Pointer {
        let across = self.x / SCREEN_W * 65534.0 - 32767.0;
        let down = self.y / SCREEN_H * 32767.0;
        Pointer {
            x: across.round() as i16,
            y: down.round() as i16,
            pressed: self.touching,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pointer_lands_where_dsperate_reads_the_bottom_screen() {
        // DSperate: x = (px + 0x7fff) * 256 / 0xffff, y = py * 192 / 0x8000.
        for (x, y) in [(0.0, 0.0), (128.0, 96.0), (255.0, 191.0), (17.0, 150.0)] {
            let p = Stylus::at((x, y)).pointer();
            let px = (i32::from(p.x) + 0x7fff) * 256 / 0xffff;
            let py = i32::from(p.y) * 192 / 0x8000;
            assert!((px - x as i32).abs() <= 1, "{x} read as {px}");
            assert!((py - y as i32).abs() <= 1, "{y} read as {py}");
            assert!(p.y >= 0, "on the bottom screen");
        }
    }

    #[test]
    fn a_held_direction_moves_slowly_then_faster_and_stops_at_the_edge() {
        let mut s = Stylus::at((128.0, 96.0));
        s.steer([false, false, false, true], false);
        s.tick(0.1);
        let first = s.x - 128.0;
        assert!((6.0..10.0).contains(&first), "a slow first step, {first}");
        for _ in 0..10 {
            s.tick(0.1);
        }
        assert_eq!(s.x, SCREEN_W - 1.0, "held against the edge");
        s.steer([false; 4], false);
        let at = s.x;
        s.tick(0.1);
        assert_eq!(s.x, at, "let go, it stays");
    }

    #[test]
    fn touch_presses_the_pointer() {
        let mut s = Stylus::at((10.0, 10.0));
        assert!(!s.pointer().pressed);
        s.steer([false; 4], true);
        assert!(s.pointer().pressed);
    }
}
