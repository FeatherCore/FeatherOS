/****************************************************************************
 * arch/arm/src/ra8p/ra8p_i3c.c
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
#include <nuttx/semaphore.h>
#include <nuttx/i3c/i3c.h>
#include <nuttx/i3c/master.h>
#include <nuttx/i3c/slave.h>
#include <arch/board/board.h>

#include "arm_internal.h"
#include "chip.h"
#include "ra8p_config.h"
#include "ra8p_peripherals.h"
#include "hardware/ra8p_icu.h"
#include "hardware/ra8p_i3c.h"

#ifdef CONFIG_RA8P_I3C0

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Helper macros for register access */

#define ra8p_i3c_putreg32(addr, offset, val) \
  putreg32((val), (addr) + (offset))

#define ra8p_i3c_getreg32(addr, offset) \
  getreg32((addr) + (offset))

/* Helper macros for bit manipulation */

#define ra8p_i3c_modifyreg32(addr, offset, clrbits, setbits) \
  ra8p_i3c_putreg32(addr, offset, \
    (ra8p_i3c_getreg32(addr, offset) & ~(clrbits)) | (setbits))

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_i3c_dev_s
{
  struct i3c_master_s master;     /* I3C master interface */
  uint32_t base;                  /* Base address of I3C registers */
  int irq;                        /* I3C interrupt number */
  mutex_t lock;                   /* Thread-safe lock */
  sem_t wait_sem;                 /* Wait semaphore for interrupt handling */
  uint32_t flags;                 /* Current operation flags */
  uint8_t *rxbuf;                 /* Receive buffer */
  uint8_t *txbuf;                 /* Transmit buffer */
  size_t rxlen;                   /* Expected receive length */
  size_t txlen;                   /* Transmit length */
  size_t rxidx;                   /* Current receive index */
  size_t txidx;                   /* Current transmit index */
  bool initialized;               /* True if initialized */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int ra8p_i3c_wait(struct ra8p_i3c_dev_s *priv, uint32_t mask, int delay);
static int ra8p_i3c_transfer_common(struct ra8p_i3c_dev_s *priv,
                                   struct i3c_msg_s *msg);
static int ra8p_i3c_setup_xfr(struct ra8p_i3c_dev_s *priv,
                              struct i3c_msg_s *msg);
static void ra8p_i3c_reset(struct ra8p_i3c_dev_s *priv);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_i3c_dev_s g_i3c0_priv;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static int ra8p_i3c_wait(struct ra8p_i3c_dev_s *priv, uint32_t mask, int delay)
{
  int ret = OK;
  int timeout = delay * 1000; /* Convert to microseconds */

  while ((ra8p_i3c_getreg32(priv->base, RA8P_I3C_MSTSTAT_OFFSET) & mask) == 0)
    {
      if (timeout <= 0)
        {
          ret = -ETIMEDOUT;
          break;
        }

      up_udelay(1);
      timeout--;
    }

  return ret;
}

static void ra8p_i3c_reset(struct ra8p_i3c_dev_s *priv)
{
  /* Disable I3C controller */
  ra8p_i3c_modifyreg32(priv->base, RA8P_I3C_MSTICR_OFFSET,
                       RA8P_I3C_MSTICR_MI3CEN, 0);

  /* Reset all relevant registers to default values */
  ra8p_i3c_putreg32(priv->base, RA8P_I3C_MSTDCTRC_OFFSET, 0);
  ra8p_i3c_putreg32(priv->base, RA8P_I3C_MSTDBCT_OFFSET, 0);
  ra8p_i3c_putreg32(priv->base, RA8P_I3C_MSTDAT_OFFSET, 0);
  ra8p_i3c_putreg32(priv->base, RA8P_I3C_MSTCRQ_OFFSET, 0);
  ra8p_i3c_putreg32(priv->base, RA8P_I3C_MSTRSP_OFFSET, 0);
  ra8p_i3c_putreg32(priv->base, RA8P_I3C_MSTSTAT_OFFSET, 0);
  ra8p_i3c_putreg32(priv->base, RA8P_I3C_MSTINT_OFFSET, 0);

  /* Re-enable I3C controller */
  ra8p_i3c_modifyreg32(priv->base, RA8P_I3C_MSTICR_OFFSET,
                       0, RA8P_I3C_MSTICR_MI3CEN);
}

static int ra8p_i3c_setup_xfr(struct ra8p_i3c_dev_s *priv,
                              struct i3c_msg_s *msg)
{
  uint32_t cmd = 0;
  uint32_t data = 0;
  uint32_t status = 0;
  int ret = OK;

  /* Initialize transfer parameters */
  priv->txbuf = msg->buffer;
  priv->txlen = msg->length;
  priv->txidx = 0;

  if (I3C_MSG_IS_DATA(msg->flags))
    {
      /* Handle data transfer */
      if (I3C_MSG_IS_READ(msg->flags))
        {
          /* Prepare for read operation */
          priv->rxbuf = msg->buffer;
          priv->rxlen = msg->length;
          priv->rxidx = 0;

          /* Set up command for read operation */
          cmd = (msg->addr << 1) | 1; /* Set read bit */
          cmd |= (1 << 8); /* Enable read operation */
        }
      else
        {
          /* Prepare for write operation */
          cmd = msg->addr << 1; /* Set write bit (0) */
          cmd |= (0 << 8); /* Enable write operation */

          /* Send first byte if available */
          if (priv->txlen > 0)
            {
              data = priv->txbuf[0];
              priv->txidx = 1;
            }
        }
    }
  else
    {
      /* Handle address-only operation */
      cmd = (msg->addr << 1) | (I3C_MSG_IS_READ(msg->flags) ? 1 : 0);
    }

  /* Set data byte count */
  ra8p_i3c_putreg32(priv->base, RA8P_I3C_MSTDBCT_OFFSET, priv->txlen & 0xFF);

  /* Set data register if transmitting */
  if (!I3C_MSG_IS_READ(msg->flags) && priv->txlen > 0)
    {
      ra8p_i3c_putreg32(priv->base, RA8P_I3C_MSTDAT_OFFSET, data);
    }

  /* Issue command request */
  ra8p_i3c_putreg32(priv->base, RA8P_I3C_MSTCRQ_OFFSET, cmd | RA8P_I3C_MSTCRQ_CRQEN);

  /* Wait for completion */
  ret = ra8p_i3c_wait(priv, RA8P_I3C_MSTSTAT_IBAMT, 1000); /* 1ms timeout */

  if (ret == OK)
    {
      /* Check for errors in status register */
      status = ra8p_i3c_getreg32(priv->base, RA8P_I3C_MSTSTAT_OFFSET);

      if (status & RA8P_I3C_MSTSTAT_MCTRL)
        {
          /* Master controller error */
          ret = -EIO;
        }
      else if (I3C_MSG_IS_READ(msg->flags))
        {
          /* Read received data */
          if (priv->rxlen > 0)
            {
              priv->rxbuf[0] = ra8p_i3c_getreg32(priv->base, RA8P_I3C_MSTDAT_OFFSET) & 0xFF;
            }
        }
    }

  return ret;
}

static int ra8p_i3c_transfer_common(struct ra8p_i3c_dev_s *priv,
                                   struct i3c_msg_s *msg)
{
  int ret = OK;
  irqstate_t flags;

  flags = enter_critical_section();

  /* Perform the I3C transfer */
  ret = ra8p_i3c_setup_xfr(priv, msg);

  leave_critical_section(flags);
  return ret;
}

static int ra8p_i3c_transfer(struct i3c_master_s *master,
                            struct i3c_msg_s *msgs, int nmsgs)
{
  struct ra8p_i3c_dev_s *priv = (struct ra8p_i3c_dev_s *)master;
  int i;
  int ret = OK;

  if (!priv || !msgs || nmsgs <= 0)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  for (i = 0; i < nmsgs; i++)
    {
      ret = ra8p_i3c_transfer_common(priv, &msgs[i]);
      if (ret != OK)
        {
          break;
        }
    }

  nxmutex_unlock(&priv->lock);
  return ret;
}

static int ra8p_i3c_reset_bus(struct i3c_master_s *master)
{
  struct ra8p_i3c_dev_s *priv = (struct ra8p_i3c_dev_s *)master;

  nxmutex_lock(&priv->lock);
  ra8p_i3c_reset(priv);
  nxmutex_unlock(&priv->lock);

  return OK;
}

static int ra8p_i3c_setup(struct i3c_master_s *master,
                         struct i3c_device_s *dev)
{
  struct ra8p_i3c_dev_s *priv = (struct ra8p_i3c_dev_s *)master;
  /* In the basic implementation, we just store the device info */
  UNUSED(priv);
  UNUSED(dev);
  return OK;
}

/****************************************************************************
 * Name: ra8p_i3c_interrupt
 *
 * Description:
 *   I3C interrupt handler
 *
 ****************************************************************************/

static int ra8p_i3c_interrupt(int irq, void *context, void *arg)
{
  struct ra8p_i3c_dev_s *priv = (struct ra8p_i3c_dev_s *)arg;
  uint32_t status;

  status = ra8p_i3c_getreg32(priv->base, RA8P_I3C_MSTSTAT_OFFSET);

  /* Clear status bits that caused the interrupt */
  ra8p_i3c_putreg32(priv->base, RA8P_I3C_MSTSTAT_OFFSET, status);

  /* Wake up waiting thread */
  nxsem_post(&priv->wait_sem);

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int ra8p_i3c_initialize(void)
{
  struct ra8p_i3c_dev_s *priv = &g_i3c0_priv;
  int ret;

  /* Initialize private data structure */
  memset(priv, 0, sizeof(struct ra8p_i3c_dev_s));
  priv->base = RA8P_I3C0_BASE;
  priv->irq = RA8P_IRQ_I3C0;

  nxmutex_init(&priv->lock);
  nxsem_init(&priv->wait_sem, 0, 0);

  /* Initialize master ops */
  priv->master.ops = &g_i3c_master_ops;

  /* Set up interrupt handler */
  ret = irq_attach(priv->irq, ra8p_i3c_interrupt, priv);
  if (ret < 0)
    {
      goto errout_with_locks;
    }

  /* Enable I3C interrupt */
  up_enable_irq(priv->irq);

  /* Reset and initialize hardware */
  ra8p_i3c_reset(priv);

  /* Register with NuttX I3C subsystem */
  ret = i3c_master_register(&priv->master, "/dev/i3c0");
  if (ret < 0)
    {
      goto errout_with_irq;
    }

  priv->initialized = true;
  return OK;

errout_with_irq:
  up_disable_irq(priv->irq);
  irq_detach(priv->irq);
errout_with_locks:
  nxmutex_destroy(&priv->lock);
  nxsem_destroy(&priv->wait_sem);
  return ret;
}

/****************************************************************************
 * Name: g_i3c_master_ops
 *
 * Description:
 *   I3C master operations
 *
 ****************************************************************************/

struct i3c_master_ops_s g_i3c_master_ops =
{
  .transfer    = ra8p_i3c_transfer,
  .reset       = ra8p_i3c_reset_bus,
  .setup       = ra8p_i3c_setup
};

#endif /* CONFIG_RA8P_I3C0 */