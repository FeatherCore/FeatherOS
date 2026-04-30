/****************************************************************************
 * arch/arm/src/ra8p/ra8p_ceu.c
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
#include <nuttx/video/video.h>
#include <arch/board/board.h>

#include "arm_internal.h"
#include "chip.h"
#include "ra8p_config.h"
#include "ra8p_peripherals.h"
#include "hardware/ra8p_ceu.h"

#ifdef CONFIG_RA8P_CEU

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Helper macros for register access */

#define ra8p_ceu_putreg32(offset, val) \
  putreg32((val), RA8P_CEU_BASE + (offset))

#define ra8p_ceu_getreg32(offset) \
  getreg32(RA8P_CEU_BASE + (offset))

#define ra8p_ceu_modifyreg32(offset, clrbits, setbits) \
  ra8p_ceu_putreg32(offset, \
    (ra8p_ceu_getreg32(offset) & ~(clrbits)) | (setbits))

/* CEU timeout */

#define RA8P_CEU_TIMEOUT_MS                    (100)

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_ceu_dev_s
{
  uint32_t base;                    /* Base address of CEU registers */
  int irq;                          /* CEU interrupt number */
  mutex_t lock;                     /* Thread-safe lock */
  sem_t wait_sem;                   /* Wait semaphore for capture complete */
  struct video_format_s format;     /* Current video format */
  void *capture_buffer;             /* Capture buffer address */
  size_t capture_size;              /* Capture buffer size */
  bool initialized;                 /* True if initialized */
  bool capturing;                   /* True if capturing */
};

static struct ra8p_ceu_dev_s g_ceu_priv;

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int ra8p_ceu_start_capture(void);
static void ra8p_ceu_stop_capture(void);
static int ra8p_ceu_configure(void);

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_ceu_configure
 ****************************************************************************/

static int ra8p_ceu_configure(void)
{
  struct ra8p_ceu_dev_s *priv = &g_ceu_priv;
  uint32_t regval;

  /* Configure capture size */

  regval = (priv->format.width << RA8P_CEU_ICR1_CAPW_SHIFT) |
           (priv->format.height << RA8P_CEU_ICR1_CAPH_SHIFT);
  ra8p_ceu_putreg32(RA8P_CEU_ICR1_OFFSET, regval);

  /* Configure data format */

  regval = 0;
  switch (priv->format.format)
    {
      case VIDEO_FMT_YUV422:
        regval |= RA8P_CEU_ICR2_YCBCR;
        regval |= (RA8P_CEU_ICR2_DT_FMT_YUV422 << RA8P_CEU_ICR2_DT_FMT_SHIFT);
        break;

      case VIDEO_FMT_RGB565:
        regval |= (RA8P_CEU_ICR2_DT_FMT_RGB565 << RA8P_CEU_ICR2_DT_FMT_SHIFT);
        break;

      case VIDEO_FMT_RGB888:
        regval |= (RA8P_CEU_ICR2_DT_FMT_RGB888 << RA8P_CEU_ICR2_DT_FMT_SHIFT);
        break;

      case VIDEO_FMT_JPEG:
        regval |= (RA8P_CEU_ICR2_DT_FMT_JPEG << RA8P_CEU_ICR2_DT_FMT_SHIFT);
        regval |= RA8P_CEU_ICR2_JDT;
        break;

      default:
        return -EINVAL;
    }

  ra8p_ceu_putreg32(RA8P_CEU_ICR2_OFFSET, regval);

  return OK;
}

/****************************************************************************
 * Name: ra8p_ceu_start_capture
 ****************************************************************************/

