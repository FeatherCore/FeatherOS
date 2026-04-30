/****************************************************************************
 * drivers/wireless/esp32/esp_event.c
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
 * ESP32 Event Processing Implementation
 *
 * This module handles events from ESP32 WiFi firmware and forwards them
 * to the cfg80211 layer.
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
#  define evt_info(fmt, ...)  ninfo(fmt, ##__VA_ARGS__)
#else
#  define evt_info(fmt, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_ERROR
#  define evt_err(fmt, ...)   nerr(fmt, ##__VA_ARGS__)
#else
#  define evt_err(fmt, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_VERBOSE
#  define evt_verbose(fmt, ...) ninfo(fmt, ##__VA_ARGS__)
#else
#  define evt_verbose(fmt, ...)
#endif

#define evt_dbg evt_verbose

/* Maximum events to queue */

#define ESP_MAX_EVENT_QUEUE_SIZE   32

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* Queued event entry */

struct esp_queued_event
{
  sq_entry_t node;                 /* Queue node */
  uint8_t *data;                   /* Event data */
  size_t len;                      /* Event data length */
  uint32_t timestamp;              /* Time when event was queued */
};

/* Event queue */

struct esp_event_queue
{
  sq_queue_t events;               /* Queue of events */
  sem_t lock;                      /* Lock for queue access */
  uint8_t count;                   /* Current event count */
  uint8_t max_count;               /* Maximum allowed events */
};

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct esp_event_queue g_esp_evt_queue;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp_enqueue_event
 *
 * Description:
 *   Enqueue an event for later processing.
 *
 ****************************************************************************/

static int esp_enqueue_event(FAR const uint8_t *data, size_t len)
{
  FAR struct esp_queued_event *event;

  if (!data || len == 0)
    {
      return -EINVAL;
    }

  /* Check queue limits */

  if (g_esp_evt_queue.count >= g_esp_evt_queue.max_count)
    {
      evt_err("Event queue full (%d events)\n", g_esp_evt_queue.count);
      return -EBUSY;
    }

  /* Allocate event entry */

  event = kmm_malloc(sizeof(struct esp_queued_event));
  if (!event)
    {
      return -ENOMEM;
    }

  /* Allocate data buffer */

  event->data = kmm_malloc(len);
  if (!event->data)
    {
      kmm_free(event);
      return -ENOMEM;
    }

  /* Copy event data */

  memcpy(event->data, data, len);
  event->len = len;
  event->timestamp = clock_systime_ticks();  /* Get current system time */

  /* Lock queue and add event */

  nxsem_wait(&g_esp_evt_queue.lock);
  sq_addlast(&event->node, &g_esp_evt_queue.events);
  g_esp_evt_queue.count++;
  nxsem_post(&g_esp_evt_queue.lock);

  evt_verbose("Enqueued event: len=%zu, queue=%d\n", len, g_esp_evt_queue.count);

  return OK;
}

/****************************************************************************
 * Name: esp_dequeue_event
 *
 * Description:
 *   Dequeue an event for processing.
 *
 ****************************************************************************/

static FAR struct esp_queued_event *esp_dequeue_event(void)
{
  FAR struct esp_queued_event *event;

  nxsem_wait(&g_esp_evt_queue.lock);

  event = (FAR struct esp_queued_event *)sq_peek(&g_esp_evt_queue.events);
  if (event)
    {
      sq_rem(&event->node, &g_esp_evt_queue.events);
      g_esp_evt_queue.count--;
    }

  nxsem_post(&g_esp_evt_queue.lock);

  return event;
}

/****************************************************************************
 * Name: esp_process_mgmt_rx_event
 *
 * Description:
 *   Process management frame RX event from ESP32.
 *
 ****************************************************************************/

static int esp_process_mgmt_rx_event(FAR struct esp_adapter *adapter,
                                   FAR const uint8_t *data, size_t len)
{
  FAR struct esp_mgmt_event *event;
  FAR struct esp_wifi_device *priv;
  uint8_t frame_type;
  uint8_t channel;
  int32_t rssi;
  int32_t nf;  /* Noise floor */

  if (len < sizeof(struct esp_mgmt_event))
    {
      evt_err("Invalid management RX event length: %zu\n", len);
      return -EINVAL;
    }

  event = (FAR struct esp_mgmt_event *)data;

  /* Extract management frame info */

  frame_type = (event->frame[0] & 0xFC) >> 2;  /* FC[1:0] */
  channel = event->chan;
  rssi = event->rssi;
  nf = event->nf;

  evt_info("MGMT RX: type=%d, ch=%d, rssi=%d, nf=%d, len=%d\n",
           frame_type, channel, rssi, nf, event->frame_len);

  /* Find appropriate interface */

  priv = adapter->priv[ESP_STA_NW_IF];  /* Default to STA */
  if (!priv)
    {
      /* Try AP interface */
      priv = adapter->priv[ESP_AP_NW_IF];
    }

  if (!priv)
    {
      evt_err("No interface found\n");
      return -ENODEV;
    }

  /* Forward management frame to cfg80211 */
  if (priv && priv->wdev.netdev)
    {
      /* Forward frame to cfg80211 */
      cfg80211_rx_mgmt(&priv->wdev, channel, frame_type, frame, frame_len, rssi, 0);
    }

  return OK;
}

