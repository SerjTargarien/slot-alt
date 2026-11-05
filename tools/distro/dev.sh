#!/bin/sh
# Dev loop for rgbslot on the RGB20SX, run from WSL while the device runs ROCKNIX.
# The ROCKNIX card stays in slot 1 (mmcblk1), the dev card in slot 2 (mmcblk2).
# See tools/distro/PLAN.md, phase 1.
#
#   sh tools/distro/dev.sh harvest        copy modules, firmware and libmali from ROCKNIX
#   sh tools/distro/dev.sh build          build the root with Buildroot
#   sh tools/distro/dev.sh format --yes   partition and format the dev card (erases it)
#   sh tools/distro/dev.sh install        write the built root to the dev card
#   sh tools/distro/dev.sh bootable       make the dev card boot rgbslot on its own from slot 1
#   sh tools/distro/dev.sh boot           boot rgbslot once from slot 2 (risky: a kernel hang loops)
#   sh tools/distro/dev.sh restore        make ROCKNIX the default again, from ROCKNIX
#   sh tools/distro/dev.sh log            print the dev card's last rgbslot boot log
#   sh tools/distro/dev.sh capture NAME [VAR=x ...]
#                                         on a running rgbslot: restart slot with those
#                                         variables, save its screen as frames/NAME.png
#   sh tools/distro/dev.sh shot NAME      on a running rgbslot: save the screen as it is
#   sh tools/distro/dev.sh kernel [rocknix-dtb]
#                                         on a running rgbslot: install our own kernel and
#                                         boot it next (ROCKNIX's stays as the fallback),
#                                         with our dtb or ROCKNIX's
#   sh tools/distro/dev.sh kernel-back    boot ROCKNIX's kernel again (from rgbslot, or
#                                         from ROCKNIX with the dev card in slot 2)
#
# DEVICE (default root@192.168.1.72) is the device while it runs ROCKNIX, RGBSLOT
# (default the same address) while it runs rgbslot, with its own known_hosts file.
# BUILD (default ~/rgbslot-build) holds Buildroot, its output, the harvest and frames/.

set -eu

DEVICE="${DEVICE:-root@192.168.1.72}"
RGBSLOT="${RGBSLOT:-root@192.168.1.72}"
BUILD="${BUILD:-$HOME/rgbslot-build}"
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

on_rgbslot() {
	ssh -o BatchMode=yes -o UserKnownHostsFile="$HOME/.ssh/known_hosts_rgbslot" "$RGBSLOT" "$@"
}

# Saves rgbslot's screen as $FRAMES/NAME.png. With "yes", slot is restarted first with
# the given VAR=value pairs, which the frontend reads from /run/slot.env. slot is found by
# process name (pidof), never by a pattern that could match this very command.
frame() {
	name="$1"
	restart="$2"
	shift 2
	on_rgbslot "NAME='$name' RESTART=$restart VARS='$*' sh -s" << 'FRAME'
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
	on_rgbslot "cat '/mnt/sdcard/Logs/frames/$name.png'" > "$FRAMES/$name.png"
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
	make -s O=../output BR2_EXTERNAL="$EXTERNAL" rgbslot_rgb20sx_defconfig
	# WSL puts Windows folders with spaces on PATH, which Buildroot refuses.
	env -i HOME="$HOME" USER="${USER:-root}" TERM=dumb RGBSLOT_HARVEST="$HARVEST" \
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
mkfs.ext4 -F -q -L RGBSLOT "${CARD}p2"
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
root=/tmp/rgbslot-root
mkdir -p "$root"
grep -q " $root " /proc/mounts || mount -t ext4 "${CARD}p2" "$root"
find "$root" -mindepth 1 -maxdepth 1 ! -name lost+found -exec rm -rf {} +
EOF
	on_device "tar -C /tmp/rgbslot-root -xf -" < "$tarball"
	# The Wi-Fi network is copied from ROCKNIX's iwd on the device itself, so its key
	# never leaves the device and never reaches git or the image.
	on_device "sh -s" << 'EOF'
set -eu
root=/tmp/rgbslot-root
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
	echo "no saved iwd network, rgbslot will have no Wi-Fi"
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
check=/tmp/rgbslot-check
mkdir -p "$check"
mount -t ext4 -o ro "${CARD}p2" "$check"
ok=yes
{ [ -x "$check/sbin/init" ] || [ -L "$check/sbin/init" ]; } || ok=no
[ -x "$check/etc/rgbslot/boot" ] || ok=no
umount "$check"
[ "$ok" = yes ] || { echo "the dev root is incomplete, not booting it"; exit 1; }
uuid="$(blkid -s PARTUUID -o value "${CARD}p2")"
[ -n "$uuid" ] || { echo "no PARTUUID for ${CARD}p2"; exit 1; }
mount -o remount,rw /flash
conf=/flash/extlinux/extlinux.conf
[ -f "$conf.rocknix" ] || cp "$conf" "$conf.rocknix"
append="$(sed -n 's/^ *APPEND //p' "$conf.rocknix" | head -1 | sed 's/ quiet//')"
{
	echo "DEFAULT rgbslot"
	cat "$conf.rocknix"
	echo "LABEL rgbslot"
	echo "  LINUX /KERNEL"
	echo "  FDT /device_trees/rk3566-powkiddy-rgb20sx.dtb"
	echo "  APPEND $append rdinit=/nonexistent root=PARTUUID=$uuid rootwait rootfstype=ext4 init=/sbin/init panic=10"
} > "$conf.rgbslot"
cp "$conf.rgbslot" "$conf"
sync
mount -o remount,ro /flash
echo "booting rgbslot once:"
grep APPEND "$conf" | tail -1
(sleep 1; reboot) > /dev/null 2>&1 &
EOF
}

