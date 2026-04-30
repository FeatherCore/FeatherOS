/****************************************************************************
 * arch/arm/src/stm32u5/hardware/stm32u5_gpdma.h
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

#ifndef __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_GPDMA_H
#define __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_GPDMA_H

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Register Offsets *********************************************************/

/* Each channel has the following registers */

#define STM32U5_GPDMA_CH0_CCR_OFFSET         0x0000  /* GPDMA channel x configuration register */
#define STM32U5_GPDMA_CH0_CNDTR_OFFSET       0x0004  /* GPDMA channel x number of data register */
#define STM32U5_GPDMA_CH0_CSAR_OFFSET        0x0008  /* GPDMA channel x source address register */
#define STM32U5_GPDMA_CH0_CDAR_OFFSET        0x000c  /* GPDMA channel x destination address register */
#define STM32U5_GPDMA_CH0_CBR1_OFFSET        0x0010  /* GPDMA channel x block register 1 */
#define STM32U5_GPDMA_CH0_CBR2_OFFSET        0x0014  /* GPDMA channel x block register 2 */
#define STM32U5_GPDMA_CH0_CSRLI_OFFSET       0x0018  /* GPDMA channel x linked-list item (source) address register */
#define STM32U5_GPDMA_CH0_CDRLI_OFFSET       0x001c  /* GPDMA channel x linked-list item (destination) address register */
#define STM32U5_GPDMA_CH0_CTR1_OFFSET        0x0020  /* GPDMA channel x transfer register 1 */
#define STM32U5_GPDMA_CH0_CTR2_OFFSET        0x0024  /* GPDMA channel x transfer register 2 */
#define STM32U5_GPDMA_CH0_CBR3_OFFSET        0x0028  /* GPDMA channel x block register 3 */
#define STM32U5_GPDMA_CH0_CQBAR_OFFSET       0x002c  /* GPDMA channel x queue bus address register */
#define STM32U5_GPDMA_CH0_CQBER_OFFSET       0x0030  /* GPDMA channel x queue bus end register */
#define STM32U5_GPDMA_CH0_CSR_OFFSET         0x0034  /* GPDMA channel x status register */
#define STM32U5_GPDMA_CH0_CFCR_OFFSET        0x0038  /* GPDMA channel x flag clear register */

/* Register addresses for channel 0 */
#define STM32U5_GPDMA_CH0_CCR                (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CCR_OFFSET)
#define STM32U5_GPDMA_CH0_CNDTR              (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CNDTR_OFFSET)
#define STM32U5_GPDMA_CH0_CSAR               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CSAR_OFFSET)
#define STM32U5_GPDMA_CH0_CDAR               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CDAR_OFFSET)
#define STM32U5_GPDMA_CH0_CBR1               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CBR1_OFFSET)
#define STM32U5_GPDMA_CH0_CBR2               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CBR2_OFFSET)
#define STM32U5_GPDMA_CH0_CSRLI              (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CSRLI_OFFSET)
#define STM32U5_GPDMA_CH0_CDRLI              (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CDRLI_OFFSET)
#define STM32U5_GPDMA_CH0_CTR1               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CTR1_OFFSET)
#define STM32U5_GPDMA_CH0_CTR2               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CTR2_OFFSET)
#define STM32U5_GPDMA_CH0_CBR3               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CBR3_OFFSET)
#define STM32U5_GPDMA_CH0_CQBAR              (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CQBAR_OFFSET)
#define STM32U5_GPDMA_CH0_CQBER              (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CQBER_OFFSET)
#define STM32U5_GPDMA_CH0_CSR                (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CSR_OFFSET)
#define STM32U5_GPDMA_CH0_CFCR               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CFCR_OFFSET)

/* Register offsets for other channels */
#define STM32U5_GPDMA_CH_OFFSET              0x0040  /* Offset between channels */

