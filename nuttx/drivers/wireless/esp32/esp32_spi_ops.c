/****************************************************************************
 * drivers/wireless/esp32/esp32_spi_ops.c
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.  The
 * ASF licenses this file to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance with the
 * License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.  See the
 * License for the specific language governing permissions and limitations
 * under the License.
 *
 ****************************************************************************/

/****************************************************************************
 * ESP32 SPI Interface Operations
 *
 * This module implements the interface operations for ESP32 SPI interface.
 ****************************************************************************/

#include <nuttx/config.h>

#include <sys/types.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <debug.h>

#include <nuttx/kmalloc.h>
#include <nuttx/spi/spi.h>
#include <nuttx/gpio.h>
#include <nuttx/wireless/esp32_wifi.h>

#include "esp_cfg80211.h"
#include "esp_host_if.h"

#ifdef CONFIG_ESP32_WIFI_SPI

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#ifdef CONFIG_DEBUG_WIRELESS_INFO
#  define spi_info(format, ...)  ninfo(format, ##__VA_ARGS__)
#else
#  define spi_info(format, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_ERROR
#  define spi_err(format, ...)   nerr(format, ##__VA_ARGS__)
#else
#  define spi_err(format, ...)
#endif

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct esp_if_ops g_esp32_spi_ops =
{
  .init = esp32_spi_init,
  .deinit = esp32_spi_deinit,
  .send = esp32_spi_write_packet,
  .receive = esp32_spi_read_packet,
  .data_available = NULL,  /* SPI uses GPIO interrupt */
  .reset = NULL,           /* Hardware reset handled separately */
};

/****************************************************************************
 * Public Data
 ****************************************************************************/

struct esp_if_ops g_esp32_spi_ops;  /* Declared in header for main.c */

/****************************************************************************
 * Public Functions
 ****************************************************************************/

struct esp_if_ops *get_esp32_spi_ops(void)
{
  return &g_esp32_spi_ops;
}

#endif /* CONFIG_ESP32_WIFI_SPI */