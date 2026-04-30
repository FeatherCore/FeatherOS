############################################################################
# boards/sim/sim/sim/src/Makefile
#
# SPDX-License-Identifier: Apache-2.0
#
# Licensed to the Apache Software Foundation (ASF) under one or more
# contributor license agreements.  See the NOTICE file distributed with
# this work for additional information regarding copyright ownership.  The
# ASF licenses this file to you under the Apache License, Version 2.0 (the
# "License"); you may not use this file except in compliance with the
# License.  You may obtain a copy of the License at
#
#   http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
# WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.  See the
# License for the specific language governing permissions and limitations
# under the License.
#
############################################################################

include $(TOPDIR)/Make.defs

# Add dummy.c to ensure that we have at least one object.
# On some platforms like macOS, we can't create an empty archive.
CSRCS = dummy.c

ifeq ($(CONFIG_BOARD_LATE_INITIALIZE),y)
  CSRCS += sim_boot.c
endif

ifeq ($(CONFIG_BOARDCTL),y)
  CSRCS += sim_appinit.c
endif

ifeq ($(CONFIG_BOARD_LATE_INITIALIZE),y)
  CSRCS += sim_bringup.c
ifeq ($(CONFIG_LIBC_ZONEINFO_ROMFS),y)
  CSRCS += sim_zoneinfo.c
endif
else ifeq ($(CONFIG_BOARDCTL),y)
  CSRCS += sim_bringup.c
ifeq ($(CONFIG_LIBC_ZONEINFO_ROMFS),y)
  CSRCS += sim_zoneinfo.c
endif
endif

ifeq ($(CONFIG_EXAMPLES_GPIO),y)
ifeq ($(CONFIG_GPIO_LOWER_HALF),y)
  CSRCS += sim_ioexpander.c
else
  CSRCS += sim_gpio.c
endif
endif

