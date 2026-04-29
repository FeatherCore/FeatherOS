/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_flash.c
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

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_memorymap.h"
#include "stm32n6_flash.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define FLASH_BASE               0x08000000
#define FLASH_ACR_OFFSET         0x00
#define FLASH_KEYR_OFFSET       0x08
#define FLASH_OPTKEYR_OFFSET     0x0C
#define FLASH_CR_OFFSET         0x10
#define FLASH_SR_OFFSET         0x14
#define FLASH_CCR_OFFSET        0x18
#define FLASH_ECC_FAEENR_OFFSET 0x30

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static inline uint32_t stm32n6_flash_getreg(uint32_t offset)
{
  return getreg32(FLASH_BASE + offset);
}

static inline void stm32n6_flash_putreg(uint32_t offset, uint32_t value)
{
  putreg32(value, FLASH_BASE + offset);
}

static void stm32n6_flash_waitidle(void)
{
  while ((stm32n6_flash_getreg(FLASH_SR_OFFSET) & FLASH_SR_BSY) != 0);
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

void stm32n6_flash_init(void)
{
  uint32_t regval;

  regval = stm32n6_flash_getreg(FLASH_ACR_OFFSET);
  regval |= (1 << 0);
  stm32n6_flash_putreg(FLASH_ACR_OFFSET, regval);
}

int stm32n6_flash_unlock(void)
{
  if ((stm32n6_flash_getreg(FLASH_CR_OFFSET) & FLASH_CR_LOCK) == 0)
    {
      return OK;
    }

  stm32n6_flash_putreg(FLASH_KEYR_OFFSET, FLASH_KEY1);
  stm32n6_flash_putreg(FLASH_KEYR_OFFSET, FLASH_KEY2);

  if ((stm32n6_flash_getreg(FLASH_CR_OFFSET) & FLASH_CR_LOCK) != 0)
    {
      return -EACCES;
    }

  return OK;
}

int stm32n6_flash_lock(void)
{
  uint32_t regval;

  stm32n6_flash_waitidle();

  regval = stm32n6_flash_getreg(FLASH_CR_OFFSET);
  regval |= FLASH_CR_LOCK;
  stm32n6_flash_putreg(FLASH_CR_OFFSET, regval);

  return OK;
}

int stm32n6_flash_erase_page(uint32_t pageaddr)
{
  uint32_t regval;

  if (stm32n6_flash_unlock() != OK)
    {
      return -EACCES;
    }

  stm32n6_flash_waitidle();

  regval = stm32n6_flash_getreg(FLASH_CR_OFFSET);
  regval |= FLASH_CR_PER;
  stm32n6_flash_putreg(FLASH_CR_OFFSET, regval);

  regval = stm32n6_flash_getreg(FLASH_CR_OFFSET);
  regval |= (pageaddr & ~0x7f);
  stm32n6_flash_putreg(FLASH_CR_OFFSET, regval);

  regval = stm32n6_flash_getreg(FLASH_CR_OFFSET);
  regval |= FLASH_CR_STRT;
  stm32n6_flash_putreg(FLASH_CR_OFFSET, regval);

  stm32n6_flash_waitidle();

  regval = stm32n6_flash_getreg(FLASH_CR_OFFSET);
  regval &= ~FLASH_CR_PER;
  stm32n6_flash_putreg(FLASH_CR_OFFSET, regval);

  stm32n6_flash_lock();

  if ((stm32n6_flash_getreg(FLASH_SR_OFFSET) &
       (FLASH_SR_WRPERR | FLASH_SR_PGSERR | FLASH_SR_STRBERR)) != 0)
    {
      return -EIO;
    }

  return OK;
}

int stm32n6_flash_program(uint32_t addr, uint32_t data)
{
  uint32_t regval;

  if (stm32n6_flash_unlock() != OK)
    {
      return -EACCES;
    }

  stm32n6_flash_waitidle();

  regval = stm32n6_flash_getreg(FLASH_CR_OFFSET);
  regval |= FLASH_CR_PG;
  stm32n6_flash_putreg(FLASH_CR_OFFSET, regval);

  putreg32(data, addr);

  stm32n6_flash_waitidle();

  regval = stm32n6_flash_getreg(FLASH_CR_OFFSET);
  regval &= ~FLASH_CR_PG;
  stm32n6_flash_putreg(FLASH_CR_OFFSET, regval);

  stm32n6_flash_lock();

  return OK;
}

int stm32n6_flash_read(uint32_t addr, uint32_t *data)
{
  if (data == NULL)
    {
      return -EINVAL;
    }

  *data = getreg32(addr);
  return OK;
}
