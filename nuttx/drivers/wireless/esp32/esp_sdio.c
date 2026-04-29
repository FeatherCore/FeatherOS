/****************************************************************************
 * drivers/wireless/esp32/esp_sdio.c
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
#include <nuttx/spinlock.h>

#ifdef CONFIG_ESP32_WIFI_SDIO

#include <nuttx/sdio.h>
#include <nuttx/sdio/sdio.h>

#include "esp_cfg80211.h"
#include "esp_host_if.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Debug output */

#ifdef CONFIG_DEBUG_WIRELESS_INFO
#  define sdio_info(fmt, ...)  ninfo(fmt, ##__VA_ARGS__)
#else
#  define sdio_info(fmt, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_ERROR
#  define sdio_err(fmt, ...)   nerr(fmt, ##__VA_ARGS__)
#else
#  define sdio_err(fmt, ...)
#endif

/* Maximum retries for SDIO operations */

#define ESP_SDIO_MAX_RETRIES            3

/* SDIO block size for ESP32 */

#define ESP_SDIO_BLOCK_SIZE            512

/* SDIO function number for WiFi */

#define ESP_SDIO_FUNC_WIFI             1

/* Maximum packet size */

#define ESP_SDIO_MAX_PKT_SIZE          4096

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* ESP32 SDIO context */

struct esp_sdio_context
{
  struct sdio_dev_s *dev;           /* SDIO device */
  uint8_t func;                     /* SDIO function number */
  bool initialized;                 /* Initialization flag */
  uint32_t rx_byte_count;           /* RX byte counter */
  uint32_t tx_buffer_count;         /* TX buffer counter */
  sem_t lock;                       /* Lock for operations */
  spinlock_t irq_lock;              /* IRQ lock */
};

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct esp_sdio_context g_esp_sdio_ctx;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp_sdio_claim
 *
 * Description:
 *   Claim SDIO device for exclusive access.
 *
 ****************************************************************************/

static void esp_sdio_claim(FAR struct esp_sdio_context *ctx)
{
  if (ctx && ctx->dev)
    {
      nxsem_wait(&ctx->lock);
      SDIO_LOCK(ctx->dev);
    }
}

/****************************************************************************
 * Name: esp_sdio_release
 *
 * Description:
 *   Release SDIO device.
 *
 ****************************************************************************/

static void esp_sdio_release(FAR struct esp_sdio_context *ctx)
{
  if (ctx && ctx->dev)
    {
      SDIO_UNLOCK(ctx->dev);
      nxsem_post(&ctx->lock);
    }
}

/****************************************************************************
 * Name: esp_sdio_read_byte
 *
 * Description:
 *   Read a single byte from SDIO register.
 *
 ****************************************************************************/

static int esp_sdio_read_byte(FAR struct esp_sdio_context *ctx,
                              uint32_t addr, FAR uint8_t *data)
{
  int ret;

  if (!ctx || !ctx->dev || !data)
    {
      return -EINVAL;
    }

  ret = SDIO_READBYTE(ctx->dev, addr, data);
  if (ret < 0)
    {
      sdio_err("SDIO read byte failed: %d\n", ret);
    }

  return ret;
}

/****************************************************************************
 * Name: esp_sdio_write_byte
 *
 * Description:
 *   Write a single byte to SDIO register.
 *
 ****************************************************************************/

static int esp_sdio_write_byte(FAR struct esp_sdio_context *ctx,
                               uint32_t addr, uint8_t data)
{
  int ret;

  if (!ctx || !ctx->dev)
    {
      return -EINVAL;
    }

  ret = SDIO_WRITEBYTE(ctx->dev, addr, data);
  if (ret < 0)
    {
      sdio_err("SDIO write byte failed: %d\n", ret);
    }

  return ret;
}

/****************************************************************************
 * Name: esp_sdio_read_bytes
 *
 * Description:
 *   Read multiple bytes from SDIO.
 *
 ****************************************************************************/

static int esp_sdio_read_bytes(FAR struct esp_sdio_context *ctx,
                               uint32_t addr, FAR uint8_t *data,
                               uint16_t len)
{
  int ret;

  if (!ctx || !ctx->dev || !data || len == 0)
    {
      return -EINVAL;
    }

  ret = SDIO_READ(ctx->dev, addr, data, len);
  if (ret < 0)
    {
      sdio_err("SDIO read bytes failed: %d\n", ret);
    }

  return ret;
}

