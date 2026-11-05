#!/bin/sh
# Dev loop for slotsq on the RGB20SX, run from WSL while the device runs ROCKNIX.
# The ROCKNIX card stays in slot 1 (mmcblk1), the dev card in slot 2 (mmcblk2).
# See tools/distro/PLAN.md, phase 1.
#
#   sh tools/distro/dev.sh harvest        copy modules, firmware and libmali from ROCKNIX
#   sh tools/distro/dev.sh build          build the root with Buildroot
#   sh tools/distro/dev.sh format --yes   partition and format the dev card (erases it)
#   sh tools/distro/dev.sh install        write the built root to the dev card
#   sh tools/distro/dev.sh bootable       make the dev card boot slotsq on its own from slot 1
#   sh tools/distro/dev.sh boot           boot slotsq once from slot 2 (risky: a kernel hang loops)
#   sh tools/distro/dev.sh restore        make ROCKNIX the default again, from ROCKNIX
#   sh tools/distro/dev.sh log            print the dev card's last slotsq boot log
#   sh tools/distro/dev.sh capture NAME [VAR=x ...]
#                                         on a running slotsq: restart slot with those
#                                         variables, save its screen as frames/NAME.png
#   sh tools/distro/dev.sh shot NAME      on a running slotsq: save the screen as it is
#   sh tools/distro/dev.sh overlay        on a running slotsq: push the OS's own files
#                                         (rootfs overlay, hostname, os-release)
#   sh tools/distro/dev.sh kernel [rocknix-dtb]
#                                         on a running slotsq: install our own kernel and
#                                         boot it next (ROCKNIX's stays as the fallback),
#                                         with our dtb or ROCKNIX's
#   sh tools/distro/dev.sh kernel-back    boot ROCKNIX's kernel again (from slotsq, or
#                                         from ROCKNIX with the dev card in slot 2)
#
# DEVICE (default root@192.168.1.72) is the device while it runs ROCKNIX, SLOTSQ
# (default the same address) while it runs slotsq, with its own known_hosts file.
# BUILD (default ~/slotsq-build) holds Buildroot, its output, the harvest and frames/.

set -eu

DEVICE="${DEVICE:-root@192.168.1.72}"
SLOTSQ="${SLOTSQ:-root@192.168.1.72}"
BUILD="${BUILD:-$HOME/slotsq-build}"
HARVEST="$BUILD/harvest"
FRAMES="$BUILD/frames"
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
EXTERNAL="$REPO/tools/distro/br2-external"
CARD=/dev/mmcblk2
KVER=7.2.11
OVERLAY=/usr/lib/kernel-overlays/base/lib

on_device() {
	ssh "$DEVICE" "$@"
}

on_slotsq() {
	ssh -o BatchMode=yes -o UserKnownHostsFile="$HOME/.ssh/known_hosts_slotsq" "$SLOTSQ" "$@"
}

# Saves slotsq's screen as $FRAMES/NAME.png. With "yes", slot is restarted first with
# the given VAR=value pairs, which the frontend reads from /run/slot.env. slot is found by
# process name (pidof), never by a pattern that could match this very command.
frame() {
	name="$1"
	restart="$2"
	shift 2
	on_slotsq "NAME='$name' RESTART=$restart VARS='$*' sh -s" << 'FRAME'
set -eu
if [ "$RESTART" = yes ]; then
	rm -f /run/slot.env
	for kv in $VARS; do echo "export $kv" >> /run/slot.env; done
	kill -TERM $(pidof ld-linux-aarch64.so.1) 2> /dev/null || true
	sleep 8
fi
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
rm -f /run/slot.env
FRAME
	mkdir -p "$FRAMES"
	on_slotsq "cat '/mnt/sdcard/Logs/frames/$name.png'" > "$FRAMES/$name.png"
	echo "$FRAMES/$name.png"
}

