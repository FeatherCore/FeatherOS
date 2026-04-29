/****************************************************************************
 * drivers/wireless/esp32/esp32_sdio.c
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
#include <errno.h>
#include <debug.h>

#include <nuttx/kmalloc.h>
#include <nuttx/semaphore.h>
#include <nuttx/sdio.h>
#include <nuttx/wireless/esp32_wifi.h>

#ifdef CONFIG_ESP32_WIFI_SDIO

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#ifdef CONFIG_DEBUG_WIRELESS_INFO
#  define sdioinfo(format, ...)   ninfo(format, ##__VA_ARGS__)
#else
#  define sdioinfo(format, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_ERROR
#  define sdioerr(format, ...)    nerr(format, ##__VA_ARGS__)
#else
#  define sdioerr(format, ...)
#endif

/* ESP32 SDIO registers */

#define ESP32_SDIO_BASE            0x3ff55000

/* Slave interrupt status registers */

#define ESP32_SLCHOST_INT_RAW_REG  (ESP32_SDIO_BASE + 0x50)
#define ESP32_SLCHOST_INT_ST_REG   (ESP32_SDIO_BASE + 0x58)
#define ESP32_SLCHOST_INT_CLR_REG  (ESP32_SDIO_BASE + 0xd4)

/* Packet length register */

#define ESP32_SLCHOST_PKT_LEN_REG  (ESP32_SDIO_BASE + 0x60)

/* Token read register */

#define ESP32_SLCHOST_TOKEN_RDATA  (ESP32_SDIO_BASE + 0x44)

/* Scratch registers */

#define ESP32_SLCHOST_SCRATCH0_REG (ESP32_SDIO_BASE + 0x6c)
#define ESP32_SLCHOST_SCRATCH7_REG (ESP32_SDIO_BASE + 0x8c)

/* Block size */

#define ESP32_SDIO_BLOCK_SIZE      512

/* Interrupt flags */

#define ESP32_SLAVE_RX_NEW_PACKET_INT    (1 << 23)
#define ESP32_SLAVE_TX_EOF_INT           (1 << 24)
#define ESP32_SLAVE_RX_EOF_INT           (1 << 25)

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct esp32_sdio_priv_s
{
  struct sdio_dev_s *sdio;      /* SDIO device handle */
  uint8_t func;                 /* SDIO function number */
  bool initialized;             /* Initialization flag */
  sem_t lock;                   /* Access protection */
  uint32_t rx_byte_count;       /* RX byte counter */
  uint32_t tx_buffer_count;     /* TX buffer counter */
};

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct esp32_sdio_priv_s g_esp32_sdio_priv;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp32_sdio_lock
 *
 * Description:
 *   Take the SDIO lock
 *
 ****************************************************************************/

static void esp32_sdio_lock(struct esp32_sdio_priv_s *priv)
{
  nxsem_wait(&priv->lock);
}

/****************************************************************************
 * Name: esp32_sdio_unlock
 *
 * Description:
 *   Release the SDIO lock
 *
 ****************************************************************************/

static void esp32_sdio_unlock(struct esp32_sdio_priv_s *priv)
{
  nxsem_post(&priv->lock);
}

/****************************************************************************
 * Name: esp32_sdio_read_reg
 *
 * Description:
 *   Read from an ESP32 SDIO register
 *
 ****************************************************************************/

static int esp32_sdio_read_reg(struct esp32_sdio_priv_s *priv,
                              uint32_t addr, uint8_t *value)
{
  int ret;

  if (!priv || !priv->sdio || !value)
    {
      return -EINVAL;
    }

  esp32_sdio_lock(priv);

  /* Use SDIO function-specific register read */
  ret = SDIO_READ_BYTE(priv->sdio, priv->func, addr, value);

  esp32_sdio_unlock(priv);
  return ret;
}

/****************************************************************************
 * Name: esp32_sdio_write_reg
 *
 * Description:
 *   Write to an ESP32 SDIO register
 *
 ****************************************************************************/

static int esp32_sdio_write_reg(struct esp32_sdio_priv_s *priv,
                               uint32_t addr, uint8_t value)
{
  int ret;

  if (!priv || !priv->sdio)
    {
      return -EINVAL;
    }

  esp32_sdio_lock(priv);

  /* Use SDIO function-specific register write */
  ret = SDIO_WRITE_BYTE(priv->sdio, priv->func, addr, value);

  esp32_sdio_unlock(priv);
  return ret;
}

/****************************************************************************
 * Name: esp32_sdio_read_block
 *
 * Description:
 *   Read a block from ESP32 via SDIO
 *
 ****************************************************************************/

static int esp32_sdio_read_block(struct esp32_sdio_priv_s *priv,
                                uint32_t addr, uint8_t *data, size_t len)
{
  int ret;

  if (!priv || !priv->sdio || !data || len == 0)
    {
      return -EINVAL;
    }

  esp32_sdio_lock(priv);

  /* Perform block read */
  ret = SDIO_READ_BLOCK(priv->sdio, priv->func, addr, data, len);

  esp32_sdio_unlock(priv);
  return ret;
}

