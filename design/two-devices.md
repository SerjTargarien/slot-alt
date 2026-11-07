# Two devices, one Slot²

Planned 2026-11-07. Both handhelds are development devices from now on, and the mission
is the same player-facing features on both, wherever the hardware allows. Where it
doesn't, the difference is written down here (Differences), never left to be found.

| | RGB20SX | RG34XX |
|---|---|---|
| OS | Base² (ours, `os/`) | BaseOS 1.4.0 (h700), kernel 4.9.170 |
| SoC, RAM | RK3566, 1 GB | H700 (`sun50iw9`), 1 GB, 480-1512 MHz |
| Panel | 720x720 (square: the tall layouts) | 720x480 (3:2: upstream's band, `extra() = 0`) |
| Display path | KMS/GBM (`/etc/slot-device`) | fbdev and Mali (no description file: BaseOS) |
| Buttons | gamepad codes, FN as a key, two sticks, L3/R3 | `ANBERNIC-keys`, no sticks (it still reports axes 3-5 and the hat) |
| Wi-Fi | RTL8723DS, 2.4 GHz only, `wlan0` only | RTL8821CS, 2.4 and 5 GHz, `wlan0` and `wlan1` |
| glibc | 2.44 | 2.35 |
| HDMI | `HDMI-A-1`, not driven yet | mini HDMI, slotPlus's H700 switch (fbdev) |
| SSH | `known_hosts_basesq`, `root@192.168.1.72` | `known_hosts_rg35xx`, `root@192.168.1.75` |

The fork is ours: upstream slot and slotPlus are sources to take features from by hand
when we want them, not branches to stay mergeable with (the user, 2026-11-07).

## What stands in the way today (found 2026-11-07)

1. **Our build doesn't start on BaseOS.** It needs `GLIBC_2.39` (Rust std's
   `pidfd_spawnp`/`pidfd_getpid`, a version need without the weak flag) and `GLIBC_2.35`;
   BaseOS 1.4.0 has 2.35. Upstream builds in Debian bullseye (glibc 2.31,
   `taskfile.yml`); we build against Ubuntu 24.04's 2.39. Our patched Beetle Cygne needs
   only 2.17.
2. **It would update itself back to slotPlus.** With no `/etc/slot-device`,
   `SELF_UPDATE` is on (`hardware.rs`); our version is `0.1.0` (no tags), so any slotPlus
   release looks newer, and the update deletes `System/` files not in its manifest
   (`launch_frontend.sh`), the patched core among them. BaseOS's own updates (`.bosupd`)
   are offered too.
3. **The card's launcher is the stock one.** `card/System/launch_frontend.sh` has no link
   channel (the RG34XX's copy was edited by hand for the link test) and doesn't read
   `/run/slot.env` or a development switch, so the RG34XX can't be driven like the RGB20SX
   (`dev.sh restart VAR=x`).
4. **A stick that isn't there.** Upstream read a left stick only on models that have one
   (`LEFT_STICK_TARGETS`: rg34xxsp, rg35xxh, rg35xxpro, rg40xxh, ...; not the rg34xx).
   Slot² opens any node with stick axes (`f5f1586`), and `ANBERNIC-keys` reports axis 3,
   which the Anbernic layout reads as the left stick's Y. To check on the device first:
   menus scrolling by themselves would be this.
5. **Controls for inputs it lacks.** The controls pages offer both sticks and L3/R3, and
   two systems' defaults use the right stick (the WonderSwan's Y pad, the DS stylus).
6. **Link play.** The RGB20SX can't host yet (no `wlan1`), and the channel is set by hand
   in two launchers (see Link play below).

