/****************************************************************************
 * arch/arm/src/stm32u5/stm32u5_lptim.c
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

#include <assert.h>
#include <errno.h>
#include <debug.h>

#include <arch/board/board.h>

#include "stm32.h"
#include "stm32_gpio.h"
#include "stm32_rcc.h"
#include "hardware/stm32u5_lptim.h"

#if defined(CONFIG_STM32U5_LPTIM1) || defined(CONFIG_STM32U5_LPTIM2) || \
    defined(CONFIG_STM32U5_LPTIM3) || defined(CONFIG_STM32U5_LPTIM4)

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct stm32u5_lptim_priv_s
{
  uint8_t  lptim_id;
  uint32_t base;
  uint8_t  mode;
};

/****************************************************************************
 * Private Data
 ****************************************************************************/

#if defined(CONFIG_STM32U5_LPTIM1)
static struct stm32u5_lptim_priv_s stm32u5_lptim1_priv =
{
  .lptim_id = 1,
  .base = STM32_LPTIM1_BASE,
  .mode = LPTIM_MODE_UNUSED,
};
#endif

#if defined(CONFIG_STM32U5_LPTIM2)
static struct stm32u5_lptim_priv_s stm32u5_lptim2_priv =
{
  .lptim_id = 2,
  .base = STM32_LPTIM2_BASE,
  .mode = LPTIM_MODE_UNUSED,
};
#endif

#if defined(CONFIG_STM32U5_LPTIM3)
static struct stm32u5_lptim_priv_s stm32u5_lptim3_priv =
{
  .lptim_id = 3,
  .base = STM32_LPTIM3_BASE,
  .mode = LPTIM_MODE_UNUSED,
};
#endif

#if defined(CONFIG_STM32U5_LPTIM4)
static struct stm32u5_lptim_priv_s stm32u5_lptim4_priv =
{
  .lptim_id = 4,
  .base = STM32_LPTIM4_BASE,
  .mode = LPTIM_MODE_UNUSED,
};
#endif

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32u5_lptim_getpriv
 ****************************************************************************/

static struct stm32u5_lptim_priv_s *stm32u5_lptim_getpriv(int timer)
{
  switch (timer)
    {
#if defined(CONFIG_STM32U5_LPTIM1)
      case 1:
        return &stm32u5_lptim1_priv;
#endif
#if defined(CONFIG_STM32U5_LPTIM2)
      case 2:
        return &stm32u5_lptim2_priv;
#endif
#if defined(CONFIG_STM32U5_LPTIM3)
      case 3:
        return &stm32u5_lptim3_priv;
#endif
#if defined(CONFIG_STM32U5_LPTIM4)
      case 4:
        return &stm32u5_lptim4_priv;
#endif
      default:
        return NULL;
    }
}

/****************************************************************************
 * Name: stm32u5_lptim_enableclock
 ****************************************************************************/

static void stm32u5_lptim_enableclock(int timer)
{
#if defined(CONFIG_STM32U5_LPTIM1)
  if (timer == 1)
    {
      modifyreg32(STM32_RCC_APB3ENR, 0, RCC_APB3ENR_LPTIM1EN);
      return;
    }
#endif

#if defined(CONFIG_STM32U5_LPTIM2)
  if (timer == 2)
    {
      modifyreg32(STM32_RCC_APB1ENR1, 0, RCC_APB1ENR1_LPTIM2EN);
      return;
    }
#endif

#if defined(CONFIG_STM32U5_LPTIM3)
  if (timer == 3)
    {
      modifyreg32(STM32_RCC_APB3ENR, 0, RCC_APB3ENR_LPTIM3EN);
      return;
    }
#endif

#if defined(CONFIG_STM32U5_LPTIM4)
  if (timer == 4)
    {
      modifyreg32(STM32_RCC_APB3ENR, 0, RCC_APB3ENR_LPTIM4EN);
      return;
    }
#endif
}

/****************************************************************************
 * Name: stm32u5_lptim_disableclock
 ****************************************************************************/

