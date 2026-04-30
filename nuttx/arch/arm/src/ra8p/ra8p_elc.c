/****************************************************************************
 * arch/arm/src/ra8p/ra8p_elc.c
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
#include <arch/board/board.h>

#include "arm_internal.h"
#include "chip.h"
#include "ra8p_config.h"
#include "ra8p_peripherals.h"
#include "hardware/ra8p_elc.h"

#ifdef CONFIG_RA8P_ELC

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Helper macros for register access */

#define ra8p_elc_putreg32(offset, val) \
  putreg32((val), RA8P_ELC_BASE + (offset))

#define ra8p_elc_getreg32(offset) \
  getreg32(RA8P_ELC_BASE + (offset))

#define ra8p_elc_modifyreg32(offset, clrbits, setbits) \
  ra8p_elc_putreg32(offset, \
    (ra8p_elc_getreg32(offset) & ~(clrbits)) | (setbits))

/* ELC timeout */

#define RA8P_ELC_TIMEOUT_MS                   (100)

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_elc_dev_s
{
  uint32_t base;                    /* Base address of ELC registers */
  int irq;                          /* ELC interrupt number */
  mutex_t lock;                     /* Thread-safe lock */
  sem_t wait_sem;                   /* Wait semaphore for interrupt handling */
  struct ra8p_elc_config_s config;  /* Current ELC configuration */
  bool initialized;                 /* True if initialized */
};

static struct ra8p_elc_dev_s g_elc_priv;

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int ra8p_elc_configure(struct ra8p_elc_config_s *config);
static void ra8p_elc_start(void);
static void ra8p_elc_stop(void);

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_elc_configure
 ****************************************************************************/

static int ra8p_elc_configure(struct ra8p_elc_config_s *config)
{
  uint32_t regval;
  uint8_t i;

  if (!config)
    {
      return -EINVAL;
    }

  /* Reset ELC */

  ra8p_elc_modifyreg32(RA8P_ELC_ELCCR_OFFSET, 0, RA8P_ELC_ELCCR_ELCRST);
  up_udelay(10);
  ra8p_elc_modifyreg32(RA8P_ELC_ELCCR_OFFSET, RA8P_ELC_ELCCR_ELCRST, 0);

  /* Configure event link table */

  for (i = 0; i < config->num_links && i < 32; i++)
    {
      /* Set event link */

      regval = (config->links[i].event << RA8P_ELC_ELCTBL_EVTSEL_SHIFT) |
                (config->links[i].module << RA8P_ELC_ELCTBL_ModSELR_SHIFT);

      ra8p_elc_putreg32(RA8P_ELC_ELCTBL_OFFSET + (i * 4), regval);

      /* Configure event type/polarity */

      regval = 0;
      if (config->links[i].type == RA8P_ELC_TYPE_FALLING)
        {
          regval |= (RA8P_ELC_ELCECR_ELCET_FALLING << RA8P_ELC_ELCECR_ELCET_SHIFT);
        }
      else if (config->links[i].type == RA8P_ELC_TYPE_BOTH)
        {
          regval |= (RA8P_ELC_ELCECR_ELCET_BOTH << RA8P_ELC_ELCECR_ELCET_SHIFT);
        }
      else if (config->links[i].type == RA8P_ELC_TYPE_LOW)
        {
          regval |= (RA8P_ELC_ELCECR_ELCET_LOW << RA8P_ELC_ELCECR_ELCET_SHIFT);
        }
      else
        {
          regval |= (RA8P_ELC_ELCECR_ELCET_RISING << RA8P_ELC_ELCECR_ELCET_SHIFT);
        }

      ra8p_elc_putreg32(RA8P_ELC_ELCECR_OFFSET + (i * 4), regval);

      /* Enable interrupt if requested */

      if (config->links[i].interrupt_enable)
        {
          ra8p_elc_modifyreg32(RA8P_ELC_ELCIER_OFFSET, 0, (1 << i));
        }
    }

  return OK;
}

/****************************************************************************
 * Name: ra8p_elc_start
 ****************************************************************************/

static void ra8p_elc_start(void)
{
  /* Enable ELC */

  ra8p_elc_modifyreg32(RA8P_ELC_ELCCR_OFFSET, 0, RA8P_ELC_ELCCR_ELCEN);
}

/****************************************************************************
 * Name: ra8p_elc_stop
 ****************************************************************************/

