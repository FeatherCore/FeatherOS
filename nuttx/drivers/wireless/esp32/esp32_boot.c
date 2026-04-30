/****************************************************************************
 * drivers/wireless/esp32/esp32_boot.c
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
 * ESP32 Firmware Boot and Capability Negotiation
 *
 * This module handles the ESP32 firmware bootup sequence, capability
 * negotiation, and chipset detection.
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
#include <nuttx/gpio.h>

#include "esp_cfg80211.h"
#include "esp_host_if.h"

#ifdef CONFIG_ESP32_WIFI

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#ifdef CONFIG_DEBUG_WIRELESS_INFO
#  define boot_info(format, ...)  ninfo(format, ##__VA_ARGS__)
#else
#  define boot_info(format, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_ERROR
#  define boot_err(format, ...)   nerr(format, ##__VA_ARGS__)
#else
#  define boot_err(format, ...)
#endif

#define ESP_BOOTUP_TIMEOUT_MS         5000
#define ESP_RESET_DELAY_MS            100
#define ESP_INIT_DELAY_MS             500

#define ESP_MAX_BOOTUP_RETRIES        3

/****************************************************************************
 * Private Types
 ****************************************************************************/

enum esp_bootup_tag_type
{
  ESP_BOOTUP_CAPABILITY = 0,
  ESP_BOOTUP_FW_DATA,
  ESP_BOOTUP_SPI_CLK_MHZ,
  ESP_BOOTUP_FIRMWARE_CHIP_ID,
  ESP_BOOTUP_TEST_RAW_TP,
};

struct esp_bootup_event
{
  uint8_t event_code;
  uint8_t status;
  uint16_t len;
  uint8_t data[];
} __packed;

struct esp_capability_tlv
{
  uint8_t tag;
  uint8_t len;
  uint8_t value[];
} __packed;

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int esp_parse_bootup_event(struct esp_adapter *adapter,
                                   FAR const uint8_t *data, size_t len);

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static const char *esp_chipname_from_id(uint8_t chipset)
{
  switch (chipset)
    {
      case ESP_CHIPSET_ESP32:
        return "ESP32";
      case ESP_CHIPSET_ESP32S2:
        return "ESP32-S2";
      case ESP_CHIPSET_ESP32C3:
        return "ESP32-C3";
      case ESP_CHIPSET_ESP32S3:
        return "ESP32-S3";
      case ESP_CHIPSET_ESP32C2:
        return "ESP32-C2";
      case ESP_CHIPSET_ESP32C6:
        return "ESP32-C6";
      case ESP_CHIPSET_ESP32C61:
        return "ESP32-C61";
      case ESP_CHIPSET_ESP32C5:
        return "ESP32-C5";
      default:
        return "Unknown";
    }
}

static int esp_parse_capability_tlv(struct esp_adapter *adapter,
                                     FAR const uint8_t *data, size_t len)
{
  FAR const struct esp_capability_tlv *tlv;
  size_t offset = 0;

  while (offset + sizeof(struct esp_capability_tlv) <= len)
    {
      tlv = (FAR const struct esp_capability_tlv *)(data + offset);

      if (offset + sizeof(struct esp_capability_tlv) + tlv->len > len)
        {
          break;
        }

      switch (tlv->tag)
        {
          case ESP_BOOTUP_CAPABILITY:
            if (tlv->len >= sizeof(uint32_t))
              {
                adapter->capabilities = *(uint32_t *)tlv->value;
                boot_info("Capabilities: 0x%08x\n", adapter->capabilities);
              }
            break;

          case ESP_BOOTUP_FIRMWARE_CHIP_ID:
            if (tlv->len >= sizeof(uint8_t))
              {
                adapter->chipset = tlv->value[0];
                boot_info("Chipset: %s (ID=0x%02x)\n",
                          esp_chipname_from_id(adapter->chipset),
                          adapter->chipset);
              }
            break;

          case ESP_BOOTUP_SPI_CLK_MHZ:
            if (tlv->len >= sizeof(uint8_t))
              {
                boot_info("SPI Clock: %d MHz\n", tlv->value[0]);
              }
            break;

          case ESP_BOOTUP_TEST_RAW_TP:
            boot_info("Raw throughput test mode\n");
            break;

          default:
            boot_info("Unknown TLV tag: %d, len: %d\n", tlv->tag, tlv->len);
            break;
        }

      offset += sizeof(struct esp_capability_tlv) + tlv->len;
    }

  return OK;
}

