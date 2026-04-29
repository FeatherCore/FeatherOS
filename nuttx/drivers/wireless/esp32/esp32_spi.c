/****************************************************************************
 * drivers/wireless/esp32/esp32_spi.c
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
#include <nuttx/spi/spi.h>
#include <nuttx/gpio.h>
#include <nuttx/wireless/esp32_wifi.h>

#ifdef CONFIG_ESP32_WIFI_SPI

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#ifdef CONFIG_DEBUG_WIRELESS_INFO
#  define esp32_spiinfo(format, ...)   ninfo(format, ##__VA_ARGS__)
#else
#  define esp32_spiinfo(format, ...)
#endif

#ifdef CONFIG_DEBUG_WIRELESS_ERROR
#  define esp32_spierr(format, ...)    nerr(format, ##__VA_ARGS__)
#else
#  define esp32_spierr(format, ...)
#endif

/* Block size */

#define ESP32_SPI_BLOCK_SIZE             512

/* Command bit fields */

#define ESP32_SPI_CMD_BITS               6
#define ESP32_SPI_ADDR_BITS             19
#define ESP32_SPI_LEN_BITS               7

/* Command field masks */

#define ESP32_SPI_CMD_MASK               ((1 << ESP32_SPI_CMD_BITS) - 1)
#define ESP32_SPI_ADDR_MASK              ((1 << ESP32_SPI_ADDR_BITS) - 1)
#define ESP32_SPI_LEN_MASK               ((1 << ESP32_SPI_LEN_BITS) - 1)

/* Header field shifts */

#define ESP32_SPI_HDR_CMD_SHIFT          0
#define ESP32_SPI_HDR_ADDR_SHIFT         ESP32_SPI_CMD_BITS
#define ESP32_SPI_HDR_LEN_SHIFT          (ESP32_SPI_CMD_BITS + ESP32_SPI_ADDR_BITS)

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct esp32_spi_priv_s
{
  struct spi_dev_s *spi;          /* SPI device handle */
  int cs_gpio;                   /* Chip select GPIO */
  int irq_gpio;                  /* Interrupt GPIO */
  bool initialized;              /* Initialization flag */
  sem_t lock;                    /* Access protection */
  uint32_t rx_byte_count;        /* RX byte counter */
  uint32_t tx_buffer_count;      /* TX buffer counter */
};

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct esp32_spi_priv_s g_esp32_spi_priv;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp32_spi_lock
 *
 * Description:
 *   Take the SPI lock
 *
 ****************************************************************************/

static void esp32_spi_lock(FAR struct esp32_spi_priv_s *priv)
{
  nxsem_wait(&priv->lock);
}

/****************************************************************************
 * Name: esp32_spi_unlock
 *
 * Description:
 *   Release the SPI lock
 *
 ****************************************************************************/

static void esp32_spi_unlock(FAR struct esp32_spi_priv_s *priv)
{
  nxsem_post(&priv->lock);
}

/****************************************************************************
 * Name: esp32_spi_select
 *
 * Description:
 *   Select ESP32 SPI device
 *
 ****************************************************************************/

static void esp32_spi_select(FAR struct esp32_spi_priv_s *priv)
{
  if (priv->cs_gpio >= 0)
    {
      gpio_write(priv->cs_gpio, 0);
    }
}

/****************************************************************************
 * Name: esp32_spi_deselect
 *
 * Description:
 *   Deselect ESP32 SPI device
 *
 ****************************************************************************/

static void esp32_spi_deselect(FAR struct esp32_spi_priv_s *priv)
{
  if (priv->cs_gpio >= 0)
    {
      gpio_write(priv->cs_gpio, 1);
    }
}

/****************************************************************************
 * Name: esp32_spi_setup
 *
 * Description:
 *   Setup SPI parameters for ESP32 communication
 *
 ****************************************************************************/

static void esp32_spi_setup(FAR struct esp32_spi_priv_s *priv)
{
  if (priv->spi)
    {
      SPI_SETFREQUENCY(priv->spi, CONFIG_ESP32_SPI_FREQ);
      SPI_SETMODE(priv->spi, SPIDEV_MODE0);
      SPI_SETBITS(priv->spi, 8);
    }
}

/****************************************************************************
 * Name: esp32_spi_exchange
 *
 * Description:
 *   Perform SPI exchange with ESP32
 *
 ****************************************************************************/

