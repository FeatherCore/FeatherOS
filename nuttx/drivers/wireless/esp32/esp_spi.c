/****************************************************************************
 * drivers/wireless/esp32/esp_spi.c
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
#include <string.h>
#include <semaphore.h>
#include <errno.h>
#include <debug.h>

#include <nuttx/kmalloc.h>
#include <nuttx/semaphore.h>
#include <nuttx/spi/spi.h>
#include <nuttx/gpio.h>

#ifdef CONFIG_ESP32_WIFI_SPI

#include "esp_cfg80211.h"
#include "esp_host_if.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Debug output */

#ifdef CONFIG_DEBUG_WIRELESS_INFO
#  define spi_info(fmt, ...)  ninfo(fmt, ##__VA_ARGS__)
#else
#  define spi_info(fmt, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_ERROR
#  define spi_err(fmt, ...)   nerr(fmt, ##__VA_ARGS__)
#else
#  define spi_err(fmt, ...)
#endif

/* SPI protocol definitions */

#define ESP_SPI_CMD_READ                    0x03
#define ESP_SPI_CMD_WRITE                   0x02
#define ESP_SPI_CMD_READ_REG                0x13
#define ESP_SPI_CMD_WRITE_REG               0x12
#define ESP_SPI_CMD_READ_FIFO               0x23
#define ESP_SPI_CMD_WRITE_FIFO              0x22
#define ESP_SPI_CMD_WRITE_END               0x04

/* SPI header definitions */

#define ESP_SPI_HEADER_LEN                  4
#define ESP_SPI_MAX_PKT_SIZE               4096

/* GPIO pin definitions for handshake */

#define ESP_SPI_HANDSHAKE_PIN              CONFIG_ESP32_SPI_HANDSHAKE_PIN

/* SPI transfer timeout (in ms) */

#define ESP_SPI_TRANSFER_TIMEOUT            1000

/* Maximum retries for SPI operations */

#define ESP_SPI_MAX_RETRIES                 3

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* ESP32 SPI context */

struct esp_spi_context
{
  struct spi_dev_s *spi_dev;       /* SPI device */
  int irq_gpio;                    /* IRQ GPIO pin */
  int cs_gpio;                     /* CS GPIO pin */
  int handshake_gpio;              /* Handshake GPIO pin */
  bool initialized;                /* Initialization flag */
  uint32_t rx_byte_count;          /* RX byte counter */
  uint32_t tx_buffer_count;        /* TX buffer counter */
  sem_t lock;                      /* Lock for operations */
  sem_t handshake_sem;             /* Handshake semaphore */
};

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct esp_spi_context g_esp_spi_ctx;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp_spi_claim
 *
 * Description:
 *   Claim SPI device for exclusive access.
 *
 ****************************************************************************/

static void esp_spi_claim(FAR struct esp_spi_context *ctx)
{
  if (ctx && ctx->spi_dev)
    {
      SPI_LOCK(ctx->spi_dev, true);
    }
}

/****************************************************************************
 * Name: esp_spi_release
 *
 * Description:
 *   Release SPI device.
 *
 ****************************************************************************/

static void esp_spi_release(FAR struct esp_spi_context *ctx)
{
  if (ctx && ctx->spi_dev)
    {
      SPI_LOCK(ctx->spi_dev, false);
    }
}

/****************************************************************************
 * Name: esp_spi_setup
 *
 * Description:
 *   Configure SPI device for ESP32 communication.
 *
 ****************************************************************************/

static void esp_spi_setup(FAR struct esp_spi_context *ctx)
{
  if (ctx && ctx->spi_dev)
    {
      /* Configure SPI for ESP32 */

      SPI_SETFREQUENCY(ctx->spi_dev, CONFIG_ESP32_SPI_FREQ);
      SPI_SETMODE(ctx->spi_dev, SPIDEV_MODE0);
      SPI_SETBITS(ctx->spi_dev, 8);

      /* Make sure CS is deselected initially */

      if (ctx->cs_gpio >= 0)
        {
          gpio_write(ctx->cs_gpio, 1);
        }
    }
}

/****************************************************************************
 * Name: esp_spi_select
 *
 * Description:
 *   Select ESP32 SPI device.
 *
 ****************************************************************************/

