/****************************************************************************
 * arch/arm/src/ra8p/ra8p_wdt.c
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
#include <errno.h>
#include <debug.h>

#include "chip.h"
#include "arm_internal.h"
#include "hardware/ra8p_wdt.h"
#include "hardware/ra8p_memorymap.h"

#ifdef CONFIG_RA8P_WDT

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define RA8P_WDT_BASE                    RA8P_WDT_BASE

/* WDT timeout in milliseconds */
#define RA8P_WDT_TIMEOUT_MS              1000

/* WDT unlock sequence */
#define RA8P_WDT_UNLOCK_SEQ              0xA5

/* WDT refresh sequences */
#define RA8P_WDT_REFRESH_SEQ1            0xAC
#define RA8P_WDT_REFRESH_SEQ2            0x53

/* WDT overflow clear sequence */
#define RA8P_WDT_OVERFLOW_CLR_SEQ        0xA5

/* Default timeout period (2048 cycles) */
#define RA8P_WDT_DEFAULT_TIMEOUT_PERIOD  2048

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* RA8P1 WDT driver state structure */

struct ra8p_wdt_priv_s
{
  uint32_t base;                              /* Base address of WDT registers */
  bool initialized;                           /* Initialization flag */
  bool enabled;                               /* Enable flag */
  bool reset_enabled;                         /* Reset on timeout enabled */
  bool interrupt_only;                        /* Interrupt only mode */
  uint32_t timeout_ms;                        /* Current timeout period in ms */
  uint8_t clock_divider;                      /* Clock divider value */
  uint8_t timeout_setting;                    /* Timeout setting */
  bool overflow_occurred;                     /* Overflow flag */
  uint32_t overflow_count;                    /* Overflow counter */
  uint32_t pclk_frequency;                    /* PCLK frequency for calculations */
  struct ra8p_wdt_callback_s callback;       /* Overflow callback */
  bool low_power_mode;                        /* Low power mode enabled */
  bool clock_source_subclk;                   /* Clock source is sub-clock */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int wdt_wait_ready(struct ra8p_wdt_priv_s *priv);
static int wdt_calc_timeout(struct ra8p_wdt_priv_s *priv, uint32_t timeout_ms,
                           uint8_t *divider, uint8_t *setting);
static void wdt_putreg32(struct ra8p_wdt_priv_s *priv, uint32_t offset, uint32_t value);
static uint32_t wdt_getreg32(struct ra8p_wdt_priv_s *priv, uint32_t offset);
static void wdt_putreg16(struct ra8p_wdt_priv_s *priv, uint32_t offset, uint16_t value);
static uint16_t wdt_getreg16(struct ra8p_wdt_priv_s *priv, uint32_t offset);
static void wdt_putreg8(struct ra8p_wdt_priv_s *priv, uint32_t offset, uint8_t value);
static uint8_t wdt_getreg8(struct ra8p_wdt_priv_s *priv, uint32_t offset);
static int wdt_set_timeout(struct ra8p_wdt_priv_s *priv, uint32_t timeout_ms);
static int wdt_unlock_registers(struct ra8p_wdt_priv_s *priv);
static int wdt_calculate_timeout_params(uint32_t timeout_ms, uint32_t pclk_freq,
                                       uint8_t *divider, uint8_t *setting);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_wdt_priv_s g_wdt;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: wdt_putreg32
 ****************************************************************************/

static inline void wdt_putreg32(struct ra8p_wdt_priv_s *priv, uint32_t offset, uint32_t value)
{
  putreg32(value, priv->base + offset);
}

/****************************************************************************
 * Name: wdt_getreg32
 ****************************************************************************/

static inline uint32_t wdt_getreg32(struct ra8p_wdt_priv_s *priv, uint32_t offset)
{
  return getreg32(priv->base + offset);
}

/****************************************************************************
 * Name: wdt_putreg16
 ****************************************************************************/

static inline void wdt_putreg16(struct ra8p_wdt_priv_s *priv, uint32_t offset, uint16_t value)
{
  putreg16(value, priv->base + offset);
}

/****************************************************************************
 * Name: wdt_getreg16
 ****************************************************************************/

static inline uint16_t wdt_getreg16(struct ra8p_wdt_priv_s *priv, uint32_t offset)
{
  return getreg16(priv->base + offset);
}

/****************************************************************************
 * Name: wdt_putreg8
 ****************************************************************************/

static inline void wdt_putreg8(struct ra8p_wdt_priv_s *priv, uint32_t offset, uint8_t value)
{
  putreg8(value, priv->base + offset);
}

/****************************************************************************
 * Name: wdt_getreg8
 ****************************************************************************/

static inline uint8_t wdt_getreg8(struct ra8p_wdt_priv_s *priv, uint32_t offset)
{
  return getreg8(priv->base + offset);
}

/****************************************************************************
 * Name: wdt_unlock_registers
 ****************************************************************************/

static int wdt_unlock_registers(struct ra8p_wdt_priv_s *priv)
{
  /* Write unlock sequence to WDTULOCK register */
  wdt_putreg8(priv, RA8P_WDT_WDTULOCK_OFFSET, RA8P_WDT_UNLOCK_SEQ);
  
  return OK;
}

/****************************************************************************
 * Name: wdt_calculate_timeout_params
 ****************************************************************************/

static int wdt_calculate_timeout_params(uint32_t timeout_ms, uint32_t pclk_freq,
                                       uint8_t *divider, uint8_t *setting)
{
  uint32_t total_cycles;
  uint32_t divisor;
  uint8_t calc_divider = 0;
  uint8_t calc_setting = 0;
  uint32_t calc_timeout_ms;
  uint32_t error;
  uint32_t best_error = UINT32_MAX;
  uint8_t best_divider = 0;
  uint8_t best_setting = 0;
  uint8_t setting_val;

  if (timeout_ms == 0)
    {
      return -EINVAL;
    }

  /* Calculate total cycles needed for timeout */
  total_cycles = (pclk_freq / 1000) * timeout_ms;  /* cycles = (cycles/sec) * (ms/1000) */

  /* For Renesas RA8P WDT, we have:
   * - Clock Select (CKS) with dividers: 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768
   * - Timeout Period Select (TOPS): 128, 512, 1024, 2048 cycles
   */
  for (divisor = 1; divisor <= RA8P_WDT_MAX_CLK_DIVIDER; divisor *= 2)
    {
      for (setting_val = 0; setting_val < 4; setting_val++)
        {
          uint32_t timeout_periods[] = {128, 512, 1024, 2048};
          uint32_t cycles = timeout_periods[setting_val];
          uint32_t calc_timeout = (cycles * divisor * 1000) / pclk_freq;

          error = (calc_timeout > timeout_ms) ? (calc_timeout - timeout_ms) : (timeout_ms - calc_timeout);

          if (error < best_error)
            {
              best_error = error;
              best_divider = 0;
              // Calculate appropriate clock divider index
              for (int i = 0; i < 16; i++)
                {
                  if ((1 << i) == divisor)
                    {
                      best_divider = i;
                      break;
                    }
                }
              best_setting = setting_val;
              
              if (error == 0)  /* Perfect match found */
                {
                  break;
                }
            }
        }
    }

  *divider = best_divider;
  *setting = best_setting;

  return OK;
}

/****************************************************************************
 * Name: wdt_set_timeout
 ****************************************************************************/

static int wdt_set_timeout(struct ra8p_wdt_priv_s *priv, uint32_t timeout_ms)
{
  uint8_t divider;
  uint8_t setting;
  uint32_t regval;
  int ret;

  /* Calculate appropriate divider and timeout setting */
  ret = wdt_calculate_timeout_params(timeout_ms, priv->pclk_frequency, &divider, &setting);
  if (ret != OK)
    {
      return ret;
    }

  /* Unlock registers first */
  ret = wdt_unlock_registers(priv);
  if (ret != OK)
    {
      return ret;
    }

  /* Read current control register */
  regval = wdt_getreg16(priv, RA8P_WDT_WDTCR_OFFSET);

  /* Clear timeout period and clock select bits */
  regval &= ~(RA8P_WDT_WDTCR_TOPS_MASK | RA8P_WDT_WDTCR_CKS_MASK);

  /* Set new timeout period and clock divider */
  regval |= (setting << RA8P_WDT_WDTCR_TOPS_SHIFT) |
            (divider << RA8P_WDT_WDTCR_CKS_SHIFT);

  wdt_putreg16(priv, RA8P_WDT_WDTCR_OFFSET, regval);

  /* Store timeout parameters */
  priv->timeout_ms = timeout_ms;
  priv->clock_divider = divider;
  priv->timeout_setting = setting;

  return OK;
}

/****************************************************************************
 * Name: wdt_refresh_counter
 ****************************************************************************/

static int wdt_refresh_counter(struct ra8p_wdt_priv_s *priv)
{
  /* Perform the refresh sequence as specified by RA8P WDT */
  wdt_putreg8(priv, RA8P_WDT_WDTRR_OFFSET, RA8P_WDT_REFRESH_SEQ1);
  wdt_putreg8(priv, RA8P_WDT_WDTRR_OFFSET, RA8P_WDT_REFRESH_SEQ2);

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_wdt_initialize
 *
 * Description:
 *   Initialize the WDT based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   config - Pointer to WDT configuration structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_initialize(const struct ra8p_wdt_config_s *config)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;
  uint16_t wdtrcr;
  uint16_t wdtcr;
  uint8_t divider;
  uint8_t setting;
  int ret;

  priv->base = RA8P_WDT_BASE;
  priv->initialized = false;
  priv->enabled = false;
  priv->reset_enabled = true;
  priv->interrupt_only = false;
  priv->pclk_frequency = 62500000;  /* Use appropriate PCLKB frequency */
  priv->overflow_occurred = false;
  priv->overflow_count = 0;
  priv->low_power_mode = false;
  priv->clock_source_subclk = false;
  
  if (config != NULL)
    {
      priv->timeout_ms = config->timeout_ms ? config->timeout_ms : RA8P_WDT_DEFAULT_TIMEOUT_MS;
      priv->reset_enabled = config->reset_enabled;
      priv->interrupt_only = config->interrupt_only;
      priv->low_power_mode = config->low_power_mode;
      priv->clock_source_subclk = config->clock_select_subclk;
    }
  else
    {
      priv->timeout_ms = RA8P_WDT_DEFAULT_TIMEOUT_MS;
    }

  /* Calculate timeout parameters */
  ret = wdt_calculate_timeout_params(priv->timeout_ms, priv->pclk_frequency, &divider, &setting);
  if (ret != OK)
    {
      return ret;
    }

  /* Unlock registers */
  ret = wdt_unlock_registers(priv);
  if (ret != OK)
    {
      return ret;
    }

  /* Configure control register */
  wdtcr = 0;
  wdtcr |= (setting << RA8P_WDT_WDTCR_TOPS_SHIFT);  /* Timeout Period Select */
  wdtcr |= (divider << RA8P_WDT_WDTCR_CKS_SHIFT);    /* Clock Select */
  wdtcr |= RA8P_WDT_WDTCR_RPES_256CYCLES;           /* Reset Pulse Extension Select */
  
  if (priv->reset_enabled)
    {
      wdtcr |= RA8P_WDT_WDTCR_RPTEN;  /* Reset Enable */
    }
  else
    {
      wdtcr &= ~RA8P_WDT_WDTCR_RPTEN;
    }

  wdt_putreg16(priv, RA8P_WDT_WDTCR_OFFSET, wdtcr);

  /* Configure reset control register */
  wdtrcr = wdt_getreg8(priv, RA8P_WDT_WDTRCR_OFFSET);
  wdtrcr &= ~RA8P_WDT_WDTRCR_RSTIRQS_MASK;  /* Clear reset interrupt select */
  
  if (priv->interrupt_only)
    {
      wdtrcr |= RA8P_WDT_WDTRCR_RSTIRQS_IRQ;  /* Generate interrupt instead of reset */
    }
  else
    {
      wdtrcr |= RA8P_WDT_WDTRCR_RSTIRQS_RST;  /* Generate reset */
    }
  
  wdt_putreg8(priv, RA8P_WDT_WDTRCR_OFFSET, wdtrcr);

  /* Clear overflow flags */
  wdt_putreg8(priv, RA8P_WDT_WDTOVFCLR_OFFSET, RA8P_WDT_OVERFLOW_CLR_SEQ);

  /* Store calculated values */
  priv->clock_divider = divider;
  priv->timeout_setting = setting;

  priv->initialized = true;

  wdtinfo("WDT initialized: timeout=%u ms, reset=%s, interrupt_only=%s\n", 
          priv->timeout_ms, 
          priv->reset_enabled ? "yes" : "no",
          priv->interrupt_only ? "yes" : "no");
  return OK;
}

/****************************************************************************
 * Name: ra8p_wdt_enable
 *
 * Description:
 *   Enable the WDT based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_enable(void)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;
  uint16_t regval;
  int ret;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  /* Unlock registers first */
  ret = wdt_unlock_registers(priv);
  if (ret != OK)
    {
      return ret;
    }

