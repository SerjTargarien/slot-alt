# Handover: Base² and Slot² on the RGB20SX and the RG34XX

Last updated 2026-11-07. Read this first. Older detail is in
[history/handoff-2026-11-05.md](history/handoff-2026-11-05.md).

## The two halves

- **Base²** (`basesq`): the OS, in [`os/`](os/README.md). Buildroot, our own kernel, and
  just enough to run a frontend.
- **Slot²** (`slotsq`): the frontend, this repo's `crates/` and `card/`. A fork of
  [slot](https://github.com/BrandonKowalski/slot) (at `f1f6ef9`) with
  [slotPlusUltra](https://github.com/nelynes/slotPlus) merged in (WonderSwan, Lynx, Game
  Gear, NeoGeo Pocket, TurboGrafx-16, DS; the DS removed since) and our changes. Fork
  changes in the code are marked "Slot²", slotPlus's "slotPlus".
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

## The devices

Both are development devices, aiming at the same player-facing features
([design/two-devices.md](design/two-devices.md)). One build runs on both.

- **Powkiddy RGB20SX** (RK3566, 720x720, two sticks), running **Base²**. Dev card in slot 1;
  the ROCKNIX card in slot 2 is the recovery (swap it into slot 1 to boot ROCKNIX and read
  the dev card). SSH: `ssh -o UserKnownHostsFile=~/.ssh/known_hosts_basesq
  root@192.168.1.72`. Its `System/` holds Slot²'s build and slotPlus's cores from release
  `v1.14.0-plus2.6`, but Beetle Cygne, ours (`System/premerge/` and `System/slot.premerge`
  are the pre-merge ones).
- **Anbernic RG34XX** (H700, 720x480, no sticks), running **BaseOS 1.4.0**. SSH: `ssh -o
  UserKnownHostsFile=~/.ssh/known_hosts_rg35xx root@192.168.1.75` (ask before writing to
  it). Its card is a fresh BaseOS install with test games, easy to make again. Its
  `System/` matches the RGB20SX's, with our `launch_frontend.sh` (stock's kept as
  `launch_frontend.sh.stock`), stock slotPlus's binary kept as `slot.prev`, and stock
  Cygne as `cygne_libretro.so.slotplus`. BaseOS respawns slot when it exits.
- On both, a `basesq-dev` file at the card's root keeps Wi-Fi (and SSH) up in games
  (`SLOT_WIFI_KEEP`). Remove it for battery runs. Never print `Config/wifi.conf`.

## Working

- Base²: `os/dev.sh` (see [os/README.md](os/README.md)).
- Slot²: `tools/slotsq/dev.sh build | deploy | restart | logs NAME | shot NAME |
  capture NAME VAR=x`, on the RGB20SX unless `DEVICE=rg34xx`. Deploy and restart only
  once the user is on the shelf: a restart ends a game in play. A deploy clears
  `/run/slot.env`, so on the RGB20SX follow it with `dev.sh restart AGS_LINK_FREQ=2437`
  until its Base² image carries the link channel.
- The build links against Debian bullseye's glibc 2.31 (`tools/slotsq/sysroot.sh`, once,
  into `vendor/`), so it runs on BaseOS too; `dev.sh build` refuses a binary that needs
  more.
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
- **Menus on both devices:** `menu_map` walks every menu on both panels, keeps a map per
  device and their differences in `crates/slot/tests/menu-map/`, and checks rules on
  every page (a setting shows something, fits, changes with Left/Right, a page fits or
  scrolls). A menu change fails it until the maps are written again:
  `SLOT_MENU_MAP=write cargo test -p slot --test tests menu_map`; read the diff.
- **Flaky under load:** `server_games::purge_…` and `rename_…` (the test server gives no
  answer, status 0) and `render_screenshot::a_shot_on_…` sometimes fail in a full run and
  pass alone. Worth fixing.
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
  sticks and L3/R3 as bindable inputs, the left stick steering outside games.
- Base² reports the sticks the standard way (device tree, and a fix to the kernel's
  `adc-joystick`, worth sending upstream) and polls them and FN at 10 ms
  ([design/input-layers.md](design/input-layers.md)).
