#!/bin/sh
# Finds out why FN does not reach slot. On the RGB20SX, FN is BTN_MODE (316) on its own
# "adc-keys" input device, polled from an ADC. Run over SSH as root, device in hand:
#
#   sh fn-check.sh
#
# Part 1 stops InputPlumber. Part 2 also stops EmulationStation and sway, so the screen
# goes black for about 15 seconds, the way it will while slot runs. Everything is started
# again at the end. Output is printed and saved to /storage/slot-fn-check.txt.

OUT="${PROBE_OUT:-/storage/slot-fn-check.txt}"
TMP="$(mktemp -d)"
STOPPED=""

say() {
	printf '%s\n' "$*" > /dev/tty
}

section() {
	printf '\n===== %s\n' "$1"
}

restore() {
	case "$STOPPED" in
	*inputplumber*) systemctl start inputplumber 2>/dev/null ;;
	esac
	case "$STOPPED" in
	*" sway"*) systemctl start sway 2>/dev/null ;;
	esac
	case "$STOPPED" in
	*essway*) systemctl start essway 2>/dev/null ;;
	esac
	STOPPED=""
	rm -rf "$TMP"
}

stop() {
	if [ "$(systemctl is-active "$1" 2>/dev/null)" = active ]; then
		systemctl stop "$1" && STOPPED="$STOPPED $1"
	fi
}

# Print each 24-byte input_event in a capture as "type code value".
events() {
	od -An -v -tu2 "$1" | tr -s ' \n' '\n\n' | grep . |
		awk '{ w[n++ % 12] = $1 } n % 12 == 0 { v = w[10] + w[11] * 65536; if (v > 2147483647) v -= 4294967296; print w[8], w[9], v }'
}

holders() {
	for f in /proc/[0-9]*/fd/*; do
		t="$(readlink "$f" 2>/dev/null)"
		case "$t" in
		*/"$1")
			pid="${f#/proc/}"
			pid="${pid%%/*}"
			echo "pid $pid $(cat "/proc/$pid/comm" 2>/dev/null) has $t open"
			;;
		esac
	done | sort -u
}

# Opens the node, tries to grab it (EBUSY means another process holds a grab), reads the
# kernel's key state with FN released and held, then listens for events.
python_check() {
	python3 - "$1" <<'EOF'
import errno, fcntl, os, select, sys, time

path = sys.argv[1]
EVIOCGRAB = 0x40044590
KEY_BYTES = 96
EVIOCGKEY = 0x80000000 | (KEY_BYTES << 16) | (ord("E") << 8) | 0x18
BTN_MODE = 316

def tty(msg):
    with open("/dev/tty", "w") as t:
        t.write(msg + "\n")

def held(fd):
    buf = bytearray(KEY_BYTES)
    fcntl.ioctl(fd, EVIOCGKEY, buf)
    return bool(buf[BTN_MODE // 8] & (1 << (BTN_MODE % 8)))

fd = os.open(path, os.O_RDONLY | os.O_NONBLOCK)
try:
    fcntl.ioctl(fd, EVIOCGRAB, 1)
    fcntl.ioctl(fd, EVIOCGRAB, 0)
    print("grab: free, no other process holds a grab")
except OSError as e:
    print("grab: refused, %s" % errno.errorcode.get(e.errno, e.errno))

tty("Leave FN alone for 2 seconds...")
time.sleep(2)
print("kernel says FN pressed, while released: %s" % held(fd))
tty("Now HOLD FN down (3 seconds)...")
time.sleep(3)
print("kernel says FN pressed, while held: %s" % held(fd))
tty("Let go, then press and release FN three times, slowly (8 seconds)...")
end = time.time() + 8
got = 0
while time.time() < end:
    r, _, _ = select.select([fd], [], [], max(0, end - time.time()))
    if not r:
        continue
    data = os.read(fd, 24 * 64)
    for i in range(0, len(data) - 23, 24):
        typ = int.from_bytes(data[i + 16:i + 18], "little")
        code = int.from_bytes(data[i + 18:i + 20], "little")
        val = int.from_bytes(data[i + 20:i + 24], "little", signed=True)
        if typ != 0:
            got += 1
            print("event: type %d code %d value %d" % (typ, code, val))
print("events seen: %d" % got)
EOF
}

capture() {
	say "$2"
	timeout "$1" cat "$NODEPATH" > "$TMP/fn.bin" 2> "$TMP/fn.err"
	echo "captured $(wc -c < "$TMP/fn.bin") bytes"
	cat "$TMP/fn.err"
	events "$TMP/fn.bin" | awk '$1 != 0'
}

check() {
	trap 'restore; exit 1' HUP INT TERM
	section "adc-keys device"
	grep -B1 -A9 'Name="adc-keys"' /proc/bus/input/devices || echo "no device named adc-keys"
	NODE="$(grep -A9 'Name="adc-keys"' /proc/bus/input/devices | grep -o 'event[0-9]*' | head -n 1)"
	INPUT="$(grep -A9 'Name="adc-keys"' /proc/bus/input/devices | sed -n 's|^S: Sysfs=||p' | head -n 1)"
	echo "node: ${NODE:-none}, sysfs: ${INPUT:-none}"
	for a in inhibited poll poll_min poll_max; do
		[ -n "$INPUT" ] && [ -e "/sys$INPUT/$a" ] && echo "$a: $(cat "/sys$INPUT/$a")"
	done
	for f in /sys/bus/iio/devices/iio:device0/in_voltage_scale /sys/bus/iio/devices/iio:device0/name; do
		[ -r "$f" ] && echo "$f: $(cat "$f")"
	done
	dmesg 2>/dev/null | grep -iE 'adc-keys|saradc' | tail -n 5
	command -v python3 > /dev/null && HAVE_PY=1 || HAVE_PY=""
	echo "python3: ${HAVE_PY:+yes}${HAVE_PY:-no}"
	[ -n "$NODE" ] || { restore; return; }

	section "part 1: InputPlumber stopped, sway running"
	stop inputplumber
	sleep 2
	NODEPATH="/dev/input/$NODE"
	ls -l "$NODEPATH" 2>&1
	holders "$NODE"
	if [ -n "$HAVE_PY" ]; then
		python_check "$NODEPATH"
	else
		capture 8 "Press and release FN three times, slowly (8 seconds)..."
	fi

	section "part 2: EmulationStation and sway stopped too"
	say "The screen will go black for about 15 seconds. That is expected."
	stop essway
	stop sway
	sleep 2
	holders "$NODE"
	if [ -n "$HAVE_PY" ]; then
		python_check "$NODEPATH"
	else
		capture 8 "Press and release FN three times, slowly (8 seconds)..."
	fi

	say "Starting everything again..."
	restore
	trap - HUP INT TERM
	sleep 3
	for s in inputplumber sway essway; do
		echo "$s now: $(systemctl is-active "$s" 2>/dev/null)"
	done
}

{
	echo "slot FN check, $(date)"
	check
	echo
	echo "===== done"
} 2>&1 | tee "$OUT"
say "Saved to $OUT"
