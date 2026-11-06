use std::path::Path;

use slot_gfx::{Sizing, SRC_H, SRC_W, WHOLE_TEXTURE};
use slot_store::{ini, Platform, Scale, Stretch};

pub const VIDEO_MODE_FILE: &str = "Config/video_mode.txt";

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum VideoMode {
    #[default]
    Actual,
    Stretch,
}

impl VideoMode {
    pub fn as_str(self) -> &'static str {
        match self {
            VideoMode::Actual => "actual",
            VideoMode::Stretch => "stretch",
        }
    }

    pub fn parse(s: &str) -> Option<VideoMode> {
        match s.trim().to_ascii_lowercase().as_str() {
            "actual" => Some(VideoMode::Actual),
            "stretch" => Some(VideoMode::Stretch),
            _ => None,
        }
    }
}

pub fn video_mode_for(root: &Path, stem: &str) -> VideoMode {
    ini::value(root, VIDEO_MODE_FILE, stem)
        .as_deref()
        .and_then(VideoMode::parse)
        .unwrap_or_default()
}

pub fn write_video_mode(root: &Path, stem: &str, mode: VideoMode) -> std::io::Result<()> {
    ini::write(root, VIDEO_MODE_FILE, stem, mode.as_str())
}

/// Platforms whose stretch widens the picture only, keeping every line and
/// the band above and below it: the Game Boy and Game Boy Color, as Slot
/// 1.7.0 does. Every other platform's stretch fills the screen both ways.
fn widens_only(platform: Platform) -> bool {
    matches!(platform, Platform::Gb | Platform::Gbc)
}

pub fn source_rect(platform: Platform, mode: VideoMode) -> [f32; 4] {
    source_rect_for(platform, mode, false)
}

/// Which part of the platform's video buffer to stretch over the canvas.
/// A vertical frame (`portrait`) always shows the whole buffer, which is
/// sized so it lands at an exact 2x.
pub fn source_rect_for(platform: Platform, mode: VideoMode, portrait: bool) -> [f32; 4] {
    source_rect_shown(platform, mode, portrait, (0, 0))
}

/// The picture a game draws, for a platform whose games change their width:
/// a TurboGrafx-16 game draws 256, 320, 336 or 512 pixels wide, and the frame
/// it last drew (`shown`) says which. Every other platform's picture is fixed.
pub fn picture_of(platform: Platform, shown: (u32, u32)) -> (u32, u32) {
    let (bw, bh) = platform.source_size();
    match platform {
        Platform::Tg16 if shown.0 >= 64 && shown.1 >= 64 => (shown.0.min(bw), shown.1.min(bh)),
        _ => platform.picture(),
    }
}

/// Slot²: the platforms whose picture follows the frame the core last drew: a
/// TurboGrafx-16 game changes its width, and a WonderSwan or Lynx game turns on its side
/// (the host turns the frame, so it arrives taller than wide), even mid-game.
pub fn follows_frame(platform: Platform) -> bool {
    matches!(platform, Platform::Tg16 | Platform::Ws | Platform::Lynx)
}

/// Slot²: such a frame's size on screen and where it sits, centred, in its buffer. A
/// 512-wide TurboGrafx-16 frame has half-width pixels, so it takes the room of a 256-wide
/// one: Integer and Fit size the picture by that, not by its raw width.
pub fn drawn_picture(platform: Platform, shown: (u32, u32)) -> ((u32, u32), [f32; 4]) {
    let (bw, bh) = platform.source_size();
    let (w, h) = match platform {
        Platform::Tg16 => picture_of(platform, shown),
        _ if shown.0 >= 64 && shown.1 >= 64 => (shown.0.min(bw), shown.1.min(bh)),
        _ => platform.picture(),
    };
    let room = match platform == Platform::Tg16 && w > platform.actual_window().0 {
        true => (w / 2, h),
        false => (w, h),
    };
    let (x, y) = (bw.saturating_sub(w) / 2, bh.saturating_sub(h) / 2);
    let (bw, bh) = (bw as f32, bh as f32);
    let crop = [x as f32 / bw, y as f32 / bh, w as f32 / bw, h as f32 / bh];
    (room, crop)
}

