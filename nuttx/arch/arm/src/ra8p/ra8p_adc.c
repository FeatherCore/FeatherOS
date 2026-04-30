/****************************************************************************
 * arch/arm/src/ra8p/ra8p_adc.c
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
#include <nuttx/analog/adc.h>
#include <arch/board/board.h>

#include "arm_internal.h"
#include "chip.h"
#include "ra8p_config.h"
#include "ra8p_peripherals.h"
#include "hardware/ra8p_adc.h"

#ifdef CONFIG_RA8P_ADC0

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Helper macros for register access */

#define ra8p_adc_putreg32(base, offset, val) \
  putreg32((val), (base) + (offset))

#define ra8p_adc_getreg32(base, offset) \
  getreg32((base) + (offset))

#define ra8p_adc_putreg16(base, offset, val) \
  putreg16((val), (base) + (offset))

#define ra8p_adc_getreg16(base, offset) \
  getreg16((base) + (offset))

#define ra8p_adc_putreg8(base, offset, val) \
  putreg8((val), (base) + (offset))

#define ra8p_adc_getreg8(base, offset) \
  getreg8((base) + (offset))

#define ra8p_adc_modifyreg32(base, offset, clrbits, setbits) \
  ra8p_adc_putreg32(base, offset, \
    (ra8p_adc_getreg32(base, offset) & ~(clrbits)) | (setbits))

/* ADC resolution */

#define RA8P_ADC_RESOLUTION_12BIT             (12)
#define RA8P_ADC_RESOLUTION_16BIT             (16)

/* Reference voltage sources */

