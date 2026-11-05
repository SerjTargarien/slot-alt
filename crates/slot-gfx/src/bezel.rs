use crate::surface::{OUT_H, OUT_W};

/// Whether the panel is laid out "framed": drawn over the whole panel through a buffer
/// of its own size, with games free to use the whole panel. That's 4:3 panels such as
/// 640x480, and square or portrait ones such as the RGB20SX's 720x720 (see Canvas::of for
/// their layout canvas). Other shapes get the 3:2 band, fitted. SLOT_TALL_PANELS=0 puts
/// square and portrait panels back on the band, letterboxed, to compare the two.
pub fn framed(panel: (u32, u32)) -> bool {
    match panel.0 * 3 == panel.1 * 4 {
        true => panel.0 > 0,
        false => panel.0 > 0 && panel.0 <= panel.1 && tall_panels(),
    }
}

fn tall_panels() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("SLOT_TALL_PANELS").is_none_or(|v| v != "0"))
}

pub fn screen_rect(panel: (u32, u32)) -> (i32, i32, i32, i32) {
    let w = panel.0;
    let h = (w * OUT_H + OUT_W / 2) / OUT_W;
    ((panel.0 - w) as i32 / 2, 0, w as i32, h as i32)
}

pub fn game_rect(panel: (u32, u32), sp_frame: bool) -> (i32, i32, i32, i32) {
    let (x, _, w, h) = screen_rect(panel);
    let gap = panel.1 as i32 - h;
    let y = match sp_frame {
        true => 0,
        false => gap - gap / 2,
    };
    (x, y, w, h)
}

pub fn bezel_column(rgba: &[u8], panel: (u32, u32)) -> Vec<u8> {
    let (w, h) = (panel.0 as usize, panel.1 as usize);
    if !framed(panel) || rgba.len() < w * h * 4 {
        return Vec::new();
    }
    (0..h).map(|y| rgba[(y * w + w / 2) * 4 + 3]).collect()
}

pub fn hole_on_top(column: &[u8], panel: (u32, u32), picture: (f32, f32)) -> bool {
    let h = panel.1 as usize;
    if !framed(panel) || column.len() != h {
        return false;
    }
    let game_h = screen_rect(panel).3 as f32;
    let lift = (h - game_h as usize) - (h - game_h as usize) / 2;
    let bottom = lift + ((picture.0 + picture.1) * game_h) as usize;
    (bottom - lift + 1..bottom)
        .chain([h - 1])
        .all(|y| column[y] >= 128)
}

pub fn lifted_rect(panel: (u32, u32), lift: f32) -> (i32, i32, i32, i32) {
    let (x, _, w, h) = screen_rect(panel);
    let t = lift.clamp(0.0, 1.0);
    let eased = t * t * (3.0 - 2.0 * t);
    let drop = ((panel.1 as i32 - h) as f32 * (1.0 - eased)).round() as i32;
    (x, drop, w, h)
}
