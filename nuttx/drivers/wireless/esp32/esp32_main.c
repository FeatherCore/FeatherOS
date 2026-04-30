/****************************************************************************
 * drivers/wireless/esp32/esp32_main.c
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
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <sys/types.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <errno.h>
#include <debug.h>

#include <nuttx/kmalloc.h>
#include <nuttx/wireless/esp32_wifi.h>
#include <nuttx/wireless/cfg80211.h>

#include "esp_cfg80211.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#ifdef CONFIG_DEBUG_WIRELESS_INFO
#  define esp32_wlaninfo(format, ...) \
     ninfo("esp32: " format, ##__VA_ARGS__)
#else
#  define esp32_wlaninfo(format, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_ERROR
#  define esp32_wlanerr(format, ...) \
     nerr("esp32: " format, ##__VA_ARGS__)
#else
#  define esp32_wlanerr(format, ...)
#endif

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* ESP32 WiFi master context */

struct esp32_wifi_master_priv_s
{
  bool initialized;                     /* Initialization state */
  bool communication_initialized;      /* Communication interface initialized */
  bool cfg80211_initialized;           /* cfg80211 interface initialized */
  enum esp32_comm_mode_e comm_mode;    /* Communication mode (SDIO/SPI) */
  
  /* Communication interface handlers */
  union
  {
#ifdef CONFIG_ESP32_WIFI_SDIO
    struct esp32_sdio_priv_s sdio;
#endif
#ifdef CONFIG_ESP32_WIFI_SPI
    struct esp32_spi_priv_s spi;
#endif
  } comm;
  
  /* cfg80211 interface */
  struct wiphy *wiphy;
  struct esp_adapter *adapter;
};

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct esp32_wifi_master_priv_s g_esp32_wifi_master;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp32_wifi_init_communication
 *
 * Description:
 *   Initialize ESP32 communication interface based on configuration.
 *
 ****************************************************************************/

static int esp32_wifi_init_communication(FAR struct esp32_wifi_master_priv_s *master)
{
  int ret = OK;

  esp32_wlaninfo("Initializing ESP32 communication interface\n");

#ifdef CONFIG_ESP32_WIFI_SDIO
  if (master->comm_mode == ESP32_COMM_SDIO)
    {
      ret = esp32_sdio_initialize(CONFIG_ESP32_WIFI_SDIO_DEVMINOR,
                                  CONFIG_ESP32_WIFI_SDIO_FUNCTION);
      if (ret >= 0)
        {
          master->communication_initialized = true;
          master->comm.sdio.initialized = true;
          esp32_wlaninfo("SDIO communication initialized\n");
        }
      else
        {
          esp32_wlanerr("Failed to initialize SDIO communication: %d\n", ret);
        }
    }
#endif

#ifdef CONFIG_ESP32_WIFI_SPI
  if (master->comm_mode == ESP32_COMM_SPI)
    {
      ret = esp32_spi_initialize(CONFIG_ESP32_WIFI_SPI_DEVMINOR,
                                 CONFIG_ESP32_WIFI_SPI_INTR_PIN,
                                 CONFIG_ESP32_WIFI_SPI_CS_PIN);
      if (ret >= 0)
        {
          master->communication_initialized = true;
          master->comm.spi.initialized = true;
          esp32_wlaninfo("SPI communication initialized\n");
        }
      else
        {
          esp32_wlanerr("Failed to initialize SPI communication: %d\n", ret);
        }
    }
#endif

  return ret;
}

/****************************************************************************
 * Name: esp32_wifi_init_cfg80211
 *
 * Description:
 *   Initialize cfg80211 interface for ESP32.
 *
 ****************************************************************************/

