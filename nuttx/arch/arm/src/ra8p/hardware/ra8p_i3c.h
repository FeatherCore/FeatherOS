/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_i3c.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_I3C_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_I3C_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* I3C Base Address */

#define RA8P_I3C0_BASE                        (0x4035f000) /* I3C0 base address */

/* I3C Register Offsets */

/* Global Control Register (MSTICR) */

#define RA8P_I3C_MSTICR_OFFSET                (0x000)
#define RA8P_I3C_MSTICR_MI3CEN                (1 << 0)   /* Bit 0: Master I3C Enable */
#define RA8P_I3C_MSTICR_MSCLHDUR              (1 << 1)   /* Bit 1: Multi-byte SCL High Duration */
#define RA8P_I3C_MSTICR_SPPURG                (1 << 2)   /* Bit 2: SPURG Enable */
#define RA8P_I3C_MSTICR_BDAAEN                (1 << 3)   /* Bit 3: Bus Detection Auto-Apply Enable */
#define RA8P_I3C_MSTICR_BDAPEN                (1 << 4)   /* Bit 4: Bus Detection Apply Enable */

/* Device Control Register (MSTDCTRC) */

#define RA8P_I3C_MSTDCTRC_OFFSET              (0x004)
#define RA8P_I3C_MSTDCTRC_DCTRE               (1 << 0)   /* Bit 0: Device Characteristic Table Register Enable */
#define RA8P_I3C_MSTDCTRC_DCAS                (1 << 1)   /* Bit 1: Device Characteristic Table Address Stop */
#define RA8P_I3C_MSTDCTRC_DCBRS               (1 << 2)   /* Bit 2: Device Characteristic Table BR Stop */
#define RA8P_I3C_MSTDCTRC_DCBCAS              (1 << 3)   /* Bit 3: Device Characteristic Table BCAS Stop */

/* Data Byte Count Register (MSTDBCT) */

#define RA8P_I3C_MSTDBCT_OFFSET               (0x008)
#define RA8P_I3C_MSTDBCT_DBCT_MASK            (0xFF)     /* Bits 0-7: Data Byte Count */

/* Data Register (MSTDAT) */

#define RA8P_I3C_MSTDAT_OFFSET                (0x00c)
#define RA8P_I3C_MSTDAT_DAT_MASK              (0xFF)     /* Bits 0-7: Data */

/* Command Request Register (MSTCRQ) */

#define RA8P_I3C_MSTCRQ_OFFSET                (0x010)
#define RA8P_I3C_MSTCRQ_CRQA_MASK             (0xFF)     /* Bits 0-7: Command Request A */
#define RA8P_I3C_MSTCRQ_CRQB_MASK             (0xFF00)   /* Bits 8-15: Command Request B */
#define RA8P_I3C_MSTCRQ_CRQEN                 (1 << 16)  /* Bit 16: Command Request Enable */

/* Response Register (MSTRSP) */

#define RA8P_I3C_MSTRSP_OFFSET                (0x014)
#define RA8P_I3C_MSTRSP_RSP_MASK              (0x3)      /* Bits 0-1: Response */

/* Master Status Register (MSTSTAT) */

#define RA8P_I3C_MSTSTAT_OFFSET               (0x018)
#define RA8P_I3C_MSTSTAT_IBAMT                (1 << 0)   /* Bit 0: IBAM Transfer Complete */
#define RA8P_I3C_MSTSTAT_IBARX                (1 << 1)   /* Bit 1: IBAM Receive Complete */
#define RA8P_I3C_MSTSTAT_IBATX                (1 << 2)   /* Bit 2: IBAM Transmit Complete */
#define RA8P_I3C_MSTSTAT_MCTRL                (1 << 3)   /* Bit 3: Master Controller Error */
#define RA8P_I3C_MSTSTAT_HJCNCT               (1 << 4)   /* Bit 4: Hot-Join Connect */
#define RA8P_I3C_MSTSTAT_DDARCV               (1 << 5)   /* Bit 5: Dynamic Address Receiving */
#define RA8P_I3C_MSTSTAT_MBUSY                (1 << 6)   /* Bit 6: Master Busy */
#define RA8P_I3C_MSTSTAT_SLVADR               (1 << 7)   /* Bit 7: Slave Address Reception */

/* Master Interrupt Enable Register (MSTINT) */

