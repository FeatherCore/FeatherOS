/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_gpio.c
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
#include <errno.h>
#include <debug.h>

#include <nuttx/irq.h>
#include <nuttx/spinlock.h>

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_gpio.h"
#include "hardware/stm32n6_rcc.h"
#include "stm32n6_gpio.h"

/****************************************************************************
 * Private Data
 ****************************************************************************/

static spinlock_t g_configgpio_lock = SP_UNLOCKED;

/****************************************************************************
 * Public Data
 ****************************************************************************/

const uint32_t g_gpiobase[STM32N6_NGPIO] =
{
  STM32_GPIOA_BASE,
  STM32_GPIOB_BASE,
  STM32_GPIOC_BASE,
  STM32_GPIOD_BASE,
  STM32_GPIOE_BASE,
  STM32_GPIOF_BASE,
  STM32_GPIOG_BASE,
  STM32_GPIOH_BASE,
  0,
  0,
  0,
  STM32_GPION_BASE,
  STM32_GPIOO_BASE,
  STM32_GPIOP_BASE,
  STM32_GPIOQ_BASE,
};

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static inline int stm32n6_gpiobase(uint32_t pinset, uintptr_t *base)
{
  int port = (pinset >> 4) & 0xf;

  if (port >= STM32N6_NGPIO || g_gpiobase[port] == 0)
    {
      return -EINVAL;
    }

  *base = g_gpiobase[port];
  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

void stm32n6_gpioinit(void)
{
}

int stm32n6_configgpio(uint32_t cfgset)
{
  uintptr_t base;
  uint32_t pin;
  uint32_t mode;
  uint32_t otype;
  uint32_t ospeed;
  uint32_t pupd;
  uint32_t alt;
  uint32_t regval;
  uint32_t shift;
  irqstate_t flags;
  int ret;

  ret = stm32n6_gpiobase(cfgset, &base);
  if (ret < 0)
    {
      return ret;
    }

  pin     = cfgset & 0xf;
  mode    = (cfgset >> 8) & 0x3;
  otype   = (cfgset >> 10) & 0x1;
  ospeed  = (cfgset >> 11) & 0x3;
  pupd    = (cfgset >> 13) & 0x3;
  alt     = (cfgset >> 16) & 0xf;

  flags = spin_lock_irqsave(&g_configgpio_lock);

  regval = getreg32(STM32_RCC_AHB4ENR);
  regval |= (1 << ((cfgset >> 4) & 0xf));
  putreg32(regval, STM32_RCC_AHB4ENR);

  shift = pin * 2;
  regval = getreg32(base + STM32_GPIO_MODER_OFFSET);
  regval &= ~(3 << shift);
  regval |= (mode << shift);
  putreg32(regval, base + STM32_GPIO_MODER_OFFSET);

  regval = getreg32(base + STM32_GPIO_OTYPER_OFFSET);
  regval &= ~(1 << pin);
  regval |= (otype << pin);
  putreg32(regval, base + STM32_GPIO_OTYPER_OFFSET);

  shift = pin * 2;
  regval = getreg32(base + STM32_GPIO_OSPEEDR_OFFSET);
  regval &= ~(3 << shift);
  regval |= (ospeed << shift);
  putreg32(regval, base + STM32_GPIO_OSPEEDR_OFFSET);

  shift = pin * 2;
  regval = getreg32(base + STM32_GPIO_PUPDR_OFFSET);
  regval &= ~(3 << shift);
  regval |= (pupd << shift);
  putreg32(regval, base + STM32_GPIO_PUPDR_OFFSET);

  if (mode == GPIO_MODE_ALTERNATE)
    {
      shift = (pin & 0x7) * 4;
      regval = getreg32(base + STM32_GPIO_AFRL_OFFSET + (pin >> 3) * 4);
      regval &= ~(0xf << shift);
      regval |= (alt << shift);
      putreg32(regval, base + STM32_GPIO_AFRL_OFFSET + (pin >> 3) * 4);
    }

  spin_unlock_irqrestore(&g_configgpio_lock, flags);

  return OK;
}

void stm32n6_gpiowrite(uint32_t pinset, bool value)
{
  uintptr_t base;
  uint32_t pin;
  uint32_t regval;

  if (stm32n6_gpiobase(pinset, &base) < 0)
    {
      return;
    }

  pin = pinset & 0xf;

  if (value)
    {
      regval = 1 << pin;
    }
  else
    {
      regval = 1 << (pin + 16);
    }

  putreg32(regval, base + STM32_GPIO_BSRR_OFFSET);
}

bool stm32n6_gpioread(uint32_t pinset)
{
  uintptr_t base;
  uint32_t pin;
  uint32_t regval;

  if (stm32n6_gpiobase(pinset, &base) < 0)
    {
      return false;
    }

  pin = pinset & 0xf;
  regval = getreg32(base + STM32_GPIO_IDR_OFFSET);

  return (regval & (1 << pin)) != 0;
}
