/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_eth.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_ETH_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_ETH_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Ethernet Switch Module (ESWM) Base Address */

#define RA8P_ESWM_BASE                        (0x403c8000)

/* Ethernet MAC Base Addresses */

#define RA8P_ETHERC0_BASE                     (0x403cb000) /* Ethernet MAC 0 */
#define RA8P_ETHERC1_BASE                     (0x403cd000) /* Ethernet MAC 1 */

/* MDIO Base Addresses */

#define RA8P_MDIO0_BASE                       (0x403cb000) /* MDIO 0 */
#define RA8P_MDIO1_BASE                       (0x403cd000) /* MDIO 1 */

/* ESW (Ethernet Switch) Register Offsets */

#define RA8P_ESW_LAB_OFFSET                   (0x000)     /* Link Aggregation Block */
#define RA8P_ESW_GMACB_OFFSET                 (0x100)     /* GMAC Block */
#define RA8P_ESW_MIB_OFFSET                   (0x200)     /* MIB Block */
#define RA8P_ESW_CAM_OFFSET                   (0x400)     /* CAM Block */
#define RA8P_ESW_TSB_OFFSET                   (0x600)     /* TSB Block */
#define RA8P_ESW_TLB_OFFSET                   (0x800)     /* TLB Block */
#define RA8P_ESW_TPB_OFFSET                   (0xa00)     /* TPB Block */
#define RA8P_ESW_RMAC_OFFSET                  (0xc00)     /* RMAC Block */
#define RA8P_ESW_RTSB_OFFSET                  (0xe00)     /* RTSB Block */
#define RA8P_ESW_RLB_OFFSET                   (0x1000)    /* RLB Block */
#define RA8P_ESW_RPB_OFFSET                   (0x1200)    /* RPB Block */
#define RA8P_ESW_COMA_OFFSET                  (0x1400)    /* COMA Block */
#define RA8P_ESW_TOPS_OFFSET                  (0x1600)    /* TOPS Block */
#define RA8P_ESW_GPTA_OFFSET                  (0x1800)    /* GPTA Block */
#define RA8P_ESW_GWCA_OFFSET                  (0x1a00)    /* GWCA Block */

/* ETHERC (Ethernet Controller) Register Offsets */