static int esp32_spi_exchange(FAR struct esp32_spi_priv_s *priv,
                             FAR const uint8_t *txbuf, FAR uint8_t *rxbuf, size_t len)
{
  int ret;

  if (!priv || !priv->spi || !txbuf || !rxbuf || len == 0)
    {
      return -EINVAL;
    }

  SPI_LOCK(priv->spi, true);
  esp32_spi_setup(priv);
  esp32_spi_select(priv);

  ret = SPI_EXCHANGE(priv->spi, txbuf, rxbuf, len);

  esp32_spi_deselect(priv);
  SPI_UNLOCK(priv->spi);

  return ret;
}

/****************************************************************************
 * Name: esp32_spi_read_reg
 *
 * Description:
 *   Read from an ESP32 SPI register
 *
 ****************************************************************************/

int esp32_spi_read_reg(FAR struct esp32_spi_priv_s *priv,
                      uint32_t addr, FAR uint8_t *value)
{
  uint8_t txbuf[5];  /* Command + address (4 bytes) + dummy byte */
  uint8_t rxbuf[5];
  uint32_t cmd;
  int ret;

  if (!priv || !value)
    {
      return -EINVAL;
    }

  /* Build SPI command for register read */
  cmd = ESP32_SPI_READ_REG_CMD |
        ((addr & ESP32_SPI_ADDR_MASK) << ESP32_SPI_HDR_ADDR_SHIFT);

  txbuf[0] = (cmd >> 0) & 0xFF;
  txbuf[1] = (cmd >> 8) & 0xFF;
  txbuf[2] = (cmd >> 16) & 0xFF;
  txbuf[3] = (cmd >> 24) & 0xFF;
  txbuf[4] = 0x00;  /* Dummy byte */

  esp32_spi_lock(priv);

  ret = esp32_spi_exchange(priv, txbuf, rxbuf, sizeof(txbuf));
  if (ret < 0)
    {
      esp32_spi_unlock(priv);
      return ret;
    }

  /* Response contains register value after header */
  *value = rxbuf[4];

  esp32_spi_unlock(priv);
  return OK;
}

/****************************************************************************
 * Name: esp32_spi_write_reg
 *
 * Description:
 *   Write to an ESP32 SPI register
 *
 ****************************************************************************/

int esp32_spi_write_reg(FAR struct esp32_spi_priv_s *priv,
                       uint32_t addr, uint8_t value)
{
  uint8_t txbuf[6];  /* Command + address (4 bytes) + data (1 byte) + dummy */
  uint8_t rxbuf[6];
  uint32_t cmd;
  int ret;

  if (!priv)
    {
      return -EINVAL;
    }

  /* Build SPI command for register write */
  cmd = ESP32_SPI_WRITE_REG_CMD |
        ((addr & ESP32_SPI_ADDR_MASK) << ESP32_SPI_HDR_ADDR_SHIFT) |
        (1 << ESP32_SPI_HDR_LEN_SHIFT);  /* 1 byte of data */

  txbuf[0] = (cmd >> 0) & 0xFF;
  txbuf[1] = (cmd >> 8) & 0xFF;
  txbuf[2] = (cmd >> 16) & 0xFF;
  txbuf[3] = (cmd >> 24) & 0xFF;
  txbuf[4] = value;  /* Data byte */
  txbuf[5] = 0x00;   /* Dummy byte */

  esp32_spi_lock(priv);

  ret = esp32_spi_exchange(priv, txbuf, rxbuf, sizeof(txbuf));
  if (ret < 0)
    {
      esp32_spi_unlock(priv);
      return ret;
    }

  esp32_spi_unlock(priv);
  return OK;
}

/****************************************************************************
 * Name: esp32_spi_read_fifo
 *
 * Description:
 *   Read from ESP32 SPI FIFO
 *
 ****************************************************************************/

int esp32_spi_read_fifo(FAR struct esp32_spi_priv_s *priv,
                       uint32_t addr, FAR uint8_t *data, size_t len)
{
  size_t total_len;
  FAR uint8_t *txbuf;
  FAR uint8_t *rxbuf;
  uint32_t cmd;
  int ret;

  if (!priv || !data || len == 0)
    {
      return -EINVAL;
    }

  /* Allocate buffers */
  total_len = 4 + len;  /* 4-byte header + data */
  txbuf = kmm_malloc(total_len);
  rxbuf = kmm_malloc(total_len);
  if (!txbuf || !rxbuf)
    {
      if (txbuf) kmm_free(txbuf);
      if (rxbuf) kmm_free(rxbuf);
      return -ENOMEM;
    }

  /* Build SPI command for FIFO read */
  cmd = ESP32_SPI_READ_FIFO_CMD |
        ((addr & ESP32_SPI_ADDR_MASK) << ESP32_SPI_HDR_ADDR_SHIFT) |
        ((len & ESP32_SPI_LEN_MASK) << ESP32_SPI_HDR_LEN_SHIFT);

  txbuf[0] = (cmd >> 0) & 0xFF;
  txbuf[1] = (cmd >> 8) & 0xFF;
  txbuf[2] = (cmd >> 16) & 0xFF;
  txbuf[3] = (cmd >> 24) & 0xFF;

  /* Fill rest of buffer with dummy bytes */
  memset(&txbuf[4], 0, len);

  esp32_spi_lock(priv);

  ret = esp32_spi_exchange(priv, txbuf, rxbuf, total_len);
  if (ret < 0)
    {
      esp32_spi_unlock(priv);
      kmm_free(txbuf);
      kmm_free(rxbuf);
      return ret;
    }

  /* Copy received data */
  memcpy(data, &rxbuf[4], len);

  esp32_spi_unlock(priv);
  kmm_free(txbuf);
  kmm_free(rxbuf);

  return OK;
}

