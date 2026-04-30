/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_glcdc.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_GLCDC_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_GLCDC_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* GLCDC Base Address */

#define RA8P_GLCDC_BASE                        (0x40342000)

/* GLCDC Register Offsets */

/* Graphics Layer 1 Control Register (GR1CTL) */

#define RA8P_GLCDC_GR1CTL_OFFSET               (0x000)
#define RA8P_GLCDC_GR1CTL_GR1EN                (1 << 0)    /* Bit 0: Graphics Layer 1 Enable */
#define RA8P_GLCDC_GR1CTL_GR1RST               (1 << 1)    /* Bit 1: Graphics Layer 1 Reset */

/* Graphics Layer 2 Control Register (GR2CTL) */

#define RA8P_GLCDC_GR2CTL_OFFSET               (0x004)
#define RA8P_GLCDC_GR2CTL_GR2EN                (1 << 0)    /* Bit 0: Graphics Layer 2 Enable */
#define RA8P_GLCDC_GR2CTL_GR2RST               (1 << 1)    /* Bit 1: Graphics Layer 2 Reset */

/* Graphics Layer 1 Frame Buffer Address Register (GR1FB) */

#define RA8P_GLCDC_GR1FB_OFFSET                (0x008)
#define RA8P_GLCDC_GR1FB_FBA_MASK              (0xFFFFFFFF) /* Bits 0-31: Frame Buffer Address */

/* Graphics Layer 2 Frame Buffer Address Register (GR2FB) */

#define RA8P_GLCDC_GR2FB_OFFSET                (0x00C)
#define RA8P_GLCDC_GR2FB_FBA_MASK              (0xFFFFFFFF) /* Bits 0-31: Frame Buffer Address */

/* Graphics Layer 1 Format Register (GR1FRM) */

#define RA8P_GLCDC_GR1FRM_OFFSET               (0x010)
#define RA8P_GLCDC_GR1FRM_CFMT_MASK            (0x7)       /* Bits 0-2: Color Format */
#define RA8P_GLCDC_GR1FRM_CFMT_SHIFT           (0)
#define RA8P_GLCDC_GR1FRM_CFMT_RGB565          (0x0)       /* RGB565 */
#define RA8P_GLCDC_GR1FRM_CFMT_RGB888          (0x1)       /* RGB888 */
#define RA8P_GLCDC_GR1FRM_CFMT_ARGB8888        (0x2)       /* ARGB8888 */
#define RA8P_GLCDC_GR1FRM_CFMT_CLUT8           (0x4)       /* 8-bit CLUT */
#define RA8P_GLCDC_GR1FRM_CFMT_CLUT4           (0x5)       /* 4-bit CLUT */
#define RA8P_GLCDC_GR1FRM_BPP_MASK             (0x70)      /* Bits 4-6: Bytes Per Pixel */
#define RA8P_GLCDC_GR1FRM_BPP_SHIFT            (4)

/* Graphics Layer 2 Format Register (GR2FRM) */

#define RA8P_GLCDC_GR2FRM_OFFSET               (0x014)
#define RA8P_GLCDC_GR2FRM_CFMT_MASK            (0x7)       /* Bits 0-2: Color Format */
#define RA8P_GLCDC_GR2FRM_CFMT_SHIFT           (0)

/* Graphics Layer 1 Display Area Register (GR1DA) */

#define RA8P_GLCDC_GR1DA_OFFSET                (0x018)
#define RA8P_GLCDC_GR1DA_WIDTH_MASK            (0x7FF)     /* Bits 0-10: Width */
#define RA8P_GLCDC_GR1DA_WIDTH_SHIFT           (0)
#define RA8P_GLCDC_GR1DA_HEIGHT_MASK           (0x7FF0000) /* Bits 16-26: Height */
#define RA8P_GLCDC_GR1DA_HEIGHT_SHIFT          (16)

/* Graphics Layer 2 Display Area Register (GR2DA) */

