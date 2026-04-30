/****************************************************************************
 * arch/arm/src/ra8p/ra8p_icu.c
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

#include <stdint.h>
#include <stdbool.h>

#include "arm_internal.h"
#include "hardware/ra8p_icu.h"
#include "hardware/ra8p_memorymap.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* ICU base address */
#define ICU_BASE        RA8P_ICU_BASE

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_ext_irq_s
{
  ra8p_ext_irq_callback_t callback;
  void *arg;
  bool enabled;
};

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_ext_irq_s g_ext_irq[16];

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_icu_init
 *
 * Description:
 *   Initialize the ICU (Interrupt Controller Unit).
 *   Based on Zephyr RA8P1 ICU initialization.
 *
 ****************************************************************************/

void ra8p_icu_init(void)
{
  int i;

  /* Initialize all external interrupt channels */
  for (i = 0; i < 16; i++)
    {
      g_ext_irq[i].callback = NULL;
      g_ext_irq[i].arg = NULL;
      g_ext_irq[i].enabled = false;
    }
}

/****************************************************************************
 * Name: ra8p_icu_configure_irq
 *
 * Description:
 *   Configure an external interrupt channel.
 *   Based on Zephyr gpio_ra_interrupt_set implementation.
 *
 ****************************************************************************/

int ra8p_icu_configure_irq(uint8_t channel, 
                           enum ra8p_ext_irq_trigger_e trigger,
                           bool filter, uint8_t sample_div)
{
  uint8_t irqcr;
  uintptr_t irqcr_addr;

  if (channel >= 16)
    {
      return -EINVAL;
    }

  /* Calculate IRQCR address */
  irqcr_addr = ICU_BASE + RA8P_ICU_IRQCR_OFFSET(channel);

  /* Read current IRQCR value */
  irqcr = getreg8(irqcr_addr);

  /* Clear IRQMD bits */
  irqcr &= ~RA8P_ICU_IRQCR_IRQMD_MASK;

  /* Set trigger mode */
  irqcr |= (trigger << RA8P_ICU_IRQCR_IRQMD_SHIFT);

  /* Set digital filter */
  if (filter)
    {
      irqcr |= RA8P_ICU_IRQCR_FLTEN;
    }
  else
    {
      irqcr &= ~RA8P_ICU_IRQCR_FLTEN;
    }

  /* Write IRQCR value */
  putreg8(irqcr, irqcr_addr);

  return OK;
}

/****************************************************************************
 * Name: ra8p_icu_enable_irq
 *
 * Description:
 *   Enable an external interrupt channel.
 *
 ****************************************************************************/

void ra8p_icu_enable_irq(uint8_t channel)
{
  uint8_t irqcr;
  uintptr_t irqcr_addr;

  if (channel >= 16)
    {
      return;
    }

  /* Calculate IRQCR address */
  irqcr_addr = ICU_BASE + RA8P_ICU_IRQCR_OFFSET(channel);

  /* Read current IRQCR value */
  irqcr = getreg8(irqcr_addr);

  /* Enable IRQ */
  irqcr |= RA8P_ICU_IRQCR_ISEL;

  /* Write IRQCR value */
  putreg8(irqcr, irqcr_addr);

  g_ext_irq[channel].enabled = true;
}

/****************************************************************************
 * Name: ra8p_icu_disable_irq
 *
 * Description:
 *   Disable an external interrupt channel.
 *
 ****************************************************************************/

void ra8p_icu_disable_irq(uint8_t channel)
{
  uint8_t irqcr;
  uintptr_t irqcr_addr;

  if (channel >= 16)
    {
      return;
    }

  /* Calculate IRQCR address */
  irqcr_addr = ICU_BASE + RA8P_ICU_IRQCR_OFFSET(channel);

  /* Read current IRQCR value */
  irqcr = getreg8(irqcr_addr);

  /* Disable IRQ */
  irqcr &= ~RA8P_ICU_IRQCR_ISEL;

  /* Write IRQCR value */
  putreg8(irqcr, irqcr_addr);

  g_ext_irq[channel].enabled = false;
}

/****************************************************************************
 * Name: ra8p_icu_clear_irq
 *
 * Description:
 *   Clear an external interrupt flag.
 *
 ****************************************************************************/

void ra8p_icu_clear_irq(uint8_t channel)
{
  uintptr_t ielsr_addr;
  uint32_t ielsr;

  if (channel >= 16)
    {
      return;
    }

  /* Calculate IELSR address */
  ielsr_addr = ICU_BASE + RA8P_ICU_IELSR_OFFSET(channel);

  /* Read IELSR value */
  ielsr = getreg32(ielsr_addr);

  /* Write back to clear the IR bit */
  putreg32(ielsr, ielsr_addr);
}

/****************************************************************************
 * Name: ra8p_icu_set_callback
 *
 * Description:
 *   Set callback for an external interrupt.
 *
 ****************************************************************************/

int ra8p_icu_set_callback(uint8_t channel, 
                          ra8p_ext_irq_callback_t callback,
                          void *arg)
{
  if (channel >= 16)
    {
      return -EINVAL;
    }

  g_ext_irq[channel].callback = callback;
  g_ext_irq[channel].arg = arg;

  return OK;
}

/****************************************************************************
 * Name: ra8p_icu_irq_handler
 *
 * Description:
 *   Common IRQ handler for external interrupts.
 *   Based on Zephyr gpio_ra_isr implementation.
 *
 ****************************************************************************/

void ra8p_icu_irq_handler(uint8_t channel)
{
  if (channel >= 16)
    {
      return;
    }

  /* Clear interrupt flag */
  ra8p_icu_clear_irq(channel);

  /* Call user callback */
  if (g_ext_irq[channel].callback)
    {
      g_ext_irq[channel].callback(channel, g_ext_irq[channel].arg);
    }
}