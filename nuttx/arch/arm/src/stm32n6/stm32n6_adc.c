/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_adc.c
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
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <errno.h>
#include <assert.h>
#include <debug.h>

#include <arch/board/board.h>
#include <nuttx/irq.h>
#include <nuttx/analog/adc.h>
#include <nuttx/analog/ioctl.h>

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_memorymap.h"
#include "hardware/stm32n6_rcc.h"
#include "stm32n6_adc.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define ADC_TIMEOUT_US              100000

/* Helper macros */

#define ADC_GETREG(base, offset)    getreg32((base) + (offset))
#define ADC_PUTREG(base, offset, value) putreg32((value), (base) + (offset))

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct stm32n6_adc_priv_s
{
  struct adc_dev_s dev;           /* Instance of the upper-half driver */
  uint32_t base;                  /* Base address of registers */
  uint8_t irq;                    /* Interrupt number */
  uint8_t nchannels;              /* Number of channels */
  uint8_t current;                /* Current channel being sampled */
  uint8_t samples_left;           /* Number of samples remaining in scan */
  bool enabled;                   /* Is the ADC enabled? */
  bool scanning;                  /* Is scanning in progress? */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int adc_bind(struct adc_dev_s *dev, const struct adc_callback_s *callback);
static void adc_reset(struct adc_dev_s *dev);
static int adc_setup(struct adc_dev_s *dev);
static void adc_shutdown(struct adc_dev_s *dev);
static void adc_rxint(struct adc_dev_s *dev, bool enable);
static int adc_ioctl(struct adc_dev_s *dev, int cmd, unsigned long arg);

static int stm32n6_adc_interrupt(int irq, void *context, void *arg);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static const struct adc_ops_s g_adcops =
{
  .ao_bind        = adc_bind,
  .ao_reset       = adc_reset,
  .ao_setup       = adc_setup,
  .ao_shutdown    = adc_shutdown,
  .ao_rxint       = adc_rxint,
  .ao_ioctl       = adc_ioctl
};

static struct stm32n6_adc_priv_s g_adc1_priv =
{
  .dev =
  {
    .ad_ops     = &g_adcops,
  },
  .base         = STM32_ADC1_BASE,
  .irq          = STM32_IRQ_ADC1,
};

static struct stm32n6_adc_priv_s g_adc2_priv =
{
  .dev =
  {
    .ad_ops     = &g_adcops,
  },
  .base         = STM32_ADC2_BASE,
  .irq          = STM32_IRQ_ADC2,
};

/* ADC calibration values */
static uint32_t g_cal_fact = 0;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static int adc_wait_for_ready(struct stm32n6_adc_priv_s *priv)
{
  uint32_t timeout = ADC_TIMEOUT_US;

  while ((ADC_GETREG(priv->base, STM32_ADC_ISR_OFFSET) & ADC_ISR_ADRDY) == 0)
    {
      if (timeout-- == 0)
        {
          return -ETIMEDOUT;
        }
      up_udelay(1);
    }

  return OK;
}

static int adc_enable_clock(uint32_t base)
{
  uint32_t regval;

  /* Enable clock for the appropriate ADC */
  regval = getreg32(STM32_RCC_AHB2ENR);
  if (base == STM32_ADC1_BASE)
    {
      regval |= RCC_AHB2ENR_ADC1EN;
    }
  else if (base == STM32_ADC2_BASE)
    {
      regval |= RCC_AHB2ENR_ADC2EN;
    }
  putreg32(regval, STM32_RCC_AHB2ENR);

  return OK;
}

static void adc_configure_sampling_times(struct stm32n6_adc_priv_s *priv)
{
  uint32_t regval;
  int i;

  /* Configure sampling times for all channels in the sequence */
  regval = 0;
  for (i = 0; i < priv->nchannels; i++)
    {
      uint32_t channel = priv->dev.ad_chanlist[i];
      uint32_t smpr_reg;
      uint32_t smpr_shift;

      if (channel < 10)
        {
          smpr_reg = STM32_ADC_SMPR1_OFFSET;
          smpr_shift = channel * 3;
        }
      else
        {
          smpr_reg = STM32_ADC_SMPR2_OFFSET;
          smpr_shift = (channel - 10) * 3;
        }

      regval = ADC_GETREG(priv->base, smpr_reg);
      regval &= ~(7 << smpr_shift);
      regval |= (ADC_SMPR_SMP_12_5CYCLES << smpr_shift);
      ADC_PUTREG(priv->base, smpr_reg, regval);
    }
}

static int adc_configure(struct stm32n6_adc_priv_s *priv)
{
  uint32_t regval;

  /* Enable ADC voltage regulator */
  regval = ADC_GETREG(priv->base, STM32_ADC_CR_OFFSET);
  regval |= ADC_CR_ADVREGEN;
  ADC_PUTREG(priv->base, STM32_ADC_CR_OFFSET, regval);

  /* Wait for voltage regulator to stabilize */
  up_udelay(10);

  /* Configure ADC resolution and other settings */
  regval = ADC_CFGR1_RES_12BIT;    /* 12-bit resolution */
  regval |= ADC_CFGR1_CONT;        /* Continuous mode */
  regval |= ADC_CFGR1_OVRMOD;      /* Overrun mode */

  /* If multiple channels, configure for scan mode */
  if (priv->nchannels > 1)
    {
      regval |= ADC_CFGR1_SCANDIR; /* Scan forward direction */
    }

  ADC_PUTREG(priv->base, STM32_ADC_CFGR1_OFFSET, regval);

  /* Configure sampling times */
  adc_configure_sampling_times(priv);

  /* Configure channel sequence */
  regval = 0;
  for (int i = 0; i < priv->nchannels; i++)
    {
      regval |= ADC_CHSELR_CHSEL(priv->dev.ad_chanlist[i]);
    }
  ADC_PUTREG(priv->base, STM32_ADC_CHSELR_OFFSET, regval);

  /* Configure interrupt enable */
  regval = ADC_IER_EOCIE | ADC_IER_EOSIE | ADC_IER_OVRIE;
  ADC_PUTREG(priv->base, STM32_ADC_IER_OFFSET, regval);

  return OK;
}

static int adc_bind(struct adc_dev_s *dev, const struct adc_callback_s *callback)
{
  /* Just return OK for now */
  return OK;
}

static void adc_reset(struct adc_dev_s *dev)
{
  struct stm32n6_adc_priv_s *priv = (struct stm32n6_adc_priv_s *)dev;
  uint32_t regval;

  /* Disable ADC */
  regval = ADC_GETREG(priv->base, STM32_ADC_CR_OFFSET);
  regval |= ADC_CR_ADDIS;
  ADC_PUTREG(priv->base, STM32_ADC_CR_OFFSET, regval);

  /* Wait for disable */
  while ((ADC_GETREG(priv->base, STM32_ADC_CR_OFFSET) & ADC_CR_ADEN) != 0);

  /* Clear all interrupts */
  ADC_PUTREG(priv->base, STM32_ADC_ISR_OFFSET, 0x0FFF);

  /* Re-enable ADC */
  regval = ADC_GETREG(priv->base, STM32_ADC_CR_OFFSET);
  regval |= ADC_CR_ADEN;
  ADC_PUTREG(priv->base, STM32_ADC_CR_OFFSET, regval);

  /* Wait for ready */
  adc_wait_for_ready(priv);
}

static int adc_setup(struct adc_dev_s *dev)
{
  struct stm32n6_adc_priv_s *priv = (struct stm32n6_adc_priv_s *)dev;
  uint32_t regval;
  int ret;

  /* Enable clock */
  ret = adc_enable_clock(priv->base);
  if (ret < 0)
    {
      return ret;
    }

  /* Configure the ADC */
  ret = adc_configure(priv);
  if (ret < 0)
    {
      return ret;
    }

  /* Enable ADC */
  regval = ADC_GETREG(priv->base, STM32_ADC_CR_OFFSET);
  regval |= ADC_CR_ADEN;
  ADC_PUTREG(priv->base, STM32_ADC_CR_OFFSET, regval);

  /* Wait for ready */
  ret = adc_wait_for_ready(priv);
  if (ret < 0)
    {
      return ret;
    }

  priv->enabled = true;

  /* Attach interrupt handler */
  ret = irq_attach(priv->irq, stm32n6_adc_interrupt, priv);
  if (ret < 0)
    {
      return ret;
    }

  /* Enable the interrupt */
  up_enable_irq(priv->irq);

  return OK;
}

static void adc_shutdown(struct adc_dev_s *dev)
{
  struct stm32n6_adc_priv_s *priv = (struct stm32n6_adc_priv_s *)dev;
  uint32_t regval;

  /* Disable interrupt */
  up_disable_irq(priv->irq);
  irq_detach(priv->irq);

  /* Stop any ongoing conversions */
  regval = ADC_GETREG(priv->base, STM32_ADC_CR_OFFSET);
  if (regval & ADC_CR_ADSTART)
    {
      regval |= ADC_CR_ADSTP;
      ADC_PUTREG(priv->base, STM32_ADC_CR_OFFSET, regval);
      
      /* Wait for stop */
      while ((ADC_GETREG(priv->base, STM32_ADC_CR_OFFSET) & ADC_CR_ADSTP) != 0);
    }

  /* Disable ADC */
  regval = ADC_GETREG(priv->base, STM32_ADC_CR_OFFSET);
  regval |= ADC_CR_ADDIS;
  ADC_PUTREG(priv->base, STM32_ADC_CR_OFFSET, regval);

  /* Wait for disable */
  while ((ADC_GETREG(priv->base, STM32_ADC_CR_OFFSET) & ADC_CR_ADEN) != 0);

  priv->enabled = false;
}

static void adc_rxint(struct adc_dev_s *dev, bool enable)
{
  struct stm32n6_adc_priv_s *priv = (struct stm32n6_adc_priv_s *)dev;
  uint32_t regval;

  regval = ADC_GETREG(priv->base, STM32_ADC_IER_OFFSET);
  if (enable)
    {
      regval |= ADC_IER_EOCIE | ADC_IER_EOSIE;
    }
  else
    {
      regval &= ~(ADC_IER_EOCIE | ADC_IER_EOSIE);
    }
  ADC_PUTREG(priv->base, STM32_ADC_IER_OFFSET, regval);
}

static int adc_ioctl(struct adc_dev_s *dev, int cmd, unsigned long arg)
{
  struct stm32n6_adc_priv_s *priv = (struct stm32n6_adc_priv_s *)dev;

  switch (cmd)
    {
      case ANIOC_TRIGGER:
        {
          uint32_t regval;

          /* Check if ADC is enabled */
          if (!priv->enabled)
            {
              return -ENODEV;
            }

          /* Start conversion */
          regval = ADC_GETREG(priv->base, STM32_ADC_CR_OFFSET);
          regval |= ADC_CR_ADSTART;
          ADC_PUTREG(priv->base, STM32_ADC_CR_OFFSET, regval);

          priv->scanning = true;
          priv->samples_left = priv->nchannels;
          priv->current = 0;

          return OK;
        }

      default:
        return -ENOTTY;
    }
}

static int stm32n6_adc_interrupt(int irq, void *context, void *arg)
{
  struct stm32n6_adc_priv_s *priv = (struct stm32n6_adc_priv_s *)arg;
  uint32_t isr;
  uint32_t data;

  isr = ADC_GETREG(priv->base, STM32_ADC_ISR_OFFSET);

  /* Clear interrupts */
  ADC_PUTREG(priv->base, STM32_ADC_ISR_OFFSET, isr);

  /* Handle End of Conversion */
  if ((isr & ADC_ISR_EOC) != 0)
    {
      /* Read converted data */
      data = ADC_GETREG(priv->base, STM32_ADC_DR_OFFSET) & 0xFFF;

      /* Call upper half callback */
      if (priv->dev.ad_cb != NULL)
        {
          priv->dev.ad_cb(&priv->dev, priv->current, data);
        }

      priv->current++;
      priv->samples_left--;

      if (priv->samples_left == 0)
        {
          /* All samples in sequence completed */
          priv->scanning = false;
          priv->current = 0;
        }
    }

  /* Handle End of Sequence */
  if ((isr & ADC_ISR_EOS) != 0)
    {
      /* Sequence completed */
      priv->scanning = false;
      priv->samples_left = 0;
      priv->current = 0;
    }

  /* Handle overrun */
  if ((isr & ADC_ISR_OVR) != 0)
    {
      /* TODO: Handle overrun condition */
      adc_reset(&priv->dev);
    }

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int stm32n6_adc_initialize(void)
{
  struct stm32n6_adc_priv_s *priv;
  int ret;

  /* Initialize ADC1 */
  priv = &g_adc1_priv;
  priv->nchannels = 0;
  priv->current = 0;
  priv->samples_left = 0;
  priv->enabled = false;
  priv->scanning = false;

  ret = adc_register("/dev/adc0", &priv->dev);
  if (ret < 0)
    {
      return ret;
    }

  /* Initialize ADC2 if enabled */
#ifdef CONFIG_STM32N6_ADC2
  priv = &g_adc2_priv;
  priv->nchannels = 0;
  priv->current = 0;
  priv->samples_left = 0;
  priv->enabled = false;
  priv->scanning = false;

  ret = adc_register("/dev/adc1", &priv->dev);
  if (ret < 0)
    {
      return ret;
    }
#endif

  return OK;
}