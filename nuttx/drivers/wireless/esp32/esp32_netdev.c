/****************************************************************************
 * drivers/wireless/esp32/esp32_netdev.c
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
 * ESP32 WiFi Network Device Implementation
 *
 * This module implements the network device interface for ESP32 WiFi,
 * handling TX/RX data packets and connecting to the NuttX network stack.
 ****************************************************************************/

#include <nuttx/config.h>

#include <sys/types.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include <errno.h>
#include <debug.h>

#include <nuttx/kmalloc.h>
#include <nuttx/semaphore.h>
#include <nuttx/net/net.h>
#include <nuttx/wireless/cfg80211.h>

#include "esp_cfg80211.h"
#include "esp_host_if.h"

#ifdef CONFIG_ESP32_WIFI

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#ifdef CONFIG_DEBUG_WIRELESS_INFO
#  define netdev_info(format, ...)  ninfo(format, ##__VA_ARGS__)
#else
#  define netdev_info(format, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_ERROR
#  define netdev_err(format, ...)   nerr(format, ##__VA_ARGS__)
#else
#  define netdev_err(format, ...)
#endif

#define ESP_NETDEV_MTU                 1500
#define ESP_NETDEV_TX_TIMEOUT_MS       5000
#define ESP_MAX_TX_QUEUE_LEN           100

#define SKB_DATA_ADDR_ALIGNMENT        4
#define INTERFACE_HEADER_PADDING       (SKB_DATA_ADDR_ALIGNMENT * 3)

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct esp_skb
{
  uint8_t *data;
  size_t len;
  size_t data_len;
  uint8_t *head;
  uint8_t *tail;
  uint8_t *end;
  struct esp_wifi_device *priv;
};

struct esp_netdev_priv_s
{
  struct net_device *netdev;
  struct esp_wifi_device *wifi_dev;
  bool initialized;
  sem_t tx_lock;
  sem_t rx_lock;
  uint32_t tx_queue_len;
  uint32_t tx_dropped;
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int esp32_netdev_ifup(struct net_device *dev);
static int esp32_netdev_ifdown(struct net_device *dev);
static int esp32_netdev_tx(struct net_device *dev, void *pkt, size_t len);
static void esp32_netdev_txpoll(struct net_driver_s *dev);
static int esp32_netdev_ioctl(struct net_device *dev, int cmd, unsigned long arg);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static const struct net_driver_ops_s g_esp32_netdev_ops =
{
  .ndo_ifup    = esp32_netdev_ifup,
  .ndo_ifdown  = esp32_netdev_ifdown,
  .ndo_tx      = esp32_netdev_tx,
  .ndo_txpoll  = esp32_netdev_txpoll,
  .ndo_ioctl   = esp32_netdev_ioctl,
};

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static struct esp_skb *esp_alloc_skb(struct esp_wifi_device *priv, size_t len)
{
  struct esp_skb *skb;
  size_t alloc_len;
  uint8_t offset;

  alloc_len = len + INTERFACE_HEADER_PADDING;

  skb = kmm_malloc(sizeof(struct esp_skb));
  if (!skb)
    {
      return NULL;
    }

  skb->head = kmm_malloc(alloc_len);
  if (!skb->head)
    {
      kmm_free(skb);
      return NULL;
    }

  skb->data = skb->head;
  skb->end = skb->head + alloc_len;
  skb->tail = skb->head;
  skb->len = 0;
  skb->data_len = 0;
  skb->priv = priv;

  offset = ((uintptr_t)skb->data) & (SKB_DATA_ADDR_ALIGNMENT - 1);
  if (offset)
    {
      skb_reserve(skb, INTERFACE_HEADER_PADDING - offset);
    }

  return skb;
}

static void esp_free_skb(struct esp_skb *skb)
{
  if (skb)
    {
      if (skb->head)
        {
          kmm_free(skb->head);
        }
      kmm_free(skb);
    }
}

static void skb_reserve(struct esp_skb *skb, size_t len)
{
  skb->data += len;
  skb->tail += len;
}

static void skb_put(struct esp_skb *skb, size_t len)
{
  skb->tail += len;
  skb->len += len;
}

static void skb_push(struct esp_skb *skb, size_t len)
{
  skb->data -= len;
  skb->len += len;
}

static void skb_pull(struct esp_skb *skb, size_t len)
{
  skb->data += len;
  skb->len -= len;
}

static uint16_t esp_compute_checksum(FAR const uint8_t *buf, uint16_t len)
{
  uint16_t checksum = 0;
  uint16_t i;

  for (i = 0; i < len; i++)
    {
      checksum += buf[i];
    }

  return checksum;
}

static int esp_process_tx_packet(struct esp_wifi_device *priv,
                                  struct esp_skb *skb)
{
  struct esp_payload_header *payload_header;
  uint8_t pad_len;
  uint16_t len;
  uint16_t total_len;
  int ret;

