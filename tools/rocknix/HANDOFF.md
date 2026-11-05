# Handover: slot on the Powkiddy RGB20SX

Last updated 2026-11-05, end of the first local session. Read this first, then
`tools/distro/PLAN.md` (where the project is going) and `tools/rocknix/README.md`
(measured device facts and what the ROCKNIX port does).

Branch: `claude/beautiful-feynman-ffzfnx` (fork `SerjTargarien/slot-alt` of
`BrandonKowalski/slot`). Upstream `main` was still exactly our fork point `f1f6ef9` on
2026-11-05. Commits from this session are local only, not pushed.

## Resume here (paused 2026-11-05 for lunch)

**Device state when we stopped:**
- The **dev card is in slot 1** and running **slot²**, our own OS. It boots straight
  into slot.
- The **ROCKNIX card is in slot 2**, unused and unmounted. It boots ROCKNIX again when
  put back in slot 1.
- SSH into slot²:
  `ssh -o UserKnownHostsFile=~/.ssh/known_hosts_slotsq root@192.168.1.72`.
  It needs that separate known_hosts file, since the address is the same as ROCKNIX's
  and the host key differs.
- **The card's slot build is current:** `System/slot` is the HEAD build (`e5e8386`),
  with the square layout and the frame dump. `System/slot.band` is an older build kept
  only for reference.
- **The card's root** has the HEAD overlay pushed by hand; the image in
  `~/slotsq-build/output/images/rootfs.tar` matches it.

**What we were doing:** making slot use the whole 720x720 screen. The shelf is done
(`e5e8386`): panels taller than 4:3 are now laid out "framed" like the 640x480 devices,
on a 480x480 layout canvas drawn at 1.5x, with games able to use the whole panel. The
footer readouts are centred beside the slot's bay.

**Our own kernel works** (2026-11-05 afternoon, BRINGUP.md attempts 6 and 7): Linux
7.2.11-slot² with our own device tree. Picture, speaker and headphones, every button,
FN once per press, brightness and slot's hotkeys, and Wi-Fi all work. The GBA audio that
sounded out of tune on ROCKNIX's kernel sounds right now. The dev card boots it by default
(`extlinux.conf`); `extlinux.conf.rocknix-kernel` on the SLOT partition boots ROCKNIX's
kernel again (`dev.sh kernel-back`). A fresh dev card gets our kernel from
`dev.sh bootable`. Still borrowed from ROCKNIX: U-Boot, libmali and its wrappers, and the
Wi-Fi firmware and regdb (PLAN.md decision 6).

**Next, in order:**
1. **Screen tour.** The user opens each screen and names it, and we capture it with
   `sh tools/distro/dev.sh shot NAME` (saved to `~/slotsq-build/frames/`, readable as
   an image). For a before and after of one screen, use
   `dev.sh capture NAME SLOT_TALL_PANELS=0` (the old band layout) against
   `dev.sh capture NAME`, though that restarts slot, so it suits the shelf only.
   - Screens to cover: GB and GBA in game, with each picture size and bezel; the
     in-game menu (FN) and its settings; screenshots and polaroids; the clock screen;
     the power menu; the eject animation.
   - Expected trouble: the on-screen keyboard (`keyboard.rs` uses a fixed
     `Canvas::of((720, 480))`) and anything else laid out for 720 units of width.
   - Already reported by the user:
     - GB: bezel "default" with SP scale looks great; bezel "none" with any scale looks
       great; stretch grows the picture only horizontally.
     - GBA: stretch grows it only vertically. AGS-102 sits the picture toward the top,
       which is slot's intended layout for that bezel.
   - Later, optional: UI text and art are rasterised at layout size, then drawn 1.5x
     with linear filtering, so they're a little soft. Rasterising at 1.5x would make
     them crisp, but that's a wider change.
2. **GBA audio:** it sounded out of tune on ROCKNIX's kernel under slot², and right on
   ours. Keep an ear on it.
3. **Wi-Fi radio off when slot doesn't need it,** and power save off during link play
   (measured: 0.13-0.19 W, see PLAN.md).
4. **slot's device layer** (replace `SLOT_PLATFORM=rocknix`; the `timedatectl` call),
   then **our own kernel** (phase 1b) and **U-Boot** (no 2 s countdown). The OS gets
   its real name with the first proper version: "slot²" is a working name.
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
