# SPDX-License-Identifier: Apache-2.0

# examples/esp32_wifi_demo/Makefile

CONFIG_EXAMPLES_ESP32_WIFI_DEMO=n
CONFIG_EXAMPLES_ESP32_WIFI_DEMO_PRIORITY=100
CONFIG_EXAMPLES_ESP32_WIFI_DEMO_STACKSIZE=4096
CONFIG_EXAMPLES_ESP32_WIFI_DEMO_NSH_ENTRYPOINT=wifi_demo_main

EXAMPLES_ESP32_WIFI_DEMO = esp32_wifi_demo$(EXEEXT)

ifeq ($(CONFIG_EXAMPLES_ESP32_WIFI_DEMO),y)
  CSRCS = 
  MAINSRC = esp32_wifi_demo.c
  
  # Add dependency on wireless subsystem
  ifeq ($(CONFIG_WIRELESS),y)
    ifeq ($(CONFIG_CFG80211),y)
      CSRCS += esp32_wifi_demo.c
    endif
  endif
endif