static void esp_spi_select(FAR struct esp_spi_context *ctx)
{
  if (ctx && ctx->cs_gpio >= 0)
    {
      gpio_write(ctx->cs_gpio, 0);
    }
}

/****************************************************************************
 * Name: esp_spi_deselect
 *
 * Description:
 *   Deselect ESP32 SPI device.
 *
 ****************************************************************************/

static void esp_spi_deselect(FAR struct esp_spi_context *ctx)
{
  if (ctx && ctx->cs_gpio >= 0)
    {
      gpio_write(ctx->cs_gpio, 1);
    }
}

/****************************************************************************
 * Name: esp_spi_transfer
 *
 * Description:
 *   Perform SPI transfer to/from ESP32.
 *
 ****************************************************************************/

static int esp_spi_transfer(FAR struct esp_spi_context *ctx,
                           FAR const uint8_t *txdata,
                           FAR uint8_t *rxdata,
                           size_t len)
{
  int ret;

  if (!ctx || !ctx->spi_dev || len == 0)
    {
      return -EINVAL;
    }

  /* Select device */

  esp_spi_select(ctx);

  /* Perform transfer */

  ret = SPI_EXCHANGE(ctx->spi_dev, txdata, rxdata, len);

  /* Deselect device */

  esp_spi_deselect(ctx);

  return ret;
}

/****************************************************************************
 * Name: esp_spi_write_reg
 *
 * Description:
 *   Write to ESP32 SPI register using direct register access.
 *
 ****************************************************************************/

static int esp_spi_write_reg_direct(FAR struct esp_spi_context *ctx,
                                   uint32_t reg_addr,
                                   uint32_t reg_value)
{
  uint8_t cmd[ESP_SPI_HEADER_LEN + 4];
  uint8_t dummy[ESP_SPI_HEADER_LEN + 4];
  uint32_t header;

  /* Build SPI header */
  header = ESP_SPI_CMD_WRITE_REG |
           ((reg_addr & 0x1FFFF) << 7) |
           (4 << 23) |  /* Length in bytes */
           (1 << 31);   /* Write flag */

  cmd[0] = header & 0xFF;
  cmd[1] = (header >> 8) & 0xFF;
  cmd[2] = (header >> 16) & 0xFF;
  cmd[3] = (header >> 24) & 0xFF;

  /* Data to write */
  cmd[4] = reg_value & 0xFF;
  cmd[5] = (reg_value >> 8) & 0xFF;
  cmd[6] = (reg_value >> 16) & 0xFF;
  cmd[7] = (reg_value >> 24) & 0xFF;

  /* Transfer data */
  return esp_spi_transfer(ctx, cmd, dummy, sizeof(cmd));
}

/****************************************************************************
 * Name: esp_spi_read_reg
 *
 * Description:
 *   Read from ESP32 SPI register using direct register access.
 *
 ****************************************************************************/