#define RA8P_ETHERC_ECMR_OFFSET               (0x000)     /* Ethernet Control Register */
#define RA8P_ETHERC_RFLR_OFFSET               (0x008)     /* Receive Frame Length Register */
#define RA8P_ETHERC_BCFRR_OFFSET              (0x00c)     /* Broadcast Frame Receive Register */
#define RA8P_ETHERC_MAHR_OFFSET               (0x010)     /* MAC Address High Register */
#define RA8P_ETHERC_MALR_OFFSET               (0x014)     /* MAC Address Low Register */
#define RA8P_ETHERC_TPAUSER_OFFSET            (0x018)     /* Transmit Pause Address Register */
#define RA8P_ETHERC_TPALR_OFFSET              (0x01c)     /* Transmit Pause Address Register */
#define RA8P_ETHERC_TSR_OFFSET                (0x020)     /* Transmit Status Register */
#define RA8P_ETHERC_RSR_OFFSET                (0x024)     /* Receive Status Register */
#define RA8P_ETHERC_ECSR_OFFSET               (0x028)     /* Ethernet Control Status Register */
#define RA8P_ETHERC_RFCR_OFFSET               (0x030)     /* Receive Frame Control Register */
#define RA8P_ETHERC_TFAR_OFFSET               (0x038)     /* Transmit FIFO Address Register */
#define RA8P_ETHERC_TFRS_OFFSET               (0x03c)     /* Transmit FIFO Size Register */
#define RA8P_ETHERC_FCFTR_OFFSET              (0x040)     /* Frame Capture Filter Register */
#define RA8P_ETHERC_RPADIR_OFFSET             (0x044)     /* Receive Padding Insert Register */
#define RA8P_ETHERC_TRIMD_OFFSET              (0x048)     /* Transmit Descriptor Interrupt Mask Register */
#define RA8P_ETHERC_RBWAR_OFFSET              (0x04c)     /* Receive Buffer Write Address Register */
#define RA8P_ETHERC_RBRAR_OFFSET              (0x050)     /* Receive Buffer Read Address Register */
#define RA8P_ETHERC_TBRAR_OFFSET              (0x054)     /* Transmit Buffer Read Address Register */
#define RA8P_ETHERC_CCR_OFFSET                (0x100)     /* Common Control Register */
#define RA8P_ETHERC_CSR_OFFSET                (0x104)     /* Common Status Register */
#define RA8P_ETHERC_TRSC_OFFSET               (0x108)     /* Transmit Receive Status Clear Register */
#define RA8P_ETHERC_TROCR_OFFSET              (0x110)     /* Transmit Retry Over Count Register */
#define RA8P_ETHERC_CDCR_OFFSET               (0x114)     /* Carrier Depletion Count Register */
#define RA8P_ETHERC_LCCR_OFFSET               (0x118)     /* Late Collision Count Register */
#define RA8P_ETHERC_CNDCR_OFFSET              (0x11c)     /* Collision Count Register */
#define RA8P_ETHERC_TLCCR_OFFSET              (0x120)     /* Transmit Lost Carrier Count Register */
#define RA8P_ETHERC_TSRQ0_OFFSET              (0x130)     /* Transmit Start Request Queue 0 */
#define RA8P_ETHERC_TSRQ1_OFFSET              (0x134)     /* Transmit Start Request Queue 1 */
#define RA8P_ETHERC_TSRQ2_OFFSET              (0x138)     /* Transmit Start Request Queue 2 */
#define RA8P_ETHERC_TSRQ3_OFFSET              (0x13c)     /* Transmit Start Request Queue 3 */
#define RA8P_ETHERC_TSRQ4_OFFSET              (0x140)     /* Transmit Start Request Queue 4 */
#define RA8P_ETHERC_TSRQ5_OFFSET              (0x144)     /* Transmit Start Request Queue 5 */
#define RA8P_ETHERC_TSRQ6_OFFSET              (0x148)     /* Transmit Start Request Queue 6 */
#define RA8P_ETHERC_TSRQ7_OFFSET              (0x14c)     /* Transmit Start Request Queue 7 */

/* EDMAC (Ethernet DMA Controller) Register Offsets */