static int ra8p_ceu_start_capture(void)
{
  struct ra8p_ceu_dev_s *priv = &g_ceu_priv;
  uint32_t regval;

  if (priv->capturing)
    {
      return OK;
    }

  /* Configure frame buffer address */

  ra8p_ceu_putreg32(RA8P_CEU_FSAA_OFFSET, (uint32_t)priv->capture_buffer);

  /* Calculate and set frame end address */

  ra8p_ceu_putreg32(RA8P_CEU_FEAA_OFFSET,
                     (uint32_t)priv->capture_buffer + priv->capture_size);

  /* Configure capture control */

  regval = RA8P_CEU_CAPCR_CE;
  if (priv->format.mode == VIDEO_MODE_CONTINUOUS)
    {
      regval |= RA8P_CEU_CAPCR_CTN;
    }

  /* Configure polarity */

  if (priv->format.vpol == VIDEO_POLARITY_HIGH)
    {
      regval |= RA8P_CEU_CAPCR_VPOL;
    }

  if (priv->format.hpol == VIDEO_POLARITY_HIGH)
    {
      regval |= RA8P_CEU_CAPCR_HPOL;
    }

  if (priv->format.dpol == VIDEO_POLARITY_HIGH)
    {
      regval |= RA8P_CEU_CAPCR_DPOL;
    }

  ra8p_ceu_putreg32(RA8P_CEU_CAPCR_OFFSET, regval);

  /* Enable DMA */

  ra8p_ceu_modifyreg32(RA8P_CEU_DMAOR_OFFSET, 0, RA8P_CEU_DMAOR_DAE);

  /* Enable interrupts */

  ra8p_ceu_modifyreg32(RA8P_CEU_CEIER_OFFSET, 0,
                        RA8P_CEU_CEIER_CEIE |
                        RA8P_CEU_CEIER_CEFEIE |
                        RA8P_CEU_CEIER_OVRFIE);

  /* Start capture */

  ra8p_ceu_modifyreg32(RA8P_CEU_CSTCR_OFFSET, 0, RA8P_CEU_CSTCR_CST);

  priv->capturing = true;

  return OK;
}

/****************************************************************************
 * Name: ra8p_ceu_stop_capture
 ****************************************************************************/

static void ra8p_ceu_stop_capture(void)
{
  struct ra8p_ceu_dev_s *priv = &g_ceu_priv;

  if (!priv->capturing)
    {
      return;
    }

  /* Disable capture */

  ra8p_ceu_modifyreg32(RA8P_CEU_CAPCR_OFFSET, RA8P_CEU_CAPCR_CE, 0);

  /* Stop capture */

  ra8p_ceu_modifyreg32(RA8P_CEU_CSTCR_OFFSET, RA8P_CEU_CSTCR_CST, 0);

  /* Disable DMA */

  ra8p_ceu_modifyreg32(RA8P_CEU_DMAOR_OFFSET, RA8P_CEU_DMAOR_DAE, 0);

  /* Disable interrupts */

  ra8p_ceu_modifyreg32(RA8P_CEU_CEIER_OFFSET,
                        RA8P_CEU_CEIER_CEIE | RA8P_CEU_CEIER_CEFEIE |
                        RA8P_CEU_CEIER_OVRFIE, 0);

  priv->capturing = false;
}

/****************************************************************************
 * Name: ra8p_ceu_interrupt
 ****************************************************************************/