- Tests: one program each for `slot` and `slot-ui`, render tests on Linux, a full run in
  about 90 s, one known failure (see Testing).
- Default controls per system ([design/controls.md](design/controls.md), step 4): turbo
  gone, the WonderSwan's Y pad on the right stick and shoulders. More systems as the user
  plays them.
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
  shortcuts. A `/code-review` (medium) of it found eight more, all fixed the same day:
  `fmt.sh` formatting child modules too (now through stdin, one file alone), a stray doc
  comment from the move, the per-game palette rule decided by system, the option test
  calling the real turn and Game Boy Player values, the Sprite Limit note kept and
  tested beside its cores, the generator stopping loudly without an options table, the
  option test's name, and a test that every shelf row shows a value, which found the
  Game Gear, Pocket, TurboGrafx-16, WonderSwan and Lynx Stretch rows blank on a 4:3
  panel (now hidden there).
- Two devices ([design/two-devices.md](design/two-devices.md), 2026-11-07): one build
  for both (glibc 2.31); the RG34XX on BaseOS runs Slot² with upstream's update channel
  off everywhere; each device's parts (sticks, stick clicks, lid) detected at boot
  (`parts.rs`, keys in `/etc/slot-device` to override); our 112 commits reviewed against
  stock slotPlus and checked on the device (two slips fixed: the added systems' bezels
  on 3:2, an empty in-game Stretch row; menu pages that ran off a 3:2 screen now scroll).
- Link play between the two (2026-11-07): Tetris linked, the RG34XX hosting on channel 6.
- The Nintendo DS removed from both (2026-11-07): its layouts suited neither panel and
  neither has touch.
- Core options ([design/core-options.md](design/core-options.md)): every core's surveyed;
  the WonderSwan and TurboGrafx-16 palettes and a Sprite Limit row added (its effect not
  yet seen); gpSP at 32 kHz and the Lynx at 60 Hz measured and left as they are; the
  shelf's GBA Frame Blending says gpSP has no ghosting. `SLOT_CORE_OPTIONS` tries core
  options without a build.

## The state now (2026-11-07)

- **Committed:** everything, up to the menu pages that scroll (`3efc703`), on
  `slotsq-plus`, local only (not pushed). Both devices run that build.
