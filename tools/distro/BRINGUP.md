# rgbslot bring-up notes: how to build and boot it from zero

Everything learned while first booting rgbslot on the Powkiddy RGB20SX (2026-11-05),
written so the OS can be rebuilt in one go. `PLAN.md` says where the project is going.
This file says how the pieces actually fit, and which traps cost time.

Status: our own kernel and root boot from the dev card (attempt 7 below). Init starts at 1.97 s after kernel start,
and SSH is ready at 6.73 s, against ROCKNIX's 29 s to its frontend. It still uses
ROCKNIX 20261101's kernel, modules, U-Boot and libmali (phase 1). Our own kernel is phase
1b.

## The recipe

Run from WSL2 Ubuntu while the device runs ROCKNIX, with the ROCKNIX card in slot 1 and
the dev card in slot 2:

```sh
# once: Buildroot next to the repo, and its device-tree tools
mkdir -p ~/rgbslot-build && cd ~/rgbslot-build
git clone --depth 1 --branch 2026.08.1 https://gitlab.com/buildroot.org/buildroot.git
cd ~/slot-alt
sh tools/distro/dev.sh harvest        # modules, firmware, libmali from ROCKNIX (75 MB)
sh tools/distro/dev.sh build          # first build ~30 min (toolchain), then ~1-2 min
(cd ~/rgbslot-build/buildroot && env -i HOME=$HOME PATH=/usr/sbin:/usr/bin:/sbin:/bin \
	make O=../output host-dtc)        # fdtput for the dtb patch
sh tools/distro/dev.sh format --yes   # erases the card in slot 2
sh tools/distro/dev.sh install        # root to p2, Wi-Fi network copied from ROCKNIX's iwd
sh tools/distro/dev.sh bootable       # bootloader, kernel, patched dtb, initrd, extlinux
```

Then switch off, put the dev card in slot 1 and the ROCKNIX card in slot 2, and switch
on. Swap back to return to ROCKNIX. Once rgbslot is up, it's reachable as
`ssh root@<its address>` (its address is in `rgbslot-boot.log` on the SLOT partition).

On ROCKNIX, set **"games card auto-detect" off** first (`system.automount=0`). Otherwise
ROCKNIX mounts the dev card's first readable partition as its games storage and writes
136 `roms` folders onto it. In our case that was the ext4 root.

## Hardware and boot chain facts

- **Card slots:**
  - Slot 1 is `mmc@fe2b0000` (sdmmc0), which the kernel calls `mmcblk1`. The boot ROM
    boots only from this slot.
  - Slot 2 is `mmc@fe2c0000` (sdmmc1), `mmcblk2`.
  - Device names follow the slot, not the card.
- **Boot area (ROCKNIX card, MBR):**
  - Rockchip idbloader (`RKNS`) at sector 64.
  - U-Boot FIT (`d00dfeed`) at sector 16384 (8 MiB).
  - p1 starts at sector 32768 (16 MiB), is FAT32 and carries the **boot flag**.
  - U-Boot reads `extlinux/extlinux.conf` from p1.
- **Making a card bootable:** copying sectors 64 to 32767 from the ROCKNIX card is
  enough, never sector 0 (the MBR). Then set the boot flag on p1 and put `KERNEL`, the
  dtb and `extlinux/extlinux.conf` there. ROCKNIX's U-Boot honours `LINUX`, `FDT`,
  `INITRD` and `APPEND`.
- **Dev card layout:**
  - MBR, disk signature `3082b99d`.
  - p1 FAT32 `SLOT` (16 MiB to end - 1 GiB), boot flag, holding the boot files and slot's
    card contents.
  - p2 ext4 `RGBSLOT` 512 MiB (root).
  - p3 512 MiB, empty (the second A/B root later).
  - The two cards must have different disk signatures for `root=PARTUUID=` to be
    unambiguous. ROCKNIX's card is `2956ee5a`.
