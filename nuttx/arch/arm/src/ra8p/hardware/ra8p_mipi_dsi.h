/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_mipi_dsi.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_MIPI_DSI_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_MIPI_DSI_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* MIPI DSI Base Address */

#define RA8P_MIPI_DSI_BASE                     (0x40346000)

/* MIPI DSI Register Offsets */

/* DSI Control Register (DSICR) */

#define RA8P_DSI_DSICR_OFFSET                  (0x000)
#define RA8P_DSI_DSICR_DSIEN                   (1 << 0)    /* Bit 0: DSI Enable */
#define RA8P_DSI_DSICR_DSIRES                  (1 << 1)    /* Bit 1: DSI Reset */
#define RA8P_DSI_DSICR_DSIRESST                (1 << 2)    /* Bit 2: DSI Reset Status */

/* DSI Mode Register (DSIMR) */

#define RA8P_DSI_DSIMR_OFFSET                  (0x004)
#define RA8P_DSI_DSIMR_VSMODE                  (1 << 0)    /* Bit 0: Video/Command Mode */
#define RA8P_DSI_DSIMR_VSMODE_CMD              (0 << 0)    /* Command mode */
#define RA8P_DSI_DSI_DSIMR_VSMODE_VIDEO        (1 << 0)    /* Video mode */
#define RA8P_DSI_DSIMR_NL_MASK                 (0x3E)      /* Bits 1-5: Number of Lanes */
#define RA8P_DSI_DSIMR_NL_SHIFT                (1)
#define RA8P_DSI_DSIMR_NL_1LANE                (0x0 << 1)  /* 1 lane */
#define RA8P_DSI_DSIMR_NL_2LANES               (0x1 << 1)  /* 2 lanes */
#define RA8P_DSI_DSIMR_NL_3LANES               (0x2 << 1)  /* 3 lanes */
#define RA8P_DSI_DSIMR_NL_4LANES               (0x3 << 1)  /* 4 lanes */
#define RA8P_DSI_DSIMR_EOTDIS                  (1 << 6)    /* Bit 6: EOT Disable */
#define RA8P_DSI_DSIMR_CMDDIS                  (1 << 7)    /* Bit 7: Command Disable */

/* DSI Video Mode Register (DSIVMR) */

#define RA8P_DSI_DSIVMR_OFFSET                 (0x008)
#define RA8P_DSI_DSIVMR_VSEN                   (1 << 0)    /* Bit 0: Video Mode Enable */
#define RA8P_DSI_DSIVMR_LPEN                   (1 << 1)    /* Bit 1: Low Power Enable */
#define RA8P_DSI_DSIVMR_FRM_MASK               (0xC)       /* Bits 2-3: Frame Format */
#define RA8P_DSI_DSIVMR_FRM_SHIFT              (2)
#define RA8P_DSI_DSIVMR_FRM_NON_BURST_SYNC     (0x0 << 2)  /* Non-burst with sync pulses */
#define RA8P_DSI_DSIVMR_FRM_NON_BURST_EVENT    (0x1 << 2)  /* Non-burst with sync events */
#define RA8P_DSI_DSIVMR_FRM_BURST              (0x2 << 2)  /* Burst mode */

/* DSI Command Mode Register (DSICMR) */

#define RA8P_DSI_DSICMR_OFFSET                 (0x00C)
#define RA8P_DSI_DSICMR_TESSEL                 (1 << 0)    /* Bit 0: TE Source Select */
#define RA8P_DSI_DSICMR_TEPOL                  (1 << 1)    /* Bit 1: TE Polarity */
#define RA8P_DSI_DSICMR_LPEN                   (1 << 2)    /* Bit 2: Low Power Enable */

/* DSI Configuration Register (DSICFGR) */

#define RA8P_DSI_DSICFGR_OFFSET                (0x010)
#define RA8P_DSI_DSICFGR_HACT_MASK             (0xFFFF)    /* Bits 0-15: Horizontal Active */
#define RA8P_DSI_DSICFGR_HACT_SHIFT            (0)
#define RA8P_DSI_DSICFGR_VACT_MASK             (0xFFFF0000) /* Bits 16-31: Vertical Active */
#define RA8P_DSI_DSICFGR_VACT_SHIFT            (16)

/* DSI Timing Register 0 (DSITR0) */

#define RA8P_DSI_DSITR0_OFFSET                 (0x014)
#define RA8P_DSI_DSITR0_HSA_MASK               (0xFFF)     /* Bits 0-11: Horizontal Sync Active */
#define RA8P_DSI_DSITR0_HSA_SHIFT              (0)
#define RA8P_DSI_DSITR0_HBP_MASK               (0xFFF000)  /* Bits 12-23: Horizontal Back Porch */
#define RA8P_DSI_DSITR0_HBP_SHIFT              (12)
#define RA8P_DSI_DSITR0_HFP_MASK               (0xFFF000000) /* Bits 24-35: Horizontal Front Porch */
#define RA8P_DSI_DSITR0_HFP_SHIFT              (24)

/* DSI Timing Register 1 (DSITR1) */

#define RA8P_DSI_DSITR1_OFFSET                 (0x018)
#define RA8P_DSI_DSITR1_VSA_MASK               (0x3FF)     /* Bits 0-9: Vertical Sync Active */
#define RA8P_DSI_DSITR1_VSA_SHIFT              (0)
#define RA8P_DSI_DSITR1_VBP_MASK               (0x3FF000)  /* Bits 12-21: Vertical Back Porch */
#define RA8P_DSI_DSITR1_VBP_SHIFT              (12)
#define RA8P_DSI_DSITR1_VFP_MASK               (0x3FF000000) /* Bits 24-33: Vertical Front Porch */
#define RA8P_DSI_DSITR1_VFP_SHIFT              (24)