  /* Enable WDT */
  regval = wdt_getreg16(priv, RA8P_WDT_WDTCR_OFFSET);
  regval |= RA8P_WDT_WDTCR_TME;  /* Enable WDT module */
  wdt_putreg16(priv, RA8P_WDT_WDTCR_OFFSET, regval);

  /* Perform initial refresh to start WDT */
  ret = wdt_refresh_counter(priv);
  if (ret != OK)
    {
      return ret;
    }

  priv->enabled = true;

  wdtinfo("WDT enabled\n");
  return OK;
}

/****************************************************************************
 * Name: ra8p_wdt_disable
 *
 * Description:
 *   Disable the WDT based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_disable(void)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;
  uint16_t regval;
  int ret;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  /* Unlock registers first */
  ret = wdt_unlock_registers(priv);
  if (ret != OK)
    {
      return ret;
    }

  /* Disable WDT */
  regval = wdt_getreg16(priv, RA8P_WDT_WDTCR_OFFSET);
  regval &= ~RA8P_WDT_WDTCR_TME;  /* Clear WDT enable */
  wdt_putreg16(priv, RA8P_WDT_WDTCR_OFFSET, regval);

  priv->enabled = false;

  wdtinfo("WDT disabled\n");
  return OK;
}

/****************************************************************************
 * Name: ra8p_wdt_refresh
 *
 * Description:
 *   Refresh the WDT based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_refresh(void)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;

  if (!priv->initialized || !priv->enabled)
    {
      return -EAGAIN;
    }

  /* Perform refresh sequence */
  int ret = wdt_refresh_counter(priv);
  if (ret != OK)
    {
      return ret;
    }

  wdtinfo("WDT refreshed\n");
  return OK;
}

