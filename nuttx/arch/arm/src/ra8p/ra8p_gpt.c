/****************************************************************************
 * arch/arm/src/ra8p/ra8p_gpt.c
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
#include <string.h>
#include <errno.h>
#include <debug.h>

#include "chip.h"
#include "arm_internal.h"
#include "hardware/ra8p_gpt.h"

#if defined(CONFIG_RA8P_GPT_PWM0) || defined(CONFIG_RA8P_GPT_PWM1) || \
    defined(CONFIG_RA8P_GPT_PWM2) || defined(CONFIG_RA8P_GPT_PWM3) || \
    defined(CONFIG_RA8P_GPT_PWM4) || defined(CONFIG_RA8P_GPT_PWM5) || \
    defined(CONFIG_RA8P_GPT_PWM6) || defined(CONFIG_RA8P_GPT_PWM7) || \
    defined(CONFIG_RA8P_GPT_PWM8) || defined(CONFIG_RA8P_GPT_PWM9) || \
    defined(CONFIG_RA8P_GPT_PWM10) || defined(CONFIG_RA8P_GPT_PWM11) || \
    defined(CONFIG_RA8P_GPT_PWM12) || defined(CONFIG_RA8P_GPT_PWM13)

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define RA8P_GPT0_BASE              RA8P_GPT0_BASE
#define RA8P_GPT1_BASE              RA8P_GPT1_BASE
#define RA8P_GPT2_BASE              RA8P_GPT2_BASE
#define RA8P_GPT3_BASE              RA8P_GPT3_BASE
#define RA8P_GPT4_BASE              RA8P_GPT4_BASE
#define RA8P_GPT5_BASE              RA8P_GPT5_BASE
#define RA8P_GPT6_BASE              RA8P_GPT6_BASE
#define RA8P_GPT7_BASE              RA8P_GPT7_BASE
#define RA8P_GPT8_BASE              RA8P_GPT8_BASE
#define RA8P_GPT9_BASE              RA8P_GPT9_BASE
#define RA8P_GPT10_BASE             RA8P_GPT10_BASE
#define RA8P_GPT11_BASE             RA8P_GPT11_BASE
#define RA8P_GPT12_BASE             RA8P_GPT12_BASE
#define RA8P_GPT13_BASE             RA8P_GPT13_BASE

/* GPT timeout in milliseconds */
#define RA8P_GPT_TIMEOUT_MS         100

/* PWM period for standard applications */
#define RA8P_GPT_DEFAULT_PERIOD_US  1000  /* 1ms default period */

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* RA8P1 GPT driver state structure */

