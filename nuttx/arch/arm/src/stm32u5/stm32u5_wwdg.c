/****************************************************************************
 * arch/arm/src/stm32u5/stm32u5_wwdg.c
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

#include <inttypes.h>
#include <assert.h>
#include <errno.h>
#include <debug.h>

#include <nuttx/arch.h>
#include <nuttx/irq.h>
#include <nuttx/clock.h>
#include <nuttx/timers/watchdog.h>
#include <arch/board/board.h>

#include "arm_internal.h"
#include "stm32_rcc.h"
#include "hardware/stm32u5_wdg.h"

#if defined(CONFIG_WATCHDOG) && defined(CONFIG_STM32U5_WWDG)

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Clocking *****************************************************************/

/* The WWDG clock is PCLK1 (APB1) divided by 4096.
 *
 * PCLK1_MAX is the maximum frequency of the APB1 bus.
 * WWDG_FMIN is the minimum frequency of the WWDG clock.
 * WWDG_MAXTIMEOUT is the maximum timeout in milliseconds.
 */

#define WWDG_FMIN       (STM32_PCLK1_FREQUENCY / 4096)
#define WWDG_MAXTIMEOUT (1000 * 64 / WWDG_FMIN)

#ifndef CONFIG_STM32U5_WWDG_DEFTIMOUT
#  define CONFIG_STM32U5_WWDG_DEFTIMOUT WWDG_MAXTIMEOUT
#endif

#ifndef CONFIG_DEBUG_WATCHDOG_INFO
#  undef CONFIG_STM32U5_WWDG_REGDEBUG
#endif

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* This structure provides the private representation of the "lower-half"
 * driver state structure.  This structure must be cast-compatible with the
 * well-known watchdog_lowerhalf_s structure.
 */