/****************************************************************************
 * Name: ra8p_wdt_set_timeout
 *
 * Description:
 *   Set WDT timeout period based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   timeout_ms - Timeout period in milliseconds
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_set_timeout(uint32_t timeout_ms)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;
  int ret;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  if (timeout_ms == 0)
    {
      return -EINVAL;
    }

  /* Temporarily disable WDT for reconfiguration */
  if (priv->enabled)
    {
      ret = ra8p_wdt_disable();
      if (ret != OK)
        {
          return ret;
        }
    }

  /* Set new timeout */
  ret = wdt_set_timeout(priv, timeout_ms);
  if (ret != OK)
    {
      return ret;
    }

  /* Re-enable WDT if it was enabled */
  if (priv->enabled)
    {
      ret = ra8p_wdt_enable();
      if (ret != OK)
        {
          return ret;
        }
    }

  wdtinfo("WDT timeout set to %u ms\n", timeout_ms);
  return OK;
}

/****************************************************************************
 * Name: ra8p_wdt_is_enabled
 *
 * Description:
 *   Check if WDT is enabled based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_wdt_is_enabled(void)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;
  uint16_t regval;

  if (!priv->initialized)
    {
      return false;
    }

  regval = wdt_getreg16(priv, RA8P_WDT_WDTCR_OFFSET);
  return (regval & RA8P_WDT_WDTCR_TME) != 0;
}

/****************************************************************************
 * Name: ra8p_wdt_is_overflow_occurred
 *
 * Description:
 *   Check if WDT overflow has occurred based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if overflow occurred, false otherwise
 *
 ****************************************************************************/