#define RA8P_ADC_REF_VCC                      (0)  /* VCC reference */
#define RA8P_ADC_REF_INTERNAL                 (1)  /* Internal reference */
#define RA8P_ADC_REF_EXTERNAL                 (2)  /* External reference */

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_adc_dev_s
{
  struct adc_dev_s dev;             /* ADC interface */
  uint32_t base;                    /* Base address of ADC registers */
  int irq;                          /* ADC interrupt number */
  mutex_t lock;                     /* Thread-safe lock */
  uint8_t resolution;               /* ADC resolution */
  uint8_t channels;                 /* Number of channels */
  bool initialized;                 /* True if initialized */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int ra8p_adc_bind(struct adc_dev_s *dev,
                         const struct adc_callback_s *callback);
static void ra8p_adc_reset(struct adc_dev_s *dev);
static int ra8p_adc_setup(struct adc_dev_s *dev);
static void ra8p_adc_shutdown(struct adc_dev_s *dev);
static void ra8p_adc_rxint(struct adc_dev_s *dev, bool enable);
static int ra8p_adc_ioctl(struct adc_dev_s *dev, int cmd,
                          unsigned long arg);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_adc_dev_s g_adc0_priv;

static const struct adc_ops_s g_adc_ops =
{
  .ao_bind     = ra8p_adc_bind,
  .ao_reset    = ra8p_adc_reset,
  .ao_setup    = ra8p_adc_setup,
  .ao_shutdown = ra8p_adc_shutdown,
  .ao_rxint    = ra8p_adc_rxint,
  .ao_ioctl    = ra8p_adc_ioctl,
};

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_adc_bind
 ****************************************************************************/

static int ra8p_adc_bind(struct adc_dev_s *dev,
                         const struct adc_callback_s *callback)
{
  struct ra8p_adc_dev_s *priv = (struct ra8p_adc_dev_s *)dev;

  if (!priv)
    {
      return -EINVAL;
    }

  /* Save the callback structure */

  priv->dev.ad_callback = callback;
  return OK;
}

/****************************************************************************
 * Name: ra8p_adc_reset
 ****************************************************************************/

static void ra8p_adc_reset(struct adc_dev_s *dev)
{
  struct ra8p_adc_dev_s *priv = (struct ra8p_adc_dev_s *)dev;

  if (!priv)
    {
      return;
    }

  /* Stop any ongoing conversion */

  ra8p_adc_modifyreg32(priv->base, RA8P_ADC_ADCR2_OFFSET,
                       0, RA8P_ADC_ADCR2_ADSTP);

  /* Wait for stop to complete */

  while ((ra8p_adc_getreg32(priv->base, RA8P_ADC_ADCR2_OFFSET) &
          RA8P_ADC_ADCR2_ADSTPC) != 0);

  /* Clear all registers */

  ra8p_adc_putreg32(priv->base, RA8P_ADC_ADCR_OFFSET, 0);
  ra8p_adc_putreg32(priv->base, RA8P_ADC_ADANSA_OFFSET, 0);
  ra8p_adc_putreg32(priv->base, RA8P_ADC_ADANSB_OFFSET, 0);
  ra8p_adc_putreg32(priv->base, RA8P_ADC_ADIER_OFFSET, 0);
  ra8p_adc_putreg32(priv->base, RA8P_ADC_ADISR_OFFSET, 0);
}

/****************************************************************************
 * Name: ra8p_adc_setup
 ****************************************************************************/

static int ra8p_adc_setup(struct adc_dev_s *dev)
{
  struct ra8p_adc_dev_s *priv = (struct ra8p_adc_dev_s *)dev;
  uint32_t regval;

  if (!priv)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Enable ADC power */

  ra8p_adc_modifyreg32(priv->base, RA8P_ADC_ADPWR_OFFSET,
                       0, RA8P_ADC_ADPWR_ADPC);

  /* Wait for power to stabilize */

  up_udelay(10);

  /* Configure ADC:
   * - Single scan mode
   * - Hardware trigger disabled
   * - 12-bit resolution
   */

  regval = RA8P_ADC_ADCR_ADCS_SINGLE;
  ra8p_adc_putreg32(priv->base, RA8P_ADC_ADCR_OFFSET, regval);

  /* Set sampling time */

  ra8p_adc_putreg8(priv->base, RA8P_ADC_ADSSTR_OFFSET, RA8P_ADC_ADSST_MIN);

  /* Enable ADC interrupt */

  ra8p_adc_modifyreg32(priv->base, RA8P_ADC_ADIER_OFFSET,
                       0, RA8P_ADC_ADIER_ADIE);

  priv->initialized = true;

  nxmutex_unlock(&priv->lock);
  return OK;
}

/****************************************************************************
 * Name: ra8p_adc_shutdown
 ****************************************************************************/

static void ra8p_adc_shutdown(struct adc_dev_s *dev)
{
  struct ra8p_adc_dev_s *priv = (struct ra8p_adc_dev_s *)dev;

  if (!priv)
    {
      return;
    }

  nxmutex_lock(&priv->lock);

  /* Stop any ongoing conversion */

  ra8p_adc_modifyreg32(priv->base, RA8P_ADC_ADCR2_OFFSET,
                       0, RA8P_ADC_ADCR2_ADSTP);

  /* Disable ADC interrupt */

  ra8p_adc_modifyreg32(priv->base, RA8P_ADC_ADIER_OFFSET,
                       RA8P_ADC_ADIER_ADIE, 0);

  /* Disable ADC power */

  ra8p_adc_modifyreg32(priv->base, RA8P_ADC_ADPWR_OFFSET,
                       RA8P_ADC_ADPWR_ADPC, 0);

  priv->initialized = false;

  nxmutex_unlock(&priv->lock);
}

/****************************************************************************
 * Name: ra8p_adc_rxint
 ****************************************************************************/

static void ra8p_adc_rxint(struct adc_dev_s *dev, bool enable)
{
  struct ra8p_adc_dev_s *priv = (struct ra8p_adc_dev_s *)dev;

  if (!priv)
    {
      return;
    }

  if (enable)
    {
      ra8p_adc_modifyreg32(priv->base, RA8P_ADC_ADIER_OFFSET,
                           0, RA8P_ADC_ADIER_ADIE);
    }
  else
    {
      ra8p_adc_modifyreg32(priv->base, RA8P_ADC_ADIER_OFFSET,
                           RA8P_ADC_ADIER_ADIE, 0);
    }
}

/****************************************************************************
 * Name: ra8p_adc_ioctl
 ****************************************************************************/

static int ra8p_adc_ioctl(struct adc_dev_s *dev, int cmd,
                          unsigned long arg)
{
  struct ra8p_adc_dev_s *priv = (struct ra8p_adc_dev_s *)dev;
  int ret = OK;

  if (!priv)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  switch (cmd)
    {
      case ANIOC_TRIGGER:
        {
          /* Start a single conversion */

          ra8p_adc_modifyreg32(priv->base, RA8P_ADC_ADCR_OFFSET,
                               0, RA8P_ADC_ADCR_ADST);
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
          /* Return the ADC resolution */

          ret = priv->resolution;
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
 * Name: ra8p_adc_interrupt
 ****************************************************************************/

static int ra8p_adc_interrupt(int irq, void *context, void *arg)
{
  struct ra8p_adc_dev_s *priv = (struct ra8p_adc_dev_s *)arg;
  uint32_t status;
  uint16_t data;
  int channel;

  if (!priv)
    {
      return OK;
    }

  /* Read interrupt status */

  status = ra8p_adc_getreg32(priv->base, RA8P_ADC_ADISR_OFFSET);

  /* Clear interrupt flags */

  ra8p_adc_putreg32(priv->base, RA8P_ADC_ADISR_OFFSET, status);

  /* Check for conversion complete */

  if (status & RA8P_ADC_ADISR_ADIF)
    {
      /* Read data from all enabled channels */

      uint32_t chansel = ra8p_adc_getreg32(priv->base, RA8P_ADC_ADANSA_OFFSET);

      for (channel = 0; channel < 32; channel++)
        {
          if (chansel & (1 << channel))
            {
              /* Read channel data */

              data = ra8p_adc_getreg16(priv->base,
                                        RA8P_ADC_ADDR_OFFSET(channel));

              /* Call the upper-half driver */

              if (priv->dev.ad_callback)
                {
                  priv->dev.ad_callback->au_receive(&priv->dev,
                                                     channel, data);
                }
            }
        }
    }

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_adc_initialize
 *
 * Description:
 *   Initialize the ADC driver
 *
 ****************************************************************************/

int ra8p_adc_initialize(void)
{
  struct ra8p_adc_dev_s *priv = &g_adc0_priv;
  int ret;

  /* Initialize private data structure */

  memset(priv, 0, sizeof(struct ra8p_adc_dev_s));
  priv->base = RA8P_ADC0_BASE;
  priv->irq = RA8P_IRQ_ADC0;
  priv->resolution = RA8P_ADC_RESOLUTION_12BIT;
  priv->channels = 16;  /* ADC0 has 16 channels */

  nxmutex_init(&priv->lock);

  /* Initialize ADC ops */

  priv->dev.ad_ops = &g_adc_ops;

  /* Attach interrupt handler */

  ret = irq_attach(priv->irq, ra8p_adc_interrupt, priv);
  if (ret < 0)
    {
      goto errout_with_mutex;
    }

  /* Enable ADC interrupt */

  up_enable_irq(priv->irq);

  /* Register the ADC driver */

  ret = adc_register("/dev/adc0", &priv->dev);
  if (ret < 0)
    {
      goto errout_with_irq;
    }

  return OK;

errout_with_irq:
  up_disable_irq(priv->irq);
  irq_detach(priv->irq);
errout_with_mutex:
  nxmutex_destroy(&priv->lock);
  return ret;
}

/****************************************************************************
 * Name: ra8p_adc_read_channel
 *
 * Description:
 *   Read a single ADC channel
 *
 ****************************************************************************/

int ra8p_adc_read_channel(uint8_t channel, uint16_t *value)
{
  struct ra8p_adc_dev_s *priv = &g_adc0_priv;
  uint32_t timeout;
  uint32_t status;

  if (!priv || !value || channel >= priv->channels)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Select the channel */

  ra8p_adc_putreg32(priv->base, RA8P_ADC_ADANSA_OFFSET, (1 << channel));

  /* Start conversion */

  ra8p_adc_modifyreg32(priv->base, RA8P_ADC_ADCR_OFFSET,
                       0, RA8P_ADC_ADCR_ADST);

  /* Wait for conversion to complete */

  timeout = 10000;
  do
    {
      status = ra8p_adc_getreg32(priv->base, RA8P_ADC_ADISR_OFFSET);
      if (status & RA8P_ADC_ADISR_ADIF)
        {
          break;
        }

      up_udelay(1);
    }
  while (--timeout > 0);

  if (timeout == 0)
    {
      nxmutex_unlock(&priv->lock);
      return -ETIMEDOUT;
    }

  /* Clear interrupt flag */

  ra8p_adc_putreg32(priv->base, RA8P_ADC_ADISR_OFFSET,
                    RA8P_ADC_ADISR_ADIF);

  /* Read the result */

  *value = ra8p_adc_getreg16(priv->base, RA8P_ADC_ADDR_OFFSET(channel));

  nxmutex_unlock(&priv->lock);
  return OK;
}

#endif /* CONFIG_RA8P_ADC0 */