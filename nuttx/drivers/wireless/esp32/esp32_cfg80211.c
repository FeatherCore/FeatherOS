/****************************************************************************
 * drivers/wireless/esp32/esp32_cfg80211.c
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
 * ESP32 cfg80211 Driver Implementation
 *
 * This file implements the cfg80211 operations for ESP32 WiFi driver.
 * It provides standard cfg80211 interface for wpa_supplicant/hostapd.
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
#include <nuttx/nuttx.h>
#include <nuttx/semaphore.h>
#include <nuttx/wqueue.h>
#include <nuttx/wireless/cfg80211.h>
#include <nuttx/wireless/nl80211.h>

#include "esp_cfg80211.h"
#include "esp_host_if.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#ifdef CONFIG_DEBUG_WIRELESS_INFO
#  define esp_info(fmt, ...)  ninfo("esp32: " fmt, ##__VA_ARGS__)
#else
#  define esp_info(fmt, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_ERROR
#  define esp_err(fmt, ...)   nerr("esp32: " fmt, ##__VA_ARGS__)
#else
#  define esp_err(fmt, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_WARN
#  define esp_warn(fmt, ...)  nwarn("esp32: " fmt, ##__VA_ARGS__)
#else
#  define esp_warn(fmt, ...)
#endif

/* Default values */

#define ESP_DEFAULT_TX_POWER_DBM 15

/* Channels and rates for 2.4GHz band */

static struct ieee80211_channel g_esp_channels_2ghz[] =
{
  { .band = IEEE80211_BAND_2GHZ, .center_freq = 2412, .hw_value = 1, .max_power = 20 },
  { .band = IEEE80211_BAND_2GHZ, .center_freq = 2417, .hw_value = 2, .max_power = 20 },
  { .band = IEEE80211_BAND_2GHZ, .center_freq = 2422, .hw_value = 3, .max_power = 20 },
  { .band = IEEE80211_BAND_2GHZ, .center_freq = 2427, .hw_value = 4, .max_power = 20 },
  { .band = IEEE80211_BAND_2GHZ, .center_freq = 2432, .hw_value = 5, .max_power = 20 },
  { .band = IEEE80211_BAND_2GHZ, .center_freq = 2437, .hw_value = 6, .max_power = 20 },
  { .band = IEEE80211_BAND_2GHZ, .center_freq = 2442, .hw_value = 7, .max_power = 20 },
  { .band = IEEE80211_BAND_2GHZ, .center_freq = 2447, .hw_value = 8, .max_power = 20 },
  { .band = IEEE80211_BAND_2GHZ, .center_freq = 2452, .hw_value = 9, .max_power = 20 },
  { .band = IEEE80211_BAND_2GHZ, .center_freq = 2457, .hw_value = 10, .max_power = 20 },
  { .band = IEEE80211_BAND_2GHZ, .center_freq = 2462, .hw_value = 11, .max_power = 20 },
  { .band = IEEE80211_BAND_2GHZ, .center_freq = 2467, .hw_value = 12, .max_power = 20 },
  { .band = IEEE80211_BAND_2GHZ, .center_freq = 2472, .hw_value = 13, .max_power = 20 },
  { .band = IEEE80211_BAND_2GHZ, .center_freq = 2484, .hw_value = 14, .max_power = 20 },
};

/* Rates */

static struct ieee80211_rate g_esp_rates[] =
{
  { .bitrate = 10, .hw_value = 0x00 },  /* 1 Mbps */
  { .bitrate = 20, .hw_value = 0x01 },  /* 2 Mbps */
  { .bitrate = 55, .hw_value = 0x02 },  /* 5.5 Mbps */
  { .bitrate = 110, .hw_value = 0x03 }, /* 11 Mbps */
  { .bitrate = 60, .hw_value = 0x0B },  /* 6 Mbps */
  { .bitrate = 90, .hw_value = 0x0F },  /* 9 Mbps */
  { .bitrate = 120, .hw_value = 0x0A }, /* 12 Mbps */
  { .bitrate = 180, .hw_value = 0x0E }, /* 18 Mbps */
  { .bitrate = 240, .hw_value = 0x09 }, /* 24 Mbps */
  { .bitrate = 360, .hw_value = 0x0D }, /* 36 Mbps */
  { .bitrate = 480, .hw_value = 0x08 }, /* 48 Mbps */
  { .bitrate = 540, .hw_value = 0x0C }, /* 54 Mbps */
};