static void ra8p_elc_stop(void)
{
  /* Disable ELC */

  ra8p_elc_modifyreg32(RA8P_ELC_ELCCR_OFFSET, RA8P_ELC_ELCCR_ELCEN, 0);
}

/****************************************************************************
 * Name: ra8p_elc_interrupt
 ****************************************************************************/

static int ra8p_elc_interrupt(int irq, void *context, void *arg)
{
  struct ra8p_elc_dev_s *priv = (struct ra8p_elc_dev_s *)arg;
  uint32_t status;
  uint8_t i;

  if (!priv)
    {
      return OK;
    }

  /* Read event status */

  status = ra8p_elc_getreg32(RA8P_ELC_ELCISR_OFFSET);

  /* Clear interrupt flags */

  ra8p_elc_putreg32(RA8P_ELC_ELCISR_OFFSET, status);

  /* Process triggered events */

  for (i = 0; i < 32; i++)
    {
      if (status & (1 << i))
        {
          /* Event i triggered - could call registered callback */
        }
    }

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_elc_initialize
 *
 * Description:
 *   Initialize the ELC (Event Link Controller) driver
 *
 ****************************************************************************/

int ra8p_elc_initialize(void)
{
  struct ra8p_elc_dev_s *priv = &g_elc_priv;
  int ret;

  /* Initialize private data structure */

  memset(priv, 0, sizeof(struct ra8p_elc_dev_s));
  priv->base = RA8P_ELC_BASE;
  priv->irq = RA8P_IRQ_ELC;

  nxmutex_init(&priv->lock);
  nxsem_init(&priv->wait_sem, 0, 0);

  /* Reset ELC */

  ra8p_elc_modifyreg32(RA8P_ELC_ELCCR_OFFSET, 0, RA8P_ELC_ELCCR_ELCRST);
  up_udelay(10);
  ra8p_elc_modifyreg32(RA8P_ELC_ELCCR_OFFSET, RA8P_ELC_ELCCR_ELCRST, 0);

  /* Attach interrupt handler */

  ret = irq_attach(priv->irq, ra8p_elc_interrupt, priv);
  if (ret < 0)
    {
      goto errout_with_locks;
    }

  /* Enable interrupt */

  up_enable_irq(priv->irq);

  priv->initialized = true;

  return OK;

errout_with_locks:
  nxmutex_destroy(&priv->lock);
  nxsem_destroy(&priv->wait_sem);
  return ret;
}

/****************************************************************************
 * Name: ra8p_elc_configure_links
 *
 * Description:
 *   Configure event link table
 *
 ****************************************************************************/

int ra8p_elc_configure_links(struct ra8p_elc_link_s *links,
                               uint8_t num_links)
{
  struct ra8p_elc_dev_s *priv = &g_elc_priv;
  int ret;

  if (!priv || !priv->initialized || !links || num_links == 0)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Stop ELC if running */

  ra8p_elc_stop();

  /* Update configuration */

  priv->config.links = *links;
  priv->config.num_links = num_links;

  /* Configure ELC */

  ret = ra8p_elc_configure(&priv->config);
  if (ret < 0)
    {
      nxmutex_unlock(&priv->lock);
      return ret;
    }

  /* Start ELC */

  ra8p_elc_start();

  nxmutex_unlock(&priv->lock);
  return OK;
}

/****************************************************************************
 * Name: ra8p_elc_trigger_event
 *
 * Description:
 *   Manually trigger an event
 *
 ****************************************************************************/

int ra8p_elc_trigger_event(enum ra8p_elc_event_e event)
{
  if (event > RA8P_ELC_EVENT_MAX)
    {
      return -EINVAL;
    }

  /* Trigger event */

  ra8p_elc_putreg32(RA8P_ELC_ELCTRG_OFFSET, (1 << event));

  return OK;
}

/****************************************************************************
 * Name: ra8p_elc_get_status
 *
 * Description:
 *   Get ELC status
 *
 ****************************************************************************/

bool ra8p_elc_is_busy(void)
{
  struct ra8p_elc_dev_s *priv = &g_elc_priv;

  if (!priv || !priv->initialized)
    {
      return false;
    }

  uint32_t status = ra8p_elc_getreg32(RA8P_ELC_ELCSR_OFFSET);
  return !(status & RA8P_ELC_ELCSR_ELCBSY);
}

#endif /* CONFIG_RA8P_ELC */