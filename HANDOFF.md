# Handover: Base² and Slot² on the Powkiddy RGB20SX

Last updated 2026-11-06. Read this first. Older detail is in
[history/handoff-2026-11-05.md](history/handoff-2026-11-05.md).

## The two halves

- **Base²** (`basesq`): the OS, in [`os/`](os/README.md). Buildroot, our own kernel, and
  just enough to run a frontend.
- **Slot²** (`slotsq`): the frontend, this repo's `crates/` and `card/`. A fork of
  [slot](https://github.com/BrandonKowalski/slot) (at `f1f6ef9`) with
  [slotPlusUltra](https://github.com/nelynes/slotPlus) merged in (WonderSwan, Lynx, Game
  Gear, NeoGeo Pocket, TurboGrafx-16, DS) and our changes. Fork changes in the code are
  marked "Slot²", slotPlus's "slotPlus".
- They meet only through [CONTRACT.md](CONTRACT.md). Neither names the other's internals,
  and Slot² never names the device: it reads `/etc/slot-device`.
- Running slot on ROCKNIX was retired on 2026-11-06; its notes and probe scripts are in
  `history/rocknix/`.

## Branches

- `slotsq-plus`: the current work (local only, not pushed yet). It is
  `claude/beautiful-feynman-ffzfnx` plus the slotPlusUltra merge and everything since.
- `claude/beautiful-feynman-ffzfnx`: on the fork (`SerjTargarien/slot-alt`) up to
  `e9c035d`. Fold `slotsq-plus` into it and push when the user asks.
- No release until Base² and Slot² are stable and feature complete.

## The device

- Dev card in slot 1 runs Base²; the ROCKNIX card in slot 2 is the recovery (swap it into
  slot 1 to boot ROCKNIX and read the dev card).
- SSH: `ssh -o UserKnownHostsFile=~/.ssh/known_hosts_basesq root@192.168.1.72`.
- The card's `System/` holds Slot²'s build and slotPlus's nine cores from its release
  `v1.14.0-plus2.6` (`System/premerge/` and `System/slot.premerge` are the pre-merge ones).
- A `basesq-dev` file at the card's root keeps Wi-Fi (and SSH) up in games. Remove it
  for battery runs.

## Working

- Base²: `os/dev.sh` (see [os/README.md](os/README.md)).
- Slot²: `tools/slotsq/dev.sh build | deploy | restart | logs NAME | shot NAME |
  capture NAME VAR=x`. Deploy and restart only once the user is on the shelf: a restart
  ends a game in play.
- Measurements: `SLOT_TRACE_POWER=1` in `/run/slot.env`, then
  `python3 -I tools/slotsq/medians.py NAME` on logs saved with `dev.sh logs NAME`.
- Run commands as `. "$HOME/.cargo/env"` first.

## Testing

- `sh tools/slotsq/test.sh` while working: the crates changed since the last commit, and
  `slot` through them. `sh tools/slotsq/test.sh all` before a deploy. Both run every test
  and fail only on failures not in `tools/slotsq/known-failures`, which says why each one
  is there. The output goes to `logs/test.log`.
- `slot` and `slot-ui` build their integration tests as one program each
  (`tests/all.rs`, which a test checks lists every file). A test that changes state the
  whole process shares (environment variables, the Wi-Fi worker, the ROM header index)
  gets its own `[[test]]` in Cargo.toml instead.
- Render tests run here: `slot-gfx`'s Linux `HeadlessSurface` is Mesa's EGL with no window
  and GLES2, as on the device. A test checks it starts, since render tests skip quietly
  without it. Frame-time tests are skipped on Linux (the software renderer is no measure).
- `romm_ui` needs `--features romm`.
- **Cores:** [cores/README.md](cores/README.md) maps every core (source, pin, patches, how
  current it is). Once a day or session, `python3 -I tools/slotsq/cores.py daily` checks
  the map names every pin and reports what upstream published since (only what changed
  since its last run, kept in `logs/cores-check.json`). Update the map with any pin, patch
  or core change.
- Build settings for this machine are in `~/.cargo/config.toml`, not the repo: mold as the
  linker, and line numbers only in debug builds. A full test build takes about a minute
  and `target/` stays at a few GB; `cargo clean` if it grows.
- Format with `sh tools/slotsq/fmt.sh`: it formats the changed files that were formatted
  already and names upstream's unformatted ones (88), whose changed lines are formatted
  by hand. Never `cargo fmt --all`, nor `rustfmt` on one of those.
- Adding a setting, a row, a system feature or a core option:
  [design/checklist.md](design/checklist.md) lists the places and the test that catches
  each one missed.
- Commits: plain messages, author and committer Serj Targarien
  (`xergy10@gmail.com`), no `Co-Authored-By` or session lines.
- Remote commands: background only the restart, as its own statement
  (`...; setsid sh -c "..." < /dev/null > /dev/null 2>&1 &`). A backgrounded
  `a && b &` chain dies with the SSH session.
- Ask before anything costly or hard to undo. Wi-Fi credentials never leave the device.

## Where things stand

Done and checked on the device:
- Base² boots in 11-12 s on our kernel; every button, FN, sound (speaker and headphones),
  brightness on a linear scale, Wi-Fi run by the frontend with power save, network time,
  Gamebit for files.
- Slot² uses the whole 720x720 panel (a height-aware canvas), 10-row menus, a Stretch
  shortcut pair (Menu+L1/R1) and Stretch On/Off per platform.
- slotPlusUltra merged: RetroAchievements off at build time (its row hidden); its H700
  HDMI runs only on BaseOS's fbdev; its update channel hidden (`SELF_UPDATE=0`).
- Scale (Integer or Fit, never "actual") and Stretch for Game Gear, NeoGeo Pocket,
  TurboGrafx-16 (following its width), WonderSwan and Lynx (either orientation, following
  the frame). No bezels at these sizes yet.
- Controls ([design/controls.md](design/controls.md)): a page per system, up to three
  bindings per input (chords, MENU and SELECT chords), each system's own names, both
  sticks and L3/R3 as bindable inputs, the left stick steering outside games, the DS's
  screens and stylus as inputs.
- Base² reports the sticks the standard way (device tree, and a fix to the kernel's
  `adc-joystick`, worth sending upstream) and polls them and FN at 10 ms
  ([design/input-layers.md](design/input-layers.md)).
- The DS on the whole square panel: four layouts at Integer or Fit, Stretch for one
  screen, a Layout row on the shelf (a new game's) and in game (the game's own).
- Tests: one program each for `slot` and `slot-ui`, render tests on Linux, a full run in
  about 90 s, one known failure (see Testing).
- Default controls per system ([design/controls.md](design/controls.md), step 4): turbo
  gone, the WonderSwan's Y pad on the right stick and shoulders, the DS stylus on the right
  stick and R2. More systems as the user plays them.
- Shortcuts ([design/shortcuts.md](design/shortcuts.md)): the game's input wins over a
  shortcut on the same button; General plus a page per system on the shelf.
- Game overrides ([design/game-overrides.md](design/game-overrides.md)): one file per game
  (`Config/games/<system>/<file name>.txt`). In a game, the Screen, Gameplay, Controls and
  Shortcuts pages set the game's own values, drawn in amber, with Y back to the shelf's.
- WonderSwan orientation ([design/ws-orientation.md](design/ws-orientation.md)): Rotate
  (SELECT) turns any game, a game keeps the way it was left, and a game turns by itself
  when it lights the LCD's orientation icons (Klonoa does).
- Cores mapped in [cores/README.md](cores/README.md): slotPlus's v1.14.0-plus2.6 builds,
  but Beetle Cygne, Slot²'s own build since 2026-11-06 (patched for the LCD icons,
  cross-compiled here).
