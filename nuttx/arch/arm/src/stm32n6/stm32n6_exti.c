/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_exti.c
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
#include <debug.h>
#include <errno.h>

#include <nuttx/irq.h>
#include <arch/irq.h>

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_memorymap.h"
#include "stm32n6_exti.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define EXTI_IMR1_OFFSET          0x00
#define EXTI_EMR1_OFFSET          0x20
#define EXTI_RTSR1_OFFSET         0x40
#define EXTI_FTSR1_OFFSET         0x60
#define EXTI_SWIER1_OFFSET        0x80
#define EXTI_PR1_OFFSET           0xA0
#define EXTI_IMR2_OFFSET          0x04
#define EXTI_EMR2_OFFSET          0x24
#define EXTI_RTSR2_OFFSET         0x44
#define EXTI_FTSR2_OFFSET         0x64
#define EXTI_SWIER2_OFFSET        0x84
#define EXTI_PR2_OFFSET           0xA4
#define EXTI_IMR3_OFFSET          0x08
#define EXTI_EMR3_OFFSET          0x28
#define EXTI_RTSR3_OFFSET         0x48
#define EXTI_FTSR3_OFFSET         0x68
#define EXTI_SWIER3_OFFSET        0x88
#define EXTI_PR3_OFFSET           0xA8

/****************************************************************************
 * Private Data
 ****************************************************************************/

static exti_handler_t g_exti_handlers[EXTI_MAX_LINES];
static void *g_exti_args[EXTI_MAX_LINES];

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static inline uint32_t stm32n6_exti_getreg(uint32_t offset)
{
  return getreg32(STM32_EXTI_BASE + offset);
}

static inline void stm32n6_exti_putreg(uint32_t offset, uint32_t value)
{
  putreg32(value, STM32_EXTI_BASE + offset);
}