  if (!priv || !skb || !skb->data)
    {
      netdev_err("Invalid parameters\n");
      return -EINVAL;
    }

  if (priv->stop_data)
    {
      netdev_err("Data path stopped\n");
      return -EPERM;
    }

  len = skb->len;

  pad_len = sizeof(struct esp_payload_header);
  total_len = len + pad_len;

  pad_len += SKB_DATA_ADDR_ALIGNMENT - (total_len % SKB_DATA_ADDR_ALIGNMENT);

  skb_push(skb, pad_len);

  payload_header = (struct esp_payload_header *)skb->data;
  memset(payload_header, 0, pad_len);

  payload_header->if_type = priv->if_type;
  payload_header->if_num = priv->if_num;
  payload_header->len = len;
  payload_header->offset = pad_len;
  payload_header->packet_type = ESP_PACKET_TYPE_DATA;

  if (priv->adapter && priv->adapter->capabilities & ESP_CHECKSUM_ENABLED)
    {
      payload_header->checksum = esp_compute_checksum(skb->data, total_len);
    }

  if (priv->adapter && priv->adapter->if_ops && priv->adapter->if_ops->send)
    {
      ret = priv->adapter->if_ops->send(priv->adapter, skb->data, skb->len);
      if (ret < 0)
        {
          netdev_err("Failed to send packet: %d\n", ret);
          priv->tx_dropped++;
        }
      else
        {
          priv->tx_packets++;
          priv->tx_bytes += skb->len;
        }
    }
  else
    {
      netdev_err("No interface operations\n");
      ret = -ENODEV;
    }

