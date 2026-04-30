/****************************************************************************
 * arch/arm/src/ra8p/ra8p_dac.c
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
#include <nuttx/analog/dac.h>
#include <arch/board/board.h>

#include "arm_internal.h"
#include "chip.h"
#include "ra8p_config.h"
#include "ra8p_peripherals.h"
#include "hardware/ra8p_dac.h"

#ifdef CONFIG_RA8P_DAC0

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Helper macros for register access */

#define ra8p_dac_putreg16(base, offset, val) \
  putreg16((val), (base) + (offset))

#define ra8p_dac_getreg16(base, offset) \
  getreg16((base) + (offset))

#define ra8p_dac_modifyreg16(base, offset, clrbits, setbits) \
  ra8p_dac_putreg16(base, offset, \
    (ra8p_dac_getreg16(base, offset) & ~(clrbits)) | (setbits))

/* DAC resolution */

#define RA8P_DAC_RESOLUTION                   (12)
#define RA8P_DAC_MAX_VALUE                    (0xFFF)

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_dac_dev_s
{
  struct dac_dev_s dev;             /* DAC interface */
  uint32_t base;                    /* Base address of DAC registers */
  mutex_t lock;                     /* Thread-safe lock */
  uint8_t channels;                 /* Number of channels */
  bool initialized;                 /* True if initialized */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int ra8p_dac_bind(struct dac_dev_s *dev,
                         const struct dac_callback_s *callback);
static void ra8p_dac_reset(struct dac_dev_s *dev);
static int ra8p_dac_setup(struct dac_dev_s *dev);
static void ra8p_dac_shutdown(struct dac_dev_s *dev);
static int ra8p_dac_ioctl(struct dac_dev_s *dev, int cmd,
                          unsigned long arg);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_dac_dev_s g_dac0_priv;

static const struct dac_ops_s g_dac_ops =
{
  .ao_bind     = ra8p_dac_bind,
  .ao_reset    = ra8p_dac_reset,
  .ao_setup    = ra8p_dac_setup,
  .ao_shutdown = ra8p_dac_shutdown,
  .ao_ioctl    = ra8p_dac_ioctl,
};

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_dac_bind
 ****************************************************************************/

static int ra8p_dac_bind(struct dac_dev_s *dev,
                         const struct dac_callback_s *callback)
{
  struct ra8p_dac_dev_s *priv = (struct ra8p_dac_dev_s *)dev;

  if (!priv)
    {
      return -EINVAL;
    }

  /* Save the callback structure */

  priv->dev.ad_callback = callback;
  return OK;
}

/****************************************************************************
 * Name: ra8p_dac_reset
 ****************************************************************************/

static void ra8p_dac_reset(struct dac_dev_s *dev)
{
  struct ra8p_dac_dev_s *priv = (struct ra8p_dac_dev_s *)dev;

  if (!priv)
    {
      return;
    }

  /* Disable DAC output */

  ra8p_dac_modifyreg16(priv->base, RA8P_DAC_DACR_OFFSET,
                       RA8P_DAC_DACR_DAOE, 0);

  /* Clear DAC data registers */

  ra8p_dac_putreg16(priv->base, RA8P_DAC_DADR_OFFSET, 0);
  ra8p_dac_putreg16(priv->base, RA8P_DAC_DADRB_OFFSET, 0);

  /* Disable DAC power */

  ra8p_dac_modifyreg16(priv->base, RA8P_DAC_DACPCON_OFFSET,
                       RA8P_DAC_DACPCON_DACEN, 0);
}

/****************************************************************************
 * Name: ra8p_dac_setup
 ****************************************************************************/

static int ra8p_dac_setup(struct dac_dev_s *dev)
{
  struct ra8p_dac_dev_s *priv = (struct ra8p_dac_dev_s *)dev;

  if (!priv)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Enable DAC power */

  ra8p_dac_modifyreg16(priv->base, RA8P_DAC_DACPCON_OFFSET,
                       0, RA8P_DAC_DACPCON_DACEN);

  /* Wait for power to stabilize */

  up_udelay(10);

  /* Configure DAC:
   * - Normal mode (not sine wave)
   * - Normal gain
   * - Internal reference voltage
   */

  ra8p_dac_putreg16(priv->base, RA8P_DAC_DACR_OFFSET, 0);
  ra8p_dac_putreg16(priv->base, RA8P_DAC_DADRVR_OFFSET,
                    RA8P_DAC_DADRVR_REFC_INT_VREF);

  priv->initialized = true;

  nxmutex_unlock(&priv->lock);
  return OK;
}

/****************************************************************************
 * Name: ra8p_dac_shutdown
 ****************************************************************************/

static void ra8p_dac_shutdown(struct dac_dev_s *dev)
{
  struct ra8p_dac_dev_s *priv = (struct ra8p_dac_dev_s *)dev;

  if (!priv)
    {
      return;
    }

  nxmutex_lock(&priv->lock);

  /* Disable DAC output */

  ra8p_dac_modifyreg16(priv->base, RA8P_DAC_DACR_OFFSET,
                       RA8P_DAC_DACR_DAOE, 0);

  /* Disable DAC power */

  ra8p_dac_modifyreg16(priv->base, RA8P_DAC_DACPCON_OFFSET,
                       RA8P_DAC_DACPCON_DACEN, 0);

  priv->initialized = false;

  nxmutex_unlock(&priv->lock);
}

/****************************************************************************
 * Name: ra8p_dac_ioctl
 ****************************************************************************/

static int ra8p_dac_ioctl(struct dac_dev_s *dev, int cmd,
                          unsigned long arg)
{
  struct ra8p_dac_dev_s *priv = (struct ra8p_dac_dev_s *)dev;
  int ret = OK;

  if (!priv)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  switch (cmd)
    {
      case ANIOC_DAC_OUTPUT:
        {
          /* Enable/disable DAC output */

          if (arg)
            {
              ra8p_dac_modifyreg16(priv->base, RA8P_DAC_DACR_OFFSET,
                                   0, RA8P_DAC_DACR_DAOE);
            }
          else
            {
              ra8p_dac_modifyreg16(priv->base, RA8P_DAC_DACR_OFFSET,
                                   RA8P_DAC_DACR_DAOE, 0);
            }
        }
        break;

      case ANIOC_DAC_SET_GAIN:
        {
          /* Set amplifier gain */

          uint16_t gain = (uint16_t)arg;
          if (gain <= 7)
            {
              ra8p_dac_modifyreg16(priv->base, RA8P_DAC_DACAAMP_OFFSET,
                                   RA8P_DAC_DACAAMP_AGAIN_MASK,
                                   (gain << RA8P_DAC_DACAAMP_AGAIN_SHIFT));
            }
          else
            {
              ret = -EINVAL;
            }
        }
        break;

      case ANIOC_GET_NCHANNELS:
        {
          /* Return the number of channels */

          ret = priv->channels;
        }
        break;

      case ANIOC_GET_RESOLUTION:
        {
          /* Return the DAC resolution */

          ret = RA8P_DAC_RESOLUTION;
        }
        break;

      default:
        ret = -ENOTTY;
        break;
    }

  nxmutex_unlock(&priv->lock);
  return ret;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_dac_initialize
 *
 * Description:
 *   Initialize the DAC driver
 *
 ****************************************************************************/

int ra8p_dac_initialize(void)
{
  struct ra8p_dac_dev_s *priv = &g_dac0_priv;
  int ret;

  /* Initialize private data structure */

  memset(priv, 0, sizeof(struct ra8p_dac_dev_s));
  priv->base = RA8P_DAC0_BASE;
  priv->channels = 2;  /* DAC0 has 2 channels */

  nxmutex_init(&priv->lock);

  /* Initialize DAC ops */

  priv->dev.ad_ops = &g_dac_ops;

  /* Register the DAC driver */

  ret = dac_register("/dev/dac0", &priv->dev);
  if (ret < 0)
    {
      goto errout_with_mutex;
    }

  return OK;

errout_with_mutex:
  nxmutex_destroy(&priv->lock);
  return ret;
}

/****************************************************************************
 * Name: ra8p_dac_write
 *
 * Description:
 *   Write a value to the DAC
 *
 ****************************************************************************/

int ra8p_dac_write(uint8_t channel, uint16_t value)
{
  struct ra8p_dac_dev_s *priv = &g_dac0_priv;

  if (!priv || channel >= priv->channels)
    {
      return -EINVAL;
    }

  /* Limit value to 12-bit range */

  if (value > RA8P_DAC_MAX_VALUE)
    {
      value = RA8P_DAC_MAX_VALUE;
    }

  nxmutex_lock(&priv->lock);

  /* Write value to appropriate register */

  if (channel == 0)
    {
      ra8p_dac_putreg16(priv->base, RA8P_DAC_DADR_OFFSET, value);
    }
  else
    {
      ra8p_dac_putreg16(priv->base, RA8P_DAC_DADRB_OFFSET, value);
    }

  nxmutex_unlock(&priv->lock);
  return OK;
}

/****************************************************************************
 * Name: ra8p_dac_enable_output
 *
 * Description:
 *   Enable or disable DAC output
 *
 ****************************************************************************/

int ra8p_dac_enable_output(uint8_t channel, bool enable)
{
  struct ra8p_dac_dev_s *priv = &g_dac0_priv;

  if (!priv || channel >= priv->channels)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  if (enable)
    {
      if (channel == 0)
        {
          ra8p_dac_modifyreg16(priv->base, RA8P_DAC_DACR_OFFSET,
                               0, RA8P_DAC_DACR_DAOE);
        }
      else
        {
          ra8p_dac_modifyreg16(priv->base, RA8P_DAC_DACR_OFFSET,
                               0, RA8P_DAC_DACR_DAOE1);
        }
    }
  else
    {
      if (channel == 0)
        {
          ra8p_dac_modifyreg16(priv->base, RA8P_DAC_DACR_OFFSET,
                               RA8P_DAC_DACR_DAOE, 0);
        }
      else
        {
          ra8p_dac_modifyreg16(priv->base, RA8P_DAC_DACR_OFFSET,
                               RA8P_DAC_DACR_DAOE1, 0);
        }
    }

  nxmutex_unlock(&priv->lock);
  return OK;
}

#endif /* CONFIG_RA8P_DAC0 */