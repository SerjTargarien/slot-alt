#!/bin/sh
# Slot²: the sysroot slot is linked against, so one build runs on Base² (glibc 2.44) and on
# BaseOS 1.4.0 (glibc 2.35): Debian bullseye's arm64 glibc 2.31, as upstream builds in
# (taskfile.yml, rust:1-bullseye). Built here, Ubuntu 24.04's 2.39 made slot need
# GLIBC_2.39, which BaseOS refuses (design/two-devices.md, phase 1).
#
#     sh tools/slotsq/sysroot.sh        fetch, check and unpack into vendor/sysroot-bullseye
#
# The packages are pinned by checksum, from bullseye-security (glibc, kernel headers) and
# bullseye (libgcc), on archive.debian.org, which keeps them (the security mirror had
# already dropped them on 2026-11-07). Unpacked with dpkg-deb -x: their scripts never run.
set -eu
cd "$(dirname "$0")/../.."
OUT=vendor/sysroot-bullseye
DEBS="$OUT/debs"
ROOT="$OUT/root"

# file  sha256  base URL
PKGS="
libc6_2.31-13+deb11u14_arm64.deb d73eeec775c9c13bdac6b7afcb075e011920a9ecac28dc169e4c405b0dd90cd5 https://archive.debian.org/debian-security/pool/updates/main/g/glibc
libc6-dev_2.31-13+deb11u14_arm64.deb e7da98c4a0e13c1935a1410d56c5f4b9217864c715eeac9e79acf08fe993df3e https://archive.debian.org/debian-security/pool/updates/main/g/glibc
linux-libc-dev_5.10.262-1_arm64.deb 5060053f18a14a9be3c4d27274ab81598809d7927083d9cd8ca738aeb110885e https://archive.debian.org/debian-security/pool/updates/main/l/linux
libgcc-s1_10.2.1-6_arm64.deb e2fcdb378d3c1ad1bcb64d4fb6b37aab44011152beca12a4944f435a2582df1f https://archive.debian.org/debian/pool/main/g/gcc-10
"

mkdir -p "$DEBS"
rm -rf "$ROOT"
mkdir -p "$ROOT"
echo "$PKGS" | while read -r file sum url; do
	[ -n "$file" ] || continue
	deb="$DEBS/$file"
	if [ ! -f "$deb" ] || ! echo "$sum  $deb" | sha256sum -c --quiet - 2>/dev/null; then
		curl -fsSL -o "$deb.part" "$url/$file"
		mv "$deb.part" "$deb"
	fi
	echo "$sum  $deb" | sha256sum -c --quiet - || {
		echo "sysroot: $file doesn't match its checksum" >&2
		rm -f "$deb"
		exit 1
	}
	dpkg-deb -x "$deb" "$ROOT"
	echo "sysroot: $file"
done
# Debian's links are absolute (libm.so -> /lib/aarch64-linux-gnu/libm.so.6): read here,
# they'd point at this machine's own libraries. Made relative, inside the sysroot.
find "$ROOT" -type l | while read -r link; do
	to="$(readlink "$link")"
	case "$to" in
	/*) ln -sfn "$(realpath -m --relative-to="$(dirname "$link")" "$ROOT$to")" "$link" ;;
	esac
done
echo "sysroot: $ROOT"