bool ra8p_wdt_is_overflow_occurred(void)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;
  uint8_t regval;

  if (!priv->initialized)
    {
      return false;
    }

  regval = wdt_getreg8(priv, RA8P_WDT_WDTRCR_OFFSET);
  bool overflow_occurred = (regval & RA8P_WDT_WDTRCR_WDTRF) != 0;

  if (overflow_occurred)
    {
      priv->overflow_occurred = true;
      priv->overflow_count++;
    }

  return overflow_occurred;
}

/****************************************************************************
 * Name: ra8p_wdt_clear_overflow_flag
 *
 * Description:
 *   Clear WDT overflow flag based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_clear_overflow_flag(void)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  /* Unlock registers */
  int ret = wdt_unlock_registers(priv);
  if (ret != OK)
    {
      return ret;
    }

  /* Clear overflow flag by writing to clear register */
  wdt_putreg8(priv, RA8P_WDT_WDTOVFCLR_OFFSET, RA8P_WDT_OVERFLOW_CLR_SEQ);

  priv->overflow_occurred = false;

  wdtinfo("WDT overflow flag cleared\n");
  return OK;
}

/****************************************************************************
 * Name: ra8p_wdt_get_timeout
 *
 * Description:
 *   Get current WDT timeout period based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Current timeout period in milliseconds
 *
 ****************************************************************************/

