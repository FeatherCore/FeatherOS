/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_sdram.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_SDRAM_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_SDRAM_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* SDRAM Controller Register Offsets - Based on RA8P Hardware Manual and Zephyr definitions */
#define RA8P_SDRAMC_SDMR_OFFSET          0x0000  /* SDRAM Mode Register */
#define RA8P_SDRAMC_SDTR_OFFSET          0x0004  /* SDRAM Timing Register */
#define RA8P_SDRAMC_SDCCR_OFFSET         0x0008  /* SDRAM Control Register */
#define RA8P_SDRAMC_SDCMOD_OFFSET        0x000C  /* SDRAM Command Mode Register */
#define RA8P_SDRAMC_SDCR_OFFSET          0x0010  /* SDRAM Control Register */
#define RA8P_SDRAMC_SDRFCR_OFFSET        0x0014  /* SDRAM Refresh Control Register */
#define RA8P_SDRAMC_SDTMCR_OFFSET        0x0018  /* SDRAM Test Mode Control Register */
#define RA8P_SDRAMC_SDMODR_OFFSET        0x001C  /* SDRAM Mode Register */
#define RA8P_SDRAMC_SDADR_OFFSET         0x0020  /* SDRAM Address Register */
#define RA8P_SDRAMC_SDSTMR_OFFSET        0x0024  /* SDRAM Status Monitor Register */
#define RA8P_SDRAMC_SDCR2_OFFSET         0x0028  /* SDRAM Control Register 2 */
#define RA8P_SDRAMC_SDTR2_OFFSET         0x002C  /* SDRAM Timing Register 2 */
#define RA8P_SDRAMC_SDMR2_OFFSET         0x0030  /* SDRAM Mode Register 2 */
#define RA8P_SDRAMC_SDRFCR2_OFFSET       0x0034  /* SDRAM Refresh Control Register 2 */
#define RA8P_SDRAMC_SDMISR_OFFSET        0x0038  /* SDRAM MISR Register */

/* SDCCR - SDRAM Control Register */
#define RA8P_SDRAMC_SDCCR_CACT           (1 << 0)   /* Column Address Width */
#define RA8P_SDRAMC_SDCCR_ADLT_MASK      (0x07 << 1) /* Address Latency */
#define RA8P_SDRAMC_SDCCR_ADLT_SHIFT     1
#define RA8P_SDRAMC_SDCCR_RDLT_MASK      (0x07 << 4) /* Read Latency */
#define RA8P_SDRAMC_SDCCR_RDLT_SHIFT     4
#define RA8P_SDRAMC_SDCCR_WDLT_MASK      (0x07 << 8) /* Write Latency */
#define RA8P_SDRAMC_SDCCR_WDLT_SHIFT     8
#define RA8P_SDRAMC_SDCCR_CLKSTP         (1 << 12)  /* Clock Stop */
#define RA8P_SDRAMC_SDCCR_POWOFF         (1 << 13)  /* Power Off */

/* SDCR - SDRAM Control Register */
#define RA8P_SDRAMC_SDCR_MRS             (1 << 0)   /* Mode Register Set */
#define RA8P_SDRAMC_SDCR_MRST            (1 << 1)   /* Mode Register Set Terminate */
#define RA8P_SDRAMC_SDCR_RFSH            (1 << 2)   /* Refresh */
#define RA8P_SDRAMC_SDCR_RFSHT           (1 << 3)   /* Refresh Terminate */
#define RA8P_SDRAMC_SDCR_SMRST           (1 << 4)   /* Self Refresh Start */
#define RA8P_SDRAMC_SDCR_SMEND           (1 << 5)   /* Self Refresh End */
#define RA8P_SDRAMC_SDCR_INI             (1 << 6)   /* Initialize */
#define RA8P_SDRAMC_SDCR_INIRR           (1 << 7)   /* Initialization Repeat */
#define RA8P_SDRAMC_SDCR_INIS            (1 << 8)   /* Initialization Status */

/* SDCMOD - SDRAM Command Mode Register */
#define RA8P_SDRAMC_SDCMOD_CMOD_MASK     (0x07 << 0) /* Command Mode */
#define RA8P_SDRAMC_SDCMOD_CMOD_NOP      0           /* NOP */
#define RA8P_SDRAMC_SDCMOD_CMOD_ACTIVE   1           /* Active */
#define RA8P_SDRAMC_SDCMOD_CMOD_READ     2           /* Read */
#define RA8P_SDRAMC_SDCMOD_CMOD_WRITE    3           /* Write */
#define RA8P_SDRAMC_SDCMOD_CMOD_PRECHG   4           /* Precharge */
#define RA8P_SDRAMC_SDCMOD_CMOD_AUTO_RFSH 5          /* Auto Refresh */
#define RA8P_SDRAMC_SDCMOD_CMOD_LOAD_MODE 6          /* Load Mode Register */
#define RA8P_SDRAMC_SDCMOD_CMOD_SELF_RFSH 7          /* Self Refresh */