- **SD speed:** the dev card (GC1S5 64 GB) **fails SDR104 tuning in slot 1**
  (`mmc1: tuning execution failed: -5`, then `error -5 whilst initialising SD card`), but
  tunes fine in slot 2. The ROCKNIX card (SC256) tunes fine in slot 1. Fix: remove
  `sd-uhs-sdr104` from `/mmc@fe2b0000` in the dev card's copy of the dtb (`fdtput -d`).
  The card then runs at high speed, 50 MHz. Other RK3566 distros keep SDR104, and
  Batocera and ROCKNIX add `card-detect-delay = <800>` on sdmmc1.
- **Console:** `ttyS2` at 1500000 baud (uart2, `0xfe660000`) and `tty0`. The last
  `console=` becomes `/dev/console`.

## The kernel: what made it boot

ROCKNIX's `KERNEL` (Linux 7.2.11) has its own initramfs **built in**
(`CONFIG_INITRAMFS_SOURCE`, LZO). That initramfs always ends in `switch_root` to systemd.
What it takes to boot our root with that kernel unchanged:

1. **`rdinit=/nonexistent`**: when the initramfs program is missing, the kernel prints
   `check access for rdinit=... failed: -2, ignoring` and mounts `root=` itself
   (`init/main.c` 1675-1681).
2. **An extra initrd with a `/root` directory (`INITRD /rootdir.cpio`):** Linux 7.2
   mounts `root=` on the fixed path `/root` inside the initramfs (`init/do_mounts.c:157`).
   - The default initramfs list has `dir /root`, but ROCKNIX's
     `packages/virtual/initramfs/config/initramfs.conf` has only `/dev`, `/dev/console`
     and `usr`, so the mount failed with ENOENT and the kernel panicked:
     `VFS: Unable to mount root fs`.
   - A 512-byte newc cpio holding only `root/` fixes it (`echo root | cpio -o -H newc`).
     The kernel unpacks an external initrd on top of the built-in one.
3. **Command line:**
   `console=ttyS2,1500000 console=tty0 loglevel=8 rdinit=/nonexistent root=PARTUUID=3082b99d-02 rootwait rootfstype=ext4 init=/sbin/init panic=0`
   - MBR PARTUUIDs are `<disk signature>-<partition, 2 hex digits>`.
   - The root mounts read-only, and our init remounts it read-write.
   - `panic=0` keeps a panicked device frozen instead of rebooting.

Our own kernel (phase 1b) makes 1 and 2 unnecessary: build it with
`CONFIG_INITRAMFS_SOURCE=""`, and the default `/dev`, `/dev/console` and `/root` are
built in, as JEBRISH (`github.com/davidthings/spokelinux`) does with
`root=PARTUUID=... rootwait console=tty0 console=ttyS2,1500000 rootfstype=ext4 panic=10 loglevel=8`.

### Seeing what the kernel does without a serial cable

- **Panics never reach the screen.** rockchip's fbdev has a `.dirty` hook, so fbcon
  draws into a shadow buffer that a worker copies out every 50 ms.
  `drm_fb_helper_damage()` skips that copy once `oops_in_progress` is set. So the panic
  text, and about the last 50 ms before it, never appear, and the screen freezes at a
  different line each time. `CONFIG_DRM_PANIC` is off, and rockchip has no scanout hook
  in 7.2 anyway.
- **No crash log survives.** ROCKNIX's kernel has `CONFIG_PSTORE=y` but not
  `CONFIG_PSTORE_RAM`, so nothing is kept for the next boot. Our own kernel can add
  ramoops through a `reserved-memory` node.
- **Log levels in ROCKNIX's kernel:** console 7, quiet 4, default message 7. Plain
  `printk()` lines (such as `VFS: Cannot open root device` and the partition list) need
  `loglevel=8`. `quiet` hides warnings, including `Waiting for root device`.
- **Probing:** running the initramfs's own BusyBox as `rdinit` prints a listing without
  any panic:
  `rdinit=/usr/bin/busybox ... -- sh -c "ls -la /; sleep 60"`.
- **Traces left on the cards, readable from ROCKNIX afterwards:**
  - our boot log, `rgbslot-boot.log` on SLOT, written from the moment SLOT is mounted;
  - the ext4 superblock of the root: mount count at offset 1024+0x34, last mount time at
    1024+0x2C, last-mounted-on at 1024+0x88. Our init's read-write remount bumps the count.

