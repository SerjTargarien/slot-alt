use crate::bezel::framed;
use crate::surface::{OUT_H, OUT_W};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Canvas {
    pub w: f32,
    pub h: f32,
}

impl Canvas {
    pub const FRAMED: Canvas = Canvas {
        w: OUT_W as f32,
        h: OUT_H as f32,
    };

    /// The layout canvas for a panel, in layout units. The 3:2 band is 720x480. A 4:3
    /// panel keeps the band's height and narrows to 640x480. A panel taller than 4:3 keeps
    /// the band's width instead and grows taller, so a square panel gets 720x720 and its
    /// UI is drawn 1:1 at the size it has on the band.
    pub fn of(panel: (u32, u32)) -> Canvas {
        if !framed(panel) {
            return Canvas::FRAMED;
        }
        match panel.0 * 3 < panel.1 * 4 {
            true => Canvas {
                w: OUT_W as f32,
                h: (OUT_W * panel.1 / panel.0) as f32,
            },
            false => Canvas {
                w: (OUT_H * panel.0 / panel.1) as f32,
                h: OUT_H as f32,
            },
        }
    }

    pub fn centred(self, w: f32) -> f32 {
        (self.w - w) / 2.0
    }

    /// How much taller than the 480-unit band the canvas is: 0 except on panels taller
    /// than 4:3.
    pub fn extra(self) -> f32 {
        (self.h - OUT_H as f32).max(0.0)
    }

    /// A band y for something anchored to the bottom of the screen (the slot, the footer),
    /// moved to the bottom of a taller canvas.
    pub fn below(self, y: f32) -> f32 {
        y + self.extra()
    }

    /// A band y for something centred on the screen (toasts, the shelf), moved to the
    /// middle of a taller canvas.
    pub fn mid(self, y: f32) -> f32 {
        y + self.extra() / 2.0
    }

    pub fn window(self, tex_w: f32) -> [f32; 4] {
        if tex_w <= self.w {
            return [0.0, 0.0, 1.0, 1.0];
        }
        [(tex_w - self.w) / 2.0 / tex_w, 0.0, self.w / tex_w, 1.0]
    }

    /// The part of a band-high texture (`tex_w` x 480) that covers the whole canvas at
    /// its aspect ratio, centred. On the band this crops the sides of a wide texture, or
    /// the top and bottom of a narrow one; a taller canvas crops more of the sides.
    pub fn cover(self, tex_w: f32) -> [f32; 4] {
        let tex_h = OUT_H as f32;
        if self.h <= tex_h {
            if tex_w >= self.w {
                return self.window(tex_w);
            }
            let shown = tex_w / self.w;
            return [0.0, (1.0 - shown) / 2.0, 1.0, shown];
        }
        let scale = (self.w / tex_w).max(self.h / tex_h);
        let (shown_w, shown_h) = (self.w / scale / tex_w, self.h / scale / tex_h);
        [
            (1.0 - shown_w) / 2.0,
            (1.0 - shown_h) / 2.0,
            shown_w,
            shown_h,
        ]
    }
}
