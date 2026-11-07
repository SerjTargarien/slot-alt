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
# DEVICE picks the handheld (design/two-devices.md): rgb20sx (the default; Base²,
# root@192.168.1.72, ~/.ssh/known_hosts_basesq) or rg34xx (BaseOS, root@192.168.1.75,
# ~/.ssh/known_hosts_rg35xx). BASESQ still names another address. Both start slot again
# by themselves when it exits, so a restart is the same on both.
# Restarting slot stops a game in play: be on the shelf first.

set -eu

DEVICE="${DEVICE:-rgb20sx}"
case "$DEVICE" in
rgb20sx) BASESQ="${BASESQ:-root@192.168.1.72}" KNOWN="$HOME/.ssh/known_hosts_basesq" ;;
rg34xx) BASESQ="${BASESQ:-root@192.168.1.75}" KNOWN="$HOME/.ssh/known_hosts_rg35xx" ;;
*)
	echo "DEVICE is rgb20sx or rg34xx, not $DEVICE" >&2
	exit 2
	;;
esac
TARGET=aarch64-unknown-linux-gnu
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$REPO/target/$TARGET/release/slot"
CARD=/mnt/sdcard

on_basesq() {
	ssh -o BatchMode=yes -o UserKnownHostsFile="$KNOWN" "$BASESQ" "$@"
}

# One build for both devices: linked against Debian bullseye's glibc 2.31
# (tools/slotsq/sysroot.sh), so it runs on BaseOS 1.4.0 (2.35) as on Base² (2.44).
SYSROOT="$REPO/vendor/sysroot-bullseye/root"
GLIBC_MAX=2.31

build() {
	cd "$REPO"
	[ -d "$SYSROOT" ] || {
		echo "no $SYSROOT: run tools/slotsq/sysroot.sh" >&2
		exit 1
	}
	# --sysroot alone isn't enough: Ubuntu's cross gcc looks in its own glibc 2.39
	# (/usr/aarch64-linux-gnu/lib) first, startup files too. -B puts the sysroot first.
	libs="-B$SYSROOT/usr/lib/aarch64-linux-gnu/ -B$SYSROOT/lib/aarch64-linux-gnu/"
	flags="--sysroot=$SYSROOT $libs"
	link=""
	for f in $flags; do link="$link -C link-arg=$f"; done
	CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="$link" \
		CFLAGS_aarch64_unknown_linux_gnu="$flags" \
		CXXFLAGS_aarch64_unknown_linux_gnu="$flags" \
		cargo build --release --target "$TARGET" -p slot --no-default-features --features device
	glibc_check
}

# Refuses a binary that needs a newer glibc than both devices have.
glibc_check() {
	need="$(aarch64-linux-gnu-readelf -V "$BIN" | grep -oE 'GLIBC_[0-9]+\.[0-9]+' | sed 's/GLIBC_//' | sort -V | tail -1)"
	if [ "$(printf '%s\n%s\n' "$need" "$GLIBC_MAX" | sort -V | tail -1)" != "$GLIBC_MAX" ]; then
		echo "slot needs glibc $need, more than $GLIBC_MAX: it wouldn't start on BaseOS" >&2
		exit 1
	fi
	echo "slot needs glibc $need at most"
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
	glibc_check
	# Through ssh, not scp: Base² has no SFTP server.
	on_basesq "cat > $CARD/System/slot.new" < "$BIN"
	want="$(md5sum < "$BIN" | cut -d' ' -f1)"
	got="$(on_basesq "md5sum < $CARD/System/slot.new" | cut -d' ' -f1)"
	[ "$want" = "$got" ] || {
		echo "the copy on the device differs from the build" >&2
		exit 1
	}
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
	sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'
	exit 2
	;;
esac