### Ways the device was stranded, and how to avoid them

- **Don't use a one-shot boot entry on the ROCKNIX card.** That was the first attempt: a
  second entry made the default, which our init switches back first thing. Our init
  never ran, because the kernel panicked first, so the device rebooted into the dev entry
  in a loop. Recovery: on Windows, copy `extlinux/extlinux.conf.rocknix` over
  `extlinux/extlinux.conf` on the ROCKNIX card. Only something outside the kernel can
  guarantee a way back, so **use card swaps**.
- **With the dev card in slot 1 and SDR104 still on,** the kernel never sees the card and
  waits forever with only a cursor showing (`rootwait`).

## Our own kernel (phase 1b, built 2026-11-05)

`board/rgb20sx/linux/README.md` has the provenance. What it took:
- **Source:** Linux 7.2.11 from kernel.org (sha256 checked against kernel.org's
  `sha256sums.asc`), with ROCKNIX's RK3566 config and four patch sets, in ROCKNIX's
  order: `mainline`, `mainline-rockchip`, `7.2.6`, then the device set.
- **Placeholders:** ROCKNIX's config has two, `@INITRAMFS_SOURCE@` and
  `@DEVICENAME@`. Ours are `""` and `rgbslot`. Without a built-in initramfs, the
  `/root` initrd and `rdinit=/nonexistent` aren't needed.
- **`CONFIG_LOCALVERSION="-rgbslot"`:** modules go to `/lib/modules/7.2.11-rgbslot`,
  beside ROCKNIX's `7.2.11`, so the same root boots either kernel.
- **Patches left out:**
  - `0004-input-adc-keys-redirect...`, which dropped FN on RK3566;
  - `0005-Bluetooth-btrtl-...`, whose `btusb.c` hunk doesn't apply to 7.2.11 and
    which is for a chip the RGB20SX doesn't have.
- **Fuzz:** ROCKNIX applies its patches with plain `patch` (fuzz 2, and it substitutes
  `@TARGET_CPU@` and `@DEVICE@`, which none of these use). Buildroot's
  `apply-patches.sh` uses `-F0`, and the 1992 MHz OPP patch only applies with fuzz,
  since 7.2.11 moved the `&vop` block it anchors on.
  - So `external.mk` applies the sets in a `LINUX_POST_PATCH_HOOKS` step with `-F2`,
    and fails on any `.rej`.
  - 47 patches apply, several with fuzz. The OPP node lands at the top level at the
    end of `rk3566.dtsi`, as it should.
- **`mali_kbase`** is `package/mali-kbase`, built through the driver's own Makefile
  with ROCKNIX's options (`CONFIG_MALI_PLATFORM_NAME=devicetree`). It installs to
  `/lib/modules/<release>/mali-bifrost/`.
- **FN on our kernel:** with the redirect patch gone, `adc-keys` reports `BTN_MODE`,
  which slot's gamepad table maps to Menu. slot also polls the ADC for FN, so FN would
  fire twice. On a `-rgbslot` kernel, the frontend sets `SLOT_FN_ADC=0`, and slot then
  skips the ADC poll.
- **Installing and falling back:** `dev.sh kernel` (on a running rgbslot) copies
  `Image`, the dtb and the modules, and points `extlinux.conf` at them, keeping the old
  file as `extlinux.conf.rocknix-kernel`. `dev.sh kernel-back` restores it, from rgbslot,
  or from ROCKNIX with the dev card in slot 2.
- **Release check:** the device runs ROCKNIX tag `20261101` (commit `cefc1433`,
  branch `next`, per `/etc/os-release` in its `SYSTEM`). Its kernel patches are
  byte-identical to the `main` commit we copied (`e344c810`), and its config differs only
  by `CONFIG_NET_PTP_CLASSIFY=y`.
