/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_ospi.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_OSPI_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_OSPI_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* OSPI Base Addresses */

#define RA8P_OSPI0_BASE                        (0x4035e000) /* OSPI0 base address */
#define RA8P_OSPI1_BASE                        (0x4035f000) /* OSPI1 base address */

/* OSPI Register Offsets */

/* Control Register (CCR) */

#define RA8P_OSPI_CCR_OFFSET                   (0x000)
#define RA8P_OSPI_CCR_MOD_EN                   (1 << 0)   /* Bit 0: Memory Octal mode Enable */
#define RA8P_OSPI_CCR_PHY_RST                  (1 << 1)   /* Bit 1: PHY Reset */
#define RA8P_OSPI_CCR_HYPE_EN                  (1 << 2)   /* Bit 2: HyperBus Enable */
#define RA8P_OSPI_CCR_DLY_SEL                  (1 << 3)   /* Bit 3: Delay Selection */
#define RA8P_OSPI_CCR_DTR_EN                   (1 << 4)   /* Bit 4: DDR/DTR Enable */
#define RA8P_OSPI_CCR_DLY_ADJ_EN               (1 << 5)   /* Bit 5: Delay Adjust Enable */
#define RA8P_OSPI_CCR_RW_DLY_ADJ_SEL           (1 << 6)   /* Bit 6: Read/Write Delay Adjust Selection */
#define RA8P_OSPI_CCR_DQS_DIS                  (1 << 7)   /* Bit 7: DQS Disable */

/* Clock Control Register (CKCR) */

#define RA8P_OSPI_CKCR_OFFSET                  (0x004)
#define RA8P_OSPI_CKCR_PSHDV_MASK              (0xFF)     /* Bits 0-7: Clock Phase Delay Value */
#define RA8P_OSPI_CKCR_PSHDV_SHIFT             (0)
#define RA8P_OSPI_CKCR_NSHDV_MASK              (0xFF00)   /* Bits 8-15: Clock Negative Phase Delay Value */
#define RA8P_OSPI_CKCR_NSHDV_SHIFT             (8)

/* Serial Control Register (SCR) */

#define RA8P_OSPI_SCR_OFFSET                   (0x008)
#define RA8P_OSPI_SCR_SPNDL_MASK               (0xF)      /* Bits 0-3: Suspend Length */
#define RA8P_OSPI_SCR_SPNDL_SHIFT              (0)
#define RA8P_OSPI_SCR_SSL_ASEL                 (1 << 4)   /* Bit 4: SSL Assertion Select */
#define RA8P_OSPI_SCR_SPND_EN                  (1 << 8)   /* Bit 8: Suspend Enable */
#define RA8P_OSPI_SCR_SPND                     (1 << 9)   /* Bit 9: Suspend */
#define RA8P_OSPI_SCR_WE_SET                   (1 << 16)  /* Bit 16: WE Signal Setup Time */
#define RA8P_OSPI_SCR_WE_HOLD                  (1 << 24)  /* Bit 24: WE Signal Hold Time */

/* Bus Control Register (BCR) */

#define RA8P_OSPI_BCR_OFFSET                   (0x00c)
#define RA8P_OSPI_BCR_DME                      (1 << 0)   /* Bit 0: Dummy Enable */
#define RA8P_OSPI_BCR_OCT                      (1 << 1)   /* Bit 1: Octal mode */
#define RA8P_OSPI_BCR_SSLKP                    (1 << 2)   /* Bit 2: SSLKP Signal Output */
#define RA8P_OSPI_BCR_TBCK                     (1 << 3)   /* Bit 3: Toggle Clock Output */
#define RA8P_OSPI_BCR_TCSH                     (1 << 4)   /* Bit 4: SSL Negation Period */
#define RA8P_OSPI_BCR_TCHL                     (1 << 5)   /* Bit 5: SSL Assertion Period */
#define RA8P_OSPI_BCR_TEAE                     (1 << 6)   /* Bit 6: Early Warning Enable */
#define RA8P_OSPI_BCR_CMD                      (1 << 7)   /* Bit 7: Command Enable */
#define RA8P_OSPI_BCR_DTC                      (1 << 8)   /* Bit 8: Data Counter Enable */
#define RA8P_OSPI_BCR_SSL                      (1 << 9)   /* Bit 9: SSL Signal Level */
#define RA8P_OSPI_BCR_DUP                      (1 << 10)  /* Bit 10: Dual Quad Port */
#define RA8P_OSPI_BCR_EIVM                     (1 << 11)  /* Bit 11: External Address Invalidation Mode */
#define RA8P_OSPI_BCR_DRM                      (1 << 12)  /* Bit 12: Dummy Read Mode */

/* Bus Control Register 2 (BCR2) */

#define RA8P_OSPI_BCR2_OFFSET                  (0x010)
#define RA8P_OSPI_BCR2_DHCNT_MASK              (0xFF)     /* Bits 0-7: Dummy Counter */
#define RA8P_OSPI_BCR2_DHCNT_SHIFT             (0)
#define RA8P_OSPI_BCR2_DLF_MASK                (0xF00)    /* Bits 8-11: Data Length Field */
#define RA8P_OSPI_BCR2_DLF_SHIFT               (8)
#define RA8P_OSPI_BCR2_OCDE_MASK               (0xF000)   /* Bits 12-15: Octal Command Extension */
#define RA8P_OSPI_BCR2_OCDE_SHIFT              (12)