struct stm32u5_wwdg_lowerhalf_s
{
  const struct watchdog_ops_s *ops;   /* Lower half operations */
  uint32_t pclk1freq;                 /* The frequency of PCLK1 */
  uint32_t timeout;                   /* The (actual) selected timeout */
  uint32_t lastreset;                 /* The last reset time */
  bool     started;                   /* true: The watchdog timer has been started */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

/* Register operations ******************************************************/

#ifdef CONFIG_STM32U5_WWDG_REGDEBUG
static uint32_t stm32u5_wwdg_getreg(uint32_t addr);
static void     stm32u5_wwdg_putreg(uint32_t val, uint32_t addr);
#else
#  define       stm32u5_wwdg_getreg(addr)     getreg32(addr)
#  define       stm32u5_wwdg_putreg(val,addr) putreg32(val,addr)
#endif

/* "Lower half" driver methods **********************************************/

static int stm32u5_wwdg_start(struct watchdog_lowerhalf_s *lower);
static int stm32u5_wwdg_stop(struct watchdog_lowerhalf_s *lower);
static int stm32u5_wwdg_keepalive(struct watchdog_lowerhalf_s *lower);
static int stm32u5_wwdg_getstatus(struct watchdog_lowerhalf_s *lower,
                                    struct watchdog_status_s *status);
static int stm32u5_wwdg_settimeout(struct watchdog_lowerhalf_s *lower,
                                      uint32_t timeout);

/****************************************************************************
 * Private Data
 ****************************************************************************/

/* "Lower half" driver methods */

static const struct watchdog_ops_s g_wwdgops =
{
  .start      = stm32u5_wwdg_start,
  .stop       = stm32u5_wwdg_stop,
  .keepalive  = stm32u5_wwdg_keepalive,
  .getstatus  = stm32u5_wwdg_getstatus,
  .settimeout = stm32u5_wwdg_settimeout,
  .capture    = NULL,
  .ioctl      = NULL,
};

/* "Lower half" driver state */

static struct stm32u5_wwdg_lowerhalf_s g_wwdgdev;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32u5_wwdg_getreg
 *
 * Description:
 *   Get the contents of an STM32U5 WWDG register
 *
 ****************************************************************************/

#ifdef CONFIG_STM32U5_WWDG_REGDEBUG
static uint32_t stm32u5_wwdg_getreg(uint32_t addr)
{
  static uint32_t prevaddr = 0;
  static uint32_t count = 0;
  static uint32_t preval = 0;

  uint32_t val = getreg32(addr);

  if (addr == prevaddr && val == preval)
    {
      if (count == 0xffffffff || ++count > 3)
        {
          if (count == 4)
            {
              wdinfo("...\n");
            }

          return val;
        }
    }
  else
    {
      if (count > 3)
        {
          wdinfo("[repeats %d more times]\n", count - 3);
        }

      wdinfo("%08x -> %08x\n", addr, val);
      count = 1;
      prevaddr = addr;
      preval = val;
    }

  return val;
}
#endif

/****************************************************************************
 * Name: stm32u5_wwdg_putreg
 *
 * Description:
 *   Put the contents of an STM32U5 WWDG register
 *
 ****************************************************************************/

#ifdef CONFIG_STM32U5_WWDG_REGDEBUG
static void stm32u5_wwdg_putreg(uint32_t val, uint32_t addr)
{
  static uint32_t prevaddr = 0;
  static uint32_t count = 0;
  static uint32_t prevval = 0;

  putreg32(val, addr);

  if (addr == prevaddr && val == prevval)
    {
      if (count == 0xffffffff || ++count > 3)
        {
          if (count == 4)
            {
              wdinfo("...\n");
            }

          return;
        }
    }
  else
    {
      if (count > 3)
        {
          wdinfo("[repeats %d more times]\n", count - 3);
        }

      wdinfo("%08x <- %08x\n", addr, val);
      count = 1;
      prevaddr = addr;
      prevval = val;
    }
}
#endif

/****************************************************************************
 * Name: stm32u5_wwdg_start
 *
 * Description:
 *   Start the WWDG timer
 *
 * Input Parameters:
 *   lower - A reference to the watchdog state structure
 *
 * Returned Values:
 *   Zero on success; a negated errno value on failure
 *
 ****************************************************************************/

static int stm32u5_wwdg_start(struct watchdog_lowerhalf_s *lower)
{
  struct stm32u5_wwdg_lowerhalf_s *priv =
    (struct stm32u5_wwdg_lowerhalf_s *)lower;

  wdvinfo("Entry: started: %d\n", priv->started);

  if (!priv->started)
    {
      uint32_t regval;

      /* Get the current CR register value and set the enable bit */

      regval = stm32u5_wwdg_getreg(STM32U5_WWDG_CR);
      regval |= WWDG_CR_WDGA;
      stm32u5_wwdg_putreg(regval, STM32U5_WWDG_CR);

      priv->started = true;
      wdvinfo("WWDG started\n");
    }

  return OK;
}

/****************************************************************************
 * Name: stm32u5_wwdg_stop
 *
 * Description:
 *   Stop the WWDG timer
 *
 * Input Parameters:
 *   lower - A reference to the watchdog state structure
 *
 * Returned Values:
 *   Zero on success; a negated errno value on failure
 *
 ****************************************************************************/

static int stm32u5_wwdg_stop(struct watchdog_lowerhalf_s *lower)
{
  wdvinfo("Entry: not supported\n");
  return -ENOSYS;
}

/****************************************************************************
 * Name: stm32u5_wwdg_keepalive
 *
 * Description:
 *   Reset the WWDG timer (reload)
 *
 * Input Parameters:
 *   lower - A reference to the watchdog state structure
 *
 * Returned Values:
 *   Zero on success; a negated errno value on failure
 *
 ****************************************************************************/

static int stm32u5_wwdg_keepalive(struct watchdog_lowerhalf_s *lower)
{
  struct stm32u5_wwdg_lowerhalf_s *priv =
    (struct stm32u5_wwdg_lowerhalf_s *)lower;

  wdvinfo("Entry\n");

  /* Set the T bits in the CR register to reload the counter */

  uint32_t timeout = priv->timeout;
  uint32_t t = ((timeout * priv->pclk1freq) / 1000) >> 13;

  /* T must be greater than window value W, and less than or equal to 0x3f */

  if (t > 0x3f)
    {
      t = 0x3f;
    }

  uint32_t regval = stm32u5_wwdg_getreg(STM32U5_WWDG_CR);
  regval &= ~WWDG_CR_T_MASK;
  regval |= (t & 0x3f);
  stm32u5_wwdg_putreg(regval, STM32U5_WWDG_CR);

  /* Remember the reset time */

  priv->lastreset = clock_systime_ticks();

  return OK;
}

/****************************************************************************
 * Name: stm32u5_wwdg_getstatus
 *
 * Description:
 *   Get the current WWDG timer status
 *
 * Input Parameters:
 *   lower  - A reference to the watchdog state structure
 *   status - The location to return the watchdog status
 *
 * Returned Values:
 *   Zero on success; a negated errno value on failure
 *
 ****************************************************************************/

static int stm32u5_wwdg_getstatus(struct watchdog_lowerhalf_s *lower,
                                    struct watchdog_status_s *status)
{
  struct stm32u5_wwdg_lowerhalf_s *priv =
    (struct stm32u5_wwdg_lowerhalf_s *)lower;
  uint32_t elapsed;

