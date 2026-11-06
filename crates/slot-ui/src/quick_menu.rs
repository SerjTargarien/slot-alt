use crate::bitmaps::{button_legend, ButtonLegend};
use crate::draw::{shift, Canvas, Draw, TexId, OUT_H, OUT_W};
use crate::icon::{Icon, WARNING_INK};
use crate::plate::{blit, word_face, word_ink, UndoFace, CAP_GAP, HINT_EDGE};
use crate::power_menu::{MENU_H, MENU_INK, MENU_PAD, MENU_PX};
use crate::slot_chrome::{opening, MOUTH_H};
use crate::text;
use slot_store::controls::{self, Input};
use slot_store::{CarouselAction, Platform, Shortcut};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum QuickRow {
    FastForward,
    FastForwardSound,
    Screen,
    Rumble,
    DateTime,
    About,
    GbaShader,
    GbShader,
    GbaColourCorrection,
    GbColourCorrection,
    Game,
    EjectSave,
    Rewind,
    GbPalette,
    Theme,
    Wallpaper,
    Recents,
    ShowTitles,
    CartSlot,
    WallpaperTint,
    GbcShader,
    GbcColourCorrection,
    Resume,
    States,
    Screenshots,
    Settings,
    ExitGame,
    UndoReset,
    PlayScreen,
    PlayGameplay,
    Shader,
    ColourCorrection,
    Palette,
    ScreenGba,
    GbBezel,
    GbcBezel,
    Bezel,
    GbaBezel,
    ScreenGbc,
    ScreenGb,
    ScreenWs,
    ScreenLynx,
    WsShader,
    LynxShader,
    WsBezel,
    LynxBezel,
    WsScale,
    LynxScale,
    WsStretch,
    LynxStretch,
    ScreenGg,
    GgShader,
    GgBezel,
    GgScale,
    GgStretch,
    ScreenNgp,
    NgpShader,
    NgpBezel,
    NgpScale,
    NgpStretch,
    ScreenTg16,
    Tg16Shader,
    Tg16Bezel,
    Tg16Scale,
    Tg16Stretch,
    ScreenNds,
    NdsShader,
    NdsBezel,
    /// Slot²: the DS's layout a game starts with, its Scale and Stretch.
    NdsLayout,
    /// Slot²: whether moving the DS's stylus brings its touch screen up, for every DS
    /// game (the shelf's) and for the one playing.
    NdsStylusScreen,
    NdsScale,
    NdsStretch,
    General,
    ShelfPrev,
    ShelfNext,
    ShelfFavorite,
    ShelfForget,
    PlayControls,
    Shortcuts,
    GameUp,
    GameDown,
    GameLeft,
    GameRight,
    GameA,
    GameB,
    GameSelect,
    GameStart,
    GameL,
    GameR,
    GameX,
    GameY,
    GameY1,
    GameY2,
    GameY3,
    GameY4,
    GameSwapScreens,
    GameNextLayout,
    GameStylus,
    GameTouch,
    GameStylusUp,
    GameStylusDown,
    GameStylusLeft,
    GameStylusRight,
    ScSave,
    ScLoad,
    ScScreenshot,
    ScBrightUp,
    ScBrightDown,
    ScBlueDown,
    ScBlueUp,
    ScColour,
    ScPaletteBack,
    ScPaletteNext,
    ScShaderNext,
    ScShaderBack,
    ScFastForward,
    ScFastForwardToggle,
    ScRewind,
    ScSunlightUp,
    ScSunlightDown,
    ScReset,
    LidOff,
    FetchArt,
    Controls,
    ControlsShelf,
    ControlsInGame,
    ControlsGb,
    ControlsGbc,
    ControlsGba,
    ControlsWs,
    ControlsLynx,
    ControlsGg,
    ControlsNgp,
    ControlsTg16,
    ControlsNds,
    ScreenGeneral,
    Brightness,
    BlueLight,
    Saturation,
    Contrast,
    PlayBrightness,
    PlayBlueLight,
    PlaySaturation,
    PlayContrast,
    AutoLoad,
    UndoSave,
    UndoLoad,
    GbaScale,
    GbScale,
    GbStretch,
    GbcScale,
    GbcStretch,
    Scale,
    Stretch,
    GbaBlending,
    GbcBlending,
    GbBlending,
    Blending,
    /// Slot²: the playing DS game's own layout.
    Layout,
    StylusScreen,
    Guide,
    ScGuide,
    ScStretchOn,
    ScStretchOff,
    Network,
    Wifi,
    WifiPower,
    Networks,
    CheckUpdates,
    Server,
    Achievements,
    RaUsername,
    RaPassword,
    RaMode,
    RaEncore,
    Romm,
    RommProtocol,
    RommServer,
    RommPort,
    RommSelfSigned,
    RommPair,
    RommStatus,
    RommPairedServer,
    RommDisconnect,
}

impl QuickRow {
    pub const ALL: [QuickRow; 184] = [
        QuickRow::FastForward,
        QuickRow::FastForwardSound,
        QuickRow::Screen,
        QuickRow::Rumble,
        QuickRow::DateTime,
        QuickRow::About,
        QuickRow::GbaShader,
        QuickRow::GbShader,
        QuickRow::GbaColourCorrection,
        QuickRow::GbColourCorrection,
        QuickRow::Game,
        QuickRow::EjectSave,
        QuickRow::Rewind,
        QuickRow::GbPalette,
        QuickRow::Theme,
        QuickRow::Wallpaper,
        QuickRow::Recents,
        QuickRow::ShowTitles,
        QuickRow::CartSlot,
        QuickRow::WallpaperTint,
        QuickRow::GbcShader,
        QuickRow::GbcColourCorrection,
        QuickRow::Resume,
        QuickRow::States,
        QuickRow::Screenshots,
        QuickRow::Settings,
        QuickRow::ExitGame,
        QuickRow::UndoReset,
        QuickRow::PlayScreen,
        QuickRow::PlayGameplay,
        QuickRow::Shader,
        QuickRow::ColourCorrection,
        QuickRow::Palette,
        QuickRow::ScreenGba,
        QuickRow::GbBezel,
        QuickRow::GbcBezel,
        QuickRow::Bezel,
        QuickRow::GbaBezel,
        QuickRow::ScreenGbc,
        QuickRow::ScreenGb,
        QuickRow::ScreenWs,
        QuickRow::ScreenLynx,
        QuickRow::WsShader,
        QuickRow::LynxShader,
        QuickRow::WsBezel,
        QuickRow::LynxBezel,
        QuickRow::WsScale,
        QuickRow::LynxScale,
        QuickRow::WsStretch,
        QuickRow::LynxStretch,
        QuickRow::ScreenGg,
        QuickRow::GgShader,
        QuickRow::GgBezel,
        QuickRow::GgScale,
        QuickRow::GgStretch,
        QuickRow::ScreenNgp,
        QuickRow::NgpShader,
        QuickRow::NgpBezel,
        QuickRow::NgpScale,
        QuickRow::NgpStretch,
        QuickRow::ScreenTg16,
        QuickRow::Tg16Shader,
        QuickRow::Tg16Bezel,
        QuickRow::Tg16Scale,
        QuickRow::Tg16Stretch,
        QuickRow::ScreenNds,
        QuickRow::NdsShader,
        QuickRow::NdsBezel,
        QuickRow::NdsLayout,
        QuickRow::NdsStylusScreen,
        QuickRow::NdsScale,
        QuickRow::NdsStretch,
        QuickRow::General,
        QuickRow::ShelfPrev,
        QuickRow::ShelfNext,
        QuickRow::ShelfFavorite,
        QuickRow::ShelfForget,
        QuickRow::PlayControls,
        QuickRow::Shortcuts,
        QuickRow::GameUp,
        QuickRow::GameDown,
        QuickRow::GameLeft,
        QuickRow::GameRight,
        QuickRow::GameA,
        QuickRow::GameB,
        QuickRow::GameSelect,
        QuickRow::GameStart,
        QuickRow::GameL,
        QuickRow::GameR,
        QuickRow::GameX,
        QuickRow::GameY,
        QuickRow::GameY1,
        QuickRow::GameY2,
        QuickRow::GameY3,
        QuickRow::GameY4,
        QuickRow::GameSwapScreens,
        QuickRow::GameNextLayout,
        QuickRow::GameStylus,
        QuickRow::GameTouch,
        QuickRow::GameStylusUp,
        QuickRow::GameStylusDown,
        QuickRow::GameStylusLeft,
        QuickRow::GameStylusRight,
        QuickRow::ScSave,
        QuickRow::ScLoad,
        QuickRow::ScScreenshot,
        QuickRow::ScBrightUp,
        QuickRow::ScBrightDown,
        QuickRow::ScBlueDown,
        QuickRow::ScBlueUp,
        QuickRow::ScColour,
        QuickRow::ScPaletteBack,
        QuickRow::ScPaletteNext,
        QuickRow::ScShaderNext,
        QuickRow::ScShaderBack,
        QuickRow::ScFastForward,
        QuickRow::ScFastForwardToggle,
        QuickRow::ScRewind,
        QuickRow::ScSunlightUp,
        QuickRow::ScSunlightDown,
        QuickRow::ScReset,
        QuickRow::LidOff,
        QuickRow::FetchArt,
        QuickRow::Controls,
        QuickRow::ControlsShelf,
        QuickRow::ControlsInGame,
        QuickRow::ControlsGb,
        QuickRow::ControlsGbc,
        QuickRow::ControlsGba,
        QuickRow::ControlsWs,
        QuickRow::ControlsLynx,
        QuickRow::ControlsGg,
        QuickRow::ControlsNgp,
        QuickRow::ControlsTg16,
        QuickRow::ControlsNds,
        QuickRow::ScreenGeneral,
        QuickRow::Brightness,
        QuickRow::BlueLight,
        QuickRow::Saturation,
        QuickRow::Contrast,
        QuickRow::PlayBrightness,
        QuickRow::PlayBlueLight,
        QuickRow::PlaySaturation,
        QuickRow::PlayContrast,
        QuickRow::AutoLoad,
        QuickRow::UndoSave,
        QuickRow::UndoLoad,
        QuickRow::GbaScale,
        QuickRow::GbScale,
        QuickRow::GbStretch,
        QuickRow::GbcScale,
        QuickRow::GbcStretch,
        QuickRow::Scale,
        QuickRow::Stretch,
        QuickRow::GbaBlending,
        QuickRow::GbcBlending,
        QuickRow::GbBlending,
        QuickRow::Blending,
        QuickRow::Layout,
        QuickRow::StylusScreen,
        QuickRow::Guide,
        QuickRow::ScGuide,
        QuickRow::ScStretchOn,
        QuickRow::ScStretchOff,
        QuickRow::Network,
        QuickRow::Wifi,
        QuickRow::WifiPower,
        QuickRow::Networks,
        QuickRow::CheckUpdates,
        QuickRow::Server,
        QuickRow::Achievements,
        QuickRow::RaUsername,
        QuickRow::RaPassword,
        QuickRow::RaMode,
        QuickRow::RaEncore,
        QuickRow::Romm,
        QuickRow::RommProtocol,
        QuickRow::RommServer,
        QuickRow::RommPort,
        QuickRow::RommSelfSigned,
        QuickRow::RommPair,
        QuickRow::RommStatus,
        QuickRow::RommPairedServer,
        QuickRow::RommDisconnect,
    ];

