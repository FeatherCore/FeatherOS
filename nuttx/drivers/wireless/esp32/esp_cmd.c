/****************************************************************************
 * drivers/wireless/esp32/esp_cmd.c
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
 * ESP32 Command Protocol Implementation
 *
 * This module implements the command/response protocol between the host
 * and ESP32 WiFi firmware.
 ****************************************************************************/

#include <nuttx/config.h>

#include <sys/types.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include <semaphore.h>
#include <errno.h>
#include <debug.h>

#include <nuttx/kmalloc.h>
#include <nuttx/semaphore.h>
#include <nuttx/wqueue.h>

#include "esp_cfg80211.h"
#include "esp_host_if.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Debug output */

#ifdef CONFIG_DEBUG_WIRELESS_INFO
#  define cmd_info(fmt, ...)  ninfo(fmt, ##__VA_ARGS__)
#else
#  define cmd_info(fmt, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_ERROR
#  define cmd_err(fmt, ...)   nerr(fmt, ##__VA_ARGS__)
#else
#  define cmd_err(fmt, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_VERBOSE
#  define cmd_verbose(fmt, ...) ninfo(fmt, ##__VA_ARGS__)
#else
#  define cmd_verbose(fmt, ...)
#endif

#define cmd_dbg cmd_verbose

/* Default values */

#define ESP_DEFAULT_CHANNEL          6
#define ESP_DEFAULT_AUTHMODE        0
#define ESP_DEFAULT_SCAN_METHOD     0
#define ESP_DEFAULT_FOUR_ADDR       0

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* Command timeout */

struct cmd_timeout_s
{
  uint8_t cmd_code;
  uint16_t timeout_ms;
};

/* Command timeout table */

static const struct cmd_timeout_s g_cmd_timeouts[] =
{
  { ESP_CMD_INIT_INTERFACE,  1000 },
  { ESP_CMD_GET_MAC,        500 },
  { ESP_CMD_SET_MAC,        500 },
  { ESP_CMD_SCAN_REQUEST,    10000 },
  { ESP_CMD_STA_CONNECT,    15000 },
  { ESP_CMD_DISCONNECT,     5000 },
  { ESP_CMD_STA_AUTH,       10000 },
  { ESP_CMD_STA_ASSOC,      10000 },
  { ESP_CMD_ADD_KEY,        1000 },
  { ESP_CMD_DEL_KEY,        1000 },
  { ESP_CMD_SET_DEFAULT_KEY, 500 },
  { ESP_CMD_SET_MODE,       1000 },
  { ESP_CMD_SET_AP_CONFIG,  2000 },
  { ESP_CMD_MGMT_TX,       5000 },
  { ESP_CMD_AP_STATION,     2000 },
  { ESP_CMD_STA_RSSI,       500 },
  { ESP_CMD_GET_TXPOWER,    500 },
  { ESP_CMD_SET_TXPOWER,    500 },
  { ESP_CMD_GET_REG_DOMAIN, 500 },
  { ESP_CMD_SET_REG_DOMAIN, 1000 },
  { ESP_CMD_MAX,           5000 },
};

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp_get_cmd_timeout
 *
 * Description:
 *   Get timeout for a specific command.
 *
 ****************************************************************************/

static uint16_t esp_get_cmd_timeout(uint8_t cmd_code)
{
  for (int i = 0; i < sizeof(g_cmd_timeouts) / sizeof(g_cmd_timeouts[0]); i++)
    {
      if (g_cmd_timeouts[i].cmd_code == cmd_code)
        {
          return g_cmd_timeouts[i].timeout_ms;
        }
    }
  return 5000;  /* Default 5 seconds */
}

/****************************************************************************
 * Name: esp_process_scan_result_event
 *
 * Description:
 *   Process scan result event from ESP32.
 *
 ****************************************************************************/

static int esp_process_scan_result_event(FAR struct esp_adapter *adapter,
                                        FAR const uint8_t *data, size_t len)
{
  FAR struct esp_scan_result_event *event;
  uint8_t bssid[ESP_MAC_ADDR_LEN];
  uint8_t channel_num;
  int signal;
  FAR struct esp_wifi_device *priv;
  FAR const uint8_t *frame;
  uint16_t frame_len;

  if (len < sizeof(struct esp_event_header))
    {
      cmd_err("Invalid scan event length: %zu\n", len);
      return -EINVAL;
    }

  event = (FAR struct esp_scan_result_event *)data;

  /* Extract BSS information */

  memcpy(bssid, event->bssid, ESP_MAC_ADDR_LEN);
  channel_num = event->channel;
  signal = event->rssi;
  frame_len = event->frame_len;

  /* Frame data follows the event structure */
  frame = data + sizeof(struct esp_scan_result_event);

  cmd_info("Scan result: BSSID=%02x:%02x:%02x:%02x:%02x:%02x ch=%d rssi=%d len=%d\n",
           bssid[0], bssid[1], bssid[2], bssid[3], bssid[4], bssid[5],
           channel_num, signal, frame_len);

  /* Find or create channel entry for 2.4GHz band */
  if (channel_num >= 1 && channel_num <= 14)
    {
      /* 2.4GHz channel: freq = 2407 + 5 * (channel - 1) */
      static struct ieee80211_channel chan = 
        {
          .band = IEEE80211_BAND_2GHZ,
          .center_freq = 0,
          .hw_value = 0,
          .max_power = 20,
        };
      
      chan.center_freq = 2407 + 5 * (channel_num - 1);
      chan.hw_value = channel_num;
      
      /* Find STA device to get wiphy */
      priv = adapter->priv[ESP_STA_NW_IF];
      if (priv && priv->wdev.wiphy)
        {
          /* Notify cfg80211 of the BSS */
          cfg80211_inform_bss(priv->wdev.wiphy, &chan, bssid,
                            0, 0,  /* TSF */
                            0, 0,  /* capability, beacon interval */
                            frame, frame_len,  /* IEs */
                            signal,  /* signal (RSSI in dBm) */
                            0);  /* gfp_flags */
        }
    }

  return OK;
}

