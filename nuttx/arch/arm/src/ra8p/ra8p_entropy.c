/****************************************************************************
 * arch/arm/src/ra8p/ra8p_entropy.c
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
#include <nuttx/random.h>
#include <arch/board/board.h>

#include "arm_internal.h"
#include "chip.h"
#include "ra8p_config.h"
#include "ra8p_peripherals.h"
#include "hardware/ra8p_entropy.h"

#ifdef CONFIG_RA8P_ENTROPY

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Helper macros for register access */

#define ra8p_trng_putreg32(offset, val) \
  putreg32((val), RA8P_TRNG_BASE + (offset))

#define ra8p_trng_getreg32(offset) \
  getreg32(RA8P_TRNG_BASE + (offset))

#define ra8p_trng_modifyreg32(offset, clrbits, setbits) \
  ra8p_trng_putreg32(offset, \
    (ra8p_trng_getreg32(offset) & ~(clrbits)) | (setbits))

#define ra8p_rsip_putreg32(offset, val) \
  putreg32((val), RA8P_RSIP_BASE + (offset))

#define ra8p_rsip_getreg32(offset) \
  getreg32(RA8P_RSIP_BASE + (offset))

#define ra8p_rsip_modifyreg32(offset, clrbits, setbits) \
  ra8p_rsip_putreg32(offset, \
    (ra8p_rsip_getreg32(offset) & ~(clrbits)) | (setbits))

/* Entropy timeout */

#define RA8P_ENTROPY_TIMEOUT_MS                (100)

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_entropy_dev_s
{
  mutex_t lock;                     /* Thread-safe lock */
  enum ra8p_entropy_source_e source; /* Current entropy source */
  bool initialized;                 /* True if initialized */
};

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_entropy_dev_s g_entropy_priv;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_trng_wait_ready
 ****************************************************************************/

static int ra8p_trng_wait_ready(void)
{
  uint32_t timeout = RA8P_ENTROPY_TIMEOUT_MS * 1000;
  uint32_t status;

  while (timeout > 0)
    {
      status = ra8p_trng_getreg32(RA8P_TRNG_TRNGSTR_OFFSET);
      if (status & RA8P_TRNG_TRNGSTR_TRNGRDY)
        {
          return OK;
        }

      up_udelay(1);
      timeout--;
    }

  return -ETIMEDOUT;
}

/****************************************************************************
 * Name: ra8p_trng_generate
 ****************************************************************************/

static int ra8p_trng_generate(uint32_t *data, size_t len)
{
  int ret;
  size_t i;

  if (!data || len == 0)
    {
      return -EINVAL;
    }

  for (i = 0; i < len; i++)
    {
      /* Wait for TRNG to be ready */

      ret = ra8p_trng_wait_ready();
      if (ret < 0)
        {
          return ret;
        }

      /* Read random data */

      data[i] = ra8p_trng_getreg32(RA8P_TRNG_TRNGDR_OFFSET);

      /* Clear ready flag */

      ra8p_trng_putreg32(RA8P_TRNG_TRNGSTR_OFFSET, RA8P_TRNG_TRNGSTR_TRNGRDY);
    }

  return OK;
}

/****************************************************************************
 * Name: ra8p_rsip_wait_ready
 ****************************************************************************/

static int ra8p_rsip_wait_ready(void)
{
  uint32_t timeout = RA8P_ENTROPY_TIMEOUT_MS * 1000;
  uint32_t status;

  while (timeout > 0)
    {
      status = ra8p_rsip_getreg32(RA8P_RSIP_RSIPSTR_OFFSET);
      if (status & RA8P_RSIP_RSIPSTR_RSIPRDY)
        {
          return OK;
        }

      up_udelay(1);
      timeout--;
    }

  return -ETIMEDOUT;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_entropy_initialize
 *
 * Description:
 *   Initialize the entropy driver
 *
 ****************************************************************************/

int ra8p_entropy_initialize(void)
{
  struct ra8p_entropy_dev_s *priv = &g_entropy_priv;

  /* Initialize private data structure */

  memset(priv, 0, sizeof(struct ra8p_entropy_dev_s));
  priv->source = RA8P_ENTROPY_SOURCE_TRNG;
  priv->initialized = false;

  nxmutex_init(&priv->lock);

  /* Enable TRNG */

  ra8p_trng_modifyreg32(RA8P_TRNG_TRNGCR_OFFSET, 0, RA8P_TRNG_TRNGCR_TRNGEN);

  /* Wait for TRNG to be ready */

  if (ra8p_trng_wait_ready() < 0)
    {
      nxmutex_destroy(&priv->lock);
      return -ETIMEDOUT;
    }

  priv->initialized = true;

  return OK;
}

/****************************************************************************
 * Name: ra8p_entropy_generate
 *
 * Description:
 *   Generate random data
 *
 ****************************************************************************/

int ra8p_entropy_generate(uint8_t *buffer, size_t len)
{
  struct ra8p_entropy_dev_s *priv = &g_entropy_priv;
  uint32_t data[16];
  size_t remaining;
  size_t to_copy;
  size_t offset = 0;
  int ret = OK;

  if (!priv || !priv->initialized || !buffer || len == 0)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  if (priv->source == RA8P_ENTROPY_SOURCE_TRNG)
    {
      /* Generate random data in 32-bit chunks */

      while (offset < len)
        {
          remaining = len - offset;
          to_copy = (remaining < sizeof(data)) ? remaining : sizeof(data);

          ret = ra8p_trng_generate(data, to_copy / sizeof(uint32_t));
          if (ret < 0)
            {
                break;
            }

          memcpy(&buffer[offset], data, to_copy);
          offset += to_copy;
        }
    }
  else
    {
      /* RSIP-E50D source - not implemented yet */

      ret = -ENOTSUP;
    }

  nxmutex_unlock(&priv->lock);
  return ret;
}

/****************************************************************************
 * Name: ra8p_entropy_set_source
 *
 * Description:
 *   Set entropy source
 *
 ****************************************************************************/

int ra8p_entropy_set_source(enum ra8p_entropy_source_e source)
{
  struct ra8p_entropy_dev_s *priv = &g_entropy_priv;

  if (!priv || !priv->initialized)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);
  priv->source = source;
  nxmutex_unlock(&priv->lock);

  return OK;
}

/****************************************************************************
 * Name: ra8p_random_fill
 *
 * Description:
 *   Fill buffer with random data (for NuttX random pool)
 *
 ****************************************************************************/

int ra8p_random_fill(uint8_t *buffer, size_t len)
{
  return ra8p_entropy_generate(buffer, len);
}

#endif /* CONFIG_RA8P_ENTROPY */