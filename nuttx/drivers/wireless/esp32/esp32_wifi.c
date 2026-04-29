/****************************************************************************
 * drivers/wireless/esp32/esp32_wifi.c
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
#  define wlinfo(format, ...)   ninfo("esp32_wifi: " format, ##__VA_ARGS__)
#else
#  define wlinfo(format, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_ERROR
#  define wlerr(format, ...)    nerr("esp32_wifi: " format, ##__VA_ARGS__)
#else
#  define wlerr(format, ...)
#endif

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct esp32_wifi_priv_s
{
  struct wireless_dev wdev;
  struct net_device *ndev;
  uint8_t mac_addr[6];
  uint8_t iftype;
  bool connected;
  uint8_t ssid[ESP32_SSID_MAX_LEN];
  size_t ssid_len;
  uint8_t bssid[6];
  int8_t rssi;
};

static struct wiphy *g_esp32_wiphy;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp32_add_virtual_intf
 *
 * Description:
 *   Add a virtual interface.
 *
 ****************************************************************************/

static int esp32_add_virtual_intf(struct wiphy *wiphy,
                                 const char *name,
                                 unsigned char name_assign_type,
                                 enum nl80211_iftype type,
                                 struct vif_params *params)
{
  struct esp32_wifi_priv_s *priv;
  struct net_device *ndev;
  struct wireless_dev *wdev;
  int ret;

  wlinfo("Adding interface: %s, type: %d\n", name, type);

  /* Allocate private data */

  priv = kmm_zalloc(sizeof(struct esp32_wifi_priv_s));
  if (!priv)
    {
      wlerr("Failed to allocate private data\n");
      return -ENOMEM;
    }

  /* Allocate network device */

  ndev = netdev_alloc(name, name_assign_type, sizeof(struct esp32_wifi_priv_s *));
  if (!ndev)
    {
      wlerr("Failed to allocate netdev\n");
      kmm_free(priv);
      return -ENOMEM;
    }

  /* Initialize netdev */

  ndev->d_private = priv;
  ether_setup(ndev);
  eth_random_ethaddr(ndev->d_mac.ether.ether_addr_octet);

  /* Set up wireless device */

  priv->wdev.wiphy = wiphy;
  priv->wdev.netdev = ndev;
  priv->wdev.iftype = type;

  /* Store in private data */

  priv->ndev = ndev;
  priv->iftype = type;
  memcpy(priv->mac_addr, ndev->d_mac.ether.ether_addr_octet, 6);

  /* Register network device */

  ret = netdev_register(ndev);
  if (ret < 0)
    {
      wlerr("Failed to register netdev: %d\n", ret);
      netdev_free(ndev);
      kmm_free(priv);
      return ret;
    }

  wlinfo("Interface %s added successfully\n", name);
  return 0;
}

/****************************************************************************
 * Name: esp32_del_virtual_intf
 *
 * Description:
 *   Delete a virtual interface.
 *
 ****************************************************************************/

static int esp32_del_virtual_intf(struct wiphy *wiphy,
                                 struct wireless_dev *wdev)
{
  struct net_device *ndev = wdev->netdev;
  struct esp32_wifi_priv_s *priv = netdev_priv(ndev);

  wlinfo("Deleting interface\n");

  /* Unregister and free netdev */

  netdev_unregister(ndev);
  netdev_free(ndev);
  kmm_free(priv);

  wlinfo("Interface deleted\n");
  return 0;
}

/****************************************************************************
 * Name: esp32_scan
 *
 * Description:
 *   Perform a scan operation.
 *
 ****************************************************************************/

static int esp32_scan(struct wiphy *wiphy,
                     struct cfg80211_scan_request *request)
{
  wlinfo("Starting scan\n");

  /* In a real implementation, this would trigger a scan on the ESP32 chip
   * and report results using cfg80211_inform_bss() and cfg80211_scan_done().
   *
   * For simulation, we'll just complete the scan immediately.
   */

  cfg80211_scan_done(request, false);
  return 0;
}

/****************************************************************************
 * Name: esp32_connect
 *
 * Description:
 *   Connect to a network.
 *
 ****************************************************************************/

static int esp32_connect(struct wiphy *wiphy,
                        struct net_device *dev,
                        struct cfg80211_connect_params *sme)
{
  struct esp32_wifi_priv_s *priv = netdev_priv(dev);
  int ret;

  if (!sme || !sme->ssid || !sme->ssid_len)
    {
      wlerr("Invalid connect parameters\n");
      return -EINVAL;
    }

  wlinfo("Connecting to SSID: %.*s\n", (int)sme->ssid_len, sme->ssid);

  /* Store connection parameters */

  if (sme->ssid_len > sizeof(priv->ssid))
    {
      return -EINVAL;
    }

  memcpy(priv->ssid, sme->ssid, sme->ssid_len);
  priv->ssid_len = sme->ssid_len;

  if (sme->bssid)
    {
      memcpy(priv->bssid, sme->bssid, 6);
    }

  /* In a real implementation, this would send connection command to ESP32 */

