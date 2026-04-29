############################################################################
# examples/esp32_wifi_test/Makefile
#
# SPDX-License-Identifier: Apache-2.0
#
# Copyright 2026 Espressif Systems
#
############################################################################

CONFIG_EXAMPLES_ESP32_WIFI_TEST = y

ifeq ($(CONFIG_EXAMPLES_ESP32_WIFI_TEST),y)

ifeq ($(CONFIG_CFGLINK),y)
  LDREQUIRES += cfg80211 nl80211 netlink
endif

ifeq ($(CONFIG_WIRELESS),y)
  ifeq ($(CONFIG_CFG80211),y)
    CSRCS += esp32_wifi_test.c
  endif
endif

endif