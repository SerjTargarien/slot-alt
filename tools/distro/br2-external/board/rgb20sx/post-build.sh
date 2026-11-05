#!/bin/sh
# Copies what phase 1 borrows from ROCKNIX (modules, firmware, libmali) into the target,
# from the harvest that tools/distro/harvest.sh fetched. Blobs never go into git.
set -eu
TARGET="$1"
HARVEST="${RGBSLOT_HARVEST:-$HOME/rgbslot-build/harvest}"
if [ ! -d "$HARVEST" ]; then
	echo "post-build: no harvest at $HARVEST, run tools/distro/harvest.sh" >&2
	exit 1
fi
cp -a "$HARVEST/." "$TARGET/"
# Buildroot links /etc/dropbear to /var/run/dropbear, which doesn't exist after boot, so
# Dropbear couldn't create its host key and dropped every connection. A real folder on
# the root also keeps the host key the same across boots.
if [ -L "$TARGET/etc/dropbear" ]; then
	rm "$TARGET/etc/dropbear"
	mkdir -p "$TARGET/etc/dropbear"
	chmod 700 "$TARGET/etc/dropbear"
fi
if [ -f "$HOME/.ssh/id_ed25519.pub" ]; then
	mkdir -p "$TARGET/root/.ssh"
	cp "$HOME/.ssh/id_ed25519.pub" "$TARGET/root/.ssh/authorized_keys"
	chmod 700 "$TARGET/root/.ssh"
	chmod 600 "$TARGET/root/.ssh/authorized_keys"
fi
