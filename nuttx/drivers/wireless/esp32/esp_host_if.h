/****************************************************************************
 * drivers/wireless/esp32/esp_host_if.h
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
 * ESP32 Host Interface Header
 *
 * This file defines the interface between ESP32 and the host system.
 * Based on Espressif's esp-hosted-ng architecture.
 ****************************************************************************/

#ifndef __DRIVERS_WIRELESS_ESP32_ESP_HOST_IF_H
#define __DRIVERS_WIRELESS_ESP32_ESP_HOST_IF_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <nuttx/compiler.h>

#include <stdint.h>
#include <stdbool.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* ESP32 SDIO/SPI Register Addresses */

/* SDIO Slave registers (ESP32_SLCHOST_BASE = 0x3FF55000) */

#define ESP_SLAVE_SLCHOST_BASE           0x3FF55000

/* Interrupt registers */

#define ESP_SLAVE_INT_RAW_REG           (ESP_SLAVE_SLCHOST_BASE + 0x50)
#define ESP_SLAVE_INT_ST_REG            (ESP_SLAVE_SLCHOST_BASE + 0x58)
#define ESP_SLAVE_INT_CLR_REG           (ESP_SLAVE_SLCHOST_BASE + 0xD4)

/* Data path registers */

#define ESP_SLAVE_PACKET_LEN_REG         (ESP_SLAVE_SLCHOST_BASE + 0x60)
#define ESP_SLAVE_TOKEN_RDATA            (ESP_SLAVE_SLCHOST_BASE + 0x44)

/* Scratch registers */

#define ESP_SLAVE_SCRATCH_REG_0          (ESP_SLAVE_SLCHOST_BASE + 0x6C)
#define ESP_SLAVE_SCRATCH_REG_1          (ESP_SLAVE_SLCHOST_BASE + 0x70)
#define ESP_SLAVE_SCRATCH_REG_2          (ESP_SLAVE_SLCHOST_BASE + 0x74)
#define ESP_SLAVE_SCRATCH_REG_3          (ESP_SLAVE_SLCHOST_BASE + 0x78)
#define ESP_SLAVE_SCRATCH_REG_4          (ESP_SLAVE_SLCHOST_BASE + 0x7C)
#define ESP_SLAVE_SCRATCH_REG_6          (ESP_SLAVE_SLCHOST_BASE + 0x88)
#define ESP_SLAVE_SCRATCH_REG_7          (ESP_SLAVE_SLCHOST_BASE + 0x8C)
#define ESP_SLAVE_SCRATCH_REG_8          (ESP_SLAVE_SLCHOST_BASE + 0x9C)
#define ESP_SLAVE_SCRATCH_REG_9          (ESP_SLAVE_SLCHOST_BASE + 0xA0)
#define ESP_SLAVE_SCRATCH_REG_10         (ESP_SLAVE_SLCHOST_BASE + 0xA4)
#define ESP_SLAVE_SCRATCH_REG_11         (ESP_SLAVE_SLCHOST_BASE + 0xA8)
#define ESP_SLAVE_SCRATCH_REG_12         (ESP_SLAVE_SLCHOST_BASE + 0xAC)
#define ESP_SLAVE_SCRATCH_REG_13         (ESP_SLAVE_SLCHOST_BASE + 0xB0)
#define ESP_SLAVE_SCRATCH_REG_14         (ESP_SLAVE_SLCHOST_BASE + 0xB4)
#define ESP_SLAVE_SCRATCH_REG_15         (ESP_SLAVE_SLCHOST_BASE + 0xB8)

/* Interrupt bits */

