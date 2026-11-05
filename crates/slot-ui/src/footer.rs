use slot_gfx::{Canvas, Draw, TexId, OUT_H};
use slot_power::{Battery, Charge};

use crate::battery::{draw_gauge, BOLT_LEAD, GAUGE_H, GAUGE_ROW_W};
use crate::hud::{HUD_INK, PLATE_H};
use crate::plate::HINT_H;
use crate::slot_chrome::MOUTH_H;

pub const FOOTER_Y: f32 = OUT_H as f32 - MOUTH_H + (MOUTH_H - HINT_H as f32) / 2.0;
const FOOTER_MARGIN: f32 = 24.0;
const FOOTER_GAP: f32 = 8.0;
const CORNER_MARGIN: f32 = 16.0;

#[derive(Copy, Clone, Default, PartialEq, Eq, Debug)]
pub struct Printed {
    pub face: Option<TexId>,
    pub w: u32,
}

impl Printed {
    pub fn new(face: TexId, w: u32) -> Self {
        Printed {
            face: Some(face),
            w,
        }
    }
}

pub fn draw_footer(
    canvas: Canvas,
    battery: Option<Battery>,
    percent: Printed,
    bolt: Option<TexId>,
    clock: Printed,
    ink: [u8; 3],
    out: &mut Vec<Draw>,
) {
    let y = canvas.below(FOOTER_Y) + (HINT_H as f32 - GAUGE_H) / 2.0;
    let charging = matches!(battery, Some(b) if b.charge == Charge::Charging) && bolt.is_some();
    let lead = if charging { 0.0 } else { BOLT_LEAD };
    let x = left_x(canvas, GAUGE_ROW_W + percent.w as f32, lead);
    draw_gauge(x, y, battery, percent, bolt, ink, out);
    printed(
        canvas,
        canvas.w - left_x(canvas, clock.w as f32, 0.0) - clock.w as f32,
        clock,
        out,
    );
}

/// Where a footer readout of width `w` starts from the left (or, mirrored, ends from the
/// right), `lead` of it being blank space at its start. It's the usual margin when the
/// readout fits beside the slot's bay. On canvases too narrow for that, such as a square
/// panel's, what's visible of it is centred in the space beside the bay instead.
fn left_x(canvas: Canvas, w: f32, lead: f32) -> f32 {
    let room = crate::slot_chrome::bay_x(canvas);
    match FOOTER_MARGIN + w + FOOTER_GAP <= room {
        true => FOOTER_MARGIN,
        false => ((room - w - lead) / 2.0).max(-lead).round(),
    }
}

const WIFI_GAP: f32 = 12.0;

pub fn draw_wifi_icon(
    canvas: Canvas,
    percent: Printed,
    face: Option<(TexId, u32, u32)>,
    out: &mut Vec<Draw>,
) {
    let Some((tex, w, h)) = face else {
        return;
    };
    let row = GAUGE_ROW_W + percent.w as f32;
    let x = left_x(canvas, row, BOLT_LEAD) + row + WIFI_GAP;
    let y = canvas.below(FOOTER_Y) + (HINT_H as f32 - h as f32) / 2.0;
    out.push(Draw::Tex {
        x: x.round(),
        y: y.round(),
        w: w as f32,
        h: h as f32,
        tex,
        alpha: 1.0,
    });
}

pub fn draw_corners(
    canvas: Canvas,
    battery: Option<Battery>,
    percent: Printed,
    bolt: Option<TexId>,
    clock: Printed,
    out: &mut Vec<Draw>,
) {
    let y = (PLATE_H - GAUGE_H) / 2.0;
    draw_gauge(CORNER_MARGIN, y, battery, percent, bolt, HUD_INK, out);
    let x = canvas.w - CORNER_MARGIN - clock.w as f32;
    draw_printed(x, (PLATE_H - HINT_H as f32) / 2.0, clock, out);
}

pub(crate) fn draw_printed(x: f32, y: f32, p: Printed, out: &mut Vec<Draw>) {
    if p.w == 0 {
        return;
    }
    let (w, h) = (p.w as f32, HINT_H as f32);
    out.push(match p.face {
        Some(tex) => Draw::Tex {
            x,
            y,
            w,
            h,
            tex,
            alpha: 1.0,
        },
        None => Draw::Rect {
            x,
            y,
            w,
            h,
            colour: [1.0, 1.0, 1.0, 0.08],
        },
    });
}

fn printed(canvas: Canvas, x: f32, p: Printed, out: &mut Vec<Draw>) {
    draw_printed(x, canvas.below(FOOTER_Y), p, out);
}