/* 2.4GHz band definition */

static struct ieee80211_supported_band g_esp_wifi_bands_2ghz =
{
  .band = IEEE80211_BAND_2GHZ,
  .channels = g_esp_channels_2ghz,
  .n_channels = sizeof(g_esp_channels_2ghz) / sizeof(g_esp_channels_2ghz[0]),
  .bitrates = g_esp_rates,
  .n_bitrates = sizeof(g_esp_rates) / sizeof(g_esp_rates[0]),
  .ht_cap.ht_supported = true,
  .ht_cap.cap = IEEE80211_HT_CAP_SUP_WIDTH_20_40 | IEEE80211_HT_CAP_SGI_20 |
                IEEE80211_HT_CAP_RX_STBC | IEEE80211_HT_CAP_DSSSCCK40,
};

/* Supported cipher suites */

static const uint32_t g_esp_cipher_suites[] =
{
  NL80211_CIPHER_SUITE_WEP40,
  NL80211_CIPHER_SUITE_WEP104,
  NL80211_CIPHER_SUITE_TKIP,
  NL80211_CIPHER_SUITE_CCMP,
  NL80211_CIPHER_SUITE_AES_CMAC,
};

/* Default management frame types */

static const struct ieee80211_txrx_stypes
g_esp_default_mgmt_stypes[NL80211_IFTYPE_MAX] =
{
  [NL80211_IFTYPE_STATION] = {
    .tx = 0xffff,
    .rx = BIT(IEEE80211_STYPE_ACTION >> 4) | BIT(IEEE80211_STYPE_PROBE_REQ >> 4),
  },
  [NL80211_IFTYPE_AP] = {
    .tx = 0xffff,
    .rx = BIT(IEEE80211_STYPE_ASSOC_REQ >> 4) |
          BIT(IEEE80211_STYPE_REASSOC_REQ >> 4) |
          BIT(IEEE80211_STYPE_PROBE_REQ >> 4) |
          BIT(IEEE80211_STYPE_DISASSOC >> 4) |
          BIT(IEEE80211_STYPE_AUTH >> 4) |
          BIT(IEEE80211_STYPE_DEAUTH >> 4) |
          BIT(IEEE80211_STYPE_ACTION >> 4),
  },
};

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct esp_adapter g_esp_adapter;

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

/* cfg80211 operations */

static int esp_cfg80211_add_iface(struct wiphy *wiphy,
                                   const char *name,
                                   enum nl80211_iftype type,
                                   struct vif_params *params);
static int esp_cfg80211_del_iface(struct wiphy *wiphy,
                                   struct wireless_dev *wdev);
static int esp_cfg80211_change_iface(struct wiphy *wiphy,
                                    struct wireless_dev *wdev,
                                    enum nl80211_iftype type,
                                    struct vif_params *params);
static int esp_cfg80211_scan(struct wiphy *wiphy,
                                struct cfg80211_scan_request *request);
static int esp_cfg80211_connect(struct wiphy *wiphy,
                                 struct net_device *dev,
                                 struct cfg80211_connect_params *sme);
static int esp_cfg80211_disconnect(struct wiphy *wiphy,
                                    struct net_device *dev,
                                    uint16_t reason_code);
static int esp_cfg80211_add_key(struct wiphy *wiphy,
                                 struct net_device *dev,
                                 uint8_t key_index, bool pairwise,
                                 const uint8_t *mac_addr,
                                 struct key_params *params);
static int esp_cfg80211_del_key(struct wiphy *wiphy,
                                 struct net_device *dev,
                                 uint8_t key_index, bool pairwise,
                                 const uint8_t *mac_addr);
static int esp_cfg80211_set_default_key(struct wiphy *wiphy,
                                          struct net_device *dev,
                                          uint8_t key_index,
                                          bool unicast, bool multicast);
static int esp_cfg80211_start_ap(struct wiphy *wiphy,
                                 struct net_device *dev,
                                 struct cfg80211_ap_settings *settings);
static int esp_cfg80211_stop_ap(struct wiphy *wiphy,
                                struct net_device *dev);
static int esp_cfg80211_change_beacon(struct wiphy *wiphy,
                                       struct net_device *dev,
                                       struct cfg80211_beacon_settings *info);
