/****************************************************************************
 * drivers/wireless/esp32_wifi.c
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
#  define esp32_wlaninfo(format, ...)   _info(format, ##__VA_ARGS__)
#else
#  define esp32_wlaninfo(format, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_ERROR
#  define esp32_wlanerr(format, ...)    _err(format, ##__VA_ARGS__)
#else
#  define esp32_wlanerr(format, ...)
#endif

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct esp32_priv
{
  struct wireless_dev wdev;      /* Wireless device must be first */
  struct net_device *ndev;
  uint8_t mac_addr[6];
  bool connected;
  uint8_t ssid[32];
  size_t ssid_len;
  uint8_t bssid[6];
  int8_t rssi;
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int esp32_add_virtual_intf(struct wiphy *wiphy,
                                 const char *name,
                                 unsigned char name_assign_type,
                                 enum nl80211_iftype type,
                                 struct vif_params *params);
static int esp32_del_virtual_intf(struct wiphy *wiphy,
                                 struct wireless_dev *wdev);
static int esp32_scan(struct wiphy *wiphy,
                     struct cfg80211_scan_request *request);
static int esp32_connect(struct wiphy *wiphy,
                        struct net_device *dev,
                        struct cfg80211_connect_params *sme);
static int esp32_disconnect(struct wiphy *wiphy,
                           struct net_device *dev,
                           u16 reason_code);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static const struct cfg80211_ops g_esp32_cfg80211_ops =
{
  .add_virtual_intf = esp32_add_virtual_intf,
  .del_virtual_intf = esp32_del_virtual_intf,
  .scan = esp32_scan,
  .connect = esp32_connect,
  .disconnect = esp32_disconnect,
};

static struct wiphy *g_esp32_wiphy;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static int esp32_add_virtual_intf(struct wiphy *wiphy,
                                 const char *name,
                                 unsigned char name_assign_type,
                                 enum nl80211_iftype type,
                                 struct vif_params *params)
{
  struct esp32_priv *priv;
  struct net_device *ndev;
  int ret;

  esp32_wlaninfo("Adding interface: %s\n", name);

  /* Allocate private data */
  priv = kmm_zalloc(sizeof(struct esp32_priv));
  if (!priv)
    {
      esp32_wlanerr("Failed to allocate private data\n");
      return -ENOMEM;
    }

  /* Allocate network device */
  ndev = netdev_alloc(name, name_assign_type, sizeof(struct esp32_priv *));
  if (!ndev)
    {
      esp32_wlanerr("Failed to allocate netdev\n");
      kmm_free(priv);
      return -ENOMEM;
    }

  /* Initialize netdev */
  ndev->d_private = priv;
  ether_setup(ndev);
  eth_random_ethaddr(ndev->d_mac.ether.ether_addr_octet);

  /* Set up wireless device */
  struct wireless_dev *wdev = &priv->wdev;
  wdev->wiphy = wiphy;
  wdev->netdev = ndev;
  wdev->iftype = type;

  /* Store in private data */
  priv->wiphy = wiphy;
  priv->ndev = ndev;
  memcpy(priv->mac_addr, ndev->d_mac.ether.ether_addr_octet, 6);

  /* Register network device */
  ret = netdev_register(ndev);
  if (ret < 0)
    {
      esp32_wlanerr("Failed to register netdev: %d\n", ret);
      netdev_free(ndev);
      kmm_free(priv);
      return ret;
    }

  esp32_wlaninfo("Interface %s added successfully\n", name);
  return 0;
}

static int esp32_del_virtual_intf(struct wiphy *wiphy,
                                 struct wireless_dev *wdev)
{
  struct net_device *ndev = wdev->netdev;
  struct esp32_priv *priv = netdev_priv(ndev);

  esp32_wlaninfo("Deleting interface\n");

  /* Unregister and free netdev */
  netdev_unregister(ndev);
  netdev_free(ndev);
  kmm_free(priv);

  esp32_wlaninfo("Interface deleted\n");
  return 0;
}

static int esp32_scan(struct wiphy *wiphy,
                     struct cfg80211_scan_request *request)
{
  esp32_wlaninfo("Scan request\n");

  /* In a real implementation, this would trigger ESP32 scan */
  cfg80211_scan_done(request, false);

  esp32_wlaninfo("Scan completed\n");
  return 0;
}

static int esp32_connect(struct wiphy *wiphy,
                        struct net_device *dev,
                        struct cfg80211_connect_params *sme)
{
  struct esp32_priv *priv = netdev_priv(dev);
  int ret;

  esp32_wlaninfo("Connect request for SSID: %.*s\n", (int)sme->ssid_len, sme->ssid);

  if (!sme || !sme->ssid || sme->ssid_len == 0)
    {
      esp32_wlanerr("Invalid connection parameters\n");
      return -EINVAL;
    }

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

  /* In a real implementation, this would send connect command to ESP32 */
  priv->connected = true;
  priv->rssi = -50;  /* Simulated RSSI */

  /* Report connection result */
  ret = cfg80211_connect_result(dev, sme->bssid, sme->ie, sme->ie_len,
                               NULL, 0, 0, GFP_KERNEL);
  if (ret < 0)
    {
      esp32_wlanerr("Failed to report connection result: %d\n", ret);
      return ret;
    }

  return 0;
}

static int esp32_disconnect(struct wiphy *wiphy,
                           struct net_device *dev,
                           u16 reason_code)
{
  struct esp32_priv *priv = netdev_priv(dev);

  esp32_wlaninfo("Disconnect request, reason: %d\n", reason_code);

  /* In a real implementation, this would send disconnect command to ESP32 */
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

  /* Set max values */
  wiphy->max_scan_ssids = 4;
  wiphy->max_scan_ie_len = 2304;
  wiphy->signal_type = CFG80211_SIGNAL_TYPE_MBM;

  /* Register wiphy */
  ret = wiphy_register(wiphy);
  if (ret < 0)
    {
      esp32_wlanerr("Failed to register wiphy: %d\n", ret);
      wiphy_free(wiphy);
      return ret;
    }

  g_esp32_wiphy = wiphy;

  esp32_wlaninfo("ESP32 WiFi driver initialized\n");
  return 0;
}

int esp32_wifi_uninitialize(void)
{
  if (g_esp32_wiphy)
    {
      wiphy_unregister(g_esp32_wiphy);
      wiphy_free(g_esp32_wiphy);
      g_esp32_wiphy = NULL;
    }

  esp32_wlaninfo("ESP32 WiFi driver uninitialized\n");
  return 0;
}

#endif /* CONFIG_ESP32_WIFI */