    pub const MAIN: [QuickRow; 7] = [
        QuickRow::General,
        QuickRow::Screen,
        QuickRow::Game,
        QuickRow::Controls,
        QuickRow::Network,
        QuickRow::Achievements,
        QuickRow::About,
    ];

    /// slotPlus: the RetroAchievements page. Hardcore needs a live connection and no save
    /// states, so Cheevo Mode is fixed at Casual.
    pub const ACHIEVEMENTS: [QuickRow; 4] = [
        QuickRow::RaUsername,
        QuickRow::RaPassword,
        QuickRow::RaMode,
        QuickRow::RaEncore,
    ];

    pub const GENERAL: [QuickRow; 9] = [
        QuickRow::Theme,
        QuickRow::Wallpaper,
        QuickRow::WallpaperTint,
        QuickRow::Recents,
        QuickRow::ShowTitles,
        QuickRow::CartSlot,
        QuickRow::LidOff,
        QuickRow::DateTime,
        QuickRow::Wifi,
    ];

    pub const WIFI: [QuickRow; 2] = [QuickRow::WifiPower, QuickRow::Networks];

    pub const NETWORK: [QuickRow; 4] = [
        QuickRow::Server,
        QuickRow::Romm,
        QuickRow::FetchArt,
        QuickRow::CheckUpdates,
    ];

    pub const ROMM: [QuickRow; 8] = [
        QuickRow::RommProtocol,
        QuickRow::RommServer,
        QuickRow::RommPort,
        QuickRow::RommSelfSigned,
        QuickRow::RommPair,
        QuickRow::RommStatus,
        QuickRow::RommPairedServer,
        QuickRow::RommDisconnect,
    ];

    pub const ROMM_SETUP: [QuickRow; 5] = [
        QuickRow::RommProtocol,
        QuickRow::RommServer,
        QuickRow::RommPort,
        QuickRow::RommSelfSigned,
        QuickRow::RommPair,
    ];

    pub const ROMM_PAIRED: [QuickRow; 3] = [
        QuickRow::RommStatus,
        QuickRow::RommPairedServer,
        QuickRow::RommDisconnect,
    ];

    pub const CONTROL_PAGES: [QuickRow; 3] = [
        QuickRow::ControlsShelf,
        QuickRow::Shortcuts,
        QuickRow::ControlsInGame,
    ];

    pub const SHELF_KEYS: [QuickRow; 4] = [
        QuickRow::ShelfPrev,
        QuickRow::ShelfNext,
        QuickRow::ShelfFavorite,
        QuickRow::ShelfForget,
    ];

    /// The systems with a controls page, in `slot_store::controls::SYSTEMS` order.
    pub const CONTROL_SYSTEMS: [QuickRow; 9] = [
        QuickRow::ControlsGb,
        QuickRow::ControlsGbc,
        QuickRow::ControlsGba,
        QuickRow::ControlsWs,
        QuickRow::ControlsLynx,
        QuickRow::ControlsGg,
        QuickRow::ControlsNgp,
        QuickRow::ControlsTg16,
        QuickRow::ControlsNds,
    ];

    /// A system's controls page: every input any system has, each system showing its own
    /// (see `slot_store::controls::inputs`).
    pub const CONTROLS: [QuickRow; 24] = [
        QuickRow::GameUp,
        QuickRow::GameDown,
        QuickRow::GameLeft,
        QuickRow::GameRight,
        QuickRow::GameY1,
        QuickRow::GameY2,
        QuickRow::GameY3,
        QuickRow::GameY4,
        QuickRow::GameA,
        QuickRow::GameB,
        QuickRow::GameX,
        QuickRow::GameY,
        QuickRow::GameSelect,
        QuickRow::GameStart,
        QuickRow::GameL,
        QuickRow::GameR,
        QuickRow::GameSwapScreens,
        QuickRow::GameNextLayout,
        QuickRow::GameStylus,
        QuickRow::GameTouch,
        QuickRow::GameStylusUp,
        QuickRow::GameStylusDown,
        QuickRow::GameStylusLeft,
        QuickRow::GameStylusRight,
    ];

    pub const SHORTCUTS: [QuickRow; 21] = [
        QuickRow::ScSave,
        QuickRow::ScLoad,
        QuickRow::ScReset,
        QuickRow::ScRewind,
        QuickRow::ScFastForward,
        QuickRow::ScFastForwardToggle,
        QuickRow::ScScreenshot,
        QuickRow::ScGuide,
        QuickRow::ScStretchOn,
        QuickRow::ScStretchOff,
        QuickRow::ScBrightDown,
        QuickRow::ScBrightUp,
        QuickRow::ScBlueDown,
        QuickRow::ScBlueUp,
        QuickRow::ScColour,
        QuickRow::ScPaletteBack,
        QuickRow::ScPaletteNext,
        QuickRow::ScShaderBack,
        QuickRow::ScShaderNext,
        QuickRow::ScSunlightDown,
        QuickRow::ScSunlightUp,
    ];

    pub const SCREEN: [QuickRow; 10] = [
        QuickRow::ScreenGeneral,
        QuickRow::ScreenGba,
        QuickRow::ScreenGbc,
        QuickRow::ScreenGb,
        QuickRow::ScreenWs,
        QuickRow::ScreenLynx,
        QuickRow::ScreenGg,
        QuickRow::ScreenNgp,
        QuickRow::ScreenTg16,
        QuickRow::ScreenNds,
    ];

    pub const SCREEN_GENERAL: [QuickRow; 4] = [
        QuickRow::Brightness,
        QuickRow::BlueLight,
        QuickRow::Saturation,
        QuickRow::Contrast,
    ];

    /// The WonderSwan, Lynx, Game Gear, NeoGeo Pocket and TurboGrafx-16 cores
    /// have no colour
    /// correction to set, so their pages hold a shader and a bezel.
    pub const SCREEN_WS: [QuickRow; 4] = [
        QuickRow::WsShader,
        QuickRow::WsBezel,
        QuickRow::WsScale,
        QuickRow::WsStretch,
    ];

    pub const SCREEN_LYNX: [QuickRow; 4] = [
        QuickRow::LynxShader,
        QuickRow::LynxBezel,
        QuickRow::LynxScale,
        QuickRow::LynxStretch,
    ];

    pub const SCREEN_GG: [QuickRow; 4] = [
        QuickRow::GgShader,
        QuickRow::GgBezel,
        QuickRow::GgScale,
        QuickRow::GgStretch,
    ];

    pub const SCREEN_NGP: [QuickRow; 4] = [
        QuickRow::NgpShader,
        QuickRow::NgpBezel,
        QuickRow::NgpScale,
        QuickRow::NgpStretch,
    ];

    pub const SCREEN_TG16: [QuickRow; 4] = [
        QuickRow::Tg16Shader,
        QuickRow::Tg16Bezel,
        QuickRow::Tg16Scale,
        QuickRow::Tg16Stretch,
    ];
    pub const SCREEN_NDS: [QuickRow; 6] = [
        QuickRow::NdsLayout,
        QuickRow::NdsStylusScreen,
        QuickRow::NdsShader,
        QuickRow::NdsScale,
        QuickRow::NdsStretch,
        QuickRow::NdsBezel,
    ];

    pub const SCREEN_GBA: [QuickRow; 5] = [
        QuickRow::GbaShader,
        QuickRow::GbaColourCorrection,
        QuickRow::GbaBlending,
        QuickRow::GbaBezel,
        QuickRow::GbaScale,
    ];

    pub const SCREEN_GBC: [QuickRow; 6] = [
        QuickRow::GbcShader,
        QuickRow::GbcColourCorrection,
        QuickRow::GbcBlending,
        QuickRow::GbcBezel,
        QuickRow::GbcScale,
        QuickRow::GbcStretch,
    ];

    pub const SCREEN_GB: [QuickRow; 7] = [
        QuickRow::GbShader,
        QuickRow::GbColourCorrection,
        QuickRow::GbBlending,
        QuickRow::GbPalette,
        QuickRow::GbBezel,
        QuickRow::GbScale,
        QuickRow::GbStretch,
    ];

    pub const GAME: [QuickRow; 6] = [
        QuickRow::FastForward,
        QuickRow::FastForwardSound,
        QuickRow::Rewind,
        QuickRow::Rumble,
        QuickRow::AutoLoad,
        QuickRow::EjectSave,
    ];

    pub const PLAY: [QuickRow; 9] = [
        QuickRow::Resume,
        QuickRow::UndoSave,
        QuickRow::UndoLoad,
        QuickRow::UndoReset,
        QuickRow::States,
        QuickRow::Screenshots,
        QuickRow::Guide,
        QuickRow::Settings,
        QuickRow::ExitGame,
    ];

    pub const SETTINGS: [QuickRow; 3] = [
        QuickRow::PlayScreen,
        QuickRow::PlayGameplay,
        QuickRow::PlayControls,
    ];

