/****************************************************************************
 * arch/arm/src/stm32u5/stm32u5_dma.c
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
#include <semaphore.h>
#include <assert.h>
#include <errno.h>
#include <debug.h>

#include <arch/board/board.h>

#include "arm_internal.h"
#include "chip.h"
#include "stm32.h"
#include "stm32_dma.h"
#include "hardware/stm32u5_gpdma.h"
#include "hardware/stm32u5_lpdma.h"
#include "stm32u5_dma.h"

#if defined(CONFIG_STM32U5_DMA)

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* This structure describes one DMA channel */

struct stm32u5_dma_channel_s
{
  uint8_t controller;      /* DMA controller (GPDMA1 or LPDMA1) */
  uint8_t channel;         /* DMA channel number */
  uint8_t inuse;           /* 1 if allocated */
  uint8_t tcdone;          /* 1 if transfer completed */
  uint8_t status;          /* Current status */
  sem_t exclsem;           /* Mutual exclusion semaphore */
  sem_t waitsem;           /* Used to wait for transfer completion */
  dma_callback_t callback; /* Transfer complete callback */
  void *arg;              /* Argument passed to the callback */
};

/****************************************************************************
 * Private Data
 ****************************************************************************/

/* GPDMA1 Channels (0-15) */

static struct stm32u5_dma_channel_s g_gpdma1_channels[16];

/* LPDMA1 Channels (0-3) */

static struct stm32u5_dma_channel_s g_lpdma1_channels[4];

/* Mutex for protecting DMA allocation */

static mutex_t g_dmalock = NXMUTEX_INITIALIZER;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32u5_dma_waitidle
 *
 * Description:
 *   Wait for DMA channel to be idle.
 *
 ****************************************************************************/

static void stm32u5_dma_waitidle(struct stm32u5_dma_channel_s *dmach)
{
  uint32_t regaddr;
  uint32_t regval;

  if (dmach->controller == DMA_GPDMA1)
    {
      regaddr = STM32U5_GPDMA_CH0_CSR + (dmach->channel * STM32U5_GPDMA_CH_OFFSET);
    }
  else
    {
      regaddr = STM32U5_LPDMA_CH0_CSR + (dmach->channel * STM32U5_LPDMA_CH_OFFSET);
    }

  /* Wait for channel to be ready */

  do
    {
      regval = getreg32(regaddr);
      up_udelay(1);
    }
  while ((regval & (GPDMA_CSR_TCBUSY | GPDMA_CSR_CHBUSY)) != 0);
}

/****************************************************************************
 * Name: stm32u5_dma_interrupt
 *
 * Description:
 *   DMA interrupt handler.
 *
 ****************************************************************************/

