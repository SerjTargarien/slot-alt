# Core map

The emulator cores Slot² runs, where each comes from, and how current it is. Checked on
2026-11-06 against GitHub (upstream counts 2026-11-07), slotPlus's release, and the card in the RGB20SX.

## At a glance

| System | Core (file) | Source | Pinned at | Upstream since |
|---|---|---|---|---|
| Game Boy Advance | mGBA (`mgba`), default | [libretro/mgba](https://github.com/libretro/mgba) | `e31759b`, 2026-08-06 | 1 commit (log noise) |
| Game Boy Advance | gpSP (`gpsp`), pick | [libretro/gpsp](https://github.com/libretro/gpsp) | `7b2dd37`, 2026-08-19 | 9 commits (fixes) |
| Game Boy, Game Boy Color | Gambatte (`gambatte`), default | [libretro/gambatte-libretro](https://github.com/libretro/gambatte-libretro) | `77f2567`, 2026-10-30 | none |
| Game Boy, Game Boy Color | mGBA (`mgba`), pick and link play | as above | | |
| WonderSwan (Color) | Beetle Cygne (`cygne`), Slot²'s build | [libretro/beetle-wswan-libretro](https://github.com/libretro/beetle-wswan-libretro) | `4b01295`, 2026-07-31 | none |
| Atari Lynx | Beetle Lynx (`lynx`) | [libretro/beetle-lynx-libretro](https://github.com/libretro/beetle-lynx-libretro) | `fcdefcf`, 2026-04-20 | none |
| Game Gear | Gearsystem (`gearsystem`) | [drhelius/Gearsystem](https://github.com/drhelius/Gearsystem) | `517ddaa`, 3.9.22, 2026-10-11 | 57 commits, releases 3.9.23 and 3.9.24 |
| NeoGeo Pocket (Color) | Beetle NeoPop (`ngp`) | [libretro/beetle-ngp-libretro](https://github.com/libretro/beetle-ngp-libretro) | `a50d5ac`, 2026-06-14 | none |
| TurboGrafx-16 | Beetle PCE Fast (`pce`) | [libretro/beetle-pce-fast-libretro](https://github.com/libretro/beetle-pce-fast-libretro) | `3f946f2`, 2026-10-02 | 2 commits (translations) |

"Default" is what a game runs on until another core is picked for it (START on the
shelf opens the cart to its chip, saved per game in `Config/selected_core.txt`). The Game
Boy's default is the card's choice, `gb_core` in `slot.state`: Gambatte unless set. Link
play runs on mGBA only; a Game Boy game on Gambatte is told it needs mGBA.

## Where the files on the card come from

The `System/*_libretro.so` on the card are **slotPlus's builds, byte for byte**, but for
Beetle Cygne, which Slot² builds itself since 2026-11-06 (below). Each of slotPlus's
matches the same file in slotPlus's
[v1.14.0-plus2.6 release](https://github.com/nelynes/slotPlus/releases/tag/v1.14.0-plus2.6)
(`slot-plusultra.zip`, 2026-11-04), checked by MD5. That release carries a build record for
each core, `System/licenses/<core>-<commit>.meta`, and every record matches this repo's
pins in `taskfile.yml` and the patches in `cores/*/`.

- mGBA, gpSP and Gambatte, and their pins, are upstream slot's (BrandonKowalski/slot,
  unchanged at `f1f6ef9` and on its main branch today). slotPlus builds them the same way.
- The other six, and their recipes, are slotPlus's, from its `slotPlusUltra` branch
  (`a61ccfe`), which Slot² merged on 2026-11-05.
- `System/premerge/` on the card holds the three cores of the slot build from before the
  merge.

Slot² builds one core itself, Beetle Cygne (2026-11-06), for its patch; the others are
still slotPlus's builds.

## How they are built

`task core:<name>` (`core:device` for mGBA) runs `cores/<name>/build.sh` in a Debian
bullseye arm64 box (`docker run rust:1-bullseye`; natively on an aarch64 Linux host).
Each recipe:

- fetches its pinned commit as a source archive, which ships beside the core as its
  corresponding source (GPL), with a `.meta` stamp of commit, recipe, flags and patches;
- applies the patches in `cores/<name>/`;
- builds for `-mcpu=cortex-a53` (the RK3566's Cortex-A55 runs it), with LTO for mGBA and
  Gearsystem.

Bullseye's glibc 2.31 keeps the cores running on BaseOS's older system. Base² has glibc
2.44, so it could run cores built on a newer system.

**Cross-building here (2026-11-06):** Docker isn't reachable from this WSL machine, so
Beetle Cygne was built with Ubuntu's cross compiler instead of the container, through the
same recipe: `CC=aarch64-linux-gnu-gcc CXX=aarch64-linux-gnu-g++ sh cores/cygne/build.sh
build <commit> vendor/cygne-src.tar.gz <workdir> vendor/cygne_libretro.so` (GCC 13.3; the
core needs nothing newer than glibc 2.17). The `.meta` stamp records the commit, recipe,
flags and patch as usual.

## Each core

### mGBA: Game Boy Advance, Game Boy, Game Boy Color

- **What:** Vicki Pfau's (endrift) accurate GBA emulator, which also runs the Game Boy
  and Game Boy Color. libretro's fork of it. Licence MPL-2.0.
- **Pin:** `e31759b` ("libretro: version", 2026-08-06). The fork has one newer commit,
  which quietens debug logging.
- **Patches (13):**
  - two backports from mGBA proper: an MBC2 save-bank fix (`543a19758`, #3859) and a
    FIFO DMA source-offset fix for GBA audio (`685023e05`);
  - slot's own: GBA flash saves keep their bank (`flash-bank-survives-mask`); Game Boy
    audio (`gb-audio-capacitor`, `gb-audio-centred-channels`, `gb-audio-stopped-channel`);
    a blank frame after the LCD switches on; the DMG palette following its option and
    choosing itself (`zz-gb-auto-palette`); and link play (`link-mode`, `zz-gb-link-mode`,
    `zz-link-peer-frameskip`, `zz-link-peer-silence`: the second console a device runs
    for a link is never drawn or heard).
- **In Slot²:** the GBA's default, and the only core for link play.

### gpSP: Game Boy Advance

- **What:** a fast GBA emulator with an ARM recompiler (Exophase's, libretro's fork).
  Licence GPL-2.0.
- **Pin:** `7b2dd37` (2026-08-19, a recompiler fix).
- **Patches (2):** `aw-session-reset` (a wireless-adapter session resets cleanly) and
  `jit-cache-in-binary` (the recompiler's cache lives in the core itself).
- **Upstream since (9 commits, 2026-08-20 to 09-19), worth having:** a scanned EEPROM
  signature no longer overrides a game's real SRAM or flash (#287); running code from VRAM
  under the recompiler (#289); GBA I/O read fixes (three commits); PSG status bits kept
  readable; no crash when memory for the ROM or the recompiler cache can't be had; and the
  default sound rate moved from 65536 to 32768 Hz (#288).
- **In Slot²:** a pick for GBA games, for speed.

### Gambatte: Game Boy, Game Boy Color

- **What:** sinamas's accurate Game Boy and Game Boy Color emulator, libretro's port.
  Licence GPL-2.0.
- **Pin:** `77f2567` (2026-10-30), the newest.
- **Patch (1):** `gb-audio-centred-channels`, as in mGBA.
- **In Slot²:** the Game Boy's default since upstream slot 1.14 (2026-11-02).

### Beetle Cygne: WonderSwan and WonderSwan Color

- **What:** Mednafen's WonderSwan module (from Cygne), as a libretro core. GPL-2.0.
- **Pin:** `4b01295` (2026-07-31), the newest.
- **Patch (1):** `0001-lcd-icons`, Slot²'s: the LCD's icon byte (port 0x15), which a game
  lights to say how the console is held, as memory id `0x5715`
  ([design/ws-orientation.md](../design/ws-orientation.md)). The core's own rotation reads
  no orientation by itself (its header check is `#if 0`).
- **Built by Slot²** (2026-11-06), cross-compiled; slotPlus's build is kept on the card as
  `System/cygne_libretro.so.slotplus`.
- **In Slot²:** opens a game as it was last left or as its cart says, turns it with Rotate
  (SELECT) and when the game lights an orientation icon, and turns its pads to follow
  (`session.rs`); its second pad is in the WonderSwan's controls.

### Beetle Lynx: Atari Lynx

- **What:** Mednafen's Lynx module (from Handy), as a libretro core. GPL-2.0.
- **Pin:** `fcdefcf` (2026-04-20), the newest. No patches.
- **In Slot²:** Slot² turns games made to be held sideways (`LYNX_ROTATION_FILE`,
  `hold_picker.rs`).

### Gearsystem: Game Gear

- **What:** Ignacio Sánchez's (drhelius) Sega 8-bit emulator: Master System, Game Gear,
  SG-1000. Its own repository, with its libretro core in `platforms/libretro`. GPL-3.0.
- **Pin:** `517ddaa`, release 3.9.22 (2026-10-11). Built with LTO. No patches.
- **Upstream since:** 3.9.23 (2026-10-24) and 3.9.24 (2026-10-30): mostly the desktop app
  (OpenGL, vsync, file dialogs), and "audio rate control" and bug fixes that may reach the
  core. 57 commits in all (2026-11-07).
- **In Slot²:** Select never reaches it, since Gearsystem takes it as a console reset.

### Beetle NeoPop: NeoGeo Pocket and Pocket Color

- **What:** Mednafen's NeoGeo Pocket module (from NeoPop), as a libretro core. GPL-2.0.
- **Pin:** `a50d5ac` (2026-06-14), the newest. No patches.
- **In Slot²:** saves in its own `Saves/NGP/` folder (`save_dir_for`).

### Beetle PCE Fast: TurboGrafx-16 (PC Engine)

- **What:** Mednafen's faster PC Engine module, as a libretro core. GPL-2.0.
- **Pin:** `3f946f2` (2026-10-02). Two newer commits, both translations. No patches.
- **In Slot²:** its games change width as they run; Slot² follows the frame it last drew.

## Every option each core declares

`cores/options.json` lists them with their values, from each core's source at its pin
and Slot²'s patches; `python3 -I tools/slotsq/core_options.py` writes it, and a test
checks every option Slot² sets against it. Regenerate it with any pin or patch change.
Which of them Slot² uses, and why, is [design/core-options.md](../design/core-options.md).

## Where to look next

1. **gpSP:** the save-type and recompiler fixes. A pin bump for us, and for upstream slot,
   which pins the same commit.
2. **Gearsystem:** 3.9.24, after checking which of its changes reach the libretro core.
3. The rest are at their newest, or behind only by logging and translations.
