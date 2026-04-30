/****************************************************************************
 * arch/arm/src/stm32u5/hardware/stm32u5_lpdma.h
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

#ifndef __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_LPDMA_H
#define __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_LPDMA_H

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Register Offsets *********************************************************/

/* Each channel has the following registers */

#define STM32U5_LPDMA_CH0_CCR_OFFSET         0x0000  /* LPDMA channel x configuration register */
#define STM32U5_LPDMA_CH0_CNDTR_OFFSET       0x0004  /* LPDMA channel x number of data register */
#define STM32U5_LPDMA_CH0_CSAR_OFFSET        0x0008  /* LPDMA channel x source address register */
#define STM32U5_LPDMA_CH0_CDAR_OFFSET        0x000c  /* LPDMA channel x destination address register */
#define STM32U5_LPDMA_CH0_CTR1_OFFSET        0x0010  /* LPDMA channel x transfer register 1 */
#define STM32U5_LPDMA_CH0_CTR2_OFFSET        0x0014  /* LPDMA channel x transfer register 2 */

/* Register addresses for channel 0 */
#define STM32U5_LPDMA_CH0_CCR                (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CCR_OFFSET)
#define STM32U5_LPDMA_CH0_CNDTR              (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CNDTR_OFFSET)
#define STM32U5_LPDMA_CH0_CSAR               (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CSAR_OFFSET)
#define STM32U5_LPDMA_CH0_CDAR               (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CDAR_OFFSET)
#define STM32U5_LPDMA_CH0_CTR1               (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CTR1_OFFSET)
#define STM32U5_LPDMA_CH0_CTR2               (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CTR2_OFFSET)

/* Register offsets for other channels */
#define STM32U5_LPDMA_CH_OFFSET              0x0018  /* Offset between channels */

/* Register addresses for channels 1-3 */
#define STM32U5_LPDMA_CH1_CCR                (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CCR_OFFSET + (1 * STM32U5_LPDMA_CH_OFFSET))
#define STM32U5_LPDMA_CH1_CNDTR              (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CNDTR_OFFSET + (1 * STM32U5_LPDMA_CH_OFFSET))
#define STM32U5_LPDMA_CH1_CSAR               (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CSAR_OFFSET + (1 * STM32U5_LPDMA_CH_OFFSET))
#define STM32U5_LPDMA_CH1_CDAR               (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CDAR_OFFSET + (1 * STM32U5_LPDMA_CH_OFFSET))
#define STM32U5_LPDMA_CH1_CTR1               (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CTR1_OFFSET + (1 * STM32U5_LPDMA_CH_OFFSET))
#define STM32U5_LPDMA_CH1_CTR2               (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CTR2_OFFSET + (1 * STM32U5_LPDMA_CH_OFFSET))

#define STM32U5_LPDMA_CH2_CCR                (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CCR_OFFSET + (2 * STM32U5_LPDMA_CH_OFFSET))
#define STM32U5_LPDMA_CH2_CNDTR              (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CNDTR_OFFSET + (2 * STM32U5_LPDMA_CH_OFFSET))
#define STM32U5_LPDMA_CH2_CSAR               (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CSAR_OFFSET + (2 * STM32U5_LPDMA_CH_OFFSET))
#define STM32U5_LPDMA_CH2_CDAR               (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CDAR_OFFSET + (2 * STM32U5_LPDMA_CH_OFFSET))
#define STM32U5_LPDMA_CH2_CTR1               (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CTR1_OFFSET + (2 * STM32U5_LPDMA_CH_OFFSET))
#define STM32U5_LPDMA_CH2_CTR2               (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CTR2_OFFSET + (2 * STM32U5_LPDMA_CH_OFFSET))

#define STM32U5_LPDMA_CH3_CCR                (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CCR_OFFSET + (3 * STM32U5_LPDMA_CH_OFFSET))
#define STM32U5_LPDMA_CH3_CNDTR              (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CNDTR_OFFSET + (3 * STM32U5_LPDMA_CH_OFFSET))
#define STM32U5_LPDMA_CH3_CSAR               (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CSAR_OFFSET + (3 * STM32U5_LPDMA_CH_OFFSET))
#define STM32U5_LPDMA_CH3_CDAR               (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CDAR_OFFSET + (3 * STM32U5_LPDMA_CH_OFFSET))
#define STM32U5_LPDMA_CH3_CTR1               (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CTR1_OFFSET + (3 * STM32U5_LPDMA_CH_OFFSET))
#define STM32U5_LPDMA_CH3_CTR2               (STM32_LPDMA1_BASE + STM32U5_LPDMA_CH0_CTR2_OFFSET + (3 * STM32U5_LPDMA_CH_OFFSET))

