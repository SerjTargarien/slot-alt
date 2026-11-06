#!/bin/sh
# Fast dev loop for slot on a ROCKNIX device, with no GitHub or Docker in the way.
# Run from a Linux shell (WSL on Windows) with Rust, the aarch64-unknown-linux-gnu target
# and gcc-aarch64-linux-gnu installed.
#
#   sh tools/rocknix/dev.sh build             cross-compile slot for the device
#   sh tools/rocknix/dev.sh deploy            copy slot and the launch script to the device
#   sh tools/rocknix/dev.sh run [VAR=x ...]   run slot over SSH with tracing on, Ctrl-C quits
#   sh tools/rocknix/dev.sh logs [name]       copy slot.log and power.log into ./logs/
#   sh tools/rocknix/dev.sh all [VAR=x ...]   build, deploy and run
#
# DEVICE (default root@192.168.1.72) and SLOT_DIR (default /storage/slot) pick the target.

set -eu

DEVICE="${DEVICE:-root@192.168.1.72}"
DIR="${SLOT_DIR:-/storage/slot}"
TARGET=aarch64-unknown-linux-gnu
BIN="target/$TARGET/release/slot"

cd "$(dirname "$0")/../.."

build() {
	cargo build --release --target "$TARGET" -p slot --no-default-features --features device
}

deploy() {
	scp "$BIN" "$DEVICE:$DIR/System/slot.new"
	ssh "$DEVICE" "mv -f '$DIR/System/slot.new' '$DIR/System/slot'"
	scp card/rocknix/rocknix.sh "$DEVICE:$DIR/System/rocknix.sh"
}

run() {
	ssh -t "$DEVICE" "SLOT_TRACE_LATENCY=1 SLOT_TRACE_POWER=1 $* bash '$DIR/System/rocknix.sh'"
}

logs() {
	name="${1:-$(date +%H%M%S)}"
	mkdir -p logs
	scp "$DEVICE:$DIR/Logs/slot.log" "logs/slot-$name.log"
	scp "$DEVICE:$DIR/Logs/power.log" "logs/power-$name.log" 2> /dev/null || true
	echo "saved logs/slot-$name.log and logs/power-$name.log"
}

cmd="${1:-}"
[ $# -gt 0 ] && shift
case "$cmd" in
build) build ;;
deploy) deploy ;;
run) run "$@" ;;
logs) logs "$@" ;;
all)
	build
	deploy
	run "$@"
	;;
*)
	sed -n '2,13p' "$0" | sed 's/^# \{0,1\}//'
	exit 2
	;;
esac
