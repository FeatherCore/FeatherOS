/****************************************************************************
 * arch/arm/src/ra8p/ra8p_i2s.c
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
#include <nuttx/arch.h>
#include <nuttx/irq.h>
#include <nuttx/kmalloc.h>
#include <nuttx/mutex.h>
#include <nuttx/audio/i2s.h>
#include <arch/board/board.h>

#include "arm_internal.h"
#include "chip.h"
#include "ra8p_config.h"
#include "ra8p_peripherals.h"
#include "hardware/ra8p_i2s.h"

#ifdef CONFIG_RA8P_I2S0

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Helper macros for register access */

#define ra8p_i2s_putreg32(offset, val) \
  putreg32((val), RA8P_I2S0_BASE + (offset))

#define ra8p_i2s_getreg32(offset) \
  getreg32(RA8P_I2S0_BASE + (offset))

#define ra8p_i2s_modifyreg32(offset, clrbits, setbits) \
  ra8p_i2s_putreg32(offset, \
    (ra8p_i2s_getreg32(offset) & ~(clrbits)) | (setbits))

/* I2S timeout */

#define RA8P_I2S_TIMEOUT_MS                    (100)

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_i2s_dev_s
{
  struct i2s_dev_s dev;             /* I2S interface */
  uint32_t base;                    /* Base address of I2S registers */
  int irq;                          /* I2S interrupt number */
  mutex_t lock;                     /* Thread-safe lock */
  sem_t wait_sem;                   /* Wait semaphore for interrupt handling */
  enum ra8p_i2s_format_e format;   /* Audio format */
  enum ra8p_i2s_word_length_e word_len; /* Word length */
  uint32_t sample_rate;             /* Sample rate in Hz */
  uint8_t channels;                 /* Number of channels */
  bool master_mode;                /* True for master mode */
  bool initialized;                 /* True if initialized */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int ra8p_i2s_configure(struct i2s_dev_s *dev,
                              struct i2s_config_s *config);
static int ra8p_i2s_start(struct i2s_dev_s *dev);
static void ra8p_i2s_stop(struct i2s_dev_s *dev);
static int ra8p_i2s_write(struct i2s_dev_s *dev,
                            const void *buffer, size_t nbytes);
static int ra8p_i2s_read(struct i2s_dev_s *dev,
                           void *buffer, size_t nbytes);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_i2s_dev_s g_i2s0_priv;

static const struct i2s_ops_s g_i2s_ops =
{
  .configure = ra8p_i2s_configure,
  .start     = ra8p_i2s_start,
  .stop      = ra8p_i2s_stop,
  .write     = ra8p_i2s_write,
  .read      = ra8p_i2s_read,
};

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_i2s_wait_tx_empty
 ****************************************************************************/

static int ra8p_i2s_wait_tx_empty(void)
{
  uint32_t timeout = RA8P_I2S_TIMEOUT_MS * 1000;
  uint32_t status;

  while (timeout > 0)
    {
      status = ra8p_i2s_getreg32(RA8P_I2S_SSISR_OFFSET);
      if (status & RA8P_I2S_SSISR_TFEMP)
        {
          return OK;
        }

      up_udelay(1);
      timeout--;
    }

  return -ETIMEDOUT;
}

/****************************************************************************
 * Name: ra8p_i2s_wait_rx_full
 ****************************************************************************/

static int ra8p_i2s_wait_rx_full(void)
{
  uint32_t timeout = RA8P_I2S_TIMEOUT_MS * 1000;
  uint32_t status;

  while (timeout > 0)
    {
      status = ra8p_i2s_getreg32(RA8P_I2S_SSISR_OFFSET);
      if (status & RA8P_I2S_SSISR_RFFUL)
        {
          return OK;
        }

      up_udelay(1);
      timeout--;
    }

  return -ETIMEDOUT;
}

/****************************************************************************
 * Name: ra8p_i2s_configure
 ****************************************************************************/

static int ra8p_i2s_configure(struct i2s_dev_s *dev,
                              struct i2s_config_s *config)
{
  struct ra8p_i2s_dev_s *priv = (struct ra8p_i2s_dev_s *)dev;
  uint32_t regval;
  uint32_t clkdiv;
  uint32_t base_clock = 48000000;  /* 48 MHz base clock */

  if (!priv || !config)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Calculate clock divider */

  clkdiv = (base_clock / (config->sample_rate * config->word_length * 2)) - 1;
  if (clkdiv > 255)
    {
      clkdiv = 255;
    }

  /* Configure clock divider */

  ra8p_i2s_putreg32(RA8P_I2S_SSICCR_OFFSET, clkdiv << RA8P_I2S_SSICCR_CKDIV_SHIFT);

  /* Configure format and word length */

  regval = 0;

  switch (config->format)
    {
      case I2S_FORMAT_I2S:
        regval |= (RA8P_I2S_SSICR0_AFS << RA8P_I2S_SSICR0_AFS_SHIFT);
        break;

      case I2S_FORMAT_LEFT:
        regval |= (RA8P_I2S_SSICR0_AFS_LEFT << RA8P_I2S_SSICR0_AFS_SHIFT);
        break;

      case I2S_FORMAT_RIGHT:
        regval |= (RA8P_I2S_SSICR0_AFS_RIGHT << RA8P_I2S_SSICR0_AFS_SHIFT);
        break;

      default:
        break;
    }

  switch (config->word_length)
    {
      case 8:
        regval |= (RA8P_I2S_SSICR0_SWL_8BIT << RA8P_I2S_SSICR0_SWL_SHIFT);
        regval |= (RA8P_I2S_SSICR0_DWL_8BIT << RA8P_I2S_SSICR0_DWL_SHIFT);
        break;

      case 16:
        regval |= (RA8P_I2S_SSICR0_SWL_16BIT << RA8P_I2S_SSICR0_SWL_SHIFT);
        regval |= (RA8P_I2S_SSICR0_DWL_16BIT << RA8P_I2S_SSICR0_DWL_SHIFT);
        break;

      case 24:
        regval |= (RA8P_I2S_SSICR0_SWL_24BIT << RA8P_I2S_SSICR0_SWL_SHIFT);
        regval |= (RA8P_I2S_SSICR0_DWL_24BIT << RA8P_I2S_SSICR0_DWL_SHIFT);
        break;

      case 32:
        regval |= (RA8P_I2S_SSICR0_SWL_32BIT << RA8P_I2S_SSICR0_SWL_SHIFT);
        regval |= (RA8P_I2S_SSICR0_DWL_32BIT << RA8P_I2S_SSICR0_DWL_SHIFT);
        break;

      default:
        break;
    }

  /* Set master/slave mode */

  if (config->master_mode)
    {
      regval |= RA8P_I2S_SSICR0_MST;
    }

  /* Set channel number */

  regval |= ((config->channels - 1) << RA8P_I2S_SSICR1_TCHNL_SHIFT);

  ra8p_i2s_putreg32(RA8P_I2S_SSICR0_OFFSET, regval);

  /* Update private data */

  priv->format = config->format;
  priv->word_len = config->word_length;
  priv->sample_rate = config->sample_rate;
  priv->channels = config->channels;
  priv->master_mode = config->master_mode;

  nxmutex_unlock(&priv->lock);
  return OK;
}

/****************************************************************************
 * Name: ra8p_i2s_start
 ****************************************************************************/

static int ra8p_i2s_start(struct i2s_dev_s *dev)
{
  struct ra8p_i2s_dev_s *priv = (struct ra8p_i2s_dev_s *)dev;

  if (!priv)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Enable I2S */

  ra8p_i2s_modifyreg32(RA8P_I2S_SSICR0_OFFSET, 0, RA8P_I2S_SSICR0_SSIEN);

  /* Enable transmit and receive if configured */

  if (priv->channels > 0)
    {
      ra8p_i2s_modifyreg32(RA8P_I2S_SSICR0_OFFSET, 0, RA8P_I2S_SSICR0_TEN);
      ra8p_i2s_modifyreg32(RA8P_I2S_SSICR0_OFFSET, 0, RA8P_I2S_SSICR0_REN);
    }

  priv->initialized = true;

  nxmutex_unlock(&priv->lock);
  return OK;
}

/****************************************************************************
 * Name: ra8p_i2s_stop
 ****************************************************************************/

static void ra8p_i2s_stop(struct i2s_dev_s *dev)
{
  struct ra8p_i2s_dev_s *priv = (struct ra8p_i2s_dev_s *)dev;

  if (!priv)
    {
      return;
    }

  nxmutex_lock(&priv->lock);

  /* Disable transmit and receive */

  ra8p_i2s_modifyreg32(RA8P_I2S_SSICR0_OFFSET,
                       RA8P_I2S_SSICR0_TEN | RA8P_I2S_SSICR0_REN, 0);

  /* Disable I2S */

  ra8p_i2s_modifyreg32(RA8P_I2S_SSICR0_OFFSET, RA8P_I2S_SSICR0_SSIEN, 0);

  priv->initialized = false;

  nxmutex_unlock(&priv->lock);
}

/****************************************************************************
 * Name: ra8p_i2s_write
 ****************************************************************************/

static int ra8p_i2s_write(struct i2s_dev_s *dev,
                            const void *buffer, size_t nbytes)
{
  struct ra8p_i2s_dev_s *priv = (struct ra8p_i2s_dev_s *)dev;
  const uint8_t *src = (const uint8_t *)buffer;
  size_t nwords;
  size_t i;
  int ret;
  uint32_t data;

  if (!priv || !buffer || nbytes == 0 || !priv->initialized)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Calculate number of words to write */

  nwords = nbytes / (priv->word_len / 8);

  /* Write data to transmit FIFO */

  for (i = 0; i < nwords; i++)
    {
      /* Wait for TX FIFO to be empty */

      ret = ra8p_i2s_wait_tx_empty();
      if (ret < 0)
        {
            nxmutex_unlock(&priv->lock);
            return ret;
        }

      /* Prepare data based on word length */

      if (priv->word_len == 8)
        {
            data = src[i];
        }
      else if (priv->word_len == 16)
        {
            data = ((uint16_t *)src)[i];
        }
      else
        {
            data = ((uint32_t *)src)[i];
        }

      /* Write to transmit data register */

      ra8p_i2s_putreg32(RA8P_I2S_SSITDR_OFFSET, data);
    }

  nxmutex_unlock(&priv->lock);
  return nbytes;
}

/****************************************************************************
 * Name: ra8p_i2s_read
 ****************************************************************************/

static int ra8p_i2s_read(struct i2s_dev_s *dev,
                           void *buffer, size_t nbytes)
{
  struct ra8p_i2s_dev_s *priv = (struct ra8p_i2s_dev_s *)dev;
  uint8_t *dst = (uint8_t *)buffer;
  size_t nwords;
  size_t i;
  int ret;
  uint32_t data;

  if (!priv || !buffer || nbytes == 0 || !priv->initialized)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Calculate number of words to read */

  nwords = nbytes / (priv->word_len / 8);

  /* Read data from receive FIFO */

  for (i = 0; i < nwords; i++)
    {
      /* Wait for RX FIFO to be full */

      ret = ra8p_i2s_wait_rx_full();
      if (ret < 0)
        {
            nxmutex_unlock(&priv->lock);
            return ret;
        }

      /* Read from receive data register */

      data = ra8p_i2s_getreg32(RA8P_I2S_SSIRDR_OFFSET);

      /* Store data based on word length */

      if (priv->word_len == 8)
        {
            dst[i] = (uint8_t)data;
        }
      else if (priv->word_len == 16)
        {
            ((uint16_t *)dst)[i] = (uint16_t)data;
        }
      else
        {
            ((uint32_t *)dst)[i] = data;
        }
    }

  nxmutex_unlock(&priv->lock);
  return nbytes;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_i2s_initialize
 *
 * Description:
 *   Initialize the I2S/SSIE driver
 *
 ****************************************************************************/

int ra8p_i2s_initialize(void)
{
  struct ra8p_i2s_dev_s *priv = &g_i2s0_priv;
  int ret;

  /* Initialize private data structure */

  memset(priv, 0, sizeof(struct ra8p_i2s_dev_s));
  priv->base = RA8P_I2S0_BASE;
  priv->irq = RA8P_IRQ_I2S0;

  nxmutex_init(&priv->lock);
  nxsem_init(&priv->wait_sem, 0, 0);

  /* Initialize I2S ops */

  priv->dev.ops = &g_i2s_ops;

  /* Configure default values */

  priv->format = RA8P_I2S_FORMAT_I2S;
  priv->word_len = RA8P_I2S_WLEN_16BIT;
  priv->sample_rate = 44100;
  priv->channels = 2;
  priv->master_mode = true;

  /* Register I2S device */

  ret = i2s_register("/dev/i2s0", &priv->dev);
  if (ret < 0)
    {
      goto errout_with_locks;
    }

  return OK;

errout_with_locks:
  nxmutex_destroy(&priv->lock);
  nxsem_destroy(&priv->wait_sem);
  return ret;
}

#endif /* CONFIG_RA8P_I2S0 */