#define RA8P_I3C_MSTINT_OFFSET                (0x01c)
#define RA8P_I3C_MSTINT_IBAMTE                (1 << 0)   /* Bit 0: IBAM Transfer Complete Enable */
#define RA8P_I3C_MSTINT_IBARXE                (1 << 1)   /* Bit 1: IBAM Receive Complete Enable */
#define RA8P_I3C_MSTINT_IBATXE                (1 << 2)   /* Bit 2: IBAM Transmit Complete Enable */
#define RA8P_I3C_MSTINT_MCTRLE                (1 << 3)   /* Bit 3: Master Controller Error Enable */
#define RA8P_I3C_MSTINT_HJCNCTE               (1 << 4)   /* Bit 4: Hot-Join Connect Enable */
#define RA8P_I3C_MSTINT_DDARCVE               (1 << 5)   /* Bit 5: Dynamic Address Receive Enable */

/* Static Address Table Register (STAR0-STAR7) */

#define RA8P_I3C_STAR0_OFFSET                 (0x020)
#define RA8P_I3C_STAR1_OFFSET                 (0x024)
#define RA8P_I3C_STAR2_OFFSET                 (0x028)
#define RA8P_I3C_STAR3_OFFSET                 (0x02c)
#define RA8P_I3C_STAR4_OFFSET                 (0x030)
#define RA8P_I3C_STAR5_OFFSET                 (0x034)
#define RA8P_I3C_STAR6_OFFSET                 (0x038)
#define RA8P_I3C_STAR7_OFFSET                 (0x03c)

/* Device Characteristic Table Register (DCTRF0-DCTRF15) */

#define RA8P_I3C_DCTRF0_OFFSET                (0x080)
#define RA8P_I3C_DCTRF1_OFFSET                (0x084)
#define RA8P_I3C_DCTRF2_OFFSET                (0x088)
#define RA8P_I3C_DCTRF3_OFFSET                (0x08c)
/* ... up to DCTRF15 */

/* Timing Parameter Register (TIMING) */

#define RA8P_I3C_TIMING_OFFSET                (0x100)
#define RA8P_I3C_TIMING_ODDF0_MASK            (0xFF)     /* Bits 0-7: Open-Drain Digital Filter 0 */
#define RA8P_I3C_TIMING_PUSHTPULLF0_MASK      (0xFF00)   /* Bits 8-15: Push-Pull Digital Filter 0 */

/* Bus Available Time Control Register (BATCTL) */

#define RA8P_I3C_BATCTL_OFFSET                (0x104)
#define RA8P_I3C_BATCTL_BATOVAL_MASK          (0xFFFF)   /* Bits 0-15: Bus Available Time-out Value */

/* Slave Status Register (SLVSTAT) */

#define RA8P_I3C_SLVSTAT_OFFSET               (0x200)
#define RA8P_I3C_SLVSTAT_SLVRXPEND            (1 << 0)   /* Bit 0: Slave Receive Pending */
#define RA8P_I3C_SLVSTAT_SLVTXPEND            (1 << 1)   /* Bit 1: Slave Transmit Pending */
#define RA8P_I3C_SLVSTAT_SLVADR_MATCH         (1 << 2)   /* Bit 2: Slave Address Match */
#define RA8P_I3C_SLVSTAT_HDRMATCH             (1 << 3)   /* Bit 3: HDR Mode Match */

/* Slave Interrupt Enable Register (SLVINT) */

#define RA8P_I3C_SLVINT_OFFSET                (0x204)
#define RA8P_I3C_SLVINT_SLVRXPENDE            (1 << 0)   /* Bit 0: Slave Receive Pending Enable */
#define RA8P_I3C_SLVINT_SLVTXPENDE            (1 << 1)   /* Bit 1: Slave Transmit Pending Enable */
#define RA8P_I3C_SLVINT_SLVADRMATCHE          (1 << 2)   /* Bit 2: Slave Address Match Enable */
#define RA8P_I3C_SLVINT_HDRMATCHE             (1 << 3)   /* Bit 3: HDR Mode Match Enable */

/* Slave Data Register (SLVDAT) */

#define RA8P_I3C_SLVDAT_OFFSET                (0x208)
#define RA8P_I3C_SLVDAT_SLVDAT_MASK           (0xFF)     /* Bits 0-7: Slave Data */

/* Slave Address Register (SLVADR) */

#define RA8P_I3C_SLVADR_OFFSET                (0x20c)
#define RA8P_I3C_SLVADR_SLVADR_MASK           (0x7F)     /* Bits 0-6: Slave Address */
#define RA8P_I3C_SLVADR_SLVADR_EN             (1 << 7)   /* Bit 7: Slave Address Enable */

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_I3C_H */