uint32_t ra8p_wdt_get_timeout(void)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;

  if (!priv->initialized)
    {
      return 0;
    }

  return priv->timeout_ms;
}

/****************************************************************************
 * Name: ra8p_wdt_set_reset_mode
 *
 * Description:
 *   Configure WDT reset mode (reset vs interrupt) based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   reset - true to enable reset on timeout, false for interrupt only
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_set_reset_mode(bool reset)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;
  uint8_t regval;
  int ret;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  /* Unlock registers first */
  ret = wdt_unlock_registers(priv);
  if (ret != OK)
    {
      return ret;
    }

  regval = wdt_getreg8(priv, RA8P_WDT_WDTRCR_OFFSET);
  regval &= ~RA8P_WDT_WDTRCR_RSTIRQS_MASK;  /* Clear reset interrupt select */

  if (reset)
    {
      regval |= RA8P_WDT_WDTRCR_RSTIRQS_RST;  /* Generate reset */
      priv->reset_enabled = true;
      priv->interrupt_only = false;
    }
  else
    {
      regval |= RA8P_WDT_WDTRCR_RSTIRQS_IRQ;  /* Generate interrupt only */
      priv->reset_enabled = false;
      priv->interrupt_only = true;
    }

  wdt_putreg8(priv, RA8P_WDT_WDTRCR_OFFSET, regval);

  wdtinfo("WDT reset mode set to %s\n", reset ? "reset" : "interrupt");
  return OK;
}

/****************************************************************************
 * Name: ra8p_wdt_is_reset_enabled
 *
 * Description:
 *   Check if WDT reset on timeout is enabled based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if reset enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_wdt_is_reset_enabled(void)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;

  if (!priv->initialized)
    {
      return false;
    }

  return priv->reset_enabled;
}

/****************************************************************************
 * Name: ra8p_wdt_set_clock_divider
 *
 * Description:
 *   Set WDT clock divider based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   divider - Clock divider index (0-15)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_set_clock_divider(uint8_t divider)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;
  uint16_t regval;
  int ret;

  if (!priv->initialized || divider > 15)
    {
      return -EAGAIN;
    }

  /* Unlock registers first */
  ret = wdt_unlock_registers(priv);
  if (ret != OK)
    {
      return ret;
    }

  regval = wdt_getreg16(priv, RA8P_WDT_WDTCR_OFFSET);
  regval &= ~RA8P_WDT_WDTCR_CKS_MASK;  /* Clear clock select */
  regval |= (divider << RA8P_WDT_WDTCR_CKS_SHIFT);  /* Set new clock select */
  wdt_putreg16(priv, RA8P_WDT_WDTCR_OFFSET, regval);

  priv->clock_divider = divider;

  wdtinfo("WDT clock divider set to index %u\n", divider);
  return OK;
}