/* Register addresses for channels 1-15 */
#define STM32U5_GPDMA_CH1_CCR                (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CCR_OFFSET + (1 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH1_CNDTR              (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CNDTR_OFFSET + (1 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH1_CSAR               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CSAR_OFFSET + (1 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH1_CDAR               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CDAR_OFFSET + (1 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH1_CBR1               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CBR1_OFFSET + (1 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH1_CBR2               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CBR2_OFFSET + (1 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH1_CSRLI              (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CSRLI_OFFSET + (1 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH1_CDRLI              (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CDRLI_OFFSET + (1 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH1_CTR1               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CTR1_OFFSET + (1 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH1_CTR2               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CTR2_OFFSET + (1 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH1_CBR3               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CBR3_OFFSET + (1 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH1_CQBAR              (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CQBAR_OFFSET + (1 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH1_CQBER              (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CQBER_OFFSET + (1 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH1_CSR                (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CSR_OFFSET + (1 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH1_CFCR               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CFCR_OFFSET + (1 * STM32U5_GPDMA_CH_OFFSET))

/* Similar definitions for channels 2-15... */
#define STM32U5_GPDMA_CH2_CCR                (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CCR_OFFSET + (2 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH2_CNDTR              (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CNDTR_OFFSET + (2 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH2_CSAR               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CSAR_OFFSET + (2 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH2_CDAR               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CDAR_OFFSET + (2 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH2_CBR1               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CBR1_OFFSET + (2 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH2_CBR2               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CBR2_OFFSET + (2 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH2_CSRLI              (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CSRLI_OFFSET + (2 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH2_CDRLI              (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CDRLI_OFFSET + (2 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH2_CTR1               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CTR1_OFFSET + (2 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH2_CTR2               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CTR2_OFFSET + (2 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH2_CBR3               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CBR3_OFFSET + (2 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH2_CQBAR              (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CQBAR_OFFSET + (2 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH2_CQBER              (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CQBER_OFFSET + (2 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH2_CSR                (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CSR_OFFSET + (2 * STM32U5_GPDMA_CH_OFFSET))
#define STM32U5_GPDMA_CH2_CFCR               (STM32_GPDMA1_BASE + STM32U5_GPDMA_CH0_CFCR_OFFSET + (2 * STM32U5_GPDMA_CH_OFFSET))

/* General GPDMA registers */
#define STM32U5_GPDMA_CGFR_OFFSET            0x03f8  /* GPDMA channel group flag register */
#define STM32U5_GPDMA_CGSFR_OFFSET           0x03fc  /* GPDMA channel group status flag register */

#define STM32U5_GPDMA_CGFR                   (STM32_GPDMA1_BASE + STM32U5_GPDMA_CGFR_OFFSET)
#define STM32U5_GPDMA_CGSFR                  (STM32_GPDMA1_BASE + STM32U5_GPDMA_CGSFR_OFFSET)

/* Register Bitfield Definitions *********************************************/

/* Channel Configuration Register (CCR) */

