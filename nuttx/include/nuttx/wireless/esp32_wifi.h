/****************************************************************************
 * include/nuttx/wireless/esp32_wifi.h
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

#ifndef __INCLUDE_NUTTX_WIRELESS_ESP32_WIFI_H
#define __INCLUDE_NUTTX_WIRELESS_ESP32_WIFI_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <nuttx/net/wireless.h>
#include <nuttx/wireless/cfg80211.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* ESP32 SPI constants */

#define ESP32_SPI_READ_REG_CMD           0x13
#define ESP32_SPI_WRITE_REG_CMD          0x12
#define ESP32_SPI_READ_FIFO_CMD          0x23
#define ESP32_SPI_WRITE_FIFO_CMD         0x22

#define ESP32_SPI_HDR_CMD_SHIFT          0
#define ESP32_SPI_HDR_ADDR_SHIFT         6
#define ESP32_SPI_HDR_LEN_SHIFT          25

#define ESP32_SPI_READ_CMD               0x03
#define ESP32_SPI_WRITE_CMD              0x02

/* SPI register addresses */

#define ESP32_SPI_SLCHOST_INT_RAW_REG    0x60000080
#define ESP32_SPI_SLCHOST_INT_ST_REG     0x60000084
#define ESP32_SPI_SLCHOST_INT_CLR_REG    0x60000088
#define ESP32_SPI_SLCHOST_PKT_LEN_REG    0x6000008C

/* SPI header shifts and masks */

#define ESP32_SPI_HDR_ADDR_SHIFT         6
#define ESP32_SPI_HDR_LEN_SHIFT          25
#define ESP32_SPI_ADDR_MASK              0x7FFFF
#define ESP32_SPI_LEN_MASK               0x7F

/* Interrupt flags */

#define ESP32_SPI_SLAVE_RX_NEW_PACKET_INT    (1 << 23)
#define ESP32_SPI_SLAVE_RX_NEW_PACKET_MASK   0x00800000

/* Interrupt flags */

#define ESP32_SPI_SLAVE_RX_NEW_PACKET_INT    (1 << 23)
#define ESP32_SPI_SLAVE_RX_NEW_PACKET_MASK   0x00800000

/* Interface types */

#define ESP32_STA_IF                     0
#define ESP32_AP_IF                      1

/* Interface numbers */

#define ESP32_STA_NW_IF                  0
#define ESP32_AP_NW_IF                   1

/* Maximum values */

#define ESP32_SSID_MAX_LEN               32
#define ESP32_BSSID_LEN                  6

/* Command timeouts (in milliseconds) */

#define ESP32_CMD_TIMEOUT_MS             5000

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* ESP32 WiFi status */

struct esp32_wifi_status_s
{
  bool connected;                    /* Connection state */
  uint8_t ssid[ESP32_SSID_MAX_LEN]; /* SSID */
  size_t ssid_len;                  /* SSID length */
  uint8_t bssid[ESP32_BSSID_LEN];  /* BSSID */
  int8_t rssi;                     /* RSSI in dBm */
  int8_t noise;                    /* Noise level in dBm */
  uint32_t bitrate;                /* Bitrate in kbps */
  uint8_t channel;                 /* Current channel */
  uint8_t mode;                    /* Operation mode */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

#ifdef __cplusplus
extern "C"
{
#endif

/* ESP32 WiFi driver functions */

int esp32_wifi_initialize(void);
int esp32_wifi_uninitialize(void);

/* ESP32 SPI functions */

int esp32_spi_initialize(FAR struct spi_dev_s *spi_dev, int cs_gpio, int irq_gpio);
int esp32_spi_uninitialize(void);
int esp32_spi_read_packet(FAR uint8_t *buffer, FAR size_t *buf_len);
int esp32_spi_write_packet(FAR const uint8_t *buffer, size_t buf_len);
int esp32_spi_get_intr_status(FAR uint32_t *intr_status);
int esp32_spi_clear_intr(uint32_t intr_mask);

/* ESP32 SDIO functions */

int esp32_sdio_initialize(FAR struct sdio_dev_s *sdio_dev, uint8_t func_num);
int esp32_sdio_uninitialize(void);
int esp32_sdio_read_packet(FAR uint8_t *buffer, FAR size_t *buf_len);
int esp32_sdio_write_packet(FAR const uint8_t *buffer, size_t buf_len);
int esp32_sdio_get_intr_status(FAR uint32_t *intr_status);
int esp32_sdio_clear_intr(uint32_t intr_mask);

/* Scan functions */

int esp32_wifi_scan_request(void);
int esp32_wifi_get_scan_results(void);

/* Connection functions */

int esp32_wifi_connect_request(FAR const char *ssid, size_t ssid_len,
                              FAR const uint8_t *bssid);
int esp32_wifi_disconnect_request(void);
int esp32_wifi_get_status(FAR struct esp32_wifi_status_s *status);

#ifdef __cplusplus
}
#endif

#endif /* __INCLUDE_NUTTX_WIRELESS_ESP32_WIFI_H */