/* DSI Packet Control Register (DSIPCR) */

#define RA8P_DSI_DSIPCR_OFFSET                 (0x01C)
#define RA8P_DSI_DSIPCR_PKTCMD_MASK            (0xFF)      /* Bits 0-7: Packet Command */
#define RA8P_DSI_DSIPCR_PKTCMD_SHIFT           (0)
#define RA8P_DSI_DSIPCR_PKTSIZE_MASK           (0xFFFF0000) /* Bits 16-31: Packet Size */
#define RA8P_DSI_DSIPCR_PKTSIZE_SHIFT          (16)

/* DSI Packet Data Register (DSIPDR) */

#define RA8P_DSI_DSIPDR_OFFSET                 (0x020)
#define RA8P_DSI_DSIPDR_DATA_MASK              (0xFFFFFFFF) /* Bits 0-31: Packet Data */

/* DSI Status Register (DSISR) */

#define RA8P_DSI_DSISR_OFFSET                  (0x024)
#define RA8P_DSI_DSISR_BUSY                    (1 << 0)    /* Bit 0: Busy */
#define RA8P_DSI_DSISR_CMDDONE                 (1 << 1)    /* Bit 1: Command Done */
#define RA8P_DSI_DSISR_TE                      (1 << 2)    /* Bit 2: TE Event */
#define RA8P_DSI_DSISR_ERR                     (1 << 3)    /* Bit 3: Error */
#define RA8P_DSI_DSISR_PLLST                   (1 << 4)    /* Bit 4: PLL Status */

/* DSI Interrupt Enable Register (DSIIER) */

#define RA8P_DSI_DSIIER_OFFSET                 (0x028)
#define RA8P_DSI_DSIIER_CMDDONEIE              (1 << 0)    /* Bit 0: Command Done Interrupt Enable */
#define RA8P_DSI_DSIIER_TEIE                   (1 << 1)    /* Bit 1: TE Interrupt Enable */
#define RA8P_DSI_DSIIER_ERRIE                  (1 << 2)    /* Bit 2: Error Interrupt Enable */

/* DSI PLL Control Register (DSIPLLCR) */

#define RA8P_DSI_DSIPLLCR_OFFSET               (0x030)
#define RA8P_DSI_DSIPLLCR_PLLEN                (1 << 0)    /* Bit 0: PLL Enable */
#define RA8P_DSI_DSIPLLCR_PLLST                (1 << 1)    /* Bit 1: PLL Status */
#define RA8P_DSI_DSIPLLCR_NDIV_MASK            (0x7F00)    /* Bits 8-14: N Divider */
#define RA8P_DSI_DSIPLLCR_NDIV_SHIFT           (8)
#define RA8P_DSI_DSIPLLCR_MDIV_MASK            (0xFF0000)  /* Bits 16-23: M Divider */
#define RA8P_DSI_DSIPLLCR_MDIV_SHIFT           (16)

/* DSI PHY Control Register (DSIPHYCR) */

#define RA8P_DSI_DSIPHYCR_OFFSET               (0x034)
#define RA8P_DSI_DSIPHYCR_PHYEN                (1 << 0)    /* Bit 0: PHY Enable */
#define RA8P_DSI_DSIPHYCR_PHYST                (1 << 1)    /* Bit 1: PHY Status */
#define RA8P_DSI_DSIPHYCR_TXCLKEN              (1 << 2)    /* Bit 2: TX Clock Enable */
#define RA8P_DSI_DSIPHYCR_RXCLKEN              (1 << 3)    /* Bit 3: RX Clock Enable */

/* DSI PHY Timing Register (DSIPHYTR) */

#define RA8P_DSI_DSIPHYTR_OFFSET               (0x038)
#define RA8P_DSI_DSIPHYTR_THSPREP_MASK         (0xFF)      /* Bits 0-7: HS Prepare Time */
#define RA8P_DSI_DSIPHYTR_THSPREP_SHIFT        (0)
#define RA8P_DSI_DSIPHYTR_THSZERO_MASK         (0xFF00)    /* Bits 8-15: HS Zero Time */
#define RA8P_DSI_DSIPHYTR_THSZERO_SHIFT        (8)
#define RA8P_DSI_DSIPHYTR_THSTRAIL_MASK        (0xFF0000)  /* Bits 16-23: HS Trail Time */
#define RA8P_DSI_DSIPHYTR_THSTRAIL_SHIFT       (16)

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* MIPI DSI configuration */

struct ra8p_dsi_config_s
{
  uint32_t base;                    /* DSI base address */
  uint8_t lanes;                    /* Number of data lanes (1-4) */
  uint32_t pixel_clock;             /* Pixel clock frequency in Hz */
  uint16_t hactive;                 /* Horizontal active pixels */
  uint16_t vactive;                 /* Vertical active lines */
  uint16_t hsync;                   /* Horizontal sync width */
  uint16_t hbp;                     /* Horizontal back porch */
  uint16_t hfp;                     /* Horizontal front porch */
  uint16_t vsync;                   /* Vertical sync width */
  uint16_t vbp;                     /* Vertical back porch */
  uint16_t vfp;                     /* Vertical front porch */
  bool video_mode;                  /* True for video mode, false for command mode */
};

/* MIPI DSI packet header */

struct ra8p_dsi_packet_s
{
  uint8_t data_type;                /* Data type */
  uint16_t word_count;              /* Word count for long packets */
  uint8_t channel;                  /* Virtual channel */
  const uint8_t *payload;           /* Payload data */
  size_t payload_len;               /* Payload length */
};

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_MIPI_DSI_H */