#define RA8P_EDMAC_EDMR_OFFSET                (0x000)     /* Ethernet DMA Mode Register */
#define RA8P_EDMAC_EDTRR_OFFSET               (0x008)     /* Transmit Descriptor Ring Register */
#define RA8P_EDMAC_EDRRR_OFFSET               (0x010)     /* Receive Descriptor Ring Register */
#define RA8P_EDMAC_TDLAR_OFFSET               (0x018)     /* Transmit Descriptor List Address Register */
#define RA8P_EDMAC_RDLAR_OFFSET               (0x020)     /* Receive Descriptor List Address Register */
#define RA8P_EDMAC_EESR_OFFSET                (0x028)     /* Ethernet Event Status Register */
#define RA8P_EDMAC_EESIPR_OFFSET              (0x030)     /* Ethernet Event Status Interrupt Permission Register */
#define RA8P_EDMAC_TRSCER_OFFSET              (0x038)     /* Transmit Receive Status Clear Enable Register */
#define RA8P_EDMAC_RMFCR_OFFSET               (0x040)     /* Receive Missed Frame Count Register */
#define RA8P_EDMAC_TFTR_OFFSET                (0x048)     /* Transmit FIFO Threshold Register */
#define RA8P_EDMAC_FDR_OFFSET                 (0x050)     /* FIFO Depth Register */
#define RA8P_EDMAC_RMCR_OFFSET                (0x058)     /* Receive Mode Control Register */
#define RA8P_EDMAC_TFUCR_OFFSET               (0x064)     /* Transmit FIFO Underflow Count Register */
#define RA8P_EDMAC_RFOCR_OFFSET               (0x068)     /* Receive FIFO Overflow Count Register */
#define RA8P_EDMAC_RBFCR_OFFSET               (0x06c)     /* Receive Buffer Frame Count Register */
#define RA8P_EDMAC_TIPG_OFFSET                (0x074)     /* Transmit Inter-Packet Gap Register */
#define RA8P_EDMAC_FCFTR_OFFSET               (0x078)     /* Frame Capture Filter Register */
#define RA8P_EDMAC_RPADIR_OFFSET              (0x07c)     /* Receive Padding Insert Register */
#define RA8P_EDMAC_FDR0_OFFSET                (0x080)     /* FIFO Depth Register 0 */
#define RA8P_EDMAC_FDR1_OFFSET                (0x084)     /* FIFO Depth Register 1 */
#define RA8P_EDMAC_FDR2_OFFSET                (0x088)     /* FIFO Depth Register 2 */
#define RA8P_EDMAC_FDR3_OFFSET                (0x08c)     /* FIFO Depth Register 3 */
#define RA8P_EDMAC_NCMDR_OFFSET               (0x090)     /* Network Command Register */
#define RA8P_EDMAC_ECMR_OFFSET                (0x100)     /* Ethernet Control Mode Register */
#define RA8P_EDMAC_RFLR_OFFSET                (0x108)     /* Receive Frame Length Register */
#define RA8P_EDMAC_BCFRR_OFFSET               (0x10c)     /* Broadcast Frame Receive Register */
#define RA8P_EDMAC_MAHR_OFFSET                (0x110)     /* MAC Address High Register */
#define RA8P_EDMAC_MALR_OFFSET                (0x114)     /* MAC Address Low Register */
#define RA8P_EDMAC_TPAUSER_OFFSET             (0x118)     /* Transmit Pause Address Register */
#define RA8P_EDMAC_TPALR_OFFSET               (0x11c)     /* Transmit Pause Address Register */
#define RA8P_EDMAC_TSR_OFFSET                 (0x120)     /* Transmit Status Register */
#define RA8P_EDMAC_RSR_OFFSET                 (0x124)     /* Receive Status Register */
#define RA8P_EDMAC_ECSR_OFFSET                (0x128)     /* Ethernet Control Status Register */
#define RA8P_EDMAC_RFCR_OFFSET                (0x130)     /* Receive Frame Control Register */
#define RA8P_EDMAC_TFAR_OFFSET                (0x138)     /* Transmit FIFO Address Register */
#define RA8P_EDMAC_TFRS_OFFSET                (0x13c)     /* Transmit FIFO Size Register */
#define RA8P_EDMAC_FCFTR_OFFSET               (0x140)     /* Frame Capture Filter Register */
#define RA8P_EDMAC_RPADIR_OFFSET              (0x144)     /* Receive Padding Insert Register */

/* ETHERC ECMR Register Bits */

#define RA8P_ETHERC_ECMR_TRCCM                (1 << 0)    /* Bit 0: Transmit Clock Counter Mode */
#define RA8P_ETHERC_ECMR_RZC                  (1 << 1)    /* Bit 1: Receive Zero Count */
#define RA8P_ETHERC_ECMR_RZCEN                (1 << 2)    /* Bit 2: Receive Zero Count Enable */
#define RA8P_ETHERC_ECMR_RE                   (1 << 6)    /* Bit 6: Receive Enable */
#define RA8P_ETHERC_ECMR_TE                   (1 << 7)    /* Bit 7: Transmit Enable */
#define RA8P_ETHERC_ECMR_DM                   (1 << 8)    /* Bit 8: Duplex Mode */
#define RA8P_ETHERC_ECMR_RTM                  (1 << 9)    /* Bit 9: Receive Timer Mode */
#define RA8P_ETHERC_ECMR_ILB                  (1 << 11)   /* Bit 11: Internal Loopback */
#define RA8P_ETHERC_ECMR_PRM                  (1 << 12)   /* Bit 12: Promiscuous Mode */
#define RA8P_ETHERC_ECMR_MCT                  (1 << 13)   /* Bit 13: Multicast */
#define RA8P_ETHERC_ECMR_MPDE                 (1 << 18)   /* Bit 18: Magic Packet Detection Enable */
#define RA8P_ETHERC_ECMR_LCH                  (1 << 20)   /* Bit 20: Link Check */
#define RA8P_ETHERC_ECMR_CAFF                 (1 << 21)   /* Bit 21: Capture Frame Filter */
#define RA8P_ETHERC_ECMR_RPF                  (1 << 22)   /* Bit 22: Receive Pause Frame */
#define RA8P_ETHERC_ECMR_PFR                  (1 << 23)   /* Bit 23: Pause Frame Response */
#define RA8P_ETHERC_ECMR_ZPF                  (1 << 24)   /* Bit 24: Zero Pause Frame */
#define RA8P_ETHERC_ECMR_RCRC                 (1 << 25)   /* Bit 25: Receive CRC Check */
#define RA8P_ETHERC_ECMR_DRC                  (1 << 26)   /* Bit 26: Disable Receive CRC */
#define RA8P_ETHERC_ECMR_RCPT                 (1 << 27)   /* Bit 27: Receive Packet */
#define RA8P_ETHERC_ECMR_PAD                  (1 << 28)   /* Bit 28: Padding */
#define RA8P_ETHERC_ECMR_RST                  (1 << 29)   /* Bit 29: Reset */
#define RA8P_ETHERC_ECMR_TXF                  (1 << 30)   /* Bit 30: Transmit Frame */
#define RA8P_ETHERC_ECMR_RXF                  (1 << 31)   /* Bit 31: Receive Frame */