/****************************************************************************
 * Name: esp32_spi_write_fifo
 *
 * Description:
 *   Write to ESP32 SPI FIFO
 *
 ****************************************************************************/

int esp32_spi_write_fifo(FAR struct esp32_spi_priv_s *priv,
                        uint32_t addr, FAR const uint8_t *data, size_t len)
{
  size_t total_len;
  FAR uint8_t *txbuf;
  FAR uint8_t *rxbuf;
  uint32_t cmd;
  int ret;

  if (!priv || !data || len == 0 || len > 127)
    {
      return -EINVAL;
    }

  /* Allocate buffers */
  total_len = 4 + len;  /* 4-byte header + data */
  txbuf = kmm_malloc(total_len);
  rxbuf = kmm_malloc(total_len);
  if (!txbuf || !rxbuf)
    {
      if (txbuf) kmm_free(txbuf);
      if (rxbuf) kmm_free(rxbuf);
      return -ENOMEM;
    }

  /* Build SPI command for FIFO write */
  cmd = ESP32_SPI_WRITE_FIFO_CMD |
        ((addr & ESP32_SPI_ADDR_MASK) << ESP32_SPI_HDR_ADDR_SHIFT) |
        ((len & ESP32_SPI_LEN_MASK) << ESP32_SPI_HDR_LEN_SHIFT);

  txbuf[0] = (cmd >> 0) & 0xFF;
  txbuf[1] = (cmd >> 8) & 0xFF;
  txbuf[2] = (cmd >> 16) & 0xFF;
  txbuf[3] = (cmd >> 24) & 0xFF;

  /* Copy data to send */
  memcpy(&txbuf[4], data, len);

  esp32_spi_lock(priv);

  ret = esp32_spi_exchange(priv, txbuf, rxbuf, total_len);
  if (ret < 0)
    {
      esp32_spi_unlock(priv);
      kmm_free(txbuf);
      kmm_free(rxbuf);
      return ret;
    }

  esp32_spi_unlock(priv);
  kmm_free(txbuf);
  kmm_free(rxbuf);

  return OK;
}

/****************************************************************************
 * Name: esp32_spi_get_rx_data_len
 *
 * Description:
 *   Get the length of available RX data
 *
 ****************************************************************************/

int esp32_spi_get_rx_data_len(FAR struct esp32_spi_priv_s *priv,
                              FAR uint32_t *len)
{
  uint8_t rxbuf[8];
  uint8_t txbuf[8];
  uint32_t reg_val;
  uint32_t cmd;
  int ret;

  if (!len)
    {
      return -EINVAL;
    }

  /* Build command to read packet length register */
  cmd = ESP32_SPI_READ_REG_CMD |
        ((ESP32_SPI_SLCHOST_PKT_LEN_REG & ESP32_SPI_ADDR_MASK) << ESP32_SPI_HDR_ADDR_SHIFT);

  txbuf[0] = (cmd >> 0) & 0xFF;
  txbuf[1] = (cmd >> 8) & 0xFF;
  txbuf[2] = (cmd >> 16) & 0xFF;
  txbuf[3] = (cmd >> 24) & 0xFF;
  txbuf[4] = 0x00;  /* Dummy bytes */
  txbuf[5] = 0x00;
  txbuf[6] = 0x00;
  txbuf[7] = 0x00;

  esp32_spi_lock(priv);

  ret = esp32_spi_exchange(priv, txbuf, rxbuf, sizeof(txbuf));
  if (ret < 0)
    {
      esp32_spi_unlock(priv);
      return ret;
    }

  /* Extract packet length */
  reg_val = (rxbuf[4] << 0) | (rxbuf[5] << 8) | (rxbuf[6] << 16) | (rxbuf[7] << 24);
  *len = reg_val & ESP32_SPI_ADDR_MASK; /* 20-bit length field */

  esp32_spi_unlock(priv);
  return OK;
}