- **The dtb on the ROCKNIX card is not what this source builds,** even allowing for
  ROCKNIX building with overlay symbols (`-@`, which also keeps pin groups that
  `/omit-if-no-ref/` would drop: 124 KB against our 61 KB). The real differences:
  - ROCKNIX's has a speaker-amplifier node (`audio-amplifier`, label `spk_amp`), a
    Wi-Fi host-wake node, and joystick `abs-range` 250-800;
  - ours has CPU idle states (`cpu-sleep`, ROCKNIX's patch 1001), more
    `regulator-boot-on` flags, and mainline's joystick range 15-1023.
  - Where ROCKNIX's dtb comes from wasn't traced. The card's `KERNEL` and
    `device_trees` are dated 2026-11-05 07:57, so the user may have updated ROCKNIX
    that morning.
  - Both were tested (attempts 6 and 7). Ours works fully, speaker and headphones
    included, so the missing `spk_amp` node isn't needed. Ours is the default.
- **Build output:** `Image` 27.3 MB (no initramfs), dtb 61 KB, 279 modules in
  `/lib/modules/7.2.11-rgbslot`, including `mali-bifrost/mali_kbase.ko`.
- **Don't** reset the kernel source with `rm` in an agent shell. Use Buildroot's
  `make O=../output linux-dirclean`.

## The root filesystem (Buildroot)

- **Buildroot `2026.08.1`** (commit `7fdeadb3`): glibc 2.44, GCC 15.3, kernel headers
  6.18. External tree `tools/distro/br2-external`, defconfig `rgbslot_rgb20sx_defconfig`.
  Output and downloads live in `~/rgbslot-build`, outside git.
- **Packages:** BusyBox (init, `udhcpc`, `ip`, `modprobe`, `blkid`), ALSA lib and utils,
  libdrm, `wpa_supplicant` (nl80211, AP mode, ctrl iface, CLI, WPA3), `iw`, Dropbear,
  and libstdc++ (`BR2_TOOLCHAIN_BUILDROOT_CXX`, needed by libmali and gambatte). Root
  size: 95 MB, of which 57 MB is libmali.
- **Runtime needs,** checked with `readelf`:
  - slot: glibc and libgcc_s, plus `libEGL`, `libGLESv2`, `libgbm`, `libdrm` and
    `libasound` through dlopen;
  - libmali: libdrm, libstdc++, GLIBC_2.28;
  - mGBA: GLIBC_2.29;
  - gambatte: libstdc++ (GLIBCXX_3.4.21);
  - gpSP: GLIBC_2.17.
- **Init:** BusyBox init. `/etc/inittab` runs `/etc/rgbslot/boot` once and
  `/etc/rgbslot/shutdown` on the way down. The boot script:
  1. mounts proc, sys, devpts and the tmpfs filesystems, and remounts `/` read-write;
  2. puts ROCKNIX back as the default boot (if a one-shot backup exists);
  3. mounts SLOT at `/mnt/sdcard` and logs there;
  4. loads `mali_kbase` and `rtw88_8723ds`;
  5. waits for `wlan0` (about 3.9 s), then starts `wpa_supplicant` and `udhcpc`
     (DHCP hostname `rgbslot`);
  6. starts Dropbear (key login only);
  7. on failure, logs `ip addr`, `wpa_cli status`, the modules and `dmesg`.
- **Harvested from ROCKNIX** (`dev.sh harvest`):
  - `/lib/modules/7.2.11` (288 plain `.ko` files with ROCKNIX's `modules.dep`; BusyBox
    `modprobe` reads them, so kmod isn't needed);
  - `/lib/firmware/rtw88/rtw8723d_fw.bin`, `regulatory.db` and `.p7s`;
  - in `/usr/lib`: `libmali.so.1.10.0` (g29p1 GBM), `libmali-hook.so.1.10.0`, and the
    wrappers `libEGL.so.1`, `libGLESv2.so.2`, `libgbm.so.1` with their symlinks.

### Booting into slot (phase 2)

- **`/etc/inittab`:** `::sysinit:/etc/rgbslot/boot`, then `::respawn:/etc/rgbslot/frontend`.
  BusyBox init waits for every sysinit entry before starting respawn entries, so the boot
  script puts Wi-Fi and SSH in a background subshell and returns at about 2.24 s.
- **`/etc/rgbslot/frontend`:** exports `SLOT_ROOT=/mnt/sdcard`, `SLOT_RUN=/run` and,
  until the device layer exists, `SLOT_PLATFORM=rocknix`. It runs `System/frontend` if
  present, else `System/slot`, through the loader, logs to `Logs/slot.log`, and sleeps
  3 s if slot dies within 5 s so init doesn't spin. The card's BaseOS
  `launch_frontend.sh` is not used yet: its LED and lid helpers are unchecked on this
  hardware.
- **`/etc/asound.conf`:** `default` is a plug on `hw:CARD=rk817ext,DEV=0`, so
  `SLOT_ALSA_DEVICE` isn't needed.
- **In the boot script:** the 400 MHz GPU floor (`min_freq`) right after `mali_kbase`;
  `iw dev wlan0 set power_save on` after `udhcpc`.
- **Kernel command line:** `quiet` instead of `loglevel=8` saved 0.4 s to init (1.59 s).
- **Timeline:** slot starts at 2.28 s and shows its first frame at 5.29 s. The 3 s of
  slot startup (surface 1.56 s, compositor 0.85 s) compare with about 0.6 s under
  ROCKNIX, where libmali was already in the page cache.
- **Power-on to shelf by stopwatch: 11-12 s** (twice). Kernel start to first frame is
  about 5.2 s, so about 6 s pass before the kernel starts:
  - ROCKNIX's U-Boot (2026.01, `bootcmd=bootflow scan -lb`, `boot_targets=mmc1 mmc0 ...`)
    has **`bootdelay=2`**, a 2 s countdown for a key that never comes;
  - it then loads the 30 MB `KERNEL` file, most of which is ROCKNIX's unused initramfs.

  Ways to cut it: our own U-Boot with `bootdelay=0` (or a patched copy, if its FIT image
  isn't hash-checked); our own kernel of around 10 MB without an initramfs; preloading
  libmali at the start of boot, the GLES-only libmali (about 22 MB), and SDR50 on slot 1
  for slot's 3 s startup. About 6-7 s looks reachable.
- **slot's 3 s startup is reading from the card,** measured with the one-line timing that
  `KmsSurface::new` now logs (`slot: kms: start took ...`):
  - cold boot: libraries 1367 ms, first frame at 3.0 s;
  - slot restarted with everything cached: libraries 16 ms, first frame at 0.45 s.
  - Loading libmali touches its 57 MB in scattered order, and the card is slow at random
    reads. It's already stripped (41 MB is `.text`), and LZ4 would bring it to 35 MB.
  - **SDR50 on slot 1** (`sd-uhs-sdr50`, 100 MHz) tunes and works on the dev card, but
    only moved the library step from 1367 to 1293 ms. `dev.sh bootable` now sets it.
  - A background `cat` of libmali at the start of boot didn't help, since it competed
    with slot's own reads. A sequential read before starting slot wasn't measured.
  - Beware: `drop_caches` can't evict pages a running process has mapped, so "cold" reads
    of libmali while slot runs are partly warm.
  - Parked by the user's call: 11-12 s to the shelf is good enough until the OS is
    independent.
- **Development tools on a running rgbslot:**
  - `dev.sh shot NAME` saves the screen as a PNG (slot's SIGUSR1 frame dump).
  - `dev.sh capture NAME VAR=x` restarts slot with extra variables first; the frontend
    reads them from `/run/slot.env`.
  - To restart slot, `kill -TERM $(pidof ld-linux-aarch64.so.1)`, and init's respawn
    starts it again. **Never** kill by a pattern such as `System/slot` that also appears
    in the killing command's own text: twice that killed the SSH session itself.
- **Rebooting over SSH:** `(sleep 1; reboot) &` dies with the SSH session. Use
  `setsid sh -c "sleep 1; reboot" < /dev/null > /dev/null 2>&1 &`.
- **Power-off:** holding power about 2 s in slot runs `poweroff`, which reaches BusyBox
  init, our shutdown script and the PMIC. It switches the device off cleanly.
- **Pushing changes to a running rgbslot:** `tar -C overlay -cf - etc/... | ssh ... 'tar -C / -xf -'`,
  then `chown root:root`, because tar keeps the build user's uid.

### Traps, each of which cost a build or a boot

- **WSL's `PATH` contains Windows folders with spaces,** and Buildroot refuses to run.
  Run make under `env -i HOME=... PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`.
- **The root is not merged-`/usr`:** `/lib` is a real directory. Modules and firmware
  must go in `/lib/...`, because BusyBox `modprobe` and the kernel's firmware loader look
  there. The first harvest put them in `/usr/lib`.
- **BusyBox has no `findfs`.** Find partitions by label with `blkid` output, with
  `/dev/mmcblk1p1` and `/dev/mmcblk2p1` as fallbacks.
- **Dropbear's key folder:** Buildroot links `/etc/dropbear` to `/var/run/dropbear`,
  which doesn't exist after boot, so Dropbear couldn't create its host key and dropped
  every connection right after the key exchange. `post-build.sh` replaces the link with
  a real folder, which also keeps the host key stable.
- **A half-finished patch step** in `host-pkgconf` broke a rebuild: run
  `make host-pkgconf-dirclean`.
- **ROCKNIX's tools:** BusyBox `cmp` has no `-i` (compare with `dd | md5sum`), and
  `parted` reads negative sizes such as `-1040MiB` as options (compute absolute MiB).
- **No SFTP server on the device,** and modern `scp` uses SFTP: it fails with
  `/usr/libexec/sftp-server: not found`. Use `scp -O` (the legacy protocol) or
  `ssh host cat file > file`.
- **Upstream slot owns Wi-Fi:** started with an empty `Config/wifi.conf`, it restarted
  `wpa_supplicant` with that file and dropped the connection, and SSH with it. Seed the
  card's `Config/wifi.conf` or run builds that hide Wi-Fi when SSH matters.
- **The Wi-Fi network** comes from ROCKNIX's iwd file `/storage/.cache/iwd/<SSID>.psk`
  (`Passphrase=` or `PreSharedKey=`). It's converted into `/etc/wpa_supplicant.conf` on
  the device, so the key never leaves it.

## Attempts, in order

| # | Setup | What happened | Cause |
|---|---|---|---|
| 1 | one-shot entry on the ROCKNIX card, root in slot 2, no `quiet`, `panic=10` | log froze around 1.6 s at varying lines, reboot loop | no `/root` in the built-in initramfs: panic, invisible on fbcon |
| 2 | dev card self-booting in slot 1, `quiet`, `panic=0` | two mmc errors, then a cursor | SDR104 tuning failed on slot 1 |
| 3 | same, SDR104 removed | cursor only | the same `/root` panic as attempt 1, hidden by `quiet` |
| 4 | same, plus `INITRD /rootdir.cpio`, `loglevel=8` | **booted**, `ssh ready` at 6.73 s | SSH then dropped connections: the Dropbear key folder |
| 5 | same, Dropbear key folder fixed | **SSH login works.** Init at 1.97 s, SSH ready at 6.7-9.7 s (mostly waiting for `wlan0`), 54 MB RAM used, 5 user processes | phase 1 done |
| 6 | **our own kernel** `7.2.11-rgbslot` with ROCKNIX's dtb (plus SDR50) | **booted**: init 1.54 s, slot 2.07 s, first frame about 5.0 s; picture, speaker, buttons, FN once per press, Wi-Fi all fine; GBA audio now sounds right | — |
| 7 | our own kernel **with our own dtb** | **booted**, all of the above plus headphones, volume, brightness and slot's hotkeys; CPU idle states active (`state0`, `state1`) | **the default from now on** |

What the running system looked like on that boot:
- `/dev/root` ext4 read-write, SLOT at `/mnt/sdcard`;
- `/dev/dri/card0`, `/dev/mali0`, ALSA cards 0 (HDMI) and 1 (`rk817ext`);
- CPU on the kernel's default `performance` governor at 1800 MHz;
- GPU on `simple_ondemand` at 200 MHz.

Both clock settings are for phase 2 (slot's clock cap, the 400 MHz GPU floor). The device
keeps the same DHCP address as under ROCKNIX (`192.168.1.72`). Use a separate known_hosts
for it (`-o UserKnownHostsFile=~/.ssh/known_hosts_rgbslot`), because the host key differs
from ROCKNIX's.