  /* Simulate successful connection */
  priv->connected = true;
  priv->rssi = -50;  /* Simulated RSSI */

  /* Report connection result */
  cfg80211_connect_result(dev, sme->bssid, sme->ie, sme->ie_len,
                         NULL, 0, 0, GFP_KERNEL);

  return 0;
}

/****************************************************************************
 * Name: esp32_disconnect
 *
 * Description:
 *   Disconnect from network.
 *
 ****************************************************************************/

static int esp32_disconnect(struct wiphy *wiphy,
                           struct net_device *dev,
                           u16 reason_code)
{
  struct esp32_wifi_priv_s *priv = netdev_priv(dev);

  wlinfo("Disconnecting, reason: %d\n", reason_code);

  /* In a real implementation, this would send disconnect command to ESP32 */

  /* Clear connection state */
  priv->connected = false;
  memset(priv->ssid, 0, sizeof(priv->ssid));
  priv->ssid_len = 0;

  /* Report disconnection */
  cfg80211_disconnected(dev, reason_code, NULL, 0, true, GFP_KERNEL);
  return 0;
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
  struct ieee80211_channel *channels;
  struct ieee80211_rate *rates;
  struct wiphy *wiphy;
  int ret;

  wlinfo("Initializing ESP32 WiFi driver\n");

  /* Allocate wiphy */

  wiphy = wiphy_new(&g_esp32_cfg80211_ops, 0);  /* No private data for now */
  if (!wiphy)
    {
      wlerr("Failed to allocate wiphy\n");
      return -ENOMEM;
    }

  /* Set up supported bands */

  /* Allocate channels */
  channels = kmm_zalloc(14 * sizeof(struct ieee80211_channel));
  if (!channels)
    {
      wlerr("Failed to allocate channels\n");
      wiphy_free(wiphy);
      return -ENOMEM;
    }

  /* Initialize 2.4GHz channels */
  for (int i = 0; i < 14; i++)
    {
      channels[i].band = IEEE80211_BAND_2GHZ;
      channels[i].center_freq = 2412 + i * 5;  /* Channels 1-14 */
      channels[i].hw_value = i + 1;
      channels[i].max_power = 20;  /* dBm */
    }

  /* Allocate rates */
  rates = kmm_zalloc(12 * sizeof(struct ieee80211_rate));
  if (!rates)
    {
      wlerr("Failed to allocate rates\n");
      kmm_free(channels);
      wiphy_free(wiphy);
      return -ENOMEM;
    }

  /* Initialize 2.4GHz rates */
  rates[0] = (struct ieee80211_rate){ .bitrate = 10, .hw_value = 0x00 };
  rates[1] = (struct ieee80211_rate){ .bitrate = 20, .hw_value = 0x01 };
  rates[2] = (struct ieee80211_rate){ .bitrate = 55, .hw_value = 0x02 };
  rates[3] = (struct ieee80211_rate){ .bitrate = 110, .hw_value = 0x03 };
  rates[4] = (struct ieee80211_rate){ .bitrate = 60, .hw_value = 0x0B };
  rates[5] = (struct ieee80211_rate){ .bitrate = 90, .hw_value = 0x0F };
  rates[6] = (struct ieee80211_rate){ .bitrate = 120, .hw_value = 0x0A };
  rates[7] = (struct ieee80211_rate){ .bitrate = 180, .hw_value = 0x0E };
  rates[8] = (struct ieee80211_rate){ .bitrate = 240, .hw_value = 0x09 };
  rates[9] = (struct ieee80211_rate){ .bitrate = 360, .hw_value = 0x0D };
  rates[10] = (struct ieee80211_rate){ .bitrate = 480, .hw_value = 0x08 };
  rates[11] = (struct ieee80211_rate){ .bitrate = 540, .hw_value = 0x0C };

  /* Set up 2.4GHz band */
  wiphy->bands[IEEE80211_BAND_2GHZ] = kmm_zalloc(sizeof(struct ieee80211_supported_band));
  if (!wiphy->bands[IEEE80211_BAND_2GHZ])
    {
      wlerr("Failed to allocate supported band\n");
      kmm_free(rates);
      kmm_free(channels);
      wiphy_free(wiphy);
      return -ENOMEM;
    }

  wiphy->bands[IEEE80211_BAND_2GHZ]->band = IEEE80211_BAND_2GHZ;
  wiphy->bands[IEEE80211_BAND_2GHZ]->channels = channels;
  wiphy->bands[IEEE80211_BAND_2GHZ]->n_channels = 14;
  wiphy->bands[IEEE80211_BAND_2GHZ]->bitrates = rates;
  wiphy->bands[IEEE80211_BAND_2GHZ]->n_bitrates = 12;

  /* Set interface modes */
  wiphy->interface_modes = BIT(NL80211_IFTYPE_STATION);
#ifdef CONFIG_ESP32_AP_MODE
  wiphy->interface_modes |= BIT(NL80211_IFTYPE_AP);
#endif

  /* Set other capabilities */
  wiphy->max_scan_ssids = 4;
  wiphy->max_scan_ie_len = 2304;
  wiphy->signal_type = CFG80211_SIGNAL_TYPE_MBM;

  /* Register wiphy */
  ret = wiphy_register(wiphy);
  if (ret < 0)
    {
      wlerr("Failed to register wiphy: %d\n", ret);
  if (wiphy->bands[IEEE80211_BAND_2GHZ])
    {
      if (wiphy->bands[IEEE80211_BAND_2GHZ]->channels)
        {
          kmm_free(wiphy->bands[IEEE80211_BAND_2GHZ]->channels);
        }
      if (wiphy->bands[IEEE80211_BAND_2GHZ]->bitrates)
        {
          kmm_free(wiphy->bands[IEEE80211_BAND_2GHZ]->bitrates);
        }
      kmm_free(wiphy->bands[IEEE80211_BAND_2GHZ]);
    }
      wiphy_free(wiphy);
      return ret;
    }

  /* Initialize 2.4GHz channels */
  for (int i = 0; i < 14; i++)
    {
      channels[i].band = IEEE80211_BAND_2GHZ;
      channels[i].center_freq = 2412 + i * 5;  /* Channels 1-14 */
      channels[i].hw_value = i + 1;
      channels[i].max_power = 20;  /* dBm */
    }

  /* Allocate rates */
  rates = kmm_zalloc(12 * sizeof(struct ieee80211_rate));
  if (!rates)
    {
      wlerr("Failed to allocate rates\n");
      kmm_free(channels);
      wiphy_free(wiphy);
      return -ENOMEM;
    }

  /* Initialize 2.4GHz rates */
  rates[0] = (struct ieee80211_rate){ .bitrate = 10, .hw_value = 0x00 };
  rates[1] = (struct ieee80211_rate){ .bitrate = 20, .hw_value = 0x01 };
  rates[2] = (struct ieee80211_rate){ .bitrate = 55, .hw_value = 0x02 };
  rates[3] = (struct ieee80211_rate){ .bitrate = 110, .hw_value = 0x03 };
  rates[4] = (struct ieee80211_rate){ .bitrate = 60, .hw_value = 0x0B };
  rates[5] = (struct ieee80211_rate){ .bitrate = 90, .hw_value = 0x0F };
  rates[6] = (struct ieee80211_rate){ .bitrate = 120, .hw_value = 0x0A };
  rates[7] = (struct ieee80211_rate){ .bitrate = 180, .hw_value = 0x0E };
  rates[8] = (struct ieee80211_rate){ .bitrate = 240, .hw_value = 0x09 };
  rates[9] = (struct ieee80211_rate){ .bitrate = 360, .hw_value = 0x0D };
  rates[10] = (struct ieee80211_rate){ .bitrate = 480, .hw_value = 0x08 };
  rates[11] = (struct ieee80211_rate){ .bitrate = 540, .hw_value = 0x0C };

  /* Set up 2.4GHz band */
  wiphy->bands[IEEE80211_BAND_2GHZ] = kmm_zalloc(sizeof(struct ieee80211_supported_band));
  if (!wiphy->bands[IEEE80211_BAND_2GHZ])
    {
      wlerr("Failed to allocate supported band\n");
      kmm_free(rates);
      kmm_free(channels);
      wiphy_free(wiphy);
      return -ENOMEM;
    }

  wiphy->bands[IEEE80211_BAND_2GHZ]->band = IEEE80211_BAND_2GHZ;
  wiphy->bands[IEEE80211_BAND_2GHZ]->channels = channels;
  wiphy->bands[IEEE80211_BAND_2GHZ]->n_channels = 14;
  wiphy->bands[IEEE80211_BAND_2GHZ]->bitrates = rates;
  wiphy->bands[IEEE80211_BAND_2GHZ]->n_bitrates = 12;

  /* Set interface modes */
  wiphy->interface_modes = BIT(NL80211_IFTYPE_STATION);
#ifdef CONFIG_ESP32_AP_MODE
  wiphy->interface_modes |= BIT(NL80211_IFTYPE_AP);
#endif

  /* Set other capabilities */
  wiphy->max_scan_ssids = 4;
  wiphy->max_scan_ie_len = 2304;
  wiphy->signal_type = CFG80211_SIGNAL_TYPE_MBM;

  /* Register wiphy */
  ret = wiphy_register(wiphy);
  if (ret < 0)
    {
      wlerr("Failed to register wiphy: %d\n", ret);
      kmm_free(wiphy->bands[IEEE80211_BAND_2GHZ]);
      kmm_free(rates);
      kmm_free(channels);
      wiphy_free(wiphy);
      return ret;
    }

  g_esp32_wiphy = wiphy;
  wlinfo("ESP32 WiFi driver initialized\n");
  return OK;
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
  struct wiphy *wiphy = g_esp32_wiphy;

  if (wiphy)
    {
      wiphy_unregister(wiphy);
      wiphy_free(wiphy);
      g_esp32_wiphy = NULL;
    }

  return OK;
}

#endif /* CONFIG_ESP32_WIFI */