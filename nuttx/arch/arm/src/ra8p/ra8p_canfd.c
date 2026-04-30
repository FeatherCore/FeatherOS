/****************************************************************************
 * arch/arm/src/ra8p/ra8p_canfd.c
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
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <debug.h>

#include "chip.h"
#include "arm_internal.h"
#include "hardware/ra8p_canfd.h"

#ifdef CONFIG_RA8P_CANFD

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define RA8P_CANFD0_BASE              RA8P_CANFD0_BASE
#define RA8P_CANFD1_BASE              RA8P_CANFD1_BASE

/* CANFD timeout in milliseconds */
#define RA8P_CANFD_TIMEOUT_MS         1000

/* Maximum number of acceptance filters */
#define RA8P_CANFD_MAX_FILTERS        32

/* Message RAM base address and size */
#define RA8P_CANFD_MSGRAM_BASE        RA8P_CANFD_MSGRAM_BASE
#define RA8P_CANFD_MSGRAM_SIZE        RA8P_CANFD_MSGRAM_SIZE

/* Maximum payload for CANFD */
#define RA8P_CANFD_MAX_PAYLOAD        RA8P_CANFD_MAX_PAYLOAD

/* Maximum number of TX message buffers */
#define RA8P_CANFD_TX_BUFFERS         RA8P_CANFD_TX_BUFFERS

/* Maximum number of RX message buffers */
#define RA8P_CANFD_RX_BUFFERS         RA8P_CANFD_RX_BUFFERS

/* CANFD base addresses */
#define RA8P_CANFD0_BASE              RA8P_CANFD0_BASE
#define RA8P_CANFD1_BASE              RA8P_CANFD1_BASE

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* RA8P1 CANFD driver state structure */

