/****************************************************************************
 * arch/arm/src/stm32u5/stm32u5_iwdg.c
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

#if defined(CONFIG_WATCHDOG) && defined(CONFIG_STM32U5_IWDG)

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Clocking *****************************************************************/

/* The minimum frequency of the IWDG clock is:
 *
 *  Fmin = Flsi / 256
 *
 * So the maximum delay (in milliseconds) is then:
 *
 *   1000 * IWDG_RLR_MAX / Fmin
 *
 * For example, if Flsi = 32Khz (the nominal, uncalibrated value), then the
 * maximum delay is approximately 32 seconds.
 */

#define IWDG_FMIN       (STM32_LSI_FREQUENCY / 256)
#define IWDG_MAXTIMEOUT (1000 * IWDG_RLR_MAX / IWDG_FMIN)

#ifndef CONFIG_STM32U5_IWDG_DEFTIMOUT
#  define CONFIG_STM32U5_IWDG_DEFTIMOUT IWDG_MAXTIMEOUT
#endif

#ifndef CONFIG_DEBUG_WATCHDOG_INFO
#  undef CONFIG_STM32U5_IWDG_REGDEBUG
#endif

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* This structure provides the private representation of the "lower-half"
 * driver state structure.  This structure must be cast-compatible with the
 * well-known watchdog_lowerhalf_s structure.
 */

