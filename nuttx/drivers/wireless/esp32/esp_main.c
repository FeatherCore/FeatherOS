/****************************************************************************
 * drivers/wireless/esp32/esp_main.c
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
 * ESP32 WiFi Driver Main Entry Point
 *
 * This module implements the main entry point for the ESP32 WiFi driver
 * based on the cfg80211/nl80211 framework.
 ****************************************************************************/

#include <nuttx/config.h>

#include <sys/types.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>
#include <semaphore.h>
#include <errno.h>
#include <debug.h>

#include <nuttx/kmalloc.h>
#include <nuttx/semaphore.h>
#include <nuttx/wqueue.h>
#include <nuttx/wireless/cfg80211.h>

#ifdef CONFIG_DRIVERS_ESP32_WIFI

#include "esp_cfg80211.h"
#include "esp_host_if.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Debug output */

#ifdef CONFIG_DEBUG_WIRELESS_INFO
#  define esp_main_info(fmt, ...)  ninfo(fmt, ##__VA_ARGS__)
#else
#  define esp_main_info(fmt, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_ERROR
#  define esp_main_err(fmt, ...)   nerr(fmt, ##__VA_ARGS__)
#else
#  define esp_main_err(fmt, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_VERBOSE
#  define esp_main_verbose(fmt, ...) ninfo(fmt, ##__VA_ARGS__)
#else
#  define esp_main_verbose(fmt, ...)
#endif

#define esp_main_dbg esp_main_verbose

/* Interface names */

#define ESP_STA_IFNAME              "wlan%d"
#define ESP_AP_IFNAME               "ap%d"

/* Default interface settings */

#define ESP_DEFAULT_STA_IFTYPE      NL80211_IFTYPE_STATION
#define ESP_DEFAULT_AP_IFTYPE       NL80211_IFTYPE_AP

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct esp_adapter g_esp_adapter;
static struct wiphy *g_esp_wiphy;

/* Interface type mapping */

static const uint8_t g_esp_iftype_map[] =
{
  [ESP_STA_IFTYPE] = ESP_STA_IF,
  [ESP_AP_IFTYPE] = ESP_AP_IF,
};

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp_if_ops_sdio
 *
 * Description:
 *   SDIO interface operations for ESP32.
 *
 ****************************************************************************/

#ifdef CONFIG_ESP32_WIFI_SDIO
static struct esp_if_ops g_esp_sdio_ops =
{
  .init = esp_sdio_init,
  .deinit = esp_sdio_deinit,
  .send = esp_sdio_write_packet,
  .receive = esp_sdio_read_packet,
  .data_available = NULL, /* TODO: Implement */
  .reset = NULL, /* TODO: Implement */
};
#endif

/****************************************************************************
 * Name: esp_if_ops_spi
 *
 * Description:
 *   SPI interface operations for ESP32.
 *
 ****************************************************************************/

#ifdef CONFIG_ESP32_WIFI_SPI
static struct esp_if_ops g_esp_spi_ops =
{
  .init = esp_spi_init,
  .deinit = esp_spi_deinit,
  .send = esp_spi_write_packet,
  .receive = esp_spi_read_packet,
  .data_available = NULL, /* TODO: Implement */
  .reset = NULL, /* TODO: Implement */
};
#endif

/****************************************************************************
 * Name: esp_get_interface_ops
 *
 * Description:
 *   Get interface operations based on configuration.
 *
 ****************************************************************************/

static FAR struct esp_if_ops *esp_get_interface_ops(void)
{
#ifdef CONFIG_ESP32_WIFI_SDIO
  return &g_esp_sdio_ops;
#elif defined(CONFIG_ESP32_WIFI_SPI)
  return &g_esp_spi_ops;
#else
  return NULL;
#endif
}

/****************************************************************************
 * Name: esp_register_interfaces
 *
 * Description:
 *   Register default interfaces with cfg80211.
 *
 ****************************************************************************/

static int esp_register_interfaces(FAR struct esp_adapter *adapter)
{
  FAR struct wireless_dev *wdev;
  int ret;

  /* Add default station interface */

  wdev = esp_cfg80211_add_iface(adapter->wiphy, ESP_STA_IFNAME,
                               ESP_DEFAULT_STA_IFTYPE, NULL);
  if (!wdev)
    {
      esp_main_err("Failed to add station interface\n");
      return -EIO;
    }

  esp_main_info("Added station interface: %s\n", wdev->netdev->name);

  /* Optionally add AP interface if enabled */

#ifdef CONFIG_ESP32_AP_MODE
  wdev = esp_cfg80211_add_iface(adapter->wiphy, ESP_AP_IFNAME,
                               ESP_DEFAULT_AP_IFTYPE, NULL);
  if (!wdev)
    {
      esp_main_err("Failed to add AP interface\n");
      /* Continue anyway, station mode is sufficient */
    }
  else
    {
      esp_main_info("Added AP interface: %s\n", wdev->netdev->name);
    }
#endif

  return OK;
}

