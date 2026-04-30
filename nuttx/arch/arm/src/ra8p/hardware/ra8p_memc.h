/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_memc.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_MEMC_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_MEMC_H_

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* MEMC (Memory Controller) Base Address */

#define RA8P_MEMC_BASE                         (0x40003c00)

/* MEMC Register Offsets */

/* MEMC Control Register (MEMCCR) */

#define RA8P_MEMC_MEMCCR_OFFSET                (0x000)
#define RA8P_MEMC_MEMCCR_MEMCEN               (1 << 0)    /* Bit 0: MEMC Enable */
#define RA8P_MEMC_MEMCCR_RSTEMC              (1 << 1)    /* Bit 1: MEMC Reset */

/* SDRAM Control Register 0 (SDCR0) */

#define RA8P_MEMC_SDCR0_OFFSET                 (0x004)
#define RA8P_MEMC_SDCR0_SDBA_MASK             (0x7)       /* Bits 0-2: SDRAM Base Address */
#define RA8P_MEMC_SDCR0_SDBA_SHIFT            (0)
#define RA8P_MEMC_SDCR0_SDSIZ_MASK            (0x70)      /* Bits 4-6: SDRAM Size */
#define RA8P_MEMC_SDCR0_SDSIZ_SHIFT           (4)
#define RA8P_MEMC_SDCR0_SDSIZ_16MB            (0x0 << 4)  /* 16 MB */
#define RA8P_MEMC_SDCR0_SDSIZ_64MB            (0x1 << 4)  /* 64 MB */
#define RA8P_MEMC_SDCR0_SDSIZ_128MB           (0x2 << 4)  /* 128 MB */
#define RA8P_MEMC_SDCR0_SDSIZ_256MB           (0x3 << 4)  /* 256 MB */
#define RA8P_MEMC_SDCR0_SDSIZ_512MB           (0x4 << 4)  /* 512 MB */
#define RA8P_MEMC_SDCR0_CSEL_MASK             (0x300)     /* Bits 8-9: CAS Latency */
#define RA8P_MEMC_SDCR0_CSEL_SHIFT            (8)
#define RA8P_MEMC_SDCR0_CSEL_2               (0x0 << 8)  /* CAS Latency 2 */
#define RA8P_MEMC_SDCR0_CSEL_3               (0x1 << 8)  /* CAS Latency 3 */
#define RA8P_MEMC_SDCR0_RASW_MASK            (0x7000)    /* Bits 12-14: RAS Wait */
#define RA8P_MEMC_SDCR0_RASW_SHIFT           (12)
#define RA8P_MEMC_SDCR0_SRAW_MASK            (0x70000)   /* Bits 16-18: SDRAM Row Address Width */
#define RA8P_MEMC_SDCR0_SRAW_SHIFT           (16)
#define RA8P_MEMC_SDCR0_SCAW_MASK            (0x300000)  /* Bits 20-21: SDRAM Column Address Width */
#define RA8P_MEMC_SDCR0_SCAW_SHIFT           (20)

/* SDRAM Control Register 1 (SDCR1) */

#define RA8P_MEMC_SDCR1_OFFSET                 (0x008)
#define RA8P_MEMC_SDCR1_SRR_WTR_MASK        (0xF)       /* Bits 0-3: SRR Write Recovery */
#define RA8P_MEMC_SDCR1_SRR_WTR_SHIFT       (0)
#define RA8P_MEMC_SDCR1_SRR_RTR_MASK        (0xF0)      /* Bits 4-7: SRR Read to Read */
#define RA8P_MEMC_SDCR1_SRR_RTR_SHIFT       (4)
#define RA8P_MEMC_SDCR1_SRR_RMW_MASK        (0xF00)     /* Bits 8-11: SRR Read to Write */
#define RA8P_MEMC_SDCR1_SRR_RMW_SHIFT       (8)
#define RA8P_MEMC_SDCR1_SRR_WTM_MASK        (0xF000)    /* Bits 12-15: SRR Write to Read */
#define RA8P_MEMC_SDCR1_SRR_WTM_SHIFT       (12)
#define RA8P_MEMC_SDCR1_SRR_WTW_MASK        (0xF0000)   /* Bits 16-19: SRR Write to Write */
#define RA8P_MEMC_SDCR1_SRR_WTW_SHIFT       (16)
#define RA8P_MEMC_SDCR1_SRR_REFW_MASK       (0xF00000)  /* Bits 20-23: SRR Refresh */
#define RA8P_MEMC_SDCR1_SRR_REFW_SHIFT      (20)

/* SDRAM Timing Register (SDTR) */

