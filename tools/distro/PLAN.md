# Plan: a minimal OS for slot on the RGB20SX

Status: direction agreed 2026-11-05 (option C). Phase 0 done: direct ALSA, the Mali blob.
Phase 1 done: our root boots and SSH logins work (init at 1.97 s, SSH at 6.7-9.7 s,
54 MB RAM). Phase 2 under way: our slot fork runs on rgbslot, started by hand (picture, buttons, FN,
sound, clock cap all work). Next: start it at boot. How it boots, and every
trap found on the way, is in `BRINGUP.md`.

## Goals

- As light and focused as possible, while keeping everything that makes slot feel like a
  proper console OS: a simple, snappy interface and multiplayer across devices.
- It should feel like slot on a different device. The more compatible with upstream slot,
  the better: same cards, same saves, same link play.
- Once all of that works well, add a few more systems from slotPlus
  (`nelynes/slotPlus`, a fork of slot that adds WonderSwan, Lynx, Game Gear, NeoGeo Pocket,
  TurboGrafx-16 and DS). Still focused and limited.

Today slot runs as a layer on ROCKNIX 20261101: a launch script stops EmulationStation,
sway, InputPlumber and the hotkey daemon, then hands the device back. The goal is an OS
whose only job is slot: it boots straight to the shelf, runs nothing slot doesn't need,
and draws less battery than ROCKNIX.

## Where we start (measured on the device)

| | ROCKNIX 20261101 on the RGB20SX |
|---|---|
| Boot to frontend target | 29.2 s: kernel 2.4 s, userspace 26.8 s, of which `userconfig` 9.2 s and `rocknix-autostart` 14.7 s. EmulationStation starts after that. |
| Boot chain | U-Boot, then `/flash/extlinux/extlinux.conf`, then `/flash/KERNEL` (30 MB, kernel plus an initramfs) and `rk3566-powkiddy-rgb20sx.dtb` |
| Root | `/flash/SYSTEM`, a 1.35 GB squashfs mounted read-only as `/`. The initramfs always hands over to `/usr/lib/systemd/systemd`. |
| Partitions | `mmcblk1p1` FAT (`/flash`, 2 GB), `mmcblk1p2` ext4 (`/storage`) |
| Modules loaded | 17: `mali_kbase`, rtw88 Wi-Fi, Bluetooth, `hantro_vpu`, `rockchip_rga`, ntfs |
| GPU | `libmali` (57 MB) over `mali_kbase`. Panfrost and Mesa 26.2.4 are in the image too. |
| Audio | PipeWire on the `rk817ext` ALSA card |
| What slot needs | glibc and `libgcc_s` at link time. At runtime it loads `libEGL`, `libGLESv2`, `libgbm`, `libdrm` and `libasound`. Plus the three libretro cores. |

Battery while playing, with slot on ROCKNIX: menu about 1.8 W, Gambatte 2.03 W, gpSP 2.13 W,
mGBA about 2.7 W. The fixed baseline is about 1.8 W, so most of what is left to win is in
the baseline (services, Wi-Fi, GPU power policy, voltages), not in slot's frame loop.

## Prior art

- **BaseOS** (`pvaibhav/BaseOS`, the base upstream slot uses on H700): no build system to
  speak of. It reuses the vendor's U-Boot, kernel 4.9 and blobs unchanged, harvested from
  the stock image, and adds a static BusyBox userspace. BusyBox init respawns a
  `frontend-session` that starts `System/launch_frontend.sh` or `System/slot`. It has no
  udev, plain ALSA, `wpa_supplicant` and `udhcpc`, and `/usr/lib` as an LZ4 squashfs.
  A/B root slots with a 3-boot trial roll back a bad `.bosupd` update. It reports about
  3.2 s to the frontend. No license file. Its binaries don't apply here, but the design does,
  and slot's BaseOS code paths (Wi-Fi through `wpa_cli`, ALSA, updates) expect that contract.
- **slot-rg353v** (`mriverodorta/slot-rg353v`, GPL-3.0, same SoC and the same libmali and
  GBM stack): builds a full image with ROCKNIX's build system pinned at `20261001`, keeps
  ROCKNIX's initramfs and systemd, slims the kernel and prunes userspace. It reports
  4.3 s (eMMC) to 5.2 s (SD) from Linux start to the shelf. Costs: builds take hours on
  an ARM64 Docker host with 24 GB of RAM, and its slot changes are a 17k-line patch.
- Nothing else boots an RK3566 handheld straight into one app.

## Options

**A. Stay a ROCKNIX layer, trimmed.** Start slot at boot in place of EmulationStation
(through `UI_SERVICE`) and mask services we don't need. Little work, but boot stays near
25 s, ROCKNIX updates can undo our changes, and services like `powerstate` keep fighting
slot. It doesn't meet the goal.

