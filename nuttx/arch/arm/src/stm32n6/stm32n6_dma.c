/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_dma.c
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

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_memorymap.h"
#include "hardware/stm32n6_rcc.h"
#include "stm32n6_dma.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_DMA_ISR_OFFSET        0x00
#define STM32_DMA_IFCR_OFFSET       0x04

#define STM32_DMA_CCR_OFFSET(ch)    (0x08 + (ch) * 0x14)
#define STM32_DMA_CNDTR_OFFSET(ch)  (0x0C + (ch) * 0x14)
#define STM32_DMA_CPAR_OFFSET(ch)   (0x10 + (ch) * 0x14)
#define STM32_DMA_CMAR_OFFSET(ch)   (0x14 + (ch) * 0x14)

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static inline void stm32n6_dma_putreg(uintptr_t dmabase,
                                      uint32_t offset, uint32_t value)
{
  putreg32(value, dmabase + offset);
}

static inline uint32_t stm32n6_dma_getreg(uintptr_t dmabase,
                                          uint32_t offset)
{
  return getreg32(dmabase + offset);
}

static void stm32n6_dma_enable_clock(uintptr_t dmabase)
{
  uint32_t regval;

  regval = getreg32(STM32_RCC_AHB1ENR);
  if (dmabase == STM32_GPDMA1_BASE)
    {
      regval |= RCC_AHB1ENR_GPDMA1EN;
    }
  putreg32(regval, STM32_RCC_AHB1ENR);
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int stm32n6_dma_initialize(uintptr_t dmabase)
{
  stm32n6_dma_enable_clock(dmabase);
  return OK;
}

int stm32n6_dma_configure(uintptr_t dmabase, uint8_t channel,
                          uint32_t periph_addr, uint32_t mem_addr,
                          uint16_t ndata, uint8_t direction,
                          uint8_t periph_size, uint8_t mem_size,
                          uint8_t priority, bool periph_inc, bool mem_inc,
                          bool circular)
{
  uint32_t regval;

  if (channel >= DMA_CHANNEL_COUNT)
    {
      return -EINVAL;
    }

  stm32n6_dma_stop(dmabase, channel);

  stm32n6_dma_putreg(dmabase, STM32_DMA_CPAR_OFFSET(channel), periph_addr);
  stm32n6_dma_putreg(dmabase, STM32_DMA_CMAR_OFFSET(channel), mem_addr);
  stm32n6_dma_putreg(dmabase, STM32_DMA_CNDTR_OFFSET(channel), ndata);

  regval = 0;
  if (direction)
    {
      regval |= DMA_CCR_DIR;
    }
  if (circular)
    {
      regval |= DMA_CCR_CIRC;
    }
  if (periph_inc)
    {
      regval |= DMA_CCR_PINC;
    }
  if (mem_inc)
    {
      regval |= DMA_CCR_MINC;
    }

  regval |= (periph_size << DMA_CCR_PSIZE_SHIFT) & DMA_CCR_PSIZE_MASK;
  regval |= (mem_size << DMA_CCR_MSIZE_SHIFT) & DMA_CCR_MSIZE_MASK;
  regval |= (priority << DMA_CCR_PL_SHIFT) & DMA_CCR_PL_MASK;

  regval |= DMA_CCR_TCIE | DMA_CCR_TEIE;

  stm32n6_dma_putreg(dmabase, STM32_DMA_CCR_OFFSET(channel), regval);

  return OK;
}

void stm32n6_dma_start(uintptr_t dmabase, uint8_t channel)
{
  uint32_t regval;

  regval = stm32n6_dma_getreg(dmabase, STM32_DMA_CCR_OFFSET(channel));
  regval |= DMA_CCR_EN;
  stm32n6_dma_putreg(dmabase, STM32_DMA_CCR_OFFSET(channel), regval);
}

void stm32n6_dma_stop(uintptr_t dmabase, uint8_t channel)
{
  uint32_t regval;

  regval = stm32n6_dma_getreg(dmabase, STM32_DMA_CCR_OFFSET(channel));
  regval &= ~DMA_CCR_EN;
  stm32n6_dma_putreg(dmabase, STM32_DMA_CCR_OFFSET(channel), regval);
}

bool stm32n6_dma_complete(uintptr_t dmabase, uint8_t channel)
{
  uint32_t regval;
  uint32_t shift;

  shift = channel * 4;
  regval = stm32n6_dma_getreg(dmabase, STM32_DMA_ISR_OFFSET);

  return (regval & (DMA_ISR_TCIF_SHIFT << shift)) != 0;
}

void stm32n6_dma_clear_interrupt(uintptr_t dmabase, uint8_t channel)
{
  uint32_t shift;

  shift = channel * 4;
  stm32n6_dma_putreg(dmabase, STM32_DMA_IFCR_OFFSET,
                     (DMA_IFCR_CGIF | DMA_IFCR_CTCIF | DMA_IFCR_CHTIF |
                      DMA_IFCR_CTEIF) << shift);
}