    pub const PLAY_SCREEN: [QuickRow; 13] = [
        QuickRow::Shader,
        QuickRow::Layout,
        QuickRow::StylusScreen,
        QuickRow::Bezel,
        QuickRow::Scale,
        QuickRow::Stretch,
        QuickRow::Palette,
        QuickRow::ColourCorrection,
        QuickRow::Blending,
        QuickRow::PlayBrightness,
        QuickRow::PlayBlueLight,
        QuickRow::PlaySaturation,
        QuickRow::PlayContrast,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn page(self) -> &'static [QuickRow] {
        match self {
            QuickRow::ScreenGeneral
            | QuickRow::ScreenGba
            | QuickRow::ScreenGbc
            | QuickRow::ScreenGb
            | QuickRow::ScreenWs
            | QuickRow::ScreenLynx
            | QuickRow::ScreenGg
            | QuickRow::ScreenNgp
            | QuickRow::ScreenTg16
            | QuickRow::ScreenNds => &QuickRow::SCREEN,
            QuickRow::Brightness
            | QuickRow::BlueLight
            | QuickRow::Saturation
            | QuickRow::Contrast => &QuickRow::SCREEN_GENERAL,
            QuickRow::PlayBrightness
            | QuickRow::PlayBlueLight
            | QuickRow::PlaySaturation
            | QuickRow::PlayContrast => &QuickRow::PLAY_SCREEN,
            QuickRow::WsShader | QuickRow::WsBezel | QuickRow::WsScale | QuickRow::WsStretch => {
                &QuickRow::SCREEN_WS
            }
            QuickRow::LynxShader
            | QuickRow::LynxBezel
            | QuickRow::LynxScale
            | QuickRow::LynxStretch => &QuickRow::SCREEN_LYNX,
            QuickRow::GgShader | QuickRow::GgBezel | QuickRow::GgScale | QuickRow::GgStretch => {
                &QuickRow::SCREEN_GG
            }
            QuickRow::NgpShader
            | QuickRow::NgpBezel
            | QuickRow::NgpScale
            | QuickRow::NgpStretch => &QuickRow::SCREEN_NGP,
            QuickRow::Tg16Shader
            | QuickRow::Tg16Bezel
            | QuickRow::Tg16Scale
            | QuickRow::Tg16Stretch => &QuickRow::SCREEN_TG16,
            QuickRow::NdsShader
            | QuickRow::NdsBezel
            | QuickRow::NdsLayout
            | QuickRow::NdsStylusScreen
            | QuickRow::NdsScale
            | QuickRow::NdsStretch => &QuickRow::SCREEN_NDS,
            QuickRow::GbaShader
            | QuickRow::GbaColourCorrection
            | QuickRow::GbaBlending
            | QuickRow::GbaBezel
            | QuickRow::GbaScale => &QuickRow::SCREEN_GBA,
            QuickRow::GbcShader
            | QuickRow::GbcColourCorrection
            | QuickRow::GbcBlending
            | QuickRow::GbcBezel
            | QuickRow::GbcScale
            | QuickRow::GbcStretch => &QuickRow::SCREEN_GBC,
            QuickRow::GbShader
            | QuickRow::GbColourCorrection
            | QuickRow::GbBlending
            | QuickRow::GbPalette
            | QuickRow::GbBezel
            | QuickRow::GbScale
            | QuickRow::GbStretch => &QuickRow::SCREEN_GB,
            QuickRow::FastForward
            | QuickRow::FastForwardSound
            | QuickRow::Rewind
            | QuickRow::Rumble
            | QuickRow::AutoLoad
            | QuickRow::EjectSave => &QuickRow::GAME,
            QuickRow::Wallpaper
            | QuickRow::WallpaperTint
            | QuickRow::DateTime
            | QuickRow::Theme
            | QuickRow::Recents
            | QuickRow::ShowTitles
            | QuickRow::CartSlot
            | QuickRow::LidOff
            | QuickRow::Wifi => &QuickRow::GENERAL,
            QuickRow::WifiPower | QuickRow::Networks => &QuickRow::WIFI,
            QuickRow::CheckUpdates
            | QuickRow::FetchArt
            | QuickRow::Romm
            | QuickRow::Server => &QuickRow::NETWORK,
            QuickRow::RaUsername | QuickRow::RaPassword | QuickRow::RaMode | QuickRow::RaEncore => {
                &QuickRow::ACHIEVEMENTS
            }
            QuickRow::RommProtocol
            | QuickRow::RommServer
            | QuickRow::RommPort
            | QuickRow::RommSelfSigned
            | QuickRow::RommPair
            | QuickRow::RommStatus
            | QuickRow::RommPairedServer
            | QuickRow::RommDisconnect => &QuickRow::ROMM,
            QuickRow::ControlsShelf | QuickRow::Shortcuts | QuickRow::ControlsInGame => {
                &QuickRow::CONTROL_PAGES
            }
            QuickRow::Resume
            | QuickRow::UndoSave
            | QuickRow::UndoLoad
            | QuickRow::UndoReset
            | QuickRow::States
            | QuickRow::Screenshots
            | QuickRow::Guide
            | QuickRow::Settings
            | QuickRow::ExitGame => &QuickRow::PLAY,
            QuickRow::PlayScreen | QuickRow::PlayGameplay | QuickRow::PlayControls => {
                &QuickRow::SETTINGS
            }
            r if r.input().is_some() => &QuickRow::CONTROLS,
            r if r.control_system().is_some() => &QuickRow::CONTROL_SYSTEMS,
            r if r.shortcut().is_some() => &QuickRow::SHORTCUTS,
            r if r.carousel_action().is_some() => &QuickRow::SHELF_KEYS,
            QuickRow::Shader
            | QuickRow::ColourCorrection
            | QuickRow::Palette
            | QuickRow::Bezel
            | QuickRow::Scale
            | QuickRow::Stretch
            | QuickRow::Blending
            | QuickRow::Layout
            | QuickRow::StylusScreen => &QuickRow::PLAY_SCREEN,
            _ => &QuickRow::MAIN,
        }
    }

    pub fn parent(self) -> Option<QuickRow> {
        match self.page()[0] {
            QuickRow::ScreenGeneral => Some(QuickRow::Screen),
            QuickRow::Brightness => Some(QuickRow::ScreenGeneral),
            QuickRow::GbaShader => Some(QuickRow::ScreenGba),
            QuickRow::GbcShader => Some(QuickRow::ScreenGbc),
            QuickRow::GbShader => Some(QuickRow::ScreenGb),
            QuickRow::WsShader => Some(QuickRow::ScreenWs),
            QuickRow::LynxShader => Some(QuickRow::ScreenLynx),
            QuickRow::GgShader => Some(QuickRow::ScreenGg),
            QuickRow::NgpShader => Some(QuickRow::ScreenNgp),
            QuickRow::Tg16Shader => Some(QuickRow::ScreenTg16),
            QuickRow::NdsShader => Some(QuickRow::ScreenNds),
            QuickRow::FastForward => Some(QuickRow::Game),
            QuickRow::PlayScreen => Some(QuickRow::Settings),
            QuickRow::Shader => Some(QuickRow::PlayScreen),
            // The app returns to the system the page belongs to.
            QuickRow::GameUp => Some(QuickRow::ControlsGb),
            QuickRow::ControlsGb => Some(QuickRow::ControlsInGame),
            QuickRow::ScSave => Some(QuickRow::Shortcuts),
            QuickRow::Theme => Some(QuickRow::General),
            QuickRow::Server => Some(QuickRow::Network),
            QuickRow::RommProtocol => Some(QuickRow::Romm),
            QuickRow::WifiPower => Some(QuickRow::Wifi),
            QuickRow::RaUsername => Some(QuickRow::Achievements),
            QuickRow::ControlsShelf => Some(QuickRow::Controls),
            QuickRow::ShelfPrev => Some(QuickRow::ControlsShelf),
            _ => None,
        }
    }

    pub fn parent_in_game(self) -> Option<QuickRow> {
        match self.parent() {
            Some(QuickRow::Game) => Some(QuickRow::PlayGameplay),
            Some(QuickRow::ControlsInGame) => Some(QuickRow::PlayControls),
            Some(r) if r.control_system().is_some() => Some(QuickRow::PlayControls),
            parent => parent,
        }
    }

    pub fn centred(self) -> bool {
        [
            &QuickRow::MAIN[..],
            &QuickRow::PLAY,
            &QuickRow::SETTINGS,
            &QuickRow::SCREEN,
            &QuickRow::NETWORK,
            &QuickRow::CONTROL_PAGES,
            &QuickRow::CONTROL_SYSTEMS,
        ]
        .contains(&self.page())
    }

    pub fn child(self) -> Option<QuickRow> {
        match self {
            QuickRow::Screen => Some(QuickRow::SCREEN[0]),
            QuickRow::Game => Some(QuickRow::GAME[0]),
            QuickRow::Settings => Some(QuickRow::SETTINGS[0]),
            QuickRow::PlayScreen => Some(QuickRow::PLAY_SCREEN[0]),
            QuickRow::PlayGameplay => Some(QuickRow::GAME[0]),
            QuickRow::PlayControls => Some(QuickRow::CONTROLS[0]),
            QuickRow::Shortcuts => Some(QuickRow::SHORTCUTS[0]),
            QuickRow::General => Some(QuickRow::GENERAL[0]),
            QuickRow::Network => Some(QuickRow::NETWORK[0]),
            QuickRow::Achievements => Some(QuickRow::ACHIEVEMENTS[0]),
            QuickRow::Romm => Some(QuickRow::ROMM[0]),
            QuickRow::Wifi => Some(QuickRow::WIFI[0]),
            QuickRow::Controls => Some(QuickRow::CONTROL_PAGES[0]),
            QuickRow::ControlsShelf => Some(QuickRow::SHELF_KEYS[0]),
            QuickRow::ControlsInGame => Some(QuickRow::CONTROL_SYSTEMS[0]),
            r if r.control_system().is_some() => Some(QuickRow::CONTROLS[0]),
            QuickRow::ScreenGeneral => Some(QuickRow::SCREEN_GENERAL[0]),
            QuickRow::ScreenGba => Some(QuickRow::SCREEN_GBA[0]),
            QuickRow::ScreenGbc => Some(QuickRow::SCREEN_GBC[0]),
            QuickRow::ScreenGb => Some(QuickRow::SCREEN_GB[0]),
            QuickRow::ScreenWs => Some(QuickRow::SCREEN_WS[0]),
            QuickRow::ScreenLynx => Some(QuickRow::SCREEN_LYNX[0]),
            QuickRow::ScreenGg => Some(QuickRow::SCREEN_GG[0]),
            QuickRow::ScreenNgp => Some(QuickRow::SCREEN_NGP[0]),
            QuickRow::ScreenTg16 => Some(QuickRow::SCREEN_TG16[0]),
            QuickRow::ScreenNds => Some(QuickRow::SCREEN_NDS[0]),
            _ => None,
        }
    }

    pub fn shown(self, hidden: &[QuickRow]) -> Vec<QuickRow> {
        self.page()
            .iter()
            .copied()
            .filter(|r| !hidden.contains(r))
            .collect()
    }

    pub fn position(self, hidden: &[QuickRow]) -> usize {
        self.shown(hidden)
            .iter()
            .position(|r| *r == self)
            .unwrap_or(0)
    }

