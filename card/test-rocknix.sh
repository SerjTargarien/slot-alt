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

sysfs() {
	rm -rf "$TMP/sys"
	cpu="$TMP/sys/devices/system/cpu/cpufreq/policy0"
	gpu="$TMP/sys/class/devfreq/fde60000.gpu"
	npu="$TMP/sys/class/devfreq/fde40000.npu"
	mkdir -p "$cpu" "$gpu" "$npu"
	echo schedutil > "$cpu/scaling_governor"
	echo 1800000 > "$cpu/scaling_max_freq"
	echo "ondemand userspace powersave performance schedutil" > "$cpu/scaling_available_governors"
	echo simple_ondemand > "$gpu/governor"
	echo "userspace powersave performance simple_ondemand" > "$gpu/available_governors"
	echo simple_ondemand > "$npu/governor"
	echo "userspace powersave performance simple_ondemand" > "$npu/available_governors"
}

setup() {
	sysfs
	rm -rf "$TMP/card" "$TMP/log"
	mkdir -p "$TMP/card/System"
	cp "$SCRIPT" "$TMP/card/System/rocknix.sh"
	: > "$TMP/log"
	cat > "$TMP/card/System/slot" << EOF
echo "root \$SLOT_ROOT platform \$SLOT_PLATFORM"
echo "cpu \$(cat "$TMP/sys/devices/system/cpu/cpufreq/policy0/scaling_governor") gpu \$(cat "$TMP/sys/class/devfreq/fde60000.gpu/governor") npu \$(cat "$TMP/sys/class/devfreq/fde40000.npu/governor")"
echo started > "$TMP/started"
trap 'echo saved; exit 0' TERM
i=0
while [ "\$i" -lt "\${FAKE_RUN:-0}" ]; do
	sleep 0.1
	i=\$((i + 1))
done
[ -n "\${FAKE_CAP:-}" ] && echo "\$FAKE_CAP" > "$TMP/sys/devices/system/cpu/cpufreq/policy0/scaling_max_freq"
exit "\${FAKE_STATUS:-0}"
EOF
	rm -f "$TMP/started"
}

launch() {
	PATH="$BIN:$PATH" FAKE_LOG="$TMP/log" SLOT_LOADER="$TMP/loader" SLOT_PROFILE="$TMP/none" SLOT_SYSFS="$TMP/sys" \
		SLOT_ROOT="$TMP/card" bash "$TMP/card/System/rocknix.sh" > /dev/null 2>&1
}

launch_bg() {
	(
		PATH="$BIN:$PATH" FAKE_LOG="$TMP/log" SLOT_LOADER="$TMP/loader" SLOT_PROFILE="$TMP/none" SLOT_SYSFS="$TMP/sys" \
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

setup
FAKE_ACTIVE="essway sway" launch || fail "a clean exit should return 0"
grep -q "cpu performance gpu performance npu simple_ondemand" "$TMP/card/Logs/slot.log" ||
	fail "slot should run with the CPU and GPU on performance, other devfreq untouched: $(grep '^cpu' "$TMP/card/Logs/slot.log")"
[ "$(cat "$TMP/sys/devices/system/cpu/cpufreq/policy0/scaling_governor")" = schedutil ] ||
	fail "the CPU governor was not put back"
[ "$(cat "$TMP/sys/class/devfreq/fde60000.gpu/governor")" = simple_ondemand ] ||
	fail "the GPU governor was not put back"

setup
FAKE_ACTIVE="essway sway" SLOT_GOVERNOR=keep launch || fail "a clean exit should return 0"
grep -q "cpu schedutil gpu simple_ondemand" "$TMP/card/Logs/slot.log" ||
	fail "SLOT_GOVERNOR=keep should leave the governors alone"

setup
FAKE_ACTIVE="essway sway" FAKE_STATUS=3 launch || true
[ "$(cat "$TMP/sys/devices/system/cpu/cpufreq/policy0/scaling_governor")" = schedutil ] ||
	fail "the CPU governor must come back when slot fails"

setup
echo powersave > "$TMP/sys/devices/system/cpu/cpufreq/policy0/scaling_governor"
FAKE_ACTIVE="essway sway" SLOT_GOVERNOR=schedutil launch || fail "a clean exit should return 0"
grep -q "cpu schedutil gpu simple_ondemand" "$TMP/card/Logs/slot.log" ||
	fail "a named governor goes only where it is offered: $(grep '^cpu' "$TMP/card/Logs/slot.log")"
[ "$(cat "$TMP/sys/devices/system/cpu/cpufreq/policy0/scaling_governor")" = powersave ] ||
	fail "the CPU governor was not put back after a named governor"

setup
bat="$TMP/sys/class/power_supply/battery"
mkdir -p "$bat"
echo Battery > "$bat/type"
echo -786556 > "$bat/current_avg"
echo 3772740 > "$bat/voltage_avg"
echo 81 > "$bat/capacity"
echo Discharging > "$bat/status"
echo 1800000 > "$TMP/sys/devices/system/cpu/cpufreq/policy0/scaling_cur_freq"
echo 800000000 > "$TMP/sys/class/devfreq/fde60000.gpu/cur_freq"
FAKE_ACTIVE="essway sway" FAKE_RUN=5 SLOT_TRACE_POWER=1 SLOT_POWER_EVERY=0.1 launch ||
	fail "a clean exit should return 0"
grep -q "slot starting, governor performance" "$TMP/card/Logs/power.log" || fail "the power log has no header"
grep -q "battery 2964 mW 81% Discharging cpu 1800 MHz gpu 800 MHz" "$TMP/card/Logs/power.log" ||
	fail "the power log lines are wrong: $(sed -n 2p "$TMP/card/Logs/power.log")"

setup
FAKE_ACTIVE="essway sway" launch || true
[ ! -f "$TMP/card/Logs/power.log" ] || fail "no power log unless SLOT_TRACE_POWER is set"

for governor in performance keep; do
	setup
	FAKE_ACTIVE="essway sway" FAKE_CAP=816000 FAKE_STATUS=3 SLOT_GOVERNOR=$governor launch || true
	[ "$(cat "$TMP/sys/devices/system/cpu/cpufreq/policy0/scaling_max_freq")" = 1800000 ] ||
		fail "a clock cap left by a crashed slot must be lifted (SLOT_GOVERNOR=$governor)"
done

echo "rocknix launch: ok"
