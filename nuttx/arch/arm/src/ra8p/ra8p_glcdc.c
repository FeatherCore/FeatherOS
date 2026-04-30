/****************************************************************************
 * arch/arm/src/ra8p/ra8p_glcdc.c
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
#include <nuttx/video/fb.h>
#include <arch/board/board.h>

#include "arm_internal.h"
#include "chip.h"
#include "ra8p_config.h"
#include "ra8p_peripherals.h"
#include "hardware/ra8p_glcdc.h"

#ifdef CONFIG_RA8P_GLCDC

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Helper macros for register access */

#define ra8p_glcdc_putreg32(base, offset, val) \
  putreg32((val), (base) + (offset))

#define ra8p_glcdc_getreg32(base, offset) \
  getreg32((base) + (offset))

#define ra8p_glcdc_modifyreg32(base, offset, clrbits, setbits) \
  ra8p_glcdc_putreg32(base, offset, \
    (ra8p_glcdc_getreg32(base, offset) & ~(clrbits)) | (setbits))

/* Color format definitions */

#define RA8P_GLCDC_FORMAT_RGB565              (0)
#define RA8P_GLCDC_FORMAT_RGB888              (1)
#define RA8P_GLCDC_FORMAT_ARGB8888            (2)

/* Bytes per pixel for each format */