- **On the RGB20SX's card:** `System/cygne_libretro.so` is Slot²'s patched Beetle Cygne
  (slotPlus's kept as `System/cygne_libretro.so.slotplus`); `Config/selected_core.txt`
  sets Castlevania - Aria of Sorrow to gpSP and Tetris to mGBA; `Config/shortcuts/ws.txt`
  has the WonderSwan's palette keys (MENU+L2/R2). Its Base² image doesn't set the link
  channel yet (`os/.../etc/basesq/frontend` does, from the next image).
- **On the RG34XX's card:** Tetris on mGBA; our launcher, build and Cygne (above).
- **On both:** the DS's core, games, labels, saves, states and config files deleted.
- **Cores here:** Docker isn't reachable from this WSL machine and `task` isn't
  installed; cores are cross-compiled through their recipes (cores/README.md, "How they
  are built"). `scp` doesn't work with Base² (no SFTP server): copy files with
  `ssh ... 'cat > path' < file`. The scratchpad in `/tmp` doesn't survive a restart of
  WSL; `vendor/` keeps the pinned sources that were fetched.
- **Tests:** the render tests that start a game wait for it to be playing rather than a
  fixed time (2026-11-07); a loaded run no longer fails them.

## Two devices (2026-11-07)

The plan is [design/two-devices.md](design/two-devices.md). Done: phases 0 (the RG34XX
safe to develop on), 1 (one build), 3 (each device's parts), most of 2 (the review
against stock slotPlus, the menu maps). Left: phase 2's hands-on feature list (saves,
states, rewind, sleep, HDMI on both), phase 4 (link play both ways) and phase 5 (our own
Gamebit, which needs bun; our own label database; the structure pass). The fork is ours:
upstream's features are taken by hand, and keeping upstream mergeable doesn't count.

## Next, in order

The order is decided with the user at the start of a session; see the list below (last
set on 2026-11-06; on 2026-11-07 the two-devices plan came first, and link play is its
phase 4).

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
3. **Link play both ways (2026-11-07; design/two-devices.md, phase 4).** First, the bug
   the user found at the end of the day: when one device ends the link, the other can't
   hold MENU to eject the game. Then the channel setting, then Pokémon and Mario Kart.
   The RGB20SX hosts on wlan0 now, starts are ~8 s faster, and joins no longer drop 3 s
   in (`f9a2227`). The history:
   **First link (2026-11-07).** Tetris on mGBA, the RG34XX (BaseOS
   1.4.0, stock slotPlus) hosting on channel 6 and the RGB20SX joining: both play, no
   stalls, a bit of lag on the RG34XX. Two fixes made it work: Base²'s launcher sets
   `AGS_LINK_FREQ=2437` (the join looked on 5745 first, a channel the RTL8723DS can't
   use), and slot no longer takes Wi-Fi back while a link starts (`SLOT_WIFI_KEEP` fought
   `slotlink.sh` for wlan0 and Wi-Fi stayed down until a restart; test
   `wifi_ui::the_development_switch_lets_go_while_a_link_starts`). The RG34XX's launcher
   exports the same channel for these tests (`System/launch_frontend.sh.stock` is the
   original; our own `card/System/launch_frontend.sh` now does the same). Next (phase
   4): a channel setting on the Network page (2.4 GHz 1, 6, 11; 5 GHz where the chip has
   it), the RGB20SX hosting (wlan0 at 2.4 GHz when there's no wlan1); the join's 10 s
   before its first scan (`join: asked` to `wlan0 present`, after `link_clear`); the
   RG34XX's lag (its log: 13338 samples dropped, 31772 starved in the link; the RGB20SX
   323844 starved); Pokémon trades and Mario Kart. Then our own KMS HDMI (the RGB20SX has
   `HDMI-A-1`) and our own U-Boot.
4. Bezels at Integer and Fit for the new systems: only needed now where the picture moves
   (the square panel, Fit); on a 3:2 panel at Integer slotPlus's bezels show.
5. Build the other cores ourselves (`cores/*/build.sh`; cross-compiling here works, as
   Beetle Cygne showed on 2026-11-06), then bump gpSP (save-type and recompiler fixes) and
   Gearsystem (3.9.24).

Waiting on a decision or an outside party:
- The rest of the structure pass: per-system settings as one record per system
  (`shader_gba`, `shader_gb`... fields), and splitting upstream's `app.rs` by topic. No
  longer blocked: the fork is ours, and upstream's features are taken by hand
  (2026-11-07, [design/two-devices.md](design/two-devices.md)). New code of Slot²'s own
  still goes in files of its own ([design/checklist.md](design/checklist.md)).
- One repository or two for Base² and Slot²: open, no hurry (two-devices.md).
- Stretch on an upright WonderSwan or Lynx game (squashed on a 3:2 panel): likely no
  Stretch while upright, since Fit does the rest (two-devices.md). Not a priority.
- The WonderSwan's second pad on the shoulders keeps the default Fast Forward (R2) and
  Rewind (L2) out of WonderSwan games (the "!" on its controls page): move those two for
  WonderSwan games, or accept it.
- A version name for our builds (`SLOT_VERSION`); builds carry their hash and date.
- Housekeeping: the flaky tests above; four `__pycache__` files tracked in git.
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
Lynx 284x190, GG 290x200, NGP 246x133, TG16 191 wide up to 163 tall.

## Renaming (2026-11-06)

The OS was called "slot²" (`slotsq`) until 2026-11-06; it is Base² (`basesq`) now and
"Slot²" is the frontend. The device was moved over the same day (kernel `7.2.11-basesq`,
`/etc/basesq/`, `/data`, card files `basesq-*`) and the old files removed. Older notes and
logs still say `slotsq` for the OS.
