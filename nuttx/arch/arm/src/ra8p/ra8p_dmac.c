/****************************************************************************
 * arch/arm/src/ra8p/ra8p_dmac.c
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
#include <errno.h>
#include <debug.h>

#include "chip.h"
#include "hardware/ra8p_dmac.h"
#include "hardware/ra8p_memorymap.h"

#ifdef CONFIG_RA8P_HAVE_DMA

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define RA8P_DMAC_BASE            RA8P_DMAC_BASE
#define RA8P_DMAC_COMMON_BASE     RA8P_DMAC_COMMON_BASE

/* DMA transfer sizes */
#define DMAC_TRANSFER_SIZE_1     0   /* 1 byte */
#define DMAC_TRANSFER_SIZE_2     1   /* 2 bytes */
#define DMAC_TRANSFER_SIZE_4     2   /* 4 bytes */
#define DMAC_TRANSFER_SIZE_8     3   /* 8 bytes */
#define DMAC_TRANSFER_SIZE_16    4   /* 16 bytes */
#define DMAC_TRANSFER_SIZE_32    5   /* 32 bytes */
#define DMAC_TRANSFER_SIZE_64    6   /* 64 bytes */
#define DMAC_TRANSFER_SIZE_128   7   /* 128 bytes */
#define DMAC_TRANSFER_SIZE_256   8   /* 256 bytes */

/* DMA address modes */
#define DMAC_ADDR_MODE_FIXED      0   /* Fixed address */
#define DMAC_ADDR_MODE_INCREMENT   1   /* Increment address */
#define DMAC_ADDR_MODE_DECREMENT   2   /* Decrement address */

/* DMA request select values for RA8P1 */
#define DMAC_REQSCI0_TXI0       0x00
#define DMAC_REQSCI0_RXI0       0x01
#define DMAC_REQSCI1_TXI1       0x02
#define DMAC_REQSCI1_RXI1       0x03
#define DMAC_REQSPI0            0x04
#define DMAC_REQSPI1            0x05
#define DMAC_REQIIC0_TXI0       0x06
#define DMAC_REQIIC0_RXI0       0x07

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_dmac_channel_s
{
  volatile uint32_t chstat;     /* Channel Status Register */
  volatile uint32_t chctrl;     /* Channel Control Register */
  volatile uint32_t chptr;     /* Channel Pointer Register */
  volatile uint32_t chcfg;     /* Channel Configuration Register */
  volatile uint32_t chspb;     /* Channel Source Pointer Register */
  volatile uint32_t chdpb;     /* Channel Destination Pointer Register */
  volatile uint32_tchtc;     /* Channel Transfer Count Register */
  volatile uint32_t chdcb;     /* Channel DCB Register */
};

struct ra8p_dmac_s
{
  volatile uint32_t dmaast;     /* DMA Status Register */
  volatile uint32_t dmasof;     /* DMA Software Force Register */
  volatile uint32_t dmast;     /* DMA Status Register */
  volatile uint32_t dmaspr;     /* DMA Status Priority Register */
  volatile uint32_t dmbrs0;    /* DMA Bus Request Select Register 0 */
  volatile uint32_t dmbrs1;    /* DMA Bus Request Select Register 1 */
  volatile uint32_t dmbrs2;    /* DMA Bus Request Select Register 2 */
  volatile uint32_t dmbrs3;    /* DMA Bus Request Select Register 3 */
  volatile uint32_t dmbrp0;    /* DMA Bus Request Priority Register 0 */
  volatile uint32_t dmbrp1;    /* DMA Bus Request Priority Register 1 */
  volatile uint32_t dmbrp2;    /* DMA Bus Request Priority Register 2 */
  volatile uint32_t dmbrp3;    /* DMA Bus Request Priority Register 3 */
  volatile uint32_t dmasb;     /* DMA Status Buffer Register */
  volatile uint32_t dmasbc;    /* DMA Status Buffer Clear Register */
  volatile uint32_t dmasbs;    /* DMA Status Buffer Status Register */
  volatile uint32_t dmatc;     /* DMA Transfer Count Register */
  volatile uint32_t dmatcc;    /* DMA Transfer Count Clear Register */
  volatile uint32_t dmatcs;    /* DMA Transfer Count Status Register */
  volatile uint32_t dmacc;     /* DMA Channel Control Register */
  volatile uint32_t dmasw;     /* DMA Software Register */
};

struct ra8p_dmac_xfer_s
{
  uint32_t src_addr;          /* Source address */
  uint32_t dst_addr;          /* Destination address */
  uint16_t count;             /* Transfer count */
  uint8_t src_size;           /* Source data size */
  uint8_t dst_size;           /* Destination data size */
  uint8_t src_addr_mode;     /* Source address mode */
  uint8_t dst_addr_mode;     /* Destination address mode */
  uint8_t request;            /* DMA request select */
};

