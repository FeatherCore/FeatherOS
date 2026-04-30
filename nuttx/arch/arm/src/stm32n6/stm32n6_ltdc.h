/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_ltdc.h
 *
 * SPDX-License-Identifier: Apache-2.0
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_LTDC_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_LTDC_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <stdint.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* LTDC Register Offsets */

#define STM32_LTDC_SSCR_OFFSET       0x000  /* Synchronization Size Configuration */
#define STM32_LTDC_BPCR_OFFSET       0x004  /* Back Porch Configuration */
#define STM32_LTDC_AWCR_OFFSET       0x008  /* Active Width Configuration */
#define STM32_LTDC_TWCR_OFFSET       0x00C  /* Total Width Configuration */
#define STM32_LTDC_GCR_OFFSET        0x010  /* Global Control */
#define STM32_LTDC_SRCR_OFFSET       0x014  /* Shadow Reload Control */
#define STM32_LTDC_BCCR_OFFSET       0x018  /* Background Color Configuration */
#define STM32_LTDC_IER_OFFSET        0x01C  /* Interrupt Enable */
#define STM32_LTDC_ISR_OFFSET        0x020  /* Interrupt Status */
#define STM32_LTDC_ICR_OFFSET        0x024  /* Interrupt Clear */
#define STM32_LTDC_LIPCR_OFFSET      0x028  /* Line Interrupt Position */
#define STM32_LTDC_CPSR_OFFSET       0x02C  /* Current Position Status */
#define STM32_LTDC_CDSR_OFFSET       0x030  /* Current Display Status */
#define STM32_LTDC_L1CR_OFFSET       0x084  /* Layer 1 Control */
#define STM32_LTDC_L1WHPCR_OFFSET    0x088  /* Layer 1 Window Horizontal Position */
#define STM32_LTDC_L1WVPCR_OFFSET    0x08C  /* Layer 1 Window Vertical Position */
#define STM32_LTDC_L1CKCR_OFFSET     0x090  /* Layer 1 Color Keying */
#define STM32_LTDC_L1PFCR_OFFSET     0x094  /* Layer 1 Pixel Format */
#define STM32_LTDC_L1CACR_OFFSET     0x098  /* Layer 1 Constant Alpha */
#define STM32_LTDC_L1DCCR_OFFSET     0x09C  /* Layer 1 Default Color */
#define STM32_LTDC_L1BFCR_OFFSET     0x0A0  /* Layer 1 Blending Factors */
#define STM32_LTDC_L1CFBAR_OFFSET    0x0AC  /* Layer 1 Color Frame Buffer Address */
#define STM32_LTDC_L1CFBLR_OFFSET    0x0B0  /* Layer 1 Color Frame Buffer Length */
#define STM32_LTDC_L1CFBLNR_OFFSET   0x0B4  /* Layer 1 Color Frame Buffer Line Number */
#define STM32_LTDC_L1CLUTWR_OFFSET   0x0C4  /* Layer 1 CLUT Write */
#define STM32_LTDC_L2CR_OFFSET       0x104  /* Layer 2 Control */
#define STM32_LTDC_L2WHPCR_OFFSET    0x108  /* Layer 2 Window Horizontal Position */
#define STM32_LTDC_L2WVPCR_OFFSET    0x10C  /* Layer 2 Window Vertical Position */
#define STM32_LTDC_L2CKCR_OFFSET     0x110  /* Layer 2 Color Keying */
#define STM32_LTDC_L2PFCR_OFFSET     0x114  /* Layer 2 Pixel Format */
#define STM32_LTDC_L2CACR_OFFSET     0x118  /* Layer 2 Constant Alpha */
#define STM32_LTDC_L2DCCR_OFFSET     0x11C  /* Layer 2 Default Color */
#define STM32_LTDC_L2BFCR_OFFSET     0x120  /* Layer 2 Blending Factors */
#define STM32_LTDC_L2CFBAR_OFFSET    0x12C  /* Layer 2 Color Frame Buffer Address */
#define STM32_LTDC_L2CFBLR_OFFSET    0x130  /* Layer 2 Color Frame Buffer Length */
#define STM32_LTDC_L2CFBLNR_OFFSET   0x134  /* Layer 2 Color Frame Buffer Line Number */
#define STM32_LTDC_L2CLUTWR_OFFSET   0x144  /* Layer 2 CLUT Write */

/* SSCR Register Fields */
#define LTDC_SSCR_VSH_SHIFT          16
#define LTDC_SSCR_VSH_MASK           (0x7FF << LTDC_SSCR_VSH_SHIFT)
#define LTDC_SSCR_HSW_SHIFT          0
#define LTDC_SSCR_HSW_MASK           (0x7FF << LTDC_SSCR_HSW_SHIFT)

/* BPCR Register Fields */
#define LTDC_BPCR_AVBP_SHIFT         16
#define LTDC_BPCR_AVBP_MASK          (0x7FF << LTDC_BPCR_AVBP_SHIFT)
#define LTDC_BPCR_AHBP_SHIFT         0
#define LTDC_BPCR_AHBP_MASK          (0x7FF << LTDC_BPCR_AHBP_SHIFT)

/* AWCR Register Fields */
#define LTDC_AWCR_AAH_SHIFT          16
#define LTDC_AWCR_AAH_MASK           (0x7FF << LTDC_AWCR_AAH_SHIFT)
#define LTDC_AWCR_AAW_SHIFT          0
#define LTDC_AWCR_AAW_MASK           (0x7FF << LTDC_AWCR_AAW_SHIFT)