    pub fn label(self) -> &'static str {
        match self {
            QuickRow::FastForward => "Fast Forward",
            QuickRow::FastForwardSound => "Fast Forward Sound",
            QuickRow::Screen => "Screen",
            QuickRow::Rumble => "Rumble",
            QuickRow::ScreenGeneral => "General",
            QuickRow::Brightness | QuickRow::PlayBrightness => "Brightness",
            QuickRow::BlueLight | QuickRow::PlayBlueLight => "Blue Light",
            QuickRow::Saturation | QuickRow::PlaySaturation => "Saturation",
            QuickRow::Contrast | QuickRow::PlayContrast => "Contrast",
            QuickRow::DateTime => "Date & Time",
            QuickRow::About => "About",
            QuickRow::GbaShader => "Shader",
            QuickRow::GbShader => "Shader",
            QuickRow::GbcShader => "Shader",
            QuickRow::GbaColourCorrection => "Color Correction",
            QuickRow::GbColourCorrection => "Color Correction",
            QuickRow::GbcColourCorrection => "Color Correction",
            QuickRow::Game => "Gameplay",
            QuickRow::AutoLoad => "Load State on Insert",
            QuickRow::EjectSave => "Save State on Eject",
            QuickRow::Rewind => "Rewind",
            QuickRow::GbPalette => "Palette",
            QuickRow::Theme => "Theme",
            QuickRow::Wallpaper => "Wallpaper",
            QuickRow::Recents => "Recents",
            QuickRow::ShowTitles => "Game Titles",
            QuickRow::CartSlot => "Cart Slot",
            QuickRow::WallpaperTint => "Wallpaper Tint",
            QuickRow::Resume => "Resume",
            QuickRow::UndoSave => "Undo Save",
            QuickRow::UndoLoad => "Undo Load",
            QuickRow::UndoReset => "Undo Reset",
            QuickRow::States => "States",
            QuickRow::Screenshots => "Screenshots",
            QuickRow::Guide => "Guide",
            QuickRow::Settings => "Settings",
            QuickRow::ExitGame => "Exit Game",
            QuickRow::General => "General",
            QuickRow::LidOff => "Lid Power Off",
            QuickRow::FetchArt => "Fetch Missing Art",
            QuickRow::Network => "Tools",
            QuickRow::Wifi | QuickRow::WifiPower => "Wi-Fi",
            QuickRow::Networks => "Network",
            QuickRow::Achievements => "RetroAchievements",
            QuickRow::RaUsername => "Username",
            QuickRow::RaPassword => "Password",
            QuickRow::RaMode => "Cheevo Mode",
            QuickRow::RaEncore => "Encore",
            QuickRow::CheckUpdates => "Check for Updates",
            QuickRow::Server => "Gamebit",
            QuickRow::Romm => "RomM",
            QuickRow::RommProtocol => "Protocol",
            QuickRow::RommServer | QuickRow::RommPairedServer => "Server",
            QuickRow::RommPort => "Port",
            QuickRow::RommSelfSigned => "Accept Self-Signed",
            QuickRow::RommPair => "Pair",
            QuickRow::RommStatus => "Status",
            QuickRow::RommDisconnect => "Disconnect",
            QuickRow::Controls => "Controls",
            QuickRow::ControlsShelf => "Shelf",
            QuickRow::ControlsInGame => "In-Game",
            QuickRow::PlayControls => "Controls",
            QuickRow::Shortcuts => "Shortcuts",
            QuickRow::GameUp
            | QuickRow::GameDown
            | QuickRow::GameLeft
            | QuickRow::GameRight
            | QuickRow::GameA
            | QuickRow::GameB
            | QuickRow::GameSelect
            | QuickRow::GameStart
            | QuickRow::GameL
            | QuickRow::GameR
            | QuickRow::GameX
            | QuickRow::GameY
            | QuickRow::GameY1
            | QuickRow::GameY2
            | QuickRow::GameY3
            | QuickRow::GameY4
            | QuickRow::GameSwapScreens
            | QuickRow::GameNextLayout
            | QuickRow::GameStylus
            | QuickRow::GameTouch
            | QuickRow::GameStylusUp
            | QuickRow::GameStylusDown
            | QuickRow::GameStylusLeft
            | QuickRow::GameStylusRight => self
                .input()
                .map_or("", |i| controls::label(Platform::Gba, i)),
            QuickRow::ControlsGb => "Game Boy",
            QuickRow::ControlsGbc => "Game Boy Color",
            QuickRow::ControlsGba => "Game Boy Advance",
            QuickRow::ControlsWs => "WonderSwan",
            QuickRow::ControlsLynx => "Atari Lynx",
            QuickRow::ControlsGg => "Game Gear",
            QuickRow::ControlsNgp => "NeoGeo Pocket",
            QuickRow::ControlsTg16 => "TurboGrafx-16",
            QuickRow::ControlsNds => "Nintendo DS",
            QuickRow::ScSave
            | QuickRow::ScLoad
            | QuickRow::ScScreenshot
            | QuickRow::ScBrightUp
            | QuickRow::ScBrightDown
            | QuickRow::ScBlueDown
            | QuickRow::ScBlueUp
            | QuickRow::ScColour
            | QuickRow::ScPaletteBack
            | QuickRow::ScPaletteNext
            | QuickRow::ScShaderNext
            | QuickRow::ScShaderBack
            | QuickRow::ScFastForward
            | QuickRow::ScFastForwardToggle
            | QuickRow::ScRewind
            | QuickRow::ScSunlightUp
            | QuickRow::ScSunlightDown
            | QuickRow::ScReset
            | QuickRow::ScGuide
            | QuickRow::ScStretchOn
            | QuickRow::ScStretchOff => self.shortcut().map_or("", Shortcut::label),
            QuickRow::ShelfPrev
            | QuickRow::ShelfNext
            | QuickRow::ShelfFavorite
            | QuickRow::ShelfForget => self.carousel_action().map_or("", CarouselAction::label),
            QuickRow::PlayScreen => "Screen",
            QuickRow::PlayGameplay => "Gameplay",
            QuickRow::Shader => "Shader",
            QuickRow::ColourCorrection => "Color Correction",
            QuickRow::Palette => "Palette",
            QuickRow::ScreenGba => "Game Boy Advance",
            QuickRow::ScreenGbc => "Game Boy Color",
            QuickRow::ScreenGb => "Game Boy",
            QuickRow::GbBezel => "Bezel",
            QuickRow::GbcBezel => "Bezel",
            QuickRow::GbaScale
            | QuickRow::GbScale
            | QuickRow::GbcScale
            | QuickRow::GgScale
            | QuickRow::NgpScale
            | QuickRow::Tg16Scale
            | QuickRow::WsScale
            | QuickRow::LynxScale
            | QuickRow::Scale => "Scale",
            QuickRow::GbStretch
            | QuickRow::GbcStretch
            | QuickRow::GgStretch
            | QuickRow::NgpStretch
            | QuickRow::Tg16Stretch
            | QuickRow::WsStretch
            | QuickRow::LynxStretch
            | QuickRow::Stretch => "Stretch",
            QuickRow::GbaBlending
            | QuickRow::GbcBlending
            | QuickRow::GbBlending
            | QuickRow::Blending => "Frame Blending",
            QuickRow::GbaBezel => "Bezel",
            QuickRow::Bezel => "Bezel",
            QuickRow::ScreenWs => "WonderSwan",
            QuickRow::ScreenLynx => "Atari Lynx",
            QuickRow::WsShader => "Shader",
            QuickRow::LynxShader => "Shader",
            QuickRow::WsBezel => "Bezel",
            QuickRow::LynxBezel => "Bezel",
            QuickRow::ScreenGg => "Game Gear",
            QuickRow::GgShader => "Shader",
            QuickRow::GgBezel => "Bezel",
            QuickRow::ScreenNgp => "NeoGeo Pocket",
            QuickRow::NgpShader => "Shader",
            QuickRow::NgpBezel => "Bezel",
            QuickRow::ScreenTg16 => "TurboGrafx-16",
            QuickRow::Tg16Shader => "Shader",
            QuickRow::Tg16Bezel => "Bezel",
            QuickRow::ScreenNds => "Nintendo DS",
            QuickRow::NdsShader => "Shader",
            QuickRow::NdsBezel => "Bezel",
            QuickRow::NdsLayout | QuickRow::Layout => "Layout",
            QuickRow::NdsStylusScreen | QuickRow::StylusScreen => "Stylus Shows Touch Screen",
            QuickRow::NdsScale => "Scale",
            QuickRow::NdsStretch => "Stretch",
        }
    }

    pub fn opens(self) -> bool {
        matches!(
            self,
            QuickRow::DateTime
                | QuickRow::FetchArt
                | QuickRow::About
                | QuickRow::Screen
                | QuickRow::Game
                | QuickRow::Resume
                | QuickRow::UndoSave
                | QuickRow::UndoLoad
                | QuickRow::UndoReset
                | QuickRow::States
                | QuickRow::Screenshots
                | QuickRow::Guide
                | QuickRow::Settings
                | QuickRow::ExitGame
                | QuickRow::PlayScreen
                | QuickRow::PlayGameplay
                | QuickRow::General
                | QuickRow::Controls
                | QuickRow::ControlsShelf
                | QuickRow::ControlsInGame
                | QuickRow::ControlsGb
                | QuickRow::ControlsGbc
                | QuickRow::ControlsGba
                | QuickRow::ControlsWs
                | QuickRow::ControlsLynx
                | QuickRow::ControlsGg
                | QuickRow::ControlsNgp
                | QuickRow::ControlsTg16
                | QuickRow::ControlsNds
                | QuickRow::PlayControls
                | QuickRow::Shortcuts
                | QuickRow::ScreenGba
                | QuickRow::ScreenGbc
                | QuickRow::ScreenGb
                | QuickRow::ScreenWs
                | QuickRow::ScreenLynx
                | QuickRow::ScreenGg
                | QuickRow::ScreenNgp
                | QuickRow::ScreenTg16
                | QuickRow::ScreenNds
                | QuickRow::ScreenGeneral
                | QuickRow::Network
                | QuickRow::Wifi
                | QuickRow::Networks
                | QuickRow::CheckUpdates
                | QuickRow::Server
                | QuickRow::Achievements
                | QuickRow::RaUsername
                | QuickRow::RaPassword
                | QuickRow::Romm
                | QuickRow::RommServer
                | QuickRow::RommPort
                | QuickRow::RommPair
                | QuickRow::RommDisconnect
        )
    }

    /// The input a controls row binds.
    pub fn input(self) -> Option<Input> {
        Some(match self {
            QuickRow::GameUp => Input::Up,
            QuickRow::GameDown => Input::Down,
            QuickRow::GameLeft => Input::Left,
            QuickRow::GameRight => Input::Right,
            QuickRow::GameA => Input::A,
            QuickRow::GameB => Input::B,
            QuickRow::GameX => Input::X,
            QuickRow::GameY => Input::Y,
            QuickRow::GameSelect => Input::Select,
            QuickRow::GameStart => Input::Start,
            QuickRow::GameL => Input::L,
            QuickRow::GameR => Input::R,
            QuickRow::GameY1 => Input::Y1,
            QuickRow::GameY2 => Input::Y2,
            QuickRow::GameY3 => Input::Y3,
            QuickRow::GameY4 => Input::Y4,
            QuickRow::GameSwapScreens => Input::SwapScreens,
            QuickRow::GameNextLayout => Input::NextLayout,
            QuickRow::GameStylus => Input::Stylus,
            QuickRow::GameTouch => Input::Touch,
            QuickRow::GameStylusUp => Input::StylusUp,
            QuickRow::GameStylusDown => Input::StylusDown,
            QuickRow::GameStylusLeft => Input::StylusLeft,
            QuickRow::GameStylusRight => Input::StylusRight,
            _ => return None,
        })
    }

    /// The row for an input, on any system's page.
    pub fn of_input(input: Input) -> QuickRow {
        QuickRow::CONTROLS
            .into_iter()
            .find(|r| r.input() == Some(input))
            .unwrap_or(QuickRow::GameUp)
    }

    /// The system a row of the In-Game controls list opens.
    pub fn control_system(self) -> Option<Platform> {
        let at = QuickRow::CONTROL_SYSTEMS.iter().position(|r| *r == self)?;
        Some(controls::SYSTEMS[at])
    }

    /// The In-Game controls list's row for a system.
    pub fn controls_of(platform: Platform) -> QuickRow {
        let platform = match platform {
            Platform::Gbav => Platform::Gba,
            p => p,
        };
        let at = controls::SYSTEMS.iter().position(|p| *p == platform);
        QuickRow::CONTROL_SYSTEMS[at.unwrap_or(0)]
    }

    pub fn shortcut(self) -> Option<Shortcut> {
        Some(match self {
            QuickRow::ScSave => Shortcut::SaveState,
            QuickRow::ScLoad => Shortcut::LoadState,
            QuickRow::ScScreenshot => Shortcut::Screenshot,
            QuickRow::ScBrightUp => Shortcut::BrightnessUp,
            QuickRow::ScBrightDown => Shortcut::BrightnessDown,
            QuickRow::ScBlueDown => Shortcut::BlueLightDown,
            QuickRow::ScBlueUp => Shortcut::BlueLightUp,
            QuickRow::ScColour => Shortcut::ColorCorrection,
            QuickRow::ScPaletteBack => Shortcut::PaletteBack,
            QuickRow::ScPaletteNext => Shortcut::PaletteNext,
            QuickRow::ScShaderNext => Shortcut::ShaderNext,
            QuickRow::ScShaderBack => Shortcut::ShaderBack,
            QuickRow::ScFastForward => Shortcut::FastForward,
            QuickRow::ScFastForwardToggle => Shortcut::FastForwardToggle,
            QuickRow::ScRewind => Shortcut::Rewind,
            QuickRow::ScSunlightUp => Shortcut::SunlightUp,
            QuickRow::ScSunlightDown => Shortcut::SunlightDown,
            QuickRow::ScReset => Shortcut::Reset,
            QuickRow::ScGuide => Shortcut::Guide,
            QuickRow::ScStretchOn => Shortcut::StretchOn,
            QuickRow::ScStretchOff => Shortcut::StretchOff,
            _ => return None,
        })
    }

    pub fn carousel_action(self) -> Option<CarouselAction> {
        Some(match self {
            QuickRow::ShelfPrev => CarouselAction::PreviousShelf,
            QuickRow::ShelfNext => CarouselAction::NextShelf,
            QuickRow::ShelfFavorite => CarouselAction::Favorite,
            QuickRow::ShelfForget => CarouselAction::ForgetRecent,
            _ => return None,
        })
    }

    pub fn binds(self) -> bool {
        self.input().is_some() || self.shortcut().is_some() || self.carousel_action().is_some()
    }

    pub fn up(self, hidden: &[QuickRow]) -> QuickRow {
        let rows = self.shown(hidden);
        rows[(self.position(hidden) + rows.len() - 1) % rows.len()]
    }

    pub fn down(self, hidden: &[QuickRow]) -> QuickRow {
        let rows = self.shown(hidden);
        rows[(self.position(hidden) + 1) % rows.len()]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum QuickValue {
    Speed1_5,
    Speed2,
    Speed3,
    Speed4,
    Speed6,
    On,
    Off,
    Lcd3x,
    Grid,
    Dot,
    Simpletex,
    Shimmerless,
    Dark,
    Light,
    Show,
    Hide,
    Low,
    Medium,
    High,
    Transparent,
    Min1,
    Min3,
    Min5,
    Min10,
    Min15,
    Min30,
    Level0,
    Level1,
    Level2,
    Level3,
    Level4,
    Level5,
    Level6,
    Level7,
    Level8,
    Level9,
    SlotDefault,
    SlotBespoke,
    Level10,
    Level11,
    Level12,
    Level13,
    Level14,
    Level15,
    SpFrame,
    Integer,
    Fit,
    Classic,
    FullScreen,
    Mix,
    Ghosting,
    Https,
    Http,
    /// Slot²: the DS's layouts, in `DS_LAYOUTS` order.
    DsStacked,
    DsSingle,
    DsBelow,
    DsCorner,
}

impl QuickValue {
    pub const ALL: [QuickValue; 57] = [
        QuickValue::Speed1_5,
        QuickValue::Speed2,
        QuickValue::Speed3,
        QuickValue::Speed4,
        QuickValue::Speed6,
        QuickValue::On,
        QuickValue::Off,
        QuickValue::Lcd3x,
        QuickValue::Grid,
        QuickValue::Dot,
        QuickValue::Simpletex,
        QuickValue::Shimmerless,
        QuickValue::Dark,
        QuickValue::Light,
        QuickValue::Show,
        QuickValue::Hide,
        QuickValue::Low,
        QuickValue::Medium,
        QuickValue::High,
        QuickValue::Transparent,
        QuickValue::Min1,
        QuickValue::Min3,
        QuickValue::Min5,
        QuickValue::Min10,
        QuickValue::Min15,
        QuickValue::Min30,
        QuickValue::Level0,
        QuickValue::Level1,
        QuickValue::Level2,
        QuickValue::Level3,
        QuickValue::Level4,
        QuickValue::Level5,
        QuickValue::Level6,
        QuickValue::Level7,
        QuickValue::Level8,
        QuickValue::Level9,
        QuickValue::SlotDefault,
        QuickValue::SlotBespoke,
        QuickValue::Level10,
        QuickValue::Level11,
        QuickValue::Level12,
        QuickValue::Level13,
        QuickValue::Level14,
        QuickValue::Level15,
        QuickValue::SpFrame,
        QuickValue::Integer,
        QuickValue::Fit,
        QuickValue::Classic,
        QuickValue::FullScreen,
        QuickValue::Mix,
        QuickValue::Ghosting,
        QuickValue::Https,
        QuickValue::Http,
        QuickValue::DsStacked,
        QuickValue::DsSingle,
        QuickValue::DsBelow,
        QuickValue::DsCorner,
    ];

    pub const LEVELS: [QuickValue; 16] = [
        QuickValue::Level0,
        QuickValue::Level1,
        QuickValue::Level2,
        QuickValue::Level3,
        QuickValue::Level4,
        QuickValue::Level5,
        QuickValue::Level6,
        QuickValue::Level7,
        QuickValue::Level8,
        QuickValue::Level9,
        QuickValue::Level10,
        QuickValue::Level11,
        QuickValue::Level12,
        QuickValue::Level13,
        QuickValue::Level14,
        QuickValue::Level15,
    ];

    pub const LID_OFF: [QuickValue; 6] = [
        QuickValue::Min1,
        QuickValue::Min3,
        QuickValue::Min5,
        QuickValue::Min10,
        QuickValue::Min15,
        QuickValue::Min30,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn text(self) -> &'static str {
        match self {
            QuickValue::Speed1_5 => "1.5×",
            QuickValue::Speed2 => "2×",
            QuickValue::Speed3 => "3×",
            QuickValue::Speed4 => "4×",
            QuickValue::Speed6 => "6×",
            QuickValue::On => "On",
            QuickValue::Off => "Off",
            QuickValue::Lcd3x => "LCD3x",
            QuickValue::Grid => "Grid",
            QuickValue::Dot => "Dot",
            QuickValue::Simpletex => "Simpletex",
            QuickValue::Shimmerless => "Shimmerless",
            QuickValue::Transparent => "Transparent",
            QuickValue::Dark => "Dark",
            QuickValue::Light => "Light",
            QuickValue::Show => "Show",
            QuickValue::Hide => "Hide",
            QuickValue::Low => "25%",
            QuickValue::Medium => "45%",
            QuickValue::High => "65%",
            QuickValue::Min1 => "1 min",
            QuickValue::Min3 => "3 min",
            QuickValue::Min5 => "5 min",
            QuickValue::Min10 => "10 min",
            QuickValue::Min15 => "15 min",
            QuickValue::Min30 => "30 min",
            QuickValue::Level0 => "0",
            QuickValue::Level1 => "1",
            QuickValue::Level2 => "2",
            QuickValue::Level3 => "3",
            QuickValue::Level4 => "4",
            QuickValue::Level5 => "5",
            QuickValue::Level6 => "6",
            QuickValue::Level7 => "7",
            QuickValue::Level8 => "8",
            QuickValue::Level9 => "9",
            QuickValue::SlotDefault => "Default",
            QuickValue::SlotBespoke => "Bespoke",
            QuickValue::Level10 => "10",
            QuickValue::Level11 => "11",
            QuickValue::Level12 => "12",
            QuickValue::Level13 => "13",
            QuickValue::Level14 => "14",
            QuickValue::Level15 => "15",
            QuickValue::SpFrame => "SP Screen",
            QuickValue::Integer => "Integer",
            QuickValue::Fit => "Fit",
            QuickValue::Classic => "Classic",
            QuickValue::FullScreen => "Full Screen",
            QuickValue::Mix => "Mix",
            QuickValue::Ghosting => "Ghosting",
            QuickValue::Https => "HTTPS",
            QuickValue::Http => "HTTP",
            QuickValue::DsStacked => crate::toast::DS_LAYOUTS[0],
            QuickValue::DsSingle => crate::toast::DS_LAYOUTS[1],
            QuickValue::DsBelow => crate::toast::DS_LAYOUTS[2],
            QuickValue::DsCorner => crate::toast::DS_LAYOUTS[3],
        }
    }

    pub fn level(n: u8) -> QuickValue {
        QuickValue::LEVELS[usize::from(n).min(QuickValue::LEVELS.len() - 1)]
    }

    pub fn speed(quarters: u8) -> Option<QuickValue> {
        match quarters {
            0 => Some(QuickValue::Off),
            6 => Some(QuickValue::Speed1_5),
            8 => Some(QuickValue::Speed2),
            12 => Some(QuickValue::Speed3),
            16 => Some(QuickValue::Speed4),
            24 => Some(QuickValue::Speed6),
            _ => None,
        }
    }

    pub fn minutes(self) -> Option<u8> {
        Some(match self {
            QuickValue::Min1 => 1,
            QuickValue::Min3 => 3,
            QuickValue::Min5 => 5,
            QuickValue::Min10 => 10,
            QuickValue::Min15 => 15,
            QuickValue::Min30 => 30,
            _ => return None,
        })
    }

    pub fn flag(on: bool) -> QuickValue {
        if on {
            QuickValue::On
        } else {
            QuickValue::Off
        }
    }
}

pub const QUICK_PITCH: f32 = 52.0;
pub const QUICK_EDGE: f32 = 32.0;
pub const QUICK_LABEL_X: f32 = 53.0;
pub const QUICK_VALUE_GAP: f32 = 24.0;
pub(crate) const VALUE_MIN_SCALE: f32 = 0.6;
pub(crate) const POINTER_GAP: f32 = 17.0;
const POINTER_SCALE: usize = 3;
const POINTER: [&str; 11] = [
    "X.....", "XX....", "XoX...", "XooX..", "XoooX.", "XooooX", "XoooX.", "XooX..", "XoX...",
    "XX....", "X.....",
];
pub(crate) const POINTER_BLEED: u32 = 4;
const GROUND_SCRIM: f32 = 0.65;
pub(crate) const TYPE_DROP: f32 = 4.0;
pub(crate) const LEGEND_Y: f32 = 427.0;
pub(crate) const DIM_INK: [u8; 3] = [0x9a, 0x9a, 0xa4];
const DESCRIPTION_PX: f32 = 16.0;
const DESCRIPTION_GAP: f32 = 8.0;
const PANEL_GAP: f32 = 12.0;
const WARNING_PX: f32 = 22.0;
const WARNING_GAP: u32 = 10;

pub fn quick_label_face(row: QuickRow, lit: bool) -> UndoFace {
    quick_text_face(row.label(), if lit { MENU_INK } else { DIM_INK })
}

/// Every name a controls row takes on some system's page, with its dim and lit faces.
pub fn input_label_faces() -> Vec<(&'static str, [UndoFace; 2])> {
    input_label_names()
        .into_iter()
        .map(|t| (t, [DIM_INK, MENU_INK].map(|ink| quick_text_face(t, ink))))
        .collect()
}

/// Every name a controls row takes on some system's page.
pub fn input_label_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = controls::SYSTEMS
        .iter()
        .flat_map(|&p| {
            controls::inputs(p)
                .iter()
                .map(move |&i| controls::label(p, i))
        })
        .collect();
    names.sort();
    names.dedup();
    names
}

