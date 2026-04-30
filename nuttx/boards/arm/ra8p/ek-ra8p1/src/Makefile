#
# boards/arm/ra8p/ek-ra8p1/src/Makefile
#

include $(TOPDIR)/Make.defs

CFLAGS += -I$(TOPDIR)/boards/$(CONFIG_ARCH_BOARD)/include

ifeq ($(CONFIG_ARCH_CHIP_RA8P),y)

# Board bringup files
CSRCS = ek-ra8p1_boot.c
ifeq ($(CONFIG_USERLED_LOWER),y)
CSRCS += ek-ra8p1_leds.c
endif
ifeq ($(CONFIG_USERSWITCH),y)
CSRCS += ek-ra8p1_buttons.c
endif
ifeq ($(CONFIG_BOARDCTL),y)
CSRCS += ek-ra8p1_appinit.c
endif
ifeq ($(CONFIG_BOARD_LATE_INITIALIZE),y)
# ra8p_bringup is included in ek-ra8p1_boot.c via board_late_initialize
endif

endif

include $(TOPDIR)/boards/Make.defs