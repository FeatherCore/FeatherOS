/****************************************************************************
 * arch/arm/src/ra8p/ra8p_mbox.c
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
#include "hardware/ra8p_mbox.h"

#ifdef CONFIG_RA8P_MBOX

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Helper macros for register access */

#define ra8p_mbox_putreg32(offset, val) \
  putreg32((val), RA8P_MBOX_BASE + (offset))

#define ra8p_mbox_getreg32(offset) \
  getreg32(RA8P_MBOX_BASE + (offset))

#define ra8p_mbox_modifyreg32(offset, clrbits, setbits) \
  ra8p_mbox_putreg32(offset, \
    (ra8p_mbox_getreg32(offset) & ~(clrbits)) | (setbits))

/* Maximum number of channels */

#define RA8P_MBOX_MAX_CHANNELS                (8)

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_mbox_dev_s
{
  uint32_t base;                    /* Base address of MBOX registers */
  int irq;                          /* MBOX interrupt number */
  mutex_t lock;                     /* Thread-safe lock */
  struct ra8p_mbox_callback_s callbacks[RA8P_MBOX_MAX_CHANNELS]; /* Callbacks for each channel */
  bool initialized;                 /* True if initialized */
};

static struct ra8p_mbox_dev_s g_mbox_priv;

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int ra8p_mbox_enable_channel(uint8_t channel);
static int ra8p_mbox_disable_channel(uint8_t channel);
static int ra8p_mbox_send(uint8_t channel, uint32_t data);
static uint32_t ra8p_mbox_receive(uint8_t channel);

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_mbox_enable_channel
 ****************************************************************************/

static int ra8p_mbox_enable_channel(uint8_t channel)
{
  if (channel >= RA8P_MBOX_MAX_CHANNELS)
    {
      return -EINVAL;
    }

  /* Enable channel */

  ra8p_mbox_putreg32(RA8P_MBOX_CCR_OFFSET(channel),
                      RA8P_MBOX_CCR_CE);

  /* Enable channel interrupt */

  ra8p_mbox_putreg32(RA8P_MBOX_CIER_OFFSET(channel),
                      RA8P_MBOX_CIER_CIE);

  return OK;
}

/****************************************************************************
 * Name: ra8p_mbox_disable_channel
 ****************************************************************************/

static int ra8p_mbox_disable_channel(uint8_t channel)
{
  if (channel >= RA8P_MBOX_MAX_CHANNELS)
    {
      return -EINVAL;
    }

  /* Disable channel */

  ra8p_mbox_modifyreg32(RA8P_MBOX_CCR_OFFSET(channel),
                         RA8P_MBOX_CCR_CE, 0);

  /* Disable channel interrupt */

  ra8p_mbox_modifyreg32(RA8P_MBOX_CIER_OFFSET(channel),
                         RA8P_MBOX_CIER_CIE, 0);

  return OK;
}

/****************************************************************************
 * Name: ra8p_mbox_send
 ****************************************************************************/

static int ra8p_mbox_send(uint8_t channel, uint32_t data)
{
  if (channel >= RA8P_MBOX_MAX_CHANNELS)
    {
      return -EINVAL;
    }

  /* Write data to channel */

  ra8p_mbox_putreg32(RA8P_MBOX_CDR_OFFSET(channel), data);

  /* Trigger transmission by writing to control register */

  ra8p_mbox_modifyreg32(RA8P_MBOX_CCR_OFFSET(channel),
                         0, RA8P_MBOX_CCR_CTF);

  return OK;
}

/****************************************************************************
 * Name: ra8p_mbox_receive
 ****************************************************************************/

static uint32_t ra8p_mbox_receive(uint8_t channel)
{
  if (channel >= RA8P_MBOX_MAX_CHANNELS)
    {
      return 0;
    }

  /* Read data from channel */

  return ra8p_mbox_getreg32(RA8P_MBOX_CDR_OFFSET(channel));
}

/****************************************************************************
 * Name: ra8p_mbox_interrupt
 ****************************************************************************/