static int stm32u5_dma_interrupt(int irq, void *context, void *arg)
{
  struct stm32u5_dma_channel_s *dmach = (struct stm32u5_dma_channel_s *)arg;
  uint32_t regaddr;
  uint32_t regval;
  uint8_t status = 0;

  /* Get the status register */

  if (dmach->controller == DMA_GPDMA1)
    {
      regaddr = STM32U5_GPDMA_CH0_CSR + (dmach->channel * STM32U5_GPDMA_CH_OFFSET);
      regval = getreg32(regaddr);

      /* Check for transfer complete */

      if (regval & GPDMA_CSR_TCF)
        {
          status |= DMA_STATUS_TCIF;
        }

      /* Check for half transfer complete */

      if (regval & GPDMA_CSR_HTFF)
        {
          status |= DMA_STATUS_HTIF;
        }

      /* Check for transfer error */

      if (regval & (GPDMA_CSR_DTEF | GPDMA_CSR_TOF))
        {
          status |= DMA_STATUS_TEIF;
        }

      /* Clear flags */

      regaddr = STM32U5_GPDMA_CH0_CFCR + (dmach->channel * STM32U5_GPDMA_CH_OFFSET);
      putreg32(status, regaddr);
    }
  else
    {
      regaddr = STM32U5_LPDMA_CH0_CSR + (dmach->channel * STM32U5_LPDMA_CH_OFFSET);
      regval = getreg32(regaddr);

      /* Check for transfer complete */

      if (regval & LPDMA_CCSR_TCF)
        {
          status |= DMA_STATUS_TCIF;
        }

      /* Check for half transfer complete */

      if (regval & LPDMA_CCSR_HTFF)
        {
          status |= DMA_STATUS_HTIF;
        }

      /* Check for transfer error */

      if (regval & (LPDMA_CCSR_DTEF | LPDMA_CCSR_TOF))
        {
          status |= DMA_STATUS_TEIF;
        }

      /* Clear flags */

      regaddr = STM32U5_LPDMA_CH0_CFCR + (dmach->channel * STM32U5_LPDMA_CH_OFFSET);
      putreg32(status, regaddr);
    }

  /* Remember the status */

  dmach->status = status;
  dmach->tcdone = 1;

  /* Check if callback is registered */

  if (dmach->callback)
    {
      dmach->callback((DMA_HANDLE)dmach, status, dmach->arg);
    }

  /* Post semaphore to release any threads waiting for the transfer to complete */

  nxsem_post(&dmach->waitsem);

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32u5_dmachannel
 *
 * Description:
 *   Allocate a DMA channel.
 *
 ****************************************************************************/

DMA_HANDLE stm32u5_dmachannel(unsigned int controller, unsigned int channel)
{
  struct stm32u5_dma_channel_s *dmach = NULL;
  uint32_t regaddr;
  int ret;

  /* Get exclusive access to the DMA subsystem */

  ret = nxmutex_lock(&g_dmalock);
  if (ret < 0)
    {
      return NULL;
    }

  /* Select the appropriate channel array based on controller */

  if (controller == DMA_GPDMA1)
    {
      /* Validate channel number */

      if (channel >= 16)
        {
          goto errout_with_lock;
        }

      dmach = &g_gpdma1_channels[channel];

      /* Enable GPDMA1 clock */

      modifyreg32(STM32_RCC_AHB1ENR, 0, RCC_AHB1ENR_GPDMA1EN);
    }
  else if (controller == DMA_LPDMA1)
    {
      /* Validate channel number */

      if (channel >= 4)
        {
          goto errout_with_lock;
        }

      dmach = &g_lpdma1_channels[channel];

      /* Enable LPDMA1 clock */

      modifyreg32(STM32_RCC_AHB3ENR, 0, RCC_AHB3ENR_LPDMA1EN);
    }
  else
    {
      goto errout_with_lock;
    }

  /* Check if channel is already in use */

  if (dmach->inuse)
    {
      dmach = NULL;
      goto errout_with_lock;
    }

  /* Mark the channel as in use */

  dmach->inuse = 1;
  dmach->controller = controller;
  dmach->channel = channel;
  dmach->tcdone = 0;
  dmach->status = 0;
  dmach->callback = NULL;
  dmach->arg = NULL;

  /* Initialize semaphores */

  nxsem_init(&dmach->exclsem, 0, 1);
  nxsem_init(&dmach->waitsem, 0, 0);

  /* Configure interrupt for the channel */

  if (controller == DMA_GPDMA1)
    {
      /* GPDMA1 uses a range of IRQs based on channel */

      irq_attach(STM32_IRQ_GPDMA1CH0 + channel, stm32u5_dma_interrupt, dmach);
      up_enable_irq(STM32_IRQ_GPDMA1CH0 + channel);
    }
  else
    {
      /* LPDMA1 uses a range of IRQs based on channel */

      irq_attach(STM32_IRQ_LPDMA1CH0 + channel, stm32u5_dma_interrupt, dmach);
      up_enable_irq(STM32_IRQ_LPDMA1CH0 + channel);
    }

  /* Release lock and return */

  nxmutex_unlock(&g_dmalock);
  return (DMA_HANDLE)dmach;

errout_with_lock:
  nxmutex_unlock(&g_dmalock);
  return NULL;
}

/****************************************************************************
 * Name: stm32u5_dmafree
 *
 * Description:
 *   Free a DMA channel.
 *
 ****************************************************************************/

void stm32u5_dmafree(DMA_HANDLE handle)
{
  struct stm32u5_dma_channel_s *dmach = (struct stm32u5_dma_channel_s *)handle;
  uint32_t regaddr;

  DEBUGASSERT(dmach != NULL && dmach->inuse == 1);

  /* Stop any ongoing transfer */

  if (dmach->controller == DMA_GPDMA1)
    {
      regaddr = STM32U5_GPDMA_CH0_CCR + (dmach->channel * STM32U5_GPDMA_CH_OFFSET);
      modifyreg32(regaddr, GPDMA_CCR_EN, 0);  /* Disable channel */
    }
  else
    {
      regaddr = STM32U5_LPDMA_CH0_CCR + (dmach->channel * STM32U5_LPDMA_CH_OFFSET);
      modifyreg32(regaddr, LPDMA_CCR_EN, 0);  /* Disable channel */
    }

  /* Disable interrupts */

  if (dmach->controller == DMA_GPDMA1)
    {
      up_disable_irq(STM32_IRQ_GPDMA1CH0 + dmach->channel);
      irq_detach(STM32_IRQ_GPDMA1CH0 + dmach->channel);
    }
  else
    {
      up_disable_irq(STM32_IRQ_LPDMA1CH0 + dmach->channel);
      irq_detach(STM32_IRQ_LPDMA1CH0 + dmach->channel);
    }

  /* Mark the channel as free */

  dmach->inuse = 0;
  dmach->callback = NULL;
  dmach->arg = NULL;

  /* Destroy semaphores */

  nxsem_destroy(&dmach->exclsem);
  nxsem_destroy(&dmach->waitsem);
}

/****************************************************************************
 * Name: stm32u5_dmastart
 *
 * Description:
 *   Start a DMA transfer.
 *
 ****************************************************************************/

int stm32u5_dmastart(DMA_HANDLE handle, const struct stm32u5_dma_config_s *config,
                     uint32_t control)
{
  struct stm32u5_dma_channel_s *dmach = (struct stm32u5_dma_channel_s *)handle;
  uint32_t regaddr;
  uint32_t regval;

  DEBUGASSERT(dmach != NULL && config != NULL);

  /* Wait for any previous transfer to complete */

  stm32u5_dma_waitidle(dmach);

  /* Save the callback information */

  dmach->callback = config->callback;
  dmach->arg = config->arg;
  dmach->tcdone = 0;
  dmach->status = 0;

  /* Configure the channel based on controller */

  if (dmach->controller == DMA_GPDMA1)
    {
      /* GPDMA1 configuration */

      /* Set peripheral address */
      regaddr = STM32U5_GPDMA_CH0_CSAR + (dmach->channel * STM32U5_GPDMA_CH_OFFSET);
      putreg32(config->paddr, regaddr);

      /* Set memory address */
      regaddr = STM32U5_GPDMA_CH0_CDAR + (dmach->channel * STM32U5_GPDMA_CH_OFFSET);
      putreg32(config->maddr, regaddr);

      /* Set number of data to transfer */
      regaddr = STM32U5_GPDMA_CH0_CNDTR + (dmach->channel * STM32U5_GPDMA_CH_OFFSET);
      putreg32(config->nbytes, regaddr);

      /* Set control register */
      regaddr = STM32U5_GPDMA_CH0_CCR + (dmach->channel * STM32U5_GPDMA_CH_OFFSET);
      putreg32(control, regaddr);

      /* Enable interrupt */
      modifyreg32(regaddr, 0, GPDMA_CCR_EN | GPDMA_CCR_TRIGPOL_RISING);
    }
  else
    {
      /* LPDMA1 configuration */

      /* Set peripheral address */
      regaddr = STM32U5_LPDMA_CH0_CSAR + (dmach->channel * STM32U5_LPDMA_CH_OFFSET);
      putreg32(config->paddr, regaddr);

      /* Set memory address */
      regaddr = STM32U5_LPDMA_CH0_CDAR + (dmach->channel * STM32U5_LPDMA_CH_OFFSET);
      putreg32(config->maddr, regaddr);

      /* Set number of data to transfer */
      regaddr = STM32U5_LPDMA_CH0_CNDTR + (dmach->channel * STM32U5_LPDMA_CH_OFFSET);
      putreg32(config->nbytes, regaddr);

      /* Set control register */
      regaddr = STM32U5_LPDMA_CH0_CCR + (dmach->channel * STM32U5_LPDMA_CH_OFFSET);
      putreg32(control, regaddr);

      /* Enable interrupt */
      modifyreg32(regaddr, 0, LPDMA_CCR_EN);
    }

  return OK;
}

/****************************************************************************
 * Name: stm32u5_dmastop
 *
 * Description:
 *   Stop a DMA transfer.
 *
 ****************************************************************************/

void stm32u5_dmastop(DMA_HANDLE handle)
{
  struct stm32u5_dma_channel_s *dmach = (struct stm32u5_dma_channel_s *)handle;
  uint32_t regaddr;

  DEBUGASSERT(dmach != NULL);

  /* Disable the channel */

  if (dmach->controller == DMA_GPDMA1)
    {
      regaddr = STM32U5_GPDMA_CH0_CCR + (dmach->channel * STM32U5_GPDMA_CH_OFFSET);
      modifyreg32(regaddr, GPDMA_CCR_EN, 0);
    }
  else
    {
      regaddr = STM32U5_LPDMA_CH0_CCR + (dmach->channel * STM32U5_LPDMA_CH_OFFSET);
      modifyreg32(regaddr, LPDMA_CCR_EN, 0);
    }

  /* Mark as stopped */

  dmach->tcdone = 1;
  nxsem_post(&dmach->waitsem);
}

/****************************************************************************
 * Name: stm32u5_dmawait
 *
 * Description:
 *   Wait for a DMA transfer to complete.
 *
 ****************************************************************************/

int stm32u5_dmawait(DMA_HANDLE handle)
{
  struct stm32u5_dma_channel_s *dmach = (struct stm32u5_dma_channel_s *)handle;

  DEBUGASSERT(dmach != NULL);

  /* Wait for the transfer to complete */

  while (!dmach->tcdone)
    {
      nxsem_wait(&dmach->waitsem);
    }

  return dmach->status;
}

/****************************************************************************
 * Name: stm32u5_dmareset
 *
 * Description:
 *   Reset a DMA channel.
 *
 ****************************************************************************/

void stm32u5_dmareset(unsigned int controller, unsigned int channel)
{
  /* Implementation depends on hardware capabilities */
  /* This function would reset specific channel registers */

  struct stm32u5_dma_channel_s *dmach;

  if (controller == DMA_GPDMA1 && channel < 16)
    {
      dmach = &g_gpdma1_channels[channel];
    }
  else if (controller == DMA_LPDMA1 && channel < 4)
    {
      dmach = &g_lpdma1_channels[channel];
    }
  else
    {
      return; /* Invalid controller/channel */
    }

  /* If channel is in use, stop it first */
  if (dmach->inuse)
    {
      stm32u5_dmastop((DMA_HANDLE)dmach);
    }
}

#endif /* CONFIG_STM32U5_DMA *//****************************************************************************
 * Name: stm32u5_dmasuspend
 *
 * Description:
 *   Suspend a DMA transfer.
 *
 ****************************************************************************/

int stm32u5_dmasuspend(DMA_HANDLE handle)
{
  struct stm32u5_dma_channel_s *dmach = (struct stm32u5_dma_channel_s *)handle;
  uint32_t regaddr;
  uint32_t regval;
  volatile int timeout;

  DEBUGASSERT(dmach != NULL);

  /* Suspend the channel */

  if (dmach->controller == DMA_GPDMA1)
    {
      regaddr = STM32U5_GPDMA_CH0_CCR + (dmach->channel * STM32U5_GPDMA_CH_OFFSET);
      
      /* Write suspend bit */
      modifyreg32(regaddr, 0, GPDMA_CCR_SUSP);

      /* Wait for suspend flag */
      regaddr = STM32U5_GPDMA_CH0_CSR + (dmach->channel * STM32U5_GPDMA_CH_OFFSET);
      for (timeout = 1000; timeout > 0; timeout--)
        {
          regval = getreg32(regaddr);
          if (regval & GPDMA_CSR_SUSPF)
            {
              break;
            }
          up_udelay(10);
        }

      if (timeout == 0)
        {
          return -ETIMEDOUT;
        }
    }
  else
    {
      /* LPDMA1 does not support suspend in the same way */
      /* Just stop the channel temporarily */
      regaddr = STM32U5_LPDMA_CH0_CCR + (dmach->channel * STM32U5_LPDMA_CH_OFFSET);
      modifyreg32(regaddr, LPDMA_CCR_EN, 0);
    }

  return OK;
}

/****************************************************************************
 * Name: stm32u5_dmaresume
 *
 * Description:
 *   Resume a suspended DMA transfer.
 *
 ****************************************************************************/

int stm32u5_dmaresume(DMA_HANDLE handle)
{
  struct stm32u5_dma_channel_s *dmach = (struct stm32u5_dma_channel_s *)handle;
  uint32_t regaddr;

  DEBUGASSERT(dmach != NULL);

  /* Resume the channel */

  if (dmach->controller == DMA_GPDMA1)
    {
      regaddr = STM32U5_GPDMA_CH0_CCR + (dmach->channel * STM32U5_GPDMA_CH_OFFSET);
      
      /* Clear suspend bit to resume */
      modifyreg32(regaddr, GPDMA_CCR_SUSP, 0);
    }
  else
    {
      /* LPDMA1: re-enable the channel */
      regaddr = STM32U5_LPDMA_CH0_CCR + (dmach->channel * STM32U5_LPDMA_CH_OFFSET);
      modifyreg32(regaddr, 0, LPDMA_CCR_EN);
    }

  return OK;
}

/****************************************************************************
 * Name: stm32u5_dmagetstatus
 *
 * Description:
 *   Get the current status of a DMA transfer.
 *
 ****************************************************************************/

int stm32u5_dmagetstatus(DMA_HANDLE handle, uint32_t *remaining)
{
  struct stm32u5_dma_channel_s *dmach = (struct stm32u5_dma_channel_s *)handle;
  uint32_t regaddr;

  DEBUGASSERT(dmach != NULL);

  if (remaining == NULL)
    {
      return -EINVAL;
    }

  /* Get the remaining bytes to transfer */

  if (dmach->controller == DMA_GPDMA1)
    {
      regaddr = STM32U5_GPDMA_CH0_CNDTR + (dmach->channel * STM32U5_GPDMA_CH_OFFSET);
      *remaining = getreg32(regaddr);
    }
  else
    {
      regaddr = STM32U5_LPDMA_CH0_CNDTR + (dmach->channel * STM32U5_LPDMA_CH_OFFSET);
      *remaining = getreg32(regaddr);
    }

  return dmach->status;
}