/****************************************************************************
 * Name: esp_sdio_write_bytes
 *
 * Description:
 *   Write multiple bytes to SDIO.
 *
 ****************************************************************************/

static int esp_sdio_write_bytes(FAR struct esp_sdio_context *ctx,
                                uint32_t addr, FAR const uint8_t *data,
                                uint16_t len)
{
  int ret;

  if (!ctx || !ctx->dev || !data || len == 0)
    {
      return -EINVAL;
    }

  ret = SDIO_WRITE(ctx->dev, addr, data, len);
  if (ret < 0)
    {
      sdio_err("SDIO write bytes failed: %d\n", ret);
    }

  return ret;
}

/****************************************************************************
 * Name: esp_sdio_get_rx_len
 *
 * Description:
 *   Get the length of available RX data from ESP32.
 *
 ****************************************************************************/

static int esp_sdio_get_rx_len(FAR struct esp_sdio_context *ctx,
                               FAR uint32_t *rx_len)
{
  uint32_t len;
  uint32_t temp;
  int ret;

  /* Read packet length register */

  ret = esp_sdio_read_bytes(ctx, ESP_SLAVE_PACKET_LEN_REG,
                            (FAR uint8_t *)&len, sizeof(len));
  if (ret < 0)
    {
      return ret;
    }

  len &= ESP_SLAVE_LEN_MASK;

  /* Calculate actual length considering wrap-around */

  if (len >= ctx->rx_byte_count)
    {
      len = (len + ESP_RX_BYTE_MAX - ctx->rx_byte_count) % ESP_RX_BYTE_MAX;
    }
  else
    {
      /* Handle wrap-around case */

      temp = ESP_RX_BYTE_MAX - ctx->rx_byte_count;
      len = temp + len;

      if (len > ESP_RX_BUFFER_SIZE)
        {
          sdio_info("RX len %u exceeds max %u\n", len, ESP_RX_BUFFER_SIZE);
        }
    }

  *rx_len = len;
  return OK;
}

/****************************************************************************
 * Name: esp_sdio_get_tx_buffer_num
 *
 * Description:
 *   Get the number of available TX buffers in ESP32.
 *
 ****************************************************************************/