static int ra8p_ceu_interrupt(int irq, void *context, void *arg)
{
  struct ra8p_ceu_dev_s *priv = (struct ra8p_ceu_dev_s *)arg;
  uint32_t status;

  if (!priv)
    {
      return OK;
    }

  /* Read interrupt status */

  status = ra8p_ceu_getreg32(RA8P_CEU_CEISR_OFFSET);

  /* Clear interrupt flags */

  ra8p_ceu_putreg32(RA8P_CEU_CEISR_OFFSET, status);

  /* Handle capture end */

  if (status & RA8P_CEU_CEISR_CEND)
    {
      priv->capturing = false;
      nxsem_post(&priv->wait_sem);
    }

  /* Handle overflow */

  if (status & RA8P_CEU_CEISR_OVRF)
    {
      /* Handle overflow - could restart capture */
    }

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_ceu_initialize
 *
 * Description:
 *   Initialize the CEU (Camera Engine Unit) driver
 *
 ****************************************************************************/

int ra8p_ceu_initialize(void)
{
  struct ra8p_ceu_dev_s *priv = &g_ceu_priv;
  int ret;

  /* Initialize private data structure */

  memset(priv, 0, sizeof(struct ra8p_ceu_dev_s));
  priv->base = RA8P_CEU_BASE;
  priv->irq = RA8P_IRQ_CEU;

  nxmutex_init(&priv->lock);
  nxsem_init(&priv->wait_sem, 0, 0);

  /* Set default format */

  priv->format.width = 640;
  priv->format.height = 480;
  priv->format.format = VIDEO_FMT_YUV422;
  priv->format.hpitch = 640 * 2; /* YUV422 = 2 bytes per pixel */
  priv->format.mode = VIDEO_MODE_CONTINUOUS;

  /* Configure default settings */

  ret = ra8p_ceu_configure();
  if (ret < 0)
    {
      goto errout_with_locks;
    }

  /* Attach interrupt handler */

  ret = irq_attach(priv->irq, ra8p_ceu_interrupt, priv);
  if (ret < 0)
    {
      goto errout_with_locks;
    }

  /* Enable interrupt */

  up_enable_irq(priv->irq);

  priv->initialized = true;

  return OK;

errout_with_locks:
  nxmutex_destroy(&priv->lock);
  nxsem_destroy(&priv->wait_sem);
  return ret;
}

/****************************************************************************
 * Name: ra8p_ceu_set_format
 *
 * Description:
 *   Set the video capture format
 *
 ****************************************************************************/

int ra8p_ceu_set_format(struct video_format_s *format)
{
  struct ra8p_ceu_dev_s *priv = &g_ceu_priv;

  if (!format)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Stop capture if running */

  if (priv->capturing)
    {
      ra8p_ceu_stop_capture();
    }

  /* Update format */

  priv->format = *format;

  /* Configure CEU */

  int ret = ra8p_ceu_configure();

  nxmutex_unlock(&priv->lock);

  return ret;
}

/****************************************************************************
 * Name: ra8p_ceu_start_stream
 *
 * Description:
 *   Start video streaming
 *
 ****************************************************************************/

int ra8p_ceu_start_stream(void *buffer, size_t size)
{
  struct ra8p_ceu_dev_s *priv = &g_ceu_priv;

  if (!buffer || size == 0)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Set capture buffer */

  priv->capture_buffer = buffer;
  priv->capture_size = size;

  /* Start capture */

  int ret = ra8p_ceu_start_capture();

  nxmutex_unlock(&priv->lock);

  return ret;
}

/****************************************************************************
 * Name: ra8p_ceu_stop_stream
 *
 * Description:
 *   Stop video streaming
 *
 ****************************************************************************/

int ra8p_ceu_stop_stream(void)
{
  struct ra8p_ceu_dev_s *priv = &g_ceu_priv;

  nxmutex_lock(&priv->lock);
  ra8p_ceu_stop_capture();
  nxmutex_unlock(&priv->lock);

  return OK;
}

/****************************************************************************
 * Name: ra8p_ceu_capture_frame
 *
 * Description:
 *   Capture a single frame
 *
 ****************************************************************************/

int ra8p_ceu_capture_frame(void *buffer, size_t size, uint32_t timeout_ms)
{
  struct ra8p_ceu_dev_s *priv = &g_ceu_priv;
  struct timespec ts;
  int ret;

  if (!buffer || size == 0)
    {
      return -EINVAL;
    }

  nxmutex_lock(&priv->lock);

  /* Set capture buffer */

  priv->capture_buffer = buffer;
  priv->capture_size = size;

  /* Start single frame capture */

  priv->format.mode = VIDEO_MODE_SINGLE;
  ra8p_ceu_configure();

  ret = ra8p_ceu_start_capture();
  if (ret < 0)
    {
      nxmutex_unlock(&priv->lock);
      return ret;
    }

  nxmutex_unlock(&priv->lock);

  /* Wait for capture complete */

  clock_gettime(CLOCK_REALTIME, &ts);
  ts.tv_sec += (timeout_ms / 1000);
  ts.tv_nsec += ((timeout_ms % 1000) * 1000000);

  ret = nxsem_timedwait(&priv->wait_sem, &ts);

  return ret;
}

#endif /* CONFIG_RA8P_CEU */