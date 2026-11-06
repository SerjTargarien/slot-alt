# Core options: what each core offers, what Slot² uses

Status: surveyed with the user's ask on 2026-11-06; chosen the same day ("Chosen"), not
built yet.

## What the user asked for

Find and document every per-core setting the cores we use expose, and choose which to
use without bloating the interface. Only the Game Boy family's cores take settings from
menus today (colour correction, frame blending, palette); others may have settings worth
having.

## How this was surveyed

Each core's source at its pin in `taskfile.yml` (the cores Slot² runs, see
[cores/README.md](../cores/README.md)), its declared options read from its
`libretro_core_options.h` (DSperate's wrapper from `src/libretro.cpp`). "Slot² sets" is
`crates/slot/src/core.rs` (`apply_core_options`, `init_options`, `colour_option`,
`blend_option`, `palette_options`) and the session's live options.

- **DSperate** (wrapper `093d50f`) declares one option, `dsperate_jit` (enabled|disabled),
  which Slot² sets enabled; Slot²'s patch adds `dsperate_frameskip`.
- **Beetle NeoPop** declares one, `ngp_language`, which Slot² sets to english.

## Every option, by core

### mGBA (`e31759b`)

| Option | What | Values (default **bold**) | Slot² sets |
|---|---|---|---|
| `mgba_gb_model` | Game Boy Model (Restart) | **Autodetect**, Game Boy, Super Game Boy, Game Boy Color, Super Game Boy Color, Game Boy Advance | per game (palette) |
| `mgba_use_bios` | Use BIOS File if Found (Restart) | **ON**, OFF | (core default) |
| `mgba_skip_bios` | Skip BIOS Intro (Restart) | **OFF**, ON |  |
| `mgba_gb_colors` | Default Game Boy Palette | **Grayscale** | Palette row |
| `mgba_gb_colors_preset` | Hardware Preset Game Boy Palettes (Restart) | **0**, 1, 2, 3 | per game (palette) |
| `mgba_sgb_borders` | Use Super Game Boy Borders (Restart) | **ON**, OFF | OFF |
| `mgba_color_correction` | Color Correction | **OFF**, GBA, GBC, Auto | Color Correction row |
| `mgba_interframe_blending` | Interframe Blending | **OFF**, mix, mix_smart, lcd_ghosting, lcd_ghosting_fast | Frame Blending row |
| `mgba_audio_low_pass_filter` | Audio Filter | **disabled**, enabled |  |
| `mgba_audio_low_pass_range` | Audio Filter Level | 5, 10, 15, 20, 25, 30, 35, 40 … (19) |  |
| `mgba_allow_opposing_directions` | Allow Opposing Directional Input | **no**, yes |  |
| `mgba_solar_sensor_level` | Solar Sensor Level | sensor, **0**, 1, 2, 3, 4, 5, 6 … (12) | Sunlight shortcuts |
| `mgba_force_gbp` | Game Boy Player Rumble (Restart) | **OFF**, ON | per game (Game Boy Player) |
| `mgba_idle_optimization` | Idle Loop Removal | **Remove Known**, Detect and Remove, Don't Remove |  |
| `mgba_frameskip` | Frameskip | **disabled**, auto, auto_threshold, fixed_interval | auto |
| `mgba_frameskip_threshold` | Frameskip Threshold (%) | 15, 18, 21, 24, 27, 30, **33**, 36 … (16) |  |
| `mgba_frameskip_interval` | Frameskip Interval | **0**, 1, 2, 3, 4, 5, 6, 7 … (11) |  |

### gpSP (`7b2dd37`)

| Option | What | Values (default **bold**) | Slot² sets |
|---|---|---|---|
| `gpsp_bios` | BIOS | **auto**, builtin, official |  |
| `gpsp_boot_mode` | Boot mode | **game**, bios | bios when one is on the card |
| `gpsp_drc` | Dynamic Recompiler | **enabled**, disabled |  |
| `gpsp_sprlim` | No Sprite Limit | **disabled**, enabled |  |
| `gpsp_rtc` | RTC support | **auto**, enabled, disabled |  |
| `gpsp_rtc_time_source` | RTC Time Source | **deterministic**, system | system |
| `gpsp_serial` | Link Cable Connectivity | **auto**, disabled, rfu, mul_poke, mul_aw1, mul_aw2 | per game (link) |
| `gpsp_rumble` | Rumble support | **auto**, enabled, disabled |  |
| `gpsp_sound_rate` | Sound Output Rate (Hz) | **65536**, 32768 |  |
| `gpsp_frameskip` | Frameskip | **disabled**, auto, auto_threshold, fixed_interval | auto |
| `gpsp_frameskip_threshold` | Frameskip Threshold (%) | 15, 18, 21, 24, 27, 30, **33**, 36 … (16) |  |
| `gpsp_frameskip_interval` | Frameskip Interval | 0, **1**, 2, 3, 4, 5, 6, 7 … (11) |  |
| `gpsp_color_correction` | Color Correction | enabled, **disabled** | Color Correction row |
| `gpsp_frame_mixing` | Interframe Blending | enabled, **disabled** | Frame Blending row |
| `gpsp_turbo_period` | Turbo Button Period | **4**, 5, 6, 7, 8, 9, 10, 11 … (117) |  |

### Gambatte (`77f2567`)

| Option | What | Values (default **bold**) | Slot² sets |
|---|---|---|---|
| `gambatte_gb_colorization` | GB Colorization | **disabled**, auto, GBC, SGB, internal, custom | Palette row |
| `gambatte_gb_internal_palette` | Internal Palette | **GB - DMG**, GB - Pocket, GB - Light, GBC - Blue, GBC - Brown, GBC - Dark Blue, GBC - Dark Brown, GBC - Dark Green … (55) | Palette row |
| `gambatte_gbc_color_correction` | Color Correction | **GBC only**, always, disabled | Color Correction row |
| `gambatte_gbc_color_correction_mode` | Color Correction Mode | **accurate**, fast |  |
| `gambatte_gbc_frontlight_position` | Color Correction - Frontlight Position | **central**, above screen, below screen |  |
| `gambatte_dark_filter_level` | Dark Filter Level (%) | **0**, 5, 10, 15, 20, 25, 30, 35 … (11) |  |
| `gambatte_mix_frames` | Interframe Blending | **disabled**, mix, lcd_ghosting, lcd_ghosting_fast | Frame Blending row |
| `gambatte_audio_resampler` | Audio Resampler | **sinc**, cc |  |
| `gambatte_gb_hwmode` | Emulated Hardware (Restart Required) | **Auto**, GB, GBC, GBA | Auto |
| `gambatte_gb_bootloader` | Use Official Bootloader (Restart Required) | **enabled**, disabled | enabled |
| `gambatte_up_down_allowed` | Allow Opposing Directions | **disabled**, enabled |  |
| `gambatte_turbo_period` | Turbo Button Period | **4**, 5, 6, 7, 8, 9, 10, 11 … (117) |  |
| `gambatte_rumble_level` | Controller Rumble Strength | 0, 1, 2, 3, 4, 5, 6, 7 … (11) | 10 |
| `gambatte_show_gb_link_settings` | Show Game Link Settings | enabled, **disabled** |  |
| `gambatte_gb_link_mode` | Game Link Mode | **Not Connected**, Network Server, Network Client |  |
| `gambatte_gb_link_network_port` | Network Link Port | **56400**, 56401, 56402, 56403, 56404, 56405, 56406, 56407 … (21) |  |
| `gambatte_gb_palette_*` | 4 palette packs (TWB64, PixelShift) | many | |
| `gambatte_gb_link_network_*` | network link mode, port and the server address, a digit an option (12) | | (Slot² links on mGBA) |

### Beetle Cygne (`4b01295`)

| Option | What | Values (default **bold**) | Slot² sets |
|---|---|---|---|
| `wswan_language` | Language (Restart Required) | **english**, japanese |  |
| `wswan_name_source` | Owner Name Source (Restart Required) | **frontend**, default |  |
| `wswan_birth_year` | Owner Birth Year (Restart Required) | 1920, 1921, 1922, 1923, 1924, 1925, 1926, 1927 … (110) |  |
| `wswan_birth_month` | Owner Birth Month (Restart Required) | 1, 2, 3, 4, 5, **6**, 7, 8 … (12) |  |
| `wswan_birth_day` | Owner Birth Day (Restart Required) | 1, 2, 3, 4, 5, 6, 7, 8 … (31) |  |
| `wswan_sex` | Owner Sex (Restart Required) | **female**, male |  |
| `wswan_blood` | Owner Blood Type (Restart Required) | **o**, a, b, ab |  |
| `wswan_rotate_display` | Display Rotation | **manual**, landscape, portrait | per game (Rotate, LCD icons) |
| `wswan_rotate_keymap` | Rotate Button Mappings | **auto**, disabled, enabled | disabled |
| `wswan_mono_palette` | Color Palette | **default**, wonderswan, wondeswan_color, swancrystal, gb_dmg, gb_pocket, gb_light, blossom_pink … (27) |  |
| `wswan_gfx_colors` | Color Depth (Restart Required) | **16bit**, 24bit |  |
| `wswan_frameskip` | Frameskip | **disabled**, auto, manual | auto |
| `wswan_frameskip_threshold` | Frameskip Threshold (%) | 15, 18, 21, 24, 27, 30, **33**, 36 … (16) |  |
| `wswan_60hz_mode` | 60Hz Mode | disabled, **enabled** |  |
| `wswan_sound_sample_rate` | Sound Output Sample Rate | 11025, 22050, **44100**, 48000 |  |
| `wswan_sound_low_pass` | Audio Filter | **disabled**, enabled |  |

### Beetle Lynx (`fcdefcf`)

| Option | What | Values (default **bold**) | Slot² sets |
|---|---|---|---|
| `lynx_rot_screen` | Auto-rotate Screen | **auto**, manual, 0, 90, 180, 270 | per game (hold picker) |
| `lynx_pix_format` | Color Format (Restart Required) | **16**, 32 |  |
| `lynx_force_60hz` | Force 60Hz | **disabled**, enabled |  |

### Gearsystem (`517ddaa`)

| Option | What | Values (default **bold**) | Slot² sets |
|---|---|---|---|
| `gearsystem_system` | System (restart) | **Auto**, Master System / Mark III, Game Gear (2 ASIC), Game Gear (2 ASIC) SMS Mode, Game Gear (1 ASIC), Game Gear (1 ASIC) SMS Mode, SG-1000 / Multivision, SG-1000 II |  |
| `gearsystem_region` | Region (restart) | **Auto**, Master System Japan, Master System Export, Game Gear Japan, Game Gear Export, Game Gear International |  |
| `gearsystem_mapper` | Mapper (restart) | **Auto**, ROM, SEGA, Codemasters, Korean, SG-1000, MSX, Janggun … (24) |  |
| `gearsystem_timing` | Refresh Rate (restart) | **Auto**, NTSC (60 Hz), PAL (50 Hz) |  |
| `gearsystem_bios_sms` | Master System BIOS (restart) | **Disabled**, Enabled | Disabled |
| `gearsystem_bios_gg` | Game Gear BIOS (restart) | **Disabled**, Enabled | Disabled |
| `gearsystem_aspect_ratio` | Aspect Ratio | **1:1 PAR**, 4:3 DAR, 16:9 DAR, 16:10 DAR | 1:1 PAR |
| `gearsystem_overscan` | Overscan | **Disabled**, Top+Bottom, Full (284 width), Full (320 width) | Disabled |
| `gearsystem_hide_left_bar` | Hide Left Bar (SMS only) | **No**, Auto, Always |  |
| `gearsystem_no_sprite_limit` | No Sprite Limit | **Disabled**, Enabled |  |
| `gearsystem_glasses` | 3D Glasses | **Both Eyes / OFF**, Left Eye, Right Eye |  |
| `gearsystem_ym2413` | YM2413 (restart) | **Auto**, Disabled |  |
| `gearsystem_psg_volume` | PSG Volume | 0, 10, 20, 30, 40, 50, 60, 70 … (21) |  |
| `gearsystem_fm_volume` | FM Volume | 0, 10, 20, 30, 40, 50, 60, 70 … (21) |  |
| `gearsystem_up_down_allowed` | Allow Up+Down / Left+Right | **Disabled**, Enabled |  |
| `gearsystem_lightgun_input` | Light Gun Input | **Light Gun**, Touchscreen |  |
| `gearsystem_lightgun_crosshair` | Light Gun Crosshair | **Disabled**, Enabled |  |
| `gearsystem_lightgun_shape` | Light Gun Crosshair Shape | **Cross**, Square |  |
| `gearsystem_lightgun_color` | Light Gun Crosshair Color | **White**, Black, Red, Green, Blue, Yellow, Magenta, Cyan |  |
| `gearsystem_lightgun_crosshair_offset_x` | Light Gun Crosshair Offset X | -10, -9, -8, -7, -6, -5, -4, -3 … (21) |  |
| `gearsystem_lightgun_crosshair_offset_y` | Light Gun Crosshair Offset Y | -10, -9, -8, -7, -6, -5, -4, -3 … (21) |  |
| `gearsystem_paddle_sensitivity` | Paddle Sensitivity | **1**, 2, 3, 4, 5, 6, 7, 8 … (15) |  |
| `gearsystem_sports_pad_sensitivity` | Sports Pad Sensitivity | 1, 2, 3, 4, 5, 6, 7, **8** … (15) |  |
| `gearsystem_link_enable` | Game Link Cable Enable (restart) | Enabled, **Disabled** | Disabled |
| `gearsystem_link_placement` | Dual Screen Placement | **Horizontal**, Vertical |  |
| `gearsystem_link_switch` | Dual Screen Switch | Enabled, **Disabled** |  |
| `gearsystem_link_screen` | Dual Screen Selection | **Both Screens**, Screen 1, Screen 2 |  |
| `gearsystem_link_audio` | Dual Screen Audio | **Screen 1**, Screen 2, Mix |  |

### Beetle NeoPop (`a50d5ac`)

| Option | What | Values (default **bold**) | Slot² sets |
|---|---|---|---|
| `ngp_language` | Language (*) | **english**, japanese | english |

### Beetle PCE Fast (`3f946f2`)

| Option | What | Values (default **bold**) | Slot² sets |
|---|---|---|---|
| `pce_fast_palette` | Color Palette | **RGB**, Composite |  |
| `pce_fast_frameskip` | Frameskip | **disabled**, auto, manual | auto |
| `pce_fast_frameskip_threshold` | Frameskip Threshold (%) | 15, 18, 21, 24, 27, 30, **33**, 36 … (16) |  |
| `pce_fast_hoverscan` | Horizontal Overscan (352 Width Mode Only) | 300, 302, 304, 306, 308, 310, 312, 314 … (27) |  |
| `pce_fast_initial_scanline` | Initial Scanline | 0, 1, 2, **3**, 4, 5, 6, 7 … (41) |  |
| `pce_fast_last_scanline` | Last Scanline | 208, 209, 210, 211, 212, 213, 214, 215 … (35) |  |
| `pce_fast_sound_channel_0_volume` | PSG Sound Channel 0 Volume % | 0, 10, 20, 30, 40, 50, 60, 70 … (11) |  |
| `pce_fast_sound_channel_1_volume` | PSG Sound Channel 1 Volume % | 0, 10, 20, 30, 40, 50, 60, 70 … (11) |  |
| `pce_fast_sound_channel_2_volume` | PSG Sound Channel 2 Volume % | 0, 10, 20, 30, 40, 50, 60, 70 … (11) |  |
| `pce_fast_sound_channel_3_volume` | PSG Sound Channel 3 Volume % | 0, 10, 20, 30, 40, 50, 60, 70 … (11) |  |
| `pce_fast_sound_channel_4_volume` | PSG Sound Channel 4 Volume % | 0, 10, 20, 30, 40, 50, 60, 70 … (11) |  |
| `pce_fast_sound_channel_5_volume` | PSG Sound Channel 5 Volume % | 0, 10, 20, 30, 40, 50, 60, 70 … (11) |  |
| `pce_fast_mouse_sensitivity` | Mouse Sensitivity | 0.25, 0.50, 0.75, 1.00, **1.25**, 1.50, 1.75, 2.00 … (20) |  |
| `pce_fast_disable_softreset` | Disable Soft Reset (RUN+SELECT) | **disabled**, enabled |  |
| `pce_fast_default_joypad_type_p1` | P1 Default Joypad Type | **2 Buttons**, 6 Buttons | 2 Buttons |
| `pce_fast_default_joypad_type_p2` | P2 Default Joypad Type | **2 Buttons**, 6 Buttons |  |
| `pce_fast_default_joypad_type_p3` | P3 Default Joypad Type | **2 Buttons**, 6 Buttons |  |
| `pce_fast_default_joypad_type_p4` | P4 Default Joypad Type | **2 Buttons**, 6 Buttons |  |
| `pce_fast_default_joypad_type_p5` | P5 Default Joypad Type | **2 Buttons**, 6 Buttons |  |
| `pce_fast_turbo_toggling` | Turbo Toggle | **disabled**, enabled |  |
| `pce_fast_turbo_toggle_hotkey` | Alternate Turbo Hotkey | **disabled**, enabled |  |
| `pce_fast_turbo_delay` | Turbo Delay | 1, 2, **3**, 4, 5, 6, 7, 8 … (15) |  |
| `pce_fast_cdimagecache` | CD Image Cache (Restart Required) | **disabled**, enabled |  |
| `pce_fast_cdbios` | CD BIOS (Restart Required) | Games Express, System Card 1, System Card 2, **System Card 3**, System Card 2 US, System Card 3 US |  |
| `pce_fast_cdspeed` | (CD) CD Speed | **1**, 2, 4, 8 |  |
| `pce_fast_adpcmvolume` | (CD) ADPCM Volume % | 0, 10, 20, 30, 40, 50, 60, 70 … (21) |  |
| `pce_fast_adpcmlp` | (CD) ADPCM Low-pass Filter | **disabled**, enabled |  |
| `pce_fast_cddavolume` | (CD) CDDA Volume % | 0, 10, 20, 30, 40, 50, 60, 70 … (21) |  |
| `pce_fast_cdpsgvolume` | (CD) PSG Volume % | 0, 10, 20, 30, 40, 50, 60, 70 … (21) |  |
| `pce_fast_cdignoreerrors` | CD Ignore EDC/L-EC Errors | **disabled**, enabled |  |
| `pce_fast_nospritelimit` | No Sprite Limit | **disabled**, enabled |  |
| `pce_fast_ocmultiplier` | CPU Overclock Multiplier (Restart Required) | **1**, 2, 3, 4, 5, 6, 7, 8 … (14) |  |

## What Slot² does with them now

- **Menus:** the Game Boy family only. Palette (Gambatte, mGBA), Color Correction (mGBA,
  gpSP, Gambatte), Frame Blending (mGBA, gpSP, Gambatte; gpSP has no ghosting).
- **Per game, by Slot²:** the GBA link mode (gpSP serial, mGBA link), the Game Boy Player
  rumble (mGBA), the Lynx's hold (`lynx_rot_screen`), the WonderSwan's turn
  (`wswan_rotate_display`), Sunlight (`mgba_solar_sensor_level`).
- **Fixed, for the device:** frameskip `auto` where a core has one; no BIOS on the Game
  Gear; the Game Gear's bare screen at square pixels; one-console Gearsystem; the
  TurboExpress's two-button pad; English on the NeoGeo Pocket; the WonderSwan's landscape
  keymap (Slot² maps the buttons); the GBA's real-time clock from the system.
- **The 75 Hz systems:** the WonderSwan core drops one frame in five to show 60 Hz
  (`wswan_60hz_mode`, on by default); the Lynx core runs its 75 Hz, and Slot² runs 1.25 of
  its frames a present (`emu::Owed`), so games keep their speed but not every present
  moves the same.

## Chosen (2026-11-06)

- **WonderSwan palette** and **TurboGrafx-16 colours**, in the Palette row (1, 2 below):
  a value per system on the shelf's Screen pages, the game's own in game, like the rest of
  the Screen settings.
- **The sprite limit row** (3), a Gameplay setting: the shelf's value, the game's own in
  game; offered only where the running core has it.
- **Lynx at 60 Hz and gpSP at 32768 Hz** (4, 5): measured on the device first, and set by
  Slot² (no row) if they help.
- **The audio filters** (6) are noted for the audio diagnosis, as things to try there.

Build order: the palettes, then the sprite limit, then the measurements.

## Candidates

Kept to what changes how a game looks, sounds or plays on this device, and what fits a
row that is already there.

### In rows that exist (no new rows)

1. **WonderSwan palette** (`wswan_mono_palette`, 27 palettes: WonderSwan, SwanCrystal, Game
   Boy, Virtual Boy and others): the Screen page's Palette row for a WonderSwan (mono)
   game, as the Game Boy's for a DMG game. A WonderSwan Color game shows no row.
2. **TurboGrafx-16 colours** (`pce_fast_palette`, RGB or Composite, "recreates the original
   console output"): the Palette row for the TurboGrafx-16, two values.

### One new row, if any

3. **Sprite limit** (Gameplay): off lifts the hardware's per-line sprite limit, so games
   flicker less, at the risk of glitches where a game relied on it. Three cores have it:
   gpSP (`gpsp_sprlim`), Gearsystem (`gearsystem_no_sprite_limit`), PCE Fast
   (`pce_fast_nospritelimit`); mGBA, Gambatte, Cygne, Lynx and NeoPop don't. Best per game
   (it's a per-game fix), and the same kind of note as gpSP's ghosting for the others.

### Set by Slot², no row, after measuring on the device

4. **Lynx at 60 Hz** (`lynx_force_60hz`): "perfectly smooth movement on 60 Hz displays".
   Measure against today's 1.25 frames a present.
5. **gpSP sound at 32768 Hz** (`gpsp_sound_rate`): upstream's new default ("halves audio
   mixing work", matches the hardware's own output). Speed on the H700.
6. **Audio filters:** mGBA's low-pass filter (`mgba_audio_low_pass_filter`, with a level),
   the WonderSwan's (`wswan_sound_low_pass`, "softens the sometimes harsh chiptunes"), and
   Gambatte's resampler (`gambatte_audio_resampler`, sinc or the cheaper cosine). These
   belong with the audio diagnosis on the roadmap.

### Left alone

- Owner profile and language (Cygne), BIOS and boot (set or not on the card), link modes
  (Slot² runs its own), turbo (gone as an input), light gun, paddle and sports pad
  (Master System), 3D glasses (Master System), the CD options (no CD games), extra pads,
  opposing directions, frameskip tuning (Slot² sets `auto`), the PCE overclock ("can cause
  glitches and crashes"), mGBA's other blending modes (`mix_smart`, `lcd_ghosting_fast`),
  Gambatte's colour-correction mode and frontlight, and its dark filter (Brightness covers
  it).