/* EDMAC EDTRR Register Bits */

#define RA8P_EDMAC_EDTRR_TR                   (1 << 0)    /* Bit 0: Transmit Request */

/* EDMAC EDRRR Register Bits */

#define RA8P_EDMAC_EDRRR_RR                   (1 << 0)    /* Bit 0: Receive Request */

/* EDMAC EESR Register Bits */

#define RA8P_EDMAC_EESR_TWB                   (1 << 0)    /* Bit 0: Transmit Write Back */
#define RA8P_EDMAC_EESR_TC                    (1 << 1)    /* Bit 1: Transmit Complete */
#define RA8P_EDMAC_EESR_FR                    (1 << 18)   /* Bit 18: Frame Receive */
#define RA8P_EDMAC_EESR_RDE                   (1 << 20)   /* Bit 20: Receive Descriptor Empty */
#define RA8P_EDMAC_EESR_TDE                   (1 << 21)   /* Bit 21: Transmit Descriptor Empty */
#define RA8P_EDMAC_EESR_TFE                   (1 << 22)   /* Bit 22: Transmit FIFO Empty */
#define RA8P_EDMAC_EESR_RFOF                  (1 << 24)   /* Bit 24: Receive FIFO Overflow */
#define RA8P_EDMAC_EESR_TFUF                  (1 << 25)   /* Bit 25: Transmit FIFO Underflow */
#define RA8P_EDMAC_EESR_CND                   (1 << 27)   /* Bit 27: Carrier No Detect */
#define RA8P_EDMAC_EESR_DLC                   (1 << 28)   /* Bit 28: Data Link Error */
#define RA8P_EDMAC_EESR_CD                    (1 << 29)   /* Bit 29: Carrier Detect */
#define RA8P_EDMAC_EESR_RMAF                  (1 << 30)   /* Bit 30: Receive Multicast Address Frame */
#define RA8P_EDMAC_EESR_TWB                   (1 << 31)   /* Bit 31: Transmit Write Back */

/* MDIO Register Offsets */

#define RA8P_MDIO_MPSM_OFFSET                 (0x000)     /* MDIO PHY Select Mode Register */
#define RA8P_MDIO_MPIC_OFFSET                 (0x004)     /* MDIO PHY Interface Control Register */
#define RA8P_MDIO_MPDRC_OFFSET                (0x008)     /* MDIO PHY Data Read Register */
#define RA8P_MDIO_MPDWC_OFFSET                (0x00c)     /* MDIO PHY Data Write Register */

/* MDIO MPSM Register Bits */

#define RA8P_MDIO_MPSM_MDC_MASK               (0x7F)      /* Bits 0-6: MDC Clock Division */
#define RA8P_MDIO_MPSM_MDC_SHIFT              (0)
#define RA8P_MDIO_MPSM_CLS                    (1 << 7)    /* Bit 7: Clock Select */
#define RA8P_MDIO_MPSM_MPE                    (1 << 8)    /* Bit 8: MDIO PHY Enable */
#define RA8P_MDIO_MPSM_MMR                    (1 << 9)    /* Bit 9: MDIO Mode Register */
#define RA8P_MDIO_MPSM_MPR                    (1 << 10)   /* Bit 10: MDIO PHY Read */
#define RA8P_MDIO_MPSM_MPW                    (1 << 11)   /* Bit 11: MDIO PHY Write */

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_ETH_H */