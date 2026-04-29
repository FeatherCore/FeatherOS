/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_venc.c
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

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_memorymap.h"
#include "hardware/stm32n6_rcc.h"
#include "stm32n6_venc.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define VENC_TIMEOUT_US              5000000  /* 5 seconds timeout */

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static inline void stm32n6_venc_putreg(uintptr_t vencbase,
                                       uint32_t offset, uint32_t value)
{
  putreg32(value, vencbase + offset);
}

static inline uint32_t stm32n6_venc_getreg(uintptr_t vencbase,
                                           uint32_t offset)
{
  return getreg32(vencbase + offset);
}

static void stm32n6_venc_enable_clock(void)
{
  uint32_t regval;

  regval = getreg32(STM32_RCC_AHB5ENR);
  regval |= (1 << 5);  /* VENCEN */
  putreg32(regval, STM32_RCC_AHB5ENR);
}

static int stm32n6_venc_wait_for_notbusy(uintptr_t vencbase)
{
  uint32_t timeout = VENC_TIMEOUT_US;

  while ((stm32n6_venc_getreg(vencbase, VENC_SR_OFFSET) & VENC_SR_BUSY) != 0)
    {
      if (timeout-- == 0)
        {
          return -ETIMEDOUT;
        }
      up_udelay(1);
    }

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int stm32n6_venc_initialize(uintptr_t vencbase, uint8_t format, uint8_t codec, 
                            uint16_t width, uint16_t height)
{
  uint32_t regval;

  stm32n6_venc_enable_clock();

  /* Disable VENC before configuration */
  stm32n6_venc_disable(vencbase);

  /* Configure format and dimensions */
  regval = (format & 0x7) << 8;  /* Set format */
  regval |= (codec & 0x1) << 12;  /* Set codec */
  stm32n6_venc_putreg(vencbase, VENC_CTR1_OFFSET, regval);

  /* Configure width and height */
  stm32n6_venc_putreg(vencbase, VENC_CTR2_OFFSET, width);
  stm32n6_venc_putreg(vencbase, VENC_CTR3_OFFSET, height);

  /* Configure other parameters */
  stm32n6_venc_putreg(vencbase, VENC_CTR4_OFFSET, 0x100);  /* Default bitrate */
  stm32n6_venc_putreg(vencbase, VENC_CTR5_OFFSET, 0x50);   /* Default quality */

  /* Enable interrupts */
  stm32n6_venc_putreg(vencbase, VENC_IER_OFFSET, VENC_IER_EOCIE | VENC_IER_ERRIE);

  return OK;
}

void stm32n6_venc_enable(uintptr_t vencbase)
{
  uint32_t regval;

  regval = stm32n6_venc_getreg(vencbase, VENC_CR_OFFSET);
  regval |= VENC_CR_EN;
  stm32n6_venc_putreg(vencbase, VENC_CR_OFFSET, regval);
}

void stm32n6_venc_disable(uintptr_t vencbase)
{
  uint32_t regval;

  regval = stm32n6_venc_getreg(vencbase, VENC_CR_OFFSET);
  regval &= ~VENC_CR_EN;
  stm32n6_venc_putreg(vencbase, VENC_CR_OFFSET, regval);
}

int stm32n6_venc_encode_frame(uintptr_t vencbase, const uint8_t *input, 
                              size_t input_size, uint8_t *output, 
                              size_t *output_size)
{
  uint32_t regval;
  size_t input_idx = 0;
  size_t output_idx = 0;
  uint32_t timeout;

  /* Wait for not busy */
  if (stm32n6_venc_wait_for_notbusy(vencbase) != OK)
    {
      return -ETIMEDOUT;
    }

  /* Enable DMA channels */
  regval = stm32n6_venc_getreg(vencbase, VENC_CR_OFFSET);
  regval |= VENC_CR_DMAINEN | VENC_CR_DMAOUTEN;
  stm32n6_venc_putreg(vencbase, VENC_CR_OFFSET, regval);

  /* Configure frame size */
  stm32n6_venc_putreg(vencbase, VENC_CTR6_OFFSET, input_size);

  /* Start encoding */
  regval = stm32n6_venc_getreg(vencbase, VENC_CR_OFFSET);
  regval |= VENC_CR_START;
  stm32n6_venc_putreg(vencbase, VENC_CR_OFFSET, regval);

  /* Wait for encoding to complete */
  timeout = VENC_TIMEOUT_US;
  while ((stm32n6_venc_getreg(vencbase, VENC_ISR_OFFSET) & VENC_ISR_EOCF) == 0)
    {
      if (timeout-- == 0)
        {
          return -ETIMEDOUT;
        }
      up_udelay(1);
    }

  /* Clear end of conversion flag */
  stm32n6_venc_putreg(vencbase, VENC_ICR_OFFSET, VENC_ICR_EOCF);

  /* Since we're using DMA for actual processing, we just indicate completion */
  *output_size = 0;  /* This would be updated with actual compressed size */

  return OK;
}

void stm32n6_venc_start(uintptr_t vencbase)
{
  uint32_t regval;

  regval = stm32n6_venc_getreg(vencbase, VENC_CR_OFFSET);
  regval |= VENC_CR_START;
  stm32n6_venc_putreg(vencbase, VENC_CR_OFFSET, regval);
}

void stm32n6_venc_stop(uintptr_t vencbase)
{
  uint32_t regval;

  regval = stm32n6_venc_getreg(vencbase, VENC_CR_OFFSET);
  regval &= ~VENC_CR_START;
  stm32n6_venc_putreg(vencbase, VENC_CR_OFFSET, regval);
}

void stm32n6_venc_suspend(uintptr_t vencbase)
{
  uint32_t regval;

  regval = stm32n6_venc_getreg(vencbase, VENC_CR_OFFSET);
  regval |= VENC_CR_SUSP;
  stm32n6_venc_putreg(vencbase, VENC_CR_OFFSET, regval);
}

void stm32n6_venc_resume(uintptr_t vencbase)
{
  uint32_t regval;

  regval = stm32n6_venc_getreg(vencbase, VENC_CR_OFFSET);
  regval &= ~VENC_CR_SUSP;
  stm32n6_venc_putreg(vencbase, VENC_CR_OFFSET, regval);
}

bool stm32n6_venc_busy(uintptr_t vencbase)
{
  return (stm32n6_venc_getreg(vencbase, VENC_SR_OFFSET) & VENC_SR_BUSY) != 0;
}