struct ra8p_canfd_priv_s
{
  uint32_t base;                              /* Base address of CANFD registers */
  uint8_t channel;                            /* CANFD channel (0 or 1) */
  bool initialized;                           /* Initialization flag */
  bool enabled;                               /* Enable flag */
  bool canfd_mode;                            /* CANFD mode enabled */
  bool is_canfd;                              /* Running in CANFD mode */
  uint32_t bitrate;                           /* Nominal bitrate */
  uint32_t data_bitrate;                      /* Data bitrate for CANFD */
  uint8_t num_filters;                        /* Number of configured filters */
  struct ra8p_canfd_filter_s filters[RA8P_CANFD_MAX_FILTERS]; /* Acceptance filters */
  uint8_t next_tx_buffer;                     /* Next TX buffer index */
  uint8_t next_rx_buffer;                     /* Next RX buffer index */
  sem_t tx_sem;                               /* Semaphore for TX synchronization */
  sem_t rx_sem;                               /* Semaphore for RX synchronization */
  bool tx_complete;                           /* TX complete flag */
  bool rx_complete;                           /* RX complete flag */
  uint32_t tx_errors;                         /* TX error count */
  uint32_t rx_errors;                         /* RX error count */
  bool loopback_mode;                         /* Loopback mode enabled */
  bool silent_mode;                           /* Silent mode enabled */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int canfd_wait_ready(struct ra8p_canfd_priv_s *priv);
static int canfd_calc_bit_timing(uint32_t clock_freq, uint32_t bitrate, uint8_t *prescaler,
                                uint8_t *tseg1, uint8_t *tseg2, uint8_t *sjw);
static void canfd_putreg32(struct ra8p_canfd_priv_s *priv, uint32_t offset, uint32_t value);
static uint32_t canfd_getreg32(struct ra8p_canfd_priv_s *priv, uint32_t offset);
static void canfd_putreg16(struct ra8p_canfd_priv_s *priv, uint32_t offset, uint16_t value);
static uint16_t canfd_getreg16(struct ra8p_canfd_priv_s *priv, uint32_t offset);
static void canfd_putreg8(struct ra8p_canfd_priv_s *priv, uint32_t offset, uint8_t value);
static uint8_t canfd_getreg8(struct ra8p_canfd_priv_s *priv, uint32_t offset);
static int canfd_set_nominal_bitrate(struct ra8p_canfd_priv_s *priv, uint32_t bitrate);
static int canfd_set_data_bitrate(struct ra8p_canfd_priv_s *priv, uint32_t bitrate);
static int canfd_configure_tx_buffer(struct ra8p_canfd_priv_s *priv, uint8_t buffer_index);
static int canfd_configure_rx_buffer(struct ra8p_canfd_priv_s *priv, uint8_t buffer_index);
static int canfd_wait_tx_complete(struct ra8p_canfd_priv_s *priv);
static int canfd_wait_rx_available(struct ra8p_canfd_priv_s *priv);
static int canfd_set_bus_mode(struct ra8p_canfd_priv_s *priv, uint8_t mode);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_canfd_priv_s g_canfd[2];  /* Two CANFD channels */

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: canfd_putreg32
 ****************************************************************************/

static inline void canfd_putreg32(struct ra8p_canfd_priv_s *priv, uint32_t offset, uint32_t value)
{
  putreg32(value, priv->base + offset);
}

/****************************************************************************
 * Name: canfd_getreg32
 ****************************************************************************/

static inline uint32_t canfd_getreg32(struct ra8p_canfd_priv_s *priv, uint32_t offset)
{
  return getreg32(priv->base + offset);
}

/****************************************************************************
 * Name: canfd_putreg16
 ****************************************************************************/

static inline void canfd_putreg16(struct ra8p_canfd_priv_s *priv, uint32_t offset, uint16_t value)
{
  putreg16(value, priv->base + offset);
}

/****************************************************************************
 * Name: canfd_getreg16
 ****************************************************************************/

static inline uint16_t canfd_getreg16(struct ra8p_canfd_priv_s *priv, uint32_t offset)
{
  return getreg16(priv->base + offset);
}

/****************************************************************************
 * Name: canfd_putreg8
 ****************************************************************************/

static inline void canfd_putreg8(struct ra8p_canfd_priv_s *priv, uint32_t offset, uint8_t value)
{
  putreg8(value, priv->base + offset);
}

/****************************************************************************
 * Name: canfd_getreg8
 ****************************************************************************/

static inline uint8_t canfd_getreg8(struct ra8p_canfd_priv_s *priv, uint32_t offset)
{
  return getreg8(priv->base + offset);
}

/****************************************************************************
 * Name: canfd_wait_ready
 ****************************************************************************/

static int canfd_wait_ready(struct ra8p_canfd_priv_s *priv)
{
  volatile int timeout = RA8P_CANFD_TIMEOUT_MS * 1000;  /* microseconds */

  /* Wait for channel to be ready (not halt mode) */
  while ((canfd_getreg32(priv, RA8P_CANFD_CFD0STS_OFFSET) & RA8P_CANFD_CFD0STS_CHALT) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  return OK;
}

/****************************************************************************
 * Name: canfd_calc_bit_timing
 ****************************************************************************/

static int canfd_calc_bit_timing(uint32_t clock_freq, uint32_t bitrate, uint8_t *prescaler,
                                uint8_t *tseg1, uint8_t *tseg2, uint8_t *sjw)
{
  uint32_t divisor;
  uint32_t best_error = UINT32_MAX;
  uint32_t best_prescaler = 1;
  uint32_t best_tseg1 = 0;
  uint32_t best_tseg2 = 0;
  uint32_t best_sjw = 1;
  uint32_t quantum;
  uint32_t calc_bitrate;
  uint32_t error;
  uint32_t i, j, k;

  if (bitrate == 0)
    {
      return -EINVAL;
    }

  /* Find optimal prescaler and segment values */
  for (i = 1; i <= 1024; i++)  /* prescaler */
    {
      for (j = 2; j <= 25; j++)  /* tseg1 (2-25) */
        {
          for (k = 1; k <= 12; k++)  /* tseg2 (1-12) */
            {
              uint32_t seg_total = 1 + j + k;  /* 1 sync seg + tseg1 + tseg2 */
              if (seg_total > 25)  /* Total time segments limit */
                {
                  continue;
                }
              
              divisor = i * seg_total;
              if (divisor == 0)
                {
                  continue;
                }
              
              calc_bitrate = clock_freq / divisor;
              error = (calc_bitrate > bitrate) ? (calc_bitrate - bitrate) : (bitrate - calc_bitrate);
              
              if (error < best_error)
                {
                  best_error = error;
                  best_prescaler = i;
                  best_tseg1 = j;
                  best_tseg2 = k;
                  best_sjw = (k < 4) ? k : 4; /* Min(tseg2, 4) for sjw */
                  
                  if (best_sjw < 1) best_sjw = 1;
                }
            }
        }
    }

  if (best_error == UINT32_MAX)
    {
      return -EINVAL;  /* Could not find valid parameters */
    }

  *prescaler = (uint8_t)best_prescaler;
  *tseg1 = (uint8_t)best_tseg1;
  *tseg2 = (uint8_t)best_tseg2;
  *sjw = (uint8_t)best_sjw;

  return OK;
}

/****************************************************************************
 * Name: canfd_set_nominal_bitrate
 ****************************************************************************/

static int canfd_set_nominal_bitrate(struct ra8p_canfd_priv_s *priv, uint32_t bitrate)
{
  uint32_t pclk = 62500000;  /* Use appropriate PCLK */
  uint8_t prescaler;
  uint8_t tseg1, tseg2, sjw;
  uint32_t regval;
  int ret;

  if (bitrate == 0)
    {
      return -EINVAL;
    }

  /* Calculate timing parameters */
  ret = canfd_calc_bit_timing(pclk, bitrate, &prescaler, &tseg1, &tseg2, &sjw);
  if (ret != OK)
    {
      return ret;
    }

  /* Configure nominal bit timing register */
  regval = ((sjw - 1) << RA8P_CANFD_CFD0NBTR_NSJW_SHIFT) |    /* SJW */
           ((tseg2 - 1) << RA8P_CANFD_CFD0NBTR_NTSEG2_SHIFT) |  /* TSEG2 */
           ((tseg1 - 1) << RA8P_CANFD_CFD0NBTR_NTSEG1_SHIFT) |  /* TSEG1 */
           ((prescaler - 1) << RA8P_CANFD_CFD0NBTR_NBRP_SHIFT); /* BRP */
  canfd_putreg32(priv, RA8P_CANFD_CFD0NBTR_OFFSET, regval);

  priv->bitrate = bitrate;

  canfdinfo("CANFD%d nominal bitrate set to %u Hz (prescaler=%u, tseg1=%u, tseg2=%u, sjw=%u)\n",
            priv->channel, bitrate, prescaler, tseg1, tseg2, sjw);
  return OK;
}

/****************************************************************************
 * Name: canfd_set_data_bitrate
 ****************************************************************************/

static int canfd_set_data_bitrate(struct ra8p_canfd_priv_s *priv, uint32_t bitrate)
{
  uint32_t pclk = 62500000;  /* Use appropriate PCLK */
  uint8_t prescaler;
  uint8_t tseg1, tseg2, sjw;
  uint32_t regval;
  int ret;

  if (bitrate == 0)
    {
      return -EINVAL;
    }

  /* Calculate timing parameters for data bitrate */
  ret = canfd_calc_bit_timing(pclk, bitrate, &prescaler, &tseg1, &tseg2, &sjw);
  if (ret != OK)
    {
      return ret;
    }

  /* Configure data bit timing register */
  regval = ((sjw - 1) << RA8P_CANFD_CFD0DBTR_DSJW_SHIFT) |    /* SJW */
           ((tseg2 - 1) << RA8P_CANFD_CFD0DBTR_DTSEG2_SHIFT) |  /* TSEG2 */
           ((tseg1 - 1) << RA8P_CANFD_CFD0DBTR_DTSEG1_SHIFT) |  /* TSEG1 */
           ((prescaler - 1) << RA8P_CANFD_CFD0DBTR_DBRP_SHIFT); /* BRP */
  canfd_putreg32(priv, RA8P_CANFD_CFD0DBTR_OFFSET, regval);

  priv->data_bitrate = bitrate;

  canfdinfo("CANFD%d data bitrate set to %u Hz (prescaler=%u, tseg1=%u, tseg2=%u, sjw=%u)\n",
            priv->channel, bitrate, prescaler, tseg1, tseg2, sjw);
  return OK;
}

/****************************************************************************
 * Name: canfd_set_bus_mode
 ****************************************************************************/

static int canfd_set_bus_mode(struct ra8p_canfd_priv_s *priv, uint8_t mode)
{
  uint32_t regval;

  regval = canfd_getreg32(priv, RA8P_CANFD_CFD0CTR_OFFSET);
  regval &= ~RA8P_CANFD_CFD0CTR_BOM_MASK;
  regval |= (mode << RA8P_CANFD_CFD0CTR_BOM_SHIFT);
  canfd_putreg32(priv, RA8P_CANFD_CFD0CTR_OFFSET, regval);

  return OK;
}

/****************************************************************************
 * Name: canfd_configure_tx_buffer
 ****************************************************************************/

static int canfd_configure_tx_buffer(struct ra8p_canfd_priv_s *priv, uint8_t buffer_index)
{
  uint32_t mb_base;

  if (buffer_index >= RA8P_CANFD_TX_BUFFERS)
    {
      return -EINVAL;
    }

  /* Calculate message buffer address */
  uint32_t buffer_addr = RA8P_CANFD_MSGRAM_BASE + (buffer_index * 0x20);

  /* Configure TX message buffer */
  mb_base = priv->base + RA8P_CANFD_CFD0TF_BASE + (buffer_index * 0x20);

  /* Set message buffer address */
  canfd_putreg32(priv, mb_base + 0x00, buffer_addr);
  canfd_putreg32(priv, mb_base + 0x04, RA8P_CANFD_MAX_PAYLOAD); /* Max payload size */

  return OK;
}

/****************************************************************************
 * Name: canfd_configure_rx_buffer
 ****************************************************************************/

static int canfd_configure_rx_buffer(struct ra8p_canfd_priv_s *priv, uint8_t buffer_index)
{
  uint32_t mb_base;

  if (buffer_index >= RA8P_CANFD_RX_BUFFERS)
    {
      return -EINVAL;
    }

  /* Calculate buffer address for RX buffer */
  uint32_t buffer_addr = RA8P_CANFD_MSGRAM_BASE + 
                         (RA8P_CANFD_TX_BUFFERS * 0x20) + 
                         (buffer_index * 0x20);

  /* Configure RX message buffer */
  mb_base = priv->base + RA8P_CANFD_CFD0RM_BASE + (buffer_index * 0x20);

  /* Set message buffer address */
  canfd_putreg32(priv, mb_base + 0x00, buffer_addr);
  canfd_putreg32(priv, mb_base + 0x04, RA8P_CANFD_MAX_PAYLOAD); /* Max payload size */

  return OK;
}

/****************************************************************************
 * Name: canfd_wait_tx_complete
 ****************************************************************************/

static int canfd_wait_tx_complete(struct ra8p_canfd_priv_s *priv)
{
  volatile int timeout = RA8P_CANFD_TIMEOUT_MS * 1000;  /* microseconds */

  /* Wait for TX completion interrupt */
  while (!priv->tx_complete && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  return OK;
}

/****************************************************************************
 * Name: canfd_wait_rx_available
 ****************************************************************************/

static int canfd_wait_rx_available(struct ra8p_canfd_priv_s *priv)
{
  volatile int timeout = RA8P_CANFD_TIMEOUT_MS * 1000;  /* microseconds */

  /* Wait for RX availability */
  while (!priv->rx_complete && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_canfd_initialize
 *
 * Description:
 *   Initialize the CANFD controller based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   bitrate - CAN bitrate in Hz
 *   data_bitrate - CANFD data bitrate in Hz (0 for classic CAN mode)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_initialize(uint8_t channel, uint32_t bitrate, uint32_t data_bitrate)
{
  struct ra8p_canfd_priv_s *priv;
  uint32_t regval;
  int ret;
  int i;

  if (channel >= 2)
    {
      return -EINVAL;
    }

  priv = &g_canfd[channel];
  priv->channel = channel;
  priv->base = (channel == 0) ? RA8P_CANFD0_BASE : RA8P_CANFD1_BASE;
  priv->bitrate = bitrate ? bitrate : 500000;      /* Default to 500 kbps */
  priv->data_bitrate = data_bitrate;               /* 0 = classic CAN mode */
  priv->canfd_mode = (data_bitrate > 0 && data_bitrate != bitrate);
  priv->num_filters = 0;
  priv->initialized = false;
  priv->enabled = false;
  priv->loopback_mode = false;
  priv->silent_mode = false;
  priv->next_tx_buffer = 0;
  priv->next_rx_buffer = 0;

  /* Initialize semaphores */
  nxsem_init(&priv->tx_sem, 0, 1);
  nxsem_init(&priv->rx_sem, 0, 1);

  /* Reset CANFD module */
  regval = canfd_getreg32(priv, RA8P_CANFD_CFDGCTR_OFFSET);
  regval |= RA8P_CANFD_CFDGCTR_GRST;  /* Set global reset */
  canfd_putreg32(priv, RA8P_CANFD_CFDGCTR_OFFSET, regval);

  /* Wait for reset to complete */
  volatile int timeout = RA8P_CANFD_TIMEOUT_MS * 1000;
  while ((canfd_getreg32(priv, RA8P_CANFD_CFDGSTS_OFFSET) & RA8P_CANFD_CFDGSTS_GRSTSTS) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Configure global settings */
  regval = canfd_getreg32(priv, RA8P_CANFD_CFDGCFG_OFFSET);
  regval |= RA8P_CANFD_CFDGCFG_TPRI;  /* Transmission priority based on ID */
  regval |= RA8P_CANFD_CFDGCFG_DCE;   /* DLC check enable */
  regval &= ~RA8P_CANFD_CFDGCFG_DCS;  /* Use data length code as payload length */
  regval |= RA8P_CANFD_CFDGCFG_MME;   /* Message lost enable */
  regval |= RA8P_CANFD_CFDGCFG_THLE;  /* TX history list enable */
  canfd_putreg32(priv, RA8P_CANFD_CFDGCFG_OFFSET, regval);

  /* Configure global error interrupts */
  regval = canfd_getreg32(priv, RA8P_CANFD_CFDGCTR_OFFSET);
  regval |= RA8P_CANFD_CFDGCTR_DEIE;   /* DLC Error Interrupt Enable */
  regval |= RA8P_CANFD_CFDGCTR_MEIE;   /* Message Lost Error Interrupt Enable */
  regval |= RA8P_CANFD_CFDGCTR_THLEIE; /* TX History List Entry Lost Interrupt Enable */
  regval |= RA8P_CANFD_CFDGCTR_CMPOFIE; /* CANFD Message Payload Overflow Flag Interrupt Enable */
  canfd_putreg32(priv, RA8P_CANFD_CFDGCTR_OFFSET, regval);

  /* Configure channel control register */
  regval = canfd_getreg32(priv, RA8P_CANFD_CFD0CTR_OFFSET);
  regval &= ~RA8P_CANFD_CFD0CTR_CHMDC_MASK;  /* Clear channel mode */
  regval |= (0x02 << RA8P_CANFD_CFD0CTR_CHMDC_SHIFT);  /* Normal mode */
  
  /* Enable CANFD mode if data bitrate specified */
  if (priv->canfd_mode)
    {
      regval |= RA8P_CANFD_CFD0CTR_FDEN;   /* Enable CANFD */
      regval |= RA8P_CANFD_CFD0CTR_ISOCC;  /* ISO CANFD CC mode */
      regval |= RA8P_CANFD_CFD0CTR_MLM;    /* Multiple bitrate (BRS) mode */
    }
  else
    {
      regval &= ~RA8P_CANFD_CFD0CTR_FDEN;  /* Classic CAN mode */
      regval &= ~RA8P_CANFD_CFD0CTR_ISOCC;
      regval &= ~RA8P_CANFD_CFD0CTR_MLM;
    }
  
  canfd_putreg32(priv, RA8P_CANFD_CFD0CTR_OFFSET, regval);

  /* Configure bitrates */
  ret = canfd_set_nominal_bitrate(priv, priv->bitrate);
  if (ret != OK)
    {
      return ret;
    }

  if (priv->canfd_mode && data_bitrate != bitrate)
    {
      ret = canfd_set_data_bitrate(priv, data_bitrate);
      if (ret != OK)
        {
          return ret;
        }
    }

  /* Configure RX FIFO */
  uint32_t rxfifo_cfg = (1 << 0) |    /* Enable FIFO */
                        (1 << 1) |    /* Enable interrupt */
                        (7 << 4) |    /* Payload size: 64 bytes */
                        (15 << 8) |   /* FIFO depth: 16 messages */
                        (1 << 12);    /* Interrupt at every received message */
  canfd_putreg32(priv, RA8P_CANFD_CFDRFCC0_OFFSET, rxfifo_cfg);

  /* Configure message RAM */
  canfd_putreg32(priv, RA8P_CANFD_CFDGMSTS_OFFSET, RA8P_CANFD_MSGRAM_BASE);
  canfd_putreg32(priv, RA8P_CANFD_CFDGMSTS_OFFSET + 4, RA8P_CANFD_MSGRAM_SIZE);

  /* Initialize TX/RX message buffers */
  for (i = 0; i < RA8P_CANFD_TX_BUFFERS; i++)
    {
      ret = canfd_configure_tx_buffer(priv, i);
      if (ret != OK)
        {
          return ret;
        }
    }

  for (i = 0; i < RA8P_CANFD_RX_BUFFERS; i++)
    {
      ret = canfd_configure_rx_buffer(priv, i);
      if (ret != OK)
        {
          return ret;
        }
    }

  /* Clear interrupt status */
  canfd_putreg32(priv, RA8P_CANFD_CFD0INTSTS_OFFSET, 0xFFFFFFFF);

  /* Enable necessary interrupts */
  regval = canfd_getreg32(priv, RA8P_CANFD_CFD0GIC_OFFSET);
  regval |= RA8P_CANFD_CFD0GIC_BRDYE |   /* Buffer ready interrupt enable */
           RA8P_CANFD_CFD0GIC_NRDYE |    /* Buffer not ready interrupt enable */
           RA8P_CANFD_CFD0GIC_BEMPE |    /* Buffer empty interrupt enable */
           RA8P_CANFD_CFD0GIC_CTRTIE;    /* Control transfer interrupt enable */
  canfd_putreg32(priv, RA8P_CANFD_CFD0GIC_OFFSET, regval);

  priv->initialized = true;

  canfdinfo("CANFD%d initialized: bitrate=%u, data_bitrate=%u, fd_mode=%s\n",
            channel, bitrate, data_bitrate, priv->canfd_mode ? "yes" : "no");
  return OK;
}

/****************************************************************************
 * Name: ra8p_canfd_start
 *
 * Description:
 *   Start CANFD channel operation based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_start(uint8_t channel)
{
  struct ra8p_canfd_priv_s *priv;
  uint32_t regval;

  if (channel >= 2 || !g_canfd[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_canfd[channel];

  /* Exit halt mode */
  regval = canfd_getreg32(priv, RA8P_CANFD_CFD0CTR_OFFSET);
  regval &= ~RA8P_CANFD_CFD0CTR_CHMNT;  /* Clear halt mode */
  canfd_putreg32(priv, RA8P_CANFD_CFD0CTR_OFFSET, regval);

  /* Wait for exit from halt mode */
  volatile int timeout = RA8P_CANFD_TIMEOUT_MS * 1000;
  while ((canfd_getreg32(priv, RA8P_CANFD_CFD0STS_OFFSET) & RA8P_CANFD_CFD0STS_CHALT) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  priv->enabled = true;

  canfdinfo("CANFD%d started\n", channel);
  return OK;
}

/****************************************************************************
 * Name: ra8p_canfd_stop
 *
 * Description:
 *   Stop CANFD channel operation based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_stop(uint8_t channel)
{
  struct ra8p_canfd_priv_s *priv;
  uint32_t regval;

  if (channel >= 2 || !g_canfd[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_canfd[channel];

  /* Enter halt mode */
  regval = canfd_getreg32(priv, RA8P_CANFD_CFD0CTR_OFFSET);
  regval |= RA8P_CANFD_CFD0CTR_CHMNT;  /* Set halt mode */
  canfd_putreg32(priv, RA8P_CANFD_CFD0CTR_OFFSET, regval);

  /* Wait for halt mode */
  volatile int timeout = RA8P_CANFD_TIMEOUT_MS * 1000;
  while (!(canfd_getreg32(priv, RA8P_CANFD_CFD0STS_OFFSET) & RA8P_CANFD_CFD0STS_CHALT) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  priv->enabled = false;

  canfdinfo("CANFD%d stopped\n", channel);
  return OK;
}

/****************************************************************************
 * Name: ra8p_canfd_send
 *
 * Description:
 *   Send a CAN/CANFD frame based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   frame - Pointer to CANFD frame structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_send(uint8_t channel, const struct ra8p_canfd_frame_s *frame)
{
  struct ra8p_canfd_priv_s *priv;
  uint32_t mb_base;
  uint32_t id_reg;
  uint32_t dlc_reg;
  int i;
  uint8_t mb_idx;

  if (channel >= 2 || !g_canfd[channel].enabled || frame == NULL)
    {
      return -EINVAL;
    }

  if (frame->dlc > (g_canfd[channel].canfd_mode ? 64 : 8))  /* Max payload check */
    {
      return -EINVAL;
    }

  priv = &g_canfd[channel];

  /* Select next available TX buffer */
  mb_idx = priv->next_tx_buffer;
  priv->next_tx_buffer = (priv->next_tx_buffer + 1) % RA8P_CANFD_TX_BUFFERS;

  /* Calculate TX message buffer address */
  mb_base = priv->base + RA8P_CANFD_CFD0TF_BASE + (mb_idx * 0x20);

  /* Prepare ID register */
  id_reg = frame->id;
  if (frame->extended)
    {
      id_reg |= RA8P_CANFD_IDE;  /* Extended ID */
    }

  if (frame->rtr)
    {
      id_reg |= RA8P_CANFD_RTR;  /* Remote transmission request */
    }

  /* Set CANFD-specific flags if enabled */
  if (priv->canfd_mode && frame->fd)
    {
      id_reg |= RA8P_CANFD_EDL;  /* Extended Data Length for CANFD */
      if (frame->brs)
        {
          id_reg |= RA8P_CANFD_BRS;  /* Bit Rate Switch */
        }
    }

  if (frame->esi)
    {
      id_reg |= RA8P_CANFD_ESI;  /* Error State Indicator */
    }

  /* Prepare DLC (Data Length Code) */
  uint8_t dlc = frame->dlc;
  if (priv->canfd_mode && frame->dlc > 8)
    {
      /* For CANFD frames with more than 8 bytes, use special DLC encoding */
      if (frame->dlc <= 12) dlc = RA8P_CANFD_DLC_12;
      else if (frame->dlc <= 16) dlc = RA8P_CANFD_DLC_16;
      else if (frame->dlc <= 20) dlc = RA8P_CANFD_DLC_20;
      else if (frame->dlc <= 24) dlc = RA8P_CANFD_DLC_24;
      else if (frame->dlc <= 32) dlc = RA8P_CANFD_DLC_32;
      else if (frame->dlc <= 48) dlc = RA8P_CANFD_DLC_48;
      else dlc = RA8P_CANFD_DLC_64;  /* 64 bytes */
    }

  /* Write ID register */
  canfd_putreg32(priv, mb_base + RA8P_CANFD_MB_ID_OFFSET, id_reg);
  
  /* Write DLC register */
  dlc_reg = (uint32_t)dlc << 24;
  if (priv->canfd_mode && frame->timestamp)
    {
      dlc_reg |= (frame->ts_value << 0);  /* Add timestamp if requested */
    }
  canfd_putreg32(priv, mb_base + RA8P_CANFD_MB_DLC_OFFSET, dlc_reg);

  /* Write data bytes */
  for (i = 0; i < frame->dlc; i += 4)
    {
      uint32_t data = frame->data[i];
      if (i + 1 < frame->dlc) data |= ((uint32_t)frame->data[i + 1]) << 8;
      if (i + 2 < frame->dlc) data |= ((uint32_t)frame->data[i + 2]) << 16;
      if (i + 3 < frame->dlc) data |= ((uint32_t)frame->data[i + 3]) << 24;
      
      canfd_putreg32(priv, mb_base + RA8P_CANFD_MB_DATA_OFFSET + i, data);
    }

  /* Set buffer control register for transmission */
  uint32_t ctrl_reg = canfd_getreg32(priv, mb_base + RA8P_CANFD_MB_CTRL_OFFSET);
  ctrl_reg |= (1 << 0);  /* Set to send state */
  canfd_putreg32(priv, mb_base + RA8P_CANFD_MB_CTRL_OFFSET, ctrl_reg);

  /* Request transmission for this message buffer */
  uint32_t cmd = canfd_getreg32(priv, RA8P_CANFD_CFD0CMDTR_OFFSET);
  cmd |= (1 << mb_idx);  /* Set command for this message buffer */
  canfd_putreg32(priv, RA8P_CANFD_CFD0CMDTR_OFFSET, cmd);

  /* Wait for transmission to complete */
  int ret = canfd_wait_tx_complete(priv);
  if (ret != OK)
    {
      canfderr("CANFD%d transmission failed\n", channel);
      return ret;
    }

  canfdinfo("CANFD%d sent frame ID=0x%x, DLC=%d\n", channel, frame->id, frame->dlc);
  return OK;
}

/****************************************************************************
 * Name: ra8p_canfd_receive
 *
 * Description:
 *   Receive a CAN/CANFD frame based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   frame - Pointer to CANFD frame structure to fill
 *   timeout - Timeout in ms (0 for non-blocking)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_receive(uint8_t channel, struct ra8p_canfd_frame_s *frame, 
                      unsigned int timeout)
{
  struct ra8p_canfd_priv_s *priv;
  uint32_t mb_base;
  uint32_t id_reg;
  uint32_t dlc_reg;
  int i;
  uint8_t mb_idx = 0;  /* Use first buffer as example */

  if (channel >= 2 || !g_canfd[channel].enabled || frame == NULL)
    {
      return -EINVAL;
    }

  priv = &g_canfd[channel];

  /* Check if there's data in the RX FIFO */
  uint32_t rxfsts = canfd_getreg32(priv, RA8P_CANFD_CFDRFSTS0_OFFSET);
  if (!(rxfsts & (1 << 16)))  /* Check RFOIFn flag */
    {
      if (timeout == 0)
        {
          return -EAGAIN;  /* Non-blocking mode and no data available */
        }
      
      /* Wait for data with timeout */
      int ret = canfd_wait_rx_available(priv);
      if (ret != OK)
        {
          return ret;
        }
    }

  /* Calculate RX message buffer address */
  mb_base = priv->base + RA8P_CANFD_CFD0RM_BASE + (mb_idx * 0x20);

  /* Read ID register */
  id_reg = canfd_getreg32(priv, mb_base + RA8P_CANFD_MB_ID_OFFSET);

  /* Parse ID and flags */
  frame->id = id_reg & 0x1FFFFFFF;  /* Mask to get ID bits */
  frame->extended = (id_reg & RA8P_CANFD_IDE) != 0;
  frame->rtr = (id_reg & RA8P_CANFD_RTR) != 0;
  frame->fd = priv->canfd_mode && (id_reg & RA8P_CANFD_EDL) != 0;
  frame->brs = (id_reg & RA8P_CANFD_BRS) != 0;
  frame->esi = (id_reg & RA8P_CANFD_ESI) != 0;

  /* Read DLC register to get length */
  dlc_reg = canfd_getreg32(priv, mb_base + RA8P_CANFD_MB_DLC_OFFSET);
  uint8_t dlc_code = (dlc_reg >> 24) & 0x0F;

  /* Convert DLC back to actual data length */
  if (frame->fd && dlc_code > 8)
    {
      /* Decode CANFD DLC to actual length */
      switch (dlc_code)
        {
          case RA8P_CANFD_DLC_12:  frame->dlc = 12; break;
          case RA8P_CANFD_DLC_16:  frame->dlc = 16; break;
          case RA8P_CANFD_DLC_20:  frame->dlc = 20; break;
          case RA8P_CANFD_DLC_24:  frame->dlc = 24; break;
          case RA8P_CANFD_DLC_32:  frame->dlc = 32; break;
          case RA8P_CANFD_DLC_48:  frame->dlc = 48; break;
          case RA8P_CANFD_DLC_64:  frame->dlc = 64; break;
          default: frame->dlc = 8; break;
        }
    }
  else
    {
      frame->dlc = (dlc_code > 8) ? 8 : dlc_code;
    }

  /* Read data bytes */
  for (i = 0; i < frame->dlc; i += 4)
    {
      uint32_t data = canfd_getreg32(priv, mb_base + RA8P_CANFD_MB_DATA_OFFSET + i);
      frame->data[i] = data & 0xFF;
      if (i + 1 < frame->dlc) frame->data[i + 1] = (data >> 8) & 0xFF;
      if (i + 2 < frame->dlc) frame->data[i + 2] = (data >> 16) & 0xFF;
      if (i + 3 < frame->dlc) frame->data[i + 3] = (data >> 24) & 0xFF;
    }

  /* Update RX FIFO read pointer */
  uint32_t rxf_rc = canfd_getreg32(priv, RA8P_CANFD_CFDRFC0_OFFSET);
  rxf_rc++;  /* Increment read counter */
  canfd_putreg32(priv, RA8P_CANFD_CFDRFC0_OFFSET, rxf_rc);

  canfdinfo("CANFD%d received frame ID=0x%x, DLC=%d\n", channel, frame->id, frame->dlc);
  return OK;
}

/****************************************************************************
 * Name: ra8p_canfd_add_filter
 *
 * Description:
 *   Add a CAN acceptance filter based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   filter - Pointer to filter configuration
 *
 * Returned Value:
 *   Filter index on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_add_filter(uint8_t channel, const struct ra8p_canfd_filter_s *filter)
{
  struct ra8p_canfd_priv_s *priv;

  if (channel >= 2 || filter == NULL || !g_canfd[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_canfd[channel];

  if (priv->num_filters >= RA8P_CANFD_MAX_FILTERS)
    {
      return -ENOMEM;
    }

  /* Store filter in local array */
  priv->filters[priv->num_filters] = *filter;
  priv->num_filters++;

  /* Configure acceptance filter in hardware (simplified implementation) */
  /* Would normally set up AFL (Acceptance Filter List) registers here */
  uint32_t afl_addr = priv->base + RA8P_CANFD_CFDGAFL_BASE + (priv->num_filters * 0x10);
  uint32_t id_reg = filter->id;
  uint32_t mask_reg = filter->mask;
  
  if (filter->extended)
    {
      id_reg |= RA8P_CANFD_IDE;    /* Extended ID */
      mask_reg |= RA8P_CANFD_IDE;  /* Extended ID mask */
    }

  canfd_putreg32(priv, afl_addr + 0x00, id_reg);   /* ID */
  canfd_putreg32(priv, afl_addr + 0x04, mask_reg); /* Mask */
  canfd_putreg32(priv, afl_addr + 0x08, 0);        /* Config */

  canfdinfo("CANFD%d filter added: ID=0x%x, MASK=0x%x\n", 
            channel, filter->id, filter->mask);
  return priv->num_filters - 1;
}

/****************************************************************************
 * Name: ra8p_canfd_remove_filter
 *
 * Description:
 *   Remove a CAN acceptance filter based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   index - Filter index to remove
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_remove_filter(uint8_t channel, int index)
{
  struct ra8p_canfd_priv_s *priv;

  if (channel >= 2 || index < 0 || !g_canfd[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_canfd[channel];

  if (index >= priv->num_filters)
    {
      return -ENOENT;
    }

  /* Shift remaining filters down */
  for (int i = index; i < priv->num_filters - 1; i++)
    {
      priv->filters[i] = priv->filters[i + 1];
    }
  priv->num_filters--;

  /* Disable the filter in hardware */
  uint32_t afl_addr = priv->base + RA8P_CANFD_CFDGAFL_BASE + (index * 0x10);
  canfd_putreg32(priv, afl_addr + 0x00, 0xFFFFFFFF); /* Disable filter ID */
  canfd_putreg32(priv, afl_addr + 0x04, 0x00000000);   /* Disable filter mask */

  canfdinfo("CANFD%d filter %d removed\n", channel, index);
  return OK;
}

/****************************************************************************
 * Name: ra8p_canfd_set_bitrate
 *
 * Description:
 *   Set CANFD bitrate based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   bitrate - Nominal bitrate in Hz
 *   data_bitrate - Data bitrate in Hz (0 for classic CAN)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_set_bitrate(uint8_t channel, uint32_t bitrate, uint32_t data_bitrate)
{
  struct ra8p_canfd_priv_s *priv;
  int ret;

  if (channel >= 2 || bitrate == 0 || !g_canfd[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_canfd[channel];

  /* Temporarily disable CANFD for reconfiguration */
  if (priv->enabled)
    {
      ret = ra8p_canfd_stop(channel);
      if (ret != OK)
        {
          return ret;
        }
    }

  /* Set nominal bitrate */
  ret = canfd_set_nominal_bitrate(priv, bitrate);
  if (ret != OK)
    {
      goto restore;
    }

  /* Set data bitrate if different (for CANFD mode) */
  if (data_bitrate > 0 && data_bitrate != bitrate)
    {
      ret = canfd_set_data_bitrate(priv, data_bitrate);
      if (ret != OK)
        {
          goto restore;
        }
      priv->canfd_mode = true;
      priv->data_bitrate = data_bitrate;
    }
  else
    {
      priv->canfd_mode = false;
      priv->data_bitrate = bitrate;
    }

  /* Update stored bitrate */
  priv->bitrate = bitrate;

  /* Re-enable CANFD if it was enabled */
  if (g_canfd[channel].enabled)
    {
      ret = ra8p_canfd_start(channel);
      if (ret != OK)
        {
          return ret;
        }
    }

  canfdinfo("CANFD%d bitrate set: nominal=%u, data=%u\n", channel, bitrate, data_bitrate);
  return OK;

restore:
  /* Restore original bitrates if failed */
  if (g_canfd[channel].enabled)
    {
      ra8p_canfd_start(channel);
    }
  return ret;
}

/****************************************************************************
 * Name: ra8p_canfd_set_mode
 *
 * Description:
 *   Set CANFD operating mode (classic CAN vs CANFD) based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   fdmode - true for CANFD mode, false for classic CAN mode
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_set_mode(uint8_t channel, bool fdmode)
{
  struct ra8p_canfd_priv_s *priv;
  uint32_t regval;

  if (channel >= 2 || !g_canfd[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_canfd[channel];

  /* Get current control register */
  regval = canfd_getreg32(priv, RA8P_CANFD_CFD0CTR_OFFSET);

