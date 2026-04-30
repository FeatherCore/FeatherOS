/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_entropy.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_ENTROPY_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_ENTROPY_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* TRNG (True Random Number Generator) Base Address */

#define RA8P_TRNG_BASE                          (0x4022a000)

/* TRNG Register Offsets */

/* TRNG Control Register (TRNGCR) */

#define RA8P_TRNG_TRNGCR_OFFSET                 (0x000)
#define RA8P_TRNG_TRNGCR_TRNGEN                 (1 << 0)    /* Bit 0: TRNG Enable */
#define RA8P_TRNG_TRNGCR_TRNGRST                (1 << 1)    /* Bit 1: TRNG Reset */
#define RA8P_TRNG_TRNGCR_TRNGMD_MASK            (0x30)      /* Bits 4-5: TRNG Mode */
#define RA8P_TRNG_TRNGCR_TRNGMD_SHIFT           (4)
#define RA8P_TRNG_TRNGCR_TRNGMD_NORMAL          (0x0 << 4)  /* Normal mode */
#define RA8P_TRNG_TRNGCR_TRNGMD_SELF_TEST       (0x1 << 4)  /* Self-test mode */

/* TRNG Status Register (TRNGSTR) */

#define RA8P_TRNG_TRNGSTR_OFFSET                (0x004)
#define RA8P_TRNG_TRNGSTR_TRNGST                (1 << 0)    /* Bit 0: TRNG Status */
#define RA8P_TRNG_TRNGSTR_TRNGRDY               (1 << 1)    /* Bit 1: TRNG Ready */
#define RA8P_TRNG_TRNGSTR_TRNGERR               (1 << 2)    /* Bit 2: TRNG Error */

/* TRNG Data Register (TRNGDR) */

#define RA8P_TRNG_TRNGDR_OFFSET                 (0x008)
#define RA8P_TRNG_TRNGDR_DATA_MASK              (0xFFFFFFFF) /* Bits 0-31: Random Data */

/* TRNG Interrupt Enable Register (TRNGIER) */

#define RA8P_TRNG_TRNGIER_OFFSET                (0x00C)
#define RA8P_TRNG_TRNGIER_TRNGIE                (1 << 0)    /* Bit 0: TRNG Interrupt Enable */
#define RA8P_TRNG_TRNGIER_TRNGEIE               (1 << 1)    /* Bit 1: TRNG Error Interrupt Enable */

/* TRNG Interrupt Status Register (TRNGISR) */

#define RA8P_TRNG_TRNGISR_OFFSET                (0x010)
#define RA8P_TRNG_TRNGISR_TRNGIF                (1 << 0)    /* Bit 0: TRNG Interrupt Flag */
#define RA8P_TRNG_TRNGISR_TRNGEIF               (1 << 1)    /* Bit 1: TRNG Error Interrupt Flag */

/* RSIP-E50D (Cryptographic Accelerator) Base Address */

#define RA8P_RSIP_BASE                          (0x4022b000)

/* RSIP Control Register (RSIPCR) */

#define RA8P_RSIP_RSIPCR_OFFSET                 (0x000)
#define RA8P_RSIP_RSIPCR_RSIPEN                 (1 << 0)    /* Bit 0: RSIP Enable */
#define RA8P_RSIP_RSIPCR_RSIPRST                (1 << 1)    /* Bit 1: RSIP Reset */

/* RSIP Status Register (RSIPSTR) */

#define RA8P_RSIP_RSIPSTR_OFFSET                (0x004)
#define RA8P_RSIP_RSIPSTR_RSIPST                (1 << 0)    /* Bit 0: RSIP Status */
#define RA8P_RSIP_RSIPSTR_RSIPRDY               (1 << 1)    /* Bit 1: RSIP Ready */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* Entropy source type */

enum ra8p_entropy_source_e
{
  RA8P_ENTROPY_SOURCE_TRNG = 0,     /* True Random Number Generator */
  RA8P_ENTROPY_SOURCE_RSIP,         /* RSIP-E50D Cryptographic Module */
};

/* Entropy configuration */

struct ra8p_entropy_config_s
{
  enum ra8p_entropy_source_e source; /* Entropy source */
  bool interrupt_mode;               /* Use interrupt mode */
};

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_ENTROPY_H */