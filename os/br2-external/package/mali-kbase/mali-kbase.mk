################################################################################
#
# mali-kbase
#
################################################################################

# Arm's Mali Bifrost kbase driver (r54p2), from ROCKNIX's mirror at the commit
# ROCKNIX 20261101 pins (packages/linux-drivers/mali-bifrost/package.mk).
MALI_KBASE_VERSION = 39da994bb6fc8819e5e8c1873907dd21d17e53c1
MALI_KBASE_SITE = $(call github,rocknix,mali_kbase,$(MALI_KBASE_VERSION))
MALI_KBASE_LICENSE = GPL-2.0
MALI_KBASE_DEPENDENCIES = linux

MALI_KBASE_DRIVER = $(@D)/product/kernel/drivers/gpu/arm/midgard

# The driver's own Makefile adds the CONFIG_MALI_* defines that Kbuild needs, so build
# through it, as ROCKNIX does, rather than with the kernel-module infrastructure.
define MALI_KBASE_BUILD_CMDS
	$(TARGET_MAKE_ENV) $(MAKE) $(LINUX_MAKE_FLAGS) -C $(MALI_KBASE_DRIVER) \
		KDIR=$(LINUX_DIR) \
		CONFIG_MALI_MIDGARD=m \
		CONFIG_MALI_PLATFORM_NAME=devicetree \
		CONFIG_MALI_REAL_HW=y
endef

define MALI_KBASE_INSTALL_TARGET_CMDS
	$(INSTALL) -D -m 0644 $(MALI_KBASE_DRIVER)/mali_kbase.ko \
		$(TARGET_DIR)/lib/modules/$(LINUX_VERSION_PROBED)/mali-bifrost/mali_kbase.ko
endef

$(eval $(generic-package))