/****************************************************************************
 * Name: ra8p_wdt_get_clock_divider
 *
 * Description:
 *   Get current WDT clock divider based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Current clock divider index
 *
 ****************************************************************************/

uint8_t ra8p_wdt_get_clock_divider(void)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;

  if (!priv->initialized)
    {
      return 0;
    }

  return priv->clock_divider;
}

/****************************************************************************
 * Name: ra8p_wdt_feed
 *
 * Description:
 *   Alias for wdt_refresh based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_feed(void)
{
  return ra8p_wdt_refresh();
}

/****************************************************************************
 * Name: ra8p_wdt_start
 *
 * Description:
 *   Start WDT operation (alias for enable) based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_start(void)
{
  return ra8p_wdt_enable();
}

/****************************************************************************
 * Name: ra8p_wdt_stop
 *
 * Description:
 *   Stop WDT operation (alias for disable) based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_stop(void)
{
  return ra8p_wdt_disable();
}

/****************************************************************************
 * Name: ra8p_wdt_reset_counters
 *
 * Description:
 *   Reset WDT overflow counter based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_reset_counters(void)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  priv->overflow_count = 0;
  priv->overflow_occurred = false;

  wdtinfo("WDT overflow counter reset\n");
  return OK;
}

/****************************************************************************
 * Name: ra8p_wdt_get_overflow_count
 *
 * Description:
 *   Get WDT overflow counter based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Number of overflow events
 *
 ****************************************************************************/

uint32_t ra8p_wdt_get_overflow_count(void)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;

  if (!priv->initialized)
    {
      return 0;
    }

  return priv->overflow_count;
}

/****************************************************************************
 * Name: ra8p_wdt_enable_interrupt
 *
 * Description:
 *   Enable/disable WDT interrupt based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   enable - true to enable interrupt, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_enable_interrupt(bool enable)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;
  uint16_t regval;
  int ret;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  /* Unlock registers first */
  ret = wdt_unlock_registers(priv);
  if (ret != OK)
    {
      return ret;
    }

  regval = wdt_getreg16(priv, RA8P_WDT_WDTCR_OFFSET);

  if (enable)
    {
      regval |= RA8P_WDT_WDTCR_WOIE;  /* Enable overflow interrupt */
    }
  else
    {
      regval &= ~RA8P_WDT_WDTCR_WOIE; /* Disable overflow interrupt */
    }

  wdt_putreg16(priv, RA8P_WDT_WDTCR_OFFSET, regval);

  wdtinfo("WDT interrupt %s\n", enable ? "enabled" : "disabled");
  return OK;
}

/****************************************************************************
 * Name: ra8p_wdt_set_callback
 *
 * Description:
 *   Set WDT overflow callback based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   callback - Pointer to callback structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_set_callback(const struct ra8p_wdt_callback_s *callback)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;

  if (!priv->initialized || callback == NULL)
    {
      return -EAGAIN;
    }

  priv->callback = *callback;

  wdtinfo("WDT callback set\n");
  return OK;
}

/****************************************************************************
 * Name: ra8p_wdt_get_status
 *
 * Description:
 *   Get WDT status flags based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Status flags
 *
 ****************************************************************************/

uint32_t ra8p_wdt_get_status(void)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;
  uint8_t status;

  if (!priv->initialized)
    {
      return 0;
    }

  status = wdt_getreg8(priv, RA8P_WDT_WDTSR_OFFSET);
  return (uint32_t)status;
}

/****************************************************************************
 * Name: ra8p_wdt_get_error_flags
 *
 * Description:
 *   Get WDT error flags based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Error flags
 *
 ****************************************************************************/