#define ESP_SLAVE_BIT0_INT               (1 << 0)
#define ESP_SLAVE_BIT1_INT               (1 << 1)
#define ESP_SLAVE_BIT2_INT               (1 << 2)
#define ESP_SLAVE_BIT3_INT               (1 << 3)
#define ESP_SLAVE_BIT4_INT               (1 << 4)
#define ESP_SLAVE_BIT5_INT               (1 << 5)
#define ESP_SLAVE_BIT6_INT               (1 << 6)
#define ESP_SLAVE_BIT7_INT               (1 << 7)
#define ESP_SLAVE_RX_UNDERFLOW_INT        (1 << 16)
#define ESP_SLAVE_TX_OVERFLOW_INT         (1 << 17)
#define ESP_SLAVE_RX_NEW_PACKET_INT       (1 << 23)

/* SDIO Constants */

#define ESP_SLAVE_CMD53_END_ADDR          0x1F800
#define ESP_BLOCK_SIZE                    512
#define ESP_RX_BYTE_MAX                   0x100000
#define ESP_RX_BUFFER_SIZE                2048
#define ESP_TX_BUFFER_MASK                0xFFF
#define ESP_TX_BUFFER_MAX                 0x1000
#define ESP_MAX_BUF_CNT                   10
#define ESP_SLAVE_LEN_MASK                0xFFFFF
#define ESP_ADDRESS_MASK                  0x3FF

/* ESP32 Device IDs */

#define ESP_VENDOR_ID_1                  0x6666
#define ESP_DEVICE_ID_ESP32_1            0x2222
#define ESP_DEVICE_ID_ESP32_2            0x3333
#define ESP_VENDOR_ID_2                  0x0092
#define ESP_DEVICE_ID_C5_C6_C61_1        0x6666
#define ESP_DEVICE_ID_C5_C6_C61_2        0x7777

/* SPI Protocol Constants */

#define ESP_SPI_HEADER_LEN               4
#define ESP_SPI_MAX_PKT_SIZE             4096

/* TX Buffer thresholds */

#define ESP_TX_MAX_PENDING_COUNT         200
#define ESP_TX_RESUME_THRESHOLD           40

/* SDIO Function number for ESP */

#define ESP_SDIO_FUNC_WIFI               1
#define ESP_SDIO_FUNC_BT                 2

/* Host interrupt to ESP */

#define ESP_HOST_INT_BIT0                0x01
#define ESP_HOST_INT_BIT1                0x02
#define ESP_HOST_INT_BIT2                0x04
#define ESP_HOST_INT_BIT3                0x08

/****************************************************************************
 * Types
 ****************************************************************************/

/* ESP32 host interface context */

struct esp_host_if_ctx
{
  /* Interface type: SDIO or SPI */

  uint8_t if_type;

  /* For SDIO interface */

#ifdef CONFIG_ESP32_WIFI_SDIO
  struct sdio_spi_dev_s *sdio_dev;
  int sdio_func;
#endif

  /* For SPI interface */

#ifdef CONFIG_ESP32_WIFI_SPI
  struct spi_dev_s *spi_dev;
  int gpio_irq;
  int gpio_cs;
  int gpio_handshake;
#endif

  /* TX/RX buffers */

  uint8_t *tx_buf;
  uint8_t *rx_buf;
  size_t tx_buf_size;
  size_t rx_buf_size;

  /* Statistics */

  uint32_t tx_count;
  uint32_t rx_count;
  uint32_t tx_errors;
  uint32_t rx_errors;

  /* Lock */

