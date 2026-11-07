#!/bin/sh
# Slot²: formats the Rust files changed since the last commit, but only those that were
# formatted already at that commit. Many of upstream's files aren't (88 on 2026-11-07),
# and rustfmt rewrites a whole file: run on one of those it reflows code nobody touched,
# which hides the change and makes upstream's harder to take. Those are named instead,
# for their changed lines to be formatted by hand.
#
#     sh tools/slotsq/fmt.sh
set -eu
cd "$(dirname "$0")/../.."
. "$HOME/.cargo/env" 2>/dev/null || true

tmp=$(mktemp)
out=$(mktemp)
trap 'rm -f "$tmp" "$out"' EXIT
changed=$( { git diff --name-only HEAD -- '*.rs'; git ls-files --others --exclude-standard -- '*.rs'; } | sort -u)
for f in $changed; do
	[ -f "$f" ] || continue
	# (rustfmt --check on stdin always passes; formatting the committed text and comparing
	# it does tell.)
	if git cat-file -e "HEAD:$f" 2>/dev/null &&
		! git show "HEAD:$f" | rustfmt --edition 2021 --emit stdout 2>/dev/null |
		cmp -s - "$(git show "HEAD:$f" > "$tmp" && echo "$tmp")"; then
		echo "fmt: left alone, unformatted upstream: $f (format your lines by hand)"
		continue
	fi
	# Through stdin: rustfmt given a path also formats every `mod x;` the file declares,
	# so a changed lib.rs would reflow the whole crate (found by the code review).
	if rustfmt --edition 2021 --emit stdout < "$f" > "$out" 2>/dev/null; then
		cmp -s "$out" "$f" || cat "$out" > "$f"
		echo "fmt: $f"
	else
		echo "fmt: rustfmt failed on $f, left as it is"
	fi
done