/* SDTR - SDRAM Timing Register */
#define RA8P_SDRAMC_SDTR_TRWL_MASK       (0x0F << 0)  /* Write Latency */
#define RA8P_SDRAMC_SDTR_TRWL_SHIFT      0
#define RA8P_SDRAMC_SDTR_TRCD_MASK       (0x0F << 4)  /* RAS to CAS Delay */
#define RA8P_SDRAMC_SDTR_TRCD_SHIFT      4
#define RA8P_SDRAMC_SDTR_TRP_MASK        (0x0F << 8)  /* Precharge Command Period */
#define RA8P_SDRAMC_SDTR_TRP_SHIFT       8
#define RA8P_SDRAMC_SDTR_TRAS_MASK       (0x1F << 12) /* Row Active to Precharge Period */
#define RA8P_SDRAMC_SDTR_TRAS_SHIFT      12
#define RA8P_SDRAMC_SDTR_TMRD_MASK       (0x0F << 17) /* Load Mode Register to Active Command Period */
#define RA8P_SDRAMC_SDTR_TMRD_SHIFT      17
#define RA8P_SDRAMC_SDTR_TCL_MASK        (0x0F << 21) /* CAS Latency */
#define RA8P_SDRAMC_SDTR_TCL_SHIFT       21
#define RA8P_SDRAMC_SDTR_TWR_MASK        (0x0F << 25) /* Write Recovery Time */
#define RA8P_SDRAMC_SDTR_TWR_SHIFT       25

/* SDRFCR - SDRAM Refresh Control Register */
#define RA8P_SDRAMC_SDRFCR_RFC_MASK      (0xFF << 0)  /* Refresh Cycle */
#define RA8P_SDRAMC_SDRFCR_RFC_SHIFT     0
#define RA8P_SDRAMC_SDRFCR_RFC_DIV256    0x08       /* Refresh every 256 cycles */
#define RA8P_SDRAMC_SDRFCR_RFC_DIV128    0x04       /* Refresh every 128 cycles */
#define RA8P_SDRAMC_SDRFCR_RFC_DIV64     0x02       /* Refresh every 64 cycles */
#define RA8P_SDRAMC_SDRFCR_RFC_DIV32     0x01       /* Refresh every 32 cycles */
#define RA8P_SDRAMC_SDRFCR_RFSHST        (1 << 8)   /* Refresh Status */
#define RA8P_SDRAMC_SDRFCR_RFSHE         (1 << 9)   /* Refresh Enable */
#define RA8P_SDRAMC_SDRFCR_REIE          (1 << 10)  /* Refresh Error Interrupt Enable */

/* SDRAMC Base Address */
#define RA8P_SDRAMC_BASE                 0x40003C00
#define RA8P_SDRAMC_SIZE                 0x54

/* SDRAM Base Address */
#define RA8P_SDRAM_BASE                  0x68000000
#define RA8P_SDRAM_SIZE                  0x04000000  /* 64MB */

/* SDRAM Memory Types */
#define RA8P_SDRAM_TYPE_SDR              0x00      /* SDR SDRAM */
#define RA8P_SDRAM_TYPE_DDR              0x01      /* DDR SDRAM */
#define RA8P_SDRAM_TYPE_DDR2             0x02      /* DDR2 SDRAM */
#define RA8P_SDRAM_TYPE_DDR3             0x03      /* DDR3 SDRAM */

/* SDRAM Bank Configurations */
#define RA8P_SDRAM_BANKS_2               0x01      /* 2 banks */
#define RA8P_SDRAM_BANKS_4               0x02      /* 4 banks */

/* SDRAM Row/Column Sizes */
#define RA8P_SDRAM_ROWS_11               0x00      /* 11-bit row address */
#define RA8P_SDRAM_ROWS_12               0x01      /* 12-bit row address */
#define RA8P_SDRAM_ROWS_13               0x02      /* 13-bit row address */
#define RA8P_SDRAM_COLS_8                0x00      /* 8-bit column address */
#define RA8P_SDRAM_COLS_9                0x01      /* 9-bit column address */
#define RA8P_SDRAM_COLS_10               0x02      /* 10-bit column address */
#define RA8P_SDRAM_COLS_11               0x03      /* 11-bit column address */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* SDRAM configuration structure */
struct ra8p_sdram_config_s
{
  uint8_t mem_type;                   /* Memory type (SDR, DDR, etc.) */
  uint8_t bank_count;                 /* Number of banks */
  uint8_t row_bits;                   /* Number of row address bits */
  uint8_t col_bits;                   /* Number of column address bits */
  uint32_t data_width;                /* Data width in bits (8, 16, 32) */
  uint32_t total_size;                /* Total memory size in bytes */
  uint32_t refresh_cycle;             /* Refresh cycle timing */
  uint8_t cas_latency;               /* CAS latency */
  uint8_t write_recovery;            /* Write recovery time */
  uint8_t trcd;                      /* RAS to CAS delay */
  uint8_t trp;                       /* Precharge command period */
  uint8_t tras;                      /* Row active to precharge period */
  uint8_t tcl;                       /* CAS latency timing */
  uint8_t twr;                       /* Write recovery time */
  uint8_t tmrd;                      /* Load mode register to active command timing */
};