static void stm32u5_lptim_disableclock(int timer)
{
#if defined(CONFIG_STM32U5_LPTIM1)
  if (timer == 1)
    {
      modifyreg32(STM32_RCC_APB3ENR, RCC_APB3ENR_LPTIM1EN, 0);
      return;
    }
#endif

#if defined(CONFIG_STM32U5_LPTIM2)
  if (timer == 2)
    {
      modifyreg32(STM32_RCC_APB1ENR1, RCC_APB1ENR1_LPTIM2EN, 0);
      return;
    }
#endif

#if defined(CONFIG_STM32U5_LPTIM3)
  if (timer == 3)
    {
      modifyreg32(STM32_RCC_APB3ENR, RCC_APB3ENR_LPTIM3EN, 0);
      return;
    }
#endif

#if defined(CONFIG_STM32U5_LPTIM4)
  if (timer == 4)
    {
      modifyreg32(STM32_RCC_APB3ENR, RCC_APB3ENR_LPTIM4EN, 0);
      return;
    }
#endif
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32u5_lptiminitialize
 ****************************************************************************/

struct stm32u5_lptim_dev_s *stm32u5_lptiminitialize(int timer)
{
  struct stm32u5_lptim_priv_s *priv;

  lptiminfo("LPTIM%" PRIu32 " initialization\n", timer);

  priv = stm32u5_lptim_getpriv(timer);
  if (priv == NULL)
    {
      lptimerr("ERROR: Invalid LPTIM%" PRIu32 "\n", timer);
      return NULL;
    }

  if (priv->mode != LPTIM_MODE_UNUSED)
    {
      lptimerr("ERROR: LPTIM%" PRIu32 " already initialized\n", timer);
      return NULL;
    }

  stm32u5_lptim_enableclock(timer);

  priv->mode = LPTIM_MODE_COUNTER;

  return (struct stm32u5_lptim_dev_s *)priv;
}

/****************************************************************************
 * Name: stm32u5_lptimuninitialize
 ****************************************************************************/

void stm32u5_lptimuninitialize(struct stm32u5_lptim_dev_s *dev)
{
  struct stm32u5_lptim_priv_s *priv = (struct stm32u5_lptim_priv_s *)dev;

  DEBUGASSERT(dev != NULL);

  stm32u5_lptim_disableclock(priv->lptim_id);

  priv->mode = LPTIM_MODE_UNUSED;
}

/****************************************************************************
 * Name: stm32u5_lptimenable
 ****************************************************************************/

int stm32u5_lptimenable(struct stm32u5_lptim_dev_s *dev)
{
  struct stm32u5_lptim_priv_s *priv = (struct stm32u5_lptim_priv_s *)dev;
  uint32_t regval;

  DEBUGASSERT(dev != NULL);

  regval = getreg32(priv->base + STM32U5_LPTIM_CR_OFFSET);
  regval |= LPTIM_CR_ENABLE;
  putreg32(regval, priv->base + STM32U5_LPTIM_CR_OFFSET);

  return OK;
}

/****************************************************************************
 * Name: stm32u5_lptimdisable
 ****************************************************************************/

int stm32u5_lptimdisable(struct stm32u5_lptim_dev_s *dev)
{
  struct stm32u5_lptim_priv_s *priv = (struct stm32u5_lptim_priv_s *)dev;
  uint32_t regval;

  DEBUGASSERT(dev != NULL);

  regval = getreg32(priv->base + STM32U5_LPTIM_CR_OFFSET);
  regval &= ~LPTIM_CR_ENABLE;
  putreg32(regval, priv->base + STM32U5_LPTIM_CR_OFFSET);

  return OK;
}

/****************************************************************************
 * Name: stm32u5_lptimsetclock
 ****************************************************************************/

int stm32u5_lptimsetclock(struct stm32u5_lptim_dev_s *dev,
                              enum stm32u5_lptim_clksrc_e clksrc)
{
  struct stm32u5_lptim_priv_s *priv = (struct stm32u5_lptim_priv_s *)dev;
  uint32_t regval;

  DEBUGASSERT(dev != NULL);

  regval = getreg32(priv->base + STM32U5_LPTIM_CFGR_OFFSET);
  regval &= ~LPTIM_CFGR_CKSEL;

  if (clksrc == LPTIM_CLKSRC_EXTERNAL)
    {
      regval |= LPTIM_CFGR_CKSEL;
    }

  putreg32(regval, priv->base + STM32U5_LPTIM_CFGR_OFFSET);

  return OK;
}

/****************************************************************************
 * Name: stm32u5_lptimsetprescaler
 ****************************************************************************/

int stm32u5_lptimsetprescaler(struct stm32u5_lptim_dev_s *dev,
                                   uint8_t prescaler)
{
  struct stm32u5_lptim_priv_s *priv = (struct stm32u5_lptim_priv_s *)dev;
  uint32_t regval;

  DEBUGASSERT(dev != NULL);

  if (prescaler > 7)
    {
      return -EINVAL;
    }

  regval = getreg32(priv->base + STM32U5_LPTIM_CFGR_OFFSET);
  regval &= ~LPTIM_CFGR_PRESC_MASK;
  regval |= (prescaler << LPTIM_CFGR_PRESC_SHIFT);
  putreg32(regval, priv->base + STM32U5_LPTIM_CFGR_OFFSET);

  return OK;
}

/****************************************************************************
 * Name: stm32u5_lptimsetperiod
 ****************************************************************************/

int stm32u5_lptimsetperiod(struct stm32u5_lptim_dev_s *dev,
                             uint16_t period)
{
  struct stm32u5_lptim_priv_s *priv = (struct stm32u5_lptim_priv_s *)dev;

  DEBUGASSERT(dev != NULL);

  putreg32(period, priv->base + STM32U5_LPTIM_ARR_OFFSET);

  return OK;
}

/****************************************************************************
 * Name: stm32u5_lptimgetcounter
 ****************************************************************************/

uint16_t stm32u5_lptimgetcounter(struct stm32u5_lptim_dev_s *dev)
{
  struct stm32u5_lptim_priv_s *priv = (struct stm32u5_lptim_priv_s *)dev;
  uint32_t cnt;

  DEBUGASSERT(dev != NULL);

  cnt = getreg32(priv->base + STM32U5_LPTIM_CNT_OFFSET);

  return (uint16_t)(cnt & 0xffff);
}

/****************************************************************************
 * Name: stm32u5_lptimsetcompare
 ****************************************************************************/

int stm32u5_lptimsetcompare(struct stm32u5_lptim_dev_s *dev,
                              uint16_t cmp)
{
  struct stm32u5_lptim_priv_s *priv = (struct stm32u5_lptim_priv_s *)dev;

  DEBUGASSERT(dev != NULL);

  putreg32(cmp, priv->base + STM32U5_LPTIM_CMP_OFFSET);

  return OK;
}

/****************************************************************************
 * Name: stm32u5_lptimstart
 ****************************************************************************/

int stm32u5_lptimstart(struct stm32u5_lptim_dev_s *dev,
                         enum stm32u5_lptim_mode_e mode)
{
  struct stm32u5_lptim_priv_s *priv = (struct stm32u5_lptim_priv_s *)dev;
  uint32_t regval;

  DEBUGASSERT(dev != NULL);

  regval = getreg32(priv->base + STM32U5_LPTIM_CR_OFFSET);

  if (mode == LPTIM_MODE_SINGLE)
    {
      regval |= LPTIM_CR_SNGSTRT;
    }
  else
    {
      regval |= LPTIM_CR_CNTSTRT;
    }

  putreg32(regval, priv->base + STM32U5_LPTIM_CR_OFFSET);

  return OK;
}

/****************************************************************************
 * Name: stm32u5_lptimstop
 ****************************************************************************/

int stm32u5_lptimstop(struct stm32u5_lptim_dev_s *dev)
{
  struct stm32u5_lptim_priv_s *priv = (struct stm32u5_lptim_priv_s *)dev;
  uint32_t regval;

  DEBUGASSERT(dev != NULL);

  regval = getreg32(priv->base + STM32U5_LPTIM_CR_OFFSET);
  regval &= ~(LPTIM_CR_SNGSTRT | LPTIM_CR_CNTSTRT);
  putreg32(regval, priv->base + STM32U5_LPTIM_CR_OFFSET);

  return OK;
}

#endif /* CONFIG_STM32U5_LPTIM1 || CONFIG_STM32U5_LPTIM2 || ... */