/****************************************************************************
 * Name: esp_process_auth_rx_event
 *
 * Description:
 *   Process authentication RX event from ESP32.
 *
 ****************************************************************************/

static int esp_process_auth_rx_event(FAR struct esp_adapter *adapter,
                                   FAR const uint8_t *data, size_t len)
{
  FAR struct esp_auth_event *event;
  uint8_t frame_type;
  uint8_t channel;
  int32_t rssi;
  uint8_t bssid[ESP_MAC_ADDR_LEN];

  if (len < sizeof(struct esp_auth_event))
    {
      evt_err("Invalid auth RX event length: %zu\n", len);
      return -EINVAL;
    }

  event = (FAR struct esp_auth_event *)data;

  /* Extract auth frame info */

  memcpy(bssid, event->bssid, ESP_MAC_ADDR_LEN);
  frame_type = event->frame_type;
  channel = event->channel;
  rssi = event->rssi;

  evt_info("AUTH RX: BSSID=%02x:%02x:%02x:%02x:%02x:%02x, "
            "frame_type=%d, ch=%d, rssi=%d\n",
            bssid[0], bssid[1], bssid[2], bssid[3], bssid[4], bssid[5],
            frame_type, channel, rssi);

  /* Find interface and forward to cfg80211 */

  if (priv && priv->wdev.netdev)
    {
      /* Forward frame to cfg80211 */
      cfg80211_rx_mgmt(&priv->wdev, channel, frame_type, frame, frame_len,
                      rssi, GFP_KERNEL);
    }

  return OK;
}

/****************************************************************************
 * Name: esp_process_assoc_rx_event
 *
 * Description:
 *   Process association RX event from ESP32.
 *
 ****************************************************************************/

static int esp_process_assoc_rx_event(FAR struct esp_adapter *adapter,
                                    FAR const uint8_t *data, size_t len)
{
  FAR struct esp_assoc_event *event;
  uint8_t frame_type;
  uint8_t channel;
  int32_t rssi;
  uint8_t bssid[ESP_MAC_ADDR_LEN];
  char ssid[ESP_MAX_SSID_LEN + 1];

  if (len < sizeof(struct esp_assoc_event))
    {
      evt_err("Invalid assoc RX event length: %zu\n", len);
      return -EINVAL;
    }

  event = (FAR struct esp_assoc_event *)data;

  /* Extract assoc frame info */

  memcpy(bssid, event->bssid, ESP_MAC_ADDR_LEN);
  frame_type = event->frame_type;
  channel = event->channel;
  rssi = event->rssi;
  strlcpy(ssid, event->ssid, sizeof(ssid));

  evt_info("ASSOC RX: BSSID=%02x:%02x:%02x:%02x:%02x:%02x, "
            "SSID=%s, frame_type=%d, ch=%d, rssi=%d\n",
            bssid[0], bssid[1], bssid[2], bssid[3], bssid[4], bssid[5],
            ssid, frame_type, channel, rssi);

  /* Find interface and forward to cfg80211 */
  
  if (priv && priv->wdev.netdev)
    {
      /* Forward frame to cfg80211 */
      cfg80211_rx_mgmt(&priv->wdev, channel, frame_type, frame, frame_len,
                      rssi, GFP_KERNEL);
    }

  return OK;
}

/****************************************************************************
 * Name: esp_event_worker
 *
 * Description:
 *   Worker thread for processing events from ESP32.
 *
 ****************************************************************************/

