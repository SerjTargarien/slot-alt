#!/bin/sh
# Slot²'s tests, run from WSL (see HANDOFF.md, "Testing").
#
#   sh tools/slotsq/test.sh              quick: the crates changed since the last commit, and
#                                        slot when any crate it uses changed
#   sh tools/slotsq/test.sh all          full: every crate, before a deploy
#   sh tools/slotsq/test.sh CRATE ...    those crates (slot, slot-ui, slot-store, ...)
#
# Every test runs, failures and all, and the run fails only on failures not listed in
# tools/slotsq/known-failures. The full output is kept in logs/test.log.

set -eu

REPO="$(cd "$(dirname "$0")/../.." && pwd)"
KNOWN="$REPO/tools/slotsq/known-failures"
LOG="$REPO/logs/test.log"
cd "$REPO"
mkdir -p logs

# The crates touched since the last commit, staged, unstaged or new.
changed() {
	{
		git diff --name-only HEAD
		git ls-files --others --exclude-standard
	} | sed -n 's|^crates/\([^/]*\)/.*|\1|p' | sort -u
}

case "${1:-}" in
"")
	crates="$(changed)"
	[ -n "$crates" ] || {
		echo "nothing changed since the last commit"
		exit 0
	}
	# slot uses every other crate, so a change anywhere is tested through it too.
	echo "$crates" | grep -qx slot || crates="$crates slot"
	;;
all) crates="$(ls crates)" ;;
*) crates="$*" ;;
esac

args=""
for c in $crates; do
	args="$args -p $c"
done
echo "testing:$args"
start=$(date +%s)
# `|| true`: failures are judged below, against the known ones.
cargo test --no-fail-fast $args > "$LOG" 2>&1 || true
echo "took $(($(date +%s) - start)) s; output in logs/test.log"

if grep -q "^error\(\[E[0-9]*\]\)\?: \|could not compile" "$LOG" && ! grep -q "^test result" "$LOG"; then
	grep -m 20 -A 6 "^error" "$LOG"
	echo "the tests did not build"
	exit 1
fi

failed="$(sed -n 's/^test \(.*\) \.\.\. FAILED$/\1/p' "$LOG" | sort -u)"
known="$(grep -v '^\s*#' "$KNOWN" | sed '/^\s*$/d' | sort -u)"
new="$(printf '%s\n' "$failed" | grep -vxF "$known" | sed '/^$/d' || true)"
passed="$(sed -n 's/^test result: [A-Za-z]*\. \([0-9]*\) passed.*/\1/p' "$LOG" | paste -sd+ | bc)"
ignored="$(sed -n 's/^test result: .* \([0-9]*\) ignored.*/\1/p' "$LOG" | paste -sd+ | bc)"
echo "$passed passed, $ignored ignored, $(printf '%s\n' "$failed" | sed '/^$/d' | wc -l) failed"

still="$(printf '%s\n' "$failed" | grep -xF "$known" || true)"
[ -z "$still" ] || printf 'known failures:\n%s\n' "$still" | sed 's/^/  /'
if grep -q "could not compile" "$LOG"; then
	grep -B 2 "could not compile" "$LOG" | grep "^error" | sort -u
	echo "some tests did not build"
	exit 1
fi
if [ -n "$new" ]; then
	printf 'NEW FAILURES:\n%s\n' "$new" | sed 's/^/  /'
	exit 1
fi
echo "no new failures"