  if (fdmode)
    {
      /* Enable CANFD mode */
      regval |= RA8P_CANFD_CFD0CTR_FDEN;   /* Enable CANFD */
      regval |= RA8P_CANFD_CFD0CTR_ISOCC;  /* Enable ISO CANFD CC */
      priv->is_canfd = true;
    }
  else
    {
      /* Disable CANFD mode */
      regval &= ~RA8P_CANFD_CFD0CTR_FDEN;  /* Disable CANFD */
      regval &= ~RA8P_CANFD_CFD0CTR_ISOCC; /* Disable ISO CANFD CC */
      priv->is_canfd = false;
    }

  canfd_putreg32(priv, RA8P_CANFD_CFD0CTR_OFFSET, regval);

  canfdinfo("CANFD%d mode set to %s\n", channel, fdmode ? "CANFD" : "classic");
  return OK;
}

/****************************************************************************
 * Name: ra8p_canfd_is_enabled
 *
 * Description:
 *   Check if CANFD channel is enabled based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   true if enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_canfd_is_enabled(uint8_t channel)
{
  if (channel >= 2 || !g_canfd[channel].initialized)
    {
      return false;
    }

  return g_canfd[channel].enabled;
}

/****************************************************************************
 * Name: ra8p_canfd_is_error
 *
 * Description:
 *   Check if CANFD has error flags set based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   true if error occurred, false otherwise
 *
 ****************************************************************************/

