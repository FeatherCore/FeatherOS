/****************************************************************************
 * arch/arm/src/stm32u5/stm32u5_dma.h
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

#ifndef __ARCH_ARM_SRC_STM32U5_STM32U5_DMA_H
#define __ARCH_ARM_SRC_STM32U5_STM32U5_DMA_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <sys/types.h>

/********************************************************************************
 * Pre-processor Definitions
 *******************************************************************************/

/* DMA register bit definitions */

#define DMA_CHAN_CCR_EN            (1 << 0)   /* Bit 0: Channel enable */
#define DMA_CHAN_CCR_TCIE         (1 << 1)   /* Bit 1: Transfer complete interrupt enable */
#define DMA_CHAN_CCR_HTIE         (1 << 2)   /* Bit 2: Half transfer interrupt enable */
#define DMA_CHAN_CCR_TEIE         (1 << 3)   /* Bit 3: Transfer error interrupt enable */
#define DMA_CHAN_CCR_DIR          (1 << 4)   /* Bit 4: Direction */
#define DMA_CHAN_CCR_CIRC         (1 << 5)   /* Bit 5: Circular mode */
#define DMA_CHAN_CCR_PINC         (1 << 6)   /* Bit 6: Peripheral increment mode */
#define DMA_CHAN_CCR_MINC         (1 << 7)   /* Bit 7: Memory increment mode */
#define DMA_CHAN_CCR_PSIZE_8B     (0 << 8)   /* Bits 8-9: Peripheral size 8-bit */
#define DMA_CHAN_CCR_PSIZE_16B     (1 << 8)   /* Bits 8-9: Peripheral size 16-bit */
#define DMA_CHAN_CCR_PSIZE_32B     (2 << 8)   /* Bits 8-9: Peripheral size 32-bit */
#define DMA_CHAN_CCR_MSIZE_8B      (0 << 10)  /* Bits 10-11: Memory size 8-bit */
#define DMA_CHAN_CCR_MSIZE_16B     (1 << 10)  /* Bits 10-11: Memory size 16-bit */
#define DMA_CHAN_CCR_MSIZE_32B     (2 << 10)  /* Bits 10-11: Memory size 32-bit */
#define DMA_CHAN_CCR_PL_LOW        (0 << 12)  /* Bits 12-13: Priority low */
#define DMA_CHAN_CCR_PL_MEDIUM     (1 << 12)  /* Bits 12-13: Priority medium */
#define DMA_CHAN_CCR_PL_HIGH       (2 << 12)  /* Bits 12-13: Priority high */
#define DMA_CHAN_CCR_PL_VERYHIGH   (3 << 12)  /* Bits 12-13: Priority very high */
#define DMA_CHAN_CCR_MEM2MEM      (1 << 14)  /* Bit 14: Memory to memory mode */

/* DMA status flags */

#define DMA_STATUS_FEIF          (1 << 0)   /* Bit 0: FIFO error interrupt flag */
#define DMA_STATUS_DMEIF        (1 << 2)   /* Bit 2: Direct mode error interrupt flag */
#define DMA_STATUS_TEIF         (1 << 3)   /* Bit 3: Transfer error interrupt flag */
#define DMA_STATUS_HTIF         (1 << 4)   /* Bit 4: Half transfer interrupt flag */
#define DMA_STATUS_TCIF         (1 << 5)   /* Bit 5: Transfer complete interrupt flag */

/* DMA error status */

#define DMA_STATUS_ERROR        (DMA_STATUS_FEIF | DMA_STATUS_DMEIF | DMA_STATUS_TEIF)
#define DMA_STATUS_SUCCESS      (DMA_STATUS_TCIF | DMA_STATUS_HTIF)

/* DMA channel configuration values */

#define DMA_CHANNEL0            (0)
#define DMA_CHANNEL1            (1)
#define DMA_CHANNEL2            (2)
#define DMA_CHANNEL3            (3)
#define DMA_CHANNEL4            (4)
#define DMA_CHANNEL5            (5)
#define DMA_CHANNEL6            (6)
#define DMA_CHANNEL7            (7)
#define DMA_CHANNEL8            (8)
#define DMA_CHANNEL9            (9)
#define DMA_CHANNEL10           (10)
#define DMA_CHANNEL11           (11)
#define DMA_CHANNEL12           (12)
#define DMA_CHANNEL13           (13)
#define DMA_CHANNEL14           (14)
#define DMA_CHANNEL15           (15)

#define DMA_GPDMA1             (0)  /* GPDMA1 controller index */
#define DMA_LPDMA1             (1)  /* LPDMA1 controller index */

/****************************************************************************
 * Public Types
 *****************************************************************************/

/* DMA_HANDLE provides an opaque reference that can be used to represent a
 * DMA channel.
 */

typedef void *DMA_HANDLE;

