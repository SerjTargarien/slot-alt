# Handover: slot on ROCKNIX, Powkiddy RGB20SX

Work so far happened in a cloud session that could not reach the device. Every change
went through GitHub Actions and a manual download. This document lets a local session
pick up and iterate directly against the device.

Branch: `claude/beautiful-feynman-ffzfnx` (fork `SerjTargarien/slot-alt` of
`BrandonKowalski/slot`). Read `tools/rocknix/README.md` first: it has the measured device
facts and what step 1 added.

## Where things stand

**slot runs on the RGB20SX under ROCKNIX 20261101**, started from SSH. It boots to the
shelf in about 0.6 s, all buttons work including FN (read from its ADC, see below),
games save, and the in-game menu works. Frame pacing is solved: 0 to 2 missed vblanks in
600 frames in mGBA.

**Current focus: battery life.** The RGB20SX battery is small, and beating stock ROCKNIX
on battery is a goal for the fork. The CPU stays on the `performance` governor, and
`slot/src/clock.rs` caps `scaling_max_freq` to the frame budget.

**Next test pending**, for commit `419dc49` (built by CI run 42010074593, same code as
this branch). Rerun mGBA, gpSP and Gambatte with power and latency tracing, and compare
with the table below. Expect:

- mGBA to leave 1800 MHz, probably settling at 1608
- the menu to settle lower than before
- the GPU to sit mostly at 200 to 400 MHz now that it is back on `simple_ondemand`
- no new missed frames

Battery draw so far (W, while playing; about 1.8 W is fixed baseline):

| Setup | mGBA | gpSP | Gambatte | Menu |
|---|---|---|---|---|
| `performance` CPU + GPU | 2.85-3.0 | | | 2.2-2.3 |
| `schedutil` (stutters badly) | 2.3-2.4 | ~2.4 | ~2.2 | 1.9-2.0 |
| clock cap v1 (`149263d`) | 2.75-2.8 (stuck at 1800) | ~2.3 at 1104 MHz | 2.1-2.2 at 816-1104 | stuck at 1416-1608 |

## Dev loop

Run everything from a Linux shell. On Windows, use WSL2 Ubuntu with the repo cloned
inside the WSL home directory: building on `/mnt/c` is very slow.

One-time setup inside WSL:

```sh
sudo apt-get update
sudo apt-get install -y build-essential pkg-config gcc-aarch64-linux-gnu libudev-dev libasound2-dev openssh-client
curl https://sh.rustup.rs -sSf | sh -s -- -y
. "$HOME/.cargo/env"
rustup target add aarch64-unknown-linux-gnu    # the repo pins Rust 1.96 in rust-toolchain.toml
git clone https://github.com/SerjTargarien/slot-alt && cd slot-alt
git switch claude/beautiful-feynman-ffzfnx
```

SSH to the device is `root@192.168.1.72`, password `rocknix` unless changed. To skip the
password, set up a key once: run `ssh-keygen`, then `ssh-copy-id root@192.168.1.72`. It
lands in `/storage/.ssh/authorized_keys`, which persists.

Then:

```sh
sh tools/rocknix/dev.sh build      # about 1.5 min clean, seconds after small changes
sh tools/rocknix/dev.sh deploy     # copies System/slot and System/rocknix.sh
sh tools/rocknix/dev.sh run        # SLOT_TRACE_LATENCY=1 SLOT_TRACE_POWER=1; extra VAR=x pass through
sh tools/rocknix/dev.sh logs mgba  # -> logs/slot-mgba.log, logs/power-mgba.log (gitignored)
```

`run` needs a real terminal for Ctrl-C (`ssh -t`). The user drives the device during a
run, and says when to quit, or quits with Ctrl-C.

The cross build links with `aarch64-linux-gnu-gcc` (`.cargo/config.toml`). It needs
glibc 2.39, and the device has 2.44. The cores (`mgba_libretro.so`, `gpsp_libretro.so`,
`gambatte_libretro.so`) are already on the device from the CI artifact, and rarely need
rebuilding. If they do, `task dist:rocknix` or the `rocknix` workflow builds them in an
arm64 container.