uint32_t ra8p_wdt_get_error_flags(void)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;

  if (!priv->initialized)
    {
      return 0;
    }

  /* Check for overflow in WDTCTR register */
  uint8_t wdtctr = wdt_getreg8(priv, RA8P_WDT_WDTRCR_OFFSET);
  uint32_t errors = 0;

  if (wdtctr & RA8P_WDT_WDTRCR_WDTRF)
    {
      errors |= 1;  /* Overflow error */
    }

  return errors;
}

/****************************************************************************
 * Name: ra8p_wdt_clear_errors
 *
 * Description:
 *   Clear WDT error flags based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   flags - Error flags to clear
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_clear_errors(uint32_t flags)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;
  uint8_t regval;
  int ret;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  /* Unlock registers */
  ret = wdt_unlock_registers(priv);
  if (ret != OK)
    {
      return ret;
    }

  /* Clear error flags */
  regval = wdt_getreg8(priv, RA8P_WDT_WDTRCR_OFFSET);
  if (flags & 1)  /* Overflow error */
    {
      regval &= ~RA8P_WDT_WDTRCR_WDTRF;
    }
  wdt_putreg8(priv, RA8P_WDT_WDTRCR_OFFSET, regval);

  return OK;
}

/****************************************************************************
 * Name: ra8p_wdt_calculate_timeout
 *
 * Description:
 *   Calculate WDT timeout parameters from desired timeout in milliseconds.
 *
 * Input Parameters:
 *   timeout_ms - Desired timeout in milliseconds
 *   divider - Pointer to store calculated clock divider
 *   setting - Pointer to store calculated timeout setting
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_calculate_timeout(uint32_t timeout_ms, uint8_t *divider, uint8_t *setting)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;

  if (!priv->initialized || timeout_ms == 0 || divider == NULL || setting == NULL)
    {
      return -EAGAIN;
    }

  return wdt_calculate_timeout_params(timeout_ms, priv->pclk_frequency, divider, setting);
}

/****************************************************************************
 * Name: ra8p_wdt_set_low_power_mode
 *
 * Description:
 *   Set WDT low power mode based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   enable - true to enable low power mode, false for normal
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_set_low_power_mode(bool enable)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;
  uint8_t regval;
  int ret;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  /* Unlock registers */
  ret = wdt_unlock_registers(priv);
  if (ret != OK)
    {
      return ret;
    }

  regval = wdt_getreg8(priv, RA8P_WDT_WDTCSTPR_OFFSET);
  
  if (enable)
    {
      regval |= RA8P_WDT_WDTCSTPR_SLCSTP;  /* Enable count stop */
    }
  else
    {
      regval &= ~RA8P_WDT_WDTCSTPR_SLCSTP; /* Disable count stop */
    }
    
  wdt_putreg8(priv, RA8P_WDT_WDTCSTPR_OFFSET, regval);

  priv->low_power_mode = enable;

  wdtinfo("WDT low power mode %s\n", enable ? "enabled" : "disabled");
  return OK;
}

/****************************************************************************
 * Name: ra8p_wdt_get_prescaler
 *
 * Description:
 *   Get current WDT prescaler value based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Prescaler value
 *
 ****************************************************************************/

uint8_t ra8p_wdt_get_prescaler(void)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;

  if (!priv->initialized)
    {
      return 0;
    }

  /* For RA8P WDT, prescaler is embedded in the CKS value */
  return priv->clock_divider;
}

/****************************************************************************
 * Name: ra8p_wdt_get_current_count
 *
 * Description:
 *   Get current WDT counter value based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Current counter value
 *
 ****************************************************************************/

uint32_t ra8p_wdt_get_current_count(void)
{
  struct ra8p_wdt_priv_s *priv = &g_wdt;

  if (!priv->initialized)
    {
      return 0;
    }

  /* For RA8P WDT, there's no direct way to read the counter value */
  /* Return a calculated approximate value based on timeout settings */
  uint8_t setting = priv->timeout_setting;
  uint32_t timeout_cycles[] = {128, 512, 1024, 2048};
  
  if (setting < 4)
    {
      uint32_t clk_freq = priv->pclk_frequency >> priv->clock_divider;
      return timeout_cycles[setting] - (priv->timeout_ms * clk_freq / 1000);
    }

  return 0;
}

#endif /* CONFIG_RA8P_WDT */