static int esp_parse_bootup_event(struct esp_adapter *adapter,
                                   FAR const uint8_t *data, size_t len)
{
  FAR const struct esp_bootup_event *event;
  size_t payload_len;

  if (!adapter || !data || len < sizeof(struct esp_bootup_event))
    {
      boot_err("Invalid bootup event\n");
      return -EINVAL;
    }

  event = (FAR const struct esp_bootup_event *)data;
  payload_len = event->len;

  if (len < sizeof(struct esp_bootup_event) + payload_len)
    {
      boot_err("Bootup event truncated\n");
      return -EINVAL;
    }

  boot_info("Bootup event: code=%d status=%d len=%d\n",
            event->event_code, event->status, event->len);

  return esp_parse_capability_tlv(adapter, event->data, payload_len);
}

static int esp_wait_for_bootup_event(struct esp_adapter *adapter)
{
  uint8_t *rx_buf;
  size_t rx_len;
  int ret;
  int retries = 0;

  rx_buf = kmm_malloc(ESP_RX_BUFFER_SIZE);
  if (!rx_buf)
    {
      return -ENOMEM;
    }

  while (retries < ESP_MAX_BOOTUP_RETRIES)
    {
      rx_len = ESP_RX_BUFFER_SIZE;

      if (adapter->if_ops && adapter->if_ops->receive)
        {
          ret = adapter->if_ops->receive(adapter, rx_buf, rx_len);
          if (ret >= 0)
            {
              struct esp_payload_header *hdr;

              hdr = (struct esp_payload_header *)rx_buf;

              if (hdr->packet_type == ESP_PACKET_TYPE_EVENT)
                {
                  ret = esp_parse_bootup_event(adapter,
                                               rx_buf + hdr->offset,
                                               hdr->len);
                  if (ret == OK)
                    {
                      kmm_free(rx_buf);
                      return OK;
                    }
                }
            }
        }

      retries++;
      usleep(ESP_INIT_DELAY_MS * 1000);
    }

  kmm_free(rx_buf);
  boot_err("Bootup event timeout\n");
  return -ETIMEDOUT;
}

static int esp_hardware_reset(struct esp_adapter *adapter)
{
#ifdef CONFIG_ESP32_WIFI_RESET_GPIO
  int reset_gpio = CONFIG_ESP32_WIFI_RESET_GPIO;

  boot_info("Resetting ESP32 via GPIO %d\n", reset_gpio);

  gpio_direction_out(reset_gpio, 0);
  usleep(ESP_RESET_DELAY_MS * 1000);
  gpio_direction_out(reset_gpio, 1);
  usleep(ESP_INIT_DELAY_MS * 1000);

  boot_info("ESP32 reset complete\n");
#endif

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int esp32_boot_init(struct esp_adapter *adapter)
{
  int ret;

  if (!adapter)
    {
      return -EINVAL;
    }

  boot_info("Initializing ESP32 boot sequence\n");

  adapter->chipset = ESP_CHIPSET_UNRECOGNIZED;
  adapter->capabilities = 0;

  ret = esp_hardware_reset(adapter);
  if (ret < 0)
    {
      boot_err("Hardware reset failed: %d\n", ret);
      return ret;
    }

  ret = esp_wait_for_bootup_event(adapter);
  if (ret < 0)
    {
      boot_err("Bootup event failed: %d\n", ret);
      return ret;
    }

  if (adapter->chipset == ESP_CHIPSET_UNRECOGNIZED)
    {
      boot_err("Unrecognized ESP chipset\n");
      return -ENODEV;
    }

  boot_info("ESP32 boot complete: %s, capabilities=0x%08x\n",
            esp_chipname_from_id(adapter->chipset),
            adapter->capabilities);

  return OK;
}

int esp32_check_capabilities(struct esp_adapter *adapter)
{
  if (!adapter)
    {
      return -EINVAL;
    }

  if (adapter->if_type == ESP_IF_TYPE_SDIO)
    {
      if (!(adapter->capabilities & ESP_WLAN_SDIO_SUPPORT))
        {
          boot_err("SDIO not supported by firmware\n");
          return -ENOTSUP;
        }
    }
  else if (adapter->if_type == ESP_IF_TYPE_SPI)
    {
      if (!(adapter->capabilities & ESP_WLAN_SPI_SUPPORT))
        {
          boot_err("SPI not supported by firmware\n");
          return -ENOTSUP;
        }
    }

  return OK;
}

bool esp_checksum_enabled(struct esp_adapter *adapter)
{
  if (adapter && (adapter->capabilities & ESP_CHECKSUM_ENABLED))
    {
      return true;
    }
  return false;
}

const char *esp_get_chipset_name(struct esp_adapter *adapter)
{
  if (adapter)
    {
      return esp_chipname_from_id(adapter->chipset);
    }
  return "Unknown";
}

#endif /* CONFIG_ESP32_WIFI */