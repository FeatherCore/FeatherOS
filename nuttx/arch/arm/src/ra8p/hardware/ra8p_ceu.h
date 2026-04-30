/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_ceu.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_CEU_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_CEU_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* CEU Base Address */

#define RA8P_CEU_BASE                           (0x40330000)

/* CEU Register Offsets */

/* Capture Enable Register (CAPCR) */

#define RA8P_CEU_CAPCR_OFFSET                   (0x000)
#define RA8P_CEU_CAPCR_CE                       (1 << 0)    /* Bit 0: Capture Enable */
#define RA8P_CEU_CAPCR_CTN                      (1 << 1)    /* Bit 1: Continuous Capture Mode */
#define RA8P_CEU_CAPCR_VPOL                     (1 << 2)    /* Bit 2: VSYNC Polarity */
#define RA8P_CEU_CAPCR_HPOL                     (1 << 3)    /* Bit 3: HSYNC Polarity */
#define RA8P_CEU_CAPCR_DPOL                     (1 << 4)    /* Bit 4: Data Polarity */
#define RA8P_CEU_CAPCR_FWR                      (1 << 5)    /* Bit 5: Field Write */
#define RA8P_CEU_CAPCR_JDT                      (1 << 6)    /* Bit 6: JPEG Data Transfer */

/* Capture Control Register (CSTCR) */

#define RA8P_CEU_CSTCR_OFFSET                   (0x004)
#define RA8P_CEU_CSTCR_CST                      (1 << 0)    /* Bit 0: Capture Start */
#define RA8P_CEU_CSTCR_VCT                      (1 << 1)    /* Bit 1: VSYNC Count */
#define RA8P_CEU_CSTCR_ICPF                     (1 << 2)    /* Bit 2: Image Capture Period Flag */

/* Interrupt Enable Register (CEIER) */

#define RA8P_CEU_CEIER_OFFSET                   (0x010)
#define RA8P_CEU_CEIER_CEIE                     (1 << 0)    /* Bit 0: Capture End Interrupt Enable */
#define RA8P_CEU_CEIER_CEFEIE                   (1 << 1)    /* Bit 1: Capture End Field Interrupt Enable */
#define RA8P_CEU_CEIER_VBPRIE                   (1 << 2)    /* Bit 2: VBP Reception Interrupt Enable */
#define RA8P_CEU_CEIER_VWBFIE                   (1 << 3)    /* Bit 3: VBW Full Interrupt Enable */
#define RA8P_CEU_CEIER_OVRFIE                   (1 << 4)    /* Bit 4: Overflow Interrupt Enable */
#define RA8P_CEU_CEIER_DMAEIE                   (1 << 5)    /* Bit 5: DMA Error Interrupt Enable */

/* Interrupt Status Register (CEISR) */

#define RA8P_CEU_CEISR_OFFSET                   (0x014)
#define RA8P_CEU_CEISR_CEND                     (1 << 0)    /* Bit 0: Capture End */
#define RA8P_CEU_CEISR_CEFE                     (1 << 1)    /* Bit 1: Capture End Field */
#define RA8P_CEU_CEISR_VBPRI                    (1 << 2)    /* Bit 2: VBP Reception */
#define RA8P_CEU_CEISR_VWBF                     (1 << 3)    /* Bit 3: VBW Full */
#define RA8P_CEU_CEISR_OVRF                     (1 << 4)    /* Bit 4: Overflow */
#define RA8P_CEU_CEISR_DMAE                     (1 << 5)    /* Bit 5: DMA Error */

/* Data Sync Control Register (DSYCR) */

#define RA8P_CEU_DSYCR_OFFSET                   (0x018)
#define RA8P_CEU_DSYCR_DSY                      (1 << 0)    /* Bit 0: Data Sync */

/* Clock Control Register (CKCR) */

#define RA8P_CEU_CKCR_OFFSET                    (0x01C)
#define RA8P_CEU_CKCR_CLKEN                     (1 << 0)    /* Bit 0: Clock Enable */

/* Image Capture Register 1 (ICR1) */

#define RA8P_CEU_ICR1_OFFSET                    (0x020)
#define RA8P_CEU_ICR1_CAPW_MASK                 (0xFFFF)    /* Bits 0-15: Capture Width */
#define RA8P_CEU_ICR1_CAPW_SHIFT                (0)
#define RA8P_CEU_ICR1_CAPH_MASK                 (0xFFFF0000) /* Bits 16-31: Capture Height */
#define RA8P_CEU_ICR1_CAPH_SHIFT                (16)

/* Image Capture Register 2 (ICR2) */