  sem_t lock;
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

#ifdef __cplusplus
extern "C"
{
#endif

/* ESP32 SDIO Interface */

/****************************************************************************
 * Name: esp_sdio_init
 *
 * Description:
 *   Initialize ESP32 SDIO interface.
 *
 * Input Parameters:
 *   ctx - ESP32 host interface context
 *
 * Returned Value:
 *   OK on success; negated errno on failure
 *
 ****************************************************************************/

int esp_sdio_init(FAR struct esp_host_if_ctx *ctx);

/****************************************************************************
 * Name: esp_sdio_deinit
 *
 * Description:
 *   Deinitialize ESP32 SDIO interface.
 *
 * Input Parameters:
 *   ctx - ESP32 host interface context
 *
 ****************************************************************************/

void esp_sdio_deinit(FAR struct esp_host_if_ctx *ctx);

/****************************************************************************
 * Name: esp_sdio_read_reg
 *
 * Description:
 *   Read from ESP32 SDIO register.
 *
 * Input Parameters:
 *   ctx - ESP32 host interface context
 *   reg - Register address
 *   data - Buffer to store read data
 *   len - Number of bytes to read
 *
 * Returned Value:
 *   OK on success; negated errno on failure
 *
 ****************************************************************************/

int esp_sdio_read_reg(FAR struct esp_host_if_ctx *ctx, uint32_t reg,
                     FAR uint8_t *data, uint16_t len);

/****************************************************************************
 * Name: esp_sdio_write_reg
 *
 * Description:
 *   Write to ESP32 SDIO register.
 *
 * Input Parameters:
 *   ctx - ESP32 host interface context
 *   reg - Register address
 *   data - Data to write
 *   len - Number of bytes to write
 *
 * Returned Value:
 *   OK on success; negated errno on failure
 *
 ****************************************************************************/

int esp_sdio_write_reg(FAR struct esp_host_if_ctx *ctx, uint32_t reg,
                      FAR const uint8_t *data, uint16_t len);

/****************************************************************************
 * Name: esp_sdio_read_packet
 *
 * Description:
 *   Read a packet from ESP32.
 *
 * Input Parameters:
 *   ctx - ESP32 host interface context
 *   buf - Buffer to store packet data
 *   maxlen - Maximum buffer size
 *   out_len - Actual packet length (output)
 *
 * Returned Value:
 *   OK on success; negated errno on failure
 *
 ****************************************************************************/

int esp_sdio_read_packet(FAR struct esp_host_if_ctx *ctx,
                       FAR uint8_t *buf, size_t maxlen,
                       FAR size_t *out_len);

/****************************************************************************
 * Name: esp_sdio_write_packet
 *
 * Description:
 *   Write a packet to ESP32.
 *
 * Input Parameters:
 *   ctx - ESP32 host interface context
 *   buf - Packet data to write
 *   len - Packet length
 *
 * Returned Value:
 *   OK on success; negated errno on failure
 *
 ****************************************************************************/

int esp_sdio_write_packet(FAR struct esp_host_if_ctx *ctx,
                         FAR const uint8_t *buf, size_t len);

/****************************************************************************
 * Name: esp_sdio_send_interrupt
 *
 * Description:
 *   Send an interrupt to ESP32.
 *
 * Input Parameters:
 *   ctx - ESP32 host interface context
 *   intr_mask - Interrupt mask to send
 *
 * Returned Value:
 *   OK on success; negated errno on failure
 *
 ****************************************************************************/

int esp_sdio_send_interrupt(FAR struct esp_host_if_ctx *ctx, uint8_t intr_mask);

/****************************************************************************
 * Name: esp_sdio_get_intr_status
 *
 * Description:
 *   Get ESP32 interrupt status.
 *
 * Input Parameters:
 *   ctx - ESP32 host interface context
 *   intr_mask - Interrupt mask (output)
 *
 * Returned Value:
 *   OK on success; negated errno on failure
 *
 ****************************************************************************/

int esp_sdio_get_intr_status(FAR struct esp_host_if_ctx *ctx,
                            FAR uint32_t *intr_mask);

/****************************************************************************
 * Name: esp_sdio_clear_intr
 *
 * Description:
 *   Clear ESP32 interrupts.
 *
 * Input Parameters:
 *   ctx - ESP32 host interface context
 *   intr_mask - Interrupt mask to clear
 *
 * Returned Value:
 *   OK on success; negated errno on failure
 *
 ****************************************************************************/

int esp_sdio_clear_intr(FAR struct esp_host_if_ctx *ctx, uint32_t intr_mask);

/* ESP32 SPI Interface */

/****************************************************************************
 * Name: esp_spi_init
 *
 * Description:
 *   Initialize ESP32 SPI interface.
 *
 * Input Parameters:
 *   ctx - ESP32 host interface context
 *
 * Returned Value:
 *   OK on success; negated errno on failure
 *
 ****************************************************************************/

int esp_spi_init(FAR struct esp_host_if_ctx *ctx);

/****************************************************************************
 * Name: esp_spi_deinit
 *
 * Description:
 *   Deinitialize ESP32 SPI interface.
 *
 * Input Parameters:
 *   ctx - ESP32 host interface context
 *
 ****************************************************************************/

void esp_spi_deinit(FAR struct esp_host_if_ctx *ctx);

/****************************************************************************
 * Name: esp_spi_read_reg
 *
 * Description:
 *   Read from ESP32 SPI register.
 *
 * Input Parameters:
 *   ctx - ESP32 host interface context
 *   reg - Register address
 *   data - Buffer to store read data
 *   len - Number of bytes to read
 *
 * Returned Value:
 *   OK on success; negated errno on failure
 *
 ****************************************************************************/

int esp_spi_read_reg(FAR struct esp_host_if_ctx *ctx, uint32_t reg,
                    FAR uint8_t *data, uint16_t len);

/****************************************************************************
 * Name: esp_spi_write_reg
 *
 * Description:
 *   Write to ESP32 SPI register.
 *
 * Input Parameters:
 *   ctx - ESP32 host interface context
 *   reg - Register address
 *   data - Data to write
 *   len - Number of bytes to write
 *
 * Returned Value:
 *   OK on success; negated errno on failure
 *
 ****************************************************************************/

int esp_spi_write_reg(FAR struct esp_host_if_ctx *ctx, uint32_t reg,
                     FAR const uint8_t *data, uint16_t len);

/****************************************************************************
 * Name: esp_spi_read_packet
 *
 * Description:
 *   Read a packet from ESP32 via SPI.
 *
 * Input Parameters:
 *   ctx - ESP32 host interface context
 *   buf - Buffer to store packet data
 *   maxlen - Maximum buffer size
 *   out_len - Actual packet length (output)
 *
 * Returned Value:
 *   OK on success; negated errno on failure
 *
 ****************************************************************************/

int esp_spi_read_packet(FAR struct esp_host_if_ctx *ctx,
                       FAR uint8_t *buf, size_t maxlen,
                       FAR size_t *out_len);

/****************************************************************************
 * Name: esp_spi_write_packet
 *
 * Description:
 *   Write a packet to ESP32 via SPI.
 *
 * Input Parameters:
 *   ctx - ESP32 host interface context
 *   buf - Packet data to write
 *   len - Packet length
 *
 * Returned Value:
 *   OK on success; negated errno on failure
 *
 ****************************************************************************/

int esp_spi_write_packet(FAR struct esp_host_if_ctx *ctx,
                        FAR const uint8_t *buf, size_t len);

/****************************************************************************
 * Name: esp_spi_send_interrupt
 *
 * Description:
 *   Send an interrupt to ESP32 via SPI.
 *
 * Input Parameters:
 *   ctx - ESP32 host interface context
 *   intr_mask - Interrupt mask to send
 *
 * Returned Value:
 *   OK on success; negated errno on failure
 *
 ****************************************************************************/

int esp_spi_send_interrupt(FAR struct esp_host_if_ctx *ctx, uint8_t intr_mask);

/****************************************************************************
 * Name: esp_spi_wait_for_handshake
 *
 * Description:
 *   Wait for ESP32 handshake signal.
 *
 * Input Parameters:
 *   ctx - ESP32 host interface context
 *   timeout - Timeout in milliseconds
 *
 * Returned Value:
 *   OK on success; negated errno on failure
 *
 ****************************************************************************/

int esp_spi_wait_for_handshake(FAR struct esp_host_if_ctx *ctx,
                              uint32_t timeout);

#ifdef __cplusplus
}
#endif

#endif /* __DRIVERS_WIRELESS_ESP32_ESP_HOST_IF_H */