static int esp_cfg80211_add_station(struct wiphy *wiphy,
                                  struct net_device *dev,
                                  const uint8_t *mac,
                                  struct station_parameters *params);
static int esp_cfg80211_del_station(struct wiphy *wiphy,
                                  struct net_device *dev,
                                  struct station_del_parameters *params);
static int esp_cfg80211_change_station(struct wiphy *wiphy,
                                     struct net_device *dev,
                                     const uint8_t *mac,
                                     struct station_parameters *params);
static int esp_cfg80211_get_station(struct wiphy *wiphy,
                                  struct net_device *dev,
                                  const uint8_t *mac,
                                  struct station_info *sinfo);
static int esp_cfg80211_set_tx_power(struct wiphy *wiphy,
                                    struct wireless_dev *wdev,
                                    enum nl80211_tx_power_setting type,
                                    int mbm);
static int esp_cfg80211_get_tx_power(struct wiphy *wiphy,
                                    struct wireless_dev *wdev,
                                    int *dbm);
static int esp_cfg80211_set_wiphy_params(struct wiphy *wiphy,
                                          uint32_t changed);
static int esp_cfg80211_mgmt_tx(struct wiphy *wiphy,
                                struct wireless_dev *wdev,
                                struct cfg80211_mgmt_tx_params *params,
                                uint64_t *cookie);
static int esp_cfg80211_set_bitrate_mask(struct wiphy *wiphy,
                                      struct net_device *dev,
                                      const uint8_t *peer,
                                      struct cfg80211_bitrate_mask *mask);

static const struct cfg80211_ops g_esp_cfg80211_ops =
{
  .add_virtual_intf = esp_cfg80211_add_iface,
  .del_virtual_intf = esp_cfg80211_del_iface,
  .change_virtual_intf = esp_cfg80211_change_iface,
  .scan = esp_cfg80211_scan,
  .connect = esp_cfg80211_connect,
  .disconnect = esp_cfg80211_disconnect,
  .add_key = esp_cfg80211_add_key,
  .del_key = esp_cfg80211_del_key,
  .set_default_key = esp_cfg80211_set_default_key,
  .start_ap = esp_cfg80211_start_ap,
  .stop_ap = esp_cfg80211_stop_ap,
  .change_beacon = esp_cfg80211_change_beacon,
  .add_station = esp_cfg80211_add_station,
  .del_station = esp_cfg80211_del_station,
  .change_station = esp_cfg80211_change_station,
  .get_station = esp_cfg80211_get_station,
  .set_tx_power = esp_cfg80211_set_tx_power,
  .get_tx_power = esp_cfg80211_get_tx_power,
  .set_wiphy_params = esp_cfg80211_set_wiphy_params,
  .mgmt_tx = esp_cfg80211_mgmt_tx,
};

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp_cfg80211_add_iface
 *
 * Description:
 *   Add a virtual interface.
 *
 ****************************************************************************/

static int esp_cfg80211_add_iface(struct wiphy *wiphy,
                                   const char *name,
                                   enum nl80211_iftype type,
                                   struct vif_params *params)
{
  struct esp_adapter *adapter;
  struct esp_wifi_device *priv;
  int ret;

  if (!wiphy || !name)
    {
      esp_err("Invalid parameters\n");
      return -EINVAL;
    }

  adapter = wiphy_priv(wiphy);
  if (!adapter)
    {
      esp_err("Invalid adapter\n");
      return -EINVAL;
    }

  /* Allocate WiFi device */
  priv = kmm_zalloc(sizeof(struct esp_wifi_device));
  if (!priv)
    {
      esp_err("Failed to allocate WiFi device\n");
      return -ENOMEM;
    }

  /* Initialize WiFi device */
  priv->wdev.wiphy = wiphy;
  priv->adapter = adapter;
  priv->if_type = (type == NL80211_IFTYPE_STATION) ? ESP_STA_IF_TYPE : ESP_AP_IF_TYPE;
  priv->if_num = (priv->if_type == ESP_STA_IF_TYPE) ? 0 : 1;
  priv->wdev.iftype = type;

  /* Register with ESP32 */
  ret = esp_cmd_init_interface(priv);
  if (ret < 0)
    {
      esp_err("Failed to initialize interface: %d\n", ret);
      kmm_free(priv);
      return ret;
    }

