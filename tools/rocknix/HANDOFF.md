# Handover: slot on the Powkiddy RGB20SX

Last updated 2026-11-05, evening. Read this first, then
`tools/distro/PLAN.md` (where the project is going) and `tools/rocknix/README.md`
(measured device facts and what the ROCKNIX port does).

Branch: `claude/beautiful-feynman-ffzfnx` (fork `SerjTargarien/slot-alt` of
`BrandonKowalski/slot`). Upstream `main` was still exactly our fork point `f1f6ef9` on
2026-11-05. Commits from this session are local only, not pushed.

## Resume here (2026-11-05, evening)

**Device state:**
- The **dev card is in slot 1** and runs **slot²** on our own kernel (Linux
  7.2.11-slotsq, our device tree). It boots straight into slot in 11-12 s.
- The **ROCKNIX card is in slot 2**, unused and unmounted. It boots ROCKNIX again when
  put back in slot 1.
- SSH into slot²:
  `ssh -o UserKnownHostsFile=~/.ssh/known_hosts_slotsq root@192.168.1.72`.
  It needs that separate known_hosts file, since the address is the same as ROCKNIX's
  and the host key differs.
- **The card's slot build is current:** `System/slot` has the height-aware layout
  (option B). `System/slot.band` is an older build kept only for reference.

**Working on our kernel** (BRINGUP.md attempts 6 and 7): picture, speaker and
headphones, every button, FN once per press, brightness and slot's hotkeys, and Wi-Fi at
the OS level. GBA audio sounds right. `extlinux.conf.rocknix-kernel` on the SLOT
partition boots ROCKNIX's kernel again (`dev.sh kernel-back`). Still borrowed from
ROCKNIX: U-Boot, libmali and its wrappers, and the Wi-Fi firmware and regdb (PLAN.md
decision 6).

**Screen layout is done (option B, PLAN.md "Screen layout on the square panel"):** a
square panel gets a 720x720 canvas drawn 1:1. Upstream's band coordinates stay; whole
groups move with `Canvas::below`, `Canvas::mid` and `shift`. The user checked the shelf,
settings, in-game menu, core picker, clock, About, screenshots and games. Capture screens
with `sh tools/distro/dev.sh shot NAME`; `dev.sh capture NAME SLOT_TALL_PANELS=0` shows
the old band layout for a before and after.

**Wi-Fi is slot's now (device layer step 2, 2026-11-05 evening).** `/etc/slot-device`
says `WIFI=1`. The OS boot script only starts SSH. slot starts `wpa_supplicant` on the
card's `Config/wifi.conf` through `System/slotlink.sh`, holds it on the shelf and in menus,
and lets it go in games. The user's network was entered again in slot's network menu; the
OS copy (`/etc/wpa_supplicant.conf`) is deleted and `dev.sh install` no longer writes one.
- **SSH during games:** a `slotsq-dev` file at the card's root makes the frontend export
  `SLOT_WIFI_KEEP=1`, and slot keeps Wi-Fi held in games too (sleep still releases it).
  It is there now. **Remove it for battery runs**, and log on the device.
- **Leaving a game** used to send `slotlink.sh link down` even with no link session.
  That also stops slot's own Wi-Fi (`link_clear` calls `wifi_stop`) while slot still
  believed it held it, so Wi-Fi stayed down until a reboot. Upstream never notices (it
  holds no Wi-Fi in games). Now only a session gets `link down`; otherwise just `cool`.
  **Step 4 must handle the same thing after a link session** with `SLOT_WIFI_KEEP`:
  `end_link` should let the Wi-Fi worker hold Wi-Fi again once `link down` has run.
- **Until slot joins a network there is no SSH.** If slot's Wi-Fi breaks, swap to the
  ROCKNIX card and read the dev card's `Logs/slot.log` from there.
- **Power save:** `/usr/share/udhcpc/default.script.d/power-save` turns it on whenever
  udhcpc gets a lease. Checked: on, in a game.
