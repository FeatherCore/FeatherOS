/****************************************************************************
 * drivers/wireless/esp32/esp32_wifi_main.c
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
#include <nuttx/wireless/cfg80211.h>
#include <nuttx/wireless/nl80211.h>
#include <nuttx/wireless/esp32_wifi.h>

#ifdef CONFIG_ESP32_WIFI

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#ifdef CONFIG_DEBUG_WIRELESS_INFO
#  define esp32_wlaninfo(format, ...)   ninfo("esp32: " format, ##__VA_ARGS__)
#else
#  define esp32_wlaninfo(format, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_ERROR
#  define esp32_wlanerr(format, ...)    nerr("esp32: " format, ##__VA_ARGS__)
#else
#  define esp32_wlanerr(format, ...)
#endif

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct wiphy *g_esp32_wiphy;

/* Supported channels (2.4GHz) */

static struct ieee80211_channel g_esp32_channels_2ghz[] =
{
  { .center_freq = 2412, .hw_value = 1, .max_power = 20 },  /* Channel 1 */
  { .center_freq = 2417, .hw_value = 2, .max_power = 20 },  /* Channel 2 */
  { .center_freq = 2422, .hw_value = 3, .max_power = 20 },  /* Channel 3 */
  { .center_freq = 2427, .hw_value = 4, .max_power = 20 },  /* Channel 4 */
  { .center_freq = 2432, .hw_value = 5, .max_power = 20 },  /* Channel 5 */
  { .center_freq = 2437, .hw_value = 6, .max_power = 20 },  /* Channel 6 */
  { .center_freq = 2442, .hw_value = 7, .max_power = 20 },  /* Channel 7 */
  { .center_freq = 2447, .hw_value = 8, .max_power = 20 },  /* Channel 8 */
  { .center_freq = 2452, .hw_value = 9, .max_power = 20 },  /* Channel 9 */
  { .center_freq = 2457, .hw_value = 10, .max_power = 20 }, /* Channel 10 */
  { .center_freq = 2462, .hw_value = 11, .max_power = 20 }, /* Channel 11 */
  { .center_freq = 2467, .hw_value = 12, .max_power = 20 }, /* Channel 12 */
  { .center_freq = 2472, .hw_value = 13, .max_power = 20 }, /* Channel 13 */
  { .center_freq = 2484, .hw_value = 14, .max_power = 20 }, /* Channel 14 */
};

/* Supported rates (2.4GHz) */

static struct ieee80211_rate g_esp32_rates_2ghz[] =
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

/* 2.4GHz band */

static struct ieee80211_supported_band g_esp32_band_2ghz =
{
  .band = IEEE80211_BAND_2GHZ,
  .channels = g_esp32_channels_2ghz,
  .n_channels = ARRAY_SIZE(g_esp32_channels_2ghz),
  .bitrates = g_esp32_rates_2ghz,
  .n_bitrates = ARRAY_SIZE(g_esp32_rates_2ghz),
  .ht_cap.ht_supported = true,
  .ht_cap.cap = IEEE80211_HT_CAP_SUP_WIDTH_20_40 |
                IEEE80211_HT_CAP_SGI_20 |
                IEEE80211_HT_CAP_RX_STBC |
                IEEE80211_HT_CAP_DSSSCCK40,
  .ht_cap.ampdu_factor = IEEE80211_HT_MAX_AMPDU_64K,
  .ht_cap.ampdu_density = IEEE80211_HT_MPDU_DENSITY_16,
  .ht_cap.mcs.rx_mask = { 0xff, 0, 0, 0, 0, 0, 0, 0, 0, 0 },
  .ht_cap.mcs.tx_params = IEEE80211_HT_MCS_TX_DEFINED,
};

/* Cipher suites */

static const u32 g_esp32_cipher_suites[] =
{
  WLAN_CIPHER_SUITE_WEP40,
  WLAN_CIPHER_SUITE_WEP104,
  WLAN_CIPHER_SUITE_TKIP,
  WLAN_CIPHER_SUITE_CCMP,
};

/* Management frame types */

static const struct ieee80211_txrx_stypes
g_esp32_default_mgmt_stypes[NUM_NL80211_IFTYPES] =
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

/* cfg80211 operations */

static int esp32_add_iface(struct wiphy *wiphy,
                          const char *name,
                          unsigned char name_assign_type,
                          enum nl80211_iftype type,
                          struct vif_params *params);
static int esp32_del_iface(struct wiphy *wiphy,
                          struct wireless_dev *wdev);
static int esp32_scan(struct wiphy *wiphy,
                     struct cfg80211_scan_request *request);
static int esp32_connect(struct wiphy *wiphy,
                        struct net_device *dev,
                        struct cfg80211_connect_params *sme);
static int esp32_disconnect(struct wiphy *wiphy,
                           struct net_device *dev,
                           u16 reason_code);

static const struct cfg80211_ops g_esp32_cfg80211_ops =
{
  .add_virtual_intf = esp32_add_iface,
  .del_virtual_intf = esp32_del_iface,
  .scan = esp32_scan,
  .connect = esp32_connect,
  .disconnect = esp32_disconnect,
};

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static int esp32_add_iface(struct wiphy *wiphy,
                          const char *name,
                          unsigned char name_assign_type,
                          enum nl80211_iftype type,
                          struct vif_params *params)
{
  struct wireless_dev *wdev;
  struct net_device *ndev;
  int ret;

  esp32_wlaninfo("Adding interface: %s, type: %d\n", name, type);

  /* Allocate wireless device */

  wdev = kmm_zalloc(sizeof(struct wireless_dev));
  if (!wdev)
    {
      esp32_wlanerr("Failed to allocate wireless device\n");
      return -ENOMEM;
    }