fn cap_centre(label: &fontdue::Font, h: u32) -> f32 {
    match label.horizontal_line_metrics(MENU_PX) {
        Some(v) => {
            (h as f32 - v.new_line_size) / 2.0 + v.ascent
                - label.metrics('H', MENU_PX).height as f32 / 2.0
        }
        None => h as f32 / 2.0,
    }
}

pub fn quick_pointer_face() -> UndoFace {
    let s = POINTER_SCALE;
    let w = POINTER[0].len() * s;
    let h = (MENU_H + 2 * POINTER_BLEED) as usize;
    let centre = match text::label_font() {
        Some(label) => cap_centre(label, MENU_H),
        None => MENU_H as f32 / 2.0,
    } + POINTER_BLEED as f32;
    let tip = POINTER.len() / 2 * s + s / 2;
    let top = (centre - tip as f32).round() as usize;
    let mut rgba = vec![0u8; w * h * 4];
    for (y, row) in POINTER.iter().enumerate() {
        for (x, _) in row.chars().enumerate().filter(|(_, ch)| *ch != '.') {
            for py in top + y * s..top + (y + 1) * s {
                for px in x * s..(x + 1) * s {
                    let at = (py * w + px) * 4;
                    rgba[at..at + 4].copy_from_slice(&[
                        MENU_INK[0],
                        MENU_INK[1],
                        MENU_INK[2],
                        0xff,
                    ]);
                }
            }
        }
    }
    UndoFace {
        rgba,
        w: w as u32,
        h: h as u32,
    }
}