/* Description:
 *   This is the type of the callback that is used to inform the user of the
 *   completion of the DMA.
 *
 * Input Parameters:
 *   handle - Refers to the DMA channel
 *   status - A bit encoded value that provides the completion status.
 *            See the DMA_STATUS_* definitions above.
 *   arg    - A user-provided value that was provided when
 *            stm32u5_dmastart() was called.
 */

typedef void (*dma_callback_t)(DMA_HANDLE handle, uint8_t status, void *arg);

/* Description:
 *   This structure defines the DMA transaction.
 *
 * Input Parameters:
 *   callback - DMA callback function
 *   arg     - User-provided callback argument
 *   paddr   - Physical address of the peripheral data register
 *   maddr   - Memory buffer address
 *   nbytes  - Number of bytes to transfer
 */

struct stm32u5_dma_config_s
{
  dma_callback_t callback;  /* DMA callback */
  void *arg;             /* User-provided callback argument */
  uint32_t paddr;       /* Peripheral address */
  uint32_t maddr;       /* Memory buffer address */
  size_t nbytes;        /* Number of bytes to transfer */
};

/****************************************************************************
 * Public Data
 *****************************************************************************/

#ifndef __ASSEMBLY__

#undef EXTERN
#if defined(__cplusplus)
#define EXTERN extern "C"
extern "C"
{
#else
#define EXTERN extern
#endif

/****************************************************************************
 * Public Function Prototypes
 *****************************************************************************/

/****************************************************************************
 * Name: stm32u5_dmastart
 *
 * Description:
 *   Start a DMA transfer.
 *
 * Input Parameters:
 *   handle  - DMA channel handle
 *   config  - DMA transfer configuration
 *   control - DMA channel control register value
 *
 * Returned Value:
 *   Zero on success; a negated errno value on failure.
 *
 ****************************************************************************/

int stm32u5_dmastart(DMA_HANDLE handle, const struct stm32u5_dma_config_s *config,
                     uint32_t control);

/****************************************************************************
 * Name: stm32u5_dmastop
 *
 * Description:
 *   Stop a DMA transfer.
 *
 * Input Parameters:
 *   handle - DMA channel handle
 *
 * Returned Value:
 *   None.
 *
 ****************************************************************************/

void stm32u5_dmastop(DMA_HANDLE handle);

/****************************************************************************
 * Name: stm32u5_dmareset
 *
 * Description:
 *   Reset a DMA channel.
 *
 * Input Parameters:
 *   controller - DMA controller index (GPDMA1 or LPDMA1)
 *   channel    - DMA channel number
 *
 * Returned Value:
 *   None.
 *
 ****************************************************************************/

void stm32u5_dmareset(unsigned int controller, unsigned int channel);

/****************************************************************************
 * Name: stm32u5_dmachannel
 *
 * Description:
 *   Allocate a DMA channel.
 *
 * Input Parameters:
 *   controller - DMA controller index (GPDMA1 or LPDMA1)
 *   channel    - DMA channel number
 *
 * Returned Value:
 *   DMA channel handle on success; NULL on failure.
 *
 ****************************************************************************/

DMA_HANDLE stm32u5_dmachannel(unsigned int controller, unsigned int channel);

/****************************************************************************
 * Name: stm32u5_dmafree
 *
 * Description:
 *   Free a DMA channel.
 *
 * Input Parameters:
 *   handle - DMA channel handle
 *
 * Returned Value:
 *   None.
 *
 ****************************************************************************/

void stm32u5_dmafree(DMA_HANDLE handle);

/****************************************************************************
 * Name: stm32u5_dmawait
 *
 * Description:
 *   Wait for a DMA transfer to complete.
 *
 * Input Parameters:
 *   handle - DMA channel handle
 *
 * Returned Value:
 *   DMA status on success; a negated errno value on failure.
 *
 ****************************************************************************/

int stm32u5_dmawait(DMA_HANDLE handle);

/****************************************************************************
 * Name: stm32u5_dmasample
 *
 * Description:
 *   Sample DMA register contents.
 *
 * Input Parameters:
 *   handle - DMA channel handle
 *   regs   - Buffer to store register contents
 *
 * Returned Value:
 *   None.
 *
 ****************************************************************************/

#ifdef CONFIG_DEBUG_DMA_INFO
void stm32u5_dmasample(DMA_HANDLE handle, struct stm32u5_dmaregs_s *regs);
#endif

/****************************************************************************
 * Name: stm32u5_dmadump
 *
 * Description:
 *   Dump DMA register contents.
 *
 * Input Parameters:
 *   handle - DMA channel handle
 *   regs   - Buffer with register contents
 *   name   - Name to identify this DMA channel
 *
 * Returned Value:
 *   None.
 *
 ****************************************************************************/

#ifdef CONFIG_DEBUG_DMA_INFO
void stm32u5_dmadump(DMA_HANDLE handle, const struct stm32u5_dmaregs_s *regs,
                     const char *name);
#endif

#undef EXTERN
#if defined(__cplusplus)
}
#endif

#endif /* __ASSEMBLY__ */
#endif /* __ARCH_ARM_SRC_STM32U5_STM32U5_DMA_H */