/****************************************************************************
 * Name: esp_wiphy_setup
 *
 * Description:
 *   Set up wiphy capabilities and features.
 *
 ****************************************************************************/

static void esp_wiphy_setup(FAR struct wiphy *wiphy)
{
  /* Set interface combinations */

  static struct ieee80211_iface_limit g_esp_limits[] =
  {
    {
      .max = 1,
      .types = BIT(NL80211_IFTYPE_STATION),
    },
#ifdef CONFIG_ESP32_AP_MODE
    {
      .max = 1,
      .types = BIT(NL80211_IFTYPE_AP),
    },
#endif
  };

  static struct ieee80211_iface_combination g_esp_combinations[] =
  {
    {
      .limits = g_esp_limits,
      .n_limits = ARRAY_SIZE(g_esp_limits),
      .max_interfaces = 2,
      .num_different_channels = 1,
    },
  };

  /* Set interface combinations */

  wiphy->iface_combinations = g_esp_combinations;
  wiphy->n_iface_combinations = ARRAY_SIZE(g_esp_combinations);

  /* Set supported bands */

  wiphy->bands[IEEE80211_BAND_2GHZ] = &g_esp_wifi_bands_2ghz;

  /* Set capabilities */

  wiphy->interface_modes = BIT(NL80211_IFTYPE_STATION);
#ifdef CONFIG_ESP32_AP_MODE
  wiphy->interface_modes |= BIT(NL80211_IFTYPE_AP);
#endif

  /* Set max scan SSIDs */

  wiphy->max_scan_ssids = 4;
  wiphy->max_sched_scan_ssids = 16;
  wiphy->max_match_sets = 16;
  wiphy->max_scan_ie_len = 2048;

  /* Set cipher suites */

  wiphy->cipher_suites = g_esp_cipher_suites;
  wiphy->n_cipher_suites = ARRAY_SIZE(g_esp_cipher_suites);

  /* Set signal type */

  wiphy->signal_type = CFG80211_SIGNAL_TYPE_MBM;

  /* Set features */

  wiphy->flags |= WIPHY_FLAG_HAS_REMAIN_ON_CHANNEL |
                  WIPHY_FLAG_OFFCHAN_TX |
                  WIPHY_FLAG_HAVE_AP_SME |
                  WIPHY_FLAG_AP_PROBE_RESP_OFFLOAD |
                  WIPHY_FLAG_IBSS_RSN;

  /* Set management frame registrations */

  wiphy->mgmt_stypes = g_esp_default_mgmt_stypes;

  esp_main_info("Wiphy configured with features and capabilities\n");
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp_wifi_driver_init
 *
 * Description:
 *   Initialize the ESP32 WiFi driver.
 *
 ****************************************************************************/

int esp_wifi_driver_init(void)
{
  FAR struct esp_if_ops *ops;
  int ret;

  esp_main_info("Initializing ESP32 WiFi driver\n");

  /* Get interface operations */

  ops = esp_get_interface_ops();
  if (!ops)
    {
      esp_main_err("No valid interface operations found\n");
      return -ENODEV;
    }

  /* Initialize the cfg80211 interface */

  ret = esp_cfg80211_init(ops, 
#ifdef CONFIG_ESP32_WIFI_SDIO
                          ESP_IF_TYPE_SDIO
#else
                          ESP_IF_TYPE_SPI
#endif
                         );
  if (ret < 0)
    {
      esp_main_err("Failed to initialize cfg80211 interface: %d\n", ret);
      return ret;
    }

  /* Initialize event queue */

  ret = esp_event_queue_init();
  if (ret < 0)
    {
      esp_main_err("Failed to initialize event queue: %d\n", ret);
      esp_cfg80211_deinit();
      return ret;
    }

  /* Initialize and register interfaces */

  ret = esp_register_interfaces(&g_esp_adapter);
  if (ret < 0)
    {
      esp_main_err("Failed to register interfaces: %d\n", ret);
      esp_event_queue_deinit();
      esp_cfg80211_deinit();
      return ret;
    }

  /* Start event processing worker */

  ret = esp_event_start_worker(&g_esp_adapter);
  if (ret < 0)
    {
      esp_main_err("Failed to start event worker: %d\n", ret);
      /* Continue anyway, may work without async events */
    }

  esp_main_info("ESP32 WiFi driver initialized successfully\n");

  return OK;
}

/****************************************************************************
 * Name: esp_wifi_driver_deinit
 *
 * Description:
 *   Deinitialize the ESP32 WiFi driver.
 *
 ****************************************************************************/

void esp_wifi_driver_deinit(void)
{
  esp_main_info("Deinitializing ESP32 WiFi driver\n");

  /* Stop event worker */

  esp_event_stop_worker();

  /* Unregister interfaces */

  /* TODO: Unregister interfaces */

  /* Deinitialize event queue */

  esp_event_queue_deinit();

  /* Deinitialize cfg80211 interface */

  esp_cfg80211_deinit();

  esp_main_info("ESP32 WiFi driver deinitialized\n");
}

/****************************************************************************
 * Name: esp_get_adapter
 *
 * Description:
 *   Get the global adapter instance.
 *
 ****************************************************************************/

FAR struct esp_adapter *esp_get_adapter(void)
{
  return &g_esp_adapter;
}

/****************************************************************************
 * Name: esp_get_wiphy
 *
 * Description:
 *   Get the global wiphy instance.
 *
 ****************************************************************************/

FAR struct wiphy *esp_get_wiphy(void)
{
  return g_esp_wiphy;
}

/****************************************************************************
 * Name: esp_send_command
 *
 * Description:
 *   Send a command to the ESP32.
 *
 ****************************************************************************/

int esp_send_command(FAR struct esp_wifi_device *priv, uint8_t cmd_code,
                     FAR const uint8_t *data, size_t len)
{
  FAR struct esp_adapter *adapter;
  struct esp_payload_header payload_hdr;
  uint8_t *buf;
  size_t total_len;
  uint16_t checksum;
  int ret;

  if (!priv || !data || len == 0 || len > ESP_SIZE_OF_CMD_NODE)
    {
      return -EINVAL;
    }

  adapter = priv->adapter;
  if (!adapter)
    {
      return -ENODEV;
    }

  /* Prepare payload header */

  memset(&payload_hdr, 0, sizeof(payload_hdr));
  payload_hdr.if_type = adapter->if_type;
  payload_hdr.if_num = priv->if_num;
  payload_hdr.flags = 0;
  payload_hdr.packet_type = ESP_PACKET_TYPE_COMMAND_REQUEST;
  payload_hdr.reserved1 = 0;
  payload_hdr.len = len + sizeof(struct esp_command_header);
  payload_hdr.offset = 0;
  payload_hdr.checksum = 0; /* Will calculate below */
  payload_hdr.reserved2 = 0;
  payload_hdr.priv_pkt_type = 0;

  /* Calculate checksum for payload header + data */

  checksum = esp_compute_checksum((FAR const uint8_t *)&payload_hdr + 4, 
                                  sizeof(payload_hdr) - 4);
  checksum += esp_compute_checksum(data, len);
  payload_hdr.checksum = checksum;

  /* Prepare full packet buffer */

  total_len = sizeof(payload_hdr) + len;
  buf = kmm_malloc(total_len);
  if (!buf)
    {
      return -ENOMEM;
    }

  /* Copy headers and data */

  memcpy(buf, &payload_hdr, sizeof(payload_hdr));
  memcpy(buf + sizeof(payload_hdr), data, len);

  /* Send via interface */

  ret = adapter->if_ops->send(adapter, buf, total_len);

  kmm_free(buf);

  if (ret < 0)
    {
      esp_main_err("Failed to send command %d: %d\n", cmd_code, ret);
      return ret;
    }

  esp_main_verbose("Sent command %d, len=%zu\n", cmd_code, total_len);

  return OK;
}

/****************************************************************************
 * Name: esp_compute_checksum
 *
 * Description:
 *   Compute checksum for ESP protocol.
 *
 ****************************************************************************/

uint16_t esp_compute_checksum(FAR const uint8_t *buf, uint16_t len)
{
  uint16_t checksum = 0;
  uint16_t i = 0;

  while (i < len)
    {
      checksum += buf[i];
      i++;
    }

  return checksum;
}

#endif /* CONFIG_DRIVERS_ESP32_WIFI */