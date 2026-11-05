#!/bin/sh
set -eu

HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="$HERE/rocknix/rocknix.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT HUP INT TERM

BIN="$TMP/bin"
mkdir -p "$BIN"

cat > "$BIN/systemctl" << 'EOF'
#!/bin/sh
case "$1" in
is-active)
	case " $FAKE_ACTIVE " in
	*" $2 "*) echo active ;;
	*) echo inactive ;;
	esac
	;;
*) echo "$1 $2" >> "$FAKE_LOG" ;;
esac
EOF

cat > "$BIN/systemd-inhibit" << 'EOF'
#!/bin/sh
echo "inhibit $1" >> "$FAKE_LOG"
while [ $# -gt 0 ]; do
	case "$1" in
	--*) shift ;;
	*) break ;;
	esac
done
exec "$@"
EOF

cat > "$TMP/loader" << 'EOF'
#!/bin/sh
exec sh "$1"
EOF

chmod 755 "$BIN/systemctl" "$BIN/systemd-inhibit" "$TMP/loader"

setup() {
	rm -rf "$TMP/card" "$TMP/log"
	mkdir -p "$TMP/card/System"
	cp "$SCRIPT" "$TMP/card/System/rocknix.sh"
	: > "$TMP/log"
	cat > "$TMP/card/System/slot" << EOF
echo "root \$SLOT_ROOT platform \$SLOT_PLATFORM"
echo started > "$TMP/started"
trap 'echo saved; exit 0' TERM
i=0
while [ "\$i" -lt "\${FAKE_RUN:-0}" ]; do
	sleep 0.1
	i=\$((i + 1))
done
exit "\${FAKE_STATUS:-0}"
EOF
	rm -f "$TMP/started"
}

launch() {
	PATH="$BIN:$PATH" FAKE_LOG="$TMP/log" SLOT_LOADER="$TMP/loader" SLOT_PROFILE="$TMP/none" \
		SLOT_ROOT="$TMP/card" bash "$TMP/card/System/rocknix.sh" > /dev/null 2>&1
}

launch_bg() {
	(
		PATH="$BIN:$PATH" FAKE_LOG="$TMP/log" SLOT_LOADER="$TMP/loader" SLOT_PROFILE="$TMP/none" \
			SLOT_ROOT="$TMP/card" exec bash "$TMP/card/System/rocknix.sh" > /dev/null 2>&1
	) &
	pid=$!
}

calls() {
	grep -v '^inhibit' "$TMP/log" | tr '\n' ',' | sed 's/,$//'
}

fail() {
	echo "$1" >&2
	exit 1
}

setup
FAKE_ACTIVE="essway sway inputplumber input" launch || fail "a clean exit should return 0"
want="stop essway,stop sway,stop inputplumber,stop input,start inputplumber,start input,start sway,start essway"
[ "$(calls)" = "$want" ] || fail "services stopped and started out of order: $(calls)"
grep -q '^inhibit --what=handle-power-key' "$TMP/log" || fail "the power key was not inhibited"
grep -q "root $TMP/card platform rocknix" "$TMP/card/Logs/slot.log" || fail "slot did not get its root and platform"

setup
FAKE_ACTIVE="sway inputplumber" launch || true
[ "$(calls)" = "stop sway,stop inputplumber,start inputplumber,start sway" ] ||
	fail "a service that was not running should be left alone: $(calls)"

setup
mkdir -p "$TMP/card/Logs"
echo previous > "$TMP/card/Logs/slot.log"
status=0
FAKE_ACTIVE="essway sway" FAKE_STATUS=3 launch || status=$?
[ "$status" = 3 ] || fail "slot's exit status should come back, got $status"
[ "$(calls)" = "stop essway,stop sway,start sway,start essway" ] ||
	fail "services must come back when slot fails: $(calls)"
grep -q previous "$TMP/card/Logs/slot.log.1" || fail "the previous log was not kept"

for sig in TERM HUP; do
	setup
	FAKE_ACTIVE="essway sway inputplumber input" FAKE_RUN=100 launch_bg
	i=0
	while [ ! -f "$TMP/started" ] && [ "$i" -lt 50 ]; do
		sleep 0.1
		i=$((i + 1))
	done
	[ -f "$TMP/started" ] || fail "slot never started"
	kill -"$sig" "$pid"
	wait "$pid" || true
	grep -q saved "$TMP/card/Logs/slot.log" || fail "SIG$sig did not ask slot to save and quit"
	[ "$(calls)" = "$want" ] || fail "services must come back after SIG$sig: $(calls)"
done

echo "rocknix launch: ok"