static int stm32n6_exti_isr(int irq, void *context, void *arg)
{
  uint32_t pending;
  uint32_t line;

  pending = stm32n6_exti_getreg(EXTI_PR1_OFFSET);
  for (line = 0; line < 32 && pending; line++)
    {
      if (pending & (1 << line))
        {
          stm32n6_exti_putreg(EXTI_PR1_OFFSET, 1 << line);
          if (g_exti_handlers[line])
            {
              g_exti_handlers[line](line, g_exti_args[line]);
            }

          pending &= ~(1 << line);
        }
    }

  pending = stm32n6_exti_getreg(EXTI_PR2_OFFSET);
  for (line = 32; line < 64 && pending; line++)
    {
      if (pending & (1 << (line - 32)))
        {
          stm32n6_exti_putreg(EXTI_PR2_OFFSET, 1 << (line - 32));
          if (g_exti_handlers[line])
            {
              g_exti_handlers[line](line, g_exti_args[line]);
            }

          pending &= ~(1 << (line - 32));
        }
    }

  pending = stm32n6_exti_getreg(EXTI_PR3_OFFSET);
  for (line = 64; line < 96 && pending; line++)
    {
      if (pending & (1 << (line - 64)))
        {
          stm32n6_exti_putreg(EXTI_PR3_OFFSET, 1 << (line - 64));
          if (g_exti_handlers[line])
            {
              g_exti_handlers[line](line, g_exti_args[line]);
            }

          pending &= ~(1 << (line - 64));
        }
    }

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int stm32n6_exti_initialize(void)
{
  uint32_t regval;

  regval = getreg32(STM32_RCC_APB4ENR);
  regval |= (1 << 0);
  putreg32(regval, STM32_RCC_APB4ENR);

  stm32n6_exti_putreg(EXTI_IMR1_OFFSET, 0);
  stm32n6_exti_putreg(EXTI_IMR2_OFFSET, 0);
  stm32n6_exti_putreg(EXTI_IMR3_OFFSET, 0);

  irq_attach(STM32_IRQ_EXTI0, stm32n6_exti_isr, NULL);
  irq_attach(STM32_IRQ_EXTI1, stm32n6_exti_isr, NULL);
  irq_attach(STM32_IRQ_EXTI2, stm32n6_exti_isr, NULL);
  irq_attach(STM32_IRQ_EXTI3, stm32n6_exti_isr, NULL);
  irq_attach(STM32_IRQ_EXTI4, stm32n6_exti_isr, NULL);

  up_enable_irq(STM32_IRQ_EXTI0);
  up_enable_irq(STM32_IRQ_EXTI1);
  up_enable_irq(STM32_IRQ_EXTI2);
  up_enable_irq(STM32_IRQ_EXTI3);
  up_enable_irq(STM32_IRQ_EXTI4);

  return OK;
}

int stm32n6_exti_configure(uint32_t line, uint32_t trigger, uint32_t mode)
{
  uint32_t offset;
  uint32_t bit;

  if (line >= EXTI_MAX_LINES)
    {
      return -EINVAL;
    }

  if (line < 32)
    {
      offset = EXTI_RTSR1_OFFSET;
      bit = 1 << line;
    }
  else if (line < 64)
    {
      offset = EXTI_RTSR2_OFFSET;
      bit = 1 << (line - 32);
    }
  else
    {
      offset = EXTI_RTSR3_OFFSET;
      bit = 1 << (line - 64);
    }

  if (trigger & EXTI_RISING_EDGE)
    {
      stm32n6_exti_putreg(offset, stm32n6_exti_getreg(offset) | bit);
    }
  else
    {
      stm32n6_exti_putreg(offset, stm32n6_exti_getreg(offset) & ~bit);
    }

  offset += 0x20;
  if (trigger & EXTI_FALLING_EDGE)
    {
      stm32n6_exti_putreg(offset, stm32n6_exti_getreg(offset) | bit);
    }
  else
    {
      stm32n6_exti_putreg(offset, stm32n6_exti_getreg(offset) & ~bit);
    }

  return OK;
}

int stm32n6_exti_enable(uint32_t line)
{
  uint32_t offset;
  uint32_t bit;

  if (line >= EXTI_MAX_LINES)
    {
      return -EINVAL;
    }

  if (line < 32)
    {
      offset = EXTI_IMR1_OFFSET;
      bit = 1 << line;
    }
  else if (line < 64)
    {
      offset = EXTI_IMR2_OFFSET;
      bit = 1 << (line - 32);
    }
  else
    {
      offset = EXTI_IMR3_OFFSET;
      bit = 1 << (line - 64);
    }

  stm32n6_exti_putreg(offset, stm32n6_exti_getreg(offset) | bit);

  return OK;
}

int stm32n6_exti_disable(uint32_t line)
{
  uint32_t offset;
  uint32_t bit;

  if (line >= EXTI_MAX_LINES)
    {
      return -EINVAL;
    }

  if (line < 32)
    {
      offset = EXTI_IMR1_OFFSET;
      bit = 1 << line;
    }
  else if (line < 64)
    {
      offset = EXTI_IMR2_OFFSET;
      bit = 1 << (line - 32);
    }
  else
    {
      offset = EXTI_IMR3_OFFSET;
      bit = 1 << (line - 64);
    }

  stm32n6_exti_putreg(offset, stm32n6_exti_getreg(offset) & ~bit);

  return OK;
}

int stm32n6_exti_register(uint32_t line, exti_handler_t handler, void *arg)
{
  if (line >= EXTI_MAX_LINES)
    {
      return -EINVAL;
    }

  g_exti_handlers[line] = handler;
  g_exti_args[line] = arg;

  return OK;
}

void stm32n6_exti_clear(uint32_t line)
{
  uint32_t offset;
  uint32_t bit;

  if (line >= EXTI_MAX_LINES)
    {
      return;
    }

  if (line < 32)
    {
      offset = EXTI_PR1_OFFSET;
      bit = 1 << line;
    }
  else if (line < 64)
    {
      offset = EXTI_PR2_OFFSET;
      bit = 1 << (line - 32);
    }
  else
    {
      offset = EXTI_PR3_OFFSET;
      bit = 1 << (line - 64);
    }

  stm32n6_exti_putreg(offset, bit);
}

bool stm32n6_exti_pending(uint32_t line)
{
  uint32_t offset;
  uint32_t bit;

  if (line >= EXTI_MAX_LINES)
    {
      return false;
    }

  if (line < 32)
    {
      offset = EXTI_PR1_OFFSET;
      bit = 1 << line;
    }
  else if (line < 64)
    {
      offset = EXTI_PR2_OFFSET;
      bit = 1 << (line - 32);
    }
  else
    {
      offset = EXTI_PR3_OFFSET;
      bit = 1 << (line - 64);
    }

  return (stm32n6_exti_getreg(offset) & bit) != 0;
}