  /* Allocate network device */

  ndev = netdev_alloc(name, name_assign_type, 0);
  if (!ndev)
    {
      esp32_wlanerr("Failed to allocate network device\n");
      kmm_free(wdev);
      return -ENOMEM;
    }

  /* Initialize wireless device */

  wdev->wiphy = wiphy;
  wdev->netdev = ndev;
  wdev->iftype = type;

  /* Initialize network device */

  ether_setup(ndev);
  eth_random_ethaddr(ndev->d_mac.ether.ether_addr_octet);

  /* Set up device operations */

  ndev->d_private = wdev;
  ndev->d_ifup = NULL;   /* Set by upper layers */
  ndev->d_ifdown = NULL;
  ndev->d_txavail = NULL;
  ndev->d_txmit = NULL;
  ndev->d_pktsize = 1514;  /* Standard Ethernet frame size */
  ndev->d_llhdrlen = 14;  /* Ethernet header length */

  /* Register network device */

  ret = netdev_register(ndev);
  if (ret < 0)
    {
      esp32_wlanerr("Failed to register network device: %d\n", ret);
      netdev_free(ndev);
      kmm_free(wdev);
      return ret;
    }

  esp32_wlaninfo("Interface %s added successfully\n", name);
  return 0;
}

static int esp32_del_iface(struct wiphy *wiphy,
                          struct wireless_dev *wdev)
{
  struct net_device *ndev = wdev->netdev;

  esp32_wlaninfo("Deleting interface\n");

  /* Unregister network device */

  netdev_unregister(ndev);
  netdev_free(ndev);

  /* Free wireless device */

  kmm_free(wdev);

  esp32_wlaninfo("Interface deleted\n");
  return 0;
}

static int esp32_scan(struct wiphy *wiphy,
                     struct cfg80211_scan_request *request)
{
  esp32_wlaninfo("Scan request received\n");

  /* In a real implementation, this would send scan command to ESP32.
   * For now, just complete the scan immediately.
   */

  cfg80211_scan_done(request, false);

  esp32_wlaninfo("Scan completed\n");
  return 0;
}

static int esp32_connect(struct wiphy *wiphy,
                        struct net_device *dev,
                        struct cfg80211_connect_params *sme)
{
  esp32_wlaninfo("Connect request for SSID: %.*s\n",
                 (int)sme->ssid_len, sme->ssid);

  if (!sme || !sme->ssid || sme->ssid_len == 0)
    {
      esp32_wlanerr("Invalid connection parameters\n");
      return -EINVAL;
    }

  /* In a real implementation, this would send connect command to ESP32.
   * For now, just report successful connection.
   */

  cfg80211_connect_result(dev, sme->bssid, sme->ie, sme->ie_len,
                         NULL, 0, 0, GFP_KERNEL);

  esp32_wlaninfo("Connection completed\n");
  return 0;
}

static int esp32_disconnect(struct wiphy *wiphy,
                           struct net_device *dev,
                           u16 reason_code)
{
  esp32_wlaninfo("Disconnect request, reason: %d\n", reason_code);

  /* In a real implementation, this would send disconnect command to ESP32.
   * For now, just report disconnection.
   */

  cfg80211_disconnected(dev, reason_code, NULL, 0, true, GFP_KERNEL);

  esp32_wlaninfo("Disconnection completed\n");
  return 0;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int esp32_wifi_initialize(void)
{
  struct wiphy *wiphy;
  int ret;

  esp32_wlaninfo("Initializing ESP32 WiFi driver\n");

  /* Allocate wiphy */

  wiphy = wiphy_new(&g_esp32_cfg80211_ops, 0);
  if (!wiphy)
    {
      esp32_wlanerr("Failed to allocate wiphy\n");
      return -ENOMEM;
    }

  /* Set up wiphy capabilities */

  wiphy->interface_modes = BIT(NL80211_IFTYPE_STATION);
#ifdef CONFIG_ESP32_AP_MODE
  wiphy->interface_modes |= BIT(NL80211_IFTYPE_AP);
#endif

  /* Set supported bands */

  wiphy->bands[IEEE80211_BAND_2GHZ] = &g_esp32_band_2ghz;

  /* Set cipher suites */

  wiphy->cipher_suites = g_esp32_cipher_suites;
  wiphy->n_cipher_suites = ARRAY_SIZE(g_esp32_cipher_suites);

  /* Set capabilities */

  wiphy->max_scan_ssids = 4;
  wiphy->max_scan_ie_len = 2304;
  wiphy->signal_type = CFG80211_SIGNAL_TYPE_MBM;
  wiphy->mgmt_stypes = g_esp32_default_mgmt_stypes;

  /* Features */

  wiphy->flags |= WIPHY_FLAG_HAS_REMAIN_ON_CHANNEL |
                  WIPHY_FLAG_OFFCHAN_TX |
                  WIPHY_FLAG_HAVE_AP_SME;

  /* Register wiphy */

  ret = wiphy_register(wiphy);
  if (ret < 0)
    {
      esp32_wlanerr("Failed to register wiphy: %d\n", ret);
      wiphy_free(wiphy);
      return ret;
    }

  g_esp32_wiphy = wiphy;

  esp32_wlaninfo("ESP32 WiFi driver initialized successfully\n");
  return 0;
}

int esp32_wifi_uninitialize(void)
{
  struct wiphy *wiphy = g_esp32_wiphy;

  if (wiphy)
    {
      wiphy_unregister(wiphy);
      wiphy_free(wiphy);
      g_esp32_wiphy = NULL;
    }

  esp32_wlaninfo("ESP32 WiFi driver uninitialized\n");
  return 0;
}

#endif /* CONFIG_ESP32_WIFI */