bool ra8p_canfd_is_error(uint8_t channel)
{
  if (channel >= 2 || !g_canfd[channel].initialized)
    {
      return false;
    }

  struct ra8p_canfd_priv_s *priv = &g_canfd[channel];
  uint32_t error_reg = canfd_getreg32(priv, RA8P_CANFD_CFD0ER_OFFSET);
  return (error_reg & (RA8P_CANFD_CFD0ER_OVERFLOW | 
                       RA8P_CANFD_CFD0ER_UNDERFLOW | 
                       RA8P_CANFD_CFD0ER_MES)) != 0;
}

/****************************************************************************
 * Name: ra8p_canfd_clear_error
 *
 * Description:
 *   Clear CANFD error flags based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_clear_error(uint8_t channel)
{
  if (channel >= 2 || !g_canfd[channel].initialized)
    {
      return -EINVAL;
    }

  struct ra8p_canfd_priv_s *priv = &g_canfd[channel];

  /* Clear error flags */
  uint32_t error_reg = canfd_getreg32(priv, RA8P_CANFD_CFD0ER_OFFSET);
  canfd_putreg32(priv, RA8P_CANFD_CFD0ER_OFFSET, error_reg);  /* Write to clear */

  /* Also clear error counters */
  canfd_putreg32(priv, RA8P_CANFD_CFD0REC_OFFSET, 0);
  canfd_putreg32(priv, RA8P_CANFD_CFD0TEC_OFFSET, 0);

  return OK;
}