#define GPDMA_CCR_EN                         (1 << 0)   /* Bit 0: Channel enable */
#define GPDMA_CCR_SUSP                       (1 << 1)   /* Bit 1: Channel suspension */
#define GPDMA_CCR_BLKHW                        (1 << 2)   /* Bit 2: Hardware request */
#define GPDMA_CCR_TRIGPOL_SHIFT              (3)        /* Bits 3-4: Trigger polarity */
#define GPDMA_CCR_TRIGPOL_MASK               (3 << GPDMA_CCR_TRIGPOL_SHIFT)
#  define GPDMA_CCR_TRIGPOL_RISING           (0 << GPDMA_CCR_TRIGPOL_SHIFT) /* Rising edge */
#  define GPDMA_CCR_TRIGPOL_FALLING          (1 << GPDMA_CCR_TRIGPOL_SHIFT) /* Falling edge */
#  define GPDMA_CCR_TRIGPOL_BOTH             (2 << GPDMA_CCR_TRIGPOL_SHIFT) /* Both edges */
#define GPDMA_CCR_TRIGSEL_SHIFT              (5)        /* Bits 5-7: Hardware trigger selection */
#define GPDMA_CCR_TRIGSEL_MASK               (7 << GPDMA_CCR_TRIGSEL_SHIFT)
#define GPDMA_CCR_SWREQ                      (1 << 8)   /* Bit 8: Software trigger */
#define GPDMA_CCR_HWREQ                      (1 << 9)   /* Bit 9: Hardware request configuration */
#define GPDMA_CCR_REQSEL_SHIFT               (10)       /* Bits 10-12: Hardware request selection */
#define GPDMA_CCR_REQSEL_MASK                (7 << GPDMA_CCR_REQSEL_SHIFT)
#define GPDMA_CCR_LOCK                       (1 << 13)  /* Bit 13: Lock configuration */
#define GPDMA_CCR_BSEL                       (1 << 14)  /* Bit 14: Burst transfer selection */
#define GPDMA_CCR_DHU                        (1 << 15)  /* Bit 15: Destination hardware handshaking */
#define GPDMA_CCR_SH3                        (1 << 16)  /* Bit 16: Source AXI master handshake */
#define GPDMA_CCR_DH3                        (1 << 17)  /* Bit 17: Destination AXI master handshake */
#define GPDMA_CCR_SINC                       (1 << 18)  /* Bit 18: Source address incrementation */
#define GPDMA_CCR_DINC                       (1 << 19)  /* Bit 19: Destination address incrementation */
#define GPDMA_CCR_SINCOS_SHIFT               (20)       /* Bits 20-21: Source address incrementation offset size */
#define GPDMA_CCR_SINCOS_MASK                (3 << GPDMA_CCR_SINCOS_SHIFT)
#define GPDMA_CCR_DINCOS_SHIFT               (22)       /* Bits 22-23: Destination address incrementation offset size */
#define GPDMA_CCR_DINCOS_MASK                (3 << GPDMA_CCR_DINCOS_SHIFT)
#define GPDMA_CCR_DSTBURST_SHIFT             (24)       /* Bits 24-25: Destination burst length */
#define GPDMA_CCR_DSTBURST_MASK              (3 << GPDMA_CCR_DSTBURST_SHIFT)
#define GPDMA_CCR_SRCBURST_SHIFT             (26)       /* Bits 26-27: Source burst length */
#define GPDMA_CCR_SRCBURST_MASK              (3 << GPDMA_CCR_SRCBURST_SHIFT)
#define GPDMA_CCR_DWSEL_SHIFT                (28)       /* Bits 28-29: Destination data width selection */
#define GPDMA_CCR_DWSEL_MASK                 (3 << GPDMA_CCR_DWSEL_SHIFT)
#  define GPDMA_CCR_DWSEL_BYTE               (0 << GPDMA_CCR_DWSEL_SHIFT)  /* 8-bit */
#  define GPDMA_CCR_DWSEL_HALFWORD           (1 << GPDMA_CCR_DWSEL_SHIFT)  /* 16-bit */
#  define GPDMA_CCR_DWSEL_WORD               (2 << GPDMA_CCR_DWSEL_SHIFT)  /* 32-bit */
#  define GPDMA_CCR_DWSEL_DOUBLEWORD         (3 << GPDMA_CCR_DWSEL_SHIFT)  /* 64-bit */
#define GPDMA_CCR_SWSEL_SHIFT                (30)       /* Bits 30-31: Source data width selection */
#define GPDMA_CCR_SWSEL_MASK                 (3 << GPDMA_CCR_SWSEL_SHIFT)
#  define GPDMA_CCR_SWSEL_BYTE               (0 << GPDMA_CCR_SWSEL_SHIFT)  /* 8-bit */
#  define GPDMA_CCR_SWSEL_HALFWORD           (1 << GPDMA_CCR_SWSEL_SHIFT)  /* 16-bit */
#  define GPDMA_CCR_SWSEL_WORD               (2 << GPDMA_CCR_SWSEL_SHIFT)  /* 32-bit */
#  define GPDMA_CCR_SWSEL_DOUBLEWORD         (3 << GPDMA_CCR_SWSEL_SHIFT)  /* 64-bit */

/* Channel Number of Data Register (CNDTR) */

#define GPDMA_CNDTR_NDT_SHIFT                (0)        /* Bits 0-15: Number of data to transfer */
#define GPDMA_CNDTR_NDT_MASK                 (0xFFFF << GPDMA_CNDTR_NDT_SHIFT)

