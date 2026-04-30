/****************************************************************************
 * arch/arm/src/ra8p/ra8p_agt.c
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
#include <nuttx/timers/timer.h>
#include <arch/board/board.h>

#include "arm_internal.h"
#include "chip.h"
#include "ra8p_config.h"
#include "ra8p_peripherals.h"
#include "hardware/ra8p_agt.h"

#ifdef CONFIG_RA8P_AGT0

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Helper macros for register access */

#define ra8p_agt_putreg16(base, offset, val) \
  putreg16((val), (base) + (offset))

#define ra8p_agt_getreg16(base, offset) \
  getreg16((base) + (offset))

#define ra8p_agt_modifyreg16(base, offset, clrbits, setbits) \
  ra8p_agt_putreg16(base, offset, \
    (ra8p_agt_getreg16(base, offset) & ~(clrbits)) | (setbits))

/* AGT timeout */

#define RA8P_AGT_TIMEOUT_MS                    (100)

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_agt_dev_s
{
  struct timer_dev_s dev;           /* Timer interface */
  uint32_t base;                    /* Base address of AGT registers */
  int irq;                          /* AGT interrupt number */
  mutex_t lock;                     /* Thread-safe lock */
  sem_t wait_sem;                   /* Wait semaphore for interrupt handling */
  struct ra8p_agt_config_s config;  /* Current AGT configuration */
  uint16_t period;                  /* Timer period */
  bool initialized;                 /* True if initialized */
  bool running;                     /* True if timer is running */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int ra8p_agt_configure(struct ra8p_agt_dev_s *priv);
static void ra8p_agt_start(struct ra8p_agt_dev_s *priv);
static void ra8p_agt_stop(struct ra8p_agt_dev_s *priv);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_agt_dev_s g_agt0_priv;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_agt_configure
 ****************************************************************************/

static int ra8p_agt_configure(struct ra8p_agt_dev_s *priv)
{
  uint16_t regval;

  /* Configure clock source and divider */

  regval = (priv->config.clock << RA8P_AGT_AGTMR_TCK_SHIFT) |
           (priv->config.div << RA8P_AGT_AGTMR_TPCS_SHIFT);

  ra8p_agt_putreg16(priv->base, RA8P_AGT_AGTMR_OFFSET, regval);

  /* Configure timer mode and function */

  regval = (priv->config.mode << RA8P_AGT_AGTCR_TMOD_SHIFT) |
           (priv->config.func << RA8P_AGT_AGTCR_TFUNC_SHIFT);

  ra8p_agt_putreg16(priv->base, RA8P_AGT_AGTCR_OFFSET, regval);

  /* Set period */

  ra8p_agt_putreg16(priv->base, RA8P_AGT_AGTCMPA_OFFSET, priv->period);

  /* Configure interrupts */

  if (priv->config.interrupt_enable)
    {
      ra8p_agt_modifyreg16(priv->base, RA8P_AGT_AGTIER_OFFSET,
                           0, RA8P_AGT_AGTIER_CCMAE);
    }
  else
    {
      ra8p_agt_modifyreg16(priv->base, RA8P_AGT_AGTIER_OFFSET,
                           RA8P_AGT_AGTIER_CCMAE, 0);
    }

  /* Configure output */

  if (priv->config.output_enable)
    {
      regval = RA8P_AGT_AGTOCR_TOE;
      if (priv->config.output_polarity)
        {
          regval |= RA8P_AGT_AGTOCR_TOPOL;
        }

      ra8p_agt_putreg16(priv->base, RA8P_AGT_AGTOCR_OFFSET, regval);
    }

  return OK;
}

/****************************************************************************
 * Name: ra8p_agt_start
 ****************************************************************************/

static void ra8p_agt_start(struct ra8p_agt_dev_s *priv)
{
  if (priv->running)
    {
      return;
    }

  /* Start timer */

  ra8p_agt_modifyreg16(priv->base, RA8P_AGT_AGTCR_OFFSET,
                       0, RA8P_AGT_AGTCR_TSTART);

  priv->running = true;
}

/****************************************************************************
 * Name: ra8p_agt_stop
 ****************************************************************************/

static void ra8p_agt_stop(struct ra8p_agt_dev_s *priv)
{
  if (!priv->running)
    {
      return;
    }

  /* Stop timer */

  ra8p_agt_modifyreg16(priv->base, RA8P_AGT_AGTCR_OFFSET,
                       0, RA8P_AGT_AGTCR_TCPE);

  priv->running = false;
}

/****************************************************************************
 * Name: ra8p_agt_interrupt
 ****************************************************************************/

static int ra8p_agt_interrupt(int irq, void *context, void *arg)
{
  struct ra8p_agt_dev_s *priv = (struct ra8p_agt_dev_s *)arg;
  uint16_t status;

  if (!priv)
    {
      return OK;
    }

  /* Read status */

  status = ra8p_agt_getreg16(priv->base, RA8P_AGT_AGTSR_OFFSET);

  /* Clear status flags */

  ra8p_agt_putreg16(priv->base, RA8P_AGT_AGTSR_OFFSET, status);

  /* Handle compare match A */

  if (status & RA8P_AGT_AGTSR_CCMFA)
    {
      /* Timer period elapsed */

      nxsem_post(&priv->wait_sem);
    }

  return OK;
}

/****************************************************************************
 * Name: ra8p_agt_handler
 ****************************************************************************/

static int ra8p_agt_handler(struct timer_dev_s *dev, void *arg)
{
  struct ra8p_agt_dev_s *priv = (struct ra8p_agt_dev_s *)dev;

  if (!priv)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Start timer */

  ra8p_agt_start(priv);

  nxmutex_unlock(&priv->lock);

  return OK;
}

/****************************************************************************
 * Name: ra8p_agt_timeout
 ****************************************************************************/

static int ra8p_agt_timeout(struct timer_dev_s *dev, uint32_t timeout)
{
  struct ra8p_agt_dev_s *priv = (struct ra8p_agt_dev_s *)dev;

  if (!priv || timeout == 0)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Calculate period based on clock frequency */

  priv->period = (uint16_t)timeout;

  /* Configure timer */

  ra8p_agt_configure(priv);

  nxmutex_unlock(&priv->lock);

  return OK;
}

/****************************************************************************
 * Name: ra8p_agt_start_timer
 ****************************************************************************/

static int ra8p_agt_start_timer(struct timer_dev_s *dev)
{
  struct ra8p_agt_dev_s *priv = (struct ra8p_agt_dev_s *)dev;

  if (!priv)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);
  ra8p_agt_start(priv);
  nxmutex_unlock(&priv->lock);

  return OK;
}