bootable() {
	# Makes the dev card boot on its own from slot 1: ROCKNIX's bootloader (the gap
	# before the first partition), its kernel and dtb, and an extlinux.conf that
	# boots our root. Swapping the two cards then switches systems, and a failed boot
	# can't strand the ROCKNIX card.
	#
	# The dtb is ROCKNIX's with slot 1 (mmc@fe2b0000) on SDR50 (100 MHz) instead of
	# SDR104: the dev card failed SDR104 tuning there ("mmc1: tuning execution failed:
	# -5") and the kernel then never found the root. SDR50 tunes fine.
	fdt="$BUILD/output/host/bin"
	[ -x "$fdt/fdtput" ] || { echo "no fdtput: run make host-dtc in $BUILD/buildroot" >&2; exit 1; }
	dtb="$BUILD/dtb"
	mkdir -p "$dtb"
	scp -q "$DEVICE:/flash/device_trees/rk3566-powkiddy-rgb20sx.dtb" "$dtb/rocknix.dtb"
	cp "$dtb/rocknix.dtb" "$dtb/rgbslot.dtb"
	"$fdt/fdtput" -d "$dtb/rgbslot.dtb" /mmc@fe2b0000 sd-uhs-sdr104
	"$fdt/fdtput" "$dtb/rgbslot.dtb" /mmc@fe2b0000 sd-uhs-sdr50
	if "$fdt/fdtget" -p "$dtb/rgbslot.dtb" /mmc@fe2b0000 | grep -q sd-uhs-sdr104; then
		echo "slot 1 still has SDR104 in the dtb" >&2
		exit 1
	fi
	scp -q "$dtb/rgbslot.dtb" "$DEVICE:/tmp/rgbslot.dtb"
	# ROCKNIX's kernel has its initramfs built in, and that initramfs has no /root, the
	# fixed directory where Linux mounts root= when there's no rdinit to run. So the
	# mount failed with ENOENT and the kernel panicked. U-Boot passes this tiny extra
	# cpio as the initrd, which the kernel unpacks on top, adding the missing /root.
	cpiodir="$dtb/rootdir"
	rm -rf "$cpiodir"
	mkdir -p "$cpiodir/root"
	(cd "$cpiodir" && echo root | cpio -o -H newc --quiet) > "$dtb/rootdir.cpio"
	scp -q "$dtb/rootdir.cpio" "$DEVICE:/tmp/rootdir.cpio"
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
slot=/tmp/rgbslot-slot
mkdir -p "$slot"
grep -q " $slot " /proc/mounts || mount -t vfat "${CARD}p1" "$slot"
mkdir -p "$slot/extlinux" "$slot/device_trees"
cp /flash/KERNEL "$slot/KERNEL"
cp /tmp/rgbslot.dtb "$slot/device_trees/rk3566-powkiddy-rgb20sx.dtb"
cp /tmp/rootdir.cpio "$slot/rootdir.cpio"
cat > "$slot/extlinux/extlinux.conf" << CONF
LABEL rgbslot
  LINUX /KERNEL
  FDT /device_trees/rk3566-powkiddy-rgb20sx.dtb
  INITRD /rootdir.cpio
  APPEND console=ttyS2,1500000 console=tty0 quiet rdinit=/nonexistent root=PARTUUID=$uuid rootwait rootfstype=ext4 init=/sbin/init panic=0
CONF
sync
umount "$slot"
a="$(dd if="$src" bs=512 skip=64 count=$((32768 - 64)) 2> /dev/null | md5sum)"
b="$(dd if="$CARD" bs=512 skip=64 count=$((32768 - 64)) 2> /dev/null | md5sum)"
[ "$a" = "$b" ] && echo "bootloader copied" || { echo "bootloader copy differs"; exit 1; }
parted -s "$CARD" unit MiB print | tail -4
echo "the dev card boots rgbslot from slot 1"
EOF
}

