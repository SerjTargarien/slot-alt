# The OS-to-frontend contract

What an OS (Base², or BaseOS) gives the slot frontend (Slot², or upstream slot), and what
the frontend gives back. Base² and Slot² meet only here: the OS never relies on the
frontend's internals, and the frontend never names the OS or the device. A change to
anything below changes both sides, so it changes this file too.

Upstream slot assumes BaseOS on Allwinner H700 handhelds. Slot² adds one thing to that
contract, the device description (`/etc/slot-device`); without it, it behaves as upstream
but for `SELF_UPDATE` (below).

## The card

- Mounted at `/mnt/sdcard`, passed to the frontend as `SLOT_ROOT`. FAT32 or exFAT.
- The frontend owns everything on it: `System/` (the frontend's binary, cores
  `*_libretro.so`, `web/`, `licenses/`, `bezels/` and its helper scripts), `Games/`,
  `Saves/`, `States/`, `Config/`, `Labels/`, `Bezels/`, `Guides/`, `BIOS/`, `Wallpapers/`,
  `Cache/` and `Logs/`. It creates what's missing on first start.
- The OS may add its own files at the card's root (Base²: `basesq-boot.log`, the
  `basesq-dev` switch) and nothing inside the frontend's folders.

## Starting and stopping

- The OS runs `System/frontend` if the card has one, else `System/slot`, with the card as
  its working directory, through the dynamic loader (`/lib/ld-linux-aarch64.so.1`), and
  sends its output to `Logs/slot.log`. When it exits, for any reason, the OS starts it
  again: that is how the frontend restarts itself after an update or a repair.
- `SIGTERM`: save, then exit. The OS sends it before shutting down or to restart slot.
- `SIGUSR1`: write the next frame to `Logs/frame-<seconds>.png` (development).
- The frontend powers off with `poweroff` and reboots with `reboot`.

## Environment

Set by the OS:

| Variable | Meaning |
|---|---|
| `SLOT_ROOT` | the card (required) |
| `SLOT_RUN` | a tmpfs for runtime state, normally `/run` |
| `SLOT_FN_ADC` | `0`: no FN from an ADC; a sysfs path: read FN from that ADC level. Overrides `FN_ADC` below; Base² sets it only when running ROCKNIX's fallback kernel |
| `SLOT_ALSA_DEVICE` | the ALSA device to play to, if not the default |

Development switches the frontend understands (Base² passes them through
`/run/slot.env`): `SLOT_WIFI_KEEP=1` (keep Wi-Fi up in games), `SLOT_TALL_PANELS=0` (the
old letterboxed band), `SLOT_CLOCK=max` (no CPU clock cap), `SLOT_PACE_MARGIN_US`,
`SLOT_KMS_FINISH`, `SLOT_TRACE`, `SLOT_NO_CORE`, `SLOT_SILENT`.

## The device description: `/etc/slot-device`

`KEY=value` lines, as in BaseOS's release file (`crates/slot/src/hardware.rs`). A missing
key keeps what upstream slot does on BaseOS; a missing file means BaseOS.

| Key | Values | BaseOS default | Base² (RGB20SX) |
|---|---|---|---|
| `NAME` | text, for the log | `BaseOS` | `Base² RGB20SX` |
| `DISPLAY` | `kms` or `fbdev` | `fbdev` | `kms` |
| `BUTTONS` | `gamepad` (`BTN_SOUTH`...) or `anbernic` | `anbernic` | `gamepad` |
| `FN_ADC` | sysfs path of an ADC that reads FN, or empty | empty | empty (the kernel reports FN) |
| `CPU_CAP` | `1`: cap the CPU clock to the frame budget | `0` | `1` |
| `WIFI` | `1`: the frontend runs Wi-Fi | `1` | `1` |
| `BOOT_LOGO` | `1`: the frontend refreshes BaseOS's boot logo | `1` | `0` |
| `SELF_UPDATE` | `1`: offer the upstream frontend's updates (and BaseOS's) | `0` (upstream's is `1`) | `0` |
| `LEFT_STICK`, `RIGHT_STICK`, `STICK_CLICKS`, `LID` | `1` or `0`: override what the frontend detects | detected | detected |

What the device has (Slot², `crates/slot/src/parts.rs`) is detected, and the four keys
above exist only for a device whose kernel says wrong:
- **Sticks and their clicks:** with `gamepad`, from what the input devices report
  (`ABS_X`/`ABS_Y`, `ABS_RX`/`ABS_RY`, `BTN_THUMBL`/`BTN_THUMBR`). With `anbernic`, from
  upstream's list of models with a left stick, by `BASEOS_TARGET` (their kernel reports
  stick axes on models without one); no right stick or clicks.
