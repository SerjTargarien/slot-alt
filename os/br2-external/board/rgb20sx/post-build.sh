#!/bin/sh
# Copies what phase 1 borrows from ROCKNIX (modules, firmware, libmali) into the target,
# from the harvest that os/dev.sh harvest fetched. Blobs never go into git.
set -eu
TARGET="$1"
HARVEST="${BASESQ_HARVEST:-$HOME/basesq-build/harvest}"
if [ ! -d "$HARVEST" ]; then
	echo "post-build: no harvest at $HARVEST, run os/dev.sh harvest" >&2
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
# The frontend keeps a little state off the card, as on BaseOS: slot's card health log
# (/data/slot-card.log) and the card repair's result (/data/slot-repair.result).
mkdir -p "$TARGET/data"
# Buildroot's os-release says "Buildroot".
cat > "$TARGET/usr/lib/os-release" << 'EOF'
NAME="Base²"
ID=basesq
PRETTY_NAME="Base² for the Powkiddy RGB20SX"
HOME_URL="https://github.com/SerjTargarien/slot-alt"
EOF
ln -sf ../usr/lib/os-release "$TARGET/etc/os-release"

if [ -f "$HOME/.ssh/id_ed25519.pub" ]; then
	mkdir -p "$TARGET/root/.ssh"
	cp "$HOME/.ssh/id_ed25519.pub" "$TARGET/root/.ssh/authorized_keys"
	chmod 700 "$TARGET/root/.ssh"
	chmod 600 "$TARGET/root/.ssh/authorized_keys"
fi