/* Command Register (CMDR) */

#define RA8P_OSPI_CMDR_OFFSET                  (0x014)
#define RA8P_OSPI_CMDR_CMDS_MASK               (0xFF)     /* Bits 0-7: Command Setting */
#define RA8P_OSPI_CMDR_CMDS_SHIFT              (0)

/* Address Register (ADDR) */

#define RA8P_OSPI_ADDR_OFFSET                  (0x018)
#define RA8P_OSPI_ADDR_ADDRS_MASK              (0xFFFFFFFF) /* Bits 0-31: Address Setting */

/* Data Counter Register (DRCR) */

#define RA8P_OSPI_DRCR_OFFSET                  (0x01c)
#define RA8P_OSPI_DRCR_DL_MASK                 (0x3FF)    /* Bits 0-9: Data Length */
#define RA8P_OSPI_DRCR_DL_SHIFT                (0)
#define RA8P_OSPI_DRCR_RCF                     (1 << 16)  /* Bit 16: Read Command Flag */
#define RA8P_OSPI_DRCR_SSLE_MASK               (0xFF0000) /* Bits 16-23: SSL Signal Level Extension */
#define RA8P_OSPI_DRCR_SSLE_SHIFT              (16)

/* Data Register (DR) */

#define RA8P_OSPI_DR_OFFSET                    (0x020)
#define RA8P_OSPI_DR_DRS_MASK                  (0xFFFFFFFF) /* Bits 0-31: Data Setting */

/* Status Register (STR) */

#define RA8P_OSPI_STR_OFFSET                   (0x024)
#define RA8P_OSPI_STR_SUSRDY                   (1 << 0)   /* Bit 0: Suspend Ready */
#define RA8P_OSPI_STR_RXFULL                   (1 << 1)   /* Bit 1: RX FIFO Full */
#define RA8P_OSPI_STR_TXEMPTY                  (1 << 2)   /* Bit 2: TX FIFO Empty */
#define RA8P_OSPI_STR_SSLF                     (1 << 3)   /* Bit 3: SSL Flag */
#define RA8P_OSPI_STR_WBUF                     (1 << 4)   /* Bit 4: Write Buffer Flag */
#define RA8P_OSPI_STR_RXFEMP                   (1 << 5)   /* Bit 5: RX FIFO Empty */
#define RA8P_OSPI_STR_TXFFUL                   (1 << 6)   /* Bit 6: TX FIFO Full */
#define RA8P_OSPI_STR_RXFFUL                   (1 << 7)   /* Bit 7: RX FIFO Full */
#define RA8P_OSPI_STR_TXFEMP                   (1 << 8)   /* Bit 8: TX FIFO Empty */

/* DMA Enable Register (DMEN) */

#define RA8P_OSPI_DMEN_OFFSET                  (0x028)
#define RA8P_OSPI_DMEN_RX_DREQEN               (1 << 0)   /* Bit 0: RX DMA Request Enable */
#define RA8P_OSPI_DMEN_TX_DREQEN               (1 << 1)   /* Bit 1: TX DMA Request Enable */

/* Endian Control Register (ER) */

#define RA8P_OSPI_ER_OFFSET                    (0x02c)
#define RA8P_OSPI_ER_BENDIAN                   (1 << 0)   /* Bit 0: Big Endian Enable */

/* Sampling Delay Control Register (SDCR) */

#define RA8P_OSPI_SDCR_OFFSET                  (0x030)
#define RA8P_OSPI_SDCR_SPMD_MASK               (0x3)      /* Bits 0-1: Sampling Mode */
#define RA8P_OSPI_SDCR_SPMD_SHIFT              (0)
#define RA8P_OSPI_SDCR_RDDQSDLY_MASK           (0xF00)    /* Bits 8-11: Read DQS Delay */
#define RA8P_OSPI_SDCR_RDDQSDLY_SHIFT          (8)
#define RA8P_OSPI_SDCR_WDQSDLY_MASK            (0xF000)   /* Bits 12-15: Write DQS Delay */
#define RA8P_OSPI_SDCR_WDQSDLY_SHIFT           (12)

/* Cyclic Redundancy Check Register (CRCR) */

#define RA8P_OSPI_CRCR_OFFSET                  (0x034)
#define RA8P_OSPI_CRCR_CRCDV_MASK              (0xFFFF)   /* Bits 0-15: CRC Data Value */
#define RA8P_OSPI_CRCR_CRCEN                   (1 << 16)  /* Bit 16: CRC Enable */

/* Command Extension Register (COMCR) */

#define RA8P_OSPI_COMCR_OFFSET                 (0x038)
#define RA8P_OSPI_COMCR_OCDS_MASK              (0xFF)     /* Bits 0-7: Octal Command Setting */
#define RA8P_OSPI_COMCR_OCDS_SHIFT             (0)

/* Software Reset Register (SWPR) */

#define RA8P_OSPI_SWPR_OFFSET                  (0x040)
#define RA8P_OSPI_SWPR_SWRST                   (1 << 0)   /* Bit 0: Software Reset */

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_OSPI_H */