pub fn quick_value_face(text: &str, lit: bool) -> UndoFace {
    quick_text_face(text, if lit { MENU_INK } else { DIM_INK })
}

pub fn quick_warning_value_face(text: &str, lit: bool) -> UndoFace {
    let words = quick_value_face(text, lit);
    let (Some((cov, iw, ih)), Some(label)) = (
        crate::icon::icon_coverage(Icon::Alert, WARNING_PX),
        text::label_font(),
    ) else {
        return words;
    };
    let (w, h) = (iw + WARNING_GAP + words.w, words.h);
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    let top = (cap_centre(label, h) - ih as f32 / 2.0).round().max(0.0) as u32;
    blit(
        &mut rgba,
        w,
        &words.rgba,
        words.w,
        words.h,
        iw + WARNING_GAP,
        0,
    );
    for gy in 0..ih.min(h.saturating_sub(top)) {
        for gx in 0..iw {
            let a = cov[(gy * iw + gx) as usize];
            if a == 0 {
                continue;
            }
            let at = (((top + gy) * w + MENU_PAD + gx) * 4) as usize;
            rgba[at..at + 4].copy_from_slice(&[WARNING_INK[0], WARNING_INK[1], WARNING_INK[2], a]);
        }
    }
    UndoFace { rgba, w, h }
}

pub(crate) fn quick_text_face(label: &str, colour: [u8; 3]) -> UndoFace {
    let Some(font) = text::label_font() else {
        return UndoFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let layout = text::fit(font, label, OUT_W as f32, 1, MENU_PX, MENU_PX);
    let set = layout
        .lines
        .iter()
        .map(|l| text::line_width(font, l, layout.px, layout.tracking))
        .fold(0.0, f32::max);
    let w = set.ceil() as u32 + 2 * MENU_PAD;
    let mut rgba = vec![0u8; (w * MENU_H * 4) as usize];
    text::draw_centred(&mut rgba, w, MENU_H, &layout, colour);
    UndoFace { rgba, w, h: MENU_H }
}

/// The quick menu's legends: Back, Change, Open, Resume, Restart, then a shortcut row's
/// Set, Clear and Reset All, then a controls row's Add, Remove and Default.
pub fn quick_legend_faces(ink: [u8; 3]) -> [UndoFace; 11] {
    let hint = Some(ink);
    [
        with_hint(button_face(ButtonLegend::B), "Back", hint, true),
        with_hint(
            button_pair(ButtonLegend::Left, ButtonLegend::Right),
            "Change",
            hint,
            false,
        ),
        with_hint(button_face(ButtonLegend::A), "Open", hint, false),
        with_hint(button_face(ButtonLegend::B), "Resume", hint, true),
        with_hint(button_face(ButtonLegend::X), "Restart", hint, false),
        with_hint(button_face(ButtonLegend::A), "Set", hint, false),
        with_hint(button_face(ButtonLegend::X), "Clear", hint, false),
        with_hint(button_face(ButtonLegend::Y), "Reset All", hint, true),
        with_hint(button_face(ButtonLegend::A), "Add", hint, false),
        with_hint(button_face(ButtonLegend::X), "Remove", hint, false),
        with_hint(button_face(ButtonLegend::Y), "Default", hint, true),
    ]
}

pub fn quick_description_face(lines: &[&str]) -> UndoFace {
    let Some(font) = text::label_font() else {
        return UndoFace {
            rgba: Vec::new(),
            w: 0,
            h: 0,
        };
    };
    let line_h = (DESCRIPTION_PX * 1.4).ceil() as u32;
    let faces: Vec<(Vec<u8>, u32)> = lines
        .iter()
        .map(|line| {
            let layout = text::fit(font, line, OUT_W as f32, 1, DESCRIPTION_PX, DESCRIPTION_PX);
            let w = text::line_width(font, &layout.lines[0], layout.px, layout.tracking).ceil()
                as u32
                + 2;
            let mut rgba = vec![0u8; (w * line_h * 4) as usize];
            text::draw_centred(&mut rgba, w, line_h, &layout, DIM_INK);
            (rgba, w)
        })
        .collect();
    let w = faces.iter().map(|(_, w)| *w).max().unwrap_or(0);
    let h = line_h * faces.len() as u32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for (i, (face, fw)) in faces.iter().enumerate() {
        crate::plate::blit(
            &mut rgba,
            w,
            face,
            *fw,
            line_h,
            (w - fw) / 2,
            i as u32 * line_h,
        );
    }
    UndoFace { rgba, w, h }
}

/// Cancel, Swap, Choose. Swap keeps the button before the word, the way the old arrows did.
pub fn core_button_legends(ink: [u8; 3]) -> Vec<UndoFace> {
    let hint = Some(ink);
    vec![
        with_hint(button_face(ButtonLegend::B), "Cancel", hint, true),
        with_hint(
            button_pair(ButtonLegend::Left, ButtonLegend::Right),
            "Swap",
            hint,
            true,
        ),
        with_hint(button_face(ButtonLegend::A), "Choose", hint, false),
        // The Lynx rotation picker's chip goes any of four ways.
        with_hint(four_way(), "Swap", hint, true),
    ]
}

/// Cancel, Mode, Swap, Link, OK, Back, End Link.
pub fn link_button_legends(ink: [u8; 3]) -> Vec<UndoFace> {
    let hint = Some(ink);
    vec![
        with_hint(button_face(ButtonLegend::B), "Cancel", hint, true),
        with_hint(button_face(ButtonLegend::Select), "Mode", hint, true),
        with_hint(
            button_pair(ButtonLegend::Left, ButtonLegend::Right),
            "Swap",
            hint,
            false,
        ),
        with_hint(button_face(ButtonLegend::A), "Link", hint, false),
        with_hint(button_face(ButtonLegend::A), "OK", hint, false),
        with_hint(button_face(ButtonLegend::B), "Back", hint, true),
        with_hint(button_face(ButtonLegend::A), "End Link", hint, false),
    ]
}

pub fn clock_legend_faces(ink: [u8; 3]) -> [UndoFace; 6] {
    let hint = Some(ink);
    [
        with_hint(button_face(ButtonLegend::A), "Done", hint, false),
        with_hint(up_down_pair(), "Change", hint, false),
        with_hint(
            button_pair(ButtonLegend::Left, ButtonLegend::Right),
            "Page",
            hint,
            false,
        ),
        with_hint(button_face(ButtonLegend::A), "Change", hint, false),
        with_hint(button_face(ButtonLegend::A), "Select", hint, false),
        with_hint(button_face(ButtonLegend::B), "Done", hint, true),
    ]
}

pub fn repair_legends(ink: [u8; 3]) -> [UndoFace; 2] {
    prompt_legends(ink, "Not now", "Repair")
}

pub fn update_legends(ink: [u8; 3]) -> [UndoFace; 3] {
    let [cancel, install] = prompt_legends(ink, "Cancel", "Install");
    [
        cancel,
        install,
        with_hint(button_face(ButtonLegend::B), "Back", Some(ink), true),
    ]
}

pub fn delete_legends(ink: [u8; 3]) -> [UndoFace; 2] {
    let hint = Some(ink);
    [
        with_hint(button_face(ButtonLegend::B), "Keep", hint, true),
        with_hint(button_face(ButtonLegend::X), "Delete", hint, false),
    ]
}

pub fn fetch_choice_legends(ink: [u8; 3]) -> [UndoFace; 2] {
    let hint = Some(ink);
    [
        with_hint(button_face(ButtonLegend::B), "Cancel", hint, true),
        with_hint(
            button_pair(ButtonLegend::Left, ButtonLegend::Right),
            "Pick",
            hint,
            false,
        ),
    ]
}

pub fn fetch_legends(ink: [u8; 3]) -> UndoFace {
    with_hint(button_face(ButtonLegend::B), "Stop", Some(ink), true)
}

pub fn print_stop_legend(ink: [u8; 3]) -> UndoFace {
    let (src, w, h) = button_legend(ButtonLegend::B);
    let side = PRINT_STOP_BUTTON;
    let rgba = crate::art::cover_rgba(src, w, h, side, side).unwrap_or_default();
    let button = UndoFace {
        rgba,
        w: side,
        h: side,
    };
    with_hint(button, "Stop", Some(ink), true)
}

pub fn fetch_skip_legend(ink: [u8; 3]) -> UndoFace {
    with_hint(button_face(ButtonLegend::X), "Skip", Some(ink), false)
}

const PRINT_STOP_BUTTON: u32 = 24;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StudioGlyph {
    A,
    B,
    X,
    Y,
    LeftRight,
    UpDown,
}

pub fn studio_legend(
    glyph: StudioGlyph,
    word: &str,
    ink: [u8; 3],
    hint_on_right: bool,
) -> UndoFace {
    let button = match glyph {
        StudioGlyph::A => button_face(ButtonLegend::A),
        StudioGlyph::B => button_face(ButtonLegend::B),
        StudioGlyph::X => button_face(ButtonLegend::X),
        StudioGlyph::Y => button_face(ButtonLegend::Y),
        StudioGlyph::LeftRight => button_pair(ButtonLegend::Left, ButtonLegend::Right),
        StudioGlyph::UpDown => up_down_pair(),
    };
    with_hint(button, word, Some(ink), hint_on_right)
}

pub fn prompt_legends(ink: [u8; 3], no: &str, yes: &str) -> [UndoFace; 2] {
    let hint = Some(ink);
    [
        with_hint(button_face(ButtonLegend::B), no, hint, true),
        with_hint(button_face(ButtonLegend::A), yes, hint, false),
    ]
}

/// `hint_on_right` puts the word after the button, for a legend parked on the left of the footer.
fn with_hint(button: UndoFace, label: &str, ink: Option<[u8; 3]>, hint_on_right: bool) -> UndoFace {
    let word = match ink {
        Some(ink) => word_ink(label, ink),
        None => word_face(label),
    };
    let w = button.w + CAP_GAP + word.w + HINT_EDGE;
    let h = button.h;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    let text_y = h.saturating_sub(word.h) / 2;
    let (button_x, text_x) = match hint_on_right {
        true => (0, button.w + CAP_GAP),
        false => (word.w + CAP_GAP, 0),
    };
    blit(&mut rgba, w, &button.rgba, button.w, button.h, button_x, 0);
    blit(&mut rgba, w, &word.rgba, word.w, word.h, text_x, text_y);
    UndoFace { rgba, w, h }
}

pub(crate) fn button_face(which: ButtonLegend) -> UndoFace {
    let (rgba, w, h) = button_legend(which);
    UndoFace {
        rgba: rgba.to_vec(),
        w,
        h,
    }
}

/// All four directions as one cross-shaped D-pad, drawn in the style of
/// Slot's button art: a black outline, a light grey face lit from above with
/// a darker bevel, dark arrows, a soft shadow above and a pale rim below.
fn four_way() -> UndoFace {
    let (_, _, h) = button_legend(ButtonLegend::Left);
    let w = h;
    let c = w as f32 / 2.0;
    // Half the length of an arm, half its width, and the corner rounding.
    let (reach, half, round) = (24.5, 10.5, 3.0);
    let sd = |x: f32, y: f32| {
        let rect = |hx: f32, hy: f32| {
            let (qx, qy) = ((x - c).abs() - hx + round, (y - c).abs() - hy + round);
            let out = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
            out + qx.max(qy).min(0.0) - round
        };
        rect(reach, half).min(rect(half, reach))
    };
    // An arrow in each arm, pointing out from the middle.
    let arrow = |x: f32, y: f32| {
        let (dx, dy) = (x - c, y - c);
        let along_across = [(-dy, dx), (dy, -dx), (-dx, dy), (dx, -dy)];
        along_across.iter().any(|&(along, across)| {
            let t = along - 10.5;
            (0.0..8.5).contains(&t) && across.abs() <= (8.5 - t) * 0.62
        })
    };
    const SS: u32 = 4;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for py in 0..h {
        for px in 0..w {
            let mut acc = [0f32; 4];
            for sy in 0..SS {
                for sx in 0..SS {
                    let x = px as f32 + (sx as f32 + 0.5) / SS as f32;
                    let y = py as f32 + (sy as f32 + 0.5) / SS as f32;
                    let d = sd(x, y);
                    let (rgb, a) = if d <= -2.5 {
                        let shade = if arrow(x, y) {
                            60.0
                        } else {
                            // Lit from above: a bright line along the top
                            // edge, darker towards the bevel at the sides.
                            let lit = if sd(x, y - 2.0) > -2.5 { 227.0 } else { 177.0 };
                            let bevel = (1.0 - ((-d - 2.5) / 3.0).min(1.0)) * 50.0;
                            lit - bevel
                        };
                        ([shade; 3], 1.0)
                    } else if d <= 0.0 {
                        ([0.0; 3], 1.0)
                    } else if sd(x, y - 3.0) <= 0.0 {
                        // A pale rim below the key.
                        ([246.0; 3], 0.3)
                    } else if d <= 2.5 {
                        // A soft shadow round it.
                        ([0.0; 3], 0.27 * (1.0 - d / 2.5))
                    } else {
                        ([0.0; 3], 0.0)
                    };
                    for k in 0..3 {
                        acc[k] += rgb[k] * a;
                    }
                    acc[3] += a;
                }
            }
            let n = (SS * SS) as f32;
            let at = ((py * w + px) * 4) as usize;
            let a = acc[3] / n;
            for k in 0..3 {
                rgba[at + k] = if acc[3] > 0.0 {
                    (acc[k] / acc[3]).round() as u8
                } else {
                    0
                };
            }
            rgba[at + 3] = (a * 255.0).round() as u8;
        }
    }
    UndoFace { rgba, w, h }
}

const ARROW_EDGE: u32 = 480;

pub(crate) fn up_down_pair() -> UndoFace {
    let (left, lw, lh) = button_legend(ButtonLegend::Left);
    let (right, rw, rh) = button_legend(ButtonLegend::Right);
    join(
        (&arrow_turned(left, lw, lh), lw, lh),
        (&arrow_turned(right, rw, rh), rw, rh),
    )
}

fn arrow_turned(px: &[u8], w: u32, h: u32) -> Vec<u8> {
    let (w, h) = (w as usize, h as usize);
    let margin = h / 5;
    let ink = |x: usize, y: usize| {
        let at = (y * w + x) * 4;
        match px[at + 3] {
            255 => px[at..at + 3].iter().map(|&c| c as u32).sum::<u32>(),
            _ => u32::MAX,
        }
    };
    let seed = (margin..h - margin)
        .flat_map(|y| (margin..w - margin).map(move |x| (x, y)))
        .min_by_key(|&(x, y)| ink(x, y));
    let mut arrow = Vec::new();
    let mut seen = vec![false; w * h];
    let mut todo: Vec<(usize, usize)> = seed.into_iter().collect();
    while let Some((x, y)) = todo.pop() {
        if x >= w || y >= h || seen[y * w + x] || ink(x, y) > ARROW_EDGE {
            continue;
        }
        seen[y * w + x] = true;
        arrow.push((x, y));
        todo.extend([
            (x + 1, y),
            (x.wrapping_sub(1), y),
            (x, y + 1),
            (x, y.wrapping_sub(1)),
        ]);
    }
    let mut out = px.to_vec();
    let (Some(x0), Some(x1), Some(y0), Some(y1)) = (
        arrow.iter().map(|p| p.0).min(),
        arrow.iter().map(|p| p.0).max(),
        arrow.iter().map(|p| p.1).min(),
        arrow.iter().map(|p| p.1).max(),
    ) else {
        return out;
    };
    let side = (x1 - x0).max(y1 - y0) + 3;
    let (left, top) = ((x0 + x1 - side) / 2, (y0 + y1 - side) / 2);
    for y in 0..side {
        for x in 0..side {
            let from = ((top + side - 1 - x) * w + left + y) * 4;
            let to = ((top + y) * w + left + x) * 4;
            out[to..to + 4].copy_from_slice(&px[from..from + 4]);
        }
    }
    out
}

pub(crate) fn button_pair(left: ButtonLegend, right: ButtonLegend) -> UndoFace {
    let (a, aw, ah) = button_legend(left);
    let (b, bw, bh) = button_legend(right);
    join((a, aw, ah), (b, bw, bh))
}

fn join((a, aw, ah): (&[u8], u32, u32), (b, bw, bh): (&[u8], u32, u32)) -> UndoFace {
    let opaque =
        |px: &[u8], w: u32, h: u32, x: u32| px[(((h / 2) * w + x) * 4 + 3) as usize] == 255;
    let keep_a = (0..aw)
        .rev()
        .find(|&x| opaque(a, aw, ah, x))
        .map_or(aw, |x| x + 1);
    let skip_b = (0..bw).find(|&x| opaque(b, bw, bh, x)).unwrap_or(0);
    let (w, h) = (keep_a + bw - skip_b, ah.max(bh));
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        let row = (y * w * 4) as usize;
        if y < ah {
            let from = (y * aw * 4) as usize;
            rgba[row..row + (keep_a * 4) as usize]
                .copy_from_slice(&a[from..from + (keep_a * 4) as usize]);
        }
        if y < bh {
            let from = ((y * bw + skip_b) * 4) as usize;
            let to = row + (keep_a * 4) as usize;
            let n = ((bw - skip_b) * 4) as usize;
            rgba[to..to + n].copy_from_slice(&b[from..from + n]);
        }
    }
    UndoFace { rgba, w, h }
}

