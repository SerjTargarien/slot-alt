#!/bin/bash
# Runs slot on ROCKNIX in place of EmulationStation, then hands the device back.
#
# It stops EmulationStation, sway, InputPlumber and ROCKNIX's hotkey daemon, keeps logind
# from suspending on the power button, switches the CPU, GPU and memory clocks to the
# performance governor as ROCKNIX does for games, and runs slot. However slot ends,
# everything is put back. Ctrl-C, or the SSH session closing, asks slot to save and quit.
# SLOT_GOVERNOR=keep leaves the governors alone, and SLOT_GOVERNOR=<name> uses that
# governor wherever it is offered. SLOT_TRACE_POWER=1 logs battery draw and clocks to
# Logs/power.log every SLOT_POWER_EVERY seconds (5 by default).

SYS="$(cd "$(dirname "$0")" && pwd)"
SD="${SLOT_ROOT:-$(dirname "$SYS")}"
LOGS="$SD/Logs"
LOADER="${SLOT_LOADER:-/usr/lib/ld-linux-aarch64.so.1}"
PROFILE="${SLOT_PROFILE:-/etc/profile}"
SYSFS="${SLOT_SYSFS:-/sys}"
STOP_ORDER="essway sway inputplumber input"
START_ORDER="inputplumber input sway essway"

STOPPED=""
SLOT_PID=""
INHIBIT_PID=""
GOVERNORS=""

[ -r "$PROFILE" ] && . "$PROFILE" > /dev/null 2>&1
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/var/run/0-runtime-dir}"
export SLOT_ROOT="$SD"
export SLOT_PLATFORM=rocknix

offers() {
	case "${1##*/}" in
	scaling_governor) list="${1%/*}/scaling_available_governors" ;;
	*) list="${1%/*}/available_governors" ;;
	esac
	grep -qw "$2" "$list" 2> /dev/null
}

boost() {
	want="${SLOT_GOVERNOR:-performance}"
	GOVERNORS="$(mktemp)"
	for f in "$SYSFS"/devices/system/cpu/cpufreq/policy*/scaling_max_freq; do
		[ -w "$f" ] && echo "$f $(cat "$f")" >> "$GOVERNORS"
	done
	[ "$want" = keep ] && return
	for f in "$SYSFS"/devices/system/cpu/cpufreq/policy*/scaling_governor "$SYSFS"/class/devfreq/*/governor; do
		[ -w "$f" ] || continue
		case "$f" in
		*/devfreq/*gpu*/governor | */devfreq/*dmc*/governor | */scaling_governor) ;;
		*) continue ;;
		esac
		offers "$f" "$want" || continue
		echo "$f $(cat "$f")" >> "$GOVERNORS"
		echo "$want" > "$f"
	done
}

unboost() {
	[ -n "$GOVERNORS" ] && [ -f "$GOVERNORS" ] || return
	while read -r f governor; do
		echo "$governor" > "$f"
	done < "$GOVERNORS"
	rm -f "$GOVERNORS"
	GOVERNORS=""
}

restore() {
	[ -n "$INHIBIT_PID" ] && kill "$INHIBIT_PID" 2> /dev/null
	INHIBIT_PID=""
	unboost
	for s in $START_ORDER; do
		case " $STOPPED " in
		*" $s "*) systemctl start "$s" ;;
		esac
	done
	STOPPED=""
}

stop_slot() {
	[ -n "$SLOT_PID" ] && kill -TERM "$SLOT_PID" 2> /dev/null
}

trap restore EXIT
trap stop_slot INT TERM HUP

for s in $STOP_ORDER; do
	if [ "$(systemctl is-active "$s" 2> /dev/null)" = active ]; then
		systemctl stop "$s" && STOPPED="$STOPPED $s"
	fi
done

if command -v systemd-inhibit > /dev/null 2>&1; then
	systemd-inhibit --what=handle-power-key --who=slot --why="slot handles the power button" \
		--mode=block sh -c "while kill -0 $$ 2> /dev/null; do sleep 1; done" &
	INHIBIT_PID=$!
fi

boost
if [ -n "$GOVERNORS" ]; then
	while read -r f governor; do
		case "$f" in
		*governor) echo "governor: ${f#"$SYSFS"/} was $governor, now $want" ;;
		esac
	done < "$GOVERNORS"
fi

power_line() {
	line="$(date +%T)"
	for p in "$SYSFS"/class/power_supply/*; do
		[ -r "$p/current_avg" ] && [ -r "$p/voltage_avg" ] || continue
		i="$(cat "$p/current_avg")"
		v="$(cat "$p/voltage_avg")"
		line="$line battery $((${i#-} / 1000 * (v / 1000) / 1000)) mW $(cat "$p/capacity" 2> /dev/null)% $(cat "$p/status" 2> /dev/null)"
		break
	done
	for f in "$SYSFS"/devices/system/cpu/cpufreq/policy*/scaling_cur_freq; do
		[ -r "$f" ] && line="$line cpu $(($(cat "$f") / 1000)) MHz"
	done
	for f in "$SYSFS"/class/devfreq/*gpu*/cur_freq; do
		[ -r "$f" ] && line="$line gpu $(($(cat "$f") / 1000000)) MHz"
	done
	echo "$line"
}

trace_power() {
	case "${SLOT_TRACE_POWER:-0}" in
	0) return ;;
	esac
	[ -f "$LOGS/power.log" ] && mv -f "$LOGS/power.log" "$LOGS/power.log.1"
	echo "$(date +%T) slot starting, governor ${SLOT_GOVERNOR:-performance}" > "$LOGS/power.log"
	(
		while kill -0 $$ 2> /dev/null; do
			power_line >> "$LOGS/power.log"
			sleep "${SLOT_POWER_EVERY:-5}"
		done
	) &
}

mkdir -p "$LOGS"
[ -f "$LOGS/slot.log" ] && mv -f "$LOGS/slot.log" "$LOGS/slot.log.1"
trace_power
echo "slot is running. Press Ctrl-C here to save and go back to ROCKNIX."
(
	trap '' HUP INT
	exec "$LOADER" "$SYS/slot"
) < /dev/null > "$LOGS/slot.log" 2>&1 &
SLOT_PID=$!

status=0
while :; do
	wait "$SLOT_PID"
	status=$?
	kill -0 "$SLOT_PID" 2> /dev/null || break
done
echo "slot exited with status $status. Its log is $LOGS/slot.log"
exit "$status"