- **A lid:** an input device with `SW_LID`, or a `hallkey` file under a power supply.
- A device without a part hides what needs it (Lid Power Off), ignores its axes, and doesn't
  show bindings to it; they stay in the card's files for a device that has it.

## What the OS provides

- **Display.** `kms`: DRM at `/dev/dri/card0`, with `libdrm`, `libgbm`, `libEGL` and
  `libGLESv2` (GLES2) that the frontend loads at run time; the panel is the first
  connected internal connector (DSI, eDP, LVDS or DPI), else the first connected one, at
  its preferred mode. `fbdev`: BaseOS's Mali fbdev build.
  The panel may be any shape: the frontend lays itself out for 3:2, 4:3 and taller panels.
- **Input.** evdev devices under `/dev/input`, laid out as `BUTTONS` says, the power key
  (`KEY_POWER`), volume keys, and FN as a key or through `FN_ADC`. The OS reports the
  hardware as it is, the standard way, and the frontend never adjusts the kernel's input
  settings. With `gamepad`:
  - buttons are the standard gamepad codes: `BTN_SOUTH`... for the face buttons,
    `BTN_TL`/`BTN_TR`/`BTN_TL2`/`BTN_TR2`, `BTN_SELECT`, `BTN_START`, `BTN_MODE` for FN,
    and `BTN_THUMBL`/`BTN_THUMBR` where the sticks click;
  - the D-pad is `BTN_DPAD_*` keys or `ABS_HAT0X`/`ABS_HAT0Y`;
  - sticks are `ABS_X`/`ABS_Y` (left) and `ABS_RX`/`ABS_RY` (right), X increasing to the
    right and Y downward, with their true range in the kernel's axis info;
  - polled devices (ADC sticks and keys) are polled often enough for play, about 10 ms.
- **Sound.** ALSA through `libasound`: the default device, or `SLOT_ALSA_DEVICE`.
- **Power and clocks.** `/sys/class/power_supply` (a supply of `type` Battery with
  `capacity` and `status`), `/sys/class/backlight` (`brightness`, `max_brightness`; a
  linear scale gives the intended brightness steps), and `/sys/devices/system/cpu/cpufreq`
  (`scaling_available_frequencies`, `scaling_max_freq`, `scaling_governor`) when
  `CPU_CAP=1`.
- **Time.** An RTC with `date` and `hwclock`, and `/usr/bin/timedatectl` answering
  `show -p NTP --value` and `set-ntp yes|no` (Base² ships a stand-in over BusyBox `ntpd`).
- **Wi-Fi, when `WIFI=1`.** Interface `wlan0`; `wpa_supplicant` with nl80211 and its
  control directory at `/var/run/wpa_supplicant`; `/usr/sbin/wpa_cli`; `/sbin/udhcpc`
  with a default script; `/sbin/ip`; `rfkill`; `iw`. The OS doesn't start Wi-Fi itself:
  the frontend does, from the card's `Config/wifi.conf`, through `System/slotlink.sh`.
  Link play's host role wants a second interface, `wlan1`, which the RGB20SX's chip can't
  give (see `os/PLAN.md`).
- **Storage.** `/run` and `/tmp` as tmpfs; `/data`, small and persistent off the card,
  for the frontend's card health log (`/data/slot-card.log`) and card repair result
  (`/data/slot-repair.result`).
- **Commands.** `/bin/sh`, `sync`, `dmesg`, `poweroff`, `reboot`, `rmmod`.

## Known gaps

- Before powering off, the frontend unloads the Wi-Fi driver by a fixed name, `8821cs`
  (the RG35XX family's). On Base² (`rtw88_8723ds`) that does nothing. It should become a
  description key if unloading ever matters here.
- BaseOS-only paths stay in the frontend for upstream's devices and do nothing on Base²:
  `/etc/baseos-release`, `/run/baseos.conf`, `baseos-update`, the H700 HDMI switch, and
  the card scripts `launch_frontend.sh`, `led.sh` and `lid.sh`, which BaseOS runs.