struct ra8p_gpt_priv_s
{
  uint32_t base;                  /* Base address of GPT registers */
  uint8_t channel;                /* GPT channel (0-13) */
  bool initialized;               /* Initialization flag */
  bool enabled;                   /* Enable flag */
  uint32_t period_us;             /* PWM period in microseconds */
  uint32_t pclk;                  /* PCLK frequency for timing calculations */
  uint8_t pin;                    /* Associated pin for output */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int gpt_set_frequency(struct ra8p_gpt_priv_s *priv, uint32_t freq);
static int gpt_set_duty(struct ra8p_gpt_priv_s *priv, uint8_t percent);
static void gpt_putreg32(struct ra8p_gpt_priv_s *priv, uint32_t offset, uint32_t value);
static uint32_t gpt_getreg32(struct ra8p_gpt_priv_s *priv, uint32_t offset);
static void gpt_putreg16(struct ra8p_gpt_priv_s *priv, uint32_t offset, uint16_t value);
static uint16_t gpt_getreg16(struct ra8p_gpt_priv_s *priv, uint32_t offset);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_gpt_priv_s g_gpt[14];  /* 14 GPT channels */

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: gpt_putreg32
 ****************************************************************************/

static inline void gpt_putreg32(struct ra8p_gpt_priv_s *priv, uint32_t offset, uint32_t value)
{
  putreg32(value, priv->base + offset);
}

/****************************************************************************
 * Name: gpt_getreg32
 ****************************************************************************/

static inline uint32_t gpt_getreg32(struct ra8p_gpt_priv_s *priv, uint32_t offset)
{
  return getreg32(priv->base + offset);
}

/****************************************************************************
 * Name: gpt_putreg16
 ****************************************************************************/

static inline void gpt_putreg16(struct ra8p_gpt_priv_s *priv, uint32_t offset, uint16_t value)
{
  putreg16(value, priv->base + offset);
}

/****************************************************************************
 * Name: gpt_getreg16
 ****************************************************************************/

static inline uint16_t gpt_getreg16(struct ra8p_gpt_priv_s *priv, uint32_t offset)
{
  return getreg16(priv->base + offset);
}

/****************************************************************************
 * Name: gpt_calc_divider
 ****************************************************************************/

static int gpt_calc_divider(struct ra8p_gpt_priv_s *priv, uint32_t period_us, 
                           uint8_t *divider, uint16_t *cycles)
{
  uint32_t pclk = priv->pclk;
  uint32_t cycles_calc;
  uint8_t div;

  /* Find appropriate divider and cycle count */
  for (div = 0; div <= 15; div++)
    {
      uint32_t clk_freq = pclk / (1 << div);  /* Divide by 1, 2, 4, 8, ..., 32768 */
      cycles_calc = clk_freq * period_us / 1000000;  /* cycles in period */

      if (cycles_calc <= 65535)  /* 16-bit counter max */
        {
          *divider = div;
          *cycles = (uint16_t)cycles_calc;
          return OK;
        }
    }

  return -ERANGE;  /* Period too large for any divider */
}

/****************************************************************************
 * Name: gpt_set_period
 ****************************************************************************/

static int gpt_set_period(struct ra8p_gpt_priv_s *priv, uint32_t period_us)
{
  uint8_t divider;
  uint16_t cycles;
  uint16_t regval;
  int ret;

  /* Calculate appropriate divider and cycle count */
  ret = gpt_calc_divider(priv, period_us, &divider, &cycles);
  if (ret != OK)
    {
      return ret;
    }

  /* Configure clock divider */
  regval = gpt_getreg16(priv, RA8P_GPT_GTCR_OFFSET);
  regval &= ~RA8P_GPT_GTCR_PCLK_DIVIDER_MASK;
  regval |= (divider << RA8P_GPT_GTCR_PCLK_DIVIDER_SHIFT);
  gpt_putreg16(priv, RA8P_GPT_GTCR_OFFSET, regval);

  /* Set period in GTPR register */
  gpt_putreg16(priv, RA8P_GPT_GTPR_OFFSET, cycles);

  priv->period_us = period_us;

  return OK;
}

/****************************************************************************
 * Name: gpt_set_duty
 ****************************************************************************/

static int gpt_set_duty(struct ra8p_gpt_priv_s *priv, uint8_t percent)
{
  uint16_t compare_val;
  uint16_t period;

  if (percent > 100)
    {
      return -EINVAL;
    }

  period = gpt_getreg16(priv, RA8P_GPT_GTPR_OFFSET);
  compare_val = (period * percent) / 100;

  /* Set duty cycle in GTCCR register */
  gpt_putreg16(priv, RA8P_GPT_GTCCRA_OFFSET, compare_val);

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_gpt_pwm_initialize
 *
 * Description:
 *   Initialize the GPT PWM channel.
 *
 * Input Parameters:
 *   channel - GPT channel (0-13)
 *   period_us - PWM period in microseconds
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpt_pwm_initialize(uint8_t channel, uint32_t period_us)
{
  struct ra8p_gpt_priv_s *priv;
  uint16_t regval;
  int ret;

  if (channel >= 14)
    {
      return -EINVAL;
    }

  priv = &g_gpt[channel];
  priv->channel = channel;

  /* Set base address based on channel */
  switch (channel)
    {
      case 0:  priv->base = RA8P_GPT0_BASE; break;
      case 1:  priv->base = RA8P_GPT1_BASE; break;
      case 2:  priv->base = RA8P_GPT2_BASE; break;
      case 3:  priv->base = RA8P_GPT3_BASE; break;
      case 4:  priv->base = RA8P_GPT4_BASE; break;
      case 5:  priv->base = RA8P_GPT5_BASE; break;
      case 6:  priv->base = RA8P_GPT6_BASE; break;
      case 7:  priv->base = RA8P_GPT7_BASE; break;
      case 8:  priv->base = RA8P_GPT8_BASE; break;
      case 9:  priv->base = RA8P_GPT9_BASE; break;
      case 10: priv->base = RA8P_GPT10_BASE; break;
      case 11: priv->base = RA8P_GPT11_BASE; break;
      case 12: priv->base = RA8P_GPT12_BASE; break;
      case 13: priv->base = RA8P_GPT13_BASE; break;
      default: return -EINVAL;
    }

  priv->period_us = period_us ? period_us : RA8P_GPT_DEFAULT_PERIOD_US;
  priv->pclk = 62500000;  /* Use appropriate PCLK frequency */
  priv->initialized = false;
  priv->enabled = false;

  /* Reset GPT module */
  regval = gpt_getreg16(priv, RA8P_GPT_GTCR_OFFSET);
  regval |= RA8P_GPT_GTCR_GPTRESET;  /* Reset bit */
  gpt_putreg16(priv, RA8P_GPT_GTCR_OFFSET, regval);

  /* Wait for reset */
  volatile int timeout = 1000;
  while (gpt_getreg16(priv, RA8P_GPT_GTCR_OFFSET) & RA8P_GPT_GTCR_GPTRESET && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Set operating mode - Continuous count mode */
  regval = gpt_getreg16(priv, RA8P_GPT_GTCR_OFFSET);
  regval &= ~RA8P_GPT_GTCR_MODE_MASK;  /* Clear mode bits */
  regval |= (RA8P_GPT_MODE_CONTINUOUS << RA8P_GPT_GTCR_MODE_SHIFT);
  gpt_putreg16(priv, RA8P_GPT_GTCR_OFFSET, regval);

  /* Configure timer for PWM operation */
  regval = gpt_getreg16(priv, RA8P_GPT_GTIOR_OFFSET);
  regval &= ~RA8P_GPT_GTIOR_GTIOA_MASK;  /* Clear GTIOA bits */
  regval |= (RA8P_GPT_GTIOA_PWM_HIGH << RA8P_GPT_GTIOR_GTIOA_SHIFT);  /* Set for PWM */
  gpt_putreg16(priv, RA8P_GPT_GTIOR_OFFSET, regval);

  /* Set period */
  ret = gpt_set_period(priv, priv->period_us);
  if (ret != OK)
    {
      return ret;
    }

  /* Set initial duty cycle to 0% */
  gpt_putreg16(priv, RA8P_GPT_GTCCRA_OFFSET, 0);

  /* Configure compare match control register */
  regval = gpt_getreg16(priv, RA8P_GPT_GTCNTCMPCLR_OFFSET);
  regval |= (1 << 0);  /* Clear compare match A flag */
  gpt_putreg16(priv, RA8P_GPT_GTCNTCMPCLR_OFFSET, regval);

  priv->initialized = true;

  gptinfo("GPT%d PWM initialized at 0x%08x, period=%u us\n", 
          channel, priv->base, period_us);
  return OK;
}

/****************************************************************************
 * Name: ra8p_gpt_pwm_enable
 *
 * Description:
 *   Enable the GPT PWM channel.
 *
 * Input Parameters:
 *   channel - GPT channel (0-13)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpt_pwm_enable(uint8_t channel)
{
  struct ra8p_gpt_priv_s *priv;
  uint16_t regval;

  if (channel >= 14 || !g_gpt[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_gpt[channel];

  /* Enable GPT counter */
  regval = gpt_getreg16(priv, RA8P_GPT_GTCR_OFFSET);
  regval |= RA8P_GPT_GTCR_COUNTER_ENABLE;
  gpt_putreg16(priv, RA8P_GPT_GTCR_OFFSET, regval);

  priv->enabled = true;

  gptinfo("GPT%d PWM enabled\n", channel);
  return OK;
}

/****************************************************************************
 * Name: ra8p_gpt_pwm_disable
 *
 * Description:
 *   Disable the GPT PWM channel.
 *
 * Input Parameters:
 *   channel - GPT channel (0-13)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpt_pwm_disable(uint8_t channel)
{
  struct ra8p_gpt_priv_s *priv;
  uint16_t regval;

  if (channel >= 14 || !g_gpt[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_gpt[channel];

  /* Disable GPT counter */
  regval = gpt_getreg16(priv, RA8P_GPT_GTCR_OFFSET);
  regval &= ~RA8P_GPT_GTCR_COUNTER_ENABLE;
  gpt_putreg16(priv, RA8P_GPT_GTCR_OFFSET, regval);

  priv->enabled = false;

  gptinfo("GPT%d PWM disabled\n", channel);
  return OK;
}

/****************************************************************************
 * Name: ra8p_gpt_pwm_set_duty
 *
 * Description:
 *   Set the PWM duty cycle.
 *
 * Input Parameters:
 *   channel - GPT channel (0-13)
 *   duty_percent - Duty cycle percentage (0-100)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpt_pwm_set_duty(uint8_t channel, uint8_t duty_percent)
{
  struct ra8p_gpt_priv_s *priv;
  int ret;

  if (channel >= 14 || duty_percent > 100)
    {
      return -EINVAL;
    }

  if (!g_gpt[channel].initialized)
    {
      return -ENODEV;
    }

  priv = &g_gpt[channel];

  ret = gpt_set_duty(priv, duty_percent);
  if (ret != OK)
    {
      return ret;
    }

  gptinfo("GPT%d PWM duty set to %u%%\n", channel, duty_percent);
  return OK;
}

/****************************************************************************
 * Name: ra8p_gpt_pwm_set_frequency
 *
 * Description:
 *   Set the PWM frequency (recalculates period).
 *
 * Input Parameters:
 *   channel - GPT channel (0-13)
 *   frequency - PWM frequency in Hz
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpt_pwm_set_frequency(uint8_t channel, uint32_t frequency)
{
  struct ra8p_gpt_priv_s *priv;
  uint32_t period_us;
  int ret;

  if (channel >= 14 || frequency == 0)
    {
      return -EINVAL;
    }

  if (!g_gpt[channel].initialized)
    {
      return -ENODEV;
    }

  priv = &g_gpt[channel];

  /* Calculate period from frequency: period(us) = 1000000 / frequency */
  period_us = 1000000 / frequency;

  /* Set new period */
  ret = gpt_set_period(priv, period_us);
  if (ret != OK)
    {
      return ret;
    }

  /* Update stored period */
  priv->period_us = period_us;

  gptinfo("GPT%d PWM frequency set to %u Hz (period=%u us)\n", 
          channel, frequency, period_us);
  return OK;
}

/****************************************************************************
 * Name: ra8p_gpt_pwm_get_frequency
 *
 * Description:
 *   Get the current PWM frequency.
 *
 * Input Parameters:
 *   channel - GPT channel (0-13)
 *
 * Returned Value:
 *   PWM frequency in Hz, or 0 if error
 *
 ****************************************************************************/

uint32_t ra8p_gpt_pwm_get_frequency(uint8_t channel)
{
  struct ra8p_gpt_priv_s *priv;

  if (channel >= 14 || !g_gpt[channel].initialized)
    {
      return 0;
    }

  priv = &g_gpt[channel];

  if (priv->period_us == 0)
    {
      return 0;  /* Invalid period */
    }

  return 1000000 / priv->period_us;  /* Frequency in Hz */
}

#endif /* CONFIG_RA8P_GPT_PWM* */