  return ret;
}

static int esp32_netdev_ifup(struct net_device *dev)
{
  struct esp_wifi_device *priv;
  int ret;

  if (!dev)
    {
      return -EINVAL;
    }

  priv = (struct esp_wifi_device *)dev->d_private;
  if (!priv)
    {
      return -EINVAL;
    }

  netdev_info("Bringing up interface %s\n", dev->d_ifname);

  priv->port_open = 1;
  priv->stop_data = 0;
  priv->link_state = ESP_LINK_UP;

  netdev_info("Interface %s is up\n", dev->d_ifname);
  return OK;
}

static int esp32_netdev_ifdown(struct net_device *dev)
{
  struct esp_wifi_device *priv;

  if (!dev)
    {
      return -EINVAL;
    }

  priv = (struct esp_wifi_device *)dev->d_private;
  if (!priv)
    {
      return -EINVAL;
    }

  netdev_info("Bringing down interface %s\n", dev->d_ifname);

  priv->port_open = 0;
  priv->stop_data = 1;
  priv->link_state = ESP_LINK_DOWN;

  netdev_info("Interface %s is down\n", dev->d_ifname);
  return OK;
}

static int esp32_netdev_tx(struct net_device *dev, void *pkt, size_t len)
{
  struct esp_wifi_device *priv;
  struct esp_skb *skb;
  int ret;

  if (!dev || !pkt || len == 0)
    {
      return -EINVAL;
    }

  priv = (struct esp_wifi_device *)dev->d_private;
  if (!priv)
    {
      return -EINVAL;
    }

  if (!priv->port_open || priv->stop_data)
    {
      netdev_err("Interface not ready for TX\n");
      return -EPERM;
    }

  skb = esp_alloc_skb(priv, len);
  if (!skb)
    {
      netdev_err("Failed to allocate SKB\n");
      return -ENOMEM;
    }

  memcpy(skb->data, pkt, len);
  skb_put(skb, len);

  ret = esp_process_tx_packet(priv, skb);

  esp_free_skb(skb);

  return ret;
}

static void esp32_netdev_txpoll(struct net_driver_s *dev)
{
}

static int esp32_netdev_ioctl(struct net_device *dev, int cmd, unsigned long arg)
{
  struct esp_wifi_device *priv;
  int ret = OK;

  if (!dev)
    {
      return -EINVAL;
    }

  priv = (struct esp_wifi_device *)dev->d_private;
  if (!priv)
    {
      return -EINVAL;
    }

  netdev_info("ioctl: cmd=%d arg=%lu\n", cmd, arg);

  switch (cmd)
    {
      case SIOCGIFADDR:
        {
          struct sockaddr *addr = (struct sockaddr *)arg;
          if (addr)
            {
              memcpy(addr->sa_data, priv->mac_address, ETH_ALEN);
            }
        }
        break;

      case SIOCSIFADDR:
        {
          struct sockaddr *addr = (struct sockaddr *)arg;
          if (addr)
            {
              ret = esp_cmd_set_mac(priv, (const uint8_t *)addr->sa_data);
            }
        }
        break;

      default:
        ret = -ENOTTY;
        break;
    }

  return ret;
}

static void esp_process_rx_packet(struct esp_adapter *adapter,
                                   FAR const uint8_t *data, size_t len)
{
  struct esp_payload_header *payload_header;
  struct esp_wifi_device *priv;
  struct net_device *netdev;
  uint8_t if_type;
  uint8_t if_num;
  uint16_t payload_len;
  uint16_t offset;

  if (!adapter || !data || len < sizeof(struct esp_payload_header))
    {
      netdev_err("Invalid RX packet\n");
      return;
    }

  payload_header = (struct esp_payload_header *)data;

  if_type = payload_header->if_type;
  if_num = payload_header->if_num;
  payload_len = payload_header->len;
  offset = payload_header->offset;

  if (payload_header->packet_type != ESP_PACKET_TYPE_DATA)
    {
      return;
    }

  if (if_num >= ESP_MAX_INTERFACE)
    {
      netdev_err("Invalid interface number: %d\n", if_num);
      return;
    }

  priv = adapter->priv[if_num];
  if (!priv || !priv->wdev.netdev)
    {
      netdev_err("No device for interface %d\n", if_num);
      return;
    }

  netdev = priv->wdev.netdev;

  if (offset > len)
    {
      netdev_err("Invalid offset: %d > %d\n", offset, len);
      return;
    }

  if (offset + payload_len > len)
    {
      netdev_err("Invalid payload length: %d + %d > %d\n",
                 offset, payload_len, len);
      return;
    }

  priv->rx_packets++;
  priv->rx_bytes += payload_len;

  netdev_input(netdev, (void *)(data + offset), payload_len);
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int esp32_netdev_receive(struct esp_adapter *adapter)
{
  uint8_t *rx_buf;
  size_t rx_len;
  int ret;

  if (!adapter || !adapter->if_ops || !adapter->if_ops->receive)
    {
      return -EINVAL;
    }

  rx_buf = kmm_malloc(ESP_RX_BUFFER_SIZE);
  if (!rx_buf)
    {
      return -ENOMEM;
    }

  rx_len = ESP_RX_BUFFER_SIZE;

  ret = adapter->if_ops->receive(adapter, rx_buf, rx_len);
  if (ret < 0)
    {
      kmm_free(rx_buf);
      return ret;
    }

  esp_process_rx_packet(adapter, rx_buf, rx_len);

  kmm_free(rx_buf);
  return OK;
}

int esp32_netdev_register(struct esp_wifi_device *priv)
{
  struct net_device *netdev;
  char ifname[IFNAMSIZ];
  int ret;

  if (!priv)
    {
      return -EINVAL;
    }

  snprintf(ifname, IFNAMSIZ, "wlan%d", priv->if_num);

  netdev = netdev_alloc(ifname, &g_esp32_netdev_ops);
  if (!netdev)
    {
      netdev_err("Failed to allocate netdev\n");
      return -ENOMEM;
    }

  netdev->d_private = priv;
  netdev->d_mtu = ESP_NETDEV_MTU;
  memcpy(netdev->d_mac.ether_addr_octet, priv->mac_address, ETH_ALEN);

  priv->wdev.netdev = netdev;
  priv->netdev = netdev;

  ret = netdev_register(netdev);
  if (ret < 0)
    {
      netdev_err("Failed to register netdev: %d\n", ret);
      netdev_free(netdev);
      return ret;
    }

  netdev_info("Registered network device %s\n", ifname);
  return OK;
}

int esp32_netdev_unregister(struct esp_wifi_device *priv)
{
  if (!priv || !priv->netdev)
    {
      return -EINVAL;
    }

  netdev_unregister(priv->netdev);
  netdev_free(priv->netdev);
  priv->netdev = NULL;
  priv->wdev.netdev = NULL;

  return OK;
}

void esp_port_open(struct esp_wifi_device *priv)
{
  if (priv)
    {
      priv->port_open = 1;
      priv->stop_data = 0;
    }
}

void esp_port_close(struct esp_wifi_device *priv)
{
  if (priv)
    {
      priv->port_open = 0;
      priv->stop_data = 1;
    }
}

void esp_tx_pause(struct esp_wifi_device *priv)
{
  if (priv && priv->netdev)
    {
      netdev_tx_stop(priv->netdev);
    }
}

void esp_tx_resume(struct esp_wifi_device *priv)
{
  if (priv && priv->netdev)
    {
      netdev_tx_wakeup(priv->netdev);
    }
}

#endif /* CONFIG_ESP32_WIFI */