/* TWCR Register Fields */
#define LTDC_TWCR_TOTALH_SHIFT       16
#define LTDC_TWCR_TOTALH_MASK        (0x7FF << LTDC_TWCR_TOTALH_SHIFT)
#define LTDC_TWCR_TOTALW_SHIFT       0
#define LTDC_TWCR_TOTALW_MASK        (0x7FF << LTDC_TWCR_TOTALW_SHIFT)

/* GCR Register Fields */
#define LTDC_GCR_LTDCEN              (1 << 0)
#define LTDC_GCR_DBW_SHIFT           4
#define LTDC_GCR_DBW_MASK            (7 << LTDC_GCR_DBW_SHIFT)
#define LTDC_GCR_DTEN                (1 << 16)
#define LTDC_GCR_PCPOL               (1 << 28)
#define LTDC_GCR_DEPOL               (1 << 29)
#define LTDC_GCR_VSPOL               (1 << 30)
#define LTDC_GCR_HSPOL               (1 << 31)

/* SRCR Register Fields */
#define LTDC_SRCR_IMR                (1 << 0)
#define LTDC_SRCR_VBR                (1 << 1)

/* L1CR/L2CR Register Fields */
#define LTDC_LxCR_LEN                (1 << 0)
#define LTDC_LxCR_COLKEN             (1 << 4)

/* L1PFCR/L2PFCR Register Fields */
#define LTDC_LxPFCR_PF_SHIFT         0
#define LTDC_LxPFCR_PF_MASK          (7 << LTDC_LxPFCR_PF_SHIFT)
#define LTDC_LxPFCR_PF_ARGB8888      0
#define LTDC_LxPFCR_PF_RGB888        1
#define LTDC_LxPFCR_PF_RGB565        2
#define LTDC_LxPFCR_PF_ARGB1555      3
#define LTDC_LxPFCR_PF_ARGB4444      4
#define LTDC_LxPFCR_PF_L8            5
#define LTDC_LxPFCR_PF_AL44          6
#define LTDC_LxPFCR_PF_AL88          7

/* L1BFCR/L2BFCR Register Fields */
#define LTDC_LxBFCR_BF2_SHIFT        8
#define LTDC_LxBFCR_BF2_MASK         (7 << LTDC_LxBFCR_BF2_SHIFT)
#define LTDC_LxBFCR_BF1_SHIFT        0
#define LTDC_LxBFCR_BF1_MASK         (7 << LTDC_LxBFCR_BF1_SHIFT)

/* Pixel formats */
#define LTDC_PIXELFORMAT_ARGB8888    0
#define LTDC_PIXELFORMAT_RGB888      1
#define LTDC_PIXELFORMAT_RGB565      2
#define LTDC_PIXELFORMAT_ARGB1555    3
#define LTDC_PIXELFORMAT_ARGB4444    4
#define LTDC_PIXELFORMAT_L8          5
#define LTDC_PIXELFORMAT_AL44        6
#define LTDC_PIXELFORMAT_AL88        7

/* Blending factors */
#define LTDC_BLENDING_FACTOR1_CA     0x04
#define LTDC_BLENDING_FACTOR1_PAxCA  0x06
#define LTDC_BLENDING_FACTOR2_CA     0x05
#define LTDC_BLENDING_FACTOR2_PAxCA  0x07

/* Layer constants */
#define LTDC_NLAYERS                 2
#define LTDC_MAX_LAYERS              2

/****************************************************************************
 * Public Types
 ****************************************************************************/

struct stm32n6_ltdc_layer_s
{
  uint32_t pf;              /* Pixel format */
  uint32_t alpha;           /* Alpha value */
  uint32_t alpha0;          /* Second alpha value for plane alpha */
  uint32_t blending1;       /* Blending factor 1 */
  uint32_t blending2;       /* Blending factor 2 */
  uint32_t colorkey;        /* Color key value */
  uint32_t default_color;   /* Default color */
  uint32_t bf1;             /* Blending factor 1 */
  uint32_t bf2;             /* Blending factor 2 */
  uint32_t framebuff;       /* Frame buffer address */
  uint32_t hspan;           /* Horizontal span */
  uint32_t vspan;           /* Vertical span */
  uint32_t hoffset;         /* Horizontal offset */
  uint32_t voffset;         /* Vertical offset */
  uint32_t stride;          /* Line offset */
};

struct stm32n6_ltdc_s
{
  uint32_t base;            /* LTDC base address */
  uint32_t irq;             /* LTDC interrupt */
  uint32_t pixelclock;      /* Pixel clock frequency */
  uint32_t hsync;           /* Horizontal synchronization width */
  uint32_t vsync;           /* Vertical synchronization height */
  uint32_t hbp;             /* Horizontal back porch */
  uint32_t vbp;             /* Vertical back porch */
  uint32_t hfp;             /* Horizontal front porch */
  uint32_t vfp;             /* Vertical front porch */
  uint32_t width;           /* Active width */
  uint32_t height;          /* Active height */
  struct stm32n6_ltdc_layer_s layers[LTDC_MAX_LAYERS]; /* Layer configurations */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

int stm32n6_ltdc_initialize(void);
int stm32n6_ltdc_configure(struct stm32n6_ltdc_s *ltdc);
int stm32n6_ltdc_enable_layer(int layer);
int stm32n6_ltdc_disable_layer(int layer);
int stm32n6_ltdc_set_layer_format(int layer, uint32_t format);
int stm32n6_ltdc_set_layer_alpha(int layer, uint8_t alpha);
int stm32n6_ltdc_set_layer_address(int layer, uint32_t address);
int stm32n6_ltdc_set_layer_window(int layer, uint16_t hstart, uint16_t hstop,
                                  uint16_t vstart, uint16_t vstop);
void stm32n6_ltdc_reload_config(void);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_LTDC_H */