harvest() {
	stage="$BUILD/harvest.tmp"
	rm -rf "$stage" "$HARVEST"
	mkdir -p "$stage"
	on_device "tar -C / -cf - \
		${OVERLAY#/}/modules/$KVER \
		${OVERLAY#/}/firmware/rtw88/rtw8723d_fw.bin \
		${OVERLAY#/}/firmware/regulatory.db ${OVERLAY#/}/firmware/regulatory.db.p7s \
		usr/lib/libmali.so.1.10.0 usr/lib/libmali.so.1 usr/lib/libmali.so \
		usr/lib/libmali-hook.so.1.10.0 usr/lib/libmali-hook.so.1 usr/lib/libmali-hook.so \
		usr/lib/mali/libEGL.so.1 usr/lib/mali/libGLESv2.so.2 usr/lib/mali/libgbm.so.1" |
		tar -C "$stage" -xf -
	# Modules and firmware go under /lib: BusyBox modprobe and the kernel's firmware
	# loader look there, and the root is not merged-/usr.
	mkdir -p "$HARVEST/lib/modules" "$HARVEST/lib/firmware/rtw88" "$HARVEST/usr/lib"
	mv "$stage${OVERLAY}/modules/$KVER" "$HARVEST/lib/modules/"
	mv "$stage${OVERLAY}/firmware/rtw88/rtw8723d_fw.bin" "$HARVEST/lib/firmware/rtw88/"
	mv "$stage${OVERLAY}"/firmware/regulatory.db* "$HARVEST/lib/firmware/"
	mv "$stage"/usr/lib/libmali* "$HARVEST/usr/lib/"
	for lib in libEGL.so.1 libGLESv2.so.2 libgbm.so.1; do
		mv "$stage/usr/lib/mali/$lib" "$HARVEST/usr/lib/"
		ln -sf "$lib" "$HARVEST/usr/lib/${lib%.*}"
	done
	rm -rf "$stage"
	du -sh "$HARVEST"
}

build() {
	cd "$BUILD/buildroot"
	make -s O=../output BR2_EXTERNAL="$EXTERNAL" slotsq_rgb20sx_defconfig
	# WSL puts Windows folders with spaces on PATH, which Buildroot refuses.
	env -i HOME="$HOME" USER="${USER:-root}" TERM=dumb SLOTSQ_HARVEST="$HARVEST" \
		PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
		make O=../output
	ls -la ../output/images/
}

format() {
	[ "${1:-}" = --yes ] || {
		echo "format erases the whole card in slot 2 ($CARD). Run: dev.sh format --yes" >&2
		exit 2
	}
	on_device "CARD=$CARD sh -s" << 'EOF'
set -eu
[ -b "$CARD" ] || { echo "no card in slot 2"; exit 1; }
case "$(readlink -f /sys/block/${CARD#/dev/}/device)" in
*fe2c0000.mmc*) ;;
*) echo "$CARD is not the slot 2 controller, refusing"; exit 1 ;;
esac
grep -q "^${CARD}p[0-9]* /flash " /proc/mounts && { echo "$CARD holds /flash, refusing"; exit 1; }
for m in $(awk -v c="$CARD" 'index($1, c) == 1 {print $2}' /proc/mounts | sort -r); do
	umount "$m" 2>/dev/null || umount -l "$m"