pub type FacePair = [(TexId, u32, u32); 2];

pub struct QuickMenuFaces {
    pub labels: Vec<[(TexId, u32, u32); 2]>,
    /// The controls rows' names on each system's page, see `input_label_faces`.
    pub input_labels: Vec<(&'static str, FacePair)>,
    pub pointer: (TexId, u32, u32),
    pub values: Vec<[(TexId, u32, u32); 2]>,
    pub legend: [(TexId, u32); 11],
    pub legend_light: [(TexId, u32); 11],
    pub descriptions: [(TexId, u32, u32); 2],
}

pub struct QuickMenu<'a> {
    pub row: QuickRow,
    pub hidden: &'a [QuickRow],
    /// slotPlus: rows shown greyed out, which can be reached but not opened.
    pub disabled: &'a [QuickRow],
    /// The system whose controls page this is, which names its rows.
    pub controls: Platform,
    pub values: [Option<QuickValue>; QuickRow::ALL.len()],
    pub clock: Option<[(TexId, u32, u32); 2]>,
    pub wallpaper: Option<[(TexId, u32, u32); 2]>,
    pub networks: Option<[(TexId, u32, u32); 2]>,
    /// slotPlus: the RetroAchievements page's Username, Password and Cheevo Mode values.
    pub achievements: [Option<[(TexId, u32, u32); 2]>; 3],
    pub romm: [Option<[(TexId, u32, u32); 2]>; 4],
    pub faces: Option<&'a QuickMenuFaces>,
    pub ground: Option<(TexId, u32)>,
    pub palette: Option<[(TexId, u32, u32); 2]>,
    pub gba_bezel: Option<[(TexId, u32, u32); 2]>,
    pub gb_bezel: Option<[(TexId, u32, u32); 2]>,
    pub gbc_bezel: Option<[(TexId, u32, u32); 2]>,
    /// The WonderSwan, Lynx, Game Gear, NeoGeo Pocket and TurboGrafx-16 Bezel
    /// rows' values, in that order.
    pub extra_bezels: [Option<[(TexId, u32, u32); 2]>; 6],
    pub bezel: Option<[(TexId, u32, u32); 2]>,
    pub bound: Option<&'a [Option<FacePair>]>,
    pub description: Option<(TexId, u32, u32)>,
    pub over_game: bool,
    /// Where the sliding footer starts. The scrim stops there so it does not darken the strip.
    pub footer_top: f32,
    pub canvas: Canvas,
}

