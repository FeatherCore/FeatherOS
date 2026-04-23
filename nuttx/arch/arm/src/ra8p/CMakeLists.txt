#
# For a description of the syntax of this configuration file,
# see the file kconfig-language.txt in the NuttX tools repository.
#

if ARCH_CHIP_RA8P

# Add source files for RA8P
CSRCS = ra8p_start.c
CSRCS += ra8p_clockconfig.c
CSRCS += ra8p_lowsetup.c

# UART support
ifeq ($(CONFIG_RA8P_SCI_B_UART0)$(CONFIG_RA8P_SCI_B_UART1)$(CONFIG_RA8P_SCI_B_UART2)$(CONFIG_RA8P_SCI_B_UART3)$(CONFIG_RA8P_SCI_B_UART4),y)
  CSRCS += ra8p_serial.c
endif

endif # ARCH_CHIP_RA8P