What already matches, from the code survey: without a description file Slot² takes
upstream's BaseOS paths for the display, the Anbernic buttons, FN (`0x138`), the lid,
battery and backlight, audio (HDMI's `ahubhdmi` first when on HDMI), the H700 HDMI
switch, Wi-Fi through `slotlink.sh`, the boot logo, network time (`/usr/bin/timedatectl`
is there) and the `8821cs` unload at power-off. 720x480 is the band upstream was laid
out for; every tall-panel branch (`App::tall()`) takes upstream's way there.

## The plan

Each phase ends checked on both devices. Nothing is written to the RG34XX before phase 0.

### Phase 0: make the RG34XX safe to develop on

1. No backup: the RG34XX's card is a fresh BaseOS install with test games only, easy to
   make again (the user, 2026-11-07).
2. **Done (2026-11-07):** self-update off in the build itself, not by device
   (`Hardware::baseos()`, test `baseos_is_never_offered_upstreams_updates`): slotPlus's
   channel would replace Slot² anywhere. The same switch gates BaseOS's own updates
   (`.bosupd`), done by hand while it's a dev device.
3. A version of our own: the build already records its git hash and date (About, the
   logs); a version name (`SLOT_VERSION` at build time) is the user's to pick.
4. **Done:** Slot²'s own `card/System/launch_frontend.sh` for BaseOS: stock's (they were
   byte-identical), plus the link channel (2437 unless set), the `basesq-dev` switch
   (`SLOT_WIFI_KEEP`) and `/run/slot.env`, so `dev.sh` drives both devices the same way.
5. **Done:** `dev.sh` learns the second device (`DEVICE=rgb20sx|rg34xx`, its known-hosts
   file). The restart is the same on both (BaseOS's `frontend-session` starts slot again
   too). Deploy copies through `ssh 'cat > path'` and checks the checksum.

### Phase 1: one build for both

1. **Done (2026-11-07):** linked against Debian bullseye's arm64 glibc 2.31, as upstream:
   `tools/slotsq/sysroot.sh` fetches `libc6`, `libc6-dev`, `linux-libc-dev` and
   `libgcc-s1` (pinned by checksum, from archive.debian.org) into
   `vendor/sysroot-bullseye`. Ubuntu's cross gcc looks in its own glibc 2.39 first, so
   `dev.sh build` puts the sysroot first with `-B`. The build needs glibc 2.30 at most,
   as stock slotPlus's; BaseOS's loader accepted it (`ld.so --verify`), and it runs on
   both.
2. **Done:** `dev.sh build` (and `deploy`) refuses a binary needing more than glibc 2.31.
3. One build profile for both (upstream's `device` profile, fat LTO, or our `release`),
   decided on measured start time and frame time.
4. The same `System/` on both: `slot`, the nine cores (Cygne ours), the scripts, `web/`.
   A `dev.sh sync` that compares the two cards' `System/` by checksum.

### Phase 2: first Slot² on the RG34XX, and the audit

1. **Done (2026-11-07):** the RG34XX runs Slot² (our launcher, build and Beetle Cygne;
   stock's kept as `.stock`, `.prev` and `.slotplus`): `device BaseOS: Fbdev display,
   Anbernic buttons`, panel and canvas 720x480, first frame at 633 ms, a rumble motor on
   `event1`. Its `System/` matches the RGB20SX's but for `fsck.exfat` (BaseOS's).
2. The phantom stick: the shelf stayed still for 6 s on its own; still to check in menus
   and games.
3. Go through the feature list below on both devices, one system at a time; each line
   ends as same, adapted, or a Difference.
4. Render tests at 720x480 for Slot²'s own pages (shortcut pages, a game's own values in
   amber, the WonderSwan Rotate, the palettes, Sprite Limit), beside the 720x720 ones.

### Phase 3: what the device has, as data

**Done (2026-11-07), as "detect, with keys as overrides"** (the user's choice over keys
alone: Base² is ours, BaseOS isn't, and detection covers the devices upstream supports as
upstream does): `crates/slot/src/parts.rs` detects the sticks, their clicks and a lid at
boot (`slot: parts: ...` in the log), with upstream's list of Anbernic models with a left
stick, and `LEFT_STICK`, `RIGHT_STICK`, `STICK_CLICKS`, `LID` in `/etc/slot-device` as
overrides (CONTRACT.md). Without a part: its axes are ignored (the phantom stick can't
happen), Lid Power Off is hidden, and bindings to it stay in the file unshown. Neither
device has a lid. The menu maps carry each device's parts. What follows was the plan.


The fix for items 4 and 5, and the way every future difference is handled: the device
says what it has, and Slot² hides what it hasn't and picks defaults from what it has.

- Keys: sticks (`none`, `left`, `both`), stick clicks (L3/R3), and later HDMI, rumble,
  Wi-Fi bands and whether it can host a link.
- Base² writes them in `/etc/slot-device` (CONTRACT.md changes with it). On BaseOS, with
  no file, they come from a table by `BASEOS_TARGET`, taken from upstream's knowledge
  (`LEFT_STICK_TARGETS`).
- The controls pages show only inputs the device has; a system's defaults that use a
  missing input fall back (the WonderSwan's Y pad to the shoulders and face buttons on a
  stickless device). A game file or a system file bound to a missing input keeps the
  binding (it's the card's, and the other device may have it) but shows it as absent.
- Config stays one format on both; a card moved between the devices keeps working.

### Phase 4: link play both ways

**Progress, rebuilt step by step (2026-11-08).** Yesterday's link changes (`f9a2227`,
`dd30214`) made links end about 10 s in and the eject seem stuck (the link had already
ended, each device playing alone). At the user's request they were reverted
(`ecaad68`), the baseline checked on the devices (links hold, end, eject), and the
changes brought back one at a time, each checked on both devices before the next:
1. **The faster home Wi-Fi stop** (`dc2e9da`): `wpa_cli terminate` waited out its 10 s
   for a reply wpa_supplicant never sends; the socket is watched instead. 2.26 s.
2. **The RGB20SX hosts on wlan0** (`dbb8ffb`) when there's no wlan1, at 2437 if its phy
   lists no 5 GHz.
3. **Silence counts once the link is armed** (`0866d3a`): both sides hold the game (the
   joiner restored the state, the host heard its READY), from zero then; 30 s for a start
   that never arms. Upstream counted the joiner's wait for the game as silence, so a
   state slower than ~1 s ended links 3 s in. The cable's own READY signal, not guesses
   from the masks (yesterday's "heard" and "running" rules both misfired).
After step 3, both directions hold, end from the menu, and eject on both.
4. **The host's game crosses packed** (`5af5bf5`): lz4 behind a "SQZ1" mark. Measured
   first (radio loggers, `logs/radio/`): the radio ran at 39-65 Mbit/s, but the RG34XX,
   joining in its driver's deepest power saving (LPS MAX), took the state at ~1 Mbit/s,
   ~3 s for 405 KB. Packed, it's ~1,900 bytes, and the joiner's first 5 s went from 58-70
   frames waiting to 2-3. A stock slotPlus joiner can't read a Slot² host's state.
5. **A link runs at full clock** (device_app.rs). The emulator paces itself against the
   other device during a link, so the RGB20SX's cap saw easy frames, went down to 600 MHz
   and stayed there; Mario Kart (mGBA) took 15-19 ms a frame and stuttered both ways. At
   1800 MHz it takes 4-7 ms, 60 fps with no stalls, both ways (2026-11-08); the clock
   drops back when the link ends.
What's left of the start (~7.7 s from the host connecting to the joiner's READY) is the
RGB20SX bringing its radio up and the Game Boy boot animation: fine as it is (the user). Not brought back, and not needed so far: clearing "lost" when a
link recovers, power saving off on a joining wlan0, the card tests in `test.sh`.

The eject "bug" of 2026-11-07 was the link having ended (yesterday's rules), the devices
playing alone. One refused eject on 2026-11-08 was short holds after a MENU press that
went down for 60 ms and again 20 ms later (a bounce, or a quick double press: a tap and a
double tap, the quick menu and the switcher); worth watching on the RG34XX.

Left: the channel setting (likely 2.4 GHz only, Default, 1, 6, 11: both devices have it),
and Pokémon trades (gpSP). Once, after hosting Mario Kart, the RG34XX's home Wi-Fi stayed
off (its log ends at "down", no "warm") until a restart; not seen since. Power saving on a joiner adds delay to each
packet; with the state packed, play is smooth from the start, so it can wait. The plan:

1. The channel setting (Network page, card-wide): Default, then 2.4 GHz 1, 6, 11, then
   5 GHz 36-48 and 149-165 where the chip has them (RG34XX). Slot² passes it to
   `slotlink.sh` as `AGS_LINK_FREQ`; Default sets nothing. Both chips' tables were read
   on 2026-11-07: 12-14 and the radar channels (52-144) are passive on both.
2. The RGB20SX hosting on `wlan0` at 2.4 GHz when there's no `wlan1` (its `rtw88`
   supports AP mode; home Wi-Fi drops while hosting, as joining already does).
3. The join's 10 s before its first scan (`join: asked` to `wlan0 present`).
4. Tested both ways (each device hosting), with Tetris, a Pokémon trade (gpSP) and
   Mario Kart (mGBA).

### Phase 5: ours

Decisions for the user, then work in this order:

1. **The DS: removed (done 2026-11-07).** Its layouts suit neither panel, neither
   device has touch, and the system takes long to optimise. Its code, layouts, stylus,
   pages, shortcuts, CPU hold, core recipe, art, Gamebit platform and card folders are
   gone; git keeps them. An old `slot.state` still reads, and a DS game left in the slot
   isn't resumed as another system's game of the same name. Left on the cards until
   removed by hand: `System/dsperate_libretro.so`, `Games/NDS` and the other NDS
   folders, `Config/nds_view.txt`, `Config/controls/nds.txt`. Gamebit's built pages on
   the cards are still slotPlus's and show an NDS tile until our own Gamebit is built
   (item 2).
2. **Gamebit our own.** Its source is in `web/` (Vue, built with bun; bun isn't
   installed here). Our systems only (no DS if it goes), our look, built by us into
   `System/web`.
3. **Labels of our own.** Labels come from upstream's server
   (`https://art.slot-cfw.fyi/`, `index.json` and `<stem>.webp`, `art/jobs.rs`).
   Our own database: where it lives (in the repo and made on the device by `art_make`,
   or hosted), which systems it covers, and what happens offline.
4. **The structure, now that merges don't count.** The rule of Slot²'s own code in files
   of its own (design/checklist.md) stays for clarity; the parked split of upstream's
   `app.rs` is no longer blocked.

## Checked against stock slotPlus (2026-11-07)

Our 112 commits since `slotplus/slot-1.14-ultra` (`a61ccfe`) were reviewed for RGB20SX
changes that reach the RG34XX, then checked on the device. Layout at 720x480, every
system's default picture size and place, fbdev, HDMI, pacing, audio, CPU clocks, Wi-Fi,
updates and the card scripts are stock's there (or gated by `/etc/slot-device`, KMS, the
gamepad layout or the tall canvas). Its LED is driven by stock's `led.sh` through the
battery driver (`work_led`, `lowpwr_led`); slot's own LED code finds no
`/sys/class/leds` there, as stock's didn't.

What had slipped in:
- **The stick gate** (`f5f1586` opens any node with stick axes): `ANBERNIC-keys` reports
  `ABS_RX`/`RY`/`RZ` at rest 0 on -1..1, and none moved in a 60 s test of every button
  and the D-pad. Harmless; phase 3 puts the gate back as data.
- **The added systems' bezels were off everywhere** (made for the square panel). Fixed
  the same day: they show where the picture is still slotPlus's (a 3:2 panel at
  Integer), test `scale::an_added_systems_bezel_shows_where_its_picture_is_slotpluss`.
- **Stretch on an upright WonderSwan or Lynx game** fills 720x480, squashed (checked on
  the device); stock never stretched upright games. Not a priority. Filling the height
  only is what Fit already does (the user), so the likely rule is no Stretch while
  upright; or rules for when Stretch is offered at all, rather than letting everything
  stretch every way.
- **An empty Stretch row in game** for the added systems on the 3:2 panel (the user saw it
  on the NeoGeo Pocket): the shelf hid those rows there, the in-game page didn't. Fixed:
  it hides the row when it has no value, and test
  `scale::every_in_game_screen_row_shown_has_a_value` checks every in-game Screen row of
  every system on both panels.

Meant for every device, and different from stock on the RG34XX: no turbo (X and Y free),
the Game Boy's Stretch on MENU+L1/R1, the WonderSwan's second pad on the shoulders (its
shoulders read L1, L2, R2, R1 too) which keeps the default Fast Forward and Rewind out of
WonderSwan games, vertical WonderSwan games opening upright, Scale rows for the added
systems, menu columns centred on the rows shown.

## Menus on both devices

`crates/slot/tests/menu_map.rs` (2026-11-07) walks every menu the way a player does, on
both panels (720x480 for the RG34XX, 720x720 for the RGB20SX): the shelf's, and each
system's in a game. It keeps:

- **A map per device**, `crates/slot/tests/menu-map/rg34xx.txt` and `rgb20sx.txt`: every
  page, the rows it shows, what each shows. A menu change fails the test until the maps
  are written again (`SLOT_MENU_MAP=write cargo test -p slot --test tests menu_map`), so
  it shows as a readable diff.
- **`differences.txt`**: what one device's menus have that the other's don't, row by row.
  On 2026-11-07, only Screen rows differ (Scale, Stretch and Bezel, by panel shape).
- **Rules on every page**: a setting shows something; what it shows fits beside its name
  (the drawing's own measure, `slot_ui::quick_value_squeeze`); a value changes with Left
  or Right and goes back; a page draws no more rows than fit. It catches the empty
  Stretch row found on the RG34XX, and the lists that ran off its screen (the shortcut and
  controls pages' systems, Gameplay, the Game Boy's Screen page; the user saw the first
  on 2026-11-07): only pages upstream had named could scroll, and now any page with more
  rows than fit does.

Not covered: what draws as its own picture (clock, wallpaper, Wi-Fi, RomM; marked
`(its own face)`), rows that need Wi-Fi, a game's own values, and how anything looks. A
contact sheet of every page per device is a possible later addition.

## The feature list (phase 2 fills it)

Same / Adapted / Difference, per device. Player-facing only.

- Shelf: systems, carts and labels, recents, favourites, wallpapers, the core picker
  (START), the Lynx hold.
- Playing: every system's core and options, palettes, Sprite Limit, WonderSwan rotation,
  Screen settings (Scale, Stretch, shaders, colour correction, blending), bezels.
- Per-game settings: Screen, Gameplay, Shortcuts, Controls; amber; Y clears.
- Controls and shortcuts: per system pages, chords, defaults per system.
- Saves, states and the switcher, resume, rewind, fast-forward.
- Link play: cable (mGBA, gpSP), wireless adapter (gpSP), both roles.
- Wi-Fi, network time, Gamebit, RomM, art fetch.
- Power: sleep, lid, brightness, volume, battery, power off; HDMI.

## Differences (kept up to date)

| Feature | RGB20SX | RG34XX | Why |
|---|---|---|---|
| Stick inputs, L3/R3 | yes | no | hardware |
| Tall layouts (Stretch to full height, ...) | yes | no | panel shape |
| Hosting a link | not yet (phase 4) | yes | the RTL8723DS has no second interface |
| 5 GHz link channels | no | yes | the RTL8723DS is 2.4 GHz only |
| HDMI out | not yet (roadmap) | yes | Base²'s KMS HDMI isn't written |
| CPU clock cap | yes | no | Base²'s description; to try on H700 |
| Rumble | yes | yes (`motor event1`) | |
| Lid Power Off | no | no | neither has a lid (`parts.rs`) |
| The OS: boot, sleep, updates | Base² | BaseOS | Base² is RK3566 only for now |

## Open, no hurry

- **One repository or two.** Base² (`os/`) and Slot² (`crates/`, `card/`) share this
  repository today and meet only through CONTRACT.md. Splitting them can wait as long as
  it needs, provided nothing either device needs breaks (the user, 2026-11-07).

## Not in this plan

- Base² on the RG34XX (an H700 port of our own OS). Possible later; BaseOS is slot's
  home and works.
- Moving saves between the devices (RomM or Gamebit could, later).