struct stm32u5_lowerhalf_s
{
  const struct watchdog_ops_s *ops;  /* Lower half operations */
  uint32_t lsifreq;                /* The frequency of the LSI oscillator */
  uint32_t timeout;                /* The (actual) selected timeout */
  uint32_t lastreset;              /* The last reset time */
  bool     started;                /* true: The watchdog timer has been started */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

/* Register operations ******************************************************/

#ifdef CONFIG_STM32U5_IWDG_REGDEBUG
static uint32_t stm32u5_getreg(uint32_t addr);
static void     stm32u5_putreg(uint32_t val, uint32_t addr);
#else
#  define       stm32u5_getreg(addr)     getreg32(addr)
#  define       stm32u5_putreg(val,addr) putreg32(val,addr)
#endif

/* "Lower half" driver methods **********************************************/

static int stm32u5_start(struct watchdog_lowerhalf_s *lower);
static int stm32u5_stop(struct watchdog_lowerhalf_s *lower);
static int stm32u5_keepalive(struct watchdog_lowerhalf_s *lower);
static int stm32u5_getstatus(struct watchdog_lowerhalf_s *lower,
                              struct watchdog_status_s *status);
static int stm32u5_settimeout(struct watchdog_lowerhalf_s *lower,
                              uint32_t timeout);

/****************************************************************************
 * Private Data
 ****************************************************************************/

/* "Lower half" driver methods */

static const struct watchdog_ops_s g_wdgops =
{
  .start      = stm32u5_start,
  .stop       = stm32u5_stop,
  .keepalive  = stm32u5_keepalive,
  .getstatus  = stm32u5_getstatus,
  .settimeout = stm32u5_settimeout,
  .capture    = NULL,
  .ioctl      = NULL,
};

/* "Lower half" driver state */

static struct stm32u5_lowerhalf_s g_wdgdev;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32u5_getreg
 *
 * Description:
 *   Get the contents of an STM32U5 IWDG register
 *
 ****************************************************************************/

#ifdef CONFIG_STM32U5_IWDG_REGDEBUG
static uint32_t stm32u5_getreg(uint32_t addr)
{
  static uint32_t prevaddr = 0;
  static uint32_t count = 0;
  static uint32_t preval = 0;

  /* Read the value from the register */

  uint32_t val = getreg32(addr);

  /* Is this the same value that we read from the same register last time?
   * Are we polling the register?  If so, suppress some of the output.
   */

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

  /* No this is a new address or value */

  else
    {
      /* Did we print "..." for the previous value? */

      if (count > 3)
        {
          /* Yes.. then show how many times the value repeated */

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
 * Name: stm32u5_putreg
 *
 * Description:
 *   Put the contents of an STM32U5 IWDG register
 *
 ****************************************************************************/

#ifdef CONFIG_STM32U5_IWDG_REGDEBUG
static void stm32u5_putreg(uint32_t val, uint32_t addr)
{
  static uint32_t prevaddr = 0;
  static uint32_t count = 0;
  static uint32_t prevval = 0;

  /* Write the value to the register */

  putreg32(val, addr);

  /* Is this the same value that we wrote to the same register last time?
   * Are we polling the register?  If so, suppress some of the output.
   */

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

  /* No this is a new address or value */

  else
    {
      /* Did we print "..." for the previous value? */

      if (count > 3)
        {
          /* Yes.. then show how many times the value repeated */

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
 * Name: stm32u5_lock
 *
 * Description:
 *   Lock the IWDG registers
 *
 ****************************************************************************/

static void stm32u5_lock(void)
{
  /* Write the lock sequence to the key register */

  stm32u5_putreg(0, STM32U5_IWDG_KR);
}

/****************************************************************************
 * Name: stm32u5_unlock
 *
 * Description:
 *   Unlock the IWDG registers
 *
 ****************************************************************************/

static void stm32u5_unlock(void)
{
  /* Write the unlock sequence to the key register */

  stm32u5_putreg(IWDG_KR_KEY_ENABLE, STM32U5_IWDG_KR);
}

/****************************************************************************
 * Name: stm32u5_reload
 *
 * Description:
 *   Reload the IWDG counter
 *
 ****************************************************************************/

static void stm32u5_reload(void)
{
  /* Write the reload sequence to the key register */

  stm32u5_putreg(IWDG_KR_KEY_RELOAD, STM32U5_IWDG_KR);
}

/****************************************************************************
 * Name: stm32u5_start
 *
 * Description:
 *   Start the watchdog timer
 *
 * Input Parameters:
 *   lower - A reference to the watchdog state structure
 *
 * Returned Values:
 *   Zero on success; a negated errno value on failure
 *
 ****************************************************************************/

static int stm32u5_start(struct watchdog_lowerhalf_s *lower)
{
  struct stm32u5_lowerhalf_s *priv = (struct stm32u5_lowerhalf_s *)lower;

  wdvinfo("Entry: started: %d\n", priv->started);

  /* Have already started? */

  if (!priv->started)
    {
      /* Enable the watchdog */

      stm32u5_unlock();
      stm32u5_putreg(IWDG_KR_KEY_START, STM32U5_IWDG_KR);
      stm32u5_lock();

      /* Mark as started */

      priv->started = true;
      wdvinfo("IWDG started\n");
    }

  return OK;
}

/****************************************************************************
 * Name: stm32u5_stop
 *
 * Description:
 *   Stop the watchdog timer
 *
 * Input Parameters:
 *   lower - A reference to the watchdog state structure
 *
 * Returned Values:
 *   Zero on success; a negated errno value on failure
 *
 ****************************************************************************/

static int stm32u5_stop(struct watchdog_lowerhalf_s *lower)
{
  wdvinfo("Entry: not supported\n");
  return -ENOSYS;
}

/****************************************************************************
 * Name: stm32u5_keepalive
 *
 * Description:
 *   Reset the watchdog timer (reload)
 *
 * Input Parameters:
 *   lower - A reference to the watchdog state structure
 *
 * Returned Values:
 *   Zero on success; a negated errno value on failure
 *
 ****************************************************************************/

static int stm32u5_keepalive(struct watchdog_lowerhalf_s *lower)
{
  struct stm32u5_lowerhalf_s *priv = (struct stm32u5_lowerhalf_s *)lower;

  wdvinfo("Entry\n");

  /* Unlock and reload */

  stm32u5_unlock();
  stm32u5_reload();
  stm32u5_lock();

  /* Remember the reset time */

  priv->lastreset = clock_systime_ticks();

  return OK;
}

/****************************************************************************
 * Name: stm32u5_getstatus
 *
 * Description:
 *   Get the current watchdog timer status
 *
 * Input Parameters:
 *   lower  - A reference to the watchdog state structure
 *   status - The location to return the watchdog status
 *
 * Returned Values:
 *   Zero on success; a negated errno value on failure
 *
 ****************************************************************************/

static int stm32u5_getstatus(struct watchdog_lowerhalf_s *lower,
                              struct watchdog_status_s *status)
{
  struct stm32u5_lowerhalf_s *priv = (struct stm32u5_lowerhalf_s *)lower;
  uint32_t elapsed;

  wdvinfo("Entry\n");

  /* Return the status of the watchdog timer */

  status->flags = 0;
  if (priv->started)
    {
      status->flags |= WDOG_FLAG_ACTIVE;
    }

  status->timeout = priv->timeout;

  /* Get the time elapsed since the last keepalive */

  elapsed = (uint32_t)clock_systime_ticks() - priv->lastreset;
  status->timeleft = (priv->timeout * TICK_PER_SEC) - elapsed;

  wdvinfo("  flags: %08x timeout: %" PRIu32 " timeleft: %" PRIu32 "\n",
           status->flags, status->timeout, status->timeleft);

  return OK;
}

/****************************************************************************
 * Name: stm32u5_settimeout
 *
 * Description:
 *   Set the watchdog timer timeout
 *
 * Input Parameters:
 *   lower   - A reference to the watchdog state structure
 *   timeout - The new timeout value in milliseconds
 *
 * Returned Values:
 *   Zero on success; a negated errno value on failure
 *
 ****************************************************************************/

static int stm32u5_settimeout(struct watchdog_lowerhalf_s *lower,
                              uint32_t timeout)
{
  struct stm32u5_lowerhalf_s *priv = (struct stm32u5_lowerhalf_s *)lower;
  uint32_t prescaler;
  uint32_t reload;
  uint32_t regval;

  wdvinfo("Entry: timeout: %" PRIu32 "\n", timeout);

  /* The maximum timeout is IWDG_MAXTIMEOUT.  Check this first. */

  if (timeout > IWDG_MAXTIMEOUT)
    {
      wdinfo("ERROR: timeout %" PRIu32 " > maximum %d\n",
             timeout, IWDG_MAXTIMEOUT);
      return -EINVAL;
    }

  /* Then calculate the prescaler and reload values.
   *
   * timeout = (prescaler * reload) / IWDG_CLOCK
   * reload = timeout * IWDG_CLOCK / prescaler
   *
   * The reload value is a 12-bit value; we will constrain it to fit.
   */

  for (prescaler = 0; prescaler <= 7; prescaler++)
    {
      uint32_t reloadval = ((uint64_t)timeout * priv->lsifreq) >>
                          (prescaler + 2);

      if (reloadval <= IWDG_RLR_MAX)
        {
          reload = reloadval;
          break;
        }
    }

  if (prescaler > 7)
    {
      wdinfo("ERROR: timeout out of range\n");
      return -EINVAL;
    }

  /* Save the actual timeout and the prescaler */

  priv->timeout = timeout;

  /* Unlock and configure the prescaler and reload registers */

  stm32u5_unlock();

  /* Set the prescaler */

  regval  = stm32u5_getreg(STM32U5_IWDG_PR);
  regval &= ~IWDG_PR_MASK;
  regval |= prescaler;
  stm32u5_putreg(regval, STM32U5_IWDG_PR);

  /* Set the reload value */

  regval = reload & IWDG_RLR_RL_MASK;
  stm32u5_putreg(regval, STM32U5_IWDG_RLR);

  stm32u5_lock();

  wdvinfo("prescaler: %" PRIu32 " reload: %" PRIu32 "\n",
          prescaler, reload);

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32u5_iwdginitialize
 *
 * Description:
 *   Initialize the IWDG driver
 *
 * Input Parameters:
 *   None
 *
 * Returned Values:
 *   A reference to the lower half watchdog driver
 *
 ****************************************************************************/

struct watchdog_lowerhalf_s *stm32u5_iwdginitialize(void)
{
  struct stm32u5_lowerhalf_s *priv = &g_wdgdev;

  wdvinfo("Entry\n");

  /* Initialize the driver state structure */

  priv->ops = &g_wdgops;
  priv->started = false;
  priv->timeout = CONFIG_STM32U5_IWDG_DEFTIMOUT;

  /* Get the LSI frequency */

  priv->lsifreq = STM32_LSI_FREQUENCY;

  /* Set the default timeout */

  stm32u5_settimeout((struct watchdog_lowerhalf_s *)priv, priv->timeout);

  /* Return the watchdog operations */

  wdvinfo("IWDG driver initialized\n");
  return (struct watchdog_lowerhalf_s *)priv;
}

#endif /* CONFIG_WATCHDOG && CONFIG_STM32U5_IWDG */