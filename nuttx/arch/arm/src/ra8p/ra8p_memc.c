/****************************************************************************
 * arch/arm/src/ra8p/ra8p_memc.c
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
#include <arch/board/board.h>

#include "arm_internal.h"
#include "chip.h"
#include "ra8p_config.h"
#include "ra8p_peripherals.h"
#include "hardware/ra8p_memc.h"

#ifdef CONFIG_RA8P_MEMC_

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Helper macros for register access */

#define ra8p_memc_putreg32(offset, val) \
  putreg32((val), RA8P_MEMC_BASE + (offset))

#define ra8p_memc_getreg32(offset) \
  getreg32(RA8P_MEMC_BASE + (offset))

#define ra8p_memc_modifyreg32(offset, clrbits, setbits) \
  ra8p_memc_putreg32(offset, \
    (ra8p_memc_getreg32(offset) & ~(clrbits)) | (setbits))

/* MEMC timeout */

#define RA8P_MEMC_TIMEOUT_MS                   (100)

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_memc_dev_s
{
  uint32_t base;                    /* Base address of MEMC registers */
  mutex_t lock;                     /* Thread-safe lock */
  struct ra8p_sdram_config_s config; /* SDRAM configuration */
  bool initialized;                 /* True if initialized */
};

static struct ra8p_memc_dev_s g_memc_priv;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_memc_reset
 ****************************************************************************/

static void ra8p_memc_reset(void)
{
  ra8p_memc_modifyreg32(RA8P_MEMC_MEMCCR_OFFSET, 0, RA8P_MEMC_MEMCCR_MEMCEN);
  up_udelay(10);
  ra8p_memc_modifyreg32(RA8P_MEMC_MEMCCR_OFFSET, RA8P_MEMC_MEMCCR_MEMCEN, 0);
  up_udelay(10);
  ra8p_memc_modifyreg32(RA8P_MEMC_MEMCCR_OFFSET, 0, RA8P_MEMC_MEMCCR_RSTEMC);
  up_udelay(10);
  ra8p_memc_modifyreg32(RA8P_MEMC_MEMCCR_OFFSET, RA8P_MEMC_MEMCCR_RSTEMC, 0);
}

/****************************************************************************
 * Name: ra8p_memc_configure
 ****************************************************************************/

static int ra8p_memc_configure(struct ra8p_sdram_config_s *config)
{
  uint32_t regval;

  if (!config)
    {
      return -EINVAL;
    }

  /* Configure SDRAM Control Register 0 */

  regval = (config->size << RA8P_MEMC_SDCR0_SDSIZ_SHIFT) |
           (config->cas << RA8P_MEMC_SDCR0_CSEL_SHIFT) |
           (config->row_addr_width << RA8P_MEMC_SDCR0_SRAW_SHIFT) |
           (config->col_addr_width << RA8P_MEMC_SDCR0_SCAW_SHIFT);

  ra8p_memc_putreg32(RA8P_MEMC_SDCR0_OFFSET, regval);

  /* Configure SDRAM Control Register 1 */

  regval = (config->refresh_interval << RA8P_MEMC_SDRFR_REFSEL_SHIFT) |
           (config->refresh_wait << RA8P_MEMC_SDRFR_REFW_SHIFT);

  ra8p_memc_putreg32(RA8P_MEMC_SDCR1_OFFSET, regval);

  /* Configure SDRAM Timing Register */

  regval = (config->tras << RA8P_MEMC_SDTR_TRAS_SHIFT) |
           (config->trcd << RA8P_MEMC_SDTR_TRCD_SHIFT) |
           (config->trc << RA8P_MEMC_SDTR_TRC_SHIFT) |
           (config->trp << RA8P_MEMC_SDTR_TRP_SHIFT) |
           (config->trfc << RA8P_MEMC_SDTR_TRFC_SHIFT);

  ra8p_memc_putreg32(RA8P_MEMC_SDTR_OFFSET, regval);

  /* Configure Bus Control Register */

  regval = (config->bus_width == 32) ? RA8P_MEMC_BUSCR_BE : 0;
  ra8p_memc_putreg32(RA8P_MEMC_BUSCR_OFFSET, regval);

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_memc_initialize
 *
 * Description:
 *   Initialize the MEMC (SDRAM Controller) driver
 *
 ****************************************************************************/

int ra8p_memc_initialize(void)
{
  struct ra8p_memc_dev_s *priv = &g_memc_priv;
  int ret;

  /* Initialize private data structure */

  memset(priv, 0, sizeof(struct ra8p_memc_dev_s));
  priv->base = RA8P_MEMC_BASE;

  nxmutex_init(&priv->lock);

  /* Reset MEMC */

  ra8p_memc_reset();

  /* Set default SDRAM configuration (64MB, CAS3) */

  priv->config.size = RA8P_SDRM_SIZE_64MB;
  priv->config.cas = RA8P_SDRM_CAS_3;
  priv->config.row_addr_width = 12;  /* 12-bit row address */
  priv->config.col_addr_width = 9;   /* 9-bit column address */
  priv->config.bus_width = 32;      /* 32-bit bus */
  priv->config.refresh_interval = 0x1;  /* Auto refresh */
  priv->config.refresh_wait = 0x7;    /* Refresh wait states */
  priv->config.tras = 0x5;             /* tRAS wait */
  priv->config.trcd = 0x2;             /* tRCD wait */
  priv->config.trc = 0x8;              /* tRC wait */
  priv->config.trp = 0x2;              /* tRP wait */
  priv->config.trfc = 0x8;             /* tRFC wait */

  /* Configure MEMC */

  ret = ra8p_memc_configure(&priv->config);
  if (ret < 0)
    {
      nxmutex_destroy(&priv->lock);
      return ret;
    }

  /* Enable MEMC */

  ra8p_memc_modifyreg32(RA8P_MEMC_MEMCCR_OFFSET, 0, RA8P_MEMC_MEMCCR_MEMCEN);

  priv->initialized = true;

  return OK;
}

/****************************************************************************
 * Name: ra8p_memc_set_refresh_interval
 *
 * Description:
 *   Set SDRAM refresh interval
 *
 ****************************************************************************/

int ra8p_memc_set_refresh_interval(uint32_t interval)
{
  struct ra8p_memc_dev_s *priv = &g_memc_priv;

  if (!priv || !priv->initialized)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  priv->config.refresh_interval = interval;
  ra8p_memc_modifyreg32(RA8P_MEMC_SDRFR_OFFSET,
                       RA8P_MEMC_SDRFR_REFSEL_MASK,
                       interval << RA8P_MEMC_SDRFR_REFSEL_SHIFT);

  nxmutex_unlock(&priv->lock);
  return OK;
}

/****************************************************************************
 * Name: ra8p_memc_get_status
 *
 * Description:
 *   Get SDRAM controller status
 *
 ****************************************************************************/

uint32_t ra8p_memc_get_status(void)
{
  struct ra8p_memc_dev_s *priv = &g_memc_priv;

  if (!priv || !priv->initialized)
    {
      return 0;
    }

  return ra8p_memc_getreg32(RA8P_MEMC_SDSR_OFFSET);
}

#endif /* CONFIG_RA8P_MEMC_ */