/* General LPDMA registers */
#define STM32U5_LPDMA_CGFR_OFFSET            0x0070  /* LPDMA channel group flag register */
#define STM32U5_LPDMA_CGSFR_OFFSET          0x0074  /* LPDMA channel group status flag register */

#define STM32U5_LPDMA_CGFR                   (STM32_LPDMA1_BASE + STM32U5_LPDMA_CGFR_OFFSET)
#define STM32U5_LPDMA_CGSFR                  (STM32_LPDMA1_BASE + STM32U5_LPDMA_CGSFR_OFFSET)

/* Register Bitfield Definitions *********************************************/

/* Channel Configuration Register (CCR) */

#define LPDMA_CCR_EN                         (1 << 0)   /* Bit 0: Channel enable */
#define LPDMA_CCR_SUSP                       (1 << 1)   /* Bit 1: Channel suspension */
#define LPDMA_CCR_BLKHW                      (1 << 2)   /* Bit 2: Hardware request */
#define LPDMA_CCR_TRIGPOL_SHIFT              (3)        /* Bits 3: Trigger polarity selection */
#define LPDMA_CCR_TRIGPOL_MASK               (1 << LPDMA_CCR_TRIGPOL_SHIFT)
#  define LPDMA_CCR_TRIGPOL_LOW             (0 << LPDMA_CCR_TRIGPOL_SHIFT)  /* Active low or falling edge */
#  define LPDMA_CCR_TRIGPOL_HIGH            (1 << LPDMA_CCR_TRIGPOL_SHIFT)  /* Active high or rising edge */
#define LPDMA_CCR_TRIGSEL_SHIFT             (4)        /* Bits 4-6: Hardware trigger selection */
#define LPDMA_CCR_TRIGSEL_MASK              (7 << LPDMA_CCR_TRIGSEL_SHIFT)
#define LPDMA_CCR_SWREQ                      (1 << 8)   /* Bit 8: Software trigger */
#define LPDMA_CCR_REQSEL_SHIFT              (10)       /* Bits 10-12: Hardware request selection */
#define LPDMA_CCR_REQSEL_MASK               (7 << LPDMA_CCR_REQSEL_SHIFT)
#define LPDMA_CCR_DHU                        (1 << 13)  /* Bit 13: Destination hardware handshaking */
#define LPDMA_CCR_SHU                        (1 << 14)  /* Bit 14: Source hardware handshaking */
#define LPDMA_CCR_GREQ                       (1 << 15)  /* Bit 15: Global hardware request */
#define LPDMA_CCR_SINC                       (1 << 16)  /* Bit 16: Source address incrementation */
#define LPDMA_CCR_DINC                       (1 << 17)  /* Bit 17: Destination address incrementation */
#define LPDMA_CCR_SINCOS_SHIFT              (18)       /* Bits 18-19: Source increment offset size */
#define LPDMA_CCR_SINCOS_MASK               (3 << LPDMA_CCR_SINCOS_SHIFT)
#define LPDMA_CCR_DINCOS_SHIFT              (20)       /* Bits 20-21: Destination increment offset size */
#define LPDMA_CCR_DINCOS_MASK               (3 << LPDMA_CCR_DINCOS_SHIFT)
#define LPDMA_CCR_BRDUM                     (1 << 22)  /* Bit 22: Destination burst */
#define LPDMA_CCR_SRCBRUM                   (1 << 23)  /* Bit 23: Source burst */
#define LPDMA_CCR_DSTBURST_SHIFT            (24)       /* Bits 24-25: Destination burst length */
#define LPDMA_CCR_DSTBURST_MASK            (3 << LPDMA_CCR_DSTBURST_SHIFT)
#define LPDMA_CCR_SRCBURST_SHIFT            (26)       /* Bits 26-27: Source burst length */
#define LPDMA_CCR_SRCBURST_MASK            (3 << LPDMA_CCR_SRCBURST_SHIFT)
#define LPDMA_CCR_DWSEL_SHIFT              (28)       /* Bits 28-29: Destination data width selection */
#define LPDMA_CCR_DWSEL_MASK               (3 << LPDMA_CCR_DWSEL_SHIFT)
#  define LPDMA_CCR_DWSEL_BYTE              (0 << LPDMA_CCR_DWSEL_SHIFT)  /* 8-bit */
#  define LPDMA_CCR_DWSEL_HALFWORD        (1 << LPDMA_CCR_DWSEL_SHIFT)  /* 16-bit */
#  define LPDMA_CCR_DWSEL_WORD             (2 << LPDMA_CCR_DWSEL_SHIFT)  /* 32-bit */
#define LPDMA_CCR_SWSEL_SHIFT              (30)       /* Bits 30-31: Source data width selection */
#define LPDMA_CCR_SWSEL_MASK               (3 << LPDMA_CCR_SWSEL_SHIFT)
#  define LPDMA_CCR_SWSEL_BYTE              (0 << LPDMA_CCR_SWSEL_SHIFT)  /* 8-bit */
#  define LPDMA_CCR_SWSEL_HALFWORD        (1 << LPDMA_CCR_SWSEL_SHIFT)  /* 16-bit */
#  define LPDMA_CCR_SWSEL_WORD             (2 << LPDMA_CCR_SWSEL_SHIFT)  /* 32-bit */