/****************************************************************************
 * Name: ra8p_canfd_get_status
 *
 * Description:
 *   Get CANFD status flags based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   Status flags
 *
 ****************************************************************************/

uint32_t ra8p_canfd_get_status(uint8_t channel)
{
  if (channel >= 2 || !g_canfd[channel].initialized)
    {
      return 0;
    }

  struct ra8p_canfd_priv_s *priv = &g_canfd[channel];
  return canfd_getreg32(priv, RA8P_CANFD_CFD0COMSTS_OFFSET);
}

/****************************************************************************
 * Name: ra8p_canfd_get_error_count
 *
 * Description:
 *   Get CANFD error count (TX/RX) based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *   count - Pointer to error count structure to fill
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_get_error_count(uint8_t channel, struct ra8p_canfd_error_count_s *count)
{
  if (channel >= 2 || count == NULL || !g_canfd[channel].initialized)
    {
      return -EINVAL;
    }

  struct ra8p_canfd_priv_s *priv = &g_canfd[channel];

  count->tx_errors = canfd_getreg32(priv, RA8P_CANFD_CFD0TEC_OFFSET) & 0xFF;
  count->rx_errors = canfd_getreg32(priv, RA8P_CANFD_CFD0REC_OFFSET) & 0xFF;
  count->tx_warning_limit = 96;  /* Default warning level */
  count->rx_warning_limit = 96;

  return OK;
}

