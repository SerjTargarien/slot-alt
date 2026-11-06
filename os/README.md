# Base²

A small Linux system for the Powkiddy RGB20SX whose only job is to run a slot frontend
(Slot², in this repo). It boots straight into the frontend in about 11-12 s and gives it
what [CONTRACT.md](../CONTRACT.md) lists: the card, a display, input, sound, Wi-Fi tools,
time and a little storage. It knows nothing else about the frontend.

Base² is personal for now: it isn't published or released.

## What it is made of

- **Buildroot** (2026.08.1) with this external tree, `br2-external/` (name `BASESQ`,
  defconfig `basesq_rgb20sx_defconfig`): BusyBox init, Dropbear, `wpa_supplicant`, `iw`,
  ALSA, glibc.
- **Our own kernel**, Linux 7.2.11 with ROCKNIX's RK3566 patches and ours
  (`br2-external/board/rgb20sx/linux/`), version suffix `-basesq`, and `mali_kbase` built
  from Arm's source.
- **The overlay** (`br2-external/board/rgb20sx/rootfs-overlay/`): `/etc/basesq/boot`
  (mounts, drivers, GPU floor, SSH), `/etc/basesq/frontend` (runs the frontend, restarts
  it, development switches, the power log), `/etc/basesq/shutdown`, `/etc/slot-device`
  (the device description), and the `timedatectl` stand-in and DHCP power-save hook.
- **Still borrowed from ROCKNIX**, never in git: U-Boot, `libmali` and its wrappers, and
  the Wi-Fi firmware and regulatory database, fetched by `dev.sh harvest`. Plans to
  replace them are in [PLAN.md](PLAN.md).

## Working on it

Everything goes through [dev.sh](dev.sh) (run it without arguments for its commands).
The build lives in `~/basesq-build`. The device is `root@192.168.1.72`, checked against
`~/.ssh/known_hosts_basesq`.

- Change the OS's own files: edit the overlay, `sh os/dev.sh build`, then
  `sh os/dev.sh overlay` pushes them to a running Base².
- Change the kernel or the device tree: `sh os/dev.sh build`, then
  `sh os/dev.sh kernel`, then reboot. `sh os/dev.sh kernel-back` boots ROCKNIX's kernel
  instead.
- A new card: [BRINGUP.md](BRINGUP.md) has the whole recipe and every trap found so far.
- The frontend has its own loop: `tools/slotsq/dev.sh`.

On the device, a `basesq-dev` file at the card's root keeps Wi-Fi up in games, so SSH
stays reachable (remove it for battery runs), and `/run/slot.env` holds variables for the
frontend's next start.

## More

- [PLAN.md](PLAN.md): the plan, decisions and measurements (battery, Wi-Fi, boot time,
  the square screen).
- [BRINGUP.md](BRINGUP.md): how Base² was brought up, attempt by attempt.
- [../HANDOFF.md](../HANDOFF.md): where the work stands now.