/// The part of the buffer an "actual size" picture is drawn into. A
/// TurboGrafx-16 picture up to 360 wide gets the 360x240 window, so it lands
/// at 2x; a wider, high resolution one gets the whole 720x240 buffer, so its
/// narrow pixels are drawn at 1x across and 2x down.
fn actual_window_of(platform: Platform, shown: (u32, u32)) -> (u32, u32) {
    match platform {
        Platform::Tg16 if picture_of(platform, shown).0 > platform.actual_window().0 => {
            platform.source_size()
        }
        _ => platform.actual_window(),
    }
}

/// As `source_rect_for`, given the size of the frame the core last drew.
pub fn source_rect_shown(
    platform: Platform,
    mode: VideoMode,
    portrait: bool,
    shown: (u32, u32),
) -> [f32; 4] {
    let (bw, bh) = platform.source_size();
    if portrait {
        return WHOLE_TEXTURE;
    }
    let (w, h) = match mode {
        VideoMode::Actual => actual_window_of(platform, shown),
        VideoMode::Stretch => picture_of(platform, shown),
    };
    if (w, h) == (bw, bh) {
        return WHOLE_TEXTURE;
    }
    let x = bw.saturating_sub(w) / 2;
    if mode == VideoMode::Stretch && widens_only(platform) {
        // A Game Boy picture is stretched sideways only, as a GBA's is,
        // keeping all 144 lines and the dark band above and below them.
        return [x as f32 / bw as f32, 0.0, w as f32 / bw as f32, 1.0];
    }
    let y = bh.saturating_sub(h) / 2;
    [
        x as f32 / bw as f32,
        y as f32 / bh as f32,
        w as f32 / bw as f32,
        h as f32 / bh as f32,
    ]
}

/// How the game picture is sized on the panel, and which part of the buffer
/// it comes from. The Game Boy family follows Scale and Stretch (Slot's).
/// slotPlus's systems keep their own buffers and their own stretch, from
/// `Config/video_mode.txt`, drawn over the band as before.
pub fn picture_for(
    platform: Platform,
    mode: VideoMode,
    scale: Scale,
    stretch: Stretch,
) -> (Sizing, [f32; 4]) {
    // Slot²: the Game Gear and NeoGeo Pocket share the Game Boy's 240x160 buffer, so they
    // scale (Integer or Fit) and stretch (to the whole screen) the same way.
    if !matches!(
        platform.hardware(),
        Platform::Gba | Platform::Gb | Platform::Gbc | Platform::Gg | Platform::Ngp
    ) {
        return (Sizing::Band, source_rect(platform, mode));
    }
    let (w, h) = platform.picture();
    let x = SRC_W.saturating_sub(w) as f32 / 2.0 / SRC_W as f32;
    let y = SRC_H.saturating_sub(h) as f32 / 2.0 / SRC_H as f32;
    let (fw, fh) = (w as f32 / SRC_W as f32, h as f32 / SRC_H as f32);
    let crop = [x, y, fw, fh];
    let window = [x, 0.0, fw, 1.0];
    if platform.hardware() == Platform::Gba {
        let sizing = match scale {
            Scale::Integer => Sizing::Integer,
            Scale::FullScreen => Sizing::Panel,
            Scale::SpFrame | Scale::Fit => Sizing::Band,
        };
        return (sizing, WHOLE_TEXTURE);
    }
    match (mode, scale, stretch) {
        (VideoMode::Actual, Scale::Integer, _) => (Sizing::Integer, crop),
        (VideoMode::Actual, Scale::Fit, _) => (Sizing::Fit, crop),
        (VideoMode::Actual, _, _) => (Sizing::Band, WHOLE_TEXTURE),
        (VideoMode::Stretch, _, Stretch::Classic) => (Sizing::Band, window),
        (VideoMode::Stretch, _, Stretch::FullScreen) => (Sizing::Panel, crop),
    }
}

/// Which of a DS game's screens are shown, kept per game.
pub const DS_VIEW_FILE: &str = "Config/nds_view.txt";

/// One of the DS's two screens.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DsScreen {
    #[default]
    Top,
    Bottom,
}

impl DsScreen {
    pub fn other(self) -> DsScreen {
        match self {
            DsScreen::Top => DsScreen::Bottom,
            DsScreen::Bottom => DsScreen::Top,
        }
    }