static int ra8p_mbox_interrupt(int irq, void *context, void *arg)
{
  struct ra8p_mbox_dev_s *priv = (struct ra8p_mbox_dev_s *)arg;
  uint32_t ccr;
  uint32_t channel;

  if (!priv)
    {
      return OK;
    }

  /* Check each channel for received data */

  for (channel = 0; channel < RA8P_MBOX_MAX_CHANNELS; channel++)
    {
      ccr = ra8p_mbox_getreg32(RA8P_MBOX_CCR_OFFSET(channel));

      if (ccr & RA8P_MBOX_CCR_CRF)
        {
          /* Data received on this channel */

          uint32_t data = ra8p_mbox_receive(channel);

          /* Clear receive flag */

          ra8p_mbox_modifyreg32(RA8P_MBOX_CCR_OFFSET(channel),
                                 RA8P_MBOX_CCR_CRF, 0);

          /* Call callback if registered */

          if (priv->callbacks[channel].received)
            {
              priv->callbacks[channel].received(data);
            }
        }

      if (ccr & RA8P_MBOX_CCR_CTF)
        {
          /* Data transmitted on this channel */

          /* Clear transmit flag */

          ra8p_mbox_modifyreg32(RA8P_MBOX_CCR_OFFSET(channel),
                                 RA8P_MBOX_CCR_CTF, 0);

          /* Call callback if registered */

          if (priv->callbacks[channel].transmitted)
            {
              priv->callbacks[channel].transmitted();
            }
        }
    }

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_mbox_initialize
 *
 * Description:
 *   Initialize the MBOX (Mailbox) driver for inter-processor communication
 *
 ****************************************************************************/

int ra8p_mbox_initialize(void)
{
  struct ra8p_mbox_dev_s *priv = &g_mbox_priv;
  int ret;
  uint8_t channel;

  /* Initialize private data structure */

  memset(priv, 0, sizeof(struct ra8p_mbox_dev_s));
  priv->base = RA8P_MBOX_BASE;
  priv->irq = RA8P_IRQ_MBOX;
  priv->num_channels = RA8P_MBOX_MAX_CHANNELS;

  nxmutex_init(&priv->lock);

  /* Reset mailbox */

  ra8p_mbox_modifyreg32(RA8P_MBOX_MBXCR_OFFSET, 0, RA8P_MBOX_MBXCR_MBRESET);
  up_udelay(10);
  ra8p_mbox_modifyreg32(RA8P_MBOX_MBXCR_OFFSET, RA8P_MBOX_MBXCR_MBRESET, 0);

  /* Disable all channels initially */

  for (channel = 0; channel < RA8P_MBOX_MAX_CHANNELS; channel++)
    {
      ra8p_mbox_disable_channel(channel);
    }

  /* Enable mailbox */

  ra8p_mbox_modifyreg32(RA8P_MBOX_MBXCR_OFFSET, 0, RA8P_MBOX_MBXCR_MBEN);

  /* Attach interrupt handler */

  ret = irq_attach(priv->irq, ra8p_mbox_interrupt, priv);
  if (ret < 0)
    {
      nxmutex_destroy(&priv->lock);
      return ret;
    }

  /* Enable interrupt */

  up_enable_irq(priv->irq);

  priv->initialized = true;

  return OK;
}

/****************************************************************************
 * Name: ra8p_mbox_register_callback
 *
 * Description:
 *   Register callback for a mailbox channel
 *
 ****************************************************************************/

int ra8p_mbox_register_callback(uint8_t channel,
                               struct ra8p_mbox_callback_s *callback)
{
  struct ra8p_mbox_dev_s *priv = &g_mbox_priv;

  if (channel >= RA8P_MBOX_MAX_CHANNELS || !callback)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Store callback */

  priv->callbacks[channel] = *callback;

  /* Enable channel */

  ra8p_mbox_enable_channel(channel);

  nxmutex_unlock(&priv->lock);

  return OK;
}

/****************************************************************************
 * Name: ra8p_mbox_unregister_callback
 *
 * Description:
 *   Unregister callback for a mailbox channel
 *
 ****************************************************************************/

int ra8p_mbox_unregister_callback(uint8_t channel)
{
  struct ra8p_mbox_dev_s *priv = &g_mbox_priv;
  struct ra8p_mbox_callback_s empty_callback = {0};

  if (channel >= RA8P_MBOX_MAX_CHANNELS)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Disable channel */

  ra8p_mbox_disable_channel(channel);

  /* Clear callback */

  priv->callbacks[channel] = empty_callback;

  nxmutex_unlock(&priv->lock);

  return OK;
}

/****************************************************************************
 * Name: ra8p_mbox_send_data
 *
 * Description:
 *   Send data to another processor
 *
 ****************************************************************************/

int ra8p_mbox_send_data(uint8_t channel, uint32_t data)
{
  struct ra8p_mbox_dev_s *priv = &g_mbox_priv;

  if (channel >= RA8P_MBOX_MAX_CHANNELS)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  int ret = ra8p_mbox_send(channel, data);

  nxmutex_unlock(&priv->lock);

  return ret;
}

/****************************************************************************
 * Name: ra8p_mbox_receive_data
 *
 * Description:
 *   Receive data from another processor
 *
 ****************************************************************************/

uint32_t ra8p_mbox_receive_data(uint8_t channel)
{
  struct ra8p_mbox_dev_s *priv = &g_mbox_priv;
  uint32_t data;

  if (channel >= RA8P_MBOX_MAX_CHANNELS)
    {
      return 0;
    }

  nxmutex_lock(&priv->lock);

  data = ra8p_mbox_receive(channel);

  nxmutex_unlock(&priv->lock);

  return data;
}

#endif /* CONFIG_RA8P_MBOX */