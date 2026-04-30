#
# boards/arm/ra8p/ek-ra8p1/src/CMakeLists.txt
#

if(CONFIG_ARCH_CHIP_RA8P)
  set(BRINGUP_SRCS ek-ra8p1_boot.c)

  if(CONFIG_USERLED_LOWER)
    list(APPEND BRINGUP_SRCS ek-ra8p1_leds.c)
  endif()

  if(CONFIG_USERSWITCH)
    list(APPEND BRINGUP_SRCS ek-ra8p1_buttons.c)
  endif()

  if(CONFIG_BOARDCTL)
    list(APPEND BRINGUP_SRCS ek-ra8p1_appinit.c)
  endif()

  target_sources(boards PRIVATE ${BRINGUP_SRCS})
endif()