- **Network time:** slot calls `timedatectl` (`slot-power` `set_ntp`). slot² ships a
  stand-in, `/usr/bin/timedatectl`, that answers `show -p NTP --value` and `set-ntp`
  with BusyBox `ntpd` (`/run/ntpd.pid`); `/etc/slotsq/ntp-synced` writes the RTC after
  each sync. Checked by hand: it corrected a 5-minute drift. slot's "network time"
  setting is off on this card.
- **BusyBox** gained `ntpd` and `rfkill` (`board/rgb20sx/busybox.fragment`). A changed
  fragment needs `make busybox-dirclean` before `dev.sh build`.
- **Files: Gamebit** (menu, then Network, then Gamebit), slot's own web file manager
  (`crates/slot/src/server`, pages from `System/web`), serves the card on port 80 with a PIN
  and a QR code. It works on slot² as it is; the user is testing it in depth. No FTP or
  SFTP is planned unless Gamebit falls short (`gesftpserver` would add SFTP to Dropbear).
- **Checked by the user:** box-art fetch. RomM isn't used.
- **Not checked yet:** the radio really
  powering off in games without `slotsq-dev` (the battery run will show it). The boot
  logo refresh stays off (`BOOT_LOGO=0`).
- **Self-update is off** (`SELF_UPDATE=0`): no "Check for updates" row, since an
  upstream build can't drive this display (decision 5 below).

**Device-layer decisions (2026-11-05, evening):**
- Order: (1) a device description file the OS ships and slot reads, replacing the
  ROCKNIX checks; (2) slot owns Wi-Fi like upstream (the OS stops starting it); (3) time
  sync, box art, RomM; (4) link play.
- **SSH during games:** a development switch on the card keeps Wi-Fi on in games. Battery
  runs switch it off and log on the device, collected afterwards.
- **The saved network:** the user re-enters it in slot's network menu. The OS copy in
  `/etc/wpa_supplicant.conf` goes away in step 2.
- **Link play:** the RTL8723DS can't run two interfaces at once ("interface combinations
  are not supported"), so it can't use upstream's `wlan1` host. Our fork hosts on `wlan0`,
  on 2.4 GHz. A second device with vanilla slot arrives the week of 2026-11-05 for tests.
- The image lacks BusyBox `ntpd` and `rfkill`. The RTC (rk817) and `hwclock` work.
- **Later:** a port to the Retroid Pocket Classic (also near-square), so the device layer
  stays data-driven.

**Known test failures that predate our work** (they also fail on a clean checkout):
`slot-store` `header_index` `keeping_every_entry_leaves_the_index_saved`, `slot`
`header_index_boot` `boot_drops_index_entries_for_games_no_longer_on_the_card`, and
`slot-ui` `house_label::a_game_boy_label_is_the_shape_of_its_own_well`. The `render_*`,
`server_ui`, `pipeline` and `quad` tests need `HeadlessSurface`, which only builds on
macOS; `romm_ui` needs `--features romm`.

**Next, in order:**
1. **slot's device layer.** Steps 1 and 2 are done: slot reads `/etc/slot-device`
   (`crates/slot/src/hardware.rs`; keys `DISPLAY`, `BUTTONS`, `FN_ADC`, `CPU_CAP`, `WIFI`,
   `LEFT_STICK`, `BOOT_LOGO`), falling back to the ROCKNIX preset or BaseOS, and it runs
   Wi-Fi. Step 3: check box art and RomM, point self-update at our builds some day, and fix the update scenes, which still draw their carts in band
   coordinates.