/* SDRAM timing parameters structure */
struct ra8p_sdram_timing_s
{
  uint8_t tcl;                       /* CAS Latency */
  uint8_t tcwl;                      /* CAS Write Latency */
  uint8_t trcd;                      /* RAS to CAS Delay */
  uint8_t trp;                       /* Row Precharge Delay */
  uint8_t tras;                      /* Row Active to Precharge Delay */
  uint8_t trfc;                      /* Row Refresh Cycle */
  uint8_t txsr;                      /* Exit Self Refresh */
  uint8_t tmrd;                      /* Load Mode Register */
  uint32_t refresh_interval_ns;      /* Refresh interval in nanoseconds */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_sdram_initialize
 *
 * Description:
 *   Initialize the SDRAM controller based on Zephyr memc_renesas_ra_sdram.c implementation.
 *
 * Input Parameters:
 *   config - Pointer to SDRAM configuration structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdram_initialize(const struct ra8p_sdram_config_s *config);

/****************************************************************************
 * Name: ra8p_sdram_enable
 *
 * Description:
 *   Enable SDRAM controller.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdram_enable(void);

/****************************************************************************
 * Name: ra8p_sdram_disable
 *
 * Description:
 *   Disable SDRAM controller.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdram_disable(void);

/****************************************************************************
 * Name: ra8p_sdram_set_timing
 *
 * Description:
 *   Set SDRAM timing parameters based on Zephyr memc_renesas_ra_sdram.c implementation.
 *
 * Input Parameters:
 *   timing - Pointer to timing parameters structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdram_set_timing(const struct ra8p_sdram_timing_s *timing);

/****************************************************************************
 * Name: ra8p_sdram_refresh
 *
 * Description:
 *   Manually refresh SDRAM based on Zephyr memc_renesas_ra_sdram.c implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdram_refresh(void);

/****************************************************************************
 * Name: ra8p_sdram_enter_self_refresh
 *
 * Description:
 *   Enter SDRAM self-refresh mode.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdram_enter_self_refresh(void);

/****************************************************************************
 * Name: ra8p_sdram_exit_self_refresh
 *
 * Description:
 *   Exit SDRAM self-refresh mode.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdram_exit_self_refresh(void);

/****************************************************************************
 * Name: ra8p_sdram_is_active
 *
 * Description:
 *   Check if SDRAM is active based on Zephyr memc_renesas_ra_sdram.c implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if SDRAM is active, false otherwise
 *
 ****************************************************************************/

bool ra8p_sdram_is_active(void);

/****************************************************************************
 * Name: ra8p_sdram_is_refresh_enabled
 *
 * Description:
 *   Check if SDRAM refresh is enabled.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if refresh is enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_sdram_is_refresh_enabled(void);

/****************************************************************************
 * Name: ra8p_sdram_set_refresh_interval
 *
 * Description:
 *   Set SDRAM refresh interval based on Zephyr memc_renesas_ra_sdram.c implementation.
 *
 * Input Parameters:
 *   ns - Refresh interval in nanoseconds
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdram_set_refresh_interval(uint32_t ns);

/****************************************************************************
 * Name: ra8p_sdram_set_cas_latency
 *
 * Description:
 *   Set SDRAM CAS latency based on Zephyr memc_renesas_ra_sdram.c implementation.
 *
 * Input Parameters:
 *   latency - CAS latency value (2, 3)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdram_set_cas_latency(uint8_t latency);

/****************************************************************************
 * Name: ra8p_sdram_set_power_mode
 *
 * Description:
 *   Set SDRAM power mode based on Zephyr memc_renesas_ra_sdram.c implementation.
 *
 * Input Parameters:
 *   power_down - true for power-down mode, false for normal
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdram_set_power_mode(bool power_down);

/****************************************************************************
 * Name: ra8p_sdram_get_memory_map
 *
 * Description:
 *   Get SDRAM memory mapping information.
 *
 * Input Parameters:
 *   base - Pointer to store base address
 *   size - Pointer to store size
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdram_get_memory_map(uint32_t *base, uint32_t *size);

/****************************************************************************
 * Name: ra8p_sdram_get_config
 *
 * Description:
 *   Get current SDRAM configuration based on Zephyr memc_renesas_ra_sdram.c implementation.
 *
 * Input Parameters:
 *   config - Pointer to config structure to fill
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdram_get_config(struct ra8p_sdram_config_s *config);

/****************************************************************************
 * Name: ra8p_sdram_reset
 *
 * Description:
 *   Reset SDRAM controller.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdram_reset(void);

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_SDRAM_H */