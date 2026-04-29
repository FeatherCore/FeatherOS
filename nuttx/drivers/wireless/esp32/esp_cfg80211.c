/****************************************************************************
 * drivers/wireless/esp32/esp_cfg80211.c
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
#include <stdlib.h>
#include <string.h>
#include <semaphore.h>
#include <errno.h>
#include <debug.h>

#include <nuttx/kmalloc.h>
#include <nuttx/semaphore.h>
#include <nuttx/wqueue.h>
#include <nuttx/wireless/cfg80211.h>
#include <nuttx/wireless/nl80211.h>

#include "esp_cfg80211.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Debug output */

#ifdef CONFIG_DEBUG_WIRELESS_INFO
#  define esp_info(fmt, ...)  ninfo(fmt, ##__VA_ARGS__)
#else
#  define esp_info(fmt, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_ERROR
#  define esp_err(fmt, ...)   nerr(fmt, ##__VA_ARGS__)
#else
#  define esp_err(fmt, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_VERBOSE
#  define esp_verbose(fmt, ...) ninfo(fmt, ##__VA_ARGS__)
#else
#  define esp_verbose(fmt, ...)
#endif

#define esp_dbg esp_verbose

/* Default values */

#define ESP_DEFAULT_TX_POWER_DBM 15

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct esp_adapter g_esp_adapter;

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
g_esp_default_mgmt_stypes[NUM_NL80211_IFTYPES] =
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
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp_cfg80211_add_iface
 *
 * Description:
 *   Add a virtual interface to the ESP32 device.
 *
 ****************************************************************************/

static struct wireless_dev *esp_cfg80211_add_iface(FAR struct wiphy *wiphy,
                                                    FAR const char *name,
                                                    enum nl80211_iftype type,
                                                    FAR struct vif_params *params)
{
  FAR struct esp_device *esp_dev;
  FAR struct esp_wifi_device *esp_wdev;
  int if_type;
  int ret;

  if (!wiphy || !name)
    {
      esp_err("Invalid parameters\n");
      return NULL;
    }

  esp_dev = wiphy_priv(wiphy);
  if (!esp_dev || !esp_dev->adapter)
    {
      esp_err("Invalid esp device\n");
      return NULL;
    }

  /* Allocate and initialize esp_wifi_device structure */
  esp_wdev = kmm_zalloc(sizeof(struct esp_wifi_device));
  if (!esp_wdev)
    {
      esp_err("Failed to allocate esp_wifi_device\n");
      return NULL;
    }

  /* Determine interface type */
  if (type == NL80211_IFTYPE_STATION)
    {
      if_type = ESP_STA_IF;
    }
  else if (type == NL80211_IFTYPE_AP)
    {
      if_type = ESP_AP_IF;
    }
  else
    {
      esp_err("Unsupported interface type: %d\n", type);
      kmm_free(esp_wdev);
      return NULL;
    }

  /* Initialize esp_wifi_device */
  esp_wdev->wdev.wiphy = wiphy;
  esp_wdev->esp_dev = esp_dev;
  esp_wdev->adapter = esp_dev->adapter;
  esp_wdev->if_type = if_type;
  esp_wdev->if_num = (if_type == ESP_STA_IF) ? ESP_STA_NW_IF : ESP_AP_NW_IF;
  esp_wdev->tx_pwr = ESP_DEFAULT_TX_POWER_DBM;
  esp_wdev->tx_pwr_type = NL80211_TX_POWER_AUTOMATIC;

  /* Initialize the wireless device */
  esp_wdev->wdev.wiphy = wiphy;
  esp_wdev->wdev.iftype = type;
  esp_wdev->wdev.current_bss = NULL;

  /* Store in adapter */
  if (esp_wdev->if_num < ESP_MAX_INTERFACE)
    {
      esp_dev->adapter->priv[esp_wdev->if_num] = esp_wdev;
    }

  /* Initialize the interface in ESP */
  ret = esp_cmd_init_interface(esp_wdev);
  if (ret < 0)
    {
      esp_err("Failed to initialize interface: %d\n", ret);
      if (esp_wdev->if_num < ESP_MAX_INTERFACE)
        {
          esp_dev->adapter->priv[esp_wdev->if_num] = NULL;
        }
      kmm_free(esp_wdev);
      return NULL;
    }

  /* Get MAC address from ESP */
  ret = esp_cmd_get_mac(esp_wdev);
  if (ret < 0)
    {
      esp_err("Failed to get MAC address: %d\n", ret);
      esp_cmd_deinit_interface(esp_wdev);
      if (esp_wdev->if_num < ESP_MAX_INTERFACE)
        {
          esp_dev->adapter->priv[esp_wdev->if_num] = NULL;
        }
      kmm_free(esp_wdev);
      return NULL;
    }

  esp_info("Added interface %s type %d MAC %02x:%02x:%02x:%02x:%02x:%02x\n",
           name, type,
           esp_wdev->mac_address[0], esp_wdev->mac_address[1],
           esp_wdev->mac_address[2], esp_wdev->mac_address[3],
           esp_wdev->mac_address[4], esp_wdev->mac_address[5]);

  return &esp_wdev->wdev;
}