static void esp_event_worker(FAR void *arg)
{
  FAR struct esp_adapter *adapter = (FAR struct esp_adapter *)arg;
  FAR struct esp_queued_event *event;
  FAR struct esp_event_header *event_hdr;

  while (adapter && adapter->wiphy)
    {
      /* Dequeue an event */

      event = esp_dequeue_event();
      if (!event)
        {
          /* No events, sleep briefly */

          usleep(10000);  /* 10ms */
          continue;
        }

      /* Process event */

      if (event->len < sizeof(struct esp_event_header))
        {
          evt_err("Invalid event header length: %zu\n", event->len);
          goto next_event;
        }

      event_hdr = (FAR struct esp_event_header *)event->data;

      evt_verbose("Processing event: code=%d, status=%d, len=%d\n",
                  event_hdr->event_code, event_hdr->status, event_hdr->len);

      switch (event_hdr->event_code)
        {
          case ESP_EVENT_SCAN_RESULT:
            esp_process_scan_result_event(adapter, event->data, event->len);
            break;

          case ESP_EVENT_STA_CONNECT:
            esp_process_connect_event(adapter, event->data, event->len);
            break;

          case ESP_EVENT_STA_DISCONNECT:
            esp_process_disconnect_event(adapter, event->data, event->len);
            break;

          case ESP_EVENT_AUTH_RX:
            esp_process_auth_rx_event(adapter, event->data, event->len);
            break;

          case ESP_EVENT_ASSOC_RX:
            esp_process_assoc_rx_event(adapter, event->data, event->len);
            break;

          case ESP_EVENT_AP_MGMT_RX:
            esp_process_mgmt_rx_event(adapter, event->data, event->len);
            break;

          default:
            evt_info("Unknown event code: %d\n", event_hdr->event_code);
            break;
        }

next_event:
      /* Free event data */

      if (event->data)
        {
          kmm_free(event->data);
        }

      kmm_free(event);
    }
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp_event_queue_init
 *
 * Description:
 *   Initialize event queue.
 *
 ****************************************************************************/

int esp_event_queue_init(void)
{
  int ret;

  /* Initialize queue */

  sq_init(&g_esp_evt_queue.events);

  /* Initialize lock */

  ret = nxsem_init(&g_esp_evt_queue.lock, 0, 1);
  if (ret < 0)
    {
      return ret;
    }

  g_esp_evt_queue.count = 0;
  g_esp_evt_queue.max_count = ESP_MAX_EVENT_QUEUE_SIZE;

  evt_info("Event queue initialized, max=%d\n", g_esp_evt_queue.max_count);

  return OK;
}

/****************************************************************************
 * Name: esp_event_queue_deinit
 *
 * Description:
 *   Deinitialize event queue.
 *
 ****************************************************************************/

void esp_event_queue_deinit(void)
{
  /* Clear any remaining events */

  while (true)
    {
      FAR struct esp_queued_event *event = esp_dequeue_event();
      if (!event)
        {
          break;
        }

      if (event->data)
        {
          kmm_free(event->data);
        }

      kmm_free(event);
    }

  nxsem_destroy(&g_esp_evt_queue.lock);
  memset(&g_esp_evt_queue, 0, sizeof(g_esp_evt_queue));
}

/****************************************************************************
 * Name: esp_event_enqueue_event
 *
 * Description:
 *   Public function to enqueue an event.
 *
 ****************************************************************************/

int esp_event_enqueue_event(FAR const uint8_t *data, size_t len)
{
  return esp_enqueue_event(data, len);
}

/****************************************************************************
 * Name: esp_event_start_worker
 *
 * Description:
 *   Start the event processing worker.
 *
 ****************************************************************************/

int esp_event_start_worker(FAR struct esp_adapter *adapter)
{
  if (!adapter)
    {
      return -EINVAL;
    }

  /* Create work queue for event processing */
  adapter->event_wq = work_queue_create("esp_event", 4096);
  if (!adapter->event_wq)
    {
      evt_err("Failed to create event work queue\n");
      return -ENOMEM;
    }

  /* Start worker thread */
  return work_queue_start(adapter->event_wq, 50);  /* 50ms interval */
}

void esp_event_stop_worker(void)
{
  FAR struct esp_adapter *adapter = esp_get_adapter();

  if (adapter && adapter->event_wq)
    {
      work_queue_free(adapter->event_wq);
      adapter->event_wq = NULL;
    }
}

/****************************************************************************
 * Name: esp_handle_interrupt
 *
 * Description:
 *   Handle SDIO/SPI interrupt from ESP32.
 *
 ****************************************************************************/

void esp_handle_interrupt(FAR struct esp_adapter *adapter)
{
  uint32_t intr_status;
  int ret;

  if (!adapter)
    {
      return;
    }

  /* Get interrupt status from ESP32 */

  ret = esp_sdio_get_intr_status(adapter, &intr_status);
  if (ret < 0)
    {
      evt_err("Failed to get interrupt status: %d\n", ret);
      return;
    }

  evt_verbose("Interrupt status: 0x%08x\n", intr_status);

  /* Process interrupts */

  if (intr_status & ESP_SLAVE_RX_NEW_PACKET_INT)
    {
      /* Handle new packet available interrupt */

      evt_verbose("RX new packet interrupt\n");

      /* Process packets until no more are available */
      while (true)
        {
          uint8_t rx_buf[ESP_RX_BUFFER_SIZE];
          size_t rx_len;
          int pkt_ret;

          pkt_ret = esp_sdio_read_packet(adapter, rx_buf, sizeof(rx_buf), &rx_len);
          if (pkt_ret < 0 || rx_len == 0)
            {
              /* No more packets available */
              break;
            }

          /* Process received packet */

          if (rx_len >= sizeof(struct esp_payload_header))
            {
              FAR struct esp_payload_header *hdr =
                (FAR struct esp_payload_header *)rx_buf;

              evt_verbose("Received packet: type=%d, len=%d\n",
                         hdr->packet_type, hdr->len);

              if (hdr->packet_type == ESP_PACKET_TYPE_EVENT)
                {
                  /* Process event packet */
                  esp_process_event(adapter, rx_buf, rx_len);
                }
              else if (hdr->packet_type == ESP_PACKET_TYPE_COMMAND_RESPONSE)
                {
                  /* Process command response */
                  esp_process_command_response(adapter, rx_buf, rx_len);
                }
              else
                {
                  evt_info("Unknown packet type: %d\n", hdr->packet_type);
                }
            }
        }
    }

  if (intr_status & ESP_SLAVE_TX_OVERFLOW_INT)
    {
      evt_err("TX overflow interrupt\n");
      /* TX buffer overflow - report error and attempt recovery */
      /* Could signal network stack or reset TX path */
    }

  if (intr_status & ESP_SLAVE_RX_UNDERFLOW_INT)
    {
      evt_err("RX underflow interrupt\n");
      /* RX buffer underflow - report error */
      /* Could signal network stack or request retransmission */
    }

  /* Clear processed interrupts */

  esp_sdio_clear_intr(adapter, intr_status);
}

/****************************************************************************
 * Name: esp_schedule_scan_done
 *
 * Description:
 *   Schedule scan completion notification.
 *
 ****************************************************************************/

void esp_schedule_scan_done(FAR struct esp_wifi_device *priv, bool aborted)
{
  if (!priv || !priv->scan_request)
    {
      return;
    }

  /* Notify cfg80211 that scan is complete */

  esp_notify_scan_done(priv, aborted);
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

/****************************************************************************
 * Name: esp_notify_new_station
 *
 * Description:
 *   Notify cfg80211 of new station.
 *
 ****************************************************************************/

void esp_notify_new_station(FAR struct esp_wifi_device *priv,
                           FAR const uint8_t *mac_addr,
                           FAR struct station_info *sinfo)
{
  if (priv && priv->wdev.netdev)
    {
      cfg80211_new_sta(priv->wdev.netdev, mac_addr, sinfo, 0);
    }
}

/****************************************************************************
 * Name: esp_notify_del_station
 *
 * Description:
 *   Notify cfg80211 of deleted station.
 *
 ****************************************************************************/

void esp_notify_del_station(FAR struct esp_wifi_device *priv,
                           FAR const uint8_t *mac_addr)
{
  if (priv && priv->wdev.netdev)
    {
      cfg80211_del_sta(priv->wdev.netdev, mac_addr, 0);
    }
}

/****************************************************************************
 * Name: esp_notify_mgmt_frame
 *
 * Description:
 *   Notify cfg80211 of management frame.
 *
 ****************************************************************************/

void esp_notify_mgmt_frame(FAR struct esp_wifi_device *priv,
                           FAR const struct ieee80211_rx_status *rx_status,
                           FAR const uint8_t *frame, size_t len)
{
  if (priv && priv->wdev.netdev && rx_status)
    {
      /* Forward management frame to cfg80211 */
      cfg80211_rx_mgmt(&priv->wdev, rx_status,
                       frame, len, 0, GFP_KERNEL);
    }
}

/****************************************************************************
 * Name: esp_update_bss_info
 *
 * Description:
 *   Update BSS information in cfg80211.
 *
 ****************************************************************************/

void esp_update_bss_info(FAR struct esp_wifi_device *priv,
                         FAR const uint8_t *bssid,
                         FAR const uint8_t *ie, size_t ie_len,
                         int channel, int signal)
{
  if (priv && priv->wdev.wiphy)
    {
      FAR struct wiphy *wiphy = priv->wdev.wiphy;
      uint16_t chan_freq = 2407 + (channel - 1) * 5;  /* 2.4GHz */
      
      /* Create channel entry */
      struct ieee80211_channel chan =
        {
          .band = IEEE80211_BAND_2GHZ,
          .center_freq = chan_freq,
          .hw_value = channel,
          .max_power = 20,
        };

      /* Inform cfg80211 of BSS */
      cfg80211_inform_bss(wiphy, &chan, bssid,
                           ie, ie_len, signal, GFP_KERNEL);
    }
}