static int esp_sdio_get_tx_buffer_num(FAR struct esp_sdio_context *ctx,
                                      FAR uint32_t *tx_num)
{
  uint32_t len;
  int ret;

  /* Read token register */

  ret = esp_sdio_read_bytes(ctx, ESP_SLAVE_TOKEN_RDATA,
                            (FAR uint8_t *)&len, sizeof(len));
  if (ret < 0)
    {
      return ret;
    }

  /* Extract TX buffer count from token */

  len = (len >> 16) & ESP_TX_BUFFER_MASK;
  len = (len + ESP_TX_BUFFER_MAX - ctx->tx_buffer_count) % ESP_TX_BUFFER_MAX;

  *tx_num = len;
  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp_sdio_init
 *
 * Description:
 *   Initialize ESP32 SDIO interface.
 *
 ****************************************************************************/

int esp_sdio_init(FAR struct esp_host_if_ctx *ctx)
{
  FAR struct esp_sdio_context *sdio_ctx = &g_esp_sdio_ctx;
  int ret;

  if (!ctx)
    {
      return -EINVAL;
    }

  sdio_info("Initializing ESP32 SDIO interface\n");

  /* Initialize context */

  memset(sdio_ctx, 0, sizeof(struct esp_sdio_context));
  nxsem_init(&sdio_ctx->lock, 0, 1);
  spin_lock_init(&sdio_ctx->irq_lock);

  /* Get SDIO device - this would typically come from platform-specific
   * initialization. For now, we assume it's passed through ctx.
   */

#ifdef CONFIG_ESP32_WIFI_SDIO
  /* The SDIO device should be initialized by the platform-specific code.
   * Here we just set up the context.
   */
  sdio_ctx->func = ESP_SDIO_FUNC_WIFI;
#endif

  /* Allocate TX/RX buffers */

  sdio_ctx->tx_buffer_count = 0;
  sdio_ctx->rx_byte_count = 0;

  sdio_ctx->initialized = true;
  ctx->if_type = ESP_IF_TYPE_SDIO;

  sdio_info("ESP32 SDIO interface initialized\n");

  return OK;
}

/****************************************************************************
 * Name: esp_sdio_deinit
 *
 * Description:
 *   Deinitialize ESP32 SDIO interface.
 *
 ****************************************************************************/

void esp_sdio_deinit(FAR struct esp_host_if_ctx *ctx)
{
  FAR struct esp_sdio_context *sdio_ctx = &g_esp_sdio_ctx;

  sdio_info("Deinitializing ESP32 SDIO interface\n");

  if (sdio_ctx->initialized)
    {
      nxsem_destroy(&sdio_ctx->lock);
      memset(sdio_ctx, 0, sizeof(struct esp_sdio_context));
    }
}

/****************************************************************************
 * Name: esp_sdio_read_reg
 *
 * Description:
 *   Read from ESP32 SDIO register.
 *
 ****************************************************************************/

int esp_sdio_read_reg(FAR struct esp_host_if_ctx *ctx, uint32_t reg,
                     FAR uint8_t *data, uint16_t len)
{
  FAR struct esp_sdio_context *sdio_ctx = &g_esp_sdio_ctx;
  int ret;

  if (!data || len == 0)
    {
      return -EINVAL;
    }

  /* Apply address mask for register access */

  reg &= ESP_ADDRESS_MASK;

  esp_sdio_claim(sdio_ctx);

  if (len == 1)
    {
      ret = esp_sdio_read_byte(sdio_ctx, reg, data);
    }
  else
    {
      ret = esp_sdio_read_bytes(sdio_ctx, reg, data, len);
    }

  esp_sdio_release(sdio_ctx);

  return ret;
}

/****************************************************************************
 * Name: esp_sdio_write_reg
 *
 * Description:
 *   Write to ESP32 SDIO register.
 *
 ****************************************************************************/

int esp_sdio_write_reg(FAR struct esp_host_if_ctx *ctx, uint32_t reg,
                      FAR const uint8_t *data, uint16_t len)
{
  FAR struct esp_sdio_context *sdio_ctx = &g_esp_sdio_ctx;
  int ret;

  if (!data || len == 0)
    {
      return -EINVAL;
    }

  /* Apply address mask for register access */

  reg &= ESP_ADDRESS_MASK;

  esp_sdio_claim(sdio_ctx);

  if (len == 1)
    {
      ret = esp_sdio_write_byte(sdio_ctx, reg, *data);
    }
  else
    {
      ret = esp_sdio_write_bytes(sdio_ctx, reg, data, len);
    }

  esp_sdio_release(sdio_ctx);

  return ret;
}

/****************************************************************************
 * Name: esp_sdio_read_packet
 *
 * Description:
 *   Read a packet from ESP32.
 *
 ****************************************************************************/

int esp_sdio_read_packet(FAR struct esp_host_if_ctx *ctx,
                       FAR uint8_t *buf, size_t maxlen,
                       FAR size_t *out_len)
{
  FAR struct esp_sdio_context *sdio_ctx = &g_esp_sdio_ctx;
  uint32_t rx_len;
  uint32_t offset;
  int ret;

  if (!buf || !out_len || maxlen == 0)
    {
      return -EINVAL;
    }

  esp_sdio_claim(sdio_ctx);

  /* Get RX data length */

  ret = esp_sdio_get_rx_len(sdio_ctx, &rx_len);
  if (ret < 0)
    {
      esp_sdio_release(sdio_ctx);
      return ret;
    }

  if (rx_len == 0)
    {
      esp_sdio_release(sdio_ctx);
      *out_len = 0;
      return OK;
    }

  if (rx_len > maxlen)
    {
      sdio_err("RX packet too large: %u > %u\n", rx_len, maxlen);
      esp_sdio_release(sdio_ctx);
      return -EMSGSIZE;
    }

  /* Read packet data from CMD53 end address */

  offset = ESP_SLAVE_CMD53_END_ADDR - rx_len;

  ret = esp_sdio_read_bytes(sdio_ctx, offset, buf, rx_len);
  if (ret < 0)
    {
      esp_sdio_release(sdio_ctx);
      return ret;
    }

  /* Update RX byte counter */

  sdio_ctx->rx_byte_count = (sdio_ctx->rx_byte_count + rx_len) % ESP_RX_BYTE_MAX;

  esp_sdio_release(sdio_ctx);

  *out_len = rx_len;
  return OK;
}

/****************************************************************************
 * Name: esp_sdio_write_packet
 *
 * Description:
 *   Write a packet to ESP32.
 *
 ****************************************************************************/

int esp_sdio_write_packet(FAR struct esp_host_if_ctx *ctx,
                         FAR const uint8_t *buf, size_t len)
{
  FAR struct esp_sdio_context *sdio_ctx = &g_esp_sdio_ctx;
  uint32_t tx_num;
  uint32_t offset;
  int retries = 0;
  int ret;

  if (!buf || len == 0 || len > ESP_SDIO_MAX_PKT_SIZE)
    {
      return -EINVAL;
    }

  esp_sdio_claim(sdio_ctx);

retry:
  /* Check if TX buffer is available */

  ret = esp_sdio_get_tx_buffer_num(sdio_ctx, &tx_num);
  if (ret < 0)
    {
      esp_sdio_release(sdio_ctx);
      return ret;
    }

  if (tx_num < ((len + ESP_BLOCK_SIZE - 1) / ESP_BLOCK_SIZE))
    {
      /* Not enough TX buffers, retry */

      if (retries++ < ESP_SDIO_MAX_RETRIES)
        {
          esp_sdio_release(sdio_ctx);
          /* Add delay here if needed */
          esp_sdio_claim(sdio_ctx);
          goto retry;
        }

      sdio_err("No TX buffer available\n");
      esp_sdio_release(sdio_ctx);
      return -EBUSY;
    }

  /* Write packet to CMD53 end address */

  offset = ESP_SLAVE_CMD53_END_ADDR - len;

  ret = esp_sdio_write_bytes(sdio_ctx, offset, buf, len);
  if (ret < 0)
    {
      esp_sdio_release(sdio_ctx);
      return ret;
    }

  /* Update TX buffer counter */

  sdio_ctx->tx_buffer_count = (sdio_ctx->tx_buffer_count +
                               (len + ESP_BLOCK_SIZE - 1) / ESP_BLOCK_SIZE) %
                              ESP_TX_BUFFER_MAX;

  /* Notify ESP32 of new data */

  ret = esp_sdio_write_byte(sdio_ctx, ESP_SLAVE_SCRATCH_REG_7,
                            ESP_SLAVE_RX_NEW_PACKET_INT);

  esp_sdio_release(sdio_ctx);

  return ret;
}

/****************************************************************************
 * Name: esp_sdio_send_interrupt
 *
 * Description:
 *   Send an interrupt to ESP32.
 *
 ****************************************************************************/

int esp_sdio_send_interrupt(FAR struct esp_host_if_ctx *ctx, uint8_t intr_mask)
{
  FAR struct esp_sdio_context *sdio_ctx = &g_esp_sdio_ctx;
  int ret;

  esp_sdio_claim(sdio_ctx);

  /* Write to scratch register 7 to trigger interrupt */

  ret = esp_sdio_write_byte(sdio_ctx, ESP_SLAVE_SCRATCH_REG_7, intr_mask);

  esp_sdio_release(sdio_ctx);

  return ret;
}

/****************************************************************************
 * Name: esp_sdio_get_intr_status
 *
 * Description:
 *   Get ESP32 interrupt status.
 *
 ****************************************************************************/

int esp_sdio_get_intr_status(FAR struct esp_host_if_ctx *ctx,
                            FAR uint32_t *intr_mask)
{
  FAR struct esp_sdio_context *sdio_ctx = &g_esp_sdio_ctx;
  int ret;

  if (!intr_mask)
    {
      return -EINVAL;
    }

  esp_sdio_claim(sdio_ctx);

  /* Read interrupt status register */

  ret = esp_sdio_read_bytes(sdio_ctx, ESP_SLAVE_INT_ST_REG,
                            (FAR uint8_t *)intr_mask, sizeof(*intr_mask));

  esp_sdio_release(sdio_ctx);

  return ret;
}

/****************************************************************************
 * Name: esp_sdio_clear_intr
 *
 * Description:
 *   Clear ESP32 interrupts.
 *
 ****************************************************************************/

int esp_sdio_clear_intr(FAR struct esp_host_if_ctx *ctx, uint32_t intr_mask)
{
  FAR struct esp_sdio_context *sdio_ctx = &g_esp_sdio_ctx;
  int ret;

  esp_sdio_claim(sdio_ctx);

  /* Write to interrupt clear register */

  ret = esp_sdio_write_bytes(sdio_ctx, ESP_SLAVE_INT_CLR_REG,
                             (FAR const uint8_t *)&intr_mask,
                             sizeof(intr_mask));

  esp_sdio_release(sdio_ctx);

  return ret;
}

#endif /* CONFIG_ESP32_WIFI_SDIO */