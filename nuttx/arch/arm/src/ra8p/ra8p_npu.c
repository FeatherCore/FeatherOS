/****************************************************************************
 * arch/arm/src/ra8p/ra8p_npu.c
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
#include "hardware/ra8p_npu.h"

#ifdef CONFIG_RA8P_NPU

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Helper macros for register access */

#define ra8p_npu_putreg32(offset, val) \
  putreg32((val), RA8P_NPU_BASE + (offset))

#define ra8p_npu_getreg32(offset) \
  getreg32(RA8P_NPU_BASE + (offset))

#define ra8p_npu_modifyreg32(offset, clrbits, setbits) \
  ra8p_npu_putreg32(offset, \
    (ra8p_npu_getreg32(offset) & ~(clrbits)) | (setbits))

/* NPU timeout */

#define RA8P_NPU_TIMEOUT_MS                    (1000)

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_npu_dev_s
{
  uint32_t base;                    /* Base address of NPU registers */
  int irq;                          /* NPU interrupt number */
  mutex_t lock;                     /* Thread-safe lock */
  sem_t wait_sem;                   /* Wait semaphore for operation completion */
  struct ra8p_npu_config_s config;  /* Current NPU configuration */
  bool initialized;                 /* True if initialized */
  bool active;                      /* True if currently processing */
};

static struct ra8p_npu_dev_s g_npu_priv;

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int ra8p_npu_configure(struct ra8p_npu_config_s *config);
static int ra8p_npu_execute(void);
static void ra8p_npu_reset(void);

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_npu_reset
 ****************************************************************************/

static void ra8p_npu_reset(void)
{
  ra8p_npu_modifyreg32(RA8P_NPU_NPUCR_OFFSET, 0, RA8P_NPU_NPUCR_NPURESET);
  up_udelay(10);
  ra8p_npu_modifyreg32(RA8P_NPU_NPUCR_OFFSET, RA8P_NPU_NPUCR_NPURESET, 0);
}

/****************************************************************************
 * Name: ra8p_npu_configure
 ****************************************************************************/

static int ra8p_npu_configure(struct ra8p_npu_config_s *config)
{
  uint32_t regval;

  if (!config)
    {
      return -EINVAL;
    }

  /* Configure NPU configuration register */

  regval = (config->macs << RA8P_NPU_NPUCFGR_MACS_SHIFT) |
           (config->planes << RA8P_NPU_NPUCFGR_PLANES_SHIFT) |
           (config->shram << RA8P_NPU_NPUCFGR_SHRAM_SHIFT);

  ra8p_npu_putreg32(RA8P_NPU_NPUCFGR_OFFSET, regval);

  /* Set buffer addresses */

  ra8p_npu_putreg32(RA8P_NPU_NPUINBR_OFFSET, (uint32_t)config->input_buffer);
  ra8p_npu_putreg32(RA8P_NPU_NPUOUTBR_OFFSET, (uint32_t)config->output_buffer);
  ra8p_npu_putreg32(RA8P_NPU_NPUWEIBR_OFFSET, (uint32_t)config->weight_buffer);
  ra8p_npu_putreg32(RA8P_NPU_NPUCTLBR_OFFSET, (uint32_t)config->control_buffer);

  /* Configure timing and power */

  regval = 0;
  if (config->clock_enable)
    {
      regval |= RA8P_NPU_NPUTIMR_CLKEN;
    }

  if (config->power_gate)
    {
      regval |= RA8P_NPU_NPUTIMR_PWRGATE;
    }

  ra8p_npu_putreg32(RA8P_NPU_NPUTIMR_OFFSET, regval);

  return OK;
}

/****************************************************************************
 * Name: ra8p_npu_execute
 ****************************************************************************/

static int ra8p_npu_execute(void)
{
  /* Set command to execute */

  ra8p_npu_putreg32(RA8P_NPU_NPUCOMR_OFFSET, RA8P_NPU_NPUCOMR_CMD_EXECUTE);

  return OK;
}

/****************************************************************************
 * Name: ra8p_npu_wait_completion
 ****************************************************************************/

static int ra8p_npu_wait_completion(uint32_t timeout_ms)
{
  struct timespec ts;
  int ret;

  clock_gettime(CLOCK_REALTIME, &ts);
  ts.tv_sec += (timeout_ms / 1000);
  ts.tv_nsec += ((timeout_ms % 1000) * 1000000);

  ret = nxsem_timedwait(&g_npu_priv.wait_sem, &ts);

  return ret;
}

/****************************************************************************
 * Name: ra8p_npu_interrupt
 ****************************************************************************/