static int esp_spi_read_reg_direct(FAR struct esp_spi_context *ctx,
                                  uint32_t reg_addr,
                                  FAR uint32_t *reg_value)
{
  uint8_t cmd[ESP_SPI_HEADER_LEN];
  uint8_t rxdata[ESP_SPI_HEADER_LEN + 4];
  uint32_t header;

  if (!reg_value)
    {
      return -EINVAL;
    }

  /* Build SPI header */
  header = ESP_SPI_CMD_READ_REG |
           ((reg_addr & 0x1FFFF) << 7) |
           (4 << 23) |  /* Length in bytes */
           (0 << 31);   /* Read flag */

  cmd[0] = header & 0xFF;
  cmd[1] = (header >> 8) & 0xFF;
  cmd[2] = (header >> 16) & 0xFF;
  cmd[3] = (header >> 24) & 0xFF;

  /* Transfer command and receive data */
  int ret = esp_spi_transfer(ctx, cmd, rxdata, sizeof(cmd) + 4);
  if (ret < 0)
    {
      return ret;
    }

  /* Extract register value */
  *reg_value = rxdata[4] | (rxdata[5] << 8) |
               (rxdata[6] << 16) | (rxdata[7] << 24);

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp_spi_init
 *
 * Description:
 *   Initialize ESP32 SPI interface.
 *
 ****************************************************************************/

int esp_spi_init(FAR struct esp_host_if_ctx *ctx)
{
  FAR struct esp_spi_context *spi_ctx = &g_esp_spi_ctx;
  int ret;

  if (!ctx)
    {
      return -EINVAL;
    }

  spi_info("Initializing ESP32 SPI interface\n");

  /* Initialize context */

  memset(spi_ctx, 0, sizeof(struct esp_spi_context));

  /* Initialize GPIO pins */

#ifdef CONFIG_ESP32_SPI_IRQ_GPIO
  spi_ctx->irq_gpio = CONFIG_ESP32_SPI_IRQ_GPIO;
  ret = gpio_config(spi_ctx->irq_gpio, GPIO_INPUT | GPIO_PULLUP);
  if (ret < 0)
    {
      spi_err("Failed to configure IRQ GPIO: %d\n", ret);
      return ret;
    }
#endif

#ifdef CONFIG_ESP32_SPI_CS_GPIO
  spi_ctx->cs_gpio = CONFIG_ESP32_SPI_CS_GPIO;
  ret = gpio_config(spi_ctx->cs_gpio, GPIO_OUTPUT);
  if (ret < 0)
    {
      spi_err("Failed to configure CS GPIO: %d\n", ret);
      return ret;
    }
  gpio_write(spi_ctx->cs_gpio, 1);  /* Deselect initially */
#endif

#ifdef CONFIG_ESP32_SPI_HANDSHAKE_GPIO
  spi_ctx->handshake_gpio = CONFIG_ESP32_SPI_HANDSHAKE_GPIO;
  ret = gpio_config(spi_ctx->handshake_gpio, GPIO_INPUT);
  if (ret < 0)
    {
      spi_err("Failed to configure handshake GPIO: %d\n", ret);
      return ret;
    }
#endif

  /* Initialize semaphores */

  ret = nxsem_init(&spi_ctx->lock, 0, 1);
  if (ret < 0)
    {
      return ret;
    }

  ret = nxsem_init(&spi_ctx->handshake_sem, 0, 0);
  if (ret < 0)
    {
      nxsem_destroy(&spi_ctx->lock);
      return ret;
    }

  /* Get SPI device */

  spi_ctx->spi_dev = up_spiinitialize(CONFIG_ESP32_SPI_DEVMINOR);
  if (!spi_ctx->spi_dev)
    {
      spi_err("Failed to get SPI device\n");
      nxsem_destroy(&spi_ctx->lock);
      nxsem_destroy(&spi_ctx->handshake_sem);
      return -ENODEV;
    }

  /* Configure SPI */

  esp_spi_setup(spi_ctx);

  spi_ctx->initialized = true;
  ctx->if_type = ESP_IF_TYPE_SPI;

  spi_info("ESP32 SPI interface initialized\n");

  return OK;
}

/****************************************************************************
 * Name: esp_spi_deinit
 *
 * Description:
 *   Deinitialize ESP32 SPI interface.
 *
 ****************************************************************************/

void esp_spi_deinit(FAR struct esp_host_if_ctx *ctx)
{
  FAR struct esp_spi_context *spi_ctx = &g_esp_spi_ctx;

  spi_info("Deinitializing ESP32 SPI interface\n");

  if (spi_ctx->initialized)
    {
      /* Release resources */

      if (spi_ctx->spi_dev)
        {
          up_spidev_uninitialize(spi_ctx->spi_dev);
          spi_ctx->spi_dev = NULL;
        }

      nxsem_destroy(&spi_ctx->lock);
      nxsem_destroy(&spi_ctx->handshake_sem);

      memset(spi_ctx, 0, sizeof(struct esp_spi_context));
    }
}

/****************************************************************************
 * Name: esp_spi_read_reg
 *
 * Description:
 *   Read from ESP32 SPI register.
 *
 ****************************************************************************/

int esp_spi_read_reg(FAR struct esp_host_if_ctx *ctx, uint32_t reg,
                    FAR uint8_t *data, uint16_t len)
{
  FAR struct esp_spi_context *spi_ctx = &g_esp_spi_ctx;
  int ret = OK;

  if (!data || len == 0)
    {
      return -EINVAL;
    }

  if (!spi_ctx->initialized)
    {
      return -ENODEV;
    }

  esp_spi_claim(spi_ctx);

  /* Handle different register sizes */
  if (len == 1)
    {
      uint32_t reg_val;
      ret = esp_spi_read_reg_direct(spi_ctx, reg, &reg_val);
      if (ret == OK)
        {
          *data = reg_val & 0xFF;
        }
    }
  else if (len == 2)
    {
      uint32_t reg_val;
      ret = esp_spi_read_reg_direct(spi_ctx, reg, &reg_val);
      if (ret == OK)
        {
          *(uint16_t *)data = reg_val & 0xFFFF;
        }
    }
  else if (len == 4)
    {
      ret = esp_spi_read_reg_direct(spi_ctx, reg, (FAR uint32_t *)data);
    }
  else
    {
      /* For longer reads, we might need to use FIFO operations */
      spi_err("Unsupported register read length: %d\n", len);
      ret = -ENOTSUP;
    }

  esp_spi_release(spi_ctx);

  return ret;
}

/****************************************************************************
 * Name: esp_spi_write_reg
 *
 * Description:
 *   Write to ESP32 SPI register.
 *
 ****************************************************************************/

int esp_spi_write_reg(FAR struct esp_host_if_ctx *ctx, uint32_t reg,
                     FAR const uint8_t *data, uint16_t len)
{
  FAR struct esp_spi_context *spi_ctx = &g_esp_spi_ctx;
  int ret = OK;

  if (!data || len == 0)
    {
      return -EINVAL;
    }

  if (!spi_ctx->initialized)
    {
      return -ENODEV;
    }

  esp_spi_claim(spi_ctx);

  /* Handle different register sizes */
  if (len == 1)
    {
      uint32_t reg_val = *data;
      ret = esp_spi_write_reg_direct(spi_ctx, reg, reg_val);
    }
  else if (len == 2)
    {
      uint32_t reg_val = *(FAR const uint16_t *)data;
      ret = esp_spi_write_reg_direct(spi_ctx, reg, reg_val);
    }
  else if (len == 4)
    {
      ret = esp_spi_write_reg_direct(spi_ctx, reg, *(FAR const uint32_t *)data);
    }
  else
    {
      /* For longer writes, we might need to use FIFO operations */
      spi_err("Unsupported register write length: %d\n", len);
      ret = -ENOTSUP;
    }

  esp_spi_release(spi_ctx);

  return ret;
}

/****************************************************************************
 * Name: esp_spi_read_packet
 *
 * Description:
 *   Read a packet from ESP32 via SPI.
 *
 ****************************************************************************/

int esp_spi_read_packet(FAR struct esp_host_if_ctx *ctx,
                       FAR uint8_t *buf, size_t maxlen,
                       FAR size_t *out_len)
{
  FAR struct esp_spi_context *spi_ctx = &g_esp_spi_ctx;
  uint32_t packet_len;
  uint32_t header;
  uint8_t cmd[ESP_SPI_HEADER_LEN];
  int ret;

  if (!buf || !out_len || maxlen == 0)
    {
      return -EINVAL;
    }

  if (!spi_ctx->initialized)
    {
      return -ENODEV;
    }

  esp_spi_claim(spi_ctx);

  /* First, read the packet length register */

  ret = esp_spi_read_reg_direct(spi_ctx, ESP_SLAVE_PACKET_LEN_REG, &packet_len);
  if (ret < 0)
    {
      esp_spi_release(spi_ctx);
      return ret;
    }

  packet_len &= ESP_SLAVE_LEN_MASK;

  if (packet_len == 0)
    {
      esp_spi_release(spi_ctx);
      *out_len = 0;
      return OK;
    }

  if (packet_len > maxlen)
    {
      spi_err("Packet too large: %u > %zu\n", packet_len, maxlen);
      esp_spi_release(spi_ctx);
      return -EMSGSIZE;
    }

  /* Build command to read FIFO */

  header = ESP_SPI_CMD_READ_FIFO |
           ((packet_len & 0xFFFF) << 7) |
           (0 << 31); /* Read flag */

  cmd[0] = header & 0xFF;
  cmd[1] = (header >> 8) & 0xFF;
  cmd[2] = (header >> 16) & 0xFF;
  cmd[3] = (header >> 24) & 0xFF;

  /* Transfer command and read packet data */

  ret = esp_spi_transfer(spi_ctx, cmd, buf, sizeof(cmd));
  if (ret < 0)
    {
      esp_spi_release(spi_ctx);
      return ret;
    }

  /* Read the actual packet data */

  ret = esp_spi_transfer(spi_ctx, NULL, buf, packet_len);
  if (ret < 0)
    {
      esp_spi_release(spi_ctx);
      return ret;
    }

  esp_spi_release(spi_ctx);

  *out_len = packet_len;
  return OK;
}

/****************************************************************************
 * Name: esp_spi_write_packet
 *
 * Description:
 *   Write a packet to ESP32 via SPI.
 *
 ****************************************************************************/

int esp_spi_write_packet(FAR struct esp_host_if_ctx *ctx,
                        FAR const uint8_t *buf, size_t len)
{
  FAR struct esp_spi_context *spi_ctx = &g_esp_spi_ctx;
  uint8_t *tx_buf;
  uint32_t header;
  size_t total_len;
  int ret;

  if (!buf || len == 0 || len > ESP_SPI_MAX_PKT_SIZE)
    {
      return -EINVAL;
    }

  if (!spi_ctx->initialized)
    {
      return -ENODEV;
    }

  esp_spi_claim(spi_ctx);

  /* Allocate buffer for header + data */

  total_len = ESP_SPI_HEADER_LEN + len;
  tx_buf = kmm_malloc(total_len);
  if (!tx_buf)
    {
      esp_spi_release(spi_ctx);
      return -ENOMEM;
    }

  /* Build SPI header for FIFO write */

  header = ESP_SPI_CMD_WRITE_FIFO |
           ((len & 0xFFFF) << 7) |
           (1 << 31); /* Write flag */

  tx_buf[0] = header & 0xFF;
  tx_buf[1] = (header >> 8) & 0xFF;
  tx_buf[2] = (header >> 16) & 0xFF;
  tx_buf[3] = (header >> 24) & 0xFF;

  /* Copy packet data */

  memcpy(&tx_buf[ESP_SPI_HEADER_LEN], buf, len);

  /* Transfer packet */

  ret = esp_spi_transfer(spi_ctx, tx_buf, NULL, total_len);
  if (ret < 0)
    {
      spi_err("SPI transfer failed: %d\n", ret);
    }

  /* Add end marker */

  if (ret == OK)
    {
      uint8_t end_cmd = ESP_SPI_CMD_WRITE_END;
      uint8_t dummy;
      esp_spi_transfer(spi_ctx, &end_cmd, &dummy, 1);
    }

  kmm_free(tx_buf);
  esp_spi_release(spi_ctx);

  return ret;
}

/****************************************************************************
 * Name: esp_spi_send_interrupt
 *
 * Description:
 *   Send an interrupt to ESP32 via SPI.
 *
 ****************************************************************************/

int esp_spi_send_interrupt(FAR struct esp_host_if_ctx *ctx, uint8_t intr_mask)
{
  FAR struct esp_spi_context *spi_ctx = &g_esp_spi_ctx;
  int ret;

  if (!spi_ctx->initialized)
    {
      return -ENODEV;
    }

  /* Write interrupt mask to scratch register 7 */

  ret = esp_spi_write_reg_direct(spi_ctx, ESP_SLAVE_SCRATCH_REG_7, intr_mask);

  return ret;
}

/****************************************************************************
 * Name: esp_spi_wait_for_handshake
 *
 * Description:
 *   Wait for ESP32 handshake signal.
 *
 ****************************************************************************/

int esp_spi_wait_for_handshake(FAR struct esp_host_if_ctx *ctx,
                              uint32_t timeout)
{
  FAR struct esp_spi_context *spi_ctx = &g_esp_spi_ctx;
  int ret;

  if (!spi_ctx->initialized)
    {
      return -ENODEV;
    }

  if (spi_ctx->handshake_gpio < 0)
    {
      /* No handshake GPIO, just return OK */
      return OK;
    }

  /* Wait for handshake signal with timeout */

  ret = nxsem_tickwait(&spi_ctx->handshake_sem, MSEC2TICK(timeout));

  return (ret == OK) ? OK : -ETIMEDOUT;
}

#endif /* CONFIG_ESP32_WIFI_SPI */