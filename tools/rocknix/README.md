# slot on ROCKNIX: Powkiddy RGB20SX

Porting notes for running slot on the RGB20SX (Rockchip RK3566) under ROCKNIX,
instead of on an H700 under BaseOS.

## Step 0: probe the device

`probe.sh` surveys the device and changes nothing. Copy it over and run it as root:

```sh
scp tools/rocknix/probe.sh root@<device-ip>:/storage/
ssh root@<device-ip> sh /storage/probe.sh
ssh -t root@<device-ip> sh /storage/probe.sh buttons
```

The `buttons` run asks you to press each button and move the sticks. It stops
InputPlumber for about two minutes, so EmulationStation won't react to buttons until it
finishes. If the script is killed partway through, reboot or run
`systemctl start inputplumber`. Both runs save their output to `/storage/slot-probe.txt`.

## What the probe confirmed

Measured on an RGB20SX running ROCKNIX 20261101 (kernel 7.2.11, glibc 2.44):

| Area | BaseOS on H700 | ROCKNIX on RGB20SX |
|---|---|---|
| Display | Mali closed driver with an fbdev EGL window | DSI-1, 720x720 at 60.69 Hz (37 MHz clock, 814x749 total). `libmali` is active and bind-mounted over Mesa's `libEGL`, `libGLESv2` and `libgbm`. HDMI is a second connector on the same card. `/dev/fb0` exists too, from DRM's fbdev emulation. |
| Frontend | `launch_frontend.sh`, started by BaseOS | `essway.service` runs EmulationStation under `sway.service` |
| Buttons | Anbernic codes, D-pad on `ABS_HAT0X/Y` | `gpio-keys-control`: A `0x131`, B `0x130`, X `0x133`, Y `0x134`, L1 `0x136`, R1 `0x137`, L2 `0x138`, R2 `0x139`, Select `0x13a`, Start `0x13b`, L3 `0x13d`, R3 `0x13e`, D-pad `0x220`-`0x223`. `gpio-keys-vol`: 115 and 114. Power: `rk805 pwrkey`, 116. |
| FN | an evdev key | Never reported by the kernel. See `rocknix-fn-report.md`. Its ADC channel, `iio:device0/in_voltage0_raw`, reads 1023 released and 15 held. |
| Input ownership | slot reads `/dev/input` | InputPlumber hides the raw nodes in `/dev/inputplumber/sources`. Stopping it puts them back. |
| Hotkeys | none | `input.service` (`input_sense`) handles volume, power and FN/hotkey combos system-wide |
| Power key | slot | logind: `HandlePowerKey=suspend` |
| Sticks | centred on 0 | `adc-joystick`: declared 250 to 800, but really moves 19 to 1032, resting near 520 |
| Audio | ALSA | PipeWire owns the `rk817_ext` card. ALSA `default` goes to PipeWire. |
| Backlight, battery | generic | `backlight` (0 to 255), `battery` and `charger` (USB, 3.8 V to 5.5 V input). LEDs: `green:status`, `red:charging`. |
| Storage | FAT/exFAT card | `/storage` (with `/storage/roms`) is ext4 on `mmcblk1p2` and allows executables |
| Wi-Fi | RTL8821CS, slot drives `wpa_supplicant` | RTL8723DS, 2.4 GHz, can run an access point. Managed by NetworkManager and iwd. No `wpa_cli` or `hostapd`. |

## Step 1: slot boots to the shelf and plays a game

**Done when:** started from SSH, slot shows the shelf on the RGB20SX panel, every
button including FN works, a GBA, GB and GBC game each run at full speed with sound,
saves land on the card, and quitting returns the device to EmulationStation.

What step 1 adds:

- **Platform detection** (`slot/src/rocknix.rs`): `OS_NAME="ROCKNIX"` in
  `/etc/os-release`, or `SLOT_PLATFORM=rocknix`. Since the device layer
  (`slot/src/hardware.rs`), this selects the ROCKNIX preset; an OS with
  `/etc/slot-device` (slot²) describes its device there instead.
- **KMS display** (`slot-gfx/src/kms.rs`): picks the built-in panel over HDMI and its
  preferred mode, renders through GBM and EGL with GLES2, and blocks on each page flip
  so the existing frame pacing still works. It loads its libraries at runtime, so the
  device build has no new dependencies.
- **Frame pacing on KMS:** slot waits for the GPU after drawing (`glFinish`), so GPU time
  counts as frame work, and draws 4 ms before each vblank instead of 1.5 ms. Without
  either, `libmali` spent 1 to 2 ms in `eglSwapBuffers` that the pacer never saw, and 6
  to 25 percent of frames reached the panel one vblank late, all of them submitted after
  their deadline. With both, it was 0 to 2 frames in 600. `SLOT_KMS_FINISH=0` and
  `SLOT_PACE_MARGIN_US` override them. With `SLOT_TRACE_LATENCY=1`, the log reports
  every 600 flips how many skipped a vblank and why, and every 600 frames the average and
  longest frame, its CPU part and GPU wait, how many misses were CPU-bound or GPU-bound,
  and the current CPU cap. The pacer measures the panel at about
  60.0 Hz, although the mode's timings say 60.69 Hz.
