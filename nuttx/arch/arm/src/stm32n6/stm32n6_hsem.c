/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_hsem.c
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

#include <stdint.h>
#include <stdbool.h>
#include <debug.h>

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_memorymap.h"
#include "stm32n6_hsem.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_HSEM_BASE            (STM32N6_PERIPH_BASE + 0x00802C000)

#define HSEM_R_OFFSET(n)           (0x000 + (n) * 4)
#define HSEM_R_LOCK_SHIFT          31
#define HSEM_R_COREID_SHIFT        8
#define HSEM_R_COREID_MASK         (0xF << HSEM_R_COREID_SHIFT)
#define HSEM_R_PROCID_SHIFT        0
#define HSEM_R_PROCID_MASK         (0xFF << HSEM_R_PROCID_SHIFT)

#define HSEM_CR_OFFSET             0x080
#define HSEM_CR_COREID_SHIFT       8
#define HSEM_CR_COREID_MASK        (0xF << HSEM_CR_COREID_SHIFT)

#define HSEM_KEYR_OFFSET(n)        (0x100 + (n) * 4)
#define HSEM_KEYR_KEY              0xC8A5

#define HSEM_C1IER_OFFSET          0x200
#define HSEM_C1ICR_OFFSET          0x204
#define HSEM_C1ISR_OFFSET          0x208
#define HSEM_C1MISR_OFFSET         0x20C

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static inline uint32_t stm32n6_hsem_getreg(uint32_t offset)
{
  return getreg32(STM32_HSEM_BASE + offset);
}

static inline void stm32n6_hsem_putreg(uint32_t offset, uint32_t value)
{
  putreg32(value, STM32_HSEM_BASE + offset);
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int stm32n6_hsem_initialize(void)
{
  return OK;
}

void stm32n6_hsem_lock(uint32_t hsem, uint32_t retry)
{
  uint32_t regval;

  if (hsem >= HSEM_MAX_SEMAPHORES)
    {
      return;
    }

  do
    {
      regval = HSEM_KEYR_KEY | (HSEM_CR_COREID_CPU1 << HSEM_CR_COREID_SHIFT);
      stm32n6_hsem_putreg(HSEM_KEYR_OFFSET(hsem), regval);

      regval = stm32n6_hsem_getreg(HSEM_R_OFFSET(hsem));
      if ((regval & (1 << HSEM_R_LOCK_SHIFT)) == 0)
        {
          return;
        }

      if (retry != HSEM_LOCK_WAIT_FOREVER)
        {
          retry--;
          if (retry == 0)
            {
              return;
            }
        }
    }
  while (1);
}

int stm32n6_hsem_try_lock(uint32_t hsem)
{
  uint32_t regval;

  if (hsem >= HSEM_MAX_SEMAPHORES)
    {
      return -1;
    }

  regval = HSEM_KEYR_KEY | (HSEM_CR_COREID_CPU1 << HSEM_CR_COREID_SHIFT);
  stm32n6_hsem_putreg(HSEM_KEYR_OFFSET(hsem), regval);

  regval = stm32n6_hsem_getreg(HSEM_R_OFFSET(hsem));
  if ((regval & (1 << HSEM_R_LOCK_SHIFT)) != 0)
    {
      return -1;
    }

  return 0;
}

void stm32n6_hsem_unlock(uint32_t hsem)
{
  uint32_t regval;

  if (hsem >= HSEM_MAX_SEMAPHORES)
    {
      return;
    }

  regval = stm32n6_hsem_getreg(HSEM_R_OFFSET(hsem));
  regval &= ~HSEM_R_COREID_MASK;
  regval |= (HSEM_CR_COREID_CPU1 << HSEM_R_COREID_SHIFT);
  stm32n6_hsem_putreg(HSEM_KEYR_OFFSET(hsem), regval);
}

bool stm32n6_hsem_is_owned(uint32_t hsem)
{
  uint32_t regval;

  if (hsem >= HSEM_MAX_SEMAPHORES)
    {
      return false;
    }

  regval = stm32n6_hsem_getreg(HSEM_R_OFFSET(hsem));
  return ((regval >> HSEM_R_COREID_SHIFT) & 0xF) == HSEM_CR_COREID_CPU1;
}