    /// Where the screen sits in the DS's 720x480 buffer, as a fraction of
    /// it: DSperate draws the two stacked, 256x384, at 232, 48.
    pub fn source_rect(self) -> [f32; 4] {
        let y = match self {
            DsScreen::Top => 48.0,
            DsScreen::Bottom => 240.0,
        };
        [232.0 / 720.0, y / 480.0, 256.0 / 720.0, 192.0 / 480.0]
    }
}

/// How a DS game's two screens are laid out. Slot²: each layout is placed on the whole
/// canvas it is drawn on (720x720 on the RGB20SX's square panel), at Integer or Fit.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DsLayout {
    /// Both, the main one above the other.
    #[default]
    Stacked,
    /// The main screen alone. The only layout Stretch fills the screen with.
    Single,
    /// The main screen, with the other at a third of its size centred below it.
    Below,
    /// The main screen, with the other at 1x over its bottom-right corner.
    Corner,
}

/// A DS screen's size in pixels.
const DS_W: f32 = 256.0;
const DS_H: f32 = 192.0;

impl DsLayout {
    pub const ALL: [DsLayout; 4] = [
        DsLayout::Stacked,
        DsLayout::Single,
        DsLayout::Below,
        DsLayout::Corner,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            DsLayout::Stacked => "stacked",
            DsLayout::Single => "single",
            DsLayout::Below => "below",
            DsLayout::Corner => "corner",
        }
    }

    /// The PNG for this layout in a DS bezel set (a folder in `Bezels/NDS/`). The sets are
    /// drawn for the 720x480 band at upstream's sizes, so only those layouts have one.
    pub fn bezel_file(self, fit: bool) -> Option<&'static str> {
        match (self, fit) {
            (DsLayout::Stacked, true) => Some("both-screens.png"),
            (DsLayout::Single, false) => Some("one-screen.png"),
            (DsLayout::Single, true) => Some("full-screen.png"),
            (DsLayout::Corner, true) => Some("picture-in-picture.png"),
            _ => None,
        }
    }

    /// Its place in `DsLayout::ALL`, which slot-ui's `DS_LAYOUTS` names.
    pub fn index(self) -> usize {
        DsLayout::ALL.iter().position(|l| *l == self).unwrap_or(0)
    }

    /// A layout by its name. Upstream's Full Screen is One Screen at Fit, and its Side by
    /// Side, made for the wide band, becomes Main and Small.
    pub fn parse(s: &str) -> Option<DsLayout> {
        match s {
            "fill" => Some(DsLayout::Single),
            "side" => Some(DsLayout::Below),
            _ => DsLayout::ALL.into_iter().find(|l| l.as_str() == s),
        }
    }

    /// The layout SELECT + L2 steps to.
    pub fn next(self) -> DsLayout {
        DsLayout::ALL[(self.index() + 1) % DsLayout::ALL.len()]
    }

    pub fn step(self, right: bool) -> DsLayout {
        let n = DsLayout::ALL.len();
        DsLayout::ALL[(self.index() + if right { 1 } else { n - 1 }) % n]
    }
}

/// One screen drawn: the part of the buffer it is, and where on the canvas it goes, in
/// canvas units.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DsPart {
    pub screen: DsScreen,
    pub src: [f32; 4],
    pub dst: [f32; 4],
}

/// How a DS game's screens are sized on a canvas of `w` by `h`: at Integer or Fit, and
/// whether One Screen stretches over all of it.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DsFrame {
    pub w: f32,
    pub h: f32,
    pub fit: bool,
    pub stretch: bool,
}

impl DsFrame {
    /// The scale a picture of `pw` by `ph` is drawn at: the most that fits, whole at
    /// Integer (never under 1x).
    fn scale(self, pw: f32, ph: f32) -> f32 {
        let s = (self.w / pw).min(self.h / ph);
        match self.fit {
            true => s,
            false => s.floor().max(1.0),
        }
    }

    /// A box of `w` by `h` centred on the canvas, its top-left corner.
    fn centred(self, w: f32, h: f32) -> (f32, f32) {
        (((self.w - w) / 2.0).round(), ((self.h - h) / 2.0).round())
    }
}

/// A DS game's view of its two screens: which is the main one, L2 swaps
/// them, and how they are laid out, which SELECT + L2 steps through.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct DsView {
    pub main: DsScreen,
    pub layout: DsLayout,
}