/* Channel Block Register 1 (CBR1) */

#define GPDMA_CBR1_BNDT0_SHIFT               (0)        /* Bits 0-15: Block number of data transfers */
#define GPDMA_CBR1_BNDT0_MASK                (0xFFFF << GPDMA_CBR1_BNDT0_SHIFT)
#define GPDMA_CBR1_BRCNT_SHIFT               (16)       /* Bits 16-23: Block repeat count */
#define GPDMA_CBR1_BRCNT_MASK                (0xFF << GPDMA_CBR1_BRCNT_SHIFT)

/* Channel Transfer Register 1 (CTR1) */

#define GPDMA_CTR1_PFREQ_SHIFT               (0)        /* Bits 0-5: Request generator signal frequency */
#define GPDMA_CTR1_PFREQ_MASK                (0x3F << GPDMA_CTR1_PFREQ_SHIFT)
#define GPDMA_CTR1_PBURST_SHIFT              (6)        /* Bits 6-8: Source burst length */
#define GPDMA_CTR1_PBURST_MASK               (7 << GPDMA_CTR1_PBURST_SHIFT)
#define GPDMA_CTR1_BRDUM                     (1 << 9)   /* Bit 9: Destination block repeat address update mode */
#define GPDMA_CTR1_SRCREQEN                  (1 << 10)  /* Bit 10: Source request enable */
#define GPDMA_CTR1_DSTREQEN                  (1 << 11)  /* Bit 11: Destination request enable */
#define GPDMA_CTR1_RDUM                      (1 << 12)  /* Bit 12: Destination address update mode */
#define GPDMA_CTR1_TDUS                      (1 << 13)  /* Bit 13: Trigger DMA request at source event */
#define GPDMA_CTR1_LINKUPDATE                (1 << 14)  /* Bit 14: Update registers after linked-list item */
#define GPDMA_CTR1_SSWREQ                    (1 << 15)  /* Bit 15: Software request on source */
#define GPDMA_CTR1_DBUS                      (1 << 16)  /* Bit 16: Destination address burst */
#define GPDMA_CTR1_SBUS                      (1 << 17)  /* Bit 17: Source address burst */
#define GPDMA_CTR1_DRL                       (1 << 18)  /* Bit 18: Destination reload */
#define GPDMA_CTR1_SRL                       (1 << 19)  /* Bit 19: Source reload */
#define GPDMA_CTR1_DREQ                      (1 << 20)  /* Bit 20: Destination hardware request */
#define GPDMA_CTR1_SREQ                      (1 << 21)  /* Bit 21: Source hardware request */
#define GPDMA_CTR1_TRGSEL0_SHIFT             (24)       /* Bits 24-26: Trigger selection */
#define GPDMA_CTR1_TRGSEL0_MASK              (7 << GPDMA_CTR1_TRGSEL0_SHIFT)

/* Channel Status Register (CSR) */

#define GPDMA_CSR_TCF                        (1 << 0)   /* Bit 0: Transfer complete flag */
#define GPDMA_CSR_TCFS                       (1 << 1)   /* Bit 1: Transfer complete flag suspended */
#define GPDMA_CSR_SUSPF                      (1 << 8)   /* Bit 8: Channel suspended flag */
#define GPDMA_CSR_USEF                       (1 << 9)   /* Bit 9: User setting error flag */
#define GPDMA_CSR_UCE                       (1 << 10)  /* Bit 10: User configuration error flag */
#define GPDMA_CSR_DTEF                       (1 << 11)  /* Bit 11: Data transfer error flag */
#define GPDMA_CSR_TOF                        (1 << 12)  /* Bit 12: Trigger overrun flag */
#define GPDMA_CSR_BDDF                       (1 << 16)  /* Bit 16: Buffer data discarded flag */
#define GPDMA_CSR_BRCF                       (1 << 17)  /* Bit 17: Block repeat complete flag */
#define GPDMA_CSR_BTFF                       (1 << 18)  /* Bit 18: Block transfer finished flag */
#define GPDMA_CSR_LTF                        (1 << 19)  /* Bit 19: List transaction finished flag */
#define GPDMA_CSR_TCBUSY                     (1 << 24)  /* Bit 24: Transaction busy */
#define GPDMA_CSR_CHBUSY                     (1 << 25)  /* Bit 25: Channel busy */