# Puts our own kernel (output/images/Image), its dtb and its modules onto a running
# rgbslot and makes the dev card boot it. ROCKNIX's kernel and its modules stay, and the
# previous extlinux.conf is kept as extlinux.conf.rocknix-kernel for kernel-back.
# With "rocknix-dtb", it boots with the dtb already on the card (ROCKNIX's, with SDR50)
# instead of ours, which differs from it in a few nodes (see BRINGUP.md).
kernel() {
	fdt=/device_trees/rgbslot.dtb
	[ "${1:-}" = rocknix-dtb ] && fdt=/device_trees/rk3566-powkiddy-rgb20sx.dtb
	images="$BUILD/output/images"
	modules="$BUILD/output/target/lib/modules"
	release="$(ls "$modules" | grep -- '-rgbslot$' | head -1)"
	[ -f "$images/Image" ] && [ -n "$release" ] || {
		echo "no built kernel: run dev.sh build" >&2
		exit 1
	}
	dtb="$images/rk3566-powkiddy-rgb20sx.dtb"
	[ -f "$dtb" ] || {
		echo "no $dtb" >&2
		exit 1
	}
	scp -O -q -o UserKnownHostsFile="$HOME/.ssh/known_hosts_rgbslot" "$images/Image" "$RGBSLOT:/mnt/sdcard/Image.rgbslot"
	scp -O -q -o UserKnownHostsFile="$HOME/.ssh/known_hosts_rgbslot" "$dtb" "$RGBSLOT:/mnt/sdcard/device_trees/rgbslot.dtb"
	tar -C "$modules" -cf - "$release" | on_rgbslot "tar -C /lib/modules -xf - && chown -R root:root /lib/modules/$release"
	on_rgbslot "RELEASE=$release FDT=$fdt sh -s" << 'KERNEL'
set -eu
conf=/mnt/sdcard/extlinux/extlinux.conf
[ -f "$conf.rocknix-kernel" ] || cp "$conf" "$conf.rocknix-kernel"
uuid="$(grep -o 'root=PARTUUID=[^ ]*' "$conf.rocknix-kernel" | head -1)"
cat > "$conf" << CONF
LABEL rgbslot
  LINUX /Image.rgbslot
  FDT $FDT
  APPEND console=ttyS2,1500000 console=tty0 quiet $uuid rootwait rootfstype=ext4 init=/sbin/init panic=0
CONF
sync
echo "next boot: our kernel $RELEASE"
cat "$conf"
KERNEL
}

# Boots ROCKNIX's kernel again, from the backup kernel() kept. Works on a running rgbslot,
# or from ROCKNIX with the dev card in slot 2 (DEVICE), if our kernel didn't boot.
kernel_back() {
	if on_rgbslot true 2> /dev/null && on_rgbslot "[ -d /mnt/sdcard/extlinux ]"; then
		on_rgbslot "cd /mnt/sdcard/extlinux && cp extlinux.conf.rocknix-kernel extlinux.conf && sync && cat extlinux.conf"
		return
	fi
	on_device "CARD=$CARD sh -s" << 'BACK'
set -eu
m=/tmp/rgbslot-slot
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
	m=/tmp/rgbslot-slot
	mkdir -p "$m"
	mount -t vfat -o ro "${CARD}p1" "$m"
fi
cat "$m/rgbslot-boot.log" 2>/dev/null || echo "no rgbslot-boot.log on the dev card yet"
EOF
}

cmd="${1:-}"
[ $# -gt 0 ] && shift
case "$cmd" in
harvest | build | install | bootable | boot | restore | log) "$cmd" ;;
kernel) kernel "$@" ;;
kernel-back) kernel_back ;;
capture)
	name="$1"
	shift
	frame "$name" yes "$@"
	;;
shot) frame "$1" no ;;
format) format "$@" ;;
*)
	sed -n '2,27p' "$0" | sed 's/^# \{0,1\}//'
	exit 2
	;;
esac