/****************************************************************************
 * Name: esp32_spi_enable_interrupt
 *
 * Description:
 *   Enable SPI interrupts for ESP32
 *
 ****************************************************************************/

int esp32_spi_enable_interrupt(FAR struct esp32_spi_priv_s *priv)
{
  int ret;
  uint8_t value;

  /* Enable interrupt in ESP32 slave */
  value = ESP32_SPI_SLAVE_RX_NEW_PACKET_INT & 0xFF;
  ret = esp32_spi_write_reg(priv, ESP32_SPI_SLCHOST_INT_CLR_REG, value);
  if (ret < 0)
    {
      esp32_spierr("Failed to enable SPI interrupt: %d\n", ret);
      return ret;
    }

  esp32_spiinfo("SPI interrupts enabled\n");
  return OK;
}

/****************************************************************************
 * Name: esp32_spi_interrupt_handler
 *
 * Description:
 *   Handle SPI interrupt from ESP32
 *
 ****************************************************************************/

static int esp32_spi_interrupt_handler(int irq, void *context, void *arg)
{
  FAR struct esp32_spi_priv_s *priv = (FAR struct esp32_spi_priv_s *)arg;
  uint32_t intr_status;
  uint8_t status_byte;
  int ret;

  if (!priv)
    {
      return OK;
    }

  /* Read interrupt status from ESP32 */
  ret = esp32_spi_read_reg(priv, ESP32_SPI_SLCHOST_INT_ST_REG, &status_byte);
  if (ret < 0)
    {
      esp32_spierr("Failed to read SPI interrupt status: %d\n", ret);
      return OK;
    }

  intr_status = status_byte;

  esp32_spiinfo("SPI interrupt: 0x%02x\n", intr_status);

  /* Process interrupt */
  if (intr_status & ESP32_SPI_SLAVE_RX_NEW_PACKET_INT)
    {
      /* New packet available - notify higher layer */
      esp32_spiinfo("New packet interrupt\n");
      /* TODO: Call upper layer handler for packet processing */
    }

  /* Clear interrupt */
  ret = esp32_spi_write_reg(priv, ESP32_SPI_SLCHOST_INT_CLR_REG,
                           (uint8_t)(intr_status & 0xFF));

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: esp32_spi_initialize
 *
 * Description:
 *   Initialize ESP32 SPI interface
 *
 ****************************************************************************/

int esp32_spi_initialize(FAR struct spi_dev_s *spi_dev, int cs_gpio, int irq_gpio)
{
  FAR struct esp32_spi_priv_s *priv = &g_esp32_spi_priv;
  int ret;

  esp32_spiinfo("Initializing ESP32 SPI interface\n");

  /* Initialize private data */
  memset(priv, 0, sizeof(struct esp32_spi_priv_s));
  ret = nxsem_init(&priv->lock, 0, 1);
  if (ret < 0)
    {
      esp32_spierr("Failed to initialize semaphore: %d\n", ret);
      return ret;
    }

  priv->spi = spi_dev;
  priv->cs_gpio = cs_gpio;
  priv->irq_gpio = irq_gpio;
  priv->initialized = true;

  /* Configure GPIO pins */
  if (cs_gpio >= 0)
    {
      ret = gpio_config(cs_gpio, OUTPUT | GPIO_OUTPUT_ONE);
      if (ret < 0)
        {
          esp32_spierr("Failed to configure CS GPIO: %d\n", ret);
          nxsem_destroy(&priv->lock);
          return ret;
        }
      gpio_write(cs_gpio, 1);  /* Deselect initially */
    }

  if (irq_gpio >= 0)
    {
      ret = gpio_config(irq_gpio, INPUT | GPIO_INT | GPIO_PULLUP);
      if (ret < 0)
        {
          esp32_spierr("Failed to configure IRQ GPIO: %d\n", ret);
          nxsem_destroy(&priv->lock);
          return ret;
        }

      /* Attach interrupt handler */
      ret = gpio_irq_attach(irq_gpio, esp32_spi_interrupt_handler, priv);
      if (ret < 0)
        {
          esp32_spierr("Failed to attach IRQ handler: %d\n", ret);
          nxsem_destroy(&priv->lock);
          return ret;
        }

      /* Enable interrupt */
      gpio_irq_enable(irq_gpio);
    }

  /* Configure SPI parameters */
  esp32_spi_setup(priv);

  esp32_spiinfo("ESP32 SPI interface initialized\n");
  return OK;
}

/****************************************************************************
 * Name: esp32_spi_uninitialize
 *
 * Description:
 *   Uninitialize ESP32 SPI interface
 *
 ****************************************************************************/

int esp32_spi_uninitialize(void)
{
  FAR struct esp32_spi_priv_s *priv = &g_esp32_spi_priv;

  if (!priv->initialized)
    {
      return -ENODEV;
    }

  esp32_spiinfo("Uninitializing ESP32 SPI interface\n");

  /* Disable interrupts */
  if (priv->irq_gpio >= 0)
    {
      gpio_irq_disable(priv->irq_gpio);
      gpio_irq_detach(priv->irq_gpio);
    }

  /* Destroy lock */
  nxsem_destroy(&priv->lock);

  /* Clear initialization state */
  memset(priv, 0, sizeof(struct esp32_spi_priv_s));
  priv->initialized = false;

  esp32_spiinfo("ESP32 SPI interface uninitialized\n");
  return OK;
}

/****************************************************************************
 * Name: esp32_spi_read_packet
 *
 * Description:
 *   Read a packet from ESP32 via SPI
 *
 ****************************************************************************/

int esp32_spi_read_packet(FAR uint8_t *buffer, FAR size_t *buf_len)
{
  FAR struct esp32_spi_priv_s *priv = &g_esp32_spi_priv;
  uint32_t packet_len;
  int ret;

  if (!priv->initialized || !buffer || !buf_len)
    {
      return -EINVAL;
    }

  /* Get packet length */
  ret = esp32_spi_get_rx_data_len(priv, &packet_len);
  if (ret < 0)
    {
      esp32_spierr("Failed to get RX data length: %d\n", ret);
      return ret;
    }

  if (packet_len == 0)
    {
      *buf_len = 0;
      return OK;
    }

  if (packet_len > *buf_len)
    {
      esp32_spierr("Buffer too small: need %u, have %zu\n", packet_len, *buf_len);
      return -EMSGSIZE;
    }

  /* Read packet data from FIFO */
  ret = esp32_spi_read_fifo(priv, 0x1F800 - packet_len, buffer, packet_len);
  if (ret < 0)
    {
      esp32_spierr("Failed to read packet: %d\n", ret);
      return ret;
    }

  *buf_len = packet_len;
  return OK;
}

/****************************************************************************
 * Name: esp32_spi_write_packet
 *
 * Description:
 *   Write a packet to ESP32 via SPI
 *
 ****************************************************************************/

int esp32_spi_write_packet(FAR const uint8_t *buffer, size_t buf_len)
{
  FAR struct esp32_spi_priv_s *priv = &g_esp32_spi_priv;
  int ret;

  if (!priv->initialized || !buffer || buf_len == 0)
    {
      return -EINVAL;
    }

  /* Write packet data to FIFO */
  ret = esp32_spi_write_fifo(priv, 0x1F800 - buf_len, buffer, buf_len);
  if (ret < 0)
    {
      esp32_spierr("Failed to write packet: %d\n", ret);
      return ret;
    }

  esp32_spiinfo("Wrote packet: %zu bytes\n", buf_len);
  return OK;
}

/****************************************************************************
 * Name: esp32_spi_get_intr_status
 *
 * Description:
 *   Get ESP32 SPI interrupt status
 *
 ****************************************************************************/

int esp32_spi_get_intr_status(FAR uint32_t *intr_status)
{
  FAR struct esp32_spi_priv_s *priv = &g_esp32_spi_priv;
  uint8_t status_byte;
  int ret;

  if (!priv->initialized || !intr_status)
    {
      return -EINVAL;
    }

  ret = esp32_spi_read_reg(priv, ESP32_SPI_SLCHOST_INT_ST_REG, &status_byte);
  if (ret < 0)
    {
      return ret;
    }

  *intr_status = status_byte;
  return OK;
}

/****************************************************************************
 * Name: esp32_spi_clear_intr
 *
 * Description:
 *   Clear ESP32 SPI interrupts
 *
 ****************************************************************************/

int esp32_spi_clear_intr(uint32_t intr_mask)
{
  FAR struct esp32_spi_priv_s *priv = &g_esp32_spi_priv;
  int ret;

  if (!priv->initialized)
    {
      return -ENODEV;
    }

  ret = esp32_spi_write_reg(priv, ESP32_SPI_SLCHOST_INT_CLR_REG,
                           (uint8_t)(intr_mask & 0xFF));
  return ret;
}

#endif /* CONFIG_ESP32_WIFI_SPI */