/****************************************************************************
 * Name: ra8p_canfd_reset_counters
 *
 * Description:
 *   Reset CANFD error counters based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_reset_counters(uint8_t channel)
{
  if (channel >= 2 || !g_canfd[channel].initialized)
    {
      return -EINVAL;
    }

  struct ra8p_canfd_priv_s *priv = &g_canfd[channel];

  /* Reset error counters */
  canfd_putreg32(priv, RA8P_CANFD_CFD0TEC_OFFSET, 0);
  canfd_putreg32(priv, RA8P_CANFD_CFD0REC_OFFSET, 0);

  return OK;
}

/****************************************************************************
 * Name: ra8p_canfd_is_canfd_mode
 *
 * Description:
 *   Check if CANFD is operating in CANFD mode based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   true if in CANFD mode, false if in classic CAN mode
 *
 ****************************************************************************/

bool ra8p_canfd_is_canfd_mode(uint8_t channel)
{
  if (channel >= 2 || !g_canfd[channel].initialized)
    {
      return false;
    }

  return g_canfd[channel].canfd_mode;
}

/****************************************************************************
 * Name: ra8p_canfd_is_bus_off
 *
 * Description:
 *   Check if CANFD is in bus-off state based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   true if in bus-off state, false otherwise
 *
 ****************************************************************************/

