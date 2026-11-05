#!/bin/bash
# Runs slot on ROCKNIX in place of EmulationStation, then hands the device back.
#
# It stops EmulationStation, sway, InputPlumber and ROCKNIX's hotkey daemon, keeps logind
# from suspending on the power button, and runs slot. However slot ends, everything is
# started again. Ctrl-C, or the SSH session closing, asks slot to save and quit.

SYS="$(cd "$(dirname "$0")" && pwd)"
SD="${SLOT_ROOT:-$(dirname "$SYS")}"
LOGS="$SD/Logs"
LOADER="${SLOT_LOADER:-/usr/lib/ld-linux-aarch64.so.1}"
PROFILE="${SLOT_PROFILE:-/etc/profile}"
STOP_ORDER="essway sway inputplumber input"
START_ORDER="inputplumber input sway essway"

STOPPED=""
SLOT_PID=""
INHIBIT_PID=""

[ -r "$PROFILE" ] && . "$PROFILE" > /dev/null 2>&1
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/var/run/0-runtime-dir}"
export SLOT_ROOT="$SD"
export SLOT_PLATFORM=rocknix

restore() {
	[ -n "$INHIBIT_PID" ] && kill "$INHIBIT_PID" 2> /dev/null
	INHIBIT_PID=""
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

mkdir -p "$LOGS"
[ -f "$LOGS/slot.log" ] && mv -f "$LOGS/slot.log" "$LOGS/slot.log.1"
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
