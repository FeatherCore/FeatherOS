/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_dma.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_DMA_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_DMA_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <stdint.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define DMA_CHANNEL_COUNT        16

#define DMA_CCR_EN               (1 << 0)
#define DMA_CCR_TCIE             (1 << 1)
#define DMA_CCR_HTIE             (1 << 2)
#define DMA_CCR_TEIE             (1 << 3)
#define DMA_CCR_DIR              (1 << 4)
#define DMA_CCR_CIRC             (1 << 5)
#define DMA_CCR_PINC             (1 << 6)
#define DMA_CCR_MINC             (1 << 7)
#define DMA_CCR_PSIZE_SHIFT      8
#define DMA_CCR_PSIZE_MASK       (3 << DMA_CCR_PSIZE_SHIFT)
#define DMA_CCR_MSIZE_SHIFT      10
#define DMA_CCR_MSIZE_MASK       (3 << DMA_CCR_MSIZE_SHIFT)
#define DMA_CCR_PL_SHIFT         12
#define DMA_CCR_PL_MASK          (3 << DMA_CCR_PL_SHIFT)
#define DMA_CCR_MEM2MEM          (1 << 14)

#define DMA_CNDTR_NDT_SHIFT      0
#define DMA_CNDTR_NDT_MASK       (0xffff << DMA_CNDTR_NDT_SHIFT)

#define DMA_ISR_GIF_SHIFT        0
#define DMA_ISR_TCIF_SHIFT       1
#define DMA_ISR_HTIF_SHIFT       2
#define DMA_ISR_TEIF_SHIFT       3

#define DMA_IFCR_CGIF            (1 << 0)
#define DMA_IFCR_CTCIF           (1 << 1)
#define DMA_IFCR_CHTIF           (1 << 2)
#define DMA_IFCR_CTEIF           (1 << 3)

#define DMA_SIZE_8BIT            0
#define DMA_SIZE_16BIT           1
#define DMA_SIZE_32BIT           2

#define DMA_PRIO_LOW             0
#define DMA_PRIO_MEDIUM          1
#define DMA_PRIO_HIGH            2
#define DMA_PRIO_VERYHIGH        3

/****************************************************************************
 * Public Types
 ****************************************************************************/

struct stm32n6_dma_channel_s
{
  uintptr_t dmabase;
  uint8_t channel;
  uint8_t direction;
  uint8_t periph_size;
  uint8_t mem_size;
  uint8_t priority;
  bool periph_inc;
  bool mem_inc;
  bool circular;
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

int stm32n6_dma_initialize(uintptr_t dmabase);
int stm32n6_dma_configure(uintptr_t dmabase, uint8_t channel,
                          uint32_t periph_addr, uint32_t mem_addr,
                          uint16_t ndata, uint8_t direction,
                          uint8_t periph_size, uint8_t mem_size,
                          uint8_t priority, bool periph_inc, bool mem_inc,
                          bool circular);
void stm32n6_dma_start(uintptr_t dmabase, uint8_t channel);
void stm32n6_dma_stop(uintptr_t dmabase, uint8_t channel);
bool stm32n6_dma_complete(uintptr_t dmabase, uint8_t channel);
void stm32n6_dma_clear_interrupt(uintptr_t dmabase, uint8_t channel);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_DMA_H */
