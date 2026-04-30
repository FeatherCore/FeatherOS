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
#include <stdbool.h>
#include <stdint.h>
#include <errno.h>

#include <nuttx/irq.h>
#include <nuttx/spinlock.h>

#include "arm_internal.h"
#include "hardware/stm32n6_gpio.h"
#include "hardware/stm32n6_rcc.h"
#include "stm32n6_gpio.h"

/****************************************************************************
 * Private Data
 ****************************************************************************/

static spinlock_t g_configgpio_lock = SP_UNLOCKED;

static const uintptr_t g_gpiobase[STM32N6_NGPIO] =
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
  0,
  0,
  STM32_GPION_BASE,
  STM32_GPIOO_BASE,
  STM32_GPIOP_BASE,
  STM32_GPIOQ_BASE,
};

static const uint32_t g_gpioclock[STM32N6_NGPIO] =
{
  RCC_AHB4ENR_GPIOAEN,
  RCC_AHB4ENR_GPIOBEN,
  RCC_AHB4ENR_GPIOCEN,
  RCC_AHB4ENR_GPIODEN,
  RCC_AHB4ENR_GPIOEEN,
  RCC_AHB4ENR_GPIOFEN,
  RCC_AHB4ENR_GPIOGEN,
  RCC_AHB4ENR_GPIOHEN,
  0,
  0,
  0,
  0,
  0,
  RCC_AHB4ENR_GPIONEN,
  RCC_AHB4ENR_GPIOOEN,
  RCC_AHB4ENR_GPIOPEN,
  RCC_AHB4ENR_GPIOQEN,
};

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static inline unsigned int stm32n6_gpioport(uint32_t pinset)
{
  return (pinset & GPIO_PORT_MASK) >> GPIO_PORT_SHIFT;
}

static int stm32n6_gpiobase(uint32_t pinset, uintptr_t *base)
{
  unsigned int port;

  port = stm32n6_gpioport(pinset);
  if (port >= STM32N6_NGPIO || g_gpiobase[port] == 0)
    {
      return -EINVAL;
    }

  *base = g_gpiobase[port];
  return OK;
}

static void stm32n6_gpio_clock_enable(unsigned int port)
{
  uint32_t regval;

  if (port >= STM32N6_NGPIO || g_gpioclock[port] == 0)
    {
      return;
    }

  regval = getreg32(STM32_RCC_AHB4ENR);
  regval |= g_gpioclock[port];
  putreg32(regval, STM32_RCC_AHB4ENR);
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
  unsigned int port;
  int ret;

  ret = stm32n6_gpiobase(cfgset, &base);
  if (ret < 0)
    {
      return ret;
    }

  port    = stm32n6_gpioport(cfgset);
  pin     = (cfgset & GPIO_PIN_MASK) >> GPIO_PIN_SHIFT;
  mode    = (cfgset & GPIO_MODE_MASK) >> GPIO_MODE_SHIFT;
  otype   = (cfgset & GPIO_OTYPE_MASK) >> GPIO_OTYPE_SHIFT;
  ospeed  = (cfgset & GPIO_SPEED_MASK) >> GPIO_SPEED_SHIFT;
  pupd    = (cfgset & GPIO_PUPD_MASK) >> GPIO_PUPD_SHIFT;
  alt     = (cfgset & GPIO_AF_MASK) >> GPIO_AF_SHIFT;

  flags = spin_lock_irqsave(&g_configgpio_lock);

  stm32n6_gpio_clock_enable(port);

  if (mode == (GPIO_OUTPUT >> GPIO_MODE_SHIFT))
    {
      stm32n6_gpiowrite(cfgset, (cfgset & GPIO_OUTPUT_SET) != 0);
    }

  shift = pin * 2;
  regval = getreg32(base + STM32_GPIO_MODER_OFFSET);
  regval &= ~(3 << shift);
  regval |= mode << shift;
  putreg32(regval, base + STM32_GPIO_MODER_OFFSET);

  regval = getreg32(base + STM32_GPIO_OTYPER_OFFSET);
  regval &= ~(1 << pin);
  regval |= otype << pin;
  putreg32(regval, base + STM32_GPIO_OTYPER_OFFSET);

  regval = getreg32(base + STM32_GPIO_OSPEEDR_OFFSET);
  regval &= ~(3 << shift);
  regval |= ospeed << shift;
  putreg32(regval, base + STM32_GPIO_OSPEEDR_OFFSET);

  regval = getreg32(base + STM32_GPIO_PUPDR_OFFSET);
  regval &= ~(3 << shift);
  regval |= pupd << shift;
  putreg32(regval, base + STM32_GPIO_PUPDR_OFFSET);

  if (mode == (GPIO_ALT >> GPIO_MODE_SHIFT))
    {
      shift = (pin & 7) * 4;
      regval = getreg32(base + STM32_GPIO_AFRL_OFFSET + (pin >> 3) * 4);
      regval &= ~(15 << shift);
      regval |= alt << shift;
      putreg32(regval, base + STM32_GPIO_AFRL_OFFSET + (pin >> 3) * 4);
    }

  spin_unlock_irqrestore(&g_configgpio_lock, flags);

  return OK;
}

int stm32n6_unconfiggpio(uint32_t cfgset)
{
  cfgset &= GPIO_PORT_MASK | GPIO_PIN_MASK;
  cfgset |= GPIO_ANALOG | GPIO_FLOAT;

  return stm32n6_configgpio(cfgset);
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

  pin = (pinset & GPIO_PIN_MASK) >> GPIO_PIN_SHIFT;

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

  pin = (pinset & GPIO_PIN_MASK) >> GPIO_PIN_SHIFT;
  regval = getreg32(base + STM32_GPIO_IDR_OFFSET);

  return (regval & (1 << pin)) != 0;
}
