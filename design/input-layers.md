# Input across Base² and Slot²

Status: done on 2026-11-06, bar the backlog (items 6 and 7).

**The DS was removed on 2026-11-07** ([two-devices.md](two-devices.md), phase 5): its mentions
below are history.

## Where we are

- Slot² reads both sticks (uncommitted, on the device): directions from the kernel's
  axis ranges, the left stick steering outside games, LS and RS directions in games.
- With the RK2023 device tree Base² inherits, both sticks' Y axes run backwards (up reads
  high). A device tree patch that flipped them was tried and reverted: games then saw Up
  held. With the original tree that doesn't happen. The cause wasn't found; it is traced
  only if it comes back (see "If Up sticks again").
- Probe (2026-11-06, raw `adc-joystick` events): every axis rests at 511-550 and reaches
  15-23 and 1004-1023; X runs left to right; Y runs down to up on both sticks; events
  come about every 16 ms once Slot² has set the poll to 10 ms.

**Found and fixed (2026-11-06):** the stuck Up was a kernel bug. `adc-joystick` flips an
axis using the range of the ABS code equal to the axis's *position* (`i`), not its code.
The RGB20SX's Y axis is third, so a flipped Y read ABS_Z's empty range and came out as
minus the reading: about -520 at rest, which reads as fully up. Base² patches the
driver (`0004-Input-adc-joystick-invert-an-axis-by-its-code-not-its-index.patch`); worth
sending upstream. With it and the device tree patch (`0003`), every axis runs 15-1023,
rests near 520, and up and left read low.

## The rule

**Base² reports the hardware as it is, the standard way. Slot² decides what a press
means.** Neither works around the other: Base² doesn't shape input for a frontend, and
Slot² doesn't patch over a device's quirks or tune the kernel.

## Inconsistencies to fix

### Base²

1. **The sticks' Y axes are upside down.** Fix in the device tree: both Y ranges
   flipped, for the RGB20SX only. The kernel's `adc-joystick` driver inverts an axis
   whose range runs high to low and reports it as 15-1023, so readers see a standard
   axis.
2. **The polled devices poll every 60 ms** (`adc-joystick`, and `adc-keys`, which is
   FN). Slot² rewrites that to 10 ms at start, through sysfs. Set it in the device tree
   instead (`poll-interval = <10>` on both).
3. **The contract doesn't say what input looks like.** Add: with `BUTTONS=gamepad`, keys
   are the standard gamepad codes (including L3 and R3 where a device has them), the
   D-pad is either keys or `ABS_HAT0X/Y`, sticks are `ABS_X/Y` and `ABS_RX/RY` with
   Y increasing downward and their true range in the kernel's axis info, and the OS sets
   a polled device's rate.

### Slot²

4. **Slot² tunes the kernel** (the poll rewrite). Keep it only for BaseOS, which has no
   device description and whose kernel we can't change; on a system with
   `/etc/slot-device` it's the OS's job. Done.
5. **L3 and R3 aren't read.** The RGB20SX's `gpio-keys-control` reports `BTN_THUMBL` and
   `BTN_THUMBR`, and both sticks click (confirmed by the user). Make them physical inputs,
   bindable and labelled L3 and R3. Done.
6. **Backlog (2026-11-06), done only if it causes trouble: directions are merged per
   device, not across devices.** If it is done, it is one general change: every input
   device read into a single input, one place that knows the state of every button and
   axis, rather than a fix for the D-pad and stick alone. The D-pad comes from
   `gpio-keys-control` as keys and the left stick from `adc-joystick`; each reader thread
   tracks only its own. Steering the menus with the stick while the D-pad is held, or the
   reverse, can let one release the other's Up. The merge (the D-pad wins over a steering
   stick) belongs in one place that sees every device.
7. **Backlog, with 6: the stick changes role only on its next movement.** Entering or leaving a game while
   it is tilted keeps its old button (Up or LS Up) until it moves. Part of 6: the merge
   point releases the old role's buttons as soon as the context changes.
8. **The controls design note says the RGB20SX has no stick clicks.** Correct it with 5.
   Done with 5.

## Order

1. **Base²: items 1-3, done.** Device tree patch (Y axes, poll interval), the driver fix,
   the contract text.
   Rebuild, keep the running device tree as `basesq.dtb.prev`, install, reboot.
   Check: the same raw probe shows up as low on both sticks and a 10 ms poll; Slot²'s log
   no longer says "poll 60 ms -> 10 ms"; the sticks steer the shelf the right way; a game
   moves with both sticks, and nothing stays held after leaving the in-game menu, binding
   a row, or switching game and shelf with a stick tilted.
2. **Slot²: commit what is on the device** (sticks, DS controls, SELECT chords): done,
   the DS controls checked by the user (the DS's screens still want polish, later).
3. **Slot²: items 4, 5 and 8, done**, built and tested together, then committed. Items 6
   and 7 wait in the backlog.

Each step is checked on the device before the next. Base² and Slot² are committed
separately.

## If Up sticks again

Restart Slot² with `SLOT_TRACE=1` and `SLOT_TRACE_INPUT=1`, play until it sticks, and read
which event wasn't released (the input trace has every raw event, the session log every
change in what the core is sent). Fix that, before anything else.