  /* Get MAC address */
  ret = esp_cmd_get_mac(priv);
  if (ret < 0)
    {
      esp_err("Failed to get MAC: %d\n", ret);
      esp_cmd_deinit_interface(priv);
      kmm_free(priv);
      return ret;
    }

  /* Register netdevice */
  ret = esp32_netdev_register(priv);
  if (ret < 0)
    {
      esp_err("Failed to register netdev: %d\n", ret);
      esp_cmd_deinit_interface(priv);
      kmm_free(priv);
      return ret;
    }

  /* Store in adapter */
  if (priv->if_num < ESP_MAX_INTERFACE)
    {
      adapter->priv[priv->if_num] = priv;
    }

  esp_info("Added interface %s (type=%d, MAC=%02x:%02x:%02x:%02x:%02x:%02x)\n",
            name, type,
            priv->mac_address[0], priv->mac_address[1],
            priv->mac_address[2], priv->mac_address[3],
            priv->mac_address[4], priv->mac_address[5]);

  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_del_iface
 *
 * Description:
 *   Delete a virtual interface.
 *
 ****************************************************************************/

static int esp_cfg80211_del_iface(struct wiphy *wiphy,
                                   struct wireless_dev *wdev)
{
  struct esp_wifi_device *priv;

  if (!wiphy || !wdev)
    {
      return -EINVAL;
    }

  priv = container_of(wdev, struct esp_wifi_device, wdev);
  if (!priv)
    {
      return -EINVAL;
    }

  /* Unregister netdevice */
  esp32_netdev_unregister(priv);

  /* Deinit interface */
  esp_cmd_deinit_interface(priv);

  /* Clear from adapter */
  if (priv->adapter && priv->if_num < ESP_MAX_INTERFACE)
    {
      priv->adapter->priv[priv->if_num] = NULL;
    }

  /* Free private data */
  kmm_free(priv);

  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_change_iface
 *
 * Description:
 *   Change interface type.
 *
 ****************************************************************************/

static int esp_cfg80211_change_iface(struct wiphy *wiphy,
                                    struct wireless_dev *wdev,
                                    enum nl80211_iftype type,
                                    struct vif_params *params)
{
  struct esp_wifi_device *priv;
  int ret;

  if (!wiphy || !wdev)
    {
      return -EINVAL;
    }

  priv = container_of(wdev, struct esp_wifi_device, wdev);
  if (!priv)
    {
      return -EINVAL;
    }

  /* Set mode */
  ret = esp_cmd_set_mode(priv, (type == NL80211_IFTYPE_STATION) ? 0 : 1);
  if (ret < 0)
    {
      esp_err("Failed to set mode: %d\n", ret);
      return ret;
    }

  /* Update type */
  priv->if_type = (type == NL80211_IFTYPE_STATION) ? ESP_STA_IF_TYPE : ESP_AP_IF_TYPE;
  wdev->iftype = type;

  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_scan
 *
 * Description:
 *   Trigger a scan.
 *
 ****************************************************************************/

static int esp_cfg80211_scan(struct wiphy *wiphy,
                                struct cfg80211_scan_request *request)
{
  struct esp_wifi_device *priv;
  int ret;

  if (!wiphy || !request || !request->wdev)
    {
      return -EINVAL;
    }

  priv = container_of(request->wdev, struct esp_wifi_device, wdev);
  if (!priv)
    {
      return -EINVAL;
    }

  priv->scan_request = request;
  priv->scan_in_progress = 1;

  ret = esp_cmd_scan_request(priv, request);
  if (ret < 0)
    {
      esp_err("Scan request failed: %d\n", ret);
      priv->scan_in_progress = 0;
      return ret;
    }

  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_connect
 *
 * Description:
 *   Connect to an AP.
 *
 ****************************************************************************/

static int esp_cfg80211_connect(struct wiphy *wiphy,
                                 struct net_device *dev,
                                 struct cfg80211_connect_params *sme)
{
  struct esp_wifi_device *priv;
  int ret;

  if (!wiphy || !dev || !sme)
    {
      return -EINVAL;
    }

  priv = (struct esp_wifi_device *)dev->d_private;
  if (!priv)
    {
      return -EINVAL;
    }