**B. Fork ROCKNIX's build system, as slot-rg353v does.** Proven on this SoC and gets to
about 5 s, but it keeps systemd and ROCKNIX's whole build and package tree. Every build is
hours, and it can't easily run on this WSL machine.

**C. Our own small root filesystem, BaseOS style (recommended).**
- Kernel side, harvested from a pinned ROCKNIX release at first: U-Boot, kernel, device
  tree, the matching modules, Wi-Fi firmware and `libmali`. Later we build the kernel
  ourselves from ROCKNIX's config and patches.
- Userspace built with Buildroot: glibc, BusyBox init, ALSA, `wpa_supplicant`, Dropbear
  for development, and slot with its cores.
- No systemd, no udev, no PipeWire.

This is the smallest system and the fastest to iterate on. It runs on this machine and
matches upstream slot's BaseOS contract.

The open question for C is the kernel. ROCKNIX's `KERNEL` has its initramfs built in, and
that initramfs only knows how to start systemd. The kernel falls back to mounting `root=`
and running `init=` when the `rdinit` program is missing, so `rdinit=/none root=... init=/sbin/init`
may let us boot ROCKNIX's exact kernel binary with our own root. Phase 1 tests that first,
because it decides whether we need a kernel build on day one.

## Phases (each ends with a measurement)

**0. Decide the GPU and audio stacks on ROCKNIX first.** These are cheap experiments on the
current card, and they change what the new image must contain.
- Panfrost against `libmali`: switch with ROCKNIX's `gpudriver` and compare slot's frame
  pacing and power. Panfrost would drop the 57 MB blob and the `always_on` GPU power
  policy, and is fully open.
- ALSA direct against PipeWire: give slot `plughw:CARD=rk817ext` and compare latency and
  the audio underruns at launch and quit.
- Exit: a measured choice for each, recorded in this file.
- **Audio, decided: direct ALSA.** Gambatte with the 400 MHz GPU floor, against run E on
  PipeWire: 1.95 W in play instead of 2.03 W at the same 1104 MHz, and 0 samples dropped
  or starved after the start, against 5466 dropped and 4202 starved. 0 misses in all 13
  play windows. Sound was clean and loud at the top volume, without distortion.
  - The headphone jack switches in hardware. Plugging in moves the sound to the headphones
    and unplugging brings it back to the speaker, with no software involved. Our system
    needs no headphone helper.
  - The speaker channels are in the right order. A device-tree comment says they are
    reversed, but a left-channel test played from the left speaker.
  - Every run, on either path, starves 6144 samples once at startup, before the game's
    first sound.
- **GPU, decided for now: the Mali blob.** Gambatte on direct ALSA with the 400 MHz floor:
  Panfrost (Mesa 26.2.4) worked with no visible problems (shader on, no bezel) and the same
  1104 MHz CPU, but its GPU wait averaged 4.4 ms against 3.5 ms. It drew 1.98 W against
  1.95 W and took 1.16 s to the first frame against 0.6 to 0.74 s, probably shader
  compiles with no warm cache yet. Panfrost stays an option for a fully open image, to
  measure again on our own system with a warm shader cache.
- Tools: `SLOT_ALSA_DEVICE` names slot's ALSA device. On our own system it is required,
  because card 0 is HDMI and the speaker is card 1 (`rk817ext`). `SLOT_AUDIO=alsa` in the
  launch script stops PipeWire for the session and points slot at
  `plughw:CARD=rk817ext,DEV=0`.

**1. Boot our root on a second card.** Keep the ROCKNIX card untouched as the fallback.
- A card with ROCKNIX's U-Boot, kernel and device tree, plus a Buildroot root with BusyBox
  init that brings up the panel console, loads the modules and starts Dropbear over Wi-Fi
  (or USB gadget networking).
- Try ROCKNIX's kernel with `rdinit=/none` first, and build the kernel only if that fails.
- Exit: SSH into our own system. Kernel-to-shell time measured.
- **Dev setup used: card swaps** (see `BRINGUP.md`). The dev card boots on its own
  from slot 1. The one-shot entry described next looped the device on a kernel panic
  and is kept only as a record.