/****************************************************************************
 * Name: esp32_sdio_write_block
 *
 * Description:
 *   Write a block to ESP32 via SDIO
 *
 ****************************************************************************/

static int esp32_sdio_write_block(struct esp32_sdio_priv_s *priv,
                                 uint32_t addr, const uint8_t *data, size_t len)
{
  int ret;

  if (!priv || !priv->sdio || !data || len == 0)
    {
      return -EINVAL;
    }

  esp32_sdio_lock(priv);

  /* Perform block write */
  ret = SDIO_WRITE_BLOCK(priv->sdio, priv->func, addr, (uint8_t *)data, len);

  esp32_sdio_unlock(priv);
  return ret;
}

/****************************************************************************
 * Name: esp32_sdio_get_rx_data_len
 *
 * Description:
 *   Get the length of available RX data
 *
 ****************************************************************************/

static int esp32_sdio_get_rx_data_len(struct esp32_sdio_priv_s *priv,
                                      uint32_t *len)
{
  uint32_t reg_val;
  int ret;

  if (!len)
    {
      return -EINVAL;
    }

  ret = esp32_sdio_read_reg(priv, ESP32_SLCHOST_PKT_LEN_REG,
                           (uint8_t *)&reg_val);
  if (ret < 0)
    {
      return ret;
    }

  /* Extract packet length */
  *len = reg_val & 0xFFFFF; /* 20-bit length field */

  return OK;
}

/****************************************************************************
 * Name: esp32_sdio_enable_interrupt
 *
 * Description:
 *   Enable SDIO interrupts for ESP32
 *
 ****************************************************************************/

static int esp32_sdio_enable_interrupt(struct esp32_sdio_priv_s *priv)
{
  int ret;

  /* Enable interrupt in ESP32 scratch register */
  ret = esp32_sdio_write_reg(priv, ESP32_SLCHOST_SCRATCH7_REG,
                            ESP32_SLAVE_RX_NEW_PACKET_INT);
  if (ret < 0)
    {
      sdioerr("Failed to enable interrupt: %d\n", ret);
      return ret;
    }

  sdioinfo("Interrupts enabled\n");
  return OK;
}

/****************************************************************************
 * Name: esp32_sdio_interrupt_handler
 *
 * Description:
 *   Handle SDIO interrupt from ESP32
 *
 ****************************************************************************/

