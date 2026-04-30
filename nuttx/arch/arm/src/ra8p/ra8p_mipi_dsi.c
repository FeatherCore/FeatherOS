/****************************************************************************
 * arch/arm/src/ra8p/ra8p_mipi_dsi.c
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
#include <nuttx/arch.h>
#include <nuttx/irq.h>
#include <nuttx/kmalloc.h>
#include <nuttx/mutex.h>
#include <nuttx/video/mipi_dsi.h>
#include <arch/board/board.h>

#include "arm_internal.h"
#include "chip.h"
#include "ra8p_config.h"
#include "ra8p_peripherals.h"
#include "hardware/ra8p_mipi_dsi.h"

#ifdef CONFIG_RA8P_MIPI_DSI

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Helper macros for register access */

#define ra8p_dsi_putreg32(base, offset, val) \
  putreg32((val), (base) + (offset))

#define ra8p_dsi_getreg32(base, offset) \
  getreg32((base) + (offset))

#define ra8p_dsi_modifyreg32(base, offset, clrbits, setbits) \
  ra8p_dsi_putreg32(base, offset, \
    (ra8p_dsi_getreg32(base, offset) & ~(clrbits)) | (setbits))

/* DSI timeout values */

#define RA8P_DSI_TIMEOUT_MS                    (100)

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_dsi_dev_s
{
  struct mipi_dsi_dev_s dev;        /* MIPI DSI interface */
  uint32_t base;                    /* Base address of DSI registers */
  int irq;                          /* DSI interrupt number */
  mutex_t lock;                     /* Thread-safe lock */
  sem_t wait_sem;                   /* Wait semaphore for interrupt handling */
  uint8_t lanes;                    /* Number of data lanes */
  bool initialized;                 /* True if initialized */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int ra8p_dsi_wait_for_status(struct ra8p_dsi_dev_s *priv,
                                     uint32_t mask, uint32_t timeout_ms);
static int ra8p_dsi_configure_pll(struct ra8p_dsi_dev_s *priv,
                                   uint32_t pixel_clock);
static int ra8p_dsi_configure_phy(struct ra8p_dsi_dev_s *priv);
static int ra8p_dsi_send_short_packet(struct ra8p_dsi_dev_s *priv,
                                       uint8_t data_type, uint8_t channel,
                                       uint16_t data);
static int ra8p_dsi_send_long_packet(struct ra8p_dsi_dev_s *priv,
                                      uint8_t data_type, uint8_t channel,
                                      const uint8_t *payload, size_t len);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_dsi_dev_s g_dsi_priv;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_dsi_wait_for_status
 ****************************************************************************/

static int ra8p_dsi_wait_for_status(struct ra8p_dsi_dev_s *priv,
                                     uint32_t mask, uint32_t timeout_ms)
{
  uint32_t timeout = timeout_ms * 1000;
  uint32_t status;

  while (timeout > 0)
    {
      status = ra8p_dsi_getreg32(priv->base, RA8P_DSI_DSISR_OFFSET);
      if (status & mask)
        {
          return OK;
        }

      up_udelay(1);
      timeout--;
    }

  return -ETIMEDOUT;
}

/****************************************************************************
 * Name: ra8p_dsi_configure_pll
 ****************************************************************************/

static int ra8p_dsi_configure_pll(struct ra8p_dsi_dev_s *priv,
                                   uint32_t pixel_clock)
{
  uint32_t pll_freq;
  uint32_t n_div;
  uint32_t m_div;
  uint32_t regval;

  /* Calculate PLL parameters based on pixel clock
   * PLL output frequency = (FREF * N) / M
   * where FREF is typically 24 MHz (XTAL)
   */

  /* For simplicity, use fixed values for typical display configurations
   * In a real implementation, these would be calculated dynamically
   */

  n_div = 40;   /* N divider */
  m_div = 1;    /* M divider */

  pll_freq = (24000000 * n_div) / m_div;

  /* Configure PLL */

  regval = (n_div << RA8P_DSI_DSIPLLCR_NDIV_SHIFT) |
           (m_div << RA8P_DSI_DSIPLLCR_MDIV_SHIFT);

  ra8p_dsi_putreg32(priv->base, RA8P_DSI_DSIPLLCR_OFFSET, regval);

  /* Enable PLL */

  ra8p_dsi_modifyreg32(priv->base, RA8P_DSI_DSIPLLCR_OFFSET,
                       0, RA8P_DSI_DSIPLLCR_PLLEN);

  /* Wait for PLL to lock */

  return ra8p_dsi_wait_for_status(priv, RA8P_DSI_DSISR_PLLST,
                                   RA8P_DSI_TIMEOUT_MS);
}

/****************************************************************************
 * Name: ra8p_dsi_configure_phy
 ****************************************************************************/

static int ra8p_dsi_configure_phy(struct ra8p_dsi_dev_s *priv)
{
  uint32_t regval;

  /* Configure PHY timing parameters */

  regval = (50 << RA8P_DSI_DSIPHYTR_THSPREP_SHIFT) |
           (20 << RA8P_DSI_DSIPHYTR_THSZERO_SHIFT) |
           (30 << RA8P_DSI_DSIPHYTR_THSTRAIL_SHIFT);

  ra8p_dsi_putreg32(priv->base, RA8P_DSI_DSIPHYTR_OFFSET, regval);

  /* Enable PHY */

  ra8p_dsi_modifyreg32(priv->base, RA8P_DSI_DSIPHYCR_OFFSET,
                       0, RA8P_DSI_DSIPHYCR_PHYEN);

  /* Wait for PHY to be ready */

  return ra8p_dsi_wait_for_status(priv, RA8P_DSI_DSIPHYCR_PHYST,
                                   RA8P_DSI_TIMEOUT_MS);
}

/****************************************************************************
 * Name: ra8p_dsi_send_short_packet
 ****************************************************************************/

static int ra8p_dsi_send_short_packet(struct ra8p_dsi_dev_s *priv,
                                       uint8_t data_type, uint8_t channel,
                                       uint16_t data)
{
  uint32_t regval;
  int ret;

  /* Configure packet control register */

  regval = (data_type << RA8P_DSI_DSIPCR_PKTCMD_SHIFT) |
           ((channel & 0x3) << 8);

  ra8p_dsi_putreg32(priv->base, RA8P_DSI_DSIPCR_OFFSET, regval);

  /* Write packet data */

  ra8p_dsi_putreg32(priv->base, RA8P_DSI_DSIPDR_OFFSET, data);

  /* Wait for command to complete */

  ret = ra8p_dsi_wait_for_status(priv, RA8P_DSI_DSISR_CMDDONE,
                                  RA8P_DSI_TIMEOUT_MS);
  if (ret < 0)
    {
      return ret;
    }

  /* Clear status */

  ra8p_dsi_putreg32(priv->base, RA8P_DSI_DSISR_OFFSET,
                    RA8P_DSI_DSISR_CMDDONE);

  return OK;
}

/****************************************************************************
 * Name: ra8p_dsi_send_long_packet
 ****************************************************************************/

static int ra8p_dsi_send_long_packet(struct ra8p_dsi_dev_s *priv,
                                      uint8_t data_type, uint8_t channel,
                                      const uint8_t *payload, size_t len)
{
  uint32_t regval;
  int ret;
  size_t i;
  uint32_t data;

  /* Configure packet control register */

  regval = (data_type << RA8P_DSI_DSIPCR_PKTCMD_SHIFT) |
           ((channel & 0x3) << 8) |
           (len << RA8P_DSI_DSIPCR_PKTSIZE_SHIFT);

  ra8p_dsi_putreg32(priv->base, RA8P_DSI_DSIPCR_OFFSET, regval);

  /* Write payload data (4 bytes at a time) */

  for (i = 0; i < len; i += 4)
    {
      data = 0;
      if (i < len) data |= payload[i];
      if (i + 1 < len) data |= (payload[i + 1] << 8);
      if (i + 2 < len) data |= (payload[i + 2] << 16);
      if (i + 3 < len) data |= (payload[i + 3] << 24);

      ra8p_dsi_putreg32(priv->base, RA8P_DSI_DSIPDR_OFFSET, data);
    }

  /* Wait for command to complete */

  ret = ra8p_dsi_wait_for_status(priv, RA8P_DSI_DSISR_CMDDONE,
                                  RA8P_DSI_TIMEOUT_MS);
  if (ret < 0)
    {
      return ret;
    }

  /* Clear status */

  ra8p_dsi_putreg32(priv->base, RA8P_DSI_DSISR_OFFSET,
                    RA8P_DSI_DSISR_CMDDONE);

  return OK;
}

/****************************************************************************
 * Name: ra8p_dsi_attach
 ****************************************************************************/

static int ra8p_dsi_attach(struct mipi_dsi_dev_s *dev)
{
  struct ra8p_dsi_dev_s *priv = (struct ra8p_dsi_dev_s *)dev;

  if (!priv)
    {
      return -EINVAL;
    }

  /* Enable DSI */

  ra8p_dsi_modifyreg32(priv->base, RA8P_DSI_DSICR_OFFSET,
                       0, RA8P_DSI_DSICR_DSIEN);

  return OK;
}

/****************************************************************************
 * Name: ra8p_dsi_detach
 ****************************************************************************/

static void ra8p_dsi_detach(struct mipi_dsi_dev_s *dev)
{
  struct ra8p_dsi_dev_s *priv = (struct ra8p_dsi_dev_s *)dev;

  if (!priv)
    {
      return;
    }

  /* Disable DSI */

  ra8p_dsi_modifyreg32(priv->base, RA8P_DSI_DSICR_OFFSET,
                       RA8P_DSI_DSICR_DSIEN, 0);
}

/****************************************************************************
 * Name: ra8p_dsi_transfer
 ****************************************************************************/

static ssize_t ra8p_dsi_transfer(struct mipi_dsi_dev_s *dev,
                                  const struct mipi_dsi_msg_s *msg)
{
  struct ra8p_dsi_dev_s *priv = (struct ra8p_dsi_dev_s *)dev;
  int ret;

  if (!priv || !msg)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Check if DSI is busy */

  if (ra8p_dsi_getreg32(priv->base, RA8P_DSI_DSISR_OFFSET) & RA8P_DSI_DSISR_BUSY)
    {
      nxmutex_unlock(&priv->lock);
      return -EBUSY;
    }

  /* Send packet based on type */

  if (msg->tx_len > 2)
    {
      /* Long packet */

      ret = ra8p_dsi_send_long_packet(priv, msg->type, msg->channel,
                                       msg->tx_buf, msg->tx_len);
    }
  else
    {
      /* Short packet */

      uint16_t data = 0;
      if (msg->tx_len > 0)
        {
          data = ((uint16_t *)msg->tx_buf)[0];
        }

      ret = ra8p_dsi_send_short_packet(priv, msg->type, msg->channel, data);
    }

  nxmutex_unlock(&priv->lock);

  return (ret < 0) ? ret : msg->tx_len;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_dsi_initialize
 *
 * Description:
 *   Initialize the MIPI DSI driver
 *
 ****************************************************************************/

int ra8p_dsi_initialize(void)
{
  struct ra8p_dsi_dev_s *priv = &g_dsi_priv;
  int ret;

  /* Initialize private data structure */

  memset(priv, 0, sizeof(struct ra8p_dsi_dev_s));
  priv->base = RA8P_MIPI_DSI_BASE;
  priv->irq = RA8P_IRQ_MIPI_DSI;
  priv->lanes = 2;  /* Default to 2 lanes */

  nxmutex_init(&priv->lock);
  nxsem_init(&priv->wait_sem, 0, 0);

  /* Initialize DSI ops */

  priv->dev.ops = &g_dsi_ops;

  /* Reset DSI */

  ra8p_dsi_modifyreg32(priv->base, RA8P_DSI_DSICR_OFFSET,
                       0, RA8P_DSI_DSICR_DSIRES);

  up_udelay(10);

  /* Configure PHY */

  ret = ra8p_dsi_configure_phy(priv);
  if (ret < 0)
    {
      goto errout_with_locks;
    }

  /* Configure PLL (default to 60 MHz pixel clock) */

  ret = ra8p_dsi_configure_pll(priv, 60000000);
  if (ret < 0)
    {
      goto errout_with_locks;
    }

  /* Configure DSI mode (command mode by default) */

  ra8p_dsi_putreg32(priv->base, RA8P_DSI_DSIMR_OFFSET,
                    RA8P_DSI_DSIMR_NL_2LANES);

  priv->initialized = true;

  /* Register MIPI DSI device */

  ret = mipi_dsi_register(&priv->dev);
  if (ret < 0)
    {
      goto errout_with_locks;
    }

  return OK;

errout_with_locks:
  nxmutex_destroy(&priv->lock);
  nxsem_destroy(&priv->wait_sem);
  return ret;
}

/****************************************************************************
 * Name: ra8p_dsi_configure_video_mode
 *
 * Description:
 *   Configure DSI for video mode operation
 *
 ****************************************************************************/

int ra8p_dsi_configure_video_mode(uint16_t hactive, uint16_t vactive,
                                   uint16_t hsync, uint16_t hbp, uint16_t hfp,
                                   uint16_t vsync, uint16_t vbp, uint16_t vfp)
{
  struct ra8p_dsi_dev_s *priv = &g_dsi_priv;
  uint32_t regval;

  if (!priv || !priv->initialized)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Configure video mode */

  ra8p_dsi_modifyreg32(priv->base, RA8P_DSI_DSIMR_OFFSET,
                       0, RA8P_DSI_DSIMR_VSMODE_VIDEO);

  /* Configure video mode settings */

  ra8p_dsi_putreg32(priv->base, RA8P_DSI_DSIVMR_OFFSET,
                    RA8P_DSI_DSIVMR_VSEN | RA8P_DSI_DSIVMR_FRM_BURST);

  /* Configure active dimensions */

  regval = (hactive << RA8P_DSI_DSICFGR_HACT_SHIFT) |
           (vactive << RA8P_DSI_DSICFGR_VACT_SHIFT);
  ra8p_dsi_putreg32(priv->base, RA8P_DSI_DSICFGR_OFFSET, regval);

  /* Configure timing parameters */

  regval = (hsync << RA8P_DSI_DSITR0_HSA_SHIFT) |
           (hbp << RA8P_DSI_DSITR0_HBP_SHIFT) |
           (hfp << RA8P_DSI_DSITR0_HFP_SHIFT);
  ra8p_dsi_putreg32(priv->base, RA8P_DSI_DSITR0_OFFSET, regval);

  regval = (vsync << RA8P_DSI_DSITR1_VSA_SHIFT) |
           (vbp << RA8P_DSI_DSITR1_VBP_SHIFT) |
           (vfp << RA8P_DSI_DSITR1_VFP_SHIFT);
  ra8p_dsi_putreg32(priv->base, RA8P_DSI_DSITR1_OFFSET, regval);

  nxmutex_unlock(&priv->lock);
  return OK;
}

/****************************************************************************
 * Name: g_dsi_ops
 *
 * Description:
 *   MIPI DSI operations
 *
 ****************************************************************************/

struct mipi_dsi_ops_s g_dsi_ops =
{
  .attach    = ra8p_dsi_attach,
  .detach    = ra8p_dsi_detach,
  .transfer  = ra8p_dsi_transfer
};

#endif /* CONFIG_RA8P_MIPI_DSI */