#define RA8P_CEU_ICR2_OFFSET                    (0x024)
#define RA8P_CEU_ICR2_BSWP                      (1 << 0)    /* Bit 0: Byte Swap */
#define RA8P_CEU_ICR2_YCBCR                     (1 << 1)    /* Bit 1: YCbCr Format */
#define RA8P_CEU_ICR2_DT_FMT_MASK               (0x70)      /* Bits 4-6: Data Format */
#define RA8P_CEU_ICR2_DT_FMT_SHIFT              (4)
#define RA8P_CEU_ICR2_DT_FMT_YUV422            (0x0 << 4)  /* YUV422 */
#define RA8P_CEU_ICR2_DT_FMT_RGB565            (0x1 << 4)  /* RGB565 */
#define RA8P_CEU_ICR2_DT_FMT_RGB888            (0x2 << 4)  /* RGB888 */
#define RA8P_CEU_ICR2_DT_FMT_JPEG              (0x3 << 4)  /* JPEG */

/* Frame Start Address Register A (FSAA) */

#define RA8P_CEU_FSAA_OFFSET                    (0x028)
#define RA8P_CEU_FSAA_FSA_MASK                  (0xFFFFFFFF) /* Bits 0-31: Frame Start Address A */

/* Frame Start Address Register B (FSAB) */

#define RA8P_CEU_FSAB_OFFSET                    (0x02C)
#define RA8P_CEU_FSAB_FSA_MASK                  (0xFFFFFFFF) /* Bits 0-31: Frame Start Address B */

/* Frame End Address Register A (FEAA) */

#define RA8P_CEU_FEAA_OFFSET                    (0x030)
#define RA8P_CEU_FEAA_FEA_MASK                  (0xFFFFFFFF) /* Bits 0-31: Frame End Address A */

/* Frame End Address Register B (FEAB) */

#define RA8P_CEU_FEAB_OFFSET                    (0x034)
#define RA8P_CEU_FEAB_FEA_MASK                  (0xFFFFFFFF) /* Bits 0-31: Frame End Address B */

/* DMA Control Register (DMAOR) */

#define RA8P_CEU_DMAOR_OFFSET                   (0x038)
#define RA8P_CEU_DMAOR_DAE                      (1 << 0)    /* Bit 0: DMA Enable */
#define RA8P_CEU_DMAOR_DDTGA                    (1 << 1)    /* Bit 1: DMA Transfer Group A */
#define RA8P_CEU_DMAOR_DDTGB                    (1 << 2)    /* Bit 2: DMA Transfer Group B */

/* Capture Line Count Register (CLCR) */

#define RA8P_CEU_CLCR_OFFSET                    (0x03C)
#define RA8P_CEU_CLCR_CLC_MASK                  (0xFFFF)    /* Bits 0-15: Capture Line Count */
#define RA8P_CEU_CLCR_CLC_SHIFT                 (0)

/* FIFO Status Register (FIFSR) */

#define RA8P_CEU_FIFSR_OFFSET                   (0x040)
#define RA8P_CEU_FIFSR_FIF_MASK                 (0x7)       /* Bits 0-2: FIFO Status */
#define RA8P_CEU_FIFSR_FIF_SHIFT                (0)

/* Frame Count Register (FCR) */

#define RA8P_CEU_FCR_OFFSET                     (0x044)
#define RA8P_CEU_FCR_FC_MASK                    (0xFFFF)    /* Bits 0-15: Frame Count */
#define RA8P_CEU_FCR_FC_SHIFT                   (0)

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* CEU data format */

enum ra8p_ceu_format_e
{
  RA8P_CEU_FORMAT_YUV422 = 0,       /* YUV422 format */
  RA8P_CEU_FORMAT_RGB565,           /* RGB565 format */
  RA8P_CEU_FORMAT_RGB888,           /* RGB888 format */
  RA8P_CEU_FORMAT_JPEG,             /* JPEG format */
};

/* CEU capture mode */

enum ra8p_ceu_capture_mode_e
{
  RA8P_CEU_CAPTURE_SINGLE = 0,      /* Single capture */
  RA8P_CEU_CAPTURE_CONTINUOUS,      /* Continuous capture */
};

/* CEU configuration */

struct ra8p_ceu_config_s
{
  uint32_t base;                    /* CEU base address */
  int irq;                          /* CEU interrupt number */
  uint16_t width;                   /* Capture width */
  uint16_t height;                  /* Capture height */
  enum ra8p_ceu_format_e format;   /* Data format */
  enum ra8p_ceu_capture_mode_e mode; /* Capture mode */
  bool vsync_pol;                  /* VSYNC polarity (true = active high) */
  bool hsync_pol;                  /* HSYNC polarity (true = active high) */
  bool data_pol;                   /* Data polarity */
};

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_CEU_H */