The device tree is at `/storage/slot`: `System/` holds the binary, cores and the launch
script, and there are also `Games/GBA|GB|GBC`, `Saves`, `States` and `Logs`.

## Device facts that bite

- **FN is not reported by the kernel.** The `adc-keys` driver never registers a press on
  this unit (kernel 7.2.11). slot polls `/sys/bus/iio/devices/iio:device0/in_voltage0_raw`
  instead (about 1023 released, 15 held). A ROCKNIX bug report is drafted in
  `tools/rocknix/rocknix-fn-report.md`; the user posts it.
- **ROCKNIX services fight slot.** `rocknix.sh` stops them and restores them on exit:
  `essway`, `sway`, `inputplumber` and `input` (`input_sense`, the hotkey daemon). It also
  inhibits logind's `HandlePowerKey=suspend`.
- **Display is KMS with libmali GBM** (`slot-gfx/src/kms.rs`). It waits for the GPU
  (`glFinish`) before each flip and draws 4 ms before the vblank. Without both, frames
  were submitted late and missed vblanks. The panel really runs at about 60.0 Hz,
  although its mode timings say 60.69.
- **Audio goes through PipeWire** (`plug:default`). It works, but direct ALSA is a
  possible later latency win.
- **Sticks** report 19 to 1032 in practice, not the declared 250 to 800.
- **Wi-Fi** is RTL8723DS on 2.4 GHz, managed by NetworkManager and iwd. slot's Wi-Fi,
  link play, web server and RomM are hidden on ROCKNIX for now.

## Open work, roughly in order

1. Verify clock cap v2 (above). Tune `slot/src/clock.rs` thresholds against measured power
   and misses if needed.
2. Turn off the Wi-Fi radio during sessions (`rfkill` or `nmcli radio wifi off`), once
   SSH is not needed during play.
3. Autostart slot at boot in place of EmulationStation, with a safe way back.
4. Device-tree overlays via ROCKNIX's `dtb_overlay`:
   - dArkOS-level lower voltages for 408-1104 MHz (825 mV against mainline's 850-900)
   - the FN threshold
   - possibly rumble

   Each is optional and measured with the power log.
5. A layout for the square 720x720 panel. The 720x480 UI is letterboxed now.
6. Left stick as D-pad, rumble on the PWM motor, sleep.
7. Wi-Fi through NetworkManager, link play on 2.4 GHz, web server, RomM.
8. Later decision: stay a frontend on ROCKNIX, or build a minimal base that reuses
   ROCKNIX's kernel, modules, firmware and Mali driver, as BaseOS does on the H700, for
   fast boot and less overhead. Measure ROCKNIX boot with `systemd-analyze` first.

## Conventions and gotchas

- **Do not run `cargo fmt --all`.** Upstream is not rustfmt-clean, and it reformats about
  90 files. Run `rustfmt --edition 2021 <file>` on new files only, and keep edits to
  existing files in the surrounding style.
- Tests that use `HeadlessSurface` only build on macOS, which is upstream's dev machine.
  On Linux, run test targets by name. Those touched by this work:

  ```sh
  cargo test -p slot --test clock --test rocknix --test rocknix_input --test evdev --test quick_menu
  cargo test -p slot-gfx --test kms --test fbdev
  cargo test -p slot-power --test device
  sh card/test-rocknix.sh
  cargo clippy -p slot --no-default-features --features device -- -D warnings
  ```

  The host build needs `libudev-dev` and `libasound2-dev`.
- Some upstream test files fail clippy on Rust 1.96. They are not ours.
- Every ROCKNIX-specific path is gated on `slot::rocknix::running()`. The BaseOS/H700 path
  must keep working unchanged: fbdev display, Anbernic button table, 1.5 ms margin.
- Commit messages are plain and descriptive, without type prefixes, matching the
  repo's history. Push only when the user asks.
- The user is on Windows, comfortable with SSH and git, and prefers measured decisions.
  Ask before changing anything on the device beyond `/storage/slot` and running the
  launch script.