  wdvinfo("Entry\n");

  status->flags = 0;
  if (priv->started)
    {
      status->flags |= WDOG_FLAG_ACTIVE;
    }

  status->timeout = priv->timeout;

  elapsed = (uint32_t)clock_systime_ticks() - priv->lastreset;
  status->timeleft = (priv->timeout * TICK_PER_SEC) - elapsed;

  wdvinfo("  flags: %08x timeout: %" PRIu32 " timeleft: %" PRIu32 "\n",
           status->flags, status->timeout, status->timeleft);

  return OK;
}

/****************************************************************************
 * Name: stm32u5_wwdg_settimeout
 *
 * Description:
 *   Set the WWDG timer timeout
 *
 * Input Parameters:
 *   lower   - A reference to the watchdog state structure
 *   timeout - The new timeout value in milliseconds
 *
 * Returned Values:
 *   Zero on success; a negated errno value on failure
 *
 ****************************************************************************/

static int stm32u5_wwdg_settimeout(struct watchdog_lowerhalf_s *lower,
                                      uint32_t timeout)
{
  struct stm32u5_wwdg_lowerhalf_s *priv =
    (struct stm32u5_wwdg_lowerhalf_s *)lower;
  uint32_t regval;
  uint32_t t;
  uint32_t w;

  wdvinfo("Entry: timeout: %" PRIu32 "\n", timeout);

  if (timeout > WWDG_MAXTIMEOUT)
    {
      wdinfo("ERROR: timeout %" PRIu32 " > maximum %d\n",
             timeout, WWDG_MAXTIMEOUT);
      return -EINVAL;
    }

  /* Calculate the T value (counter value).
   * The formula is:
   *   timeout = (4096 * T) / PCLK1
   * So: T = (timeout * PCLK1) / 4096
   */

  t = ((uint64_t)timeout * priv->pclk1freq) >> 13;

  /* T must be less than 0x40 */

  if (t > 0x3f)
    {
      t = 0x3f;
    }

  /* W must be less than T, so set W to T-1 */

  w = (t > 0) ? (t - 1) : 0;

  /* Save the actual timeout */

  priv->timeout = timeout;

  /* Configure CFR register:
   * - Set the window value W
   * - Set the prescaler (currently fixed at /1)
   */

  regval = stm32u5_wwdg_getreg(STM32U5_WWDG_CFR);
  regval &= ~(WWDG_CFR_W_MASK | WWDG_CFR_WDGTB_MASK);
  regval |= (w & WWDG_CFR_W_MASK);
  stm32u5_wwdg_putreg(regval, STM32U5_WWDG_CFR);

  /* Set the T bits in the CR register */

  regval = stm32u5_wwdg_getreg(STM32U5_WWDG_CR);
  regval &= ~WWDG_CR_T_MASK;
  regval |= (t & 0x3f);
  stm32u5_wwdg_putreg(regval, STM32U5_WWDG_CR);

  wdvinfo("t: %" PRIu32 " w: %" PRIu32 "\n", t, w);

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32u5_wwdginitialize
 *
 * Description:
 *   Initialize the WWDG driver
 *
 * Input Parameters:
 *   None
 *
 * Returned Values:
 *   A reference to the lower half watchdog driver
 *
 ****************************************************************************/

struct watchdog_lowerhalf_s *stm32u5_wwdginitialize(void)
{
  struct stm32u5_wwdg_lowerhalf_s *priv = &g_wwdgdev;

  wdvinfo("Entry\n");

  priv->ops = &g_wwdgops;
  priv->started = false;
  priv->timeout = CONFIG_STM32U5_WWDG_DEFTIMOUT;
  priv->pclk1freq = STM32_PCLK1_FREQUENCY;

  stm32u5_wwdg_settimeout((struct watchdog_lowerhalf_s *)priv,
                          priv->timeout);

  wdvinfo("WWDG driver initialized\n");
  return (struct watchdog_lowerhalf_s *)priv;
}

#endif /* CONFIG_WATCHDOG && CONFIG_STM32U5_WWDG */