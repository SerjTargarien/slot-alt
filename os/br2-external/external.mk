include $(sort $(wildcard $(BR2_EXTERNAL_BASESQ_PATH)/package/*/*.mk))

# The kernel patches come from ROCKNIX (board/rgb20sx/linux/README.md), which applies them
# with plain `patch`, allowing the default fuzz. Buildroot's apply-patches.sh uses -F0, and
# some of them only apply to 7.2.11 with fuzz (the 1992 MHz OPP one, for instance). So they
# are applied here as ROCKNIX applies them, in its order, and any reject stops the build.
BASESQ_LINUX_PATCH_SETS = 1-mainline 2-mainline-rockchip 3-7.2.6 4-rk3566 5-basesq
BASESQ_LINUX_PATCHES = $(BR2_EXTERNAL_BASESQ_PATH)/board/rgb20sx/linux/patches

define BASESQ_APPLY_LINUX_PATCHES
	set -e; \
	for set in $(BASESQ_LINUX_PATCH_SETS); do \
		for p in $(BASESQ_LINUX_PATCHES)/$$set/*.patch; do \
			echo "Applying $$set/$$(basename $$p) (as ROCKNIX does)"; \
			patch -d $(LINUX_DIR) -p1 -F2 --no-backup-if-mismatch -N < $$p; \
		done; \
	done; \
	if find $(LINUX_DIR) -name '*.rej' | grep -q .; then \
		echo "kernel patches left rejects" >&2; \
		exit 1; \
	fi
endef
LINUX_POST_PATCH_HOOKS += BASESQ_APPLY_LINUX_PATCHES