/****************************************************************************
 * Name: ra8p_agt_stop_timer
 ****************************************************************************/

static int ra8p_agt_stop_timer(struct timer_dev_s *dev)
{
  struct ra8p_agt_dev_s *priv = (struct ra8p_agt_dev_s *)dev;

  if (!priv)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);
  ra8p_agt_stop(priv);
  nxmutex_unlock(&priv->lock);

  return OK;
}

/****************************************************************************
 * Name: ra8p_agt_getstatus
 ****************************************************************************/

static int ra8p_agt_getstatus(struct timer_dev_s *dev,
                              struct timer_status_s *status)
{
  struct ra8p_agt_dev_s *priv = (struct ra8p_agt_dev_s *)dev;

  if (!priv || !status)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  status->flags = priv->running ? TCFLAGS_ACTIVE : 0;
  status->timeout = priv->period;
  status->timeleft = ra8p_agt_getreg16(priv->base, RA8P_AGT_AGT_OFFSET);

  nxmutex_unlock(&priv->lock);

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_agt_initialize
 *
 * Description:
 *   Initialize the AGT (Asynchronous General-Purpose Timer) driver
 *
 ****************************************************************************/

int ra8p_agt_initialize(void)
{
  struct ra8p_agt_dev_s *priv = &g_agt0_priv;
  int ret;

  /* Initialize private data structure */

  memset(priv, 0, sizeof(struct ra8p_agt_dev_s));
  priv->base = RA8P_AGT0_BASE;
  priv->irq = RA8P_IRQ_AGT0;

  nxmutex_init(&priv->lock);
  nxsem_init(&priv->wait_sem, 0, 0);

  /* Set default configuration */

  priv->config.clock = RA8P_AGT_CLOCK_LOCO;
  priv->config.mode = RA8P_AGT_MODE_TIMER;
  priv->config.func = RA8P_AGT_FUNC_CLOCK_COUNT;
  priv->config.div = RA8P_AGT_DIV_1;
  priv->config.interrupt_enable = true;
  priv->config.output_enable = false;

  /* Configure AGT */

  ret = ra8p_agt_configure(priv);
  if (ret < 0)
    {
      goto errout_with_locks;
    }

  /* Attach interrupt handler */

  ret = irq_attach(priv->irq, ra8p_agt_interrupt, priv);
  if (ret < 0)
    {
      goto errout_with_locks;
    }

  /* Enable interrupt */

  up_enable_irq(priv->irq);

  priv->initialized = true;

  /* Register timer device */

  ret = timer_register("/dev/timer0", &priv->dev);
  if (ret < 0)
    {
      goto errout_with_irq;
    }

  return OK;

errout_with_irq:
  up_disable_irq(priv->irq);
  irq_detach(priv->irq);
errout_with_locks:
  nxmutex_destroy(&priv->lock);
  nxsem_destroy(&priv->wait_sem);
  return ret;
}

/****************************************************************************
 * Name: g_agt_ops
 *
 * Description:
 *   AGT timer operations
 *
 ****************************************************************************/

static const struct timer_ops_s g_agt_ops =
{
  .handler    = ra8p_agt_handler,
  .timeout    = ra8p_agt_timeout,
  .start      = ra8p_agt_start_timer,
  .stop       = ra8p_agt_stop_timer,
  .getstatus  = ra8p_agt_getstatus,
};

#endif /* CONFIG_RA8P_AGT0 */