static int esp32_wifi_init_cfg80211(FAR struct esp32_wifi_master_priv_s *master)
{
  int ret;
  struct esp_if_ops *if_ops = NULL;

  esp32_wlaninfo("Initializing cfg80211 interface\n");

#ifdef CONFIG_ESP32_WIFI_SDIO
  if (master->comm_mode == ESP32_COMM_SDIO)
    {
      extern struct esp_if_ops g_esp32_sdio_ops;
      if_ops = &g_esp32_sdio_ops;
    }
#endif

#ifdef CONFIG_ESP32_WIFI_SPI
  if (master->comm_mode == ESP32_COMM_SPI)
    {
      extern struct esp_if_ops g_esp32_spi_ops;
      if_ops = &g_esp32_spi_ops;
    }
#endif

  if (!if_ops)
    {
      esp32_wlanerr("No interface operations available\n");
      return -ENODEV;
    }

  /* Initialize ESP32 boot sequence */
  ret = esp32_boot_init(NULL);  /* We'll pass adapter later after registration */
  if (ret < 0)
    {
      esp32_wlanerr("Failed to initialize ESP32 boot sequence: %d\n", ret);
      return ret;
    }

  /* Register with cfg80211 subsystem */
  ret = esp_cfg80211_init(if_ops, master->comm_mode == ESP32_COMM_SDIO ? 
                          ESP_IF_TYPE_SDIO : ESP_IF_TYPE_SPI);
  if (ret < 0)
    {
      esp32_wlanerr("Failed to register with cfg80211: %d\n", ret);
      return ret;
    }

  /* Get adapter reference */
  master->adapter = esp_get_adapter();
  if (!master->adapter)
    {
      esp32_wlanerr("Failed to get adapter\n");
      return -ENODEV;
    }

  /* Get wiphy reference */
  master->wiphy = master->adapter->wiphy;
  if (!master->wiphy)
    {
      esp32_wlanerr("Failed to get wiphy\n");
      return -ENODEV;
    }

  /* Check capabilities */
  ret = esp32_check_capabilities(master->adapter);
  if (ret < 0)
    {
      esp32_wlanerr("Capability check failed: %d\n", ret);
      return ret;
    }

  /* Add initial interfaces */
  ret = esp_add_interface(master->adapter, "wlan0", NL80211_IFTYPE_STATION);
  if (ret < 0)
    {
      esp32_wlanerr("Failed to add initial station interface: %d\n", ret);
    }
  
#ifdef CONFIG_ESP32_WIFI_AP
  ret = esp_add_interface(master->adapter, "ap0", NL80211_IFTYPE_AP);
  if (ret < 0)
    {
      esp32_wlanerr("Failed to add initial AP interface: %d\n", ret);
    }
#endif

  esp32_wlaninfo("cfg80211 interface initialized successfully\n");
  master->cfg80211_initialized = true;
  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp32_wifi_initialize
 *
 * Description:
 *   Initialize the ESP32 WiFi driver.
 *
 ****************************************************************************/

int esp32_wifi_initialize(void)
{
  FAR struct esp32_wifi_master_priv_s *master = &g_esp32_wifi_master;
  int ret;

  esp32_wlaninfo("Initializing ESP32 WiFi driver\n");

  /* Initialize master context */
  memset(master, 0, sizeof(struct esp32_wifi_master_priv_s));

  /* Determine communication mode from configuration */
#ifdef CONFIG_ESP32_WIFI_SDIO
  master->comm_mode = ESP32_COMM_SDIO;
#elif defined(CONFIG_ESP32_WIFI_SPI)
  master->comm_mode = ESP32_COMM_SPI;
#else
  esp32_wlanerr("No communication mode configured\n");
  return -ENODEV;
#endif

  /* Initialize communication interface */
  ret = esp32_wifi_init_communication(master);
  if (ret < 0)
    {
      esp32_wlanerr("Failed to initialize communication: %d\n", ret);
      return ret;
    }

  /* Initialize cfg80211 interface */
  ret = esp32_wifi_init_cfg80211(master);
  if (ret < 0)
    {
      esp32_wlanerr("Failed to initialize cfg80211: %d\n", ret);
      goto err_comm_deinit;
    }

  master->initialized = true;

  esp32_wlaninfo("ESP32 WiFi driver initialized successfully\n");
  return OK;

err_comm_deinit:
#ifdef CONFIG_ESP32_WIFI_SDIO
  if (master->comm_mode == ESP32_COMM_SDIO && master->comm.sdio.initialized)
    {
      esp32_sdio_uninitialize();
      master->comm.sdio.initialized = false;
    }
#endif

#ifdef CONFIG_ESP32_WIFI_SPI
  if (master->comm_mode == ESP32_COMM_SPI && master->comm.spi.initialized)
    {
      esp32_spi_uninitialize();
      master->comm.spi.initialized = false;
    }
#endif

  return ret;
}

/****************************************************************************
 * Name: esp32_wifi_uninitialize
 *
 * Description:
 *   Uninitialize the ESP32 WiFi driver.
 *
 ****************************************************************************/

int esp32_wifi_uninitialize(void)
{
  FAR struct esp32_wifi_master_priv_s *master = &g_esp32_wifi_master;

  if (!master->initialized)
    {
      return -ENODEV;
    }

  esp32_wlaninfo("Uninitializing ESP32 WiFi driver\n");

  /* Uninitialize cfg80211 interface */
  if (master->cfg80211_initialized)
    {
      esp_cfg80211_deinit();
      master->cfg80211_initialized = false;
    }

  /* Uninitialize communication interface */
  if (master->communication_initialized)
    {
#ifdef CONFIG_ESP32_WIFI_SDIO
      if (master->comm_mode == ESP32_COMM_SDIO)
        {
          esp32_sdio_uninitialize();
          master->comm.sdio.initialized = false;
        }
#endif

#ifdef CONFIG_ESP32_WIFI_SPI
      if (master->comm_mode == ESP32_COMM_SPI)
        {
          esp32_spi_uninitialize();
          master->comm.spi.initialized = false;
        }
#endif
      master->communication_initialized = false;
    }

  /* Clear initialization state */
  master->initialized = false;

  esp32_wlaninfo("ESP32 WiFi driver uninitialized\n");
  return OK;
}

/****************************************************************************
 * Name: esp32_wifi_get_status
 *
 * Description:
 *   Get current WiFi status.
 *
 ****************************************************************************/

int esp32_wifi_get_status(FAR struct esp32_wifi_status_s *status)
{
  FAR struct esp32_wifi_master_priv_s *master = &g_esp32_wifi_master;

  if (!master->initialized || !status)
    {
      return -EINVAL;
    }

  /* Get status from lower layer */
  return esp32_wifi_get_status_impl(status);
}

/****************************************************************************
 * Name: esp32_wifi_scan
 *
 * Description:
 *   Perform WiFi scan.
 *
 ****************************************************************************/

int esp32_wifi_scan(void)
{
  FAR struct esp32_wifi_master_priv_s *master = &g_esp32_wifi_master;

  if (!master->initialized)
    {
      return -ENODEV;
    }

  esp32_wlaninfo("Starting WiFi scan\n");

  /* Initiate scan via cfg80211 interface */
  return esp32_cfg80211_scan_request();
}

/****************************************************************************
 * Name: esp32_wifi_connect
 *
 * Description:
 *   Connect to WiFi network.
 *
 ****************************************************************************/

int esp32_wifi_connect(FAR const char *ssid, size_t ssid_len,
                      FAR const uint8_t *bssid)
{
  FAR struct esp32_wifi_master_priv_s *master = &g_esp32_wifi_master;

  if (!master->initialized || !ssid || ssid_len == 0)
    {
      return -EINVAL;
    }

  esp32_wlaninfo("Connecting to SSID: %.*s\n", (int)ssid_len, ssid);

  /* Initiate connection via cfg80211 interface */
  return esp32_cfg80211_connect_request(ssid, ssid_len, bssid);
}

/****************************************************************************
 * Name: esp32_wifi_disconnect
 *
 * Description:
 *   Disconnect from WiFi network.
 *
 ****************************************************************************/

int esp32_wifi_disconnect(void)
{
  FAR struct esp32_wifi_master_priv_s *master = &g_esp32_wifi_master;

  if (!master->initialized)
    {
      return -ENODEV;
    }

  esp32_wlaninfo("Disconnecting from WiFi network\n");

  /* Initiate disconnection via cfg80211 interface */
  return esp32_cfg80211_disconnect_request();
}

/****************************************************************************
 * Name: esp32_wifi_register_event_callback
 *
 * Description:
 *   Register an event callback for WiFi events.
 *
 ****************************************************************************/

int esp32_wifi_register_event_callback(wifi_event_callback_t callback, void *arg)
{
  FAR struct esp32_wifi_master_priv_s *master = &g_esp32_wifi_master;

  if (!master->initialized || !callback)
    {
      return -EINVAL;
    }

  /* Register callback in lower layer */
  return esp32_wifi_register_event_callback_impl(callback, arg);
}

/****************************************************************************
 * Name: esp32_wifi_send_command
 *
 * Description:
 *   Send a command to ESP32.
 *
 ****************************************************************************/

int esp32_wifi_send_command(uint8_t cmd, FAR const void *data, size_t data_len)
{
  FAR struct esp32_wifi_master_priv_s *master = &g_esp32_wifi_master;

  if (!master->initialized || !data)
    {
      return -EINVAL;
    }

  /* Send command via communication interface */
  if (master->comm_mode == ESP32_COMM_SDIO)
    {
#ifdef CONFIG_ESP32_WIFI_SDIO
      return esp32_sdio_send_command(cmd, data, data_len);
#else
      return -ENOTSUP;
#endif
    }
  else if (master->comm_mode == ESP32_COMM_SPI)
    {
#ifdef CONFIG_ESP32_WIFI_SPI
      return esp32_spi_send_command(cmd, data, data_len);
#else
      return -ENOTSUP;
#endif
    }

  return -ENOTSUP;
}

/****************************************************************************
 * Name: esp32_wifi_receive_response
 *
 * Description:
 *   Receive response from ESP32.
 *
 ****************************************************************************/

int esp32_wifi_receive_response(FAR void *buffer, size_t *buf_len)
{
  FAR struct esp32_wifi_master_priv_s *master = &g_esp32_wifi_master;

  if (!master->initialized || !buffer || !buf_len)
    {
      return -EINVAL;
    }

  /* Receive response via communication interface */
  if (master->comm_mode == ESP32_COMM_SDIO)
    {
#ifdef CONFIG_ESP32_WIFI_SDIO
      return esp32_sdio_receive_response(buffer, buf_len);
#else
      return -ENOTSUP;
#endif
    }
  else if (master->comm_mode == ESP32_COMM_SPI)
    {
#ifdef CONFIG_ESP32_WIFI_SPI
      return esp32_spi_receive_response(buffer, buf_len);
#else
      return -ENOTSUP;
#endif
    }

  return -ENOTSUP;
}

/****************************************************************************
 * Name: esp32_wifi_get_master
 *
 * Description:
 *   Get the master context for the ESP32 WiFi driver.
 *
 ****************************************************************************/

FAR struct esp32_wifi_master_priv_s *esp32_wifi_get_master(void)
{
  return &g_esp32_wifi_master;
}