#define RA8P_MEMC_SDTR_OFFSET                  (0x00C)
#define RA8P_MEMC_SDTR_TRAS_MASK              (0xF)       /* Bits 0-3: tRAS Wait */
#define RA8P_MEMC_SDTR_TRAS_SHIFT             (0)
#define RA8P_MEMC_SDTR_TRCD_MASK              (0xF0)      /* Bits 4-7: tRCD Wait */
#define RA8P_MEMC_SDTR_TRCD_SHIFT             (4)
#define RA8P_MEMC_SDTR_TRC_MASK               (0xF00)     /* Bits 8-11: tRC Wait */
#define RA8P_MEMC_SDTR_TRC_SHIFT              (8)
#define RA8P_MEMC_SDTR_TRP_MASK               (0xF000)    /* Bits 12-15: tRP Wait */
#define RA8P_MEMC_SDTR_TRP_SHIFT              (12)
#define RA8P_MEMC_SDTR_TRFC_MASK              (0xF0000)   /* Bits 16-19: tRFC Wait */
#define RA8P_MEMC_SDTR_TRFC_SHIFT             (16)

/* SDRAM Refresh Register (SDRFR) */

#define RA8P_MEMC_SDRFR_OFFSET                 (0x010)
#define RA8P_MEMC_SDRFR_REFSEL_MASK           (0x7)       /* Bits 0-2: Refresh Interval Select */
#define RA8P_MEMC_SDRFR_REFSEL_SHIFT          (0)
#define RA8P_MEMC_SDRFR_REFW_MASK            (0xF0)      /* Bits 4-7: Refresh Wait */
#define RA8P_MEMC_SDRFR_REFW_SHIFT           (4)

/* SDRAM Status Register (SDSR) */

#define RA8P_MEMC_SDSR_OFFSET                  (0x014)
#define RA8P_MEMC_SDSR_SDRF                 (1 << 0)    /* Bit 0: SDRAM Refresh Flag */
#define RA8P_MEMC_SDSR_SDWBF                (1 << 1)    /* Bit 1: SDRAM Write Buffer Flag */

/* SDRAM Address Mask Register (SDAMR) */

#define RA8P_MEMC_SDAMR_OFFSET                 (0x018)
#define RA8P_MEMC_SDAMR_AM_MASK               (0xFFFFFFFF) /* Bits 0-31: Address Mask */

/* Bus Control Register (BUSCR) */

#define RA8P_MEMC_BUSCR_OFFSET                 (0x01C)
#define RA8P_MEMC_BUSCR_BE                     (1 << 0)    /* Bit 0: Bus Enable */
#define RA8P_MEMC_BUSCR_ARE                   (1 << 1)    /* Bit 1: Address Recovery Enable */
#define RA8P_MEMC_BUSCR_ARW_MASK              (0x70)      /* Bits 4-6: Address Recovery Wait */
#define RA8P_MEMC_BUSCR_ARW_SHIFT             (4)

/* Chip Select Control Register (CSCR) */

#define RA8P_MEMC_CSCR_OFFSET                  (0x020)
#define RA8P_MEMC_CSCR_CE                      (1 << 0)    /* Bit 0: Chip Select Enable */
#define RA8P_MEMC_CSCR_WT_MASK               (0x70)      /* Bits 4-6: Wait States */
#define RA8P_MEMC_CSCR_WT_SHIFT              (4)

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* SDRAM size */

enum ra8p_sdram_size_e
{
  RA8P_SDRM_SIZE_16MB = 0,        /* 16 MB */
  RA8P_SDRM_SIZE_64MB,            /* 64 MB */
  RA8P_SDRM_SIZE_128MB,           /* 128 MB */
  RA8P_SDRM_SIZE_256MB,           /* 256 MB */
  RA8P_SDRM_SIZE_512MB,           /* 512 MB */
};

/* CAS latency */

enum ra8p_sdram_cas_latency_e
{
  RA8P_SDRM_CAS_2 = 2,              /* CAS Latency 2 */
  RA8P_SDRM_CAS_3,                  /* CAS Latency 3 */
};

/* SDRAM configuration */

struct ra8p_sdram_config_s
{
  uint32_t base;                    /* SDRAM base address */
  enum ra8p_sdram_size_e size;     /* SDRAM size */
  enum ra8p_sdram_cas_latency_e cas; /* CAS latency */
  uint8_t row_addr_width;           /* Row address width (11-13) */
  uint8_t col_addr_width;           /* Column address width (8-11) */
  uint32_t bus_width;              /* Bus width (16 or 32 bits) */
  uint32_t refresh_interval;        /* Refresh interval in clocks */
};

/* MEMC device structure */

struct ra8p_memc_dev_s
{
  uint32_t base;                    /* MEMC base address */
  struct ra8p_sdram_config_s sdram; /* SDRAM configuration */
  bool initialized;                 /* True if initialized */
};

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_MEMC_H */