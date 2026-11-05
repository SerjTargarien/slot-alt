# ROCKNIX report: RGB20SX FN button

ROCKNIX closes GitHub issues that weren't discussed on their Discord first. Post the
short message first, then open the issue at
<https://github.com/ROCKNIX/distribution/issues/new/choose> (Bug Report) and paste
everything under "GitHub issue". Fill in the two `<...>` placeholders.

---

## Discord message

> **RGB20SX: FN button dead since the RK3566 InputPlumber switch (kernel 7.2.11).**
> I think there are two causes:
> 1. `01-rgb20sx.yaml` uses the `anbernic_gen1` capability map, which now maps `BTN_BACK`
>    only. The RGB20SX's `adc-keys` emits `BTN_MODE`.
> 2. The `adc-keys` device itself never reports a press on this board, even with
>    InputPlumber and sway stopped. Yet saradc ch0 reads 15 when FN is held (1023 released).
>
> It's the same `press-threshold-microvolt = <1750>` that `0029-anbernic-fix-adc-key.patch`
> raised to 41000 on the RGxx3. Happy to test a fix. Should I open an issue?

---

## GitHub issue

**Title:** [BUG] Powkiddy RGB20SX: FN button does nothing (adc-keys never reports a press, and BTN_MODE is missing from the InputPlumber map)

### Have you first reported the issue on the rocknix discord and checked rocknix.org for a solution?

Yes, discussed on Discord: `<link to your Discord message>`

### Describe the bug

The FN button on the Powkiddy RGB20SX does nothing in EmulationStation or RetroArch. It
used to work in RetroArch, mapped to the menu, on builds before the RK3566 move to
InputPlumber.

There seem to be two separate problems:

1. **InputPlumber drops the key.** `01-rgb20sx.yaml` uses the `anbernic_gen1` capability
   map. Since `aa410c506` ("rk3566: anbernic change F button to emit KeyF1"), that map
   handles `BTN_BACK`. The RGB20SX's FN comes from the mainline `adc-keys` node in
   `rk3566-powkiddy-rgb20sx.dts`, which emits `BTN_MODE`, and no map handles `BTN_MODE`.
2. **The kernel never reports FN.** The `adc-keys` device produces no `BTN_MODE` press
   even when read directly with InputPlumber, EmulationStation and sway all stopped. The
   kernel's key state (`EVIOCGKEY`) stays released while FN is held. The ADC sees the
   button fine.

### How to reproduce

1. On an RGB20SX, press FN in EmulationStation or RetroArch. Nothing happens.
2. Over SSH, run `systemctl stop inputplumber`. Then run `evtest /dev/input/event1`
   (`adc-keys`) and press FN. No events arrive.
3. Hold FN and run `cat /sys/bus/iio/devices/iio:device0/in_voltage0_raw`. It reads
   about 15, against 1023 with FN released, so the button reaches the ADC.

### Information

- ROCKNIX Version: `<fill in from System Settings>`
- Hardware Platform: Powkiddy RGB20SX
- Kernel: 7.2.11

### Log file

Measurements on my unit:

- `adc-keys` is `event1`, with `KEY=1000000000000000 0 0 0 0` (`BTN_MODE` only),
  `inhibited` 0 and `poll` 60.
- saradc channel 0 reads 1023 released and 15 held. `in_voltage_scale` is 1.757812500,
  so that's about 1798 mV and 26 mV. The node sets `keyup-threshold-microvolt = <1800000>`
  and `press-threshold-microvolt = <1750>`.
- No process holds an `EVIOCGRAB` on the device: a test grab succeeds.
- Unbinding and rebinding the `adc_keys` driver changes nothing.
- The driver does poll. The saradc interrupt rate drops from about 77/s to about 62/s
  while `adc-keys` is unbound, which is one conversion per 60 ms poll.
- gpio3 PB7, which was FN on the RK2023/RGB30 in the old `rocknix-singleadc-joypad`
  tree, does not change when FN is held on this board. FN on the RGB20SX is only on saradc
  channel 0.

### Context

`0029-anbernic-fix-adc-key.patch` changed the same kind of node on the RGxx3:

- `BTN_MODE` became `BTN_BACK`
- `press-threshold-microvolt` went from `<1750>` to `<41000>`

The RGB20SX dts still has the old values. Making the same change in
`rk3566-powkiddy-rgb20sx.dts` might fix both problems at once, with FN becoming F1
through `anbernic_gen1`. I haven't been able to test that, so I can't say whether the
threshold explains the kernel side. A separate RGB20SX capability map that includes
`BTN_MODE` would fix the first problem, but not the second.

I can test a build or patch on my unit. I investigated this with the help of an AI
assistant (Claude). The measurements come from scripts run on the device, and I'm happy
to share them.