#define RA8P_GLCDC_GR2DA_OFFSET                (0x01C)
#define RA8P_GLCDC_GR2DA_WIDTH_MASK            (0x7FF)     /* Bits 0-10: Width */
#define RA8P_GLCDC_GR2DA_WIDTH_SHIFT           (0)
#define RA8P_GLCDC_GR2DA_HEIGHT_MASK           (0x7FF0000) /* Bits 16-26: Height */
#define RA8P_GLCDC_GR2DA_HEIGHT_SHIFT          (16)

/* Background Control Register (BGCTL) */

#define RA8P_GLCDC_BGCTL_OFFSET                (0x020)
#define RA8P_GLCDC_BGCTL_BGEN                  (1 << 0)    /* Bit 0: Background Enable */
#define RA8P_GLCDC_BGCTL_BGCLR_MASK            (0xFFFFFF00) /* Bits 8-31: Background Color */
#define RA8P_GLCDC_BGCTL_BGCLR_SHIFT           (8)

/* Output Control Register (OUTCTL) */

#define RA8P_GLCDC_OUTCTL_OFFSET               (0x024)
#define RA8P_GLCDC_OUTCTL_OUTEN                (1 << 0)    /* Bit 0: Output Enable */
#define RA8P_GLCDC_OUTCTL_OUTSEL_MASK          (0x6)       /* Bits 1-2: Output Select */
#define RA8P_GLCDC_OUTCTL_OUTSEL_SHIFT         (1)
#define RA8P_GLCDC_OUTCTL_OUTSEL_LCD           (0x0 << 1)  /* LCD output */
#define RA8P_GLCDC_OUTCTL_OUTSEL_DSI           (0x1 << 1)  /* DSI output */
#define RA8P_GLCDC_OUTCTL_DITHER               (1 << 3)    /* Bit 3: Dither Enable */

/* Timing Control Register (TCON) */

#define RA8P_GLCDC_TCON_OFFSET                 (0x028)
#define RA8P_GLCDC_TCON_HACT_MASK              (0x7FF)     /* Bits 0-10: Horizontal Active */
#define RA8P_GLCDC_TCON_HACT_SHIFT             (0)
#define RA8P_GLCDC_TCON_HSYNC_MASK             (0x7F0000)  /* Bits 16-22: Horizontal Sync Width */
#define RA8P_GLCDC_TCON_HSYNC_SHIFT            (16)
#define RA8P_GLCDC_TCON_HPOL                   (1 << 23)   /* Bit 23: Horizontal Sync Polarity */

/* Timing Control Register 2 (TCON2) */

#define RA8P_GLCDC_TCON2_OFFSET                (0x02C)
#define RA8P_GLCDC_TCON2_VACT_MASK             (0x7FF)     /* Bits 0-10: Vertical Active */
#define RA8P_GLCDC_TCON2_VACT_SHIFT            (0)
#define RA8P_GLCDC_TCON2_VSYNC_MASK            (0x7F0000)  /* Bits 16-22: Vertical Sync Width */
#define RA8P_GLCDC_TCON2_VSYNC_SHIFT           (16)
#define RA8P_GLCDC_TCON2_VPOL                  (1 << 23)   /* Bit 23: Vertical Sync Polarity */

/* Timing Control Register 3 (TCON3) */

#define RA8P_GLCDC_TCON3_OFFSET                (0x030)
#define RA8P_GLCDC_TCON3_HBP_MASK              (0x3FF)     /* Bits 0-9: Horizontal Back Porch */
#define RA8P_GLCDC_TCON3_HBP_SHIFT             (0)
#define RA8P_GLCDC_TCON3_HFP_MASK              (0x3FF0000) /* Bits 16-25: Horizontal Front Porch */
#define RA8P_GLCDC_TCON3_HFP_SHIFT             (16)

/* Timing Control Register 4 (TCON4) */

#define RA8P_GLCDC_TCON4_OFFSET                (0x034)
#define RA8P_GLCDC_TCON4_VBP_MASK              (0x3FF)     /* Bits 0-9: Vertical Back Porch */
#define RA8P_GLCDC_TCON4_VBP_SHIFT             (0)
#define RA8P_GLCDC_TCON4_VFP_MASK              (0x3FF0000) /* Bits 16-25: Vertical Front Porch */
#define RA8P_GLCDC_TCON4_VFP_SHIFT             (16)