- **Dev setup first proposed (superseded):** the dev card stays in the second slot (`mmcblk2`) and the
  ROCKNIX card stays in the first, so no card swapping is needed.
  - **Boot entry:** a second entry in TF1's `extlinux.conf` reuses ROCKNIX's own `KERNEL`
    and dtb. Its command line is ROCKNIX's, plus
    `rdinit=/nonexistent root=/dev/mmcblk2p2 rootwait init=/sbin/init`.
  - **If `rdinit` is honoured,** the kernel skips ROCKNIX's initramfs and starts our init
    from the dev card.
  - **If it isn't,** ROCKNIX's initramfs still finds `boot=` and `disk=` and boots ROCKNIX
    as usual, so the experiment can't strand the device.
  - **One-shot switch:** a script on ROCKNIX makes the dev entry the default and reboots.
    Our init's first act is to put ROCKNIX back as the default, so the next power cycle
    always returns to ROCKNIX.
  - **Remaining risk:** if the kernel can't mount our root, it panics before our init runs
    and keeps rebooting into the dev entry. Before every switch, the script mounts the
    dev root from ROCKNIX and checks `/sbin/init`. Recovery is one line in
    `extlinux/extlinux.conf` on TF1's FAT partition, editable from Windows, and a backup
    sits next to it.
  - **Alternative with no change to TF1:** make the dev card self-contained (ROCKNIX's
    U-Boot copied to it) and swap the two cards to boot it.
  - **Dev card layout:** p1 FAT32 `SLOT`, the card contents slot sees (`/mnt/sdcard`,
    BaseOS style); p2 ext4 for the root (512 MiB); p3 later for the second A/B root.

**2. Boot straight into slot.**
- Add the GPU stack chosen in phase 0, ALSA, the cores, and a `frontend-session` that
  starts and respawns slot.
- Teach slot a new platform id. Most ROCKNIX-specific code stays (KMS, gamepad table, FN
  through the ADC, the clock cap). The ROCKNIX service juggling and the GPU floor
  workaround become unnecessary.
- Exit: power-on to the shelf timed, with a target under 5 s. Menu and in-game power
  compared with the ROCKNIX numbers above.

**3. Power work on our own base.**
- A Wi-Fi radio off by default when not needed.
- CPU undervolt with ROCKNIX's tested `rk3566-undervolt-cpu-l1` overlay (408 to 1104 MHz
  at 800 mV).
- The GPU power policy, the clock-cap thresholds, and sleep.
- Exit: the baseline below 1.8 W, measured.

**4. Features back.** Wi-Fi through slot's existing `wpa_cli` path, the web server, RomM,
rumble if the hardware has it, and the 720x720 layout. Link play across devices:
- **How it works today:** the host runs its own network through `wpa_supplicant` in AP
  mode on 5745 MHz (`card/System/slotlink.sh`). The joiner tries that frequency, then
  every channel.
- **The RGB20SX's RTL8723DS is 2.4 GHz only.** It must host on a 2.4 GHz channel. An RG SP
  can still join, because its fallback scans every channel.