done
# SLOT (FAT32) takes the card up to the last GiB, then two 512 MiB root slots.
end=$(( $(cat /sys/block/${CARD#/dev/}/size) / 2048 - 16 ))
parted -s "$CARD" mklabel msdos \
	mkpart primary fat32 16MiB $((end - 1024))MiB \
	mkpart primary ext4 $((end - 1024))MiB $((end - 512))MiB \
	mkpart primary ext4 $((end - 512))MiB ${end}MiB
sleep 1
for m in $(awk -v c="$CARD" 'index($1, c) == 1 {print $2}' /proc/mounts | sort -r); do
	umount "$m" 2>/dev/null || umount -l "$m"
done
mkfs.vfat -F 32 -n SLOT "${CARD}p1" > /dev/null
mkfs.ext4 -F -q -L SLOTSQ "${CARD}p2"
for m in $(awk -v c="$CARD" 'index($1, c) == 1 {print $2}' /proc/mounts | sort -r); do
	umount "$m" 2>/dev/null || umount -l "$m"
done
parted -s "$CARD" unit MiB print
blkid "${CARD}p1" "${CARD}p2"
EOF
}

install() {
	tarball="$BUILD/output/images/rootfs.tar"
	[ -f "$tarball" ] || { echo "no $tarball, run dev.sh build" >&2; exit 1; }
	on_device "CARD=$CARD sh -s" << 'EOF'
set -eu
root=/tmp/slotsq-root
mkdir -p "$root"
grep -q " $root " /proc/mounts || mount -t ext4 "${CARD}p2" "$root"
find "$root" -mindepth 1 -maxdepth 1 ! -name lost+found -exec rm -rf {} +
EOF
	on_device "tar -C /tmp/slotsq-root -xf -" < "$tarball"
	# The Wi-Fi network is copied from ROCKNIX's iwd on the device itself, so its key
	# never leaves the device and never reaches git or the image.
	on_device "sh -s" << 'EOF'
set -eu
root=/tmp/slotsq-root
psk="$(ls /storage/.cache/iwd/*.psk 2>/dev/null | head -1)"
if [ -n "$psk" ]; then
	ssid="$(basename "$psk" .psk)"
	pass="$(sed -n 's/^Passphrase=//p' "$psk")"
	key="$(sed -n 's/^PreSharedKey=//p' "$psk")"
	umask 077
	{
		echo "ctrl_interface=/var/run/wpa_supplicant"
		echo "update_config=1"
		echo "network={"
		echo "	ssid=\"$ssid\""
		if [ -n "$pass" ]; then echo "	psk=\"$pass\""; else echo "	psk=$key"; fi
		echo "}"
	} > "$root/etc/wpa_supplicant.conf"
	echo "Wi-Fi network $ssid copied"
else
	echo "no saved iwd network, slotsq will have no Wi-Fi"
fi
[ -x "$root/sbin/init" ] || [ -L "$root/sbin/init" ] || { echo "no /sbin/init in the root"; exit 1; }
[ -e "$root/lib/ld-linux-aarch64.so.1" ] || { echo "no dynamic loader in the root"; exit 1; }
du -sh "$root"
sync
umount "$root"
EOF
}

boot() {
	on_device "CARD=$CARD sh -s" << 'EOF'
set -eu
check=/tmp/slotsq-check
mkdir -p "$check"
mount -t ext4 -o ro "${CARD}p2" "$check"
ok=yes
{ [ -x "$check/sbin/init" ] || [ -L "$check/sbin/init" ]; } || ok=no
[ -x "$check/etc/slotsq/boot" ] || ok=no
umount "$check"
[ "$ok" = yes ] || { echo "the dev root is incomplete, not booting it"; exit 1; }
uuid="$(blkid -s PARTUUID -o value "${CARD}p2")"
[ -n "$uuid" ] || { echo "no PARTUUID for ${CARD}p2"; exit 1; }
mount -o remount,rw /flash
conf=/flash/extlinux/extlinux.conf
[ -f "$conf.rocknix" ] || cp "$conf" "$conf.rocknix"
append="$(sed -n 's/^ *APPEND //p' "$conf.rocknix" | head -1 | sed 's/ quiet//')"
{
	echo "DEFAULT slotsq"
	cat "$conf.rocknix"
	echo "LABEL slotsq"
	echo "  LINUX /KERNEL"
	echo "  FDT /device_trees/rk3566-powkiddy-rgb20sx.dtb"
	echo "  APPEND $append rdinit=/nonexistent root=PARTUUID=$uuid rootwait rootfstype=ext4 init=/sbin/init panic=10"
} > "$conf.slotsq"
cp "$conf.slotsq" "$conf"
sync
mount -o remount,ro /flash
echo "booting slotsq once:"
grep APPEND "$conf" | tail -1
(sleep 1; reboot) > /dev/null 2>&1 &
EOF
}

bootable() {
	# Makes the dev card boot on its own from slot 1: ROCKNIX's bootloader (the gap
	# before the first partition, until we build our own U-Boot), our kernel and dtb
	# (dev.sh build), and an extlinux.conf that boots our root. Swapping the two cards
	# then switches systems, and a failed boot can't strand the ROCKNIX card.
	#
	# Before our own kernel, this used ROCKNIX's KERNEL with a /root initrd and a patched
	# copy of its dtb (BRINGUP.md, "The kernel: what made it boot").
	images="$BUILD/output/images"
	for f in Image rk3566-powkiddy-rgb20sx.dtb; do
		[ -f "$images/$f" ] || {
			echo "no $images/$f: run dev.sh build" >&2
			exit 1
		}
	done
	scp -q "$images/Image" "$DEVICE:/tmp/slotsq-Image"
	scp -q "$images/rk3566-powkiddy-rgb20sx.dtb" "$DEVICE:/tmp/slotsq.dtb"
	on_device "CARD=$CARD sh -s" << 'EOF'
set -eu
src=/dev/mmcblk1
grep -q "^${src}p1 /flash " /proc/mounts || { echo "$src is not the running ROCKNIX card"; exit 1; }
[ "$(cat /sys/block/${src#/dev/}/${src#/dev/}p1/start)" -ge 32768 ] || { echo "ROCKNIX's p1 starts too early"; exit 1; }
[ "$(cat /sys/block/${CARD#/dev/}/${CARD#/dev/}p1/start)" -ge 32768 ] || { echo "dev card p1 starts too early"; exit 1; }
# Sectors 64 to 32767: idbloader at 64, u-boot.itb at 16384. Sector 0 (the MBR) stays.
dd if="$src" of="$CARD" bs=512 skip=64 seek=64 count=$((32768 - 64)) conv=notrunc,fsync 2> /dev/null
parted -s "$CARD" set 1 boot on
uuid="$(blkid -s PARTUUID -o value "${CARD}p2")"
slot=/tmp/slotsq-slot
mkdir -p "$slot"
grep -q " $slot " /proc/mounts || mount -t vfat "${CARD}p1" "$slot"
mkdir -p "$slot/extlinux" "$slot/device_trees"
cp /tmp/slotsq-Image "$slot/Image.slotsq"
cp /tmp/slotsq.dtb "$slot/device_trees/slotsq.dtb"
rm -f /tmp/slotsq-Image /tmp/slotsq.dtb
cat > "$slot/extlinux/extlinux.conf" << CONF
LABEL slotsq
  LINUX /Image.slotsq
  FDT /device_trees/slotsq.dtb
  APPEND console=ttyS2,1500000 console=tty0 quiet root=PARTUUID=$uuid rootwait rootfstype=ext4 init=/sbin/init panic=0
CONF
sync
umount "$slot"
a="$(dd if="$src" bs=512 skip=64 count=$((32768 - 64)) 2> /dev/null | md5sum)"
b="$(dd if="$CARD" bs=512 skip=64 count=$((32768 - 64)) 2> /dev/null | md5sum)"
[ "$a" = "$b" ] && echo "bootloader copied" || { echo "bootloader copy differs"; exit 1; }
parted -s "$CARD" unit MiB print | tail -4
echo "the dev card boots slotsq from slot 1"
EOF
}

# Pushes the OS's own files from the last build (the rootfs overlay, plus the generated
# hostname, issue and os-release) to a running slotsq, owned by root. Changes to /etc/inittab
# take effect at the next boot.
overlay() {
	target="$BUILD/output/target"
	files="$(cd "$EXTERNAL/board/rgb20sx/rootfs-overlay" && find . -type f | sed 's|^\./||')"
	tar -C "$target" -cf - $files etc/hostname etc/issue usr/lib/os-release etc/os-release |
		on_slotsq "tar -C / -xf - && chown -R root:root /etc/slotsq /etc/slot-device /etc/inittab /etc/asound.conf /etc/hostname /etc/issue /usr/lib/os-release && echo pushed: $(echo $files | wc -w) overlay files"
}

# Puts our own kernel (output/images/Image), its dtb and its modules onto a running
# slotsq and makes the dev card boot it. ROCKNIX's kernel and its modules stay, and the
# previous extlinux.conf is kept as extlinux.conf.rocknix-kernel for kernel-back.
# With "rocknix-dtb", it boots with the dtb already on the card (ROCKNIX's, with SDR50)
# instead of ours, which differs from it in a few nodes (see BRINGUP.md).
kernel() {
	fdt=/device_trees/slotsq.dtb
	[ "${1:-}" = rocknix-dtb ] && fdt=/device_trees/rk3566-powkiddy-rgb20sx.dtb
	images="$BUILD/output/images"
	modules="$BUILD/output/target/lib/modules"
	release="$(ls "$modules" | grep -- '-slotsq$' | head -1)"
	[ -f "$images/Image" ] && [ -n "$release" ] || {
		echo "no built kernel: run dev.sh build" >&2
		exit 1
	}
	dtb="$images/rk3566-powkiddy-rgb20sx.dtb"
	[ -f "$dtb" ] || {
		echo "no $dtb" >&2
		exit 1
	}
	scp -O -q -o UserKnownHostsFile="$HOME/.ssh/known_hosts_slotsq" "$images/Image" "$SLOTSQ:/mnt/sdcard/Image.slotsq"
	scp -O -q -o UserKnownHostsFile="$HOME/.ssh/known_hosts_slotsq" "$dtb" "$SLOTSQ:/mnt/sdcard/device_trees/slotsq.dtb"
	tar -C "$modules" -cf - "$release" | on_slotsq "tar -C /lib/modules -xf - && chown -R root:root /lib/modules/$release"
	on_slotsq "RELEASE=$release FDT=$fdt sh -s" << 'KERNEL'
set -eu
conf=/mnt/sdcard/extlinux/extlinux.conf
[ -f "$conf.rocknix-kernel" ] || cp "$conf" "$conf.rocknix-kernel"
uuid="$(grep -o 'root=PARTUUID=[^ ]*' "$conf.rocknix-kernel" | head -1)"
cat > "$conf" << CONF
LABEL slotsq
  LINUX /Image.slotsq
  FDT $FDT
  APPEND console=ttyS2,1500000 console=tty0 quiet $uuid rootwait rootfstype=ext4 init=/sbin/init panic=0
CONF
sync
echo "next boot: our kernel $RELEASE"
cat "$conf"
KERNEL
}

# Boots ROCKNIX's kernel again, from the backup kernel() kept. Works on a running slotsq,
# or from ROCKNIX with the dev card in slot 2 (DEVICE), if our kernel didn't boot.
kernel_back() {
	if on_slotsq true 2> /dev/null && on_slotsq "[ -d /mnt/sdcard/extlinux ]"; then
		on_slotsq "cd /mnt/sdcard/extlinux && cp extlinux.conf.rocknix-kernel extlinux.conf && sync && cat extlinux.conf"
		return
	fi
	on_device "CARD=$CARD sh -s" << 'BACK'
set -eu
m=/tmp/slotsq-slot
mkdir -p "$m"
grep -q " $m " /proc/mounts || mount -t vfat "${CARD}p1" "$m"
cd "$m/extlinux" && cp extlinux.conf.rocknix-kernel extlinux.conf && sync && cat extlinux.conf
cd / && umount "$m"
BACK
}

restore() {
	on_device "sh -s" << 'EOF'
set -eu
conf=/flash/extlinux/extlinux.conf
[ -f "$conf.rocknix" ] || { echo "no backup, nothing to restore"; exit 0; }
mount -o remount,rw /flash
cp "$conf.rocknix" "$conf"
sync
mount -o remount,ro /flash
echo "ROCKNIX is the default again"
EOF
}

log() {
	on_device "CARD=$CARD sh -s" << 'EOF'
set -eu
m="$(awk -v p="${CARD}p1" '$1 == p {print $2; exit}' /proc/mounts)"
if [ -z "$m" ]; then
	m=/tmp/slotsq-slot
	mkdir -p "$m"
	mount -t vfat -o ro "${CARD}p1" "$m"
fi
cat "$m/slotsq-boot.log" 2>/dev/null || echo "no slotsq-boot.log on the dev card yet"
EOF
}

cmd="${1:-}"
[ $# -gt 0 ] && shift
case "$cmd" in
harvest | build | install | bootable | boot | restore | log) "$cmd" ;;
kernel) kernel "$@" ;;
overlay) overlay ;;
kernel-back) kernel_back ;;
capture)
	name="$1"
	shift
	frame "$name" yes "$@"
	;;
shot) frame "$1" no ;;
format) format "$@" ;;
*)
	sed -n '2,29p' "$0" | sed 's/^# \{0,1\}//'
	exit 2
	;;
esac
