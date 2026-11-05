#!/bin/sh
# Finds out why FN does not reach slot. On the RGB20SX, FN is BTN_MODE (316) on its own
# "adc-keys" input device, read from an ADC. Run over SSH as root, with the device in hand:
#
#   sh fn-check.sh
#
# It stops InputPlumber for about 30 seconds and starts it again at the end. Output is
# printed and saved to /storage/slot-fn-check.txt.

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
	if [ -n "$STOPPED" ]; then
		systemctl start inputplumber 2>/dev/null
		STOPPED=""
	fi
	rm -rf "$TMP"
}

# Print each 24-byte input_event in a capture as "type code value".
events() {
	od -An -v -tu2 "$1" | tr -s ' \n' '\n\n' | grep . |
		awk '{ w[n++ % 12] = $1 } n % 12 == 0 { v = w[10] + w[11] * 65536; if (v > 2147483647) v -= 4294967296; print w[8], w[9], v }'
}

adc_raw() {
	for f in /sys/bus/iio/devices/iio:device*/in_voltage0_raw; do
		[ -r "$f" ] && printf '%s=%s ' "$f" "$(cat "$f")"
	done
	echo
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

check() {
	trap 'restore; exit 1' HUP INT TERM
	section "adc-keys in /proc/bus/input/devices"
	grep -B1 -A9 'Name="adc-keys"' /proc/bus/input/devices || echo "no device named adc-keys"
	NODE="$(grep -A9 'Name="adc-keys"' /proc/bus/input/devices | grep -o 'event[0-9]*' | head -n 1)"
	echo "node: ${NODE:-none}"

	section "InputPlumber"
	systemctl show inputplumber -p ActiveState -p Restart -p TriggeredBy -p BusName 2>&1
	[ -n "$NODE" ] && holders "$NODE"
	if [ "$(systemctl is-active inputplumber 2>/dev/null)" = active ]; then
		systemctl stop inputplumber
		STOPPED=1
		sleep 2
	fi
	echo "after stop: $(systemctl is-active inputplumber 2>/dev/null)"
	ls -l /dev/input /dev/inputplumber/sources 2>&1

	path=""
	for p in "/dev/input/$NODE" "/dev/inputplumber/sources/$NODE"; do
		[ -n "$NODE" ] && [ -c "$p" ] && path="$p" && break
	done
	echo "reading: ${path:-not found}"
	[ -n "$NODE" ] && holders "$NODE"

	section "ADC voltage, FN released then held"
	say "Leave FN alone for 3 seconds..."
	sleep 3
	echo "released: $(adc_raw)"
	say "Now HOLD FN down (5 seconds)..."
	sleep 4
	echo "held: $(adc_raw)"
	say "Let go."

	section "FN events"
	if [ -n "$path" ]; then
		say "Press and release FN three times, slowly (10 seconds)..."
		timeout 10 cat "$path" > "$TMP/fn.bin" 2> "$TMP/fn.err"
		echo "captured $(wc -c < "$TMP/fn.bin") bytes"
		cat "$TMP/fn.err"
		events "$TMP/fn.bin" | awk '$1 != 0'
	else
		echo "skipped: no adc-keys node to read"
	fi
	echo "InputPlumber during the test: $(systemctl is-active inputplumber 2>/dev/null)"

	restore
	trap - HUP INT TERM
	echo "InputPlumber now: $(systemctl is-active inputplumber 2>/dev/null)"
}

{
	echo "slot FN check, $(date)"
	check
	echo
	echo "===== done"
} 2>&1 | tee "$OUT"
say "Saved to $OUT"