/* Interrupt Enable Register (INTEN) */

#define RA8P_GLCDC_INTEN_OFFSET                (0x038)
#define RA8P_GLCDC_INTEN_VINTEN                (1 << 0)    /* Bit 0: Vertical Interrupt Enable */
#define RA8P_GLCDC_INTEN_LNDETEN               (1 << 1)    /* Bit 1: Line Detection Interrupt Enable */
#define RA8P_GLCDC_INTEN_UNDREN                (1 << 2)    /* Bit 2: Underflow Interrupt Enable */

/* Interrupt Status Register (INTST) */

#define RA8P_GLCDC_INTST_OFFSET                (0x03C)
#define RA8P_GLCDC_INTST_VINT                  (1 << 0)    /* Bit 0: Vertical Interrupt */
#define RA8P_GLCDC_INTST_LNDET                 (1 << 1)    /* Bit 1: Line Detection Interrupt */
#define RA8P_GLCDC_INTST_UNDR                  (1 << 2)    /* Bit 2: Underflow Interrupt */

/* Brightness Control Register (BRIGHT) */

#define RA8P_GLCDC_BRIGHT_OFFSET               (0x040)
#define RA8P_GLCDC_BRIGHT_BRTH_MASK            (0xFF)      /* Bits 0-7: Brightness Value */
#define RA8P_GLCDC_BRIGHT_BRTH_SHIFT           (0)

/* Contrast Control Register (CONTRAST) */

#define RA8P_GLCDC_CONTRAST_OFFSET             (0x044)
#define RA8P_GLCDC_CONTRAST_CONT_MASK          (0xFF)      /* Bits 0-7: Contrast Value */
#define RA8P_GLCDC_CONTRAST_CONT_SHIFT         (0)

/* Gamma Correction Register (GAMMA) */

#define RA8P_GLCDC_GAMMA_OFFSET                (0x048)
#define RA8P_GLCDC_GAMMA_GAMEN                 (1 << 0)    /* Bit 0: Gamma Enable */

/* CLUT (Color Lookup Table) Registers */

#define RA8P_GLCDC_CLUT_OFFSET                 (0x100)
#define RA8P_GLCDC_CLUT_ENTRY(n)               (RA8P_GLCDC_CLUT_OFFSET + ((n) * 4))

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* GLCDC layer configuration */

struct ra8p_glcdc_layer_s
{
  uint32_t fb_addr;                 /* Frame buffer address */
  uint16_t width;                   /* Layer width */
  uint16_t height;                  /* Layer height */
  uint8_t format;                   /* Color format */
  uint8_t bpp;                      /* Bytes per pixel */
  bool enabled;                     /* Layer enabled flag */
};

/* GLCDC timing configuration */

struct ra8p_glcdc_timing_s
{
  uint16_t hactive;                 /* Horizontal active pixels */
  uint16_t vactive;                 /* Vertical active lines */
  uint16_t hsync;                   /* Horizontal sync width */
  uint16_t vsync;                   /* Vertical sync width */
  uint16_t hbp;                     /* Horizontal back porch */
  uint16_t hfp;                     /* Horizontal front porch */
  uint16_t vbp;                     /* Vertical back porch */
  uint16_t vfp;                     /* Vertical front porch */
  bool hpol;                        /* Horizontal sync polarity (true = active high) */
  bool vpol;                        /* Vertical sync polarity (true = active high) */
};

/* GLCDC configuration */

struct ra8p_glcdc_config_s
{
  uint32_t base;                    /* GLCDC base address */
  int irq;                          /* GLCDC interrupt number */
  struct ra8p_glcdc_timing_s timing; /* Display timing */
  struct ra8p_glcdc_layer_s layer1;  /* Layer 1 configuration */
  struct ra8p_glcdc_layer_s layer2;  /* Layer 2 configuration */
  uint8_t brightness;               /* Brightness value (0-255) */
  uint8_t contrast;                 /* Contrast value (0-255) */
  bool output_to_dsi;               /* True to output to DSI, false for LCD */
};

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_GLCDC_H */