static int ra8p_npu_interrupt(int irq, void *context, void *arg)
{
  struct ra8p_npu_dev_s *priv = (struct ra8p_npu_dev_s *)arg;
  uint32_t status;

  if (!priv)
    {
      return OK;
    }

  /* Read interrupt status */

  status = ra8p_npu_getreg32(RA8P_NPU_NPUINTSR_OFFSET);

  /* Clear interrupt flags */

  ra8p_npu_putreg32(RA8P_NPU_NPUINTSR_OFFSET, status);

  /* Handle completion */

  if (status & RA8P_NPU_NPUINTSR_DONE)
    {
      priv->active = false;
      nxsem_post(&priv->wait_sem);
    }

  /* Handle error */

  if (status & RA8P_NPU_NPUINTSR_ERROR)
    {
      /* Error occurred - need to handle appropriately */
    }

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_npu_initialize
 *
 * Description:
 *   Initialize the NPU (Neural Processing Unit) driver
 *
 ****************************************************************************/

int ra8p_npu_initialize(void)
{
  struct ra8p_npu_dev_s *priv = &g_npu_priv;
  int ret;

  /* Initialize private data structure */

  memset(priv, 0, sizeof(struct ra8p_npu_dev_s));
  priv->base = RA8P_NPU_BASE;
  priv->irq = RA8P_IRQ_NPU;

  nxmutex_init(&priv->lock);
  nxsem_init(&priv->wait_sem, 0, 0);

  /* Reset NPU */

  ra8p_npu_reset();

  /* Set default configuration */

  priv->config.macs = RA8P_NPU_MACS_512;
  priv->config.planes = RA8P_NPU_PLANES_2;
  priv->config.shram = RA8P_NPU_SHRAM_256KB;
  priv->config.clock_enable = true;
  priv->config.power_gate = false;

  /* Configure NPU */

  ret = ra8p_npu_configure(&priv->config);
  if (ret < 0)
    {
      goto errout_with_locks;
    }

  /* Enable interrupts */

  ra8p_npu_modifyreg32(RA8P_NPU_NPUINTENR_OFFSET, 0,
                       RA8P_NPU_NPUINTENR_DONE | RA8P_NPU_NPUINTENR_ERROR);

  /* Attach interrupt handler */

  ret = irq_attach(priv->irq, ra8p_npu_interrupt, priv);
  if (ret < 0)
    {
      goto errout_with_locks;
    }

  /* Enable interrupt */

  up_enable_irq(priv->irq);

  /* Enable NPU */

  ra8p_npu_modifyreg32(RA8P_NPU_NPUCR_OFFSET, 0, RA8P_NPU_NPUCR_NPUE);

  priv->initialized = true;

  return OK;

errout_with_locks:
  nxmutex_destroy(&priv->lock);
  nxsem_destroy(&priv->wait_sem);
  return ret;
}

/****************************************************************************
 * Name: ra8p_npu_set_buffers
 *
 * Description:
 *   Set NPU buffer addresses
 *
 ****************************************************************************/

int ra8p_npu_set_buffers(void *input, void *output, void *weights, void *control)
{
  struct ra8p_npu_dev_s *priv = &g_npu_priv;

  if (!input || !output || !weights || !control)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  priv->config.input_buffer = input;
  priv->config.output_buffer = output;
  priv->config.weight_buffer = weights;
  priv->config.control_buffer = control;

  /* Update registers */

  ra8p_npu_putreg32(RA8P_NPU_NPUINBR_OFFSET, (uint32_t)input);
  ra8p_npu_putreg32(RA8P_NPU_NPUOUTBR_OFFSET, (uint32_t)output);
  ra8p_npu_putreg32(RA8P_NPU_NPUWEIBR_OFFSET, (uint32_t)weights);
  ra8p_npu_putreg32(RA8P_NPU_NPUCTLBR_OFFSET, (uint32_t)control);

  nxmutex_unlock(&priv->lock);

  return OK;
}

/****************************************************************************
 * Name: ra8p_npu_run_model
 *
 * Description:
 *   Run a neural network model on the NPU
 *
 ****************************************************************************/

int ra8p_npu_run_model(void *input, void *output, void *weights, void *control,
                      uint32_t timeout_ms)
{
  struct ra8p_npu_dev_s *priv = &g_npu_priv;
  int ret;

  if (!input || !output || !weights || !control || timeout_ms == 0)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Set buffers */

  ret = ra8p_npu_set_buffers(input, output, weights, control);
  if (ret < 0)
    {
      nxmutex_unlock(&priv->lock);
      return ret;
    }

  /* Start processing */

  priv->active = true;

  ret = ra8p_npu_execute();
  if (ret < 0)
    {
      priv->active = false;
      nxmutex_unlock(&priv->lock);
      return ret;
    }

  nxmutex_unlock(&priv->lock);

  /* Wait for completion */

  ret = ra8p_npu_wait_completion(timeout_ms);

  return ret;
}

/****************************************************************************
 * Name: ra8p_npu_get_status
 *
 * Description:
 *   Get NPU status
 *
 ****************************************************************************/

uint32_t ra8p_npu_get_status(void)
{
  struct ra8p_npu_dev_s *priv = &g_npu_priv;

  if (!priv || !priv->initialized)
    {
      return 0;
    }

  return ra8p_npu_getreg32(RA8P_NPU_NPUSR_OFFSET);
}

/****************************************************************************
 * Name: ra8p_npu_is_busy
 *
 * Description:
 *   Check if NPU is busy processing
 *
 ****************************************************************************/

bool ra8p_npu_is_busy(void)
{
  struct ra8p_npu_dev_s *priv = &g_npu_priv;

  if (!priv || !priv->initialized)
    {
      return false;
    }

  uint32_t status = ra8p_npu_getreg32(RA8P_NPU_NPUSR_OFFSET);
  return !(status & RA8P_NPU_NPUSR_NPUIDLE);
}

#endif /* CONFIG_RA8P_NPU */