static int esp32_sdio_interrupt_handler(int irq, void *context, void *arg)
{
  struct esp32_sdio_priv_s *priv = (struct esp32_sdio_priv_s *)arg;
  uint32_t intr_status;
  int ret;

  if (!priv)
    {
      return OK;
    }

  /* Read interrupt status from ESP32 */
  ret = esp32_sdio_read_reg(priv, ESP32_SLCHOST_INT_ST_REG,
                           (uint8_t *)&intr_status);
  if (ret < 0)
    {
      sdioerr("Failed to read interrupt status: %d\n", ret);
      return OK;
    }

  sdioinfo("SDIO interrupt: 0x%08x\n", intr_status);

  /* Process interrupt */
  if (intr_status & ESP32_SLAVE_RX_NEW_PACKET_INT)
    {
      /* New packet available - notify higher layer */
      sdioinfo("New packet interrupt\n");
      /* TODO: Call upper layer handler for packet processing */
    }

  ret = esp32_sdio_write_reg(priv, ESP32_SLCHOST_INT_CLR_REG,
                            (uint8_t)(intr_status & 0xFF));

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp32_sdio_initialize
 *
 * Description:
 *   Initialize ESP32 SDIO interface
 *
 ****************************************************************************/

int esp32_sdio_initialize(struct sdio_dev_s *sdio_dev, uint8_t func_num)
{
  struct esp32_sdio_priv_s *priv = &g_esp32_sdio_priv;
  int ret;

  sdioinfo("Initializing ESP32 SDIO interface\n");

  /* Initialize private data */

  memset(priv, 0, sizeof(struct esp32_sdio_priv_s));
  ret = nxsem_init(&priv->lock, 0, 1);
  if (ret < 0)
    {
      sdioerr("Failed to initialize semaphore: %d\n", ret);
      return ret;
    }

  priv->sdio = sdio_dev;
  priv->func = func_num;
  priv->initialized = true;

  /* Enable SDIO function */

  ret = SDIO_ENABLE_FUNC(sdio_dev, func_num);
  if (ret < 0)
    {
      sdioerr("Failed to enable SDIO function %d: %d\n", func_num, ret);
      nxsem_destroy(&priv->lock);
      return ret;
    }

  /* Set block size */
  SDIO_CLAIM_HOST(sdio_dev);
  ret = SDIO_SET_BLOCKLEN(sdio_dev, func_num, ESP32_SDIO_BLOCK_SIZE);
  SDIO_RELEASE_HOST(sdio_dev);
  if (ret < 0)
    {
      sdioerr("Failed to set block length: %d\n", ret);
      SDIO_DISABLE_FUNC(sdio_dev, func_num);
      nxsem_destroy(&priv->lock);
      return ret;
    }

  /* Enable interrupts */
  ret = esp32_sdio_enable_interrupt(priv);
  if (ret < 0)
    {
      sdioerr("Failed to enable interrupts: %d\n", ret);
      SDIO_DISABLE_FUNC(sdio_dev, func_num);
      nxsem_destroy(&priv->lock);
      return ret;
    }

  sdioinfo("ESP32 SDIO interface initialized\n");
  return OK;
}

/****************************************************************************
 * Name: esp32_sdio_uninitialize
 *
 * Description:
 *   Uninitialize ESP32 SDIO interface
 *
 ****************************************************************************/

int esp32_sdio_uninitialize(void)
{
  struct esp32_sdio_priv_s *priv = &g_esp32_sdio_priv;

  if (!priv->initialized)
    {
      return -ENODEV;
    }

  sdioinfo("Uninitializing ESP32 SDIO interface\n");

  /* Disable function */
  if (priv->sdio)
    {
      SDIO_DISABLE_FUNC(priv->sdio, priv->func);
    }

  /* Destroy lock */
  nxsem_destroy(&priv->lock);

  /* Clear initialization state */
  memset(priv, 0, sizeof(struct esp32_sdio_priv_s));
  priv->initialized = false;

  sdioinfo("ESP32 SDIO interface uninitialized\n");
  return OK;
}

/****************************************************************************
 * Name: esp32_sdio_read_packet
 *
 * Description:
 *   Read a packet from ESP32 via SDIO
 *
 ****************************************************************************/

int esp32_sdio_read_packet(uint8_t *buffer, size_t *buf_len)
{
  struct esp32_sdio_priv_s *priv = &g_esp32_sdio_priv;
  uint32_t packet_len;
  int ret;

  if (!priv->initialized || !buffer || !buf_len)
    {
      return -EINVAL;
    }

  /* Get packet length */
  ret = esp32_sdio_get_rx_data_len(priv, &packet_len);
  if (ret < 0)
    {
      sdioerr("Failed to get RX data length: %d\n", ret);
      return ret;
    }

  if (packet_len == 0)
    {
      *buf_len = 0;
      return OK;
    }

  if (packet_len > *buf_len)
    {
      sdioerr("Buffer too small: need %u, have %zu\n", packet_len, *buf_len);
      return -EMSGSIZE;
    }

  /* Read packet data - typically at CMD53 end address */
  ret = esp32_sdio_read_block(priv, 0x1F800 - packet_len, buffer, packet_len);
  if (ret < 0)
    {
      sdioerr("Failed to read packet: %d\n", ret);
      return ret;
    }

  *buf_len = packet_len;
  return OK;
}

/****************************************************************************
 * Name: esp32_sdio_write_packet
 *
 * Description:
 *   Write a packet to ESP32 via SDIO
 *
 ****************************************************************************/

int esp32_sdio_write_packet(const uint8_t *buffer, size_t buf_len)
{
  struct esp32_sdio_priv_s *priv = &g_esp32_sdio_priv;
  int ret;

  if (!priv->initialized || !buffer || buf_len == 0)
    {
      return -EINVAL;
    }

  /* Write packet data - typically to CMD53 end address */
  ret = esp32_sdio_write_block(priv, 0x1F800 - buf_len, buffer, buf_len);
  if (ret < 0)
    {
      sdioerr("Failed to write packet: %d\n", ret);
      return ret;
    }

  sdioinfo("Wrote packet: %zu bytes\n", buf_len);
  return OK;
}

/****************************************************************************
 * Name: esp32_sdio_get_intr_status
 *
 * Description:
 *   Get ESP32 interrupt status
 *
 ****************************************************************************/

int esp32_sdio_get_intr_status(uint32_t *intr_status)
{
  struct esp32_sdio_priv_s *priv = &g_esp32_sdio_priv;
  int ret;

  if (!priv->initialized || !intr_status)
    {
      return -EINVAL;
    }

  ret = esp32_sdio_read_reg(priv, ESP32_SLCHOST_INT_ST_REG,
                           (uint8_t *)intr_status);
  return ret;
}

/****************************************************************************
 * Name: esp32_sdio_clear_intr
 *
 * Description:
 *   Clear ESP32 interrupts
 *
 ****************************************************************************/

int esp32_sdio_clear_intr(uint32_t intr_mask)
{
  struct esp32_sdio_priv_s *priv = &g_esp32_sdio_priv;
  int ret;

  if (!priv->initialized)
    {
      return -ENODEV;
    }

  ret = esp32_sdio_write_reg(priv, ESP32_SLCHOST_INT_CLR_REG,
                            (uint8_t)(intr_mask & 0xFF));
  return ret;
}

#endif /* CONFIG_ESP32_WIFI_SDIO */