/****************************************************************************
 * Name: esp_process_connect_event
 *
 * Description:
 *   Process connect event from ESP32.
 *
 ****************************************************************************/

static int esp_process_connect_event(FAR struct esp_adapter *adapter,
                                    FAR const uint8_t *data, size_t len)
{
  FAR struct esp_wifi_device *priv;
  uint8_t bssid[ESP_MAC_ADDR_LEN];
  uint16_t status;
  uint8_t ssid[ESP_MAX_SSID_LEN + 1];
  size_t ssid_len;

  if (len < sizeof(struct esp_event_header))
    {
      cmd_err("Invalid connect event length: %zu\n", len);
      return -EINVAL;
    }

  /* Find STA device */

  priv = adapter->priv[ESP_STA_NW_IF];
  if (!priv)
    {
      cmd_err("No STA device found\n");
      return -ENODEV;
    }

  /* Extract connection info from event */
  /* This would parse the event data to get BSSID, status, SSID, etc. */

  cmd_info("Connected to BSSID=%02x:%02x:%02x:%02x:%02x:%02x\n",
           bssid[0], bssid[1], bssid[2], bssid[3], bssid[4], bssid[5]);

  /* Notify cfg80211 of successful connection */

  esp_notify_connect_result(priv, bssid, 0);

  return OK;
}

/****************************************************************************
 * Name: esp_process_disconnect_event
 *
 * Description:
 *   Process disconnect event from ESP32.
 *
 ****************************************************************************/

static int esp_process_disconnect_event(FAR struct esp_adapter *adapter,
                                      FAR const uint8_t *data, size_t len)
{
  FAR struct esp_disconnect_event *event;
  FAR struct esp_wifi_device *priv;
  uint16_t reason;
  bool local_disconnect;

  if (len < sizeof(struct esp_disconnect_event))
    {
      cmd_err("Invalid disconnect event length: %zu\n", len);
      return -EINVAL;
    }

  event = (FAR struct esp_disconnect_event *)data;
  reason = event->reason;

  /* Find STA device */

  priv = adapter->priv[ESP_STA_NW_IF];
  if (!priv)
    {
      cmd_err("No STA device found\n");
      return -ENODEV;
    }

  cmd_info("Disconnected from network, reason=%d\n", reason);

  /* Determine if this was a local disconnect */

  local_disconnect = priv->local_disconnect_req;
  priv->local_disconnect_req = false;

  /* Notify cfg80211 of disconnection */

  esp_notify_disconnect(priv, reason, local_disconnect);

  return OK;
}

/****************************************************************************
 * Name: esp_process_rx_packet
 *
 * Description:
 *   Process received packet from ESP32.
 *
 ****************************************************************************/