  ret = esp_cmd_connect(priv, sme);
  if (ret < 0)
    {
      esp_err("Connect failed: %d\n", ret);
      return ret;
    }

  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_disconnect
 *
 * Description:
 *   Disconnect from an AP.
 *
 ****************************************************************************/

static int esp_cfg80211_disconnect(struct wiphy *wiphy,
                                    struct net_device *dev,
                                    uint16_t reason_code)
{
  struct esp_wifi_device *priv;
  int ret;

  if (!wiphy || !dev)
    {
      return -EINVAL;
    }

  priv = (struct esp_wifi_device *)dev->d_private;
  if (!priv)
    {
      return -EINVAL;
    }

  ret = esp_cmd_disconnect(priv, reason_code, NULL);
  if (ret < 0)
    {
      esp_err("Disconnect failed: %d\n", ret);
      return ret;
    }

  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_add_key
 *
 * Description:
 *   Add a key.
 *
 ****************************************************************************/

static int esp_cfg80211_add_key(struct wiphy *wiphy,
                                 struct net_device *dev,
                                 uint8_t key_index, bool pairwise,
                                 const uint8_t *mac_addr,
                                 struct key_params *params)
{
  struct esp_wifi_device *priv;
  int ret;

  if (!wiphy || !dev || !params)
    {
      return -EINVAL;
    }

  priv = (struct esp_wifi_device *)dev->d_private;
  if (!priv)
    {
      return -EINVAL;
    }

  ret = esp_cmd_add_key(priv, key_index, pairwise, mac_addr, params);
  if (ret < 0)
    {
      esp_err("Add key failed: %d\n", ret);
      return ret;
    }

  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_del_key
 *
 * Description:
 *   Delete a key.
 *
 ****************************************************************************/

static int esp_cfg80211_del_key(struct wiphy *wiphy,
                                 struct net_device *dev,
                                 uint8_t key_index, bool pairwise,
                                 const uint8_t *mac_addr)
{
  struct esp_wifi_device *priv;
  int ret;

  if (!wiphy || !dev)
    {
      return -EINVAL;
    }

  priv = (struct esp_wifi_device *)dev->d_private;
  if (!priv)
    {
      return -EINVAL;
    }

  ret = esp_cmd_del_key(priv, key_index, pairwise, mac_addr);
  if (ret < 0)
    {
      esp_err("Delete key failed: %d\n", ret);
      return ret;
    }

  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_set_default_key
 *
 * Description:
 *   Set default key.
 *
 ****************************************************************************/

static int esp_cfg80211_set_default_key(struct wiphy *wiphy,
                                          struct net_device *dev,
                                          uint8_t key_index,
                                          bool unicast, bool multicast)
{
  struct esp_wifi_device *priv;
  int ret;

  if (!wiphy || !dev)
    {
      return -EINVAL;
    }

  priv = (struct esp_wifi_device *)dev->d_private;
  if (!priv)
    {
      return -EINVAL;
    }

  ret = esp_cmd_set_default_key(priv, key_index);
  if (ret < 0)
    {
      esp_err("Set default key failed: %d\n", ret);
      return ret;
    }

  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_start_ap
 *
 * Description:
 *   Start AP mode.
 *
 ****************************************************************************/

static int esp_cfg80211_start_ap(struct wiphy *wiphy,
                                 struct net_device *dev,
                                 struct cfg80211_ap_settings *settings)
{
  struct esp_wifi_device *priv;
  int ret;

  if (!wiphy || !dev || !settings)
    {
      return -EINVAL;
    }

  priv = (struct esp_wifi_device *)dev->d_private;
  if (!priv)
    {
      return -EINVAL;
    }

  ret = esp_cmd_start_ap(priv, settings);
  if (ret < 0)
    {
      esp_err("Start AP failed: %d\n", ret);
      return ret;
    }

  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_stop_ap
 *
 * Description:
 *   Stop AP mode.
 *
 ****************************************************************************/

static int esp_cfg80211_stop_ap(struct wiphy *wiphy,
                                struct net_device *dev)
{
  struct esp_wifi_device *priv;
  int ret;

  if (!wiphy || !dev)
    {
      return -EINVAL;
    }

  priv = (struct esp_wifi_device *)dev->d_private;
  if (!priv)
    {
      return -EINVAL;
    }

  ret = esp_cmd_stop_ap(priv);
  if (ret < 0)
    {
      esp_err("Stop AP failed: %d\n", ret);
      return ret;
    }

  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_change_beacon
 *
 * Description:
 *   Change beacon parameters.
 *
 ****************************************************************************/

static int esp_cfg80211_change_beacon(struct wiphy *wiphy,
                                       struct net_device *dev,
                                       struct cfg80211_beacon_settings *info)
{
  struct esp_wifi_device *priv;
  int ret;

  if (!wiphy || !dev || !info)
    {
      return -EINVAL;
    }

  priv = (struct esp_wifi_device *)dev->d_private;
  if (!priv)
    {
      return -EINVAL;
    }

  esp_info("Changing beacon for dev %s\n", dev->name);

  /* Forward beacon change to ESP32 via command */
  ret = esp_cmd_set_ie(priv, ESP_IE_BEACON_PROBE_HEAD,
                         info->beacon, info->beacon_len);
  if (ret < 0)
    {
      esp_err("Failed to set beacon head IE: %d\n", ret);
      return ret;
    }

  if (info->beacon_tail && info->beacon_tail_len > 0)
    {
      ret = esp_cmd_set_ie(priv, ESP_IE_BEACON_PROBE_TAIL,
                           info->beacon_tail, info->beacon_tail_len);
      if (ret < 0)
        {
          esp_err("Failed to set beacon tail IE: %d\n", ret);
          return ret;
        }
    }

  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_add_station
 *
 * Description:
 *   Add a station to AP.
 *
 ****************************************************************************/

static int esp_cfg80211_add_station(struct wiphy *wiphy,
                                  struct net_device *dev,
                                  const uint8_t *mac,
                                  struct station_parameters *params)
{
  struct esp_wifi_device *priv;
  int ret;

  if (!wiphy || !dev || !mac)
    {
      return -EINVAL;
    }

  priv = (struct esp_wifi_device *)dev->d_private;
  if (!priv)
    {
      return -EINVAL;
    }

  ret = esp_cmd_add_station(priv, mac, params);
  if (ret < 0)
    {
      esp_err("Add station failed: %d\n", ret);
      return ret;
    }

  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_del_station
 *
 * Description:
 *   Delete a station from AP.
 *
 ****************************************************************************/

static int esp_cfg80211_del_station(struct wiphy *wiphy,
                                  struct net_device *dev,
                                  struct station_del_parameters *params)
{
  struct esp_wifi_device *priv;
  int ret;

  if (!wiphy || !dev || !params)
    {
      return -EINVAL;
    }

  priv = (struct esp_wifi_device *)dev->d_private;
  if (!priv)
    {
      return -EINVAL;
    }

  ret = esp_cmd_del_station(priv, params->mac, params->reason_code);
  if (ret < 0)
    {
      esp_err("Delete station failed: %d\n", ret);
      return ret;
    }

  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_change_station
 *
 * Description:
 *   Change station parameters.
 *
 ****************************************************************************/

static int esp_cfg80211_change_station(struct wiphy *wiphy,
                                     struct net_device *dev,
                                     const uint8_t *mac,
                                     struct station_parameters *params)
{
  struct esp_wifi_device *priv;
  int ret;

  if (!wiphy || !dev || !mac || !params)
    {
      return -EINVAL;
    }

  priv = (struct esp_wifi_device *)dev->d_private;
  if (!priv)
    {
      return -EINVAL;
    }

  esp_info("Changing station params for %02x:%02x:%02x:%02x:%02x:%02x\n",
           mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);

  /* Forward station change to ESP32 via command */
  ret = esp_cmd_change_station(priv, mac, params);
  if (ret < 0)
    {
      esp_err("Change station failed: %d\n", ret);
      return ret;
    }

  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_get_station
 *
 * Description:
 *   Get station information.
 *
 ****************************************************************************/

static int esp_cfg80211_get_station(struct wiphy *wiphy,
                                  struct net_device *dev,
                                  const uint8_t *mac,
                                  struct station_info *sinfo)
{
  struct esp_wifi_device *priv;

  if (!wiphy || !dev || !mac || !sinfo)
    {
      return -EINVAL;
    }

  priv = (struct esp_wifi_device *)dev->d_private;
  if (!priv)
    {
      return -EINVAL;
    }

  esp_info("Getting station info for %02x:%02x:%02x:%02x:%02x:%02x\n",
           mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);

  /* Fill in station information */
  sinfo->filled = 0;

  /* Get RSSI from ESP32 */
  int rssi;
  if (esp_cmd_get_rssi(priv, &rssi) == 0)
    {
      sinfo->filled |= CFG80211_STA_INFO_SIGNAL;
      sinfo->signal = rssi;
    }

  /* Indicate signal average is the same as signal */
  if (sinfo->filled & CFG80211_STA_INFO_SIGNAL)
    {
      sinfo->filled |= CFG80211_STA_INFO_SIGNAL_AVG;
      sinfo->signal_avg = sinfo->signal;
    }

  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_set_tx_power
 *
 * Description:
 *   Set TX power.
 *
 ****************************************************************************/

static int esp_cfg80211_set_tx_power(struct wiphy *wiphy,
                                    struct wireless_dev *wdev,
                                    enum nl80211_tx_power_setting type,
                                    int mbm)
{
  struct esp_wifi_device *priv;

  if (!wiphy || !wdev)
    {
      return -EINVAL;
    }

  priv = container_of(wdev, struct esp_wifi_device, wdev);
  if (!priv)
    {
      return -EINVAL;
    }

  return esp_cmd_set_tx_power(priv, mbm / 100);  /* Convert mBm to dBm */
}

/****************************************************************************
 * Name: esp_cfg80211_get_tx_power
 *
 * Description:
 *   Get TX power.
 *
 ****************************************************************************/

static int esp_cfg80211_get_tx_power(struct wiphy *wiphy,
                                    struct wireless_dev *wdev,
                                    int *dbm)
{
  struct esp_wifi_device *priv;

  if (!wiphy || !wdev || !dbm)
    {
      return -EINVAL;
    }

  priv = container_of(wdev, struct esp_wifi_device, wdev);
  if (!priv)
    {
      return -EINVAL;
    }

  return esp_cmd_get_tx_power(priv);
}

/****************************************************************************
 * Name: esp_cfg80211_set_wiphy_params
 *
 * Description:
 *   Set wiphy parameters.
 *
 ****************************************************************************/

static int esp_cfg80211_set_wiphy_params(struct wiphy *wiphy,
                                          uint32_t changed)
{
  if (!wiphy)
    {
      return -EINVAL;
    }

  esp_info("Setting wiphy params, changed=0x%x\n", changed);

  /* Handle various wiphy parameter changes */
  if (changed & WIPHY_PARAM_RTS_THRESHOLD)
    {
      esp_info("RTS threshold: %d\n", wiphy->rts_threshold);
    }

  if (changed & WIPHY_PARAM_FRAG_THRESHOLD)
    {
      esp_info("Fragmentation threshold: %d\n", wiphy->frag_threshold);
    }

  if (changed & WIPHY_PARAM_RETRY_SHORT)
    {
      esp_info("Short retry limit: %d\n", wiphy->retry_short);
    }

  if (changed & WIPHY_PARAM_RETRY_LONG)
    {
      esp_info("Long retry limit: %d\n", wiphy->retry_long);
    }

  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_mgmt_tx
 *
 * Description:
 *   Transmit a management frame.
 *
 ****************************************************************************/

static int esp_cfg80211_mgmt_tx(struct wiphy *wiphy,
                                struct wireless_dev *wdev,
                                struct cfg80211_mgmt_tx_params *params,
                                uint64_t *cookie)
{
  struct esp_wifi_device *priv;
  int ret;

  if (!wiphy || !wdev || !params)
    {
      return -EINVAL;
    }

  priv = container_of(wdev, struct esp_wifi_device, wdev);
  if (!priv)
    {
      return -EINVAL;
    }

  /* Generate cookie for tracking */
  if (cookie)
    {
      static uint64_t mgmt_cookie = 0;
      *cookie = ++mgmt_cookie;
    }

  /* Send management frame via command */
  ret = esp_cmd_mgmt_tx(priv, params);
  if (ret < 0)
    {
      esp_err("Management TX failed: %d\n", ret);
      return ret;
    }

  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_set_bitrate_mask
 *
 * Description:
 *   Set bitrate mask.
 *
 ****************************************************************************/

static int esp_cfg80211_set_bitrate_mask(struct wiphy *wiphy,
                                      struct net_device *dev,
                                      const uint8_t *peer,
                                      struct cfg80211_bitrate_mask *mask)
{
  if (!wiphy || !dev || !mask)
    {
      return -EINVAL;
    }

  esp_info("Setting bitrate mask for dev %s\n", dev->name);

  /* For now, just acknowledge the request */
  /* ESP32 firmware would handle the actual rate limiting */
  /* This could be implemented by sending a vendor-specific command */

  return 0;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp_cfg80211_init
 *
 * Description:
 *   Initialize ESP32 cfg80211.
 *
 ****************************************************************************/

int esp_cfg80211_init(void)
{
  struct wiphy *wiphy;
  struct esp_adapter *adapter = &g_esp_adapter;
  int ret;

  esp_info("Initializing ESP32 cfg80211\n");

  /* Allocate wiphy */
  wiphy = wiphy_new(&g_esp_cfg80211_ops, sizeof(struct esp_adapter));
  if (!wiphy)
    {
      esp_err("Failed to allocate wiphy\n");
      return -ENOMEM;
    }

  /* Initialize wiphy */
  wiphy->interface_modes = BIT(NL80211_IFTYPE_STATION) | BIT(NL80211_IFTYPE_AP);
  wiphy->bands[IEEE80211_BAND_2GHZ] = &g_esp_wifi_bands_2ghz;
  wiphy->cipher_suites = g_esp_cipher_suites;
  wiphy->n_cipher_suites = sizeof(g_esp_cipher_suites) / sizeof(g_esp_cipher_suites[0]);
  wiphy->mgmt_stypes = g_esp_default_mgmt_stypes;
  wiphy->max_scan_ssids = 10;
  wiphy->max_scan_ie_len = 512;
  wiphy->signal_type = CFG80211_SIGNAL_TYPE_DBM;
  wiphy->flags = WIPHY_FLAG_REPORTS_OBSS;

  /* Set private data */
  adapter = wiphy_priv(wiphy);
  memset(adapter, 0, sizeof(struct esp_adapter));
  adapter->wiphy = wiphy;

  /* Initialize queues */
  sq_init(&adapter->cmd_free_queue);
  sq_init(&adapter->cmd_pending_queue);
  sq_init(&adapter->event_queue);

  /* Initialize semaphores */
  nxsem_init(&adapter->cmd_lock, 0, 1);
  nxsem_init(&adapter->cmd_resp_sem, 0, 0);
  nxsem_init(&adapter->event_lock, 0, 1);

  /* Initialize command pool */
  adapter->cmd_pool = kmm_malloc(ESP_NUM_OF_CMD_NODES * sizeof(struct esp_cmd_node));
  if (!adapter->cmd_pool)
    {
      esp_err("Failed to allocate command pool\n");
      wiphy_free(wiphy);
      return -ENOMEM;
    }

  memset(adapter->cmd_pool, 0, ESP_NUM_OF_CMD_NODES * sizeof(struct esp_cmd_node));

  /* Register wiphy */
  ret = wiphy_register(wiphy);
  if (ret < 0)
    {
      esp_err("Failed to register wiphy: %d\n", ret);
      kmm_free(adapter->cmd_pool);
      wiphy_free(wiphy);
      return ret;
    }

  esp_info("ESP32 cfg80211 initialized\n");
  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_deinit
 *
 * Description:
 *   Deinitialize ESP32 cfg80211.
 *
 ****************************************************************************/

void esp_cfg80211_deinit(void)
{
  struct esp_adapter *adapter = &g_esp_adapter;
  int i;

  esp_info("Deinitializing ESP32 cfg80211\n");

  /* Unregister wiphy */
  if (adapter->wiphy)
    {
      wiphy_unregister(adapter->wiphy);

      /* Free command pool */
      if (adapter->cmd_pool)
        {
          for (i = 0; i < ESP_NUM_OF_CMD_NODES; i++)
            {
                if (adapter->cmd_pool[i].cmd_buf)
                  {
                      kmm_free(adapter->cmd_pool[i].cmd_buf);
                  }
            }
          kmm_free(adapter->cmd_pool);
        }

      wiphy_free(adapter->wiphy);
      adapter->wiphy = NULL;
    }

  /* Destroy semaphores */
  nxsem_destroy(&adapter->cmd_lock);
  nxsem_destroy(&adapter->cmd_resp_sem);
  nxsem_destroy(&adapter->event_lock);

  memset(adapter, 0, sizeof(struct esp_adapter));

  esp_info("ESP32 cfg80211 deinitialized\n");
}

/****************************************************************************
 * Name: esp_get_adapter
 *
 * Description:
 *   Get esp_adapter pointer.
 *
 ****************************************************************************/

struct esp_adapter *esp_get_adapter(void)
{
  return &g_esp_adapter;
}