#!/bin/sh
# Finds out whether the kernel's adc-keys driver polls the ADC at all on the RGB20SX.
# Every ADC conversion raises the saradc interrupt, so the interrupt rate is compared
# with adc-keys bound, unbound, and bound again. Polling every 60 ms should account for
# about 17 interrupts a second. No buttons need pressing. Run over SSH as root:
#
#   sh fn-irq.sh
#
# adc-keys is unbound for about 5 seconds and bound again. A reboot undoes everything.
# Output is printed and saved to /storage/slot-fn-irq.txt.

OUT="${PROBE_OUT:-/storage/slot-fn-irq.txt}"
DEV=/sys/devices/platform/adc-keys
SECS=5

irqs() {
	awk '/saradc/ {
		s = 0
		for (i = 2; i <= NF && $i ~ /^[0-9]+$/; i++) s += $i
		print s
		exit
	}' /proc/interrupts
}

rate() {
	a="$(irqs)"
	sleep "$SECS"
	b="$(irqs)"
	echo "$1: $((b - a)) saradc interrupts in $SECS s, $(((b - a) / SECS)) a second"
}

node_path() {
	n="$(grep -A9 'Name="adc-keys"' /proc/bus/input/devices | grep -o 'event[0-9]*' | head -n 1)"
	for p in "/dev/input/$n" "/dev/inputplumber/sources/$n"; do
		[ -n "$n" ] && [ -c "$p" ] && echo "$p" && return
	done
}

# Keeps the adc-keys node open, so its poller runs even if nothing else reopened it.
hold_open() {
	python3 -c 'import os, sys, time; os.open(sys.argv[1], os.O_RDONLY); time.sleep(float(sys.argv[2]))' "$1" "$2" &
	HOLDER=$!
}

{
	echo "slot FN interrupt check, $(date)"
	echo
	echo "===== kernel"
	uname -r
	if [ -r /proc/config.gz ]; then
		zcat /proc/config.gz | grep -E '^CONFIG_(KEYBOARD_ADC|INPUT_EVDEV|ROCKCHIP_SARADC|KPROBES|KPROBE_EVENTS|FTRACE|FUNCTION_TRACER|DYNAMIC_FTRACE|INPUT_ADC_JOYSTICK|IIO_MUX)='
	else
		echo "no /proc/config.gz"
	fi
	[ -d /sys/module/adc_keys ] && echo "adc_keys is a module" || echo "adc_keys is built in (or absent)"
	mount | grep -E 'tracefs|debugfs'
	[ -r /sys/kernel/tracing/available_tracers ] && echo "tracers: $(cat /sys/kernel/tracing/available_tracers)"
	grep -i saradc /proc/interrupts || echo "no saradc line in /proc/interrupts"

	echo
	echo "===== who has adc-keys open"
	p="$(node_path)"
	echo "node: ${p:-none}"
	for f in /proc/[0-9]*/fd/*; do
		t="$(readlink "$f" 2>/dev/null)"
		[ -n "$p" ] && [ "$t" = "$p" ] && { pid="${f#/proc/}"; pid="${pid%%/*}"; echo "pid $pid $(cat "/proc/$pid/comm" 2>/dev/null)"; }
	done | sort -u

	echo
	echo "===== interrupt rate"
	[ -n "$p" ] && hold_open "$p" $((SECS + 2))
	rate "adc-keys bound"
	[ -n "$HOLDER" ] && wait "$HOLDER" 2>/dev/null
	drv="$(readlink -f "$DEV/driver" 2>/dev/null)"
	if [ -n "$drv" ]; then
		echo adc-keys > "$drv/unbind"
		sleep 1
		rate "adc-keys unbound"
		echo adc-keys > "$drv/bind"
		sleep 2
		p="$(node_path)"
		HOLDER=""
		[ -n "$p" ] && hold_open "$p" $((SECS + 2))
		rate "adc-keys bound again"
		[ -n "$HOLDER" ] && wait "$HOLDER" 2>/dev/null
	else
		echo "adc-keys has no driver bound"
	fi
	echo
	echo "===== done"
} 2>&1 | tee "$OUT"
echo "Saved to $OUT"
