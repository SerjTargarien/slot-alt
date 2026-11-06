#!/bin/sh
# Surveys a ROCKNIX handheld for the slot port: display, GPU driver, input, audio,
# power, storage and Wi-Fi. Run it over SSH as root.
#
#   sh probe.sh           read-only survey, changes nothing
#   sh probe.sh buttons   also asks you to press each button and move the sticks. This
#                         stops InputPlumber for about two minutes so the raw button
#                         devices can be read, then starts it again. EmulationStation
#                         ignores the buttons while it is stopped. Needs python3.
#
# Everything is printed and also saved to /storage/slot-probe.txt.

OUT="${PROBE_OUT:-/storage/slot-probe.txt}"
MODE="${1:-survey}"

section() {
	printf '\n===== %s\n' "$1"
}

run() {
	printf '$ %s\n' "$*"
	sh -c "$*" 2>&1
}

show() {
	for f in "$@"; do
		[ -e "$f" ] || continue
		printf '%s: %s\n' "$f" "$(tr '\n' ' ' < "$f" 2>/dev/null)"
	done
}

survey() {
	section "system"
	run "cat /etc/os-release"
	run "uname -a"
	show /sys/firmware/devicetree/base/model /sys/firmware/devicetree/base/compatible
	run "ls -l /lib/ld-linux-aarch64.so.1 /usr/lib/ld-linux-aarch64.so.1"
	run "ls -l /usr/lib/libc.so.6"
	run "/usr/lib/libc.so.6 | head -n 1"
	run "cat /proc/cpuinfo | grep -E 'CPU part|Features' | sort | uniq -c"
	run "free -m"
	run "command -v python3"

	section "services"
	for s in sway essway emustation inputplumber pipewire pipewire-pulse wireplumber connman iwd NetworkManager sshd systemd-logind; do
		printf '%s: %s\n' "$s" "$(systemctl is-active "$s" 2>/dev/null)"
	done
	run "systemctl list-units --type=service --state=running --no-legend --plain | cut -d' ' -f1"
	run "grep -sE '^[^#]*Handle(PowerKey|LidSwitch)' /etc/systemd/logind.conf /etc/systemd/logind.conf.d/*.conf /usr/lib/systemd/logind.conf.d/*.conf /storage/.config/logind.conf.d/*.conf"

	section "gpu"
	run "gpudriver"
	run "lsmod | grep -E 'mali|panfrost|panthor'"
	run "ls -l /usr/lib/libEGL.so* /usr/lib/libGLESv2.so* /usr/lib/libgbm.so* /usr/lib/libdrm.so*"
	run "mount | grep -E 'libEGL|libGLES|libgbm|libmali'"
	run "ls /usr/lib/mali"
	run "ldconfig -p | grep -E 'libEGL|libGLESv2|libgbm|libdrm|libasound'"

	section "display"
	run "ls -l /dev/dri /dev/fb0"
	for c in /sys/class/drm/card*-*; do
		[ -d "$c" ] || continue
		show "$c/status" "$c/enabled" "$c/modes" "$c/dpms"
	done
	show /sys/class/graphics/fb0/modes /sys/class/graphics/fb0/virtual_size
	show /sys/kernel/debug/dri/*/name
	run "cat /sys/kernel/debug/dri/*/state | grep -E 'crtc\[|plane\[|connector\[|mode:|active=|enable=|fb=|format=|crtc-pos|rotation' | head -n 40"
	run "grep -E 'Output|current|transform|Mode' /var/log/sway.log | head -n 20"
	run "command -v drm_tool modetest swaymsg"

	section "input"
	run "cat /proc/bus/input/devices"
	run "ls -l /dev/input /dev/inputplumber /dev/inputplumber/sources"
	for d in /sys/class/input/event*; do
		[ -d "$d" ] || continue
		printf '%s %s | key: %s | abs: %s\n' "$(basename "$d")" \
			"$(cat "$d/device/name" 2>/dev/null)" \
			"$(cat "$d/device/capabilities/key" 2>/dev/null)" \
			"$(cat "$d/device/capabilities/abs" 2>/dev/null)"
	done
	run "command -v evtest evtest-ng"
	for d in /sys/bus/iio/devices/iio:device*; do
		[ -d "$d" ] || continue
		printf '%s (%s):' "$(basename "$d")" "$(cat "$d/name" 2>/dev/null)"
		for r in "$d"/in_voltage*_raw; do
			[ -r "$r" ] && printf ' %s=%s' "$(basename "$r")" "$(cat "$r" 2>/dev/null)"
		done
		[ -r "$d/in_voltage_scale" ] && printf ' scale=%s' "$(cat "$d/in_voltage_scale")"
		echo
	done

	section "audio"
	run "cat /proc/asound/cards"
	run "aplay -l"
	run "aplay -L | head -n 40"
	run "ls /etc/alsa/conf.d /usr/share/alsa/alsa.conf.d"
	run "amixer scontrols"
	run "grep -H . /proc/asound/card*/pcm*p/sub*/status"
	run "command -v wpctl pactl pw-cli"
	run "wpctl status | head -n 40"

	section "power"
	for b in /sys/class/backlight/*; do
		show "$b/brightness" "$b/max_brightness" "$b/type"
	done
	for p in /sys/class/power_supply/*; do
		[ -r "$p/uevent" ] && run "cat $p/uevent"
	done
	run "ls /sys/class/leds /sys/class/rtc /sys/class/pwm"
	run "find /sys/devices/platform -name rumble_enable"
	run "command -v poweroff reboot timedatectl hwclock rmmod"

	section "storage"
	run "df -h"
	run "mount | grep -E 'mmcblk|/storage|/roms'"

	section "wifi"
	run "ls /sys/class/net"
	run "iw list 2>/dev/null | sed -n '/Supported interface modes/,/Band 2\|software interface/p' | head -n 20"
	run "iw dev | grep -E 'Interface|type|channel'"
	run "command -v iw iwctl connmanctl wpa_cli wpa_supplicant hostapd udhcpc"
}

# Keeps every input device open for the whole test, so no press is lost between reads.
# FN on the RGB20SX never reaches its input device, so the FN pin's ADC channel is
# watched too: a press shows up as the reading dropping well below its resting 1023.
read_buttons() {
	python3 - <<'EOF'
import fcntl, glob, os, select, struct, time

BUTTONS = ["A", "B", "X", "Y", "L1", "R1", "L2", "R2", "SELECT", "START", "FN",
           "UP", "DOWN", "LEFT", "RIGHT", "L3", "R3", "VOLUME-UP", "VOLUME-DOWN"]
ADC = "/sys/bus/iio/devices/iio:device0/in_voltage0_raw"
ADC_PRESSED = 300
ADC_RELEASED = 800
ABS_NAMES = {0: "ABS_X", 1: "ABS_Y", 2: "ABS_Z", 3: "ABS_RX", 4: "ABS_RY", 5: "ABS_RZ",
             0x10: "ABS_HAT0X", 0x11: "ABS_HAT0Y"}
EVIOCGABS = 0x80184540


def tty(msg, end="\n"):
    with open("/dev/tty", "w") as t:
        t.write(msg + end)


def bits(text):
    out = set()
    for i, word in enumerate(reversed(text.split())):
        value = int(word, 16)
        out.update(i * 64 + b for b in range(64) if value >> b & 1)
    return out


def attr(node, name):
    try:
        with open("/sys/class/input/%s/device/%s" % (node, name)) as f:
            return f.read().strip()
    except OSError:
        return ""


devs = {}
for path in sorted(glob.glob("/dev/input/event*")):
    try:
        fd = os.open(path, os.O_RDONLY | os.O_NONBLOCK)
    except OSError as e:
        print("%s: %s" % (path, e.strerror))
        continue
    node = os.path.basename(path)
    devs[fd] = (node, attr(node, "name"), bits(attr(node, "capabilities/abs") or "0"))
print("reading: " + ", ".join("%s (%s)" % (n, name) for n, name, _ in devs.values()))


def events(timeout):
    ready, _, _ = select.select(list(devs), [], [], timeout)
    out = []
    for fd in ready:
        try:
            data = os.read(fd, 24 * 64)
        except BlockingIOError:
            continue
        for i in range(0, len(data) - 23, 24):
            kind, code, value = struct.unpack_from("<HHi", data, i + 16)
            out.append((fd, kind, code, value))
    return out


def adc():
    try:
        with open(ADC) as f:
            return int(f.read())
    except (OSError, ValueError):
        return None


def drain():
    while events(0):
        pass


def wait_press(limit):
    end = time.time() + limit
    while time.time() < end:
        for fd, kind, code, value in events(0.03):
            if kind == 1 and value == 1:
                return ("key", fd, code)
        level = adc()
        if level is not None and level < ADC_PRESSED:
            return ("adc", None, level)
    return None


def wait_release(how, fd, code):
    end = time.time() + 3
    while time.time() < end:
        if how == "key":
            for f, kind, c, value in events(0.03):
                if f == fd and kind == 1 and c == code and value == 0:
                    return
        else:
            level = adc()
            if level is None or level > ADC_RELEASED:
                return
            time.sleep(0.03)


print("\nresting axes:")
for fd, (node, name, absbits) in devs.items():
    for code in sorted(absbits):
        buf = bytearray(24)
        try:
            fcntl.ioctl(fd, EVIOCGABS + code, buf)
        except OSError:
            continue
        value, lo, hi, fuzz, flat, _ = struct.unpack("<6i", buf)
        print("%s %s %s: rest %d, declared %d to %d, fuzz %d, flat %d"
              % (node, name, ABS_NAMES.get(code, hex(code)), value, lo, hi, fuzz, flat))
print("FN ADC (%s) at rest: %s" % (ADC, adc()))

print("\nbuttons:")
drain()
for b in BUTTONS:
    tty("Press and release %s (10 s, or wait to skip) ... " % b, end="")
    got = wait_press(10)
    if got is None:
        print("%s: nothing" % b)
        tty("skipped")
        continue
    how, fd, x = got
    if how == "key":
        node, name, _ = devs[fd]
        print("%s: %s (%s) key code %d (0x%x)" % (b, node, name, x, x))
    else:
        print("%s: no key event, FN ADC dropped to %d" % (b, x))
    tty("ok")
    wait_release(how, fd, x)
    time.sleep(0.2)
    drain()

print("\nsticks:")
tty("Now move BOTH sticks in full circles, all the way to the edges (6 s)...")
lo, hi = {}, {}
end = time.time() + 6
while time.time() < end:
    for fd, kind, code, value in events(0.05):
        if kind == 3:
            k = (fd, code)
            lo[k] = min(lo.get(k, value), value)
            hi[k] = max(hi.get(k, value), value)
tty("Let go.")
for fd, code in sorted(lo, key=lambda k: (devs[k[0]][0], k[1])):
    node, name, _ = devs[fd]
    print("%s %s %s moved %d to %d"
          % (node, name, ABS_NAMES.get(code, hex(code)), lo[(fd, code)], hi[(fd, code)]))
EOF
}

restore() {
	if [ -n "$STOPPED_IP" ]; then
		systemctl start inputplumber 2>/dev/null
		STOPPED_IP=""
		echo "InputPlumber started again."
	fi
}

buttons() {
	section "buttons"
	if ! command -v python3 > /dev/null; then
		echo "skipped: the button test needs python3"
		return
	fi
	STOPPED_IP=""
	trap 'restore; exit 1' HUP INT TERM
	if [ "$(systemctl is-active inputplumber 2>/dev/null)" = active ]; then
		echo "Stopping InputPlumber so the raw button devices can be read."
		systemctl stop inputplumber
		STOPPED_IP=1
		sleep 2
	fi
	run "ls -l /dev/input"
	read_buttons
	restore
	trap - HUP INT TERM
}

{
	echo "slot ROCKNIX probe, $(date)"
	survey
	[ "$MODE" = buttons ] && buttons
	echo
	echo "===== done"
} 2>&1 | tee "$OUT"
echo "Saved to $OUT"