/****************************************************************************
 * Private Data
 ****************************************************************************/

static volatile struct ra8p_dmac_s *g_dmac = (volatile struct ra8p_dmac_s *)RA8P_DMAC_COMMON_BASE;
static volatile struct ra8p_dmac_channel_s *g_dmac_ch[8];

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: dmac_enable_channel
 ****************************************************************************/

static void dmac_enable_channel(int ch)
{
  uint32_t regval;
  
  if (ch < 0 || ch >= 8)
    return;
  
  /* Enable DMA channel */
  regval = g_dmac_ch[ch]->chctrl;
  regval |= (1 << 0);  /* DET flag - DMA enable */
  g_dmac_ch[ch]->chctrl = regval;
}

/****************************************************************************
 * Name: dmac_disable_channel
 ****************************************************************************/

static void dmac_disable_channel(int ch)
{
  uint32_t regval;
  
  if (ch < 0 || ch >= 8)
    return;
  
  /* Disable DMA channel */
  regval = g_dmac_ch[ch]->chctrl;
  regval &= ~(1 << 0);  /* DET flag - DMA disable */
  g_dmac_ch[ch]->chctrl = regval;
}

/****************************************************************************
 * Name: dmac_config_channel
 ****************************************************************************/

static int dmac_config_channel(int ch, const struct ra8p_dmac_xfer_s *xfer)
{
  uint32_t cfg;
  
  if (ch < 0 || ch >= 8 || xfer == NULL)
    return -EINVAL;
  
  /* Disable channel first */
  dmac_disable_channel(ch);
  
  /* Configure channel */
  cfg = 0;
  
  /* Set source address mode */
  cfg |= (xfer->src_addr_mode & 0x03) << 2;
  
  /* Set destination address mode */
  cfg |= (xfer->dst_addr_mode & 0x03) << 4;
  
  /* Set source data size */
  cfg |= (xfer->src_size & 0x07) << 6;
  
  /* Set destination data size */
  cfg |= (xfer->dst_size & 0x07) << 9;
  
  /* Set request select */
  cfg |= (xfer->request & 0x1F) << 16;
  
  /* Enable repeat mode */
  cfg |= (1 << 20);
  
  /* Enable channel */
  cfg |= (1 << 18);
  
  g_dmac_ch[ch]->chcfg = cfg;
  
  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_dmac_init
 *
 * Description:
 *   Initialize the DMA controller based on Zephyr dma_renesas_ra.c implementation.
 *
 ****************************************************************************/

int ra8p_dmac_init(void)
{
  int i;
  uint32_t base;
  
  /* Initialize channel bases */
  base = RA8P_DMAC_BASE;
  for (i = 0; i < 8; i++)
    {
      g_dmac_ch[i] = (volatile struct ra8p_dmac_channel_s *)(base + (i * 0x20));
    }
  
  /* Reset DMA controller */
  g_dmac->dmasof = 0x01;  /* Software reset */
  
  /* Clear software reset */
  g_dmac->dmasof = 0x00;
  
  /* Clear all flags */
  g_dmac->dmasbc = 0x01010101;  /* Clear all status flags */
  g_dmac->dmatcc = 0x01010101;  /* Clear all transfer count */
  g_dmac->dmasbs = 0x01;     /* Clear buffer status */
  
  /* Disable all channels initially */
  for (i = 0; i < 8; i++)
    {
      g_dmac_ch[i]->chctrl = 0x00;
    }
  
  dmavdbg("DMA controller initialized\n");
  
  return OK;
}

/****************************************************************************
 * Name: ra8p_dmac_channel_config
 *
 * Description:
 *   Configure a DMA channel for transfer.
 *
 * Input Parameters:
 *   ch - Channel number (0-7)
 *   config - Transfer configuration
 *
 ****************************************************************************/

int ra8p_dmac_channel_config(int ch, const struct ra8p_dmac_xfer_s *config)
{
  if (ch < 0 || ch >= 8 || config == NULL)
    return -EINVAL;
  
  /* Set source address */
  g_dmac_ch[ch]->chspc = config->src_addr;
  
  /* Set destination address */
  g_dmac_ch[ch]->chdpb = config->dst_addr;
  
  /* Set transfer count */
  g_dmac_ch[ch]->chtc = config->count;
  
  /* Configure channel */
  return dmac_config_channel(ch, config);
}

/****************************************************************************
 * Name: ra8p_dmac_channel_start
 *
 * Description:
 *   Start a DMA channel transfer.
 *
 * Input Parameters:
 *   ch - Channel number (0-7)
 *
 ****************************************************************************/

void ra8p_dmac_channel_start(int ch)
{
  if (ch < 0 || ch >= 8)
    return;
  
  /* Clear channel status flags */
  g_dmac_ch[ch]->chstat = 0x01;
  
  /* Enable channel */
  dmac_enable_channel(ch);
  
  /* Start DMA */
  g_dmac->dmasw = (1 << ch);
}

/****************************************************************************
 * Name: ra8p_dmac_channel_stop
 *
 * Description:
 *   Stop a DMA channel transfer.
 *
 * Input Parameters:
 *   ch - Channel number (0-7)
 *
 ****************************************************************************/

void ra8p_dmac_channel_stop(int ch)
{
  if (ch < 0 || ch >= 8)
    return;
  
  /* Disable channel */
  dmac_disable_channel(ch);
  
  /* Clear channel status flags */
  g_dmac_ch[ch]->chstat = 0x01;
}

/****************************************************************************
 * Name: ra8p_dmac_channel_status
 *
 * Description:
 *   Get DMA channel status.
 *
 * Input Parameters:
 *   ch - Channel number (0-7)
 *
 * Return Value:
 *   true if transfer is active, false otherwise
 *
 ****************************************************************************/

bool ra8p_dmac_channel_status(int ch)
{
  uint32_t stat;
  
  if (ch < 0 || ch >= 8)
    return false;
  
  /* Check channel status */
  stat = g_dmac_ch[ch]->chstat;
  return (stat & (1 << 1)) != 0;  /* ACT flag */
}

/****************************************************************************
 * Name: ra8p_dmac_select_request
 *
 * Description:
 *   Select DMA request source for a channel.
 *   Based on Zephyr dma_renesas_ra.c implementation.
 *
 * Input Parameters:
 *   ch - Channel number (0-7)
 *   request - DMA request source
 *
 ****************************************************************************/

int ra8p_dmac_select_request(int ch, uint8_t request)
{
  uint32_t reg;
  uint32_t mask;
  uint32_t pos;
  uint32_t val;
  
  if (ch < 0 || ch >= 8)
    return -EINVAL;
  
  /* Select request based on channel */
  if (ch < 2)
    {
      /* Channels 0-1: DMaRRS0 register */
      reg = g_dmac->dmbrs0;
      pos = ch * 16;
      mask = 0x1F << pos;
      val = request << pos;
    }
  else if (ch < 4)
    {
      /* Channels 2-3: DMaRRS1 register */
      reg = g_dmac->dmbrs1;
      pos = (ch - 2) * 16;
      mask = 0x1F << pos;
      val = request << pos;
    }
  else if (ch < 6)
    {
      /* Channels 4-5: DMaRRS2 register */
      reg = g_dmac->dmbrs2;
      pos = (ch - 4) * 16;
      mask = 0x1F << pos;
      val = request << pos;
    }
  else
    {
      /* Channels 6-7: DMaRRS3 register */
      reg = g_dmac->dmbrs3;
      pos = (ch - 6) * 16;
      mask = 0x1F << pos;
      val = request << pos;
    }
  
  /* Clear and set request */
  g_dmac->dmbrs0 = (g_dmac->dmbrs0 & ~mask) | val;
  
  return OK;
}

/****************************************************************************
 * Name: ra8p_dmac_memory_copy
 *
 * Description:
 *   Perform a memory-to-memory DMA copy.
 *
 * Input Parameters:
 *   ch - Channel number (0-7)
 *   dest - Destination address
 *   src - Source address
 *   len - Number of bytes to transfer
 *
 ****************************************************************************/

int ra8p_dmac_memory_copy(int ch, void *dest, const void *src, size_t len)
{
  struct ra8p_dmac_xfer_s xfer;
  
  if (ch < 0 || ch >= 8 || dest == NULL || src == NULL || len == 0)
    return -EINVAL;
  
  /* Configure transfer */
  xfer.src_addr = (uint32_t)src;
  xfer.dst_addr = (uint32_t)dest;
  xfer.count = (uint16_t)len;
  xfer.src_size = DMAC_TRANSFER_SIZE_1;
  xfer.dst_size = DMAC_TRANSFER_SIZE_1;
  xfer.src_addr_mode = DMAC_ADDR_MODE_INCREMENT;
  xfer.dst_addr_mode = DMAC_ADDR_MODE_INCREMENT;
  xfer.request = 0;  /* Software trigger */
  
  /* Configure and start transfer */
  ra8p_dmac_channel_config(ch, &xfer);
  ra8p_dmac_channel_start(ch);
  
  return OK;
}

#endif /* CONFIG_RA8P_HAVE_DMA */