- **To join an RG SP host,** that host needs to use 2.4 GHz. That's a small upstream change
  or setting to propose (the RG SP's RTL8821CS does both bands).
- **Both sides need** the same mGBA and gpSP builds and the same slot link protocol, so our
  slot stays close to upstream.
- The image needs `wpa_supplicant` built with AP mode, `udhcpc` and `ip`.

**4b. slotPlus systems.** A few of slotPlus's systems, chosen for the 720x720 screen and the
RK3566's speed, built and pinned like the current three cores.

**5. Updates and releases.**
- A/B root slots with a trial boot and rollback, as BaseOS does, but signed.
- A CI job that builds the image and the update file.
- Pin and track the ROCKNIX release we harvest from, or our own kernel build.

## Phase 2 results so far (2026-11-05)

**Upstream slot v1.14.0 (`slot-v1.14.0-h700.zip`) on rgbslot, unchanged,** run through its
own `launch_frontend.sh` with `SLOT_ROOT=/mnt/sdcard/upstream`:

| Layer | Result |
|---|---|
| Launch contract, boot logo step | ran. `--install-bootlogo` only looks for a GPT partition named `boot-resource`, found none and wrote nothing |
| Platform | read the 720x720 panel from fb0, backlight, battery, RTC |
| Display | failed silently: the surface was created and the first frame drawn at 2.3 s, but nothing reached the panel (Mali GBM blob, no fbdev path) |
| Buttons | opened pwrkey, gpio-keys-control and gpio-keys-vol with the Anbernic table; no adc-keys, no FN |
| Wi-Fi | slot owns it: it ran `slotlink.sh wifi start Config/wifi.conf`, which restarts `wpa_supplicant` with the card's own (here empty) network list, dropping our connection about 40 s after start |
| Audio | started with no error, but `plug:default` is card 0, which is HDMI |

What follows for the OS:
- Display and buttons need slot's device layer, as decided.
- Once slot runs, the OS must not run its own `wpa_supplicant`. Saved networks belong in
  the card's `Config/wifi.conf`, slot's own list.
- `/etc/asound.conf` must point `default` at `rk817ext`.

**Our fork on rgbslot** (`System/` copied from the ROCKNIX card, binary `06e6b5c2`), started
with `SLOT_ROOT=/mnt/sdcard SLOT_PLATFORM=rocknix SLOT_ALSA_DEVICE=plughw:CARD=rk817ext,DEV=0`
and the GPU floor written by hand (`min_freq` 400 MHz):
- Picture, buttons, FN (through the ADC), sound from the speaker and the clock cap all
  work. The user found nothing off.
- First frame 2.29 s after slot starts. Surface creation takes 1.1 s, against 0.03-0.08 s
  under ROCKNIX: probably the cold 57 MB libmali read from a card now running at 50 MHz.
- One ROCKNIX leftover: slot calls `timedatectl set-ntp`, which doesn't exist here.

## Decisions (2026-11-05)

1. Option C is the direction. B is the fallback if the kernel route gets stuck.
2. Phase 0 may switch ROCKNIX's GPU driver and audio on the current card.
3. A spare microSD card holds the new system. The ROCKNIX card stays as it is.
4. Platform contract: follow the BaseOS shape (a release file, `System/launch_frontend.sh`,
   `/mnt/sdcard`, `wpa_cli`, ALSA) where it fits cleanly, under our own name, so the device
   feels like slot on a different device. Where BaseOS assumptions don't fit, a small
   platform layer in slot rather than a mix of three systems.

5. **How slot fits (2026-11-05):** an unmodified slot or slotPlus `System/` folder can't
   fully run here. The OS can provide launch, cores, ALSA, Wi-Fi and the link frequency
   (`AGS_LINK_FREQ=2437`) exactly as BaseOS does. Display (the Mali GBM build, no fbdev
   winsys) and buttons (gamepad codes, FN only through the ADC) need slot's help.
   - **Direction:** a small device layer in slot, driven by data from the OS release file
     (display backend, input layout, FN sensor, link frequency, panel size), replacing the
     `rocknix::running()` checks. It is kept as an isolated patch set that slotPlus can
     take too.
   - **Maintenance:** maintaining our own fork, focused on this device and merging
     upstream releases, is accepted. It gives us control and doesn't depend on upstream.
   - **Self-update** must point at our builds, or stay off on rgbslot, until upstream
     carries the layer. An upstream build would otherwise replace a working one with one
     that can't drive the screen.
   - **OS shims** for unmodified builds (a `libEGL` shim and a button remapper) are not
     planned.
   - **Phase 2 starts with an experiment:** an official upstream `System/` folder on the
     dev card, to see exactly what breaks.
6. **Our own kernel before anything is published (phase 1b).** Today's test boots
   ROCKNIX's `KERNEL` to reach SSH fastest. Then we take, step by step: our own kernel
   (mainline 7.2.x plus only the ROCKNIX RK3566 patches the RGB20SX needs, without their
   FN redirect patch), `mali_kbase` from Arm's GPL source, firmware and regdb from
   Buildroot's packages, libmali from Rockchip's repo (or plain symlinks if slot is fine
   with them), and later our own U-Boot. Then no ROCKNIX binary remains, only their
   patches, credited.

## What phase 1 borrows from ROCKNIX 20261101

| What | Where | Licence | Replaced by |
|---|---|---|---|
| U-Boot plus Rockchip DDR/BL31 blobs | ROCKNIX card boot area, in place | GPL-2.0 plus Rockchip binaries | our mainline U-Boot (phase 5) |
| `/flash/KERNEL` 7.2.11, with ROCKNIX's initramfs (skipped by `rdinit`) | ROCKNIX card, in place | GPL-2.0 | our kernel (phase 1b) |
| `rk3566-powkiddy-rgb20sx.dtb` | ROCKNIX card, in place | GPL-2.0/MIT | built with our kernel |
| 288 modules, including `mali_kbase.ko` r54p2 | copied into the root | GPL-2.0 | our kernel, and kbase from Arm's source |
| `rtw88/rtw8723d_fw.bin`, `regulatory.db` | copied | redistributable (linux-firmware, wireless-regdb) | Buildroot packages |
| `libmali.so` g29p1 GBM | copied | Arm Mali EULA `LES-PRE-20769` | Rockchip's libmali repo (read the EULA before publishing) |
| `libmali-hook` and the `libEGL`/`libGLESv2`/`libgbm` wrappers | copied | Rockchip libmali wrapper code, patched by ROCKNIX | built ourselves, or symlinks |

## Risks

- **No serial console** without opening the device (UART on `ttyS2`). Early boot problems
  will only show on the panel (`console=tty0`) or in a log on the card.
- **Harvested blobs:** redistribution terms for `libmali` and the Wi-Fi firmware need
  checking before anything is published.
- **Drift:** ROCKNIX moves fast. We pin one release and update on purpose.
- **Cores** may need extra libraries (`libstdc++` and the like). Check each core's
  `NEEDED` before building the root.