static int esp_process_rx_packet(FAR struct esp_adapter *adapter,
                                FAR const uint8_t *data, size_t len)
{
  FAR struct esp_payload_header *hdr;
  FAR struct esp_wifi_device *priv;
  FAR struct net_device *netdev;
  uint8_t if_type;
  uint8_t if_num;
  uint8_t packet_type;

  if (len < sizeof(struct esp_payload_header))
    {
      cmd_err("Invalid packet length: %zu\n", len);
      return -EINVAL;
    }

  hdr = (FAR struct esp_payload_header *)data;

  /* Extract interface info from payload header */

  if_type = hdr->if_type;
  if_num = hdr->if_num;
  packet_type = hdr->packet_type;

  cmd_verbose("RX packet: if_type=%d if_num=%d pkt_type=%d len=%d\n",
              if_type, if_num, packet_type, len - sizeof(*hdr));

  /* Find the device */

  if (if_num >= ESP_MAX_INTERFACE)
    {
      cmd_err("Invalid interface number: %d\n", if_num);
      return -EINVAL;
    }

  priv = adapter->priv[if_num];
  if (!priv)
    {
      cmd_err("No device for interface %d\n", if_num);
      return -ENODEV;
    }

  netdev = priv->netdev;
  if (!netdev)
    {
      cmd_err("No netdev for interface %d\n", if_num);
      return -ENODEV;
    }

  /* Process based on packet type */

  switch (packet_type)
    {
      case ESP_PACKET_TYPE_EVENT:
        /* Process event from ESP32 */
        {
          FAR struct esp_event_header *event_hdr;
          size_t event_len = len - sizeof(*hdr);

          if (event_len < sizeof(struct esp_event_header))
            {
              cmd_err("Invalid event length\n");
              return -EINVAL;
            }

          event_hdr = (FAR struct esp_event_header *)(data + sizeof(*hdr));

          cmd_verbose("Event: code=%d status=%d len=%d\n",
                      event_hdr->event_code, event_hdr->status, event_hdr->len);

          switch (event_hdr->event_code)
            {
              case ESP_EVENT_SCAN_RESULT:
                esp_process_scan_result_event(adapter,
                                            data + sizeof(*hdr), event_len);
                break;

              case ESP_EVENT_STA_CONNECT:
                esp_process_connect_event(adapter,
                                        data + sizeof(*hdr), event_len);
                break;

              case ESP_EVENT_STA_DISCONNECT:
                esp_process_disconnect_event(adapter,
                                            data + sizeof(*hdr), event_len);
                break;

              default:
                cmd_info("Unknown event code: %d\n", event_hdr->event_code);
                break;
            }
        }
        break;

      case ESP_PACKET_TYPE_DATA:
        /* Process data packet - forward to network stack */
        {
          FAR const uint8_t *payload = data + sizeof(*hdr);
          size_t payload_len = len - sizeof(*hdr) - hdr->offset;

          cmd_verbose("Data packet: len=%zu\n", payload_len);

          /* Find the appropriate interface and forward to network stack */
          FAR struct esp_wifi_device *priv = NULL;
          if (hdr->if_num < ESP_MAX_INTERFACE)
            {
              priv = adapter->priv[hdr->if_num];
            }
          
          if (priv && priv->netdev && payload_len > 0)
            {
              /* Forward to network stack via netdev_rx() */
              netdev_rx(priv->netdev, payload, payload_len);
            }
        }
        break;

      default:
        cmd_info("Unknown packet type: %d\n", packet_type);
        break;
    }

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp_process_command_response
 *
 * Description:
 *   Process command response from ESP32.
 *
 ****************************************************************************/

int esp_process_command_response(FAR struct esp_adapter *adapter,
                               FAR const uint8_t *data, size_t len)
{
  FAR struct esp_command_header *hdr;
  uint8_t cmd_code;
  uint8_t cmd_status;
  int ret = OK;

  if (!adapter || !data || len < sizeof(struct esp_command_header))
    {
      return -EINVAL;
    }

  hdr = (FAR struct esp_command_header *)data;
  cmd_code = hdr->cmd_code;
  cmd_status = hdr->cmd_status;

  cmd_verbose("Command response: cmd=%d status=%d\n", cmd_code, cmd_status);

  /* Store response status */

  adapter->cmd_resp_status = cmd_status;

  /* Wake up command waiters */

  nxsem_post(&adapter->cmd_resp_sem);

  /* Process response based on command type */

  switch (cmd_code)
    {
      case ESP_CMD_GET_MAC:
        /* Copy MAC address to device */
        if (adapter->priv[0] && len >= sizeof(struct esp_mac_addr_cmd))
          {
            FAR struct esp_mac_addr_cmd *resp =
              (FAR struct esp_mac_addr_cmd *)data;
            memcpy(adapter->priv[0]->mac_address, resp->mac_addr,
                   ESP_MAC_ADDR_LEN);
          }
        break;

      case ESP_CMD_STA_RSSI:
        if (adapter->priv[ESP_STA_NW_IF] && len >= sizeof(struct esp_set_get_cmd))
          {
            FAR struct esp_set_get_cmd *resp =
              (FAR struct esp_set_get_cmd *)data;
            adapter->priv[ESP_STA_NW_IF]->rssi = resp->value;
          }
        break;

      case ESP_CMD_GET_TXPOWER:
        if (adapter->priv[0] && len >= sizeof(struct esp_set_get_cmd))
          {
            FAR struct esp_set_get_cmd *resp =
              (FAR struct esp_set_get_cmd *)data;
            adapter->priv[0]->tx_pwr = resp->value;
          }
        break;

      case ESP_CMD_SCAN_REQUEST:
        /* Scan initiated successfully */
        cmd_info("Scan initiated\n");
        break;

      case ESP_CMD_STA_CONNECT:
        if (cmd_status == ESP_CMD_RESPONSE_SUCCESS)
          {
            cmd_info("Connection initiated\n");
          }
        else
          {
            cmd_err("Connection failed: status=%d\n", cmd_status);
            ret = -EIO;
          }
        break;

      case ESP_CMD_DISCONNECT:
        cmd_info("Disconnection initiated\n");
        break;

      default:
        cmd_info("Unhandled command response: %d\n", cmd_code);
        break;
    }

  return ret;
}

/****************************************************************************
 * Name: esp_process_event
 *
 * Description:
 *   Process event from ESP32.
 *
 ****************************************************************************/

int esp_process_event(FAR struct esp_adapter *adapter,
                     FAR const uint8_t *data, size_t len)
{
  FAR struct esp_payload_header *hdr;

  if (!adapter || !data || len < sizeof(struct esp_payload_header))
    {
      return -EINVAL;
    }

  hdr = (FAR struct esp_payload_header *)data;

  cmd_verbose("Processing event: pkt_type=%d\n", hdr->packet_type);

  /* Route to appropriate handler based on packet type */

  if (hdr->packet_type == ESP_PACKET_TYPE_EVENT ||
      hdr->packet_type == ESP_PACKET_TYPE_DATA)
    {
      return esp_process_rx_packet(adapter, data, len);
    }

  return OK;
}

/****************************************************************************
 * Name: esp_cmd_scan_request
 *
 * Description:
 *   Send scan request to ESP32.
 *
 ****************************************************************************/

int esp_cmd_scan_request(FAR struct esp_wifi_device *priv,
                       FAR struct cfg80211_scan_request *request)
{
  struct esp_scan_request_cmd cmd;
  int ret;

  if (!priv || !request)
    {
      return -EINVAL;
    }

  /* Build scan request command */

  memset(&cmd, 0, sizeof(cmd));
  cmd.header.cmd_code = ESP_CMD_SCAN_REQUEST;
  cmd.header.cmd_status = 0;
  cmd.header.len = sizeof(cmd);
  cmd.header.seq_num = 0;
  cmd.header.reserved1 = 0;
  cmd.header.reserved2 = 0;

  /* Set SSID if provided */

  if (request->n_ssids > 0 && request->ssids)
    {
      cmd.ssid[0] = 0;  /* Broadcast SSID for active scan */
    }

  /* Set channel if specified (0 = all channels) */

  cmd.channel = 0;

  cmd.duration = 0;

  cmd_info("Sending scan request\n");

  /* Send command */

  ret = esp_send_command(priv, ESP_CMD_SCAN_REQUEST,
                        (FAR const uint8_t *)&cmd, sizeof(cmd));
  if (ret < 0)
    {
      cmd_err("Failed to send scan request: %d\n", ret);
      return ret;
    }

  /* Wait for response or scan to complete */

  return OK;
}

/****************************************************************************
 * Name: esp_cmd_connect
 *
 * Description:
 *   Send connect request to ESP32.
 *
 ****************************************************************************/

int esp_cmd_connect(FAR struct esp_wifi_device *priv,
                   FAR struct cfg80211_connect_params *params)
{
  struct esp_connect_cmd cmd;
  size_t cmd_len;
  int ret;

  if (!priv || !params)
    {
      return -EINVAL;
    }

  cmd_info("Connecting to SSID: %.*s\n",
           (int)params->ssid.ssid_len, params->ssid.ssid);

  /* Build connect command */

  memset(&cmd, 0, sizeof(cmd));
  cmd.header.cmd_code = ESP_CMD_STA_CONNECT;
  cmd.header.reserved1 = 0;
  cmd.header.reserved2 = 0;

  /* Set BSSID if provided */

  if (params->bssid)
    {
      memcpy(cmd.bssid, params->bssid, ESP_MAC_ADDR_LEN);
    }
  else
    {
      memset(cmd.bssid, 0xff, ESP_MAC_ADDR_LEN);  /* Broadcast for any BSS */
    }

  /* Set SSID */

  if (params->ssid.ssid_len > ESP_MAX_SSID_LEN)
    {
      cmd.ssid[ESP_MAX_SSID_LEN] = 0;
    }
  else
    {
      memcpy(cmd.ssid, params->ssid.ssid, params->ssid.ssid_len);
      cmd.ssid[params->ssid.ssid_len] = 0;
    }

  /* Set channel (0 = auto) */

  cmd.channel = ESP_DEFAULT_CHANNEL;

  /* Set auth mode */

  cmd.is_auth_open = (params->auth_type == NL80211_AUTHTYPE_OPEN_SYSTEM) ? 1 : 0;

  /* Set IE length and copy IE data */

  if (params->ie && params->ie_len > 0 &&
      params->ie_len < sizeof(cmd.assoc_ie))
    {
      memcpy(cmd.assoc_ie, params->ie, params->ie_len);
      cmd.assoc_ie_len = params->ie_len;
    }
  else
    {
      cmd.assoc_ie_len = 0;
    }

  cmd_len = sizeof(cmd.header) + sizeof(cmd.bssid) +
            sizeof(cmd.assoc_flags) + sizeof(cmd.ssid) +
            sizeof(cmd.channel) + sizeof(cmd.is_auth_open) +
            sizeof(cmd.assoc_ie_len) + cmd.assoc_ie_len;

  cmd.header.len = cmd_len;

  /* Send command */

  ret = esp_send_command(priv, ESP_CMD_STA_CONNECT,
                        (FAR const uint8_t *)&cmd, cmd_len);
  if (ret < 0)
    {
      cmd_err("Failed to send connect request: %d\n", ret);
      return ret;
    }

  return OK;
}

/****************************************************************************
 * Name: esp_cmd_disconnect
 *
 * Description:
 *   Send disconnect request to ESP32.
 *
 ****************************************************************************/

int esp_cmd_disconnect(FAR struct esp_wifi_device *priv, uint16_t reason,
                       FAR const uint8_t *mac)
{
  struct esp_disconnect_cmd cmd;
  int ret;

  if (!priv)
    {
      return -EINVAL;
    }

  cmd_info("Sending disconnect request, reason=%d\n", reason);

  /* Build disconnect command */

  memset(&cmd, 0, sizeof(cmd));
  cmd.header.cmd_code = ESP_CMD_DISCONNECT;
  cmd.header.cmd_status = 0;
  cmd.header.len = sizeof(cmd);
  cmd.header.seq_num = 0;
  cmd.header.reserved1 = 0;
  cmd.header.reserved2 = 0;

  cmd.reason_code = reason;

  if (mac)
    {
      memcpy(cmd.mac, mac, ESP_MAC_ADDR_LEN);
    }
  else
    {
      memset(cmd.mac, 0xff, ESP_MAC_ADDR_LEN);
    }

  /* Mark as local disconnect */

  priv->local_disconnect_req = true;

  /* Send command */

  ret = esp_send_command(priv, ESP_CMD_DISCONNECT,
                        (FAR const uint8_t *)&cmd, sizeof(cmd));
  if (ret < 0)
    {
      cmd_err("Failed to send disconnect request: %d\n", ret);
      priv->local_disconnect_req = false;
      return ret;
    }

  return OK;
}

/****************************************************************************
 * Name: esp_cmd_add_key
 *
 * Description:
 *   Send add key request to ESP32.
 *
 ****************************************************************************/

int esp_cmd_add_key(FAR struct esp_wifi_device *priv, uint8_t key_index,
                    bool pairwise, FAR const uint8_t *mac_addr,
                    FAR struct key_params *params)
{
  struct esp_key_cmd cmd;
  int ret;

  if (!priv || !params)
    {
      return -EINVAL;
    }

  cmd_info("Adding key: idx=%d pairwise=%d\n", key_index, pairwise);

  /* Build key command */

  memset(&cmd, 0, sizeof(cmd));
  cmd.header.cmd_code = ESP_CMD_ADD_KEY;
  cmd.header.cmd_status = 0;
  cmd.header.len = sizeof(cmd);
  cmd.header.seq_num = 0;
  cmd.header.reserved1 = 0;
  cmd.header.reserved2 = 0;

  /* Set key parameters */

  cmd.key.algo = params->cipher;
  cmd.key.index = key_index;

  if (params->key && params->key_len > 0 &&
      params->key_len <= ESP_MAX_KEY_LEN)
    {
      memcpy(cmd.key.data, params->key, params->key_len);
      cmd.key.len = params->key_len;
    }

  if (params->seq && params->seq_len > 0 &&
      params->seq_len <= ESP_MAX_SEQ_LEN)
    {
      memcpy(cmd.key.seq, params->seq, params->seq_len);
      cmd.key.seq_len = params->seq_len;
    }

  if (mac_addr)
    {
      memcpy(cmd.key.mac_addr, mac_addr, ESP_MAC_ADDR_LEN);
    }
  else
    {
      memset(cmd.key.mac_addr, 0xff, ESP_MAC_ADDR_LEN);
    }

  cmd.key.del = 0;
  cmd.key.set_cur = 1;

  /* Send command */

  ret = esp_send_command(priv, ESP_CMD_ADD_KEY,
                        (FAR const uint8_t *)&cmd, sizeof(cmd));
  if (ret < 0)
    {
      cmd_err("Failed to send add key: %d\n", ret);
      return ret;
    }

  return OK;
}

/****************************************************************************
 * Name: esp_cmd_del_key
 *
 * Description:
 *   Send delete key request to ESP32.
 *
 ****************************************************************************/

int esp_cmd_del_key(FAR struct esp_wifi_device *priv, uint8_t key_index,
                    bool pairwise, FAR const uint8_t *mac_addr)
{
  struct esp_key_cmd cmd;
  int ret;

  if (!priv)
    {
      return -EINVAL;
    }

  cmd_info("Deleting key: idx=%d\n", key_index);

  /* Build key command with delete flag */

  memset(&cmd, 0, sizeof(cmd));
  cmd.header.cmd_code = ESP_CMD_DEL_KEY;
  cmd.header.cmd_status = 0;
  cmd.header.len = sizeof(cmd);
  cmd.header.seq_num = 0;
  cmd.header.reserved1 = 0;
  cmd.header.reserved2 = 0;

  cmd.key.algo = 0;
  cmd.key.index = key_index;
  cmd.key.len = 0;
  cmd.key.seq_len = 0;
  cmd.key.del = 1;
  cmd.key.set_cur = 0;

  if (mac_addr)
    {
      memcpy(cmd.key.mac_addr, mac_addr, ESP_MAC_ADDR_LEN);
    }
  else
    {
      memset(cmd.key.mac_addr, 0xff, ESP_MAC_ADDR_LEN);
    }

  /* Send command */

  ret = esp_send_command(priv, ESP_CMD_DEL_KEY,
                        (FAR const uint8_t *)&cmd, sizeof(cmd));
  if (ret < 0)
    {
      cmd_err("Failed to send del key: %d\n", ret);
      return ret;
    }

  return OK;
}

/****************************************************************************
 * Name: esp_cmd_set_default_key
 *
 * Description:
 *   Send set default key request to ESP32.
 *
 ****************************************************************************/

int esp_cmd_set_default_key(FAR struct esp_wifi_device *priv,
                            uint8_t key_index)
{
  struct esp_command_header cmd;
  int ret;

  if (!priv)
    {
      return -EINVAL;
    }

  cmd_info("Setting default key: idx=%d\n", key_index);

  /* Build command */

  memset(&cmd, 0, sizeof(cmd));
  cmd.cmd_code = ESP_CMD_SET_DEFAULT_KEY;
  cmd.cmd_status = 0;
  cmd.len = sizeof(cmd);
  cmd.seq_num = 0;
  cmd.reserved1 = 0;
  cmd.reserved2 = 0;

  /* Note: The actual key index is embedded in the command-specific data */
  /* For simplicity, we use key_index as the payload */

  /* Send command */

  ret = esp_send_command(priv, ESP_CMD_SET_DEFAULT_KEY,
                        (FAR const uint8_t *)&cmd, sizeof(cmd));
  if (ret < 0)
    {
      cmd_err("Failed to send set default key: %d\n", ret);
      return ret;
    }

  return OK;
}

/****************************************************************************
 * Name: esp_cmd_get_rssi
 *
 * Description:
 *   Get RSSI from ESP32.
 *
 ****************************************************************************/

int esp_cmd_get_rssi(FAR struct esp_wifi_device *priv)
{
  struct esp_command_header cmd;
  int ret;

  if (!priv)
    {
      return -EINVAL;
    }

  /* Build command */

  memset(&cmd, 0, sizeof(cmd));
  cmd.cmd_code = ESP_CMD_STA_RSSI;
  cmd.cmd_status = 0;
  cmd.len = sizeof(cmd);
  cmd.seq_num = 0;
  cmd.reserved1 = 0;
  cmd.reserved2 = 0;

  /* Send command */

  ret = esp_send_command(priv, ESP_CMD_STA_RSSI,
                        (FAR const uint8_t *)&cmd, sizeof(cmd));
  if (ret < 0)
    {
      cmd_err("Failed to get RSSI: %d\n", ret);
      return ret;
    }

  return OK;
}

/****************************************************************************
 * Name: esp_cmd_set_tx_power
 *
 * Description:
 *   Set TX power on ESP32.
 *
 ****************************************************************************/

int esp_cmd_set_tx_power(FAR struct esp_wifi_device *priv, int power)
{
  struct esp_set_get_cmd cmd;
  int ret;

  if (!priv)
    {
      return -EINVAL;
    }

  cmd_info("Setting TX power: %d dBm\n", power);

  /* Build command */

  memset(&cmd, 0, sizeof(cmd));
  cmd.header.cmd_code = ESP_CMD_SET_TXPOWER;
  cmd.header.cmd_status = 0;
  cmd.header.len = sizeof(cmd);
  cmd.header.seq_num = 0;
  cmd.header.reserved1 = 0;
  cmd.header.reserved2 = 0;
  cmd.value = power;

  /* Send command */

  ret = esp_send_command(priv, ESP_CMD_SET_TXPOWER,
                        (FAR const uint8_t *)&cmd, sizeof(cmd));
  if (ret < 0)
    {
      cmd_err("Failed to set TX power: %d\n", ret);
      return ret;
    }

  return OK;
}

/****************************************************************************
 * Name: esp_cmd_get_tx_power
 *
 * Description:
 *   Get TX power from ESP32.
 *
 ****************************************************************************/

int esp_cmd_get_tx_power(FAR struct esp_wifi_device *priv)
{
  struct esp_command_header cmd;
  int ret;

  if (!priv)
    {
      return -EINVAL;
    }

  /* Build command */

  memset(&cmd, 0, sizeof(cmd));
  cmd.cmd_code = ESP_CMD_GET_TXPOWER;
  cmd.cmd_status = 0;
  cmd.len = sizeof(cmd);
  cmd.seq_num = 0;
  cmd.reserved1 = 0;
  cmd.reserved2 = 0;

  /* Send command */

  ret = esp_send_command(priv, ESP_CMD_GET_TXPOWER,
                        (FAR const uint8_t *)&cmd, sizeof(cmd));
  if (ret < 0)
    {
      cmd_err("Failed to get TX power: %d\n", ret);
      return ret;
    }

  return OK;
}

/****************************************************************************
 * Name: esp_cmd_start_ap
 *
 * Description:
 *   Start AP on ESP32.
 *
 ****************************************************************************/

int esp_cmd_start_ap(FAR struct esp_wifi_device *priv,
                     FAR struct cfg80211_ap_settings *settings)
{
  struct esp_ap_config_cmd cmd;
  int ret;

  if (!priv || !settings)
    {
      return -EINVAL;
    }

  cmd_info("Starting AP: SSID=%.*s channel=%d\n",
           (int)settings->ssid_len, settings->ssid,
           settings->beacon.channel);

  /* Build AP config command */

  memset(&cmd, 0, sizeof(cmd));
  cmd.header.cmd_code = ESP_CMD_AP_CONFIG;
  cmd.header.cmd_status = 0;
  cmd.header.len = sizeof(cmd);
  cmd.header.seq_num = 0;
  cmd.header.reserved1 = 0;
  cmd.header.reserved2 = 0;

  /* Set AP configuration */

  memcpy(cmd.ap_config.ssid, settings->ssid, settings->ssid_len);
  cmd.ap_config.ssid_len = settings->ssid_len;
  cmd.ap_config.channel = settings->beacon.channel;
  cmd.ap_config.authmode = settings->auth_type;
  cmd.ap_config.beacon_interval = settings->beacon.beacon_interval;
  cmd.ap_config.privacy = settings->privacy;

  /* Parse IE for security configuration */
  /* Security IE parsing would extract WPA/RSN info from beacon tail */
  /* For now, authmode is set from cfg80211_ap_settings.auth_type */

  /* Send command */

  ret = esp_send_command(priv, ESP_CMD_AP_CONFIG,
                        (FAR const uint8_t *)&cmd, sizeof(cmd));
  if (ret < 0)
    {
      cmd_err("Failed to start AP: %d\n", ret);
      return ret;
    }

  return OK;
}

/****************************************************************************
 * Name: esp_cmd_stop_ap
 *
 * Description:
 *   Stop AP on ESP32.
 *
 ****************************************************************************/

int esp_cmd_stop_ap(FAR struct esp_wifi_device *priv)
{
  struct esp_command_header cmd;
  int ret;

  if (!priv)
    {
      return -EINVAL;
    }

  cmd_info("Stopping AP\n");

  /* Build command */

  memset(&cmd, 0, sizeof(cmd));
  cmd.cmd_code = ESP_CMD_AP_CONFIG;
  cmd.cmd_status = 0;
  cmd.len = sizeof(cmd);
  cmd.seq_num = 0;
  cmd.reserved1 = 0;
  cmd.reserved2 = 0;

  /* Send command */

  ret = esp_send_command(priv, ESP_CMD_AP_CONFIG,
                        (FAR const uint8_t *)&cmd, sizeof(cmd));
  if (ret < 0)
    {
      cmd_err("Failed to stop AP: %d\n", ret);
      return ret;
    }

  return OK;
}

/****************************************************************************
 * Name: esp_cmd_mgmt_tx
 *
 * Description:
 *   Send management frame via ESP32.
 *
 ****************************************************************************/

int esp_cmd_mgmt_tx(FAR struct esp_wifi_device *priv,
                     FAR struct cfg80211_mgmt_tx_params *params)
{
  struct esp_mgmt_tx_cmd cmd;
  int ret;

  if (!priv || !params || !params->buf)
    {
      return -EINVAL;
    }

  cmd_info("Sending management frame: len=%zu\n", params->len);

  /* Build management TX command */

  memset(&cmd, 0, sizeof(cmd));
  cmd.header.cmd_code = ESP_CMD_MGMT_TX;
  cmd.header.cmd_status = 0;
  cmd.header.len = sizeof(struct esp_command_header) + sizeof(cmd.channel) +
                   sizeof(cmd.offchan) + sizeof(cmd.wait) + sizeof(cmd.len) +
                   params->len;
  cmd.header.seq_num = 0;
  cmd.header.reserved1 = 0;
  cmd.header.reserved2 = 0;

  cmd.channel = params->chan ? params->chan->hw_value : 0;
  cmd.offchan = params->offchan ? 1 : 0;
  cmd.wait = params->wait;
  cmd.no_cck = 0;
  cmd.dont_wait_for_ack = 0;
  cmd.len = params->len;

  /* Copy frame data after header */
  if (params->len > 0 && params->len <= ESP_MAX_IE_LEN)
    {
      memcpy(cmd.buf, params->buf, params->len);
    }

  /* Send command */

  ret = esp_send_command(priv, ESP_CMD_MGMT_TX,
                        (FAR const uint8_t *)&cmd,
                        sizeof(struct esp_command_header) + 12 + params->len);
  if (ret < 0)
    {
      cmd_err("Failed to send management frame: %d\n", ret);
      return ret;
    }

  return OK;
}

/****************************************************************************
 * Name: esp_cmd_add_station
 *
 * Description:
 *   Add station to AP.
 *
 ****************************************************************************/

int esp_cmd_add_station(FAR struct esp_wifi_device *priv,
                         FAR const uint8_t *mac,
                         FAR struct station_parameters *params)
{
  struct esp_ap_sta_cmd cmd;
  int ret;

  if (!priv || !mac)
    {
      return -EINVAL;
    }

  cmd_info("Adding station: MAC=%02x:%02x:%02x:%02x:%02x:%02x\n",
           mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);

  /* Build station command */

  memset(&cmd, 0, sizeof(cmd));
  cmd.header.cmd_code = ESP_CMD_AP_STATION;
  cmd.header.cmd_status = 0;
  cmd.header.len = sizeof(cmd);
  cmd.header.seq_num = 0;
  cmd.header.reserved1 = 0;
  cmd.header.reserved2 = 0;

  memcpy(cmd.mac, mac, ESP_MAC_ADDR_LEN);
  cmd.cmd = 0;  /* ADD_STA */

  if (params)
    {
      cmd.sta_flags_mask = params->sta_flags_mask;
      cmd.sta_flags_set = params->sta_flags_set;
      cmd.listen_interval = params->listen_interval;
      cmd.aid = params->aid;
    }

  /* Send command */

  ret = esp_send_command(priv, ESP_CMD_AP_STATION,
                        (FAR const uint8_t *)&cmd, sizeof(cmd));
  if (ret < 0)
    {
      cmd_err("Failed to add station: %d\n", ret);
      return ret;
    }

  return OK;
}

/****************************************************************************
 * Name: esp_cmd_del_station
 *
 * Description:
 *   Delete station from AP.
 *
 ****************************************************************************/

int esp_cmd_del_station(FAR struct esp_wifi_device *priv,
                         FAR const uint8_t *mac, uint16_t reason)
{
  struct esp_ap_sta_cmd cmd;
  int ret;

  if (!priv)
    {
      return -EINVAL;
    }

  cmd_info("Deleting station: reason=%d\n", reason);

  /* Build station command */

  memset(&cmd, 0, sizeof(cmd));
  cmd.header.cmd_code = ESP_CMD_AP_STATION;
  cmd.header.cmd_status = 0;
  cmd.header.len = sizeof(cmd);
  cmd.header.seq_num = 0;
  cmd.header.reserved1 = 0;
  cmd.header.reserved2 = 0;

  if (mac)
    {
      memcpy(cmd.mac, mac, ESP_MAC_ADDR_LEN);
    }
  else
    {
      memset(cmd.mac, 0xff, ESP_MAC_ADDR_LEN);  /* Delete all stations */
    }

  cmd.cmd = 2;  /* DEL_STA */

  /* Send command */

  ret = esp_send_command(priv, ESP_CMD_AP_STATION,
                        (FAR const uint8_t *)&cmd, sizeof(cmd));
  if (ret < 0)
    {
      cmd_err("Failed to delete station: %d\n", ret);
      return ret;
    }

  return OK;
}

/****************************************************************************
 * Name: esp_cmd_change_station
 *
 * Description:
 *   Change station parameters.
 *
 ****************************************************************************/

int esp_cmd_change_station(FAR struct esp_wifi_device *priv,
                           FAR const uint8_t *mac,
                           FAR struct station_parameters *params)
{
  struct esp_ap_sta_cmd cmd;
  int ret;

  if (!priv || !mac || !params)
    {
      return -EINVAL;
    }

  cmd_info("Changing station: MAC=%02x:%02x:%02x:%02x:%02x:%02x\n",
           mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);

  /* Build station command */

  memset(&cmd, 0, sizeof(cmd));
  cmd.header.cmd_code = ESP_CMD_AP_STATION;
  cmd.header.cmd_status = 0;
  cmd.header.len = sizeof(cmd);
  cmd.header.seq_num = 0;
  cmd.header.reserved1 = 0;
  cmd.header.reserved2 = 0;

  memcpy(cmd.mac, mac, ESP_MAC_ADDR_LEN);
  cmd.cmd = 1;  /* CHANGE_STA */

  cmd.sta_flags_mask = params->sta_flags_mask;
  cmd.sta_flags_set = params->sta_flags_set;
  cmd.sta_modify_mask = params->sta_modify_mask;
  cmd.listen_interval = params->listen_interval;
  cmd.aid = params->aid;

  /* Send command */

  ret = esp_send_command(priv, ESP_CMD_AP_STATION,
                        (FAR const uint8_t *)&cmd, sizeof(cmd));
  if (ret < 0)
    {
      cmd_err("Failed to change station: %d\n", ret);
      return ret;
    }

  return OK;
}