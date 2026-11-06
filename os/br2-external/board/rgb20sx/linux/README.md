# Base²'s kernel for the RGB20SX

Linux 7.2.11 from kernel.org (sha256
`fd0fc479f983a8c05313f13e9c36330fe6651cf725396748c40d5cd93fa1e6a4`), built by Buildroot
the way ROCKNIX 20261101 builds its RK3566 kernel, so it behaves the same, with these
differences:

- **No initramfs.** `CONFIG_INITRAMFS_SOURCE=""` (ROCKNIX fills in its own), so the kernel
  mounts `root=` itself. The extra `/root` initrd and `rdinit=/nonexistent` that booting
  ROCKNIX's kernel needed go away.
- **`CONFIG_LOCALVERSION="-basesq"`.** Modules live in `/lib/modules/7.2.11-basesq`, next
  to the ROCKNIX ones in `/lib/modules/7.2.11`, so the same root boots either kernel.
- **`CONFIG_DEFAULT_HOSTNAME="basesq"`** (ROCKNIX fills in the device name).
- **Without ROCKNIX's `0004-input-adc-keys-redirect-keycode-316-to-rocknix-joypa.patch`.**
  It sends FN's `BTN_MODE` to a rocknix-joypad driver that RK3566 doesn't build, so FN
  was dropped. Without it, `adc-keys` should report FN itself. Note that slot also reads
  FN from the ADC on this device, so check FN doesn't fire twice.
- **Without `0005-Bluetooth-btrtl-Add-the-support-for-RTL8733BU.patch`.** Its `btusb.c`
  hunk doesn't apply to 7.2.11 (probably already upstream), and it's for a USB Bluetooth
  chip the RGB20SX doesn't have.
- **One patch of our own,** `5-basesq`: slot 1 (`sdmmc0`) runs at SDR50 instead of
  SDR104, which a card failed to tune.

Sources, all GPL-2.0, from ROCKNIX's `distribution` repository at commit
`e344c8109029ccd1bb9019680c1cd3d5357d56da` (`main`, 2026-11-05). The patches are
byte-identical to release tag `20261101` (`cefc1433`), which the device runs, and the
config differs from it only by `CONFIG_NET_PTP_CLASSIFY=y`. They are applied in this
order, as ROCKNIX does (with fuzz, through `external.mk`):

| Directory | From |
|---|---|
| `linux.config` | `projects/ROCKNIX/devices/RK3566/linux/linux.aarch64.conf` |
| `patches/1-mainline` | `projects/ROCKNIX/packages/linux/patches/mainline` (without 0004 and 0005) |
| `patches/2-mainline-rockchip` | `projects/ROCKNIX/packages/linux/patches/mainline-rockchip` |
| `patches/3-7.2.6` | `projects/ROCKNIX/packages/linux/patches/7.2.6` |
| `patches/4-rk3566` | `projects/ROCKNIX/devices/RK3566/patches/linux` |
| `patches/5-basesq` | ours |

The Mali kbase module is `package/mali-kbase`: Arm's driver from ROCKNIX's mirror
`github.com/rocknix/mali_kbase` at `39da994bb6fc8819e5e8c1873907dd21d17e53c1`, built with
the same options as ROCKNIX's `packages/linux-drivers/mali-bifrost/package.mk`.

Thanks to the ROCKNIX and JELOS contributors, whose patches these are.