impl QuickMenu<'_> {
    pub fn draw(&self, out: &mut Vec<Draw>) {
        // Rows are laid out on the band. On a taller canvas a page's rows sit in its
        // middle, and the play-screen panel over a game at its bottom.
        let rows_dy = match self.row.page() == QuickRow::PLAY_SCREEN {
            true => self.canvas.extra(),
            false => self.canvas.extra() / 2.0,
        };
        match (self.over_game, self.row.page() == QuickRow::PLAY_SCREEN) {
            (false, _) => draw_menu_ground(self.canvas, self.ground, out),
            (true, panel) => {
                let y = match panel {
                    true => {
                        quick_row_top(self.row.page()[0], self.hidden, self.canvas) - PANEL_GAP
                            + rows_dy
                    }
                    false => 0.0,
                };
                let bottom = self.canvas.below(self.footer_top).max(y);
                out.push(Draw::Rect {
                    x: 0.0,
                    y,
                    w: self.canvas.w,
                    h: bottom - y,
                    colour: [0.0, 0.0, 0.0, GROUND_SCRIM],
                });
            }
        }
        let Some(faces) = self.faces else {
            return;
        };
        let label = |row: QuickRow| {
            let own = row.input().and_then(|i| {
                let name = controls::label(self.controls, i);
                faces.input_labels.iter().find(|(n, _)| *n == name)
            });
            match own {
                Some((_, face)) => Some(face),
                None => faces.labels.get(row.index()),
            }
        };
        let (right, pad) = (self.canvas.w - QUICK_EDGE, MENU_PAD as f32);
        // Slot²: the widest row shown, so a hidden row doesn't pull the column aside.
        let main_w = self
            .row
            .shown(self.hidden)
            .iter()
            .filter_map(|r| label(*r))
            .map(|l| l[1].1)
            .max()
            .unwrap_or(0);
        let shown = self.row.shown(self.hidden);
        let (start, len) = quick_window(self.row, self.hidden, self.canvas);
        let first = out.len();
        for row in shown.into_iter().skip(start).take(len) {
            let y = quick_row_top_at(row, self.row, self.hidden, self.canvas) + TYPE_DROP;
            let lit = row == self.row;
            // slotPlus: a greyed-out row keeps its dim face, faded, even under the pointer.
            let greyed = self.disabled.contains(&row);
            let mut label_end = QUICK_LABEL_X;
            if let Some(&(tex, w, h)) = label(row).map(|l| &l[(lit && !greyed) as usize]) {
                let x = match row.centred() {
                    true => self.canvas.centred(main_w as f32),
                    false => QUICK_LABEL_X - pad,
                };
                label_end = x + w as f32 - pad;
                out.push(Draw::Tex {
                    x: x.round(),
                    y: y.round(),
                    w: w as f32,
                    h: h as f32,
                    tex,
                    alpha: if greyed { GREYED_ALPHA } else { 1.0 },
                });
                if lit {
                    let (tex, pw, ph) = faces.pointer;
                    let px = x + pad - POINTER_GAP - pw as f32;
                    push(out, tex, px, y - POINTER_BLEED as f32, pw, ph);
                }
            }
            let value = match row {
                QuickRow::DateTime => self.clock.map(|c| c[lit as usize]),
                QuickRow::Wallpaper => self.wallpaper.map(|c| c[lit as usize]),
                QuickRow::Wifi | QuickRow::Networks => self.networks.map(|c| c[lit as usize]),
                QuickRow::Romm => self.romm[0].map(|c| c[lit as usize]),
                QuickRow::RommServer | QuickRow::RommPairedServer => {
                    self.romm[1].map(|c| c[lit as usize])
                }
                QuickRow::RommPort => self.romm[2].map(|c| c[lit as usize]),
                QuickRow::RommStatus => self.romm[3].map(|c| c[lit as usize]),
                QuickRow::RaUsername => self.achievements[0].map(|c| c[lit as usize]),
                QuickRow::RaPassword => self.achievements[1].map(|c| c[lit as usize]),
                // Casual is the only mode, so its value stays dim even when lit.
                QuickRow::RaMode => self.achievements[2].map(|c| c[0]),
                QuickRow::Palette | QuickRow::GbPalette => self.palette.map(|c| c[lit as usize]),
                QuickRow::GbaBezel => self.gba_bezel.map(|c| c[lit as usize]),
                QuickRow::GbBezel => self.gb_bezel.map(|c| c[lit as usize]),
                QuickRow::GbcBezel => self.gbc_bezel.map(|c| c[lit as usize]),
                QuickRow::WsBezel => self.extra_bezels[0].map(|c| c[lit as usize]),
                QuickRow::LynxBezel => self.extra_bezels[1].map(|c| c[lit as usize]),
                QuickRow::GgBezel => self.extra_bezels[2].map(|c| c[lit as usize]),
                QuickRow::NgpBezel => self.extra_bezels[3].map(|c| c[lit as usize]),
                QuickRow::Tg16Bezel => self.extra_bezels[4].map(|c| c[lit as usize]),
                QuickRow::NdsBezel => self.extra_bezels[5].map(|c| c[lit as usize]),
                QuickRow::Bezel => self.bezel.map(|c| c[lit as usize]),
                _ if row.binds() => self
                    .bound
                    .and_then(|b| b.get(row.index()).copied().flatten())
                    .map(|c| c[lit as usize]),
                _ => self.values[row.index()]
                    .and_then(|v| faces.values.get(v.index()))
                    .map(|v| v[lit as usize]),
            };
            let Some((tex, w, h)) = value else {
                continue;
            };
            let room = right - label_end - QUICK_VALUE_GAP;
            let s = (room / (w as f32 - 2.0 * pad)).clamp(VALUE_MIN_SCALE, 1.0);
            let (sw, sh) = (w as f32 * s, h as f32 * s);
            out.push(Draw::Tex {
                x: (right - (w as f32 - pad) * s).round(),
                y: (y + (h as f32 - sh) / 2.0).round(),
                w: sw,
                h: sh,
                tex,
                alpha: 1.0,
            });
        }
        if let Some(bar) = quick_scroll(self.row, self.hidden, self.canvas) {
            let x = self.canvas.w - SCROLL_INSET - SCROLL_W;
            let ink = DIM_INK.map(|c| f32::from(c) / 255.0);
            out.push(Draw::Rect {
                x,
                y: bar.y,
                w: SCROLL_W,
                h: bar.h,
                colour: [ink[0], ink[1], ink[2], 0.3],
            });
            let lit = crate::hud::HUD_INK.map(|c| f32::from(c) / 255.0);
            out.push(Draw::Rect {
                x,
                y: bar.thumb_y,
                w: SCROLL_W,
                h: bar.thumb_h,
                colour: [lit[0], lit[1], lit[2], 1.0],
            });
        }
        shift(&mut out[first..], rows_dy);
        if let Some((tex, w, h)) = self.description {
            let y = self
                .canvas
                .below(OUT_H as f32 - MOUTH_H - h as f32 - DESCRIPTION_GAP);
            push(out, tex, self.canvas.centred(w as f32), y, w, h);
        }
    }
}

pub fn draw_menu_ground(canvas: Canvas, ground: Option<(TexId, u32)>, out: &mut Vec<Draw>) {
    let (w, h) = (canvas.w, canvas.h);
    match ground {
        Some((tex, tex_w)) => {
            out.push(Draw::Part {
                x: 0.0,
                y: 0.0,
                w,
                h,
                tex,
                alpha: 1.0,
                uv: canvas.cover(tex_w as f32),
            });
            out.push(Draw::Rect {
                x: 0.0,
                y: 0.0,
                w,
                h,
                colour: [0.0, 0.0, 0.0, GROUND_SCRIM],
            });
        }
        None => out.push(Draw::Rect {
            x: 0.0,
            y: 0.0,
            w,
            h,
            colour: opening(),
        }),
    }
}

/// slotPlus: how faint a greyed-out row is drawn.
const GREYED_ALPHA: f32 = 0.4;

pub const QUICK_VISIBLE: usize = 6;

/// How many rows a scrolling list shows: upstream's 6 on the 480-unit band, more on a
/// taller canvas, keeping the band's margins above the rows and below them to the legend
/// (10 on a square panel).
pub fn quick_visible(canvas: Canvas) -> usize {
    let margin = (LEGEND_Y - QUICK_PITCH * QUICK_VISIBLE as f32) / 2.0;
    let room = LEGEND_Y + canvas.extra() - 2.0 * margin;
    ((room / QUICK_PITCH).floor() as usize).max(QUICK_VISIBLE)
}

const PANEL_VISIBLE: usize = 3;

pub(crate) const SCROLL_INSET: f32 = 12.0;

pub(crate) const SCROLL_W: f32 = 4.0;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct QuickScroll {
    pub y: f32,
    pub h: f32,
    pub thumb_y: f32,
    pub thumb_h: f32,
}

fn scrolls(page: &[QuickRow]) -> bool {
    [
        &QuickRow::CONTROLS[..],
        &QuickRow::SHORTCUTS,
        &QuickRow::GENERAL,
        &QuickRow::PLAY_SCREEN,
        // General and eight consoles are more rows than fit above the legend.
        &QuickRow::SCREEN,
    ]
    .contains(&page)
}

fn visible(page: &[QuickRow], canvas: Canvas) -> usize {
    match page == QuickRow::PLAY_SCREEN {
        true => PANEL_VISIBLE,
        false => quick_visible(canvas),
    }
}

pub fn quick_window(selected: QuickRow, hidden: &[QuickRow], canvas: Canvas) -> (usize, usize) {
    let count = selected.shown(hidden).len();
    let fits = visible(selected.page(), canvas);
    if !scrolls(selected.page()) || count <= fits {
        return (0, count);
    }
    let at = selected.position(hidden);
    let start = at.saturating_sub(fits / 2).min(count - fits);
    (start, fits)
}

pub fn quick_scroll(
    selected: QuickRow,
    hidden: &[QuickRow],
    canvas: Canvas,
) -> Option<QuickScroll> {
    let shown = selected.shown(hidden);
    let (start, len) = quick_window(selected, hidden, canvas);
    if len >= shown.len() {
        return None;
    }
    let y = quick_row_top_at(shown[start], selected, hidden, canvas);
    let h = QUICK_PITCH * len as f32;
    let thumb_h = h / shown.len() as f32;
    let along = selected.position(hidden) as f32 / (shown.len() - 1) as f32;
    Some(QuickScroll {
        y,
        h,
        thumb_y: y + (h - thumb_h) * along,
        thumb_h,
    })
}

pub fn quick_row_top_at(
    row: QuickRow,
    selected: QuickRow,
    hidden: &[QuickRow],
    canvas: Canvas,
) -> f32 {
    let (start, len) = quick_window(selected, hidden, canvas);
    let held = match row.page() == QuickRow::WIFI {
        true => QuickRow::WIFI.len(),
        false => len,
    };
    let rows = QUICK_PITCH * held as f32;
    let top = match row.page() == QuickRow::PLAY_SCREEN {
        true => OUT_H as f32 - MOUTH_H - PANEL_GAP - rows,
        false => (LEGEND_Y - rows) / 2.0,
    };
    top + QUICK_PITCH * (row.position(hidden) as f32 - start as f32)
}

pub fn quick_row_top(row: QuickRow, hidden: &[QuickRow], canvas: Canvas) -> f32 {
    quick_row_top_at(row, row, hidden, canvas)
}

fn push(out: &mut Vec<Draw>, tex: TexId, x: f32, y: f32, w: u32, h: u32) {
    out.push(Draw::Tex {
        x: x.round(),
        y: y.round(),
        w: w as f32,
        h: h as f32,
        tex,
        alpha: 1.0,
    });
}

#[cfg(test)]
mod layout_tests {
    use super::*;

    #[test]
    fn the_screen_page_scrolls_its_general_row_and_eight_consoles_above_the_legend() {
        assert!(scrolls(&QuickRow::SCREEN));
        assert!(QuickRow::SCREEN.len() > QUICK_VISIBLE);
        // Whichever row is chosen, it sits in a window that fits between the top and the legend.
        for row in QuickRow::SCREEN {
            let top = quick_row_top(row, &[], Canvas::FRAMED);
            assert!(
                top >= 0.0 && top + QUICK_PITCH <= LEGEND_Y,
                "{row:?} at {top}"
            );
        }
    }
}
