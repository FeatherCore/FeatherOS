/****************************************************************************
 * arch/arm/src/ra8p/ra8p_crc.c
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
#include <nuttx/kmalloc.h>
#include <nuttx/mutex.h>
#include <arch/board/board.h>

#include "arm_internal.h"
#include "chip.h"
#include "ra8p_config.h"
#include "ra8p_peripherals.h"
#include "hardware/ra8p_crc.h"

#ifdef CONFIG_RA8P_CRC

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Helper macros for register access */

#define ra8p_crc_putreg32(offset, val) \
  putreg32((val), RA8P_CRC_BASE + (offset))

#define ra8p_crc_getreg32(offset) \
  getreg32(RA8P_CRC_BASE + (offset))

#define ra8p_crc_modifyreg32(offset, clrbits, setbits) \
  ra8p_crc_putreg32(offset, \
    (ra8p_crc_getreg32(offset) & ~(clrbits)) | (setbits))

/* CRC timeout */

#define RA8P_CRC_TIMEOUT_MS                    (100)

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_crc_dev_s
{
  mutex_t lock;                     /* Thread-safe lock */
  struct ra8p_crc_config_s config;  /* Current CRC configuration */
  bool initialized;                 /* True if initialized */
};

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_crc_dev_s g_crc_priv;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_crc_wait_complete
 ****************************************************************************/

static int ra8p_crc_wait_complete(void)
{
  uint32_t timeout = RA8P_CRC_TIMEOUT_MS * 1000;
  uint32_t status;

  while (timeout > 0)
    {
      status = ra8p_crc_getreg32(RA8P_CRC_CRCSTR_OFFSET);
      if ((status & RA8P_CRC_CRCSTR_CRCST) == 0)
        {
          return OK;
        }

      up_udelay(1);
      timeout--;
    }

  return -ETIMEDOUT;
}

/****************************************************************************
 * Name: ra8p_crc_configure
 ****************************************************************************/

