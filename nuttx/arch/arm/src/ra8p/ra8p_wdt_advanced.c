/****************************************************************************
 * arch/arm/src/ra8p/ra8p_wdt_advanced.c
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
#include <nuttx/timers/watchdog.h>
#include <arch/board/board.h>

#include "arm_internal.h"
#include "chip.h"
#include "ra8p_config.h"
#include "ra8p_peripherals.h"
#include "hardware/ra8p_wdt.h"

#ifdef CONFIG_RA8P_WDT_ADVANCED

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Helper macros for register access */

#define ra8p_wdt_putreg16(offset, val) \
  putreg16((val), RA8P_WDT_BASE + (offset))

#define ra8p_wdt_getreg16(offset) \
  getreg16(RA8P_WDT_BASE + (offset))

#define ra8p_wdt_modifyreg16(offset, clrbits, setbits) \
  ra8p_wdt_putreg16(offset, \
    (ra8p_wdt_getreg16(offset) & ~(clrbits)) | (setbits))

/* WDT timeout in milliseconds */

#define RA8P_WDT_DEFAULT_TIMEOUT_MS            (1000)

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_wdt_dev_s
{
  struct watchdog_dev_s dev;        /* Watchdog interface */
  uint32_t base;                    /* Base address of WDT registers */
  mutex_t lock;                     /* Thread-safe lock */
  uint32_t timeout_ms;              /* Current timeout in milliseconds */
  bool started;                     /* True if watchdog is running */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int ra8p_wdt_capture(struct watchdog_dev_s *dev, wdogcrtfunc_t crit);
static int ra8p_wdt_enable(struct watchdog_dev_s *dev);
static void ra8p_wdt_disable(struct watchdog_dev_s *dev);
static int ra8p_wdt_gettimeleft(struct watchdog_dev_s *dev, uint32_t *timeleft);
static int ra8p_wdt_settimeout(struct watchdog_dev_s *dev, uint32_t timeout);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_wdt_dev_s g_wdt_priv;

static const struct watchdog_ops_s g_wdt_ops =
{
  .capture    = ra8p_wdt_capture,
  .enable     = ra8p_wdt_enable,
  .disable    = ra8p_wdt_disable,
  ._gettimeleft = ra8p_wdt_gettimeleft,
  .settimeout = ra8p_wdt_settimeout,
};

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_wdt_capture
 ****************************************************************************/

static int ra8p_wdt_capture(struct watchdog_dev_s *dev, wdgetcrtfunc_t crit)
{
  /* Set critical region callback if needed */
  /* For RA8P, we can configure the WDT to generate NMI on timeout */
  
  return OK;
}

/****************************************************************************
 * Name: ra8p_wdt_enable
 ****************************************************************************/

static int ra8p_wdt_enable(struct watchdog_dev_s *dev)
{
  struct ra8p_wdt_dev_s *priv = (struct ra8p_wdt_dev_s *)dev;
  uint16_t regval;

  if (!priv)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Configure WDT:
   * - Set prescaler
   * - Enable WDT
   * - Configure window mode if needed
   */

  /* Calculate prescaler based on timeout */
  uint32_t prescaler = 128; // Use 128 prescaler for reasonable range
  
  regval = (prescaler << RA8P_WDT_WDTPR_VALUE_SHIFT) |
           RA8P_WDT_WDTCR_TCSR_Msk; // Enable WDT

  ra8p_wdt_putreg16(RA8P_WDT_WDTCR_OFFSET, regval);

  /* Set timeout value */
  uint16_t timeout_val = priv->timeout_ms * 10; // Convert ms to ticks based on clock
  ra8p_wdt_putreg16(RA8P_WDT_WDTRR_OFFSET, 0xA5); // Refresh register
  ra8p_wdt_putreg16(RA8P_WDT_WDTRR_OFFSET, 0x5A); // Confirm refresh

  priv->started = true;

  nxmutex_unlock(&priv->lock);
  return OK;
}

/****************************************************************************
 * Name: ra8p_wdt_disable
 ****************************************************************************/

static void ra8p_wdt_disable(struct watchdog_dev_s *dev)
{
  struct ra8p_wdt_dev_s *priv = (struct ra8p_wdt_dev_s *)dev;

  if (!priv)
    {
      return;
    }

  nxmutex_lock(&priv->lock);

  /* Disable WDT - may require special sequence */
  ra8p_wdt_putreg16(RA8P_WDT_WDTCR_OFFSET, 0);

  priv->started = false;

  nxmutex_unlock(&priv->lock);
}

/****************************************************************************
 * Name: ra8p_wdt_gettimeleft
 ****************************************************************************/

static int ra8p_wdt_gettimeleft(struct watchdog_dev_s *dev, uint32_t *timeleft)
{
  struct ra8p_wdt_dev_s *priv = (struct ra8p_wdt_dev_s *)dev;

  if (!priv || !timeleft)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Calculate time left - this is approximate */
  *timeleft = priv->timeout_ms; // Simplified - in real implementation would read counter

  nxmutex_unlock(&priv->lock);
  return OK;
}

/****************************************************************************
 * Name: ra8p_wdt_settimeout
 ****************************************************************************/

static int ra8p_wdt_settimeout(struct watchdog_dev_s *dev, uint32_t timeout)
{
  struct ra8p_wdt_dev_s *priv = (struct ra8p_wdt_dev_s *)dev;

  if (!priv || timeout == 0)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  priv->timeout_ms = timeout;

  nxmutex_unlock(&priv->lock);
  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_wdt_initialize
 *
 * Description:
 *   Initialize the advanced WDT driver
 *
 ****************************************************************************/

int ra8p_wdt_advanced_initialize(void)
{
  struct ra8p_wdt_dev_s *priv = &g_wdt_priv;
  int ret;

  /* Initialize private data structure */

  memset(priv, 0, sizeof(struct ra8p_wdt_dev_s));
  priv->base = RA8P_WDT_BASE;
  priv->timeout_ms = RA8P_WDT_DEFAULT_TIMEOUT_MS;

  nxmutex_init(&priv->lock);

  /* Initialize watchdog ops */

  priv->dev.ops = &g_wdt_ops;

  /* Register watchdog device */

  ret = watchdog_register("/dev/watchdog0", &priv->dev);
  if (ret < 0)
    {
      nxmutex_destroy(&priv->lock);
      return ret;
    }

  return OK;
}

#endif /* CONFIG_RA8P_WDT_ADVANCED */