/* Channel Number of Data Register (CNDTR) */

#define LPDMA_CNDTR_NDT_SHIFT               (0)        /* Bits 0-15: Number of data to transfer */
#define LPDMA_CNDTR_NDT_MASK               (0xFFFF << LPDMA_CNDTR_NDT_SHIFT)

/* Channel Transfer Register 1 (CTR1) */

#define LPDMA_CTR1_PFREQ_SHIFT              (0)        /* Bits 0-5: Request generator frequency */
#define LPDMA_CTR1_PFREQ_MASK              (0x3F << LPDMA_CTR1_PFREQ_SHIFT)
#define LPDMA_CTR1_PBURST_SHIFT            (6)        /* Bits 6-8: Peripheral burst length */
#define LPDMA_CTR1_PBURST_MASK             (7 << LPDMA_CTR1_PBURST_SHIFT)
#define LPDMA_CTR1_RDUM                     (1 << 9)   /* Bit 9: Increment mode */
#define LPDMA_CTR1_SREQ                     (1 << 10)  /* Bit 10: Source request */
#define LPDMA_CTR1_DREQ                     (1 << 11)  /* Bit 11: Destination request */
#define LPDMA_CTR1_LAE                      (1 << 12)  /* Bit 12: Link address update */
#define LPDMA_CTR1_PAM                      (1 << 13)  /* Bit 13: Peripheral access mode */
#define LPDMA_CTR1_TRIGSEL_SHIFT            (16)       /* Bits 16-18: Trigger selection */
#define LPDMA_CTR1_TRIGSEL_MASK            (7 << LPDMA_CTR1_TRIGSEL_SHIFT)
#define LPDMA_CTR1_TRIGPOL                 (1 << 19)  /* Bit 19: Trigger polarity */
#define LPDMA_CTR1_REQSEL_SHIFT            (20)       /* Bits 20-22: Request selection */
#define LPDMA_CTR1_REQSEL_MASK             (7 << LPDMA_CTR1_REQSEL_SHIFT)

/* Channel Transfer Register 2 (CTR2) */

#define LPDMA_CTR2_BRCNT_SHIFT              (0)        /* Bits 0-15: Block repeat count */
#define LPDMA_CTR2_BRCNT_MASK              (0xFFFF << LPDMA_CTR2_BRCNT_SHIFT)
#define LPDMA_CTR2_DRL                      (1 << 16)  /* Bit 16: Destination reload */
#define LPDMA_CTR2_SRL                      (1 << 17)  /* Bit 17: Source reload */
#define LPDMA_CTR2_SBUS                     (1 << 18)  /* Bit 18: Source burst */
#define LPDMA_CTR2_DBUS                     (1 << 19)  /* Bit 19: Destination burst */

/* Channel Group Flag Register (CGFR) */

#define LPDMA_CGFR_IDMACH0F                (1 << 0)   /* Bit 0: GPDMA channel 0 interrupt done mask */
#define LPDMA_CGFR_IDMACH1F                (1 << 1)   /* Bit 1: GPDMA channel 1 interrupt done mask */
#define LPDMA_CGFR_IDMACH2F                (1 << 2)   /* Bit 2: GPDMA channel 2 interrupt done mask */
#define LPDMA_CGFR_IDMACH3F                (1 << 3)   /* Bit 3: GPDMA channel 3 interrupt done mask */

#define LPDMA_CGFR_IDMACH0E                (1 << 16)  /* Bit 16: GPDMA channel 0 error mask */
#define LPDMA_CGFR_IDMACH1E                (1 << 17)  /* Bit 17: GPDMA channel 1 error mask */
#define LPDMA_CGFR_IDMACH2E                (1 << 18)  /* Bit 18: GPDMA channel 2 error mask */
#define LPDMA_CGFR_IDMACH3E                (1 << 19)  /* Bit 19: GPDMA channel 3 error mask */

#endif /* __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_LPDMA_H */