- **CPU clock by frame budget** (`slot/src/clock.rs`): the CPU stays on the `performance`
  governor, and slot caps `scaling_max_freq` at the lowest step that keeps frames on
  time. Checks run every half second. Only CPU-bound frames can raise the cap: those whose
  CPU part (everything but the GPU wait, including waiting for the emulator thread) is at
  least as long as their GPU wait. A slow GPU frame would not get faster on a faster CPU.
  - **Up a step** after two missed or late frames, after one while work is over 70% of
    the budget, or when work passes 85%. Two steps after several misses. A lone miss with
    the CPU nearly idle is treated as a one-off hitch and ignored.
  - **Down a step** after three calm seconds, if the predicted work at the lower clock
    stays under 75% of the budget. Only the CPU part of the work is scaled for that
    prediction; time spent waiting for the GPU is not.
  - **To the top step** while a game is inserted and its core, save and state load, so a
    game never starts at the menu's clock. The usual calm seconds then bring it down.
  - **Unchanged while a game is put away.** Its save takes one long frame (80 to 130 ms)
    that a faster clock would not shorten, so those frames are left out.

  `SLOT_CLOCK=max` turns it off. The GPU keeps its own `simple_ondemand` governor unless
  `SLOT_GPU_GOVERNOR` is set, with a 400 MHz floor (see the launch script). The launch
  script restores the cap if slot dies without cleaning up.

  Measured on the RGB20SX (battery draw while playing):

  | Setup | mGBA | gpSP | Gambatte |
  |---|---|---|---|
  | `performance` everywhere | 2.85-3.0 W | | |
  | `schedutil` | 2.3-2.4 W, misses up to 2/3 of vblanks | ~2.4 W, misses | ~2.2 W, misses |
  | First capped version | stayed at 1800 MHz | 1104 MHz, ~2.3 W, no misses | 816-1104 MHz, 2.1-2.2 W, no misses |
  | GPU on `simple_ondemand` (200 MHz) | 1800 MHz, 2.6-2.8 W | 1800 MHz, ~2.47 W | 1800 MHz, 2.33 W |
  | CPU-bound rule, 400 MHz GPU floor | 1800 MHz, ~2.7 W | 1104 MHz, 2.13 W, no misses in play | 1104 MHz, 2.03 W, no misses in play |

  The last two rows are in-game medians. With the GPU at 200 MHz its wait was two thirds
  of a Gambatte frame, which pushed the CPU to 1800 MHz under the earlier rules. With the
  CPU-bound rule and no floor, Gambatte held 1416 MHz at 2.06 W; with the GPU on
  `performance`, 816 MHz at 2.06 W. mGBA's CPU part alone is about 6 ms on average and
  up to 10 ms at 1800 MHz, so 1800 MHz is its floor under these rules.

  The fixed baseline is about 1.8 W.
- **Buttons** (`slot/src/input/evdev.rs`): a standard-gamepad table on ROCKNIX. The
  Anbernic table stays for BaseOS, since the two disagree about codes like `0x138`.
- **FN** (`slot/src/input/adc.rs`): read from its ADC channel about 60 times a second as
  Menu. The gap between the press and release levels keeps noise from making it flicker.
- **Features with no ROCKNIX support yet:** Wi-Fi is reported absent, which hides Tools
  (web server, RomM, art fetching, update checks) and Wi-Fi, and makes link play refuse.
  The boot logo is skipped. LEDs that belong to input devices are ignored.
- **Launch script** (`card/rocknix/rocknix.sh`):
  - stops `essway`, `sway`, `inputplumber` and `input`
  - blocks logind's power-key suspend while slot runs
  - puts the CPU on the `performance` governor so slot can cap its clock
    (`SLOT_GOVERNOR` names another, `keep` opts out), and leaves the GPU on its own
    governor unless `SLOT_GPU_GOVERNOR` names one
  - sets a 400 MHz floor under the GPU (`SLOT_GPU_MIN_MHZ`, `keep` opts out). ROCKNIX
    means to set the same floor on RK3566, but its quirk `060-game_settings` runs before
    the GPU driver loads and fails. ROCKNIX's `powerstate` service resets the GPU
    governor on every charger change but leaves `min_freq` alone, so the floor holds
  - loads ROCKNIX's environment, so audio reaches PipeWire
  - starts everything again however slot ends
  - turns Ctrl-C or a dropped SSH session into SIGTERM, so slot saves first

  `card/test-rocknix.sh` tests all of this off-device.
- **Packaging:** `task dist:rocknix`, and the `rocknix` GitHub Actions workflow, which
  builds the tree on every push to a `claude/` branch and uploads it as an artifact.

### Install and run

1. Download the `slot-rocknix-<commit>` artifact from the newest **rocknix** workflow
   run on GitHub (the repository's Actions tab). Extract it into a folder named `slot`.
2. Copy it to the device: `scp -r slot root@<device-ip>:/storage/`
3. Copy a few unzipped ROMs into `/storage/slot/Games/GBA`, `Games/GB` and `Games/GBC`.
4. Run it: `ssh -t root@<device-ip> bash /storage/slot/System/rocknix.sh`
5. Press Ctrl-C in that terminal to save and go back to ROCKNIX. The log is in
   `/storage/slot/Logs/slot.log`. Prefix the command with `SLOT_TRACE_LATENCY=1` to log
   frame pacing every 600 frames.

## Later steps

- Autostart slot at boot in place of EmulationStation.
- A layout that uses the square 720x720 panel instead of letterboxing 720x480 into it.
- Left stick as D-pad, normalised from the real 19 to 1032 range rather than the
  declared one.
- Rumble on the PWM motor.
- Brightness and volume shared with ROCKNIX's settings.
- Sleep.
- Wi-Fi through NetworkManager.
- Link play on a 2.4 GHz channel (`AGS_LINK_FREQ`).
- The web server and RomM.
- An update channel for ROCKNIX builds.