static void ra8p_crc_configure(struct ra8p_crc_config_s *config)
{
  uint32_t regval;

  /* Reset CRC module */

  ra8p_crc_modifyreg32(RA8P_CRC_CRCCR_OFFSET, 0, RA8P_CRC_CRCCR_CRCRST);
  up_udelay(1);
  ra8p_crc_modifyreg32(RA8P_CRC_CRCCR_OFFSET, RA8P_CRC_CRCCR_CRCRST, 0);

  /* Configure polynomial */

  regval = (config->polynomial << RA8P_CRC_CRCCR_CRCPOL_SHIFT);

  /* Configure bit order */

  if (config->bit_order == RA8P_CRC_BIT_ORDER_LSB)
    {
      regval |= RA8P_CRC_CRCCR_CRCDIR_LSB;
    }
  else
    {
      regval |= RA8P_CRC_CRCCR_CRCDIR_MSB;
    }

  /* Configure data width */

  regval |= (config->data_width << RA8P_CRC_CRCCR_DORSEL_SHIFT);

  /* Enable CRC */

  regval |= RA8P_CRC_CRCCR_CRCEN;

  ra8p_crc_putreg32(RA8P_CRC_CRCCR_OFFSET, regval);

  /* Set initial value */

  ra8p_crc_putreg32(RA8P_CRC_CRCINIT_OFFSET, config->seed);
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_crc_initialize
 *
 * Description:
 *   Initialize the CRC driver
 *
 ****************************************************************************/

int ra8p_crc_initialize(void)
{
  struct ra8p_crc_dev_s *priv = &g_crc_priv;

  /* Initialize private data structure */

  memset(priv, 0, sizeof(struct ra8p_crc_dev_s));

  nxmutex_init(&priv->lock);

  /* Set default configuration */

  priv->config.polynomial = RA8P_CRC_POLY_CRC32;
  priv->config.bit_order = RA8P_CRC_BIT_ORDER_MSB;
  priv->config.data_width = RA8P_CRC_DATA_WIDTH_8BIT;
  priv->config.seed = 0xFFFFFFFF;

  /* Configure CRC */

  ra8p_crc_configure(&priv->config);

  priv->initialized = true;

  return OK;
}

/****************************************************************************
 * Name: ra8p_crc_calculate
 *
 * Description:
 *   Calculate CRC for given data
 *
 ****************************************************************************/

uint32_t ra8p_crc_calculate(const void *data, size_t len,
                            struct ra8p_crc_config_s *config)
{
  struct ra8p_crc_dev_s *priv = &g_crc_priv;
  const uint8_t *src = (const uint8_t *)data;
  uint32_t crc;
  size_t i;
  int ret;

  if (!priv || !priv->initialized || !data || len == 0)
    {
      return 0;
    }

  nxmutex_lock(&priv->lock);

  /* Configure CRC if different from current */

  if (config)
    {
      ra8p_crc_configure(config);
    }
  else
    {
      ra8p_crc_configure(&priv->config);
    }

  /* Write data to CRC input register */

  for (i = 0; i < len; i++)
    {
      ra8p_crc_putreg32(RA8P_CRC_CRCDIR_OFFSET, src[i]);
    }

  /* Start CRC calculation */

  ra8p_crc_modifyreg32(RA8P_CRC_CRCCR_OFFSET, 0, RA8P_CRC_CRCCR_CRCS);

  /* Wait for completion */

  ret = ra8p_crc_wait_complete();
  if (ret < 0)
    {
      nxmutex_unlock(&priv->lock);
      return 0;
    }

  /* Read CRC result */

  crc = ra8p_crc_getreg32(RA8P_CRC_CRCDOR_OFFSET);

  nxmutex_unlock(&priv->lock);
  return crc;
}

/****************************************************************************
 * Name: ra8p_crc8_calculate
 *
 * Description:
 *   Calculate CRC-8 for given data
 *
 ****************************************************************************/

uint8_t ra8p_crc8_calculate(const void *data, size_t len)
{
  struct ra8p_crc_config_s config;

  config.polynomial = RA8P_CRC_POLY_CRC8;
  config.bit_order = RA8P_CRC_BIT_ORDER_MSB;
  config.data_width = RA8P_CRC_DATA_WIDTH_8BIT;
  config.seed = 0;

  return (uint8_t)ra8p_crc_calculate(data, len, &config);
}

/****************************************************************************
 * Name: ra8p_crc16_calculate
 *
 * Description:
 *   Calculate CRC-16 for given data
 *
 ****************************************************************************/

uint16_t ra8p_crc16_calculate(const void *data, size_t len)
{
  struct ra8p_crc_config_s config;

  config.polynomial = RA8P_CRC_POLY_CRC16;
  config.bit_order = RA8P_CRC_BIT_ORDER_MSB;
  config.data_width = RA8P_CRC_DATA_WIDTH_8BIT;
  config.seed = 0;

  return (uint16_t)ra8p_crc_calculate(data, len, &config);
}

/****************************************************************************
 * Name: ra8p_crc32_calculate
 *
 * Description:
 *   Calculate CRC-32 for given data
 *
 ****************************************************************************/

uint32_t ra8p_crc32_calculate(const void *data, size_t len)
{
  struct ra8p_crc_config_s config;

  config.polynomial = RA8P_CRC_POLY_CRC32;
  config.bit_order = RA8P_CRC_BIT_ORDER_LSB;
  config.data_width = RA8P_CRC_DATA_WIDTH_8BIT;
  config.seed = 0xFFFFFFFF;

  return ra8p_crc_calculate(data, len, &config);
}

/****************************************************************************
 * Name: ra8p_crc_ccitt_calculate
 *
 * Description:
 *   Calculate CRC-CCITT for given data
 *
 ****************************************************************************/

uint16_t ra8p_crc_ccitt_calculate(const void *data, size_t len)
{
  struct ra8p_crc_config_s config;

  config.polynomial = RA8P_CRC_POLY_CRC_CCITT;
  config.bit_order = RA8P_CRC_BIT_ORDER_MSB;
  config.data_width = RA8P_CRC_DATA_WIDTH_8BIT;
  config.seed = 0xFFFF;

  return (uint16_t)ra8p_crc_calculate(data, len, &config);
}

#endif /* CONFIG_RA8P_CRC */