bool ra8p_canfd_is_bus_off(uint8_t channel)
{
  if (channel >= 2 || !g_canfd[channel].initialized)
    {
      return false;
    }

  struct ra8p_canfd_priv_s *priv = &g_canfd[channel];
  uint32_t comsts = canfd_getreg32(priv, RA8P_CANFD_CFD0COMSTS_OFFSET);
  uint8_t com_state = (comsts & RA8P_CANFD_CFD0STS_COMSTS_MASK) >> RA8P_CANFD_CFD0STS_COMSTS_SHIFT;
  
  return (com_state == RA8P_CANFD_COMSTS_BUS_OFF);
}

/****************************************************************************
 * Name: ra8p_canfd_get_bus_state
 *
 * Description:
 *   Get CANFD bus state based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   Bus state (0=error-active, 1=error-passive, 2=bus-off)
 *
 ****************************************************************************/

uint8_t ra8p_canfd_get_bus_state(uint8_t channel)
{
  if (channel >= 2 || !g_canfd[channel].initialized)
    {
      return 0;
    }

  struct ra8p_canfd_priv_s *priv = &g_canfd[channel];
  uint32_t comsts = canfd_getreg32(priv, RA8P_CANFD_CFD0COMSTS_OFFSET);
  uint8_t com_state = (comsts & RA8P_CANFD_CFD0STS_COMSTS_MASK) >> RA8P_CANFD_CFD0STS_COMSTS_SHIFT;
  
  switch (com_state)
    {
      case RA8P_CANFD_COMSTS_ERR_ACTIVE:
        return 0;  /* Error active */
      case RA8P_CANFD_COMSTS_ERR_PASSIVE:
        return 1;  /* Error passive */
      case RA8P_CANFD_COMSTS_BUS_OFF:
        return 2;  /* Bus off */
      default:
        return 0;  /* Default to error active */
    }
}

