/****************************************************************************
 * arch/arm/src/ra8p/ra8p_comparator.c
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
#include "hardware/ra8p_comparator.h"

#ifdef CONFIG_RA8P_COMPARATOR

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Helper macros for register access */

#define ra8p_comp_putreg32(offset, val) \
  putreg32((val), RA8P_COMP_BASE + (offset))

#define ra8p_comp_getreg32(offset) \
  getreg32(RA8P_COMP_BASE + (offset))

#define ra8p_comp_modifyreg32(offset, clrbits, setbits) \
  ra8p_comp_putreg32(offset, \
    (ra8p_comp_getreg32(offset) & ~(clrbits)) | (setbits))

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_comparator_dev_s
{
  mutex_t lock;                     /* Thread-safe lock */
  struct ra8p_comparator_config_s config[2]; /* Comparator configurations */
  bool initialized;                 /* True if initialized */
};

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_comparator_dev_s g_comp_priv;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_comparator_initialize
 *
 * Description:
 *   Initialize the comparator driver
 *
 ****************************************************************************/

int ra8p_comparator_initialize(void)
{
  struct ra8p_comparator_dev_s *priv = &g_comp_priv;

  /* Initialize private data structure */

  memset(priv, 0, sizeof(struct ra8p_comparator_dev_s));

  nxmutex_init(&priv->lock);

  priv->initialized = true;

  return OK;
}

/****************************************************************************
 * Name: ra8p_comparator_configure
 *
 * Description:
 *   Configure a comparator
 *
 ****************************************************************************/

int ra8p_comparator_configure(uint8_t comp_num,
                              struct ra8p_comparator_config_s *config)
{
  struct ra8p_comparator_dev_s *priv = &g_comp_priv;
  uint32_t regval;

  if (!priv || !priv->initialized || !config || comp_num > 2)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Store configuration */

  priv->config[comp_num] = *config;

  /* Configure comparator based on number */

  if (comp_num == 0)
    {
      /* Configure comparator 0 */

      regval = (config->input << RA8P_COMP_COMPDR0_CMPSEL_SHIFT) |
               (config->reference << RA8P_COMP_COMPDR0_REFSEL_SHIFT);

      if (config->output_enable)
        {
          regval |= RA8P_COMP_COMPDR0_OUTEN;
        }

      if (config->interrupt_enable)
        {
          regval |= RA8P_COMP_COMPDR0_INTEN;
        }

      if (config->monitor_enable)
        {
          regval |= RA8P_COMP_COMPDR0_OUTMON;
        }

      /* Enable comparator */

      regval |= RA8P_COMP_COMPDR0_COMPON;

      ra8p_comp_putreg32(RA8P_COMP_COMPDR0_OFFSET, regval);
    }
  else
    {
      /* Configure comparators 1 and 2 */

      regval = 0;

      if (comp_num == 1)
        {
          regval = (config->input << RA8P_COMP_COMPDR1_CMP1SEL_SHIFT) |
                   (config->reference << RA8P_COMP_COMPDR1_REF1SEL_SHIFT);

          if (config->output_enable)
            {
              regval |= RA8P_COMP_COMPDR1_OUT1EN;
            }

          if (config->monitor_enable)
            {
              regval |= RA8P_COMP_COMPDR1_OUT1MON;
            }
        }
      else if (comp_num == 2)
        {
          regval = (config->input << RA8P_COMP_COMPDR1_CMP2SEL_SHIFT) |
                   (config->reference << RA8P_COMP_COMPDR1_REF2SEL_SHIFT);

          if (config->output_enable)
            {
              regval |= RA8P_COMP_COMPDR1_OUT2EN;
            }

          if (config->monitor_enable)
            {
              regval |= RA8P_COMP_COMPDR1_OUT2MON;
            }
        }

      ra8p_comp_putreg32(RA8P_COMP_COMPDR1_OFFSET, regval);
    }

  nxmutex_unlock(&priv->lock);
  return OK;
}

/****************************************************************************
 * Name: ra8p_comparator_get_output
 *
 * Description:
 *   Get comparator output state
 *
 ****************************************************************************/

bool ra8p_comparator_get_output(uint8_t comp_num)
{
  struct ra8p_comparator_dev_s *priv = &g_comp_priv;
  uint32_t status;

  if (!priv || !priv->initialized || comp_num > 2)
    {
      return false;
    }

  nxmutex_lock(&priv->lock);

  /* Read comparator status */

  status = ra8p_comp_getreg32(RA8P_COMP_COMPSTR_OFFSET);

  nxmutex_unlock(&priv->lock);

  /* Return output based on comparator number */

  if (comp_num == 0)
    {
      return (status & RA8P_COMP_COMPSTR_CMPOUT) != 0;
    }
  else if (comp_num == 1)
    {
      return (status & RA8P_COMP_COMPSTR_CMPOUT1) != 0;
    }
  else if (comp_num == 2)
    {
      return (status & RA8P_COMP_COMPSTR_CMPOUT2) != 0;
    }

  return false;
}

/****************************************************************************
 * Name: ra8p_comparator_interrupt
 *
 * Description:
 *   Comparator interrupt handler
 *
 ****************************************************************************/

static int ra8p_comparator_interrupt(int irq, void *context, void *arg)
{
  struct ra8p_comparator_dev_s *priv = (struct ra8p_comparator_dev_s *)arg;
  uint32_t status;

  if (!priv)
    {
      return OK;
    }

  /* Read status */

  status = ra8p_comp_getreg32(RA8P_COMP_COMPSTR_OFFSET);

  /* TODO: Handle interrupt based on configured callbacks */

  return OK;
}

#endif /* CONFIG_RA8P_COMPARATOR */