2. **A battery run on our kernel without `slotsq-dev`** (CPU idle states may differ from
   ROCKNIX's; the radio should now be off in games, worth 0.13-0.19 W, PLAN.md).
3. **Link play (step 4)** when the second device (vanilla slot) arrives: host on `wlan0`
   on 2.4 GHz, power save off during play.
4. **Our own U-Boot** (no 2 s countdown).
5. **Label printer jump (reported 2026-11-05):** the label printer (Select on the shelf)
   jumps up before it opens on the square panel, as the core picker did. Fix it the same
   way: take `first = out.len()` once its band layout is computed and
   `shift(&mut out[first..], canvas.extra() / 2.0)` (see `draw_core_picker`).
6. Later: more menu rows on a tall canvas (`QUICK_VISIBLE` is 6), a crisper wallpaper,
   then systems from slotPlus.
- **Parked by the user:** boot time (11-12 s power-on to shelf is fine for now; analysis
  in BRINGUP.md).

## Where things stood after the ROCKNIX work

**slot runs well on the RGB20SX under ROCKNIX 20261101**, started from SSH. It boots to
the shelf in about 0.6 s, every button including FN works, games save, and link play and
Wi-Fi features are hidden. Frame pacing is solved.

**Battery work is done for the ROCKNIX layer.** In-game medians (`tools/rocknix/medians.py`):

| Setup | mGBA | gpSP | Gambatte | Menu |
|---|---|---|---|---|
| `performance` CPU + GPU | 2.85-3.0 W | | | 2.2-2.3 W |
| clock cap v2 (`419dc49`), GPU on `simple_ondemand` | 2.66 W at 1800 | 2.49 W at 1800 | 2.33 W at 1800 | 1.8 W at 408 |
| CPU-bound rule + 400 MHz GPU floor (today's default) | 2.66 W at 1800 | 2.13 W at 1104 | 2.03 W at 1104 | 1.8 W at 408 |
| same, direct ALSA instead of PipeWire | | | 1.95 W at 1104 | ~1.75 W |

The fixed baseline is about 1.8 W, so what's left to win is mostly in the baseline,
which only our own OS can cut. mGBA's CPU part alone is about 6 ms on average and up to
10 ms at 1800 MHz, so 1800 MHz is its floor.

**Current focus: our own minimal OS, slot²** (`tools/distro/PLAN.md`). Phase 0 is done:
direct ALSA and the Mali blob. Phase 1: our own Buildroot root boots from the dev card
(SSH ready 6.73 s after kernel start). `tools/distro/BRINGUP.md` has the full recipe and
every trap found on the way. Read it before touching the boot setup. Work with
`tools/distro/dev.sh`. Systems are switched by **swapping the two cards** (the dev card in
slot 1 boots slot²; the ROCKNIX card in slot 1 boots ROCKNIX). ROCKNIX's games card
auto-detect is off (`system.automount=0`) so it leaves the dev card alone.

## What this session found and changed

Commits, oldest first:

- `083f6cd` restores upstream's `tests/clock.rs` (35 clock-screen tests that `149263d` had
  overwritten) and moves our CPU-cap tests to `tests/cpu_clock.rs`.
- `9a8f963` makes only CPU-bound frames raise the CPU cap. A frame is CPU-bound when its
  CPU part (everything but the GPU wait, including the wait for the emulator thread) is
  at least as long as its GPU wait. The 600-frame pace line now logs frame, CPU part and
  GPU wait (average and max), misses split into CPU-bound and GPU-bound, and the cap.
- `1e8418a` gives the GPU a 400 MHz floor through `min_freq` (`SLOT_GPU_MIN_MHZ`, `keep`
  turns it off).
- `4698d92` jumps the cap to the top step while a game launches (inserting, core, save and
  state loading) and leaves frames out while a game is put away (its save takes one
  80-130 ms frame).
- `d7a7725` adds `SLOT_ALSA_DEVICE` and the launch script's `SLOT_AUDIO=alsa`.
- `a1834bf` and `cfd4eb6` hold the OS plan and phase 0's results.

Findings:

- **The GPU idled at 200 MHz** on `simple_ondemand`, which averages load over 100 ms, so
  slot's short burst each frame looked idle. The GPU wait then took two thirds of a
  Gambatte frame, and the old rules pushed the CPU to 1800 MHz. The CPU-bound rule plus
  the floor fixed it (gpSP 2.49 to 2.13 W, Gambatte 2.33 to 2.03 W).
- **ROCKNIX's own 400 MHz GPU floor never applies on RK3566.** Quirk `060-game_settings`
  runs at about 14.5 s, before `003-gpudriver` loads `mali_kbase` at about 16.1 s, so its
  `min_freq` write fails on every boot.
- **ROCKNIX's `powerstate` service resets the GPU governor on every charger change** but
  never touches `min_freq`. That's why the floor beats a `performance` governor.
- **There is no DMC devfreq on RK3566**, so `SLOT_GPU_GOVERNOR` only changes the GPU.
- **FN root cause:** ROCKNIX's kernel patch `0004-input-adc-keys-redirect-keycode-316...`
  sends `BTN_MODE` to a rocknix-joypad pointer that is NULL on RK3566, so the kernel drops
  it. `tools/rocknix/rocknix-fn-report.md` is updated with this, still unposted.
- **Audio:** direct ALSA is 80 mW cheaper than PipeWire and had no underruns after the
  start. The headphone jack switches in hardware (verified by ear). The speaker channels
  are in the right order, despite a device-tree comment saying they are reversed. Every
  run starves 6144 samples once at startup. The audio blip the user heard once in mGBA
  was not found, and is left for the minimal OS.
- **GPU drivers:** Panfrost (Mesa 26.2.4) runs slot fine but its GPU wait is longer (4.4
  against 3.5 ms) and the first frame is slower (1.16 s). We keep the Mali blob for now.
- **Launch and quit** still miss a few vblanks, all in single 25-130 ms frames spent
  loading or saving on the main thread. Clock speed can't fix those.
- **Link play across devices:** slot hosts on 5745 MHz, but the RGB20SX's RTL8723DS is
  2.4 GHz only. An RG SP can join an RGB20SX host (its fallback scans every channel), but
  the reverse needs the RG SP to host on 2.4 GHz.
- **Boot:** ROCKNIX takes 29 s (2.4 s kernel, 26.8 s userspace, of which `userconfig`
  9.2 s and `rocknix-autostart` 14.7 s). U-Boot loads `/flash/KERNEL` (kernel plus a
  built-in initramfs that always starts systemd) and the RGB20SX dtb, and the root is the
  `/flash/SYSTEM` squashfs. There is no kexec.
- **Second card slot:** `mmcblk2` (`fe2c0000.mmc`). It currently holds an old 64 GB Miyoo
  Onion card, mounted by ROCKNIX as its games folder. The user said nothing on it is
  worth saving, so it will become the dev card for phase 1. Say so before writing to it.
- A Sony DualSense was paired over Bluetooth during the session. It shows up as an input
  device.

## Dev loop

The WSL2 Ubuntu machine is set up: `gcc-aarch64-linux-gnu`, Rust 1.96.1 with the
aarch64 target, and an SSH key (`~/.ssh/id_ed25519`) installed on the device. In each
command, run `. "$HOME/.cargo/env"` first, since it isn't in `~/.bashrc`.

```sh
sh tools/rocknix/dev.sh build      # cross build, ~35 s clean, ~15 s after changes
sh tools/rocknix/dev.sh deploy     # copies System/slot and System/rocknix.sh
sh tools/rocknix/dev.sh logs NAME  # -> logs/slot-NAME.log, logs/power-NAME.log (gitignored)
python3 -I tools/rocknix/medians.py NAME [NAME...]   # in-game medians for saved runs
```

Running a measured session from an agent shell, which has no terminal for `ssh -t`:

- Start it in the background:
  `ssh -o ServerAliveInterval=15 root@192.168.1.72 "SLOT_TRACE_LATENCY=1 SLOT_TRACE_POWER=1 bash /storage/slot/System/rocknix.sh"`.
  Add `SLOT_TRACE=1` for audio lines, `SLOT_AUDIO=alsa`, or `SLOT_GPU_GOVERNOR=...` as
  needed.
- The user plays: about 20 s on the shelf, then the game for about 3 minutes, then quits
  through the in-game menu. If a session is stopped mid-game, slot resumes straight into
  that game next time.
- Stop it with
  `ssh root@192.168.1.72 'pkill -TERM -f "bash /storage/slot/System/[r]ocknix.sh"'`.
  The `[r]` keeps the pattern from matching the pkill's own SSH command, which killed its
  own session the first time.
- Never deploy `rocknix.sh` while a run is going. Overwriting a script that bash is
  executing can break the run.
- Games on the device: Castlevania Aria of Sorrow (GBA; its core, mGBA or gpSP, is chosen
  in slot's menu), Cave Noire (GB), Monster Rancher Explorer (GBC).
- `gpudriver panfrost` or `gpudriver libmali`, then a reboot, switches ROCKNIX's GPU
  driver. It's back on `libmali`.

## Conventions and gotchas

- **Commits:** plain descriptive messages, with no `Co-Authored-By` or `Claude-Session`
  trailers. The user will add attribution if the code goes public.
- **Commit identity:** WSL has no git identity. Commit as the user with
  `GIT_AUTHOR_NAME="Serj Targarien" GIT_AUTHOR_EMAIL=xergy10@gmail.com` and the matching
  `GIT_COMMITTER_*` variables. Push only when asked.
- **Formatting:** do not run `cargo fmt --all`, since upstream isn't rustfmt-clean
  (`session.rs` and `frontend.rs` already differ). Run `rustfmt --edition 2021 --check` on
  the files you touch, and compare against `git show HEAD:file | rustfmt --emit stdout`
  when unsure.
- **Tests:** those using `HeadlessSurface` only build on macOS. On Linux, run:

  ```sh
  cargo test -p slot --test cpu_clock --test clock --test rocknix --test rocknix_input --test evdev --test quick_menu --test in_game_menu
  cargo test -p slot-gfx --test kms --test fbdev
  cargo test -p slot-power --test device
  sh card/test-rocknix.sh
  cargo clippy -p slot --no-default-features --features device -- -D warnings
  ```

- **ROCKNIX paths:** every ROCKNIX-specific path is gated on `slot::rocknix::running()`.
  The BaseOS/H700 path must keep working unchanged.
- **The user:** on Windows (WSL2), comfortable with SSH and git, prefers measured
  decisions, and wants to be consulted before any decision with a cost or that's hard to
  undo. Ask before changing anything on the device beyond `/storage/slot` and running the
  launch script. A Windows-side clone at `C:\Users\xergy\Documents\GitHub\slot-alt`
  shows every file as modified, but those are only line endings. Older CI artifacts are in
  `C:\Users\xergy\Downloads\slot-rocknix-*.zip`.

## Upstream and related projects

- **Upstream slot** has no CPU or GPU clock management. Issue #162 identified the 12 ms
  emulation deadline as the battery problem and suggested setting the clock from missed
  deadlines, which is what `clock.rs` does. The maintainer doesn't plan to support other
  devices (#146, #173). Our earlier PR #197 to upstream was closed unmerged and is still
  public.
- **BaseOS** (`pvaibhav/BaseOS`): upstream's H700 base, and the model for our OS. Vendor
  kernel 4.9 and blobs reused unchanged, BusyBox init, plain ALSA, `wpa_supplicant`,
  unsigned A/B `.bosupd` updates. No license file.
- **slot-rg353v** (`mriverodorta/slot-rg353v`, GPL-3.0): an RK3566 slot image built with
  ROCKNIX's build system and systemd. About 4.3 s to the shelf on eMMC.
- **slotPlus** (`nelynes/slotPlus`): a slot fork adding WonderSwan, Lynx, Game Gear,
  NeoGeo Pocket, TurboGrafx-16 and DS. A few of those come after our OS works (plan phase
  4b).
- **ROCKNIX source** was checked at `e344c8109029ccd1bb9019680c1cd3d5357d56da`. Useful
  paths are under `projects/ROCKNIX/`:
  - `packages/sysutils/busybox/scripts/init` (the initramfs)
  - `packages/kernel-drivers/device-tree-overlays` (`dtb_overlay`, including
    `rk3566-undervolt-cpu-l1..l3`)
  - `packages/sysutils/powerstate`
  - `devices/RK3566/linux/linux.aarch64.conf`