ifeq ($(CONFIG_ETC_ROMFS),y)
  RCSRCS = etc/init.d/rc.sysinit etc/init.d/rcS
  RCRAWS = etc/group etc/passwd \
           etc/wing/wing_manifest.bin \
           etc/wing/wing_payload.bin \
           etc/wing/resource/images/shell/wallpaper_aurora_landscape.png \
           etc/wing/resource/images/shell/wallpaper_aurora_portrait.png \
           etc/wing/resource/images/shell/wallpaper_dusk_landscape.png \
	           etc/wing/resource/images/shell/wallpaper_dusk_portrait.png \
	           etc/wing/resource/fonts/simhei.ttf \
	           etc/wing/resource/fonts/shell_workset.txt \
	           etc/wing/resource/icons/shell/airplane.svg \
	           etc/wing/resource/icons/shell/alert.svg \
	           etc/wing/resource/icons/shell/bluetooth.svg \
	           etc/wing/resource/icons/shell/camera.svg \
	           etc/wing/resource/icons/shell/chat.svg \
	           etc/wing/resource/icons/shell/check.svg \
	           etc/wing/resource/icons/shell/close.svg \
	           etc/wing/resource/icons/shell/cloud.svg \
	           etc/wing/resource/icons/shell/flashlight.svg \
	           etc/wing/resource/icons/shell/folder.svg \
	           etc/wing/resource/icons/shell/mail.svg \
	           etc/wing/resource/icons/shell/moon.svg \
	           etc/wing/resource/icons/shell/more.svg \
	           etc/wing/resource/icons/shell/music.svg \
	           etc/wing/resource/icons/shell/phone.svg \
	           etc/wing/resource/icons/shell/play.svg \
	           etc/wing/resource/icons/shell/settings.svg \
	           etc/wing/resource/icons/shell/sync.svg \
	           etc/wing/resource/icons/shell/wifi.svg \
	           etc/wing/resource/icons/shell/wing.svg \
	           etc/wing/resource/windows10_mobile/LICENSE \
	           etc/wing/resource/windows10_mobile/assets/airplane_ic.png \
	           etc/wing/resource/windows10_mobile/assets/battery_ic.png \
	           etc/wing/resource/windows10_mobile/assets/bluetooth_ic.png \
	           etc/wing/resource/windows10_mobile/assets/brightness_ic.png \
	           etc/wing/resource/windows10_mobile/assets/camera_ic.png \
	           etc/wing/resource/windows10_mobile/assets/cellular_ic.png \
	           etc/wing/resource/windows10_mobile/assets/edge_ic.png \
	           etc/wing/resource/windows10_mobile/assets/embedded_tile.png \
	           etc/wing/resource/windows10_mobile/assets/file_ic.png \
	           etc/wing/resource/windows10_mobile/assets/groove_ic.png \
	           etc/wing/resource/windows10_mobile/assets/hotspot_ic.png \
	           etc/wing/resource/windows10_mobile/assets/img0.png \
	           etc/wing/resource/windows10_mobile/assets/img1.png \
	           etc/wing/resource/windows10_mobile/assets/img2.png \
	           etc/wing/resource/windows10_mobile/assets/img3.png \
	           etc/wing/resource/windows10_mobile/assets/img4.png \
	           etc/wing/resource/windows10_mobile/assets/img5.png \
	           etc/wing/resource/windows10_mobile/assets/img6.png \
	           etc/wing/resource/windows10_mobile/assets/img7.png \
	           etc/wing/resource/windows10_mobile/assets/img8.png \
	           etc/wing/resource/windows10_mobile/assets/img9.png \
	           etc/wing/resource/windows10_mobile/assets/location_ic.png \
	           etc/wing/resource/windows10_mobile/assets/message_ic.png \
	           etc/wing/resource/windows10_mobile/assets/microsoft_ic.png \
	           etc/wing/resource/windows10_mobile/assets/news_ic.png \
	           etc/wing/resource/windows10_mobile/assets/news_image.png \
	           etc/wing/resource/windows10_mobile/assets/news_tile.png \
	           etc/wing/resource/windows10_mobile/assets/outlook_ic.png \
	           etc/wing/resource/windows10_mobile/assets/people_ic.png \
	           etc/wing/resource/windows10_mobile/assets/phone_ic.png \
	           etc/wing/resource/windows10_mobile/assets/photo_tile.png \
	           etc/wing/resource/windows10_mobile/assets/photos_ic.png \
	           etc/wing/resource/windows10_mobile/assets/settings_ic.png \
	           etc/wing/resource/windows10_mobile/assets/sky_bg.png \
	           etc/wing/resource/windows10_mobile/assets/tips_ic.png \
	           etc/wing/resource/windows10_mobile/assets/vpn_ic.png \
	           etc/wing/resource/windows10_mobile/assets/weather_ic.png \
	           etc/wing/resource/windows10_mobile/assets/wifi_ic.png \
	           etc/wing/resource/windows10_mobile/assets/windows_logo.png \
	           etc/wing/resource/windows10_mobile/assets/wp_back.png \
	           etc/wing/resource/windows10_mobile/assets/wp_logo.png \
	           etc/wing/resource/windows10_mobile/assets/wp_next.png \
	           etc/wing/resource/windows10_mobile/assets/wp_search.png \
	           etc/wing/resource/windows10_mobile/assets/wp_settings.png \
	           etc/wing/resource/windows10_mobile/assets/wp_system.png \
	           etc/wing/resource/windows10_mobile/raw/padlock.fraw \
	           etc/wing/resource/windows10_mobile/raw/settings_back.fraw \
	           etc/wing/resource/windows10_mobile/raw/stars_ic.fraw \
	           etc/wing/resource/windows10_mobile/raw/wp_about.fraw \
	           etc/wing/resource/windows10_mobile/raw/wp_account.fraw \
	           etc/wing/resource/windows10_mobile/raw/wp_apps.fraw \
	           etc/wing/resource/windows10_mobile/raw/wp_devices.fraw \
	           etc/wing/resource/windows10_mobile/raw/wp_network.fraw \
	           etc/wing/resource/windows10_mobile/raw/wp_personalization.fraw \
	           etc/wing/resource/windows10_mobile/raw/wp_privacy.fraw \
	           etc/wing/resource/windows10_mobile/raw/wp_time.fraw \
	           etc/wing/resource/windows10_mobile/raw/wifi_icon.fraw
endif

ifeq ($(CONFIG_ARCH_BUTTONS),y)
  CSRCS += sim_buttons.c
endif

ifeq ($(CONFIG_MOTOR_FOC_DUMMY),y)
CSRCS += sim_foc.c
endif

ifeq ($(CONFIG_USBDEV_COMPOSITE),y)
CSRCS += sim_composite.c
endif

include $(TOPDIR)/boards/Board.mk
