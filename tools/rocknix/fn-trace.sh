#!/bin/sh
# Watches the kernel's adc-keys driver while FN is held, to see why the RGB20SX never
# reports BTN_MODE. Run over SSH as root, device in hand:
#
#   sh fn-trace.sh
#
# Part 1 traces each poll of the driver and the voltage it reads (kprobes). Part 2
# unbinds and rebinds the driver, then tests FN again. InputPlumber is stopped for
# about a minute and started again at the end. If anything goes wrong, a reboot puts
# everything back. Output is printed and saved to /storage/slot-fn-trace.txt.

OUT="${PROBE_OUT:-/storage/slot-fn-trace.txt}"
STOPPED=""
T=""

say() {
	printf '%s\n' "$*" > /dev/tty
}

section() {
	printf '\n===== %s\n' "$1"
}

unprobe() {
	[ -n "$T" ] || return 0
	echo 0 > "$T/events/slot/enable" 2>/dev/null
	for e in poll read key; do
		echo "-:slot/$e" >> "$T/kprobe_events" 2>/dev/null
	done
}

restore() {
	unprobe
	if [ -n "$STOPPED" ]; then
		systemctl start inputplumber 2>/dev/null
		STOPPED=""
	fi
}

node_of() {
	grep -A9 'Name="adc-keys"' /proc/bus/input/devices | grep -o 'event[0-9]*' | head -n 1
}

# Reads the kernel's key state for BTN_MODE released and held, then listens for events.
fn_test() {
	python3 - "$1" <<'EOF'
import fcntl, os, select, sys, time

fd = os.open(sys.argv[1], os.O_RDONLY | os.O_NONBLOCK)
KEY_BYTES = 96
EVIOCGKEY = 0x80000000 | (KEY_BYTES << 16) | (ord("E") << 8) | 0x18
BTN_MODE = 316

def tty(msg):
    with open("/dev/tty", "w") as t:
        t.write(msg + "\n")

def held():
    buf = bytearray(KEY_BYTES)
    fcntl.ioctl(fd, EVIOCGKEY, buf)
    return bool(buf[BTN_MODE // 8] & (1 << (BTN_MODE % 8)))

tty("Leave FN alone for 2 seconds...")
time.sleep(2)
print("kernel says FN pressed, while released: %s" % held())
tty("Now HOLD FN down (3 seconds)...")
time.sleep(3)
print("kernel says FN pressed, while held: %s" % held())
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

probe() {
	if echo "$1" >> "$T/kprobe_events" 2>/dev/null; then
		echo "probe ok: $1"
	else
		echo "probe refused: $1"
		tail -n 2 "$T/error_log" 2>/dev/null
	fi
}

trace_part() {
	section "part 1: trace the driver"
	T=/sys/kernel/tracing
	[ -e "$T/kprobe_events" ] || mount -t tracefs nodev "$T" 2>/dev/null
	[ -e "$T/kprobe_events" ] || T=/sys/kernel/debug/tracing
	if [ ! -e "$T/kprobe_events" ]; then
		echo "no kprobe support in this kernel, skipping the trace"
		T=""
		return
	fi
	grep -sw adc_keys_poll /proc/kallsyms || echo "adc_keys_poll is not in /proc/kallsyms"
	unprobe
	probe 'p:slot/poll adc_keys_poll'
	probe 'r:slot/read iio_read_channel_processed_scale ret=$retval:s32 val=+0($arg2):s32'
	probe 'p:slot/key input_event type=%x1:u32 code=%x2:u32 value=%x3:s32'
	echo 'code == 316' > "$T/events/slot/key/filter" 2>/dev/null
	: > "$T/trace"
	echo 1 > "$T/events/slot/enable" 2>/dev/null
	say "Leave FN alone for 2 seconds..."
	sleep 2
	echo "-- FN held from here" > "$T/trace_marker" 2>/dev/null
	say "Now HOLD FN down (3 seconds)..."
	sleep 3
	echo "-- FN released from here" > "$T/trace_marker" 2>/dev/null
	say "Let go."
	sleep 1
	echo 0 > "$T/events/slot/enable" 2>/dev/null
	grep -v '^#' "$T/trace" > /tmp/slot-fn-trace.raw
	echo "trace lines: $(wc -l < /tmp/slot-fn-trace.raw)"
	awk '
		/-- FN held/ { phase = "held" }
		/-- FN released/ { phase = "after" }
		{ if (phase == "") p = "released"; else p = phase }
		/ poll:/ { polls[p]++ }
		/ read:/ { r = $0; sub(/.*\) /, "", r); reads[p " " r]++ }
		/ key:/ { k = $0; sub(/.*\) /, "", k); keys[p " " k]++ }
		END {
			for (x in polls) printf "polls while %s: %d\n", x, polls[x]
			for (x in reads) printf "reads while %s: %d\n", x, reads[x]
			for (x in keys) printf "key events while %s: %d\n", x, keys[x]
		}' /tmp/slot-fn-trace.raw | sort
	echo "first lines of the trace:"
	head -n 12 /tmp/slot-fn-trace.raw
	rm -f /tmp/slot-fn-trace.raw
	unprobe
}

rebind_part() {
	section "part 2: rebind the driver"
	dev=/sys/devices/platform/adc-keys
	drv="$(readlink -f "$dev/driver" 2>/dev/null)"
	echo "driver: ${drv:-none}"
	if [ -z "$drv" ]; then
		echo "adc-keys has no driver bound, skipping"
		return
	fi
	echo adc-keys > "$drv/unbind" && echo "unbound"
	sleep 1
	echo adc-keys > "$drv/bind" && echo "bound again"
	sleep 2
	n="$(node_of)"
	echo "node now: ${n:-none}"
	[ -n "$n" ] && fn_test "/dev/input/$n"
}

check() {
	trap 'restore; exit 1' HUP INT TERM
	section "kernel"
	uname -r
	command -v python3 > /dev/null || echo "python3 missing, part 2 cannot run"
	n="$(node_of)"
	echo "adc-keys node: ${n:-none}"
	if [ "$(systemctl is-active inputplumber 2>/dev/null)" = active ]; then
		systemctl stop inputplumber && STOPPED=1
		sleep 2
	fi
	trace_part
	rebind_part
	restore
	trap - HUP INT TERM
	echo "inputplumber now: $(systemctl is-active inputplumber 2>/dev/null)"
}

{
	echo "slot FN trace, $(date)"
	check
	echo
	echo "===== done"
} 2>&1 | tee "$OUT"
say "Saved to $OUT"