/****************************************************************************
 * Name: ra8p_canfd_recover_from_bus_off
 *
 * Description:
 *   Recover CANFD from bus-off state based on Nuttx CAN driver implementation.
 *
 * Input Parameters:
 *   channel - CANFD channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_canfd_recover_from_bus_off(uint8_t channel)
{
  struct ra8p_canfd_priv_s *priv;
  uint32_t regval;

  if (channel >= 2 || !g_canfd[channel].initialized)
    {
      return -EINVAL;
    }

  priv = &g_canfd[channel];

  /* Reset to clear bus-off state */
  regval = canfd_getreg32(priv, RA8P_CANFD_CFD0CTR_OFFSET);
  regval |= RA8P_CANFD_CFD0CTR_CHMNT;  /* Enter halt mode first */
  canfd_putreg32(priv, RA8P_CANFD_CFD0CTR_OFFSET, regval);

  /* Wait for halt mode */
  volatile int timeout = RA8P_CANFD_TIMEOUT_MS * 1000;
  while (!(canfd_getreg32(priv, RA8P_CANFD_CFD0STS_OFFSET) & RA8P_CANFD_CFD0STS_CHALT) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Exit halt mode to reset bus-off condition */
  regval = canfd_getreg32(priv, RA8P_CANFD_CFD0CTR_OFFSET);
  regval &= ~RA8P_CANFD_CFD0CTR_CHMNT;  /* Clear halt mode */
  canfd_putreg32(priv, RA8P_CANFD_CFD0CTR_OFFSET, regval);

  /* Wait for exit from halt mode */
  timeout = RA8P_CANFD_TIMEOUT_MS * 1000;
  while ((canfd_getreg32(priv, RA8P_CANFD_CFD0STS_OFFSET) & RA8P_CANFD_CFD0STS_CHALT) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  canfdinfo("CANFD%d recovered from bus-off state\n", channel);
  return OK;
}

#endif /* CONFIG_RA8P_CANFD */