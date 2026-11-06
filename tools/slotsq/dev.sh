#!/bin/sh
# Slot² dev loop: build the frontend and try it on a running Base² (the OS: os/dev.sh).
# Run from WSL with Rust, the aarch64-unknown-linux-gnu target and gcc-aarch64-linux-gnu.
#
#   sh tools/slotsq/dev.sh build                   cross-build slot for the device
#   sh tools/slotsq/dev.sh deploy                  copy it to System/slot (the old one is kept
#                                                  as System/slot.prev) and restart slot
#   sh tools/slotsq/dev.sh restart [VAR=x ...]     restart slot, with extra variables for this
#                                                  start only (/run/slot.env)
#   sh tools/slotsq/dev.sh logs NAME               save Logs/slot.log and Logs/power.log as
#                                                  logs/slot-NAME.log and logs/power-NAME.log
#   sh tools/slotsq/dev.sh shot NAME               save the screen as it is to logs/frames/NAME.png
#   sh tools/slotsq/dev.sh capture NAME [VAR=x ...]
#                                                  restart slot with those variables, then shot
#
# BASESQ (default root@192.168.1.72) is the device, checked against ~/.ssh/known_hosts_basesq.
# Restarting slot stops a game in play: be on the shelf first.

set -eu

BASESQ="${BASESQ:-root@192.168.1.72}"
TARGET=aarch64-unknown-linux-gnu
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$REPO/target/$TARGET/release/slot"
CARD=/mnt/sdcard
KNOWN="$HOME/.ssh/known_hosts_basesq"

on_basesq() {
	ssh -o BatchMode=yes -o UserKnownHostsFile="$KNOWN" "$BASESQ" "$@"
}

build() {
	cd "$REPO"
	cargo build --release --target "$TARGET" -p slot --no-default-features --features device
}

# Restarts slot with VAR=value pairs for this start only: the frontend sources
# /run/slot.env, a tmpfs. Only the restart runs in the background, as its own statement:
# a backgrounded `a && b &` chain dies with the SSH session. slot is found by process
# name (pidof), never by a pattern that could match this very command.
restart() {
	on_basesq "rm -f /run/slot.env; for kv in $*; do echo \"export \$kv\" >> /run/slot.env; done; setsid sh -c 'kill -TERM \$(pidof ld-linux-aarch64.so.1)' < /dev/null > /dev/null 2>&1 &"
	i=0
	while [ $i -lt 12 ]; do
		sleep 5
		if on_basesq "grep -q 'boot: first frame' $CARD/Logs/slot.log" 2> /dev/null; then
			on_basesq "grep 'slot: device' $CARD/Logs/slot.log"
			return 0
		fi
		i=$((i + 1))
	done
	echo "slot did not show its first frame within a minute" >&2
	exit 1
}

deploy() {
	[ -f "$BIN" ] || {
		echo "no $BIN: run tools/slotsq/dev.sh build" >&2
		exit 1
	}
	scp -O -q -o UserKnownHostsFile="$KNOWN" "$BIN" "$BASESQ:$CARD/System/slot.new"
	on_basesq "cd $CARD/System && cp slot slot.prev && mv slot.new slot && sync"
	restart
}

logs() {
	name="$1"
	mkdir -p "$REPO/logs"
	on_basesq "cat $CARD/Logs/slot.log" > "$REPO/logs/slot-$name.log"
	on_basesq "cat $CARD/Logs/power.log" > "$REPO/logs/power-$name.log" 2> /dev/null || rm -f "$REPO/logs/power-$name.log"
	ls "$REPO"/logs/*-"$name".log
}

# Saves the screen as logs/frames/NAME.png: SIGUSR1 makes slot write its next frame to
# Logs/frame-<secs>.png.
shot() {
	name="$1"
	on_basesq "NAME='$name' sh -s" << 'SHOT'
set -eu
before="$(ls /mnt/sdcard/Logs/frame-*.png 2> /dev/null | wc -l)"
kill -USR1 $(pidof ld-linux-aarch64.so.1)
i=0
while [ "$(ls /mnt/sdcard/Logs/frame-*.png 2> /dev/null | wc -l)" -le "$before" ] && [ $i -lt 30 ]; do
	sleep 0.2
	i=$((i + 1))
done
f="$(ls -t /mnt/sdcard/Logs/frame-*.png | head -1)"
mkdir -p /mnt/sdcard/Logs/frames
mv "$f" "/mnt/sdcard/Logs/frames/$NAME.png"
SHOT
	mkdir -p "$REPO/logs/frames"
	on_basesq "cat '$CARD/Logs/frames/$name.png'" > "$REPO/logs/frames/$name.png"
	echo "$REPO/logs/frames/$name.png"
}

cmd="${1:-}"
[ $# -gt 0 ] && shift
case "$cmd" in
build) build ;;
deploy) deploy ;;
restart) restart "$@" ;;
logs) logs "$1" ;;
shot) shot "$1" ;;
capture)
	name="$1"
	shift
	restart "$@"
	shot "$name"
	;;
*)
	sed -n '2,17p' "$0" | sed 's/^# \{0,1\}//'
	exit 2
	;;
esac