/* Channel Flag Clear Register (CFCR) */

#define GPDMA_CFCR_CTCF                      (1 << 0)   /* Bit 0: Clear transfer complete flag */
#define GPDMA_CFCR_CTCFS                     (1 << 1)   /* Bit 1: Clear transfer complete flag suspended */
#define GPDMA_CFCR_CSUSPF                    (1 << 8)   /* Bit 8: Clear channel suspended flag */
#define GPDMA_CFCR_CUSEF                     (1 << 9)   /* Bit 9: Clear user setting error flag */
#define GPDMA_CFCR_CUCE                     (1 << 10)  /* Bit 10: Clear user configuration error flag */
#define GPDMA_CFCR_CDTEF                     (1 << 11)  /* Bit 11: Clear data transfer error flag */
#define GPDMA_CFCR_CTOF                      (1 << 12)  /* Bit 12: Clear trigger overrun flag */
#define GPDMA_CFCR_CBDDF                     (1 << 16)  /* Bit 16: Clear buffer data discarded flag */
#define GPDMA_CFCR_CBRCF                     (1 << 17)  /* Bit 17: Clear block repeat complete flag */
#define GPDMA_CFCR_CBTFF                     (1 << 18)  /* Bit 18: Clear block transfer finished flag */
#define GPDMA_CFCR_CLTF                      (1 << 19)  /* Bit 19: Clear list transaction finished flag */

/* Channel Group Flag Register (CGFR) */

#define GPDMA_CGFR_IDMACH0F                  (1 << 0)   /* Bit 0: GPDMA channel x interrupt done mask */
#define GPDMA_CGFR_IDMACH1F                  (1 << 1)   /* Bit 1: GPDMA channel 1 interrupt done mask */
#define GPDMA_CGFR_IDMACH2F                  (1 << 2)   /* Bit 2: GPDMA channel 2 interrupt done mask */
#define GPDMA_CGFR_IDMACH3F                  (1 << 3)   /* Bit 3: GPDMA channel 3 interrupt done mask */
#define GPDMA_CGFR_IDMACH4F                  (1 << 4)   /* Bit 4: GPDMA channel 4 interrupt done mask */
#define GPDMA_CGFR_IDMACH5F                  (1 << 5)   /* Bit 5: GPDMA channel 5 interrupt done mask */
#define GPDMA_CGFR_IDMACH6F                  (1 << 6)   /* Bit 6: GPDMA channel 6 interrupt done mask */
#define GPDMA_CGFR_IDMACH7F                  (1 << 7)   /* Bit 7: GPDMA channel 7 interrupt done mask */
#define GPDMA_CGFR_IDMACH8F                  (1 << 8)   /* Bit 8: GPDMA channel 8 interrupt done mask */
#define GPDMA_CGFR_IDMACH9F                  (1 << 9)   /* Bit 9: GPDMA channel 9 interrupt done mask */
#define GPDMA_CGFR_IDMACH10F                 (1 << 10)  /* Bit 10: GPDMA channel 10 interrupt done mask */
#define GPDMA_CGFR_IDMACH11F                 (1 << 11)  /* Bit 11: GPDMA channel 11 interrupt done mask */
#define GPDMA_CGFR_IDMACH12F                 (1 << 12)  /* Bit 12: GPDMA channel 12 interrupt done mask */
#define GPDMA_CGFR_IDMACH13F                 (1 << 13)  /* Bit 13: GPDMA channel 13 interrupt done mask */
#define GPDMA_CGFR_IDMACH14F                 (1 << 14)  /* Bit 14: GPDMA channel 14 interrupt done mask */
#define GPDMA_CGFR_IDMACH15F                 (1 << 15)  /* Bit 15: GPDMA channel 15 interrupt done mask */

#endif /* __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_GPDMA_H */