impl DsView {
    pub fn as_str(self) -> String {
        let main = match self.main {
            DsScreen::Top => "top",
            DsScreen::Bottom => "bottom",
        };
        format!("{},{main}", self.layout.as_str())
    }

    /// Reads a saved view, the views before layouts among them: "both" was
    /// both stacked, "top" and "bottom" one screen at 2x.
    pub fn parse(s: &str) -> Option<DsView> {
        let s = s.trim().to_ascii_lowercase();
        let view = |layout, main| Some(DsView { main, layout });
        match s.as_str() {
            "both" => return view(DsLayout::Stacked, DsScreen::Top),
            "top" => return view(DsLayout::Single, DsScreen::Top),
            "bottom" => return view(DsLayout::Single, DsScreen::Bottom),
            _ => {}
        }
        let (layout, main) = s.split_once(',')?;
        let main = match main {
            "top" => DsScreen::Top,
            "bottom" => DsScreen::Bottom,
            _ => return None,
        };
        view(DsLayout::parse(layout)?, main)
    }

    pub fn swapped(self) -> DsView {
        DsView {
            main: self.main.other(),
            ..self
        }
    }

    pub fn next_layout(self) -> DsView {
        DsView {
            layout: self.layout.next(),
            ..self
        }
    }

    /// The screens drawn on `frame`: the main one, and the other where the layout shows
    /// it too, drawn over the main one.
    pub fn parts(self, frame: DsFrame) -> (DsPart, Option<DsPart>) {
        let part = |screen: DsScreen, dst| DsPart {
            screen,
            src: screen.source_rect(),
            dst,
        };
        let (main, other) = (self.main, self.main.other());
        match self.layout {
            DsLayout::Stacked => {
                let s = frame.scale(DS_W, DS_H * 2.0);
                let (w, h) = ((DS_W * s).round(), (DS_H * s).round());
                let (x, y) = frame.centred(w, h * 2.0);
                (
                    part(main, [x, y, w, h]),
                    Some(part(other, [x, y + h, w, h])),
                )
            }
            DsLayout::Single if frame.stretch => (part(main, [0.0, 0.0, frame.w, frame.h]), None),
            DsLayout::Single => (part(main, single(frame)), None),
            DsLayout::Below => {
                // The pair is as tall as it is wide: 192 for the main one, 64 below.
                let s = frame.scale(DS_W, DS_W);
                let (w, h) = ((DS_W * s).round(), (DS_H * s).round());
                let (x, y) = frame.centred(w, w);
                let (sw, sh) = ((w / 3.0).round(), (h / 3.0).round());
                (
                    part(main, [x, y, w, h]),
                    Some(part(other, [x + ((w - sw) / 2.0).round(), y + h, sw, sh])),
                )
            }
            DsLayout::Corner => {
                let [x, y, w, h] = single(frame);
                (
                    part(main, [x, y, w, h]),
                    Some(part(other, [x + w - DS_W, y + h - DS_H, DS_W, DS_H])),
                )
            }
        }
    }

    /// Whether the main screen is drawn big enough for a shader: from 2x. The
    /// other screen, drawn small or beside it, is always plain.
    pub fn shaded(self, frame: DsFrame) -> bool {
        self.parts(frame).0.dst[2] >= DS_W * 2.0
    }
}

/// One screen alone at the frame's scale, centred.
fn single(frame: DsFrame) -> [f32; 4] {
    let s = frame.scale(DS_W, DS_H);
    let (w, h) = ((DS_W * s).round(), (DS_H * s).round());
    let (x, y) = frame.centred(w, h);
    [x, y, w, h]
}

/// A DS game's view as it was last left, or the shelf's layout for a game never played.
pub fn ds_view_for(root: &Path, stem: &str, starts: DsLayout) -> DsView {
    ini::value(root, DS_VIEW_FILE, stem)
        .as_deref()
        .and_then(DsView::parse)
        .unwrap_or(DsView {
            layout: starts,
            ..DsView::default()
        })
}

pub fn write_ds_view(root: &Path, stem: &str, view: DsView) -> std::io::Result<()> {
    ini::write(root, DS_VIEW_FILE, stem, &view.as_str())
}