#define RA8P_GLCDC_BPP_RGB565                 (2)
#define RA8P_GLCDC_BPP_RGB888                 (3)
#define RA8P_GLCDC_BPP_ARGB8888               (4)

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_glcdc_dev_s
{
  struct fb_dev_s dev;              /* Framebuffer interface */
  uint32_t base;                    /* Base address of GLCDC registers */
  int irq;                          /* GLCDC interrupt number */
  mutex_t lock;                     /* Thread-safe lock */
  struct fb_videoinfo_s vinfo;      /* Video information */
  struct fb_planeinfo_s pinfo;      /* Plane information */
  void *fbmem;                      /* Framebuffer memory */
  size_t fblen;                     /* Framebuffer length */
  bool initialized;                 /* True if initialized */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int ra8p_glcdc_getvideoinfo(struct fb_dev_s *dev,
                                    struct fb_videoinfo_s *vinfo);
static int ra8p_glcdc_getplaneinfo(struct fb_dev_s *dev, int planeno,
                                    struct fb_planeinfo_s *pinfo);
static int ra8p_glcdc_updatearea(struct fb_dev_s *dev,
                                  struct fb_area_s *area);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_glcdc_dev_s g_glcdc_priv;

static const struct fb_ops_s g_glcdc_ops =
{
  .getvideoinfo = ra8p_glcdc_getvideoinfo,
  .getplaneinfo = ra8p_glcdc_getplaneinfo,
  .updatearea   = ra8p_glcdc_updatearea,
};

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_glcdc_getvideoinfo
 ****************************************************************************/

static int ra8p_glcdc_getvideoinfo(struct fb_dev_s *dev,
                                    struct fb_videoinfo_s *vinfo)
{
  struct ra8p_glcdc_dev_s *priv = (struct ra8p_glcdc_dev_s *)dev;

  if (!priv || !vinfo)
    {
      return -EINVAL;
    }

  memcpy(vinfo, &priv->vinfo, sizeof(struct fb_videoinfo_s));
  return OK;
}

/****************************************************************************
 * Name: ra8p_glcdc_getplaneinfo
 ****************************************************************************/

static int ra8p_glcdc_getplaneinfo(struct fb_dev_s *dev, int planeno,
                                    struct fb_planeinfo_s *pinfo)
{
  struct ra8p_glcdc_dev_s *priv = (struct ra8p_glcdc_dev_s *)dev;

  if (!priv || !pinfo || planeno != 0)
    {
      return -EINVAL;
    }

  memcpy(pinfo, &priv->pinfo, sizeof(struct fb_planeinfo_s));
  return OK;
}

/****************************************************************************
 * Name: ra8p_glcdc_updatearea
 ****************************************************************************/

static int ra8p_glcdc_updatearea(struct fb_dev_s *dev,
                                  struct fb_area_s *area)
{
  /* For GLCDC, the framebuffer is directly displayed, so no action needed */

  return OK;
}

/****************************************************************************
 * Name: ra8p_glcdc_configure_timing
 ****************************************************************************/

static void ra8p_glcdc_configure_timing(struct ra8p_glcdc_dev_s *priv,
                                         struct ra8p_glcdc_timing_s *timing)
{
  uint32_t regval;

  /* Configure horizontal timing */

  regval = (timing->hactive << RA8P_GLCDC_TCON_HACT_SHIFT) |
           (timing->hsync << RA8P_GLCDC_TCON_HSYNC_SHIFT);
  if (timing->hpol)
    {
      regval |= RA8P_GLCDC_TCON_HPOL;
    }
  ra8p_glcdc_putreg32(priv->base, RA8P_GLCDC_TCON_OFFSET, regval);

  /* Configure vertical timing */

  regval = (timing->vactive << RA8P_GLCDC_TCON2_VACT_SHIFT) |
           (timing->vsync << RA8P_GLCDC_TCON2_VSYNC_SHIFT);
  if (timing->vpol)
    {
      regval |= RA8P_GLCDC_TCON2_VPOL;
    }
  ra8p_glcdc_putreg32(priv->base, RA8P_GLCDC_TCON2_OFFSET, regval);

  /* Configure porches */

  regval = (timing->hbp << RA8P_GLCDC_TCON3_HBP_SHIFT) |
           (timing->hfp << RA8P_GLCDC_TCON3_HFP_SHIFT);
  ra8p_glcdc_putreg32(priv->base, RA8P_GLCDC_TCON3_OFFSET, regval);

  regval = (timing->vbp << RA8P_GLCDC_TCON4_VBP_SHIFT) |
           (timing->vfp << RA8P_GLCDC_TCON4_VFP_SHIFT);
  ra8p_glcdc_putreg32(priv->base, RA8P_GLCDC_TCON4_OFFSET, regval);
}

/****************************************************************************
 * Name: ra8p_glcdc_configure_layer
 ****************************************************************************/

static void ra8p_glcdc_configure_layer(struct ra8p_glcdc_dev_s *priv,
                                        struct ra8p_glcdc_layer_s *layer,
                                        int layer_num)
{
  uint32_t ctl_offset;
  uint32_t fb_offset;
  uint32_t frm_offset;
  uint32_t da_offset;
  uint32_t regval;

  if (layer_num == 1)
    {
      ctl_offset = RA8P_GLCDC_GR1CTL_OFFSET;
      fb_offset = RA8P_GLCDC_GR1FB_OFFSET;
      frm_offset = RA8P_GLCDC_GR1FRM_OFFSET;
      da_offset = RA8P_GLCDC_GR1DA_OFFSET;
    }
  else
    {
      ctl_offset = RA8P_GLCDC_GR2CTL_OFFSET;
      fb_offset = RA8P_GLCDC_GR2FB_OFFSET;
      frm_offset = RA8P_GLCDC_GR2FRM_OFFSET;
      da_offset = RA8P_GLCDC_GR2DA_OFFSET;
    }

  /* Set frame buffer address */

  ra8p_glcdc_putreg32(priv->base, fb_offset, layer->fb_addr);

  /* Set color format */

  regval = (layer->format << RA8P_GLCDC_GR1FRM_CFMT_SHIFT);
  ra8p_glcdc_putreg32(priv->base, frm_offset, regval);

  /* Set display area */

  regval = (layer->width << RA8P_GLCDC_GR1DA_WIDTH_SHIFT) |
           (layer->height << RA8P_GLCDC_GR1DA_HEIGHT_SHIFT);
  ra8p_glcdc_putreg32(priv->base, da_offset, regval);

  /* Enable layer if requested */

  if (layer->enabled)
    {
      ra8p_glcdc_modifyreg32(priv->base, ctl_offset, 0, RA8P_GLCDC_GR1CTL_GR1EN);
    }
}

/****************************************************************************
 * Name: ra8p_glcdc_interrupt
 ****************************************************************************/

static int ra8p_glcdc_interrupt(int irq, void *context, void *arg)
{
  struct ra8p_glcdc_dev_s *priv = (struct ra8p_glcdc_dev_s *)arg;
  uint32_t status;

  if (!priv)
    {
      return OK;
    }

  /* Read interrupt status */

  status = ra8p_glcdc_getreg32(priv->base, RA8P_GLCDC_INTST_OFFSET);

  /* Clear interrupt flags */

  ra8p_glcdc_putreg32(priv->base, RA8P_GLCDC_INTST_OFFSET, status);

  /* Handle vertical sync interrupt */

  if (status & RA8P_GLCDC_INTST_VINT)
    {
      /* Could be used for double-buffering or frame synchronization */
    }

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_glcdc_initialize
 *
 * Description:
 *   Initialize the GLCDC driver
 *
 ****************************************************************************/

int ra8p_glcdc_initialize(void)
{
  struct ra8p_glcdc_dev_s *priv = &g_glcdc_priv;
  struct ra8p_glcdc_timing_s timing;
  struct ra8p_glcdc_layer_s layer;
  int ret;

  /* Initialize private data structure */

  memset(priv, 0, sizeof(struct ra8p_glcdc_dev_s));
  priv->base = RA8P_GLCDC_BASE;
  priv->irq = RA8P_IRQ_GLCDC;

  nxmutex_init(&priv->lock);

  /* Initialize framebuffer ops */

  priv->dev.ops = &g_glcdc_ops;

  /* Set default display configuration (800x480) */

  timing.hactive = 800;
  timing.vactive = 480;
  timing.hsync = 1;
  timing.vsync = 1;
  timing.hbp = 45;
  timing.hfp = 210;
  timing.vbp = 22;
  timing.vfp = 22;
  timing.hpol = false;
  timing.vpol = false;

  /* Configure timing */

  ra8p_glcdc_configure_timing(priv, &timing);

  /* Allocate framebuffer memory */

  priv->fblen = timing.hactive * timing.vactive * RA8P_GLCDC_BPP_RGB565;
  priv->fbmem = kmm_malloc(priv->fblen);
  if (!priv->fbmem)
    {
      ret = -ENOMEM;
      goto errout_with_mutex;
    }

  memset(priv->fbmem, 0, priv->fblen);

  /* Configure layer 1 */

  layer.fb_addr = (uint32_t)priv->fbmem;
  layer.width = timing.hactive;
  layer.height = timing.vactive;
  layer.format = RA8P_GLCDC_FORMAT_RGB565;
  layer.bpp = RA8P_GLCDC_BPP_RGB565;
  layer.enabled = true;

  ra8p_glcdc_configure_layer(priv, &layer, 1);

  /* Setup video info */

  priv->vinfo.fmt = FB_FMT_RGB16;
  priv->vinfo.xres = timing.hactive;
  priv->vinfo.yres = timing.vactive;
  priv->vinfo.nplanes = 1;

  /* Setup plane info */

  priv->pinfo.fbmem = priv->fbmem;
  priv->pinfo.fblen = priv->fblen;
  priv->pinfo.stride = timing.hactive * RA8P_GLCDC_BPP_RGB565;
  priv->pinfo.display = 0;
  priv->pinfo.bpp = 16;

  /* Configure output control */

  ra8p_glcdc_putreg32(priv->base, RA8P_GLCDC_OUTCTL_OFFSET,
                      RA8P_GLCDC_OUTCTL_OUTEN);

  /* Enable interrupts */

  ra8p_glcdc_modifyreg32(priv->base, RA8P_GLCDC_INTEN_OFFSET,
                         0, RA8P_GLCDC_INTEN_VINTEN);

  /* Attach interrupt handler */

  ret = irq_attach(priv->irq, ra8p_glcdc_interrupt, priv);
  if (ret < 0)
    {
      goto errout_with_fbmem;
    }

  /* Enable interrupt */

  up_enable_irq(priv->irq);

  priv->initialized = true;

  /* Register framebuffer device */

  ret = fb_register(&priv->dev, 0);
  if (ret < 0)
    {
      goto errout_with_irq;
    }

  return OK;

errout_with_irq:
  up_disable_irq(priv->irq);
  irq_detach(priv->irq);
errout_with_fbmem:
  kmm_free(priv->fbmem);
errout_with_mutex:
  nxmutex_destroy(&priv->lock);
  return ret;
}

/****************************************************************************
 * Name: ra8p_glcdc_set_brightness
 *
 * Description:
 *   Set display brightness
 *
 ****************************************************************************/

int ra8p_glcdc_set_brightness(uint8_t brightness)
{
  struct ra8p_glcdc_dev_s *priv = &g_glcdc_priv;

  if (!priv || !priv->initialized)
    {
      return -EINVAL;
    }

  ra8p_glcdc_putreg32(priv->base, RA8P_GLCDC_BRIGHT_OFFSET,
                      brightness << RA8P_GLCDC_BRIGHT_BRTH_SHIFT);

  return OK;
}

/****************************************************************************
 * Name: ra8p_glcdc_set_contrast
 *
 * Description:
 *   Set display contrast
 *
 ****************************************************************************/

int ra8p_glcdc_set_contrast(uint8_t contrast)
{
  struct ra8p_glcdc_dev_s *priv = &g_glcdc_priv;

  if (!priv || !priv->initialized)
    {
      return -EINVAL;
    }

  ra8p_glcdc_putreg32(priv->base, RA8P_GLCDC_CONTRAST_OFFSET,
                      contrast << RA8P_GLCDC_CONTRAST_CONT_SHIFT);

  return OK;
}

#endif /* CONFIG_RA8P_GLCDC */