- The structure pass (2026-11-07, [design/checklist.md](design/checklist.md)): invariant
  tests for the menu (`crates/slot-ui/tests/invariants.rs`) and the store
  (`crates/slot-store/tests/invariants.rs`); the menu's rows from one list (`quick_rows!`);
  the long "every other row" lists in `app.rs` gone, with a test in their place;
  Slot²'s own code in files of its own (`app/games.rs`, `app/game_keys.rs`,
  `app/core_options.rs`, `session/ws.rs`); render tests that wait for the game rather
  than a fixed time; `cores/options.json`, against which every option Slot² sets is
  checked; `tools/slotsq/fmt.sh`; names that say what they do. It found and fixed: the
  shortcut pages showing each other's names, Back on the DS Screen page closing the menu,
  a game file unable to keep a palette the model allowed, and the DS's Stretch without
  shortcuts.
- Core options ([design/core-options.md](design/core-options.md)): every core's surveyed;
  the WonderSwan and TurboGrafx-16 palettes and a Sprite Limit row added (its effect not
  yet seen); gpSP at 32 kHz and the Lynx at 60 Hz measured and left as they are; the
  shelf's GBA Frame Blending says gpSP has no ghosting. `SLOT_CORE_OPTIONS` tries core
  options without a build.

## The state now (2026-11-07)

- **Committed:** everything, up to the palettes and the feature-driven shortcuts
  (`45ecb6e`), on `slotsq-plus`, local only (not pushed). The device runs that build.
- **On the card:** `System/cygne_libretro.so` is Slot²'s patched Beetle Cygne (slotPlus's
  is kept as `System/cygne_libretro.so.slotplus`); `Config/selected_core.txt` sets
  Castlevania - Aria of Sorrow to gpSP; `Config/shortcuts/ws.txt` has the WonderSwan's
  palette keys (MENU+L2/R2), set while the pages showed the wrong names.
- **Cores here:** Docker isn't reachable from this WSL machine and `task` isn't
  installed; cores are cross-compiled through their recipes (cores/README.md, "How they
  are built"). `scp` doesn't work with Base² (no SFTP server): copy files with
  `ssh ... 'cat > path' < file`. The scratchpad in `/tmp` doesn't survive a restart of
  WSL; `vendor/` keeps the pinned sources that were fetched.
- **Tests:** the render tests that start a game wait for it to be playing rather than a
  fixed time (2026-11-07); a loaded run no longer fails them.

## Next, in order

The order is decided with the user at the start of a session; see the list below (last
set on 2026-11-06).

1. **Per-frame diet** (measured 2026-11-06 on the device): in a game, the bezel folder is
   listed on the card 3-5 times a frame (~65 us each, empty or not), the wallpaper folder
   once with a file or per-shelf wallpaper (~62 us), and the shortcut list rebuilt (38 us):
   2-3 % of a frame for nothing. Cache the listings (refresh at boot, after the file server
   or a sync writes, when a Screen page opens) and the shortcut list (rebuild when bindings,
   the game or the context change). In menus it's worse (every system's bezel folder, a
   table per controls row drawn). Separately: the shelf waits for cart faces while
   scrolling. Upstream's code: worth offering upstream.
2. **Audio misbehaves in general** (reported 2026-11-06): to be diagnosed, then fixed. To
   try there: the cores' own filters ([design/core-options.md](design/core-options.md),
   6: mGBA's and the WonderSwan's low-pass, Gambatte's resampler). Also from there: gpSP
   at 32768 Hz sounded unplayable, and even good runs report starved samples.
3. Link play when the second device arrives; our own KMS HDMI (the RGB20SX has
   `HDMI-A-1`); our own U-Boot.
4. Bezels at Integer and Fit for the new systems.
5. **DSperate v3.3.0:** try dsperate-lr's prebuilt PGO core (Base²'s glibc runs it)
   against today's, timed, before anything else on DS speed
   ([cores/README.md](cores/README.md)).
6. Build the other cores ourselves (`cores/*/build.sh`; cross-compiling here works, as
   Beetle Cygne showed on 2026-11-06), then bump gpSP (save-type and recompiler fixes) and
   Gearsystem (3.9.24).

Waiting on a decision or an outside party:
- The rest of the structure pass, parked (2026-11-07) because it would make upstream's
  updates hard to take, until the user decides whether to share the port back:
  per-system settings as one record per system (`shader_gba`, `shader_gb`... fields),
  and splitting upstream's `app.rs` by topic. Meanwhile new code of Slot²'s own goes in
  files of its own ([design/checklist.md](design/checklist.md)).
- The shelf's system first in Screen and Controls > In-Game (game-overrides.md): shelved
  on 2026-11-06. Amber means a game's own value, so it can't also mark the shelf's system;
  it would need another cue.
- The Game Boy label size (`tools/slotsq/known-failures`): upstream's tests disagree.
- The `adc-joystick` fix, to send to the kernel (`os/.../0004-...patch`).
- Input backlog ([design/input-layers.md](design/input-layers.md)): one merged input, only
  if the D-pad and stick ever misbehave.
- SELECT is held back up to 600 ms in games while SELECT chords exist
  ([design/shortcuts.md](design/shortcuts.md), Later).
- Turbo as a feature (removed as an input on 2026-11-06): one way for every system, such as
  a MENU chord that makes whatever is held auto-fire.

Label art for the new systems (the art server covers only GB, GBC and GBA): WS 324x132,
Lynx 284x190, GG 290x200, NGP 246x133, TG16 191 wide up to 163 tall, NDS 230x250.

## Renaming (2026-11-06)

The OS was called "slot²" (`slotsq`) until 2026-11-06; it is Base² (`basesq`) now and
"Slot²" is the frontend. The device was moved over the same day (kernel `7.2.11-basesq`,
`/etc/basesq/`, `/data`, card files `basesq-*`) and the old files removed. Older notes and
logs still say `slotsq` for the OS.