/****************************************************************************
 * Name: esp_cfg80211_change_iface
 *
 * Description:
 *   Change the type of an existing interface.
 *
 ****************************************************************************/

static int esp_cfg80211_change_iface(FAR struct wiphy *wiphy,
                                     FAR struct net_device *dev,
                                     enum nl80211_iftype type,
                                     FAR struct vif_params *params)
{
  FAR struct esp_wifi_device *priv;
  enum esp_wifi_mode esp_mode;
  int ret;

  if (!wiphy || !dev)
    {
      esp_err("Invalid parameters\n");
      return -EINVAL;
    }

  priv = dev->d_private;
  if (!priv)
    {
      esp_err("Invalid priv\n");
      return -EINVAL;
    }

  esp_info("Changing interface from %d to %d\n", priv->if_type, type);

  /* Determine ESP WiFi mode */
  if (type == NL80211_IFTYPE_STATION)
    {
      esp_mode = ESP_WIFI_MODE_STA;
    }
  else if (type == NL80211_IFTYPE_AP)
    {
      esp_mode = ESP_WIFI_MODE_AP;
    }
  else
    {
      esp_err("Unsupported interface type: %d\n", type);
      return -EOPNOTSUPP;
    }

  /* Set mode in ESP */
  ret = esp_cmd_set_mode(priv, esp_mode);
  if (ret < 0)
    {
      esp_err("Failed to set mode: %d\n", ret);
      return ret;
    }

  /* Update interface type */
  priv->if_type = (type == NL80211_IFTYPE_STATION) ? ESP_STA_IF : ESP_AP_IF;
  priv->wdev.iftype = type;

  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_scan
 *
 * Description:
 *   Start a scan operation.
 *
 ****************************************************************************/

static int esp_cfg80211_scan(FAR struct wiphy *wiphy,
                             FAR struct cfg80211_scan_request *request)
{
  FAR struct net_device *ndev;
  FAR struct esp_wifi_device *priv;
  int ret;

  if (!wiphy || !request || !request->wdev || !request->wdev->netdev)
    {
      esp_err("Invalid parameters\n");
      return -EINVAL;
    }

  ndev = request->wdev->netdev;
  priv = ndev->d_private;

  if (!priv)
    {
      esp_err("Invalid priv\n");
      return -EINVAL;
    }

  esp_info("Starting scan on interface %p\n", priv);

  /* Store request for later processing */
  priv->scan_request = request;
  priv->scan_in_progress = 1;

  /* Send scan request to ESP */
  ret = esp_cmd_scan_request(priv, request);
  if (ret < 0)
    {
      esp_err("Failed to send scan request: %d\n", ret);
      priv->scan_in_progress = 0;
      return ret;
    }

  return 0;
}

/****************************************************************************
 * Name: esp_cfg80211_connect
 *
 * Description:
 *   Connect to a BSS/ESS.
 *
 ****************************************************************************/

static int esp_cfg80211_connect(FAR struct wiphy *wiphy,
                                FAR struct net_device *dev,
                                FAR struct cfg80211_connect_params *sme)
{
  FAR struct esp_wifi_device *priv;
  int ret;

  if (!wiphy || !dev || !sme)
    {
      esp_err("Invalid parameters\n");
      return -EINVAL;
    }

  priv = dev->d_private;
  if (!priv)
    {
      esp_err("Invalid priv\n");
      return -EINVAL;
    }

  esp_info("Connecting to SSID: %.*s\n", (int)sme->ssid_len, sme->ssid);

  /* Send connect request to ESP */
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
 *   Disconnect from the BSS/ESS.
 *
 ****************************************************************************/

static int esp_cfg80211_disconnect(FAR struct wiphy *wiphy,
                                   FAR struct net_device *dev,
                                   uint16_t reason_code)
{
  FAR struct esp_wifi_device *priv;
  int ret;

  if (!wiphy || !dev)
    {
      esp_err("Invalid parameters\n");
      return -EINVAL;
    }

  priv = dev->d_private;
  if (!priv)
    {
      esp_err("Invalid priv\n");
      return -EINVAL;
    }

  esp_info("Disconnecting from network\n");

  /* Send disconnect request to ESP */
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
 *   Add a key to the device.
 *
 ****************************************************************************/

static int esp_cfg80211_add_key(FAR struct wiphy *wiphy,
                                FAR struct net_device *dev,
                                uint8_t key_index,
                                bool pairwise,
                                FAR const uint8_t *mac_addr,
                                FAR struct key_params *params)
{
  FAR struct esp_wifi_device *priv;

  if (!wiphy || !dev || !params)
    {
      esp_err("Invalid parameters\n");
      return -EINVAL;
    }

  priv = dev->d_private;
  if (!priv)
    {
      esp_err("Invalid priv\n");
      return -EINVAL;
    }

  esp_info("Adding key index: %d, pairwise: %d\n", key_index, pairwise);

  return esp_cmd_add_key(priv, key_index, pairwise, mac_addr, params);
}

/****************************************************************************
 * Name: esp_cfg80211_del_key
 *
 * Description:
 *   Delete a key from the device.
 *
 ****************************************************************************/

static int esp_cfg80211_del_key(FAR struct wiphy *wiphy,
                                FAR struct net_device *dev,
                                uint8_t key_index,
                                bool pairwise,
                                FAR const uint8_t *mac_addr)
{
  FAR struct esp_wifi_device *priv;

  if (!wiphy || !dev)
    {
      esp_err("Invalid parameters\n");
      return -EINVAL;
    }

  priv = dev->d_private;
  if (!priv)
    {
      esp_err("Invalid priv\n");
      return -EINVAL;
    }

  esp_info("Deleting key index: %d\n", key_index);

  return esp_cmd_del_key(priv, key_index, pairwise, mac_addr);
}

/****************************************************************************
 * Name: esp_cfg80211_set_default_key
 *
 * Description:
 *   Set the default key.
 *
 ****************************************************************************/

static int esp_cfg80211_set_default_key(FAR struct wiphy *wiphy,
                                        FAR struct net_device *dev,
                                        uint8_t key_index,
                                        bool unicast,
                                        bool multicast)
{
  FAR struct esp_wifi_device *priv;

  if (!wiphy || !dev)
    {
      esp_err("Invalid parameters\n");
      return -EINVAL;
    }

  priv = dev->d_private;
  if (!priv)
    {
      esp_err("Invalid priv\n");
      return -EINVAL;
    }

  esp_info("Setting default key index: %d\n", key_index);

  return esp_cmd_set_default_key(priv, key_index);
}

/****************************************************************************
 * Name: esp_cfg80211_start_ap
 *
 * Description:
 *   Start AP operation.
 *
 ****************************************************************************/

static int esp_cfg80211_start_ap(FAR struct wiphy *wiphy,
                                 FAR struct net_device *dev,
                                 FAR struct cfg80211_ap_settings *settings)
{
  FAR struct esp_wifi_device *priv;

  if (!wiphy || !dev || !settings)
    {
      esp_err("Invalid parameters\n");
      return -EINVAL;
    }

  priv = dev->d_private;
  if (!priv)
    {
      esp_err("Invalid priv\n");
      return -EINVAL;
    }

  esp_info("Starting AP mode\n");

  return esp_cmd_start_ap(priv, settings);
}

/****************************************************************************
 * Name: esp_cfg80211_stop_ap
 *
 * Description:
 *   Stop AP operation.
 *
 ****************************************************************************/

static int esp_cfg80211_stop_ap(FAR struct wiphy *wiphy,
                                FAR struct net_device *dev)
{
  FAR struct esp_wifi_device *priv;

  if (!wiphy || !dev)
    {
      esp_err("Invalid parameters\n");
      return -EINVAL;
    }

  priv = dev->d_private;
  if (!priv)
    {
      esp_err("Invalid priv\n");
      return -EINVAL;
    }

  esp_info("Stopping AP mode\n");

  return esp_cmd_stop_ap(priv);
}

/****************************************************************************
 * Name: esp_cfg80211_set_tx_power
 *
 * Description:
 *   Set TX power.
 *
 ****************************************************************************/

static int esp_cfg80211_set_tx_power(FAR struct wiphy *wiphy,
                                     FAR struct wireless_dev *wdev,
                                     enum nl80211_tx_power_setting type,
                                     int mbm)
{
  FAR struct esp_wifi_device *priv;

  if (!wiphy || !wdev)
    {
      esp_err("Invalid parameters\n");
      return -EINVAL;
    }

  priv = wdev->netdev->d_private;
  if (!priv)
    {
      esp_err("Invalid priv\n");
      return -EINVAL;
    }

  esp_info("Setting TX power type: %d, mbm: %d\n", type, mbm);

  switch (type)
    {
      case NL80211_TX_POWER_AUTOMATIC:
        priv->tx_pwr = ESP_DEFAULT_TX_POWER_DBM;
        break;
      case NL80211_TX_POWER_LIMITED:
        if (mbm > ESP_MAX_TX_POWER_DBM * 100 || mbm < ESP_MIN_TX_POWER_DBM * 100)
          {
            esp_err("TX power out of range: %d\n", mbm);
            return -EINVAL;
          }
        priv->tx_pwr = mbm / 100;  /* Convert mBm to dBm */
        break;
      case NL80211_TX_POWER_FIXED:
        esp_err("Fixed TX power not supported\n");
        return -EOPNOTSUPP;
      default:
        esp_err("Unknown TX power type: %d\n", type);
        return -EINVAL;
    }

  return esp_cmd_set_tx_power(priv, priv->tx_pwr);
}

/****************************************************************************
 * Name: esp_cfg80211_get_tx_power
 *
 * Description:
 *   Get TX power.
 *
 ****************************************************************************/

static int esp_cfg80211_get_tx_power(FAR struct wiphy *wiphy,
                                     FAR struct wireless_dev *wdev,
                                     FAR int *dbm)
{
  FAR struct esp_wifi_device *priv;

  if (!wiphy || !wdev || !dbm)
    {
      esp_err("Invalid parameters\n");
      return -EINVAL;
    }

  priv = wdev->netdev->d_private;
  if (!priv)
    {
      esp_err("Invalid priv\n");
      return -EINVAL;
    }

  /* Get current TX power from ESP */
  esp_cmd_get_tx_power(priv);

  *dbm = priv->tx_pwr;
  return 0;
}

/****************************************************************************
 * Name: esp_reg_notifier
 *
 * Description:
 *   Handle regulatory domain changes.
 *
 ****************************************************************************/

static void esp_reg_notifier(FAR struct wiphy *wiphy,
                             FAR struct regulatory_request *request)
{
  FAR struct esp_device *esp_dev;
  FAR struct esp_wifi_device *priv;

  if (!wiphy || !request)
    {
      esp_err("Invalid parameters\n");
      return;
    }

  esp_dev = wiphy_priv(wiphy);
  if (!esp_dev || !esp_dev->adapter)
    {
      esp_err("Invalid esp device\n");
      return;
    }

  priv = esp_dev->adapter->priv[0];  /* Use first interface */
  if (!priv)
    {
      esp_err("Invalid priv\n");
      return;
    }

  esp_info("Regulatory domain change: %c%c\n",
           request->alpha2[0], request->alpha2[1]);

  /* Update country code */
  if (strncmp(request->alpha2, priv->country_code, 2) != 0)
    {
      strlcpy(priv->country_code, request->alpha2, sizeof(priv->country_code));
      esp_cmd_set_reg_domain(priv);
    }
}

/* cfg80211 operations structure */

static const struct cfg80211_ops g_esp_cfg80211_ops =
{
  .add_virtual_intf = esp_cfg80211_add_iface,
  .change_virtual_intf = esp_cfg80211_change_iface,
  .scan = esp_cfg80211_scan,
  .connect = esp_cfg80211_connect,
  .disconnect = esp_cfg80211_disconnect,
  .add_key = esp_cfg80211_add_key,
  .del_key = esp_cfg80211_del_key,
  .set_default_key = esp_cfg80211_set_default_key,
  .start_ap = esp_cfg80211_start_ap,
  .stop_ap = esp_cfg80211_stop_ap,
  .set_tx_power = esp_cfg80211_set_tx_power,
  .get_tx_power = esp_cfg80211_get_tx_power,
  .reg_notifier = esp_reg_notifier,
};

/****************************************************************************
 * Name: esp_add_wiphy
 *
 * Description:
 *   Add wiphy device for ESP32.
 *
 ****************************************************************************/

static int esp_add_wiphy(FAR struct esp_adapter *adapter)
{
  FAR struct wiphy *wiphy;
  FAR struct esp_device *esp_dev;
  int ret;

  if (!adapter)
    {
      esp_err("Invalid adapter\n");
      return -EINVAL;
    }

  /* Allocate wiphy */
  wiphy = wiphy_new(&g_esp_cfg80211_ops, sizeof(struct esp_device));
  if (!wiphy)
    {
      esp_err("Failed to create wiphy\n");
      return -ENOMEM;
    }

  /* Get private data */
  esp_dev = wiphy_priv(wiphy);
  esp_dev->wiphy = wiphy;
  esp_dev->adapter = adapter;

  /* Configure wiphy */
  wiphy->interface_modes = BIT(NL80211_IFTYPE_STATION);
#ifdef CONFIG_ESP32_AP_MODE
  wiphy->interface_modes |= BIT(NL80211_IFTYPE_AP);
#endif

  /* Set supported bands */
  wiphy->bands[IEEE80211_BAND_2GHZ] = &g_esp_wifi_bands_2ghz;

  /* Set cipher suites */
  wiphy->cipher_suites = g_esp_cipher_suites;
  wiphy->n_cipher_suites = sizeof(g_esp_cipher_suites) / sizeof(g_esp_cipher_suites[0]);

  /* Set other capabilities */
  wiphy->max_scan_ssids = 10;
  wiphy->max_scan_ie_len = ESP_MAX_IE_LEN;
  wiphy->signal_type = CFG80211_SIGNAL_TYPE_MBM;
  wiphy->mgmt_stypes = g_esp_default_mgmt_stypes;

  /* Set features */
  wiphy->features |= NL80211_FEATURE_SAE;

  /* Set flags */
  wiphy->flags |= WIPHY_FLAG_REPORTS_OBSS;

  /* Set regulatory handler */
  wiphy->reg_notifier = esp_reg_notifier;

  /* Register wiphy */
  ret = wiphy_register(wiphy);
  if (ret < 0)
    {
      esp_err("Failed to register wiphy: %d\n", ret);
      wiphy_free(wiphy);
      return ret;
    }

  adapter->wiphy = wiphy;
  esp_info("Successfully added wiphy\n");

  return 0;
}

/****************************************************************************
 * Name: esp_remove_wiphy
 *
 * Description:
 *   Remove wiphy device.
 *
 ****************************************************************************/

static void esp_remove_wiphy(FAR struct esp_adapter *adapter)
{
  if (adapter && adapter->wiphy)
    {
      wiphy_unregister(adapter->wiphy);
      wiphy_free(adapter->wiphy);
      adapter->wiphy = NULL;
    }
}

/****************************************************************************
 * Name: esp_cmd_init_interface
 *
 * Description:
 *   Initialize ESP interface.
 *
 ****************************************************************************/

int esp_cmd_init_interface(FAR struct esp_wifi_device *priv)
{
  struct esp_command_header cmd_hdr;
  int ret;

  if (!priv)
    {
      return -EINVAL;
    }

  /* Prepare command header */
  cmd_hdr.cmd_code = ESP_CMD_INIT_INTERFACE;
  cmd_hdr.cmd_status = 0;
  cmd_hdr.len = sizeof(struct esp_command_header);
  cmd_hdr.seq_num = 0;  /* Will be filled by send function */
  cmd_hdr.reserved1 = 0;
  cmd_hdr.reserved2 = 0;

  /* Send command */
  ret = esp_send_command(priv, ESP_CMD_INIT_INTERFACE,
                         (FAR const uint8_t *)&cmd_hdr, sizeof(cmd_hdr));
  if (ret < 0)
    {
      esp_err("Failed to initialize interface: %d\n", ret);
      return ret;
    }

  return 0;
}

/****************************************************************************
 * Name: esp_cmd_deinit_interface
 *
 * Description:
 *   Deinitialize ESP interface.
 *
 ****************************************************************************/

int esp_cmd_deinit_interface(FAR struct esp_wifi_device *priv)
{
  struct esp_command_header cmd_hdr;
  int ret;

  if (!priv)
    {
      return -EINVAL;
    }

  /* Prepare command header */
  cmd_hdr.cmd_code = ESP_CMD_DEINIT_INTERFACE;
  cmd_hdr.cmd_status = 0;
  cmd_hdr.len = sizeof(struct esp_command_header);
  cmd_hdr.seq_num = 0;
  cmd_hdr.reserved1 = 0;
  cmd_hdr.reserved2 = 0;

  /* Send command */
  ret = esp_send_command(priv, ESP_CMD_DEINIT_INTERFACE,
                         (FAR const uint8_t *)&cmd_hdr, sizeof(cmd_hdr));
  if (ret < 0)
    {
      esp_err("Failed to deinitialize interface: %d\n", ret);
    }

  return 0;
}

/****************************************************************************
 * Name: esp_cmd_get_mac
 *
 * Description:
 *   Get MAC address from ESP.
 *
 ****************************************************************************/

int esp_cmd_get_mac(FAR struct esp_wifi_device *priv)
{
  struct esp_command_header cmd_hdr;
  int ret;

  if (!priv)
    {
      return -EINVAL;
    }

  /* Prepare command header */
  cmd_hdr.cmd_code = ESP_CMD_GET_MAC;
  cmd_hdr.cmd_status = 0;
  cmd_hdr.len = sizeof(struct esp_command_header);
  cmd_hdr.seq_num = 0;
  cmd_hdr.reserved1 = 0;
  cmd_hdr.reserved2 = 0;

  /* Send command */
  ret = esp_send_command(priv, ESP_CMD_GET_MAC,
                         (FAR const uint8_t *)&cmd_hdr, sizeof(cmd_hdr));
  if (ret < 0)
    {
      esp_err("Failed to get MAC: %d\n", ret);
      return ret;
    }

  return 0;
}

/****************************************************************************
 * Name: esp_cmd_set_mode
 *
 * Description:
 *   Set WiFi mode in ESP.
 *
 ****************************************************************************/

int esp_cmd_set_mode(FAR struct esp_wifi_device *priv, uint8_t mode)
{
  struct esp_set_mode_cmd cmd;
  int ret;

  if (!priv)
    {
      return -EINVAL;
    }

  /* Prepare command */
  cmd.header.cmd_code = ESP_CMD_SET_MODE;
  cmd.header.cmd_status = 0;
  cmd.header.len = sizeof(cmd);
  cmd.header.seq_num = 0;
  cmd.header.reserved1 = 0;
  cmd.header.reserved2 = 0;
  cmd.mode = mode;
  cmd.pad[0] = 0;
  cmd.pad[1] = 0;

  /* Send command */
  ret = esp_send_command(priv, ESP_CMD_SET_MODE,
                         (FAR const uint8_t *)&cmd, sizeof(cmd));
  if (ret < 0)
    {
      esp_err("Failed to set mode: %d\n", ret);
      return ret;
    }

  return 0;
}

/****************************************************************************
 * Name: esp_cmd_set_reg_domain
 *
 * Description:
 *   Set regulatory domain in ESP.
 *
 ****************************************************************************/

int esp_cmd_set_reg_domain(FAR struct esp_wifi_device *priv)
{
  struct esp_reg_domain_cmd cmd;
  int ret;

  if (!priv)
    {
      return -EINVAL;
    }

  /* Prepare command */
  cmd.header.cmd_code = ESP_CMD_SET_REG_DOMAIN;
  cmd.header.cmd_status = 0;
  cmd.header.len = sizeof(cmd);
  cmd.header.seq_num = 0;
  cmd.header.reserved1 = 0;
  cmd.header.reserved2 = 0;
  strlcpy(cmd.country_code, priv->country_code, sizeof(cmd.country_code));

  /* Send command */
  ret = esp_send_command(priv, ESP_CMD_SET_REG_DOMAIN,
                         (FAR const uint8_t *)&cmd, sizeof(cmd));
  if (ret < 0)
    {
      esp_err("Failed to set reg domain: %d\n", ret);
    }

  return ret;
}

/****************************************************************************
 * Name: esp_send_command
 *
 * Description:
 *   Send a command to ESP32.
 *
 ****************************************************************************/

int esp_send_command(FAR struct esp_wifi_device *priv, uint8_t cmd_code,
                     FAR const uint8_t *data, size_t len)
{
  FAR struct esp_adapter *adapter;
  struct esp_payload_header payload_hdr;
  uint8_t *buf;
  size_t total_len;
  int ret;

  if (!priv || !data || len == 0)
    {
      return -EINVAL;
    }

  adapter = priv->adapter;
  if (!adapter || !adapter->if_ops || !adapter->if_ops->send)
    {
      return -ENODEV;
    }

  /* Prepare payload header */
  payload_hdr.if_type = adapter->if_type;
  payload_hdr.if_num = priv->if_num;
  payload_hdr.flags = 0;
  payload_hdr.packet_type = ESP_PACKET_TYPE_COMMAND_REQUEST;
  payload_hdr.reserved1 = 0;
  payload_hdr.len = len;
  payload_hdr.offset = 0;
  payload_hdr.checksum = esp_compute_checksum((FAR const uint8_t *)&payload_hdr + 4, 
                                            sizeof(payload_hdr) - 4);
  payload_hdr.checksum += esp_compute_checksum(data, len);
  payload_hdr.reserved2 = 0;
  payload_hdr.reserved3 = 0;

  total_len = sizeof(payload_hdr) + len;
  buf = kmm_malloc(total_len);
  if (!buf)
    {
      return -ENOMEM;
    }

  /* Copy headers and data */
  memcpy(buf, &payload_hdr, sizeof(payload_hdr));
  memcpy(buf + sizeof(payload_hdr), data, len);

  /* Send to ESP */
  ret = adapter->if_ops->send(adapter, buf, total_len);

  kmm_free(buf);
  return ret;
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

/****************************************************************************
 * Name: esp_mbm_to_power
 *
 * Description:
 *   Convert milli-bel-milliwatts to power level.
 *
 ****************************************************************************/

int esp_mbm_to_power(int mbm)
{
  /* Convert mBm (milli bel milliwatt) to power level */
  return (mbm * 4) / 100;  /* Simplified conversion */
}

/****************************************************************************
 * Name: esp_power_to_dbm
 *
 * Description:
 *   Convert power level to dBm.
 *
 ****************************************************************************/

int esp_power_to_dbm(int power)
{
  return (power / 4);  /* Simplified conversion */
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
 * Name: esp_cfg80211_init
 *
 * Description:
 *   Initialize ESP32 cfg80211 driver.
 *
 ****************************************************************************/

int esp_cfg80211_init(FAR struct esp_if_ops *ops, int if_type)
{
  FAR struct esp_adapter *adapter = &g_esp_adapter;
  int ret;

  esp_info("Initializing ESP32 cfg80211 driver\n");

  /* Initialize adapter structure */
  memset(adapter, 0, sizeof(struct esp_adapter));
  adapter->if_type = if_type;
  adapter->if_ops = ops;

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
      goto err_out;
    }

  /* Initialize command nodes */
  for (int i = 0; i < ESP_NUM_OF_CMD_NODES; i++)
    {
      adapter->cmd_pool[i].cmd_buf = kmm_malloc(ESP_SIZE_OF_CMD_NODE);
      if (!adapter->cmd_pool[i].cmd_buf)
        {
          esp_err("Failed to allocate command buffer %d\n", i);
          goto err_free_pool;
        }
      sq_addlast(&adapter->cmd_pool[i].node, &adapter->cmd_free_queue);
    }

  /* Initialize interface */
  if (ops && ops->init)
    {
      ret = ops->init(adapter);
      if (ret < 0)
        {
          esp_err("Failed to initialize interface: %d\n", ret);
          goto err_free_pool;
        }
    }

  /* Add wiphy */
  ret = esp_add_wiphy(adapter);
  if (ret < 0)
    {
      esp_err("Failed to add wiphy: %d\n", ret);
      if (ops && ops->deinit)
        {
          ops->deinit(adapter);
        }
      goto err_free_pool;
    }

  esp_info("ESP32 cfg80211 driver initialized successfully\n");
  return OK;

err_free_pool:
  for (int i = 0; i < ESP_NUM_OF_CMD_NODES; i++)
    {
      if (adapter->cmd_pool[i].cmd_buf)
        {
          kmm_free(adapter->cmd_pool[i].cmd_buf);
        }
    }
  kmm_free(adapter->cmd_pool);

err_out:
  nxsem_destroy(&adapter->cmd_lock);
  nxsem_destroy(&adapter->cmd_resp_sem);
  nxsem_destroy(&adapter->event_lock);

  return ret;
}

/****************************************************************************
 * Name: esp_cfg80211_deinit
 *
 * Description:
 *   Deinitialize ESP32 cfg80211 driver.
 *
 ****************************************************************************/

int esp_cfg80211_deinit(void)
{
  FAR struct esp_adapter *adapter = &g_esp_adapter;

  esp_info("Deinitializing ESP32 cfg80211 driver\n");

  /* Remove wiphy */
  esp_remove_wiphy(adapter);

  /* Deinitialize interface */
  if (adapter->if_ops && adapter->if_ops->deinit)
    {
      adapter->if_ops->deinit(adapter);
    }

  /* Free command pool */
  if (adapter->cmd_pool)
    {
      for (int i = 0; i < ESP_NUM_OF_CMD_NODES; i++)
        {
          if (adapter->cmd_pool[i].cmd_buf)
            {
              kmm_free(adapter->cmd_pool[i].cmd_buf);
            }
        }
      kmm_free(adapter->cmd_pool);
    }

  /* Destroy semaphores */
  nxsem_destroy(&adapter->cmd_lock);
  nxsem_destroy(&adapter->cmd_resp_sem);
  nxsem_destroy(&adapter->event_lock);

  memset(adapter, 0, sizeof(struct esp_adapter));

  esp_info("ESP32 cfg80211 driver deinitialized\n");
  return OK;
}

/****************************************************************************
 * Name: esp_notify_scan_done
 *
 * Description:
 *   Notify cfg80211 that scan is complete.
 *
 ****************************************************************************/

void esp_notify_scan_done(FAR struct esp_wifi_device *priv, bool aborted)
{
  if (priv && priv->scan_request)
    {
      cfg80211_scan_done(priv->scan_request, aborted);
      priv->scan_request = NULL;
      priv->scan_in_progress = 0;
    }
}

/****************************************************************************
 * Name: esp_notify_connect_result
 *
 * Description:
 *   Notify cfg80211 of connection result.
 *
 ****************************************************************************/

void esp_notify_connect_result(FAR struct esp_wifi_device *priv,
                               FAR const uint8_t *bssid, int status)
{
  if (priv && priv->wdev.netdev)
    {
      cfg80211_connect_result(priv->wdev.netdev, bssid, NULL, 0, NULL, 0,
                              status, 0);
    }
}

/****************************************************************************
 * Name: esp_notify_disconnect
 *
 * Description:
 *   Notify cfg80211 of disconnection.
 *
 ****************************************************************************/

void esp_notify_disconnect(FAR struct esp_wifi_device *priv, uint16_t reason,
                           bool locally_generated)
{
  if (priv && priv->wdev.netdev)
    {
      cfg80211_disconnected(priv->wdev.netdev, reason, NULL, 0,
                            locally_generated, 0);
    }
}