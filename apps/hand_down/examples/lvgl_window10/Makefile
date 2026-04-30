############################################################################
# apps/examples/lvgl_window10/Makefile
############################################################################

include $(APPDIR)/Make.defs

PROGNAME  = lvgl_window10
PRIORITY  = $(CONFIG_EXAMPLES_LVGL_WINDOW10_PRIORITY)
STACKSIZE = $(CONFIG_EXAMPLES_LVGL_WINDOW10_STACKSIZE)
MODULE    = $(CONFIG_EXAMPLES_LVGL_WINDOW10)

CFLAGS += ${INCDIR_PREFIX}$(APPDIR)/graphics/lvgl/lvgl
CFLAGS += ${INCDIR_PREFIX}$(APPDIR)/graphics/lvgl/lvgl/src
CFLAGS += ${INCDIR_PREFIX}$(APPDIR)/examples/lvgl_window10/ui
CFLAGS += -Wno-unused-variable -Wno-shadow

MAINSRC = lvgl_window10.c
CSRCS += ui/ui.c
CSRCS += ui/ui_common.c
CSRCS += ui/ui_events.c
CSRCS += ui/ui_helpers.c
CSRCS += ui/components/ui_comp_hook.c
CSRCS += $(wildcard ui/screens/*.c)
CSRCS += $(wildcard ui/images/*.c)

include $(APPDIR)/Application.mk
