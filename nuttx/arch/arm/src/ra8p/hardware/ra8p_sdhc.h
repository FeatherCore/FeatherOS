/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_sdhc.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_SDHC_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_SDHC_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* SDHC Register Offsets - Based on RA8P Hardware Manual and Zephyr definitions */
#define RA8P_SDHC_SD_SYS_ADDR_OFFSET      0x0000  /* System Address Register */
#define RA8P_SDHC_SD_BLK_SIZE_OFFSET      0x0004  /* Block Size Register */
#define RA8P_SDHC_SD_BLK_CNT_OFFSET       0x0006  /* Block Count Register */
#define RA8P_SDHC_SD_ARG_OFFSET           0x0008  /* Argument Register */
#define RA8P_SDHC_SD_XFR_TYP_OFFSET       0x000C  /* Transfer Type Register */
#define RA8P_SDHC_SD_CMD_OFFSET           0x000E  /* Command Register */
#define RA8P_SDHC_SD_RSP_OFFSET(n)        (0x0010 + ((n) * 4))  /* Response Register 0-3 */
#define RA8P_SDHC_SD_BUF_DAT_PORT_OFFSET  0x0020  /* Buffer Data Port Register */
#define RA8P_SDHC_SD_PRSNT_STATE_OFFSET   0x0024  /* Present State Register */
#define RA8P_SDHC_SD_HOST_CTL_OFFSET      0x0028  /* Host Control Register */
#define RA8P_SDHC_SD_PWR_CTL_OFFSET       0x0029  /* Power Control Register */
#define RA8P_SDHC_SD_BLKGAP_CTL_OFFSET    0x002A  /* Block Gap Control Register */
#define RA8P_SDHC_SD_WAKUP_CTL_OFFSET     0x002B  /* Wake-up Control Register */
#define RA8P_SDHC_SD_CLK_CTL_OFFSET       0x002C  /* Clock Control Register */
#define RA8P_SDHC_SD_TIMEOUT_CTL_OFFSET   0x002E  /* Timeout Control Register */
#define RA8P_SDHC_SD_SFT_RST_OFFSET       0x002F  /* Software Reset Register */
#define RA8P_SDHC_SD_INT_STAT_OFFSET      0x0030  /* Interrupt Status Register */
#define RA8P_SDHC_SD_INT_EN_OFFSET        0x0034  /* Interrupt Enable Register */
#define RA8P_SDHC_SD_SIG_EN_OFFSET        0x0038  /* Signal Enable Register */
#define RA8P_SDHC_SD_AC12_ERR_OFFSET      0x003C  /* Auto CMD12 Error Register */
#define RA8P_SDHC_SD_HOST_CTL2_OFFSET     0x003E  /* Host Control 2 Register */
#define RA8P_SDHC_SD_CAP_ADJ_OFFSET       0x0040  /* Capability Adjustment Register */
#define RA8P_SDHC_SD_CAP_ADJ2_OFFSET      0x0044  /* Capability Adjustment Register 2 */
#define RA8P_SDHC_SD_MAX_CUR_CAP_OFFSET   0x0048  /* Maximum Current Capabilities Register */
#define RA8P_SDHC_SD_ADMA_ERR_OFFSET      0x0054  /* ADMA Error Status Register */
#define RA8P_SDHC_SD_ADMA_ADDR_OFFSET     0x0058  /* ADMA Address Register */
#define RA8P_SDHC_SD_PRESET_VAL_OFFSET    0x0060  /* Preset Value Register */
#define RA8P_SDHC_SD_PRESET_VAL2_OFFSET   0x0064  /* Preset Value Register 2 */
#define RA8P_SDHC_SD_SHARED_BUS_CTL_OFFSET 0x00E0  /* Shared Bus Control Register */
#define RA8P_SDHC_SD_SLOT_INT_STAT_OFFSET 0x00FC  /* Slot Interrupt Status Register */
#define RA8P_SDHC_SD_HOST_CTL_VER_OFFSET  0x00FE  /* Host Controller Version Register */

/* SDHC Base Addresses */
#define RA8P_SDHC0_BASE                   0x40252000
#define RA8P_SDHC1_BASE                   0x40252400

/* Maximum number of SDHC channels */
#define RA8P_SDHC_MAX_CHANNELS            2

/* SDHC Block Size */
#define RA8P_SDHC_BLOCK_SIZE              512

/* SDHC timeout in milliseconds */
#define RA8P_SDHC_TIMEOUT_MS              5000
#define RA8P_SDHC_CMD_TIMEOUT_MS          1000

/* PRSNT_STATE register bits */
#define RA8P_SDHC_SD_PRSNT_STATE_CMD_INHIBIT_CMD (1 << 0)   /* Command Inhibit CMD */
#define RA8P_SDHC_SD_PRSNT_STATE_CMD_INHIBIT_DAT (1 << 1)   /* Command Inhibit DAT */
#define RA8P_SDHC_SD_PRSNT_STATE_DAT_LINE_ACTIVE  (1 << 2)   /* DAT Line Active */
#define RA8P_SDHC_SD_PRSNT_STATE_WR_PRTEST_SW     (1 << 19)  /* Write Protect Switch */
#define RA8P_SD_SDHC_PRSNT_STATE_CARD_INSERTED    (1 << 20)  /* Card Inserted */
#define RA8P_SDHC_SD_PRSNT_STATE_CARD_STABLE      (1 << 21)  /* Card Stable */
#define RA8P_SDHC_SD_PRSNT_STATE_CARD_DET_LVL     (1 << 22)  /* Card Detection Level */
#define RA8P_SDHC_SD_PRSNT_STATE_WR_PRTEST_LVL    (1 << 23)  /* Write Protect Level */
#define RA8P_SDHC_SD_PRSNT_STATE_DAT0_LVL         (1 << 24)  /* DAT0 Line Level */
#define RA8P_SDHC_SD_PRSNT_STATE_DAT1_LVL         (1 << 25)  /* DAT1 Line Level */
#define RA8P_SDHC_SD_PRSNT_STATE_DAT2_LVL         (1 << 26)  /* DAT2 Line Level */
#define RA8P_SDHC_SD_PRSNT_STATE_DAT3_LVL         (1 << 27)  /* DAT3 Line Level */
#define RA8P_SDHC_SD_PRSNT_STATE_CMD_LVL          (1 << 28)  /* CMD Line Level */
#define RA8P_SDHC_SD_PRSNT_STATE_CARD_LVL(n)      (1 << (20 + (n))) /* Card Level (0-1) */

/* HOST_CTL register bits */
#define RA8P_SDHC_SD_HOST_CTL_LED_CTL              (1 << 0)   /* LED Control */
#define RA8P_SDHC_SD_HOST_CTL_DTW_4BIT             (1 << 1)   /* 4-bit Mode Select */
#define RA8P_SDHC_SD_HOST_CTL_HISPD                (1 << 2)   /* High Speed Enable */
#define RA8P_SDHC_SD_HOST_CTL_DTW_8BIT             (1 << 5)   /* 8-bit Mode Select */
#define RA8P_SDHC_SD_HOST_CTL_CARD_DET_TL          (1 << 6)   /* Card Detection Test Level */
#define RA8P_SDHC_SD_HOST_CTL_CARD_DET_SG          (1 << 7)   /* Card Detection Signal Selection */

/* PWR_CTL register bits */
#define RA8P_SDHC_SD_PWR_CTL_PWON                  (1 << 0)   /* Power On */
#define RA8P_SDHC_SD_PWR_CTL_VOLT_MASK             (0x0E << 1) /* Supply Voltage */
#define RA8P_SDHC_SD_PWR_CTL_VOLT_180              (0x05 << 1) /* 1.8V */
#define RA8P_SDHC_SD_PWR_CTL_VOLT_300              (0x06 << 1) /* 3.0V */
#define RA8P_SDHC_SD_PWR_CTL_VOLT_330              (0x07 << 1) /* 3.3V */

/* CLK_CTL register bits */
#define RA8P_SDHC_SD_CLK_CTL_INT_CLK_EN            (1 << 0)   /* Internal Clock Enable */
#define RA8P_SDHC_SD_CLK_CTL_INT_CLK_STABLE        (1 << 1)   /* Internal Clock Stable */
#define RA8P_SDHC_SD_CLK_CTL_SD_CLK_EN             (1 << 2)   /* SD Clock Enable */
#define RA8P_SDHC_SD_CLK_CTL_PROG_CLK_MODE         (1 << 5)   /* Programmable Clock Mode */
#define RA8P_SDHC_SD_CLK_CTL_SD_CLK_FREQ_MASK      (0xFF << 8) /* SDCLK Frequency Select */
#define RA8P_SDHC_SD_CLK_CTL_SD_CLK_FREQ_SHIFT     8

/* SFT_RST register bits */
#define RA8P_SDHC_SD_SFT_RST_SWRST_ALL             (1 << 0)   /* Software Reset For All */
#define RA8P_SDHC_SD_SFT_RST_SWRST_CMD_LINE        (1 << 1)   /* Software Reset For CMD Line */
#define RA8P_SDHC_SD_SFT_RST_SWRST_DAT_LINE        (1 << 2)   /* Software Reset For DAT Line */

/* INT_STAT register bits */
#define RA8P_SDHC_SD_INT_STAT_CMD_COMPLETE         (1 << 0)   /* Command Complete */
#define RA8P_SDHC_SD_INT_STAT_XFER_COMPLETE        (1 << 1)   /* Transfer Complete */
#define RA8P_SDHC_SD_INT_STAT_CMD_ERR              (1 << 2)   /* Command Error */
#define RA8P_SDHC_SD_INT_STAT_DATA_TIMEOUT_ERR     (1 << 3)   /* Data Timeout Error */
#define RA8P_SDHC_SD_INT_STAT_DATA_CRC_ERR         (1 << 4)   /* Data CRC Error */
#define RA8P_SDHC_SD_INT_STAT_DATA_END_ERR         (1 << 5)   /* Data End Bit Error */
#define RA8P_SDHC_SD_INT_STAT_CURR_LIM_ERR         (1 << 6)   /* Current Limit Error */
#define RA8P_SDHC_SD_INT_STAT_ACMD_ERR             (1 << 7)   /* Auto CMD Error */
#define RA8P_SDHC_SD_INT_STAT_ADMA_ERR             (1 << 9)   /* ADMA Error */
#define RA8P_SDHC_SD_INT_STAT_CARD_INS             (1 << 16)  /* Card Insertion */
#define RA8P_SDHC_SD_INT_STAT_CARD_REM             (1 << 17)  /* Card Removal */
#define RA8P_SDHC_SD_INT_STAT_CARD_INTERRUPT       (1 << 18)  /* Card Interrupt */
#define RA8P_SDHC_SD_INT_STAT_RETUNE_REQ           (1 << 19)  /* Re-Tuning Request */
#define RA8P_SDHC_SD_INT_STAT_FX_EVENT             (1 << 20)  /* FX Event */
#define RA8P_SDHC_SD_INT_STAT_CQE_EVT              (1 << 21)  /* CQE Event */

/* INT_EN register bits */
#define RA8P_SDHC_SD_INT_EN_CMD_COMPLETE_EN        (1 << 0)   /* Command Complete Interrupt Enable */
#define RA8P_SDHC_SD_INT_EN_XFER_COMPLETE_EN       (1 << 1)   /* Transfer Complete Interrupt Enable */
#define RA8P_SDHC_SD_INT_EN_CMD_ERR_EN             (1 << 2)   /* Command Error Interrupt Enable */
#define RA8P_SDHC_SD_INT_EN_DATA_TIMEOUT_ERR_EN    (1 << 3)   /* Data Timeout Error IE */
#define RA8P_SDHC_SD_INT_EN_DATA_CRC_ERR_EN        (1 << 4)   /* Data CRC Error IE */
#define RA8P_SDHC_SD_INT_EN_DATA_END_ERR_EN        (1 << 5)   /* Data End Bit Error IE */
#define RA8P_SDHC_SD_INT_EN_CURR_LIM_ERR_EN        (1 << 6)   /* Current Limit Error IE */
#define RA8P_SDHC_SD_INT_EN_ACMD_ERR_EN            (1 << 7)   /* Auto CMD Error IE */
#define RA8P_SDHC_SD_INT_EN_CARD_INS_EN            (1 << 16)  /* Card Insertion IE */
#define RA8P_SDHC_SD_INT_EN_CARD_REM_EN            (1 << 17)  /* Card Removal IE */
#define RA8P_SDHC_SD_INT_EN_CARD_INTERRUPT_EN      (1 << 18)  /* Card Interrupt IE */

/* TIMEOUT_CTL register bits */
#define RA8P_SDHC_SD_TIMEOUT_CTL_DTC_VAL_MASK      (0x0F << 0) /* Data Timeout Counter Value */
#define RA8P_SDHC_SD_TIMEOUT_CTL_DTC_VAL_SHIFT     0

/* HOST_CTL2 register bits */
#define RA8P_SDHC_SD_HOST_CTL2_UHS_MASK            (0x07 << 0) /* UHS Mode Select */
#define RA8P_SDHC_SD_HOST_CTL2_UHS_SDR12           (0x00 << 0) /* SDR12 */
#define RA8P_SDHC_SD_HOST_CTL2_UHS_SDR25           (0x01 << 0) /* SDR25 */
#define RA8P_SDHC_SD_HOST_CTL2_UHS_SDR50           (0x02 << 0) /* SDR50 */
#define RA8P_SDHC_SD_HOST_CTL2_UHS_SDR104          (0x03 << 0) /* SDR104 */
#define RA8P_SDHC_SD_HOST_CTL2_UHS_DDR50           (0x04 << 0) /* DDR50 */
#define RA8P_SDHC_SD_HOST_CTL2_1V8_EN              (1 << 3)   /* 1.8V Signaling Enable */
#define RA8P_SDHC_SD_HOST_CTL2_DRV_STR_MASK        (0x03 << 4) /* Driver Strength Select */
#define RA8P_SDHC_SD_HOST_CTL2_DRV_STR_SHIFT       4
#define RA8P_SDHC_SD_HOST_CTL2_DRV_STR_TYPE_B      (0x00 << 4) /* Driver Type B */
#define RA8P_SDHC_SD_HOST_CTL2_DRV_STR_TYPE_A      (0x01 << 4) /* Driver Type A */
#define RA8P_SDHC_SD_HOST_CTL2_DRV_STR_TYPE_C      (0x02 << 4) /* Driver Type C */
#define RA8P_SDHC_SD_HOST_CTL2_DRV_STR_TYPE_D      (0x03 << 4) /* Driver Type D */
#define RA8P_SDHC_SD_HOST_CTL2_EXEC_TUNING         (1 << 6)   /* Execute Tuning */
#define RA8P_SDHC_SD_HOST_CTL2_SAMP_CLK_SEL        (1 << 7)   /* Sampling Clock Select */
#define RA8P_SDHC_SD_HOST_CTL2_ASYNC_INT_EN        (1 << 14)  /* Asynchronous Interrupt Enable */
#define RA8P_SDHC_SD_HOST_CTL2_PRESET_EN           (1 << 15)  /* Preset Value Enable */

/* XFR_TYP register bits */
#define RA8P_SDHC_SD_XFR_TYP_DMA_EN                (1 << 0)   /* DMA Enable */
#define RA8P_SDHC_SD_XFR_TYP_BC_EN                 (1 << 1)   /* Block Count Enable */
#define RA8P_SDHC_SD_XFR_TYP_ACMD_EN_MASK          (0x03 << 2) /* Auto CMD Enable */
#define RA8P_SDHC_SD_XFR_TYP_ACMD_EN_SHIFT         2
#define RA8P_SDHC_SD_XFR_TYP_ACMD_EN_NONE          (0x00 << 2) /* No Auto CMD */
#define RA8P_SDHC_SD_XFR_TYP_ACMD_EN_12            (0x01 << 2) /* Auto CMD12 Enable */
#define RA8P_SDHC_SD_XFR_TYP_ACMD_EN_23            (0x02 << 2) /* Auto CMD23 Enable */
#define RA8P_SDHC_SD_XFR_TYP_ACMD_EN_12_23         (0x03 << 2) /* Auto CMD12 and 23 Enable */
#define RA8P_SDHC_SD_XFR_TYP_RSVD2                 (1 << 4)   /* Reserved */
#define RA8P_SDHC_SD_XFR_TYP_MUL_SIN_BLK_SEL       (1 << 5)   /* Multi/Single Block Select */
#define RA8P_SDHC_SD_XFR_TYP_RD_WR                  (1 << 6)   /* Read/Write Select */
#define RA8P_SDHC_SD_XFR_TYP_DATA_TRNSFR_SEL       (1 << 7)   /* Data Transfer Select */

/* SDMMC command definitions */
#define RA8P_SDMMC_CMD0_GO_IDLE_STATE              0
#define RA8P_SDMMC_CMD1_SEND_OP_COND              1
#define RA8P_SDMMC_CMD2_ALL_SEND_CID              2
#define RA8P_SDMMC_CMD3_SEND_RELATIVE_ADDR        3
#define RA8P_SDMMC_CMD7_SELECT_CARD               7
#define RA8P_SDMMC_CMD8_SEND_IF_COND              8
#define RA8P_SDMMC_CMD9_SEND_CSD                  9
#define RA8P_SDMMC_CMD12_STOP_TRANSMISSION       12
#define RA8P_SDMMC_CMD16_SET_BLOCKLEN            16
#define RA8P_SDMMC_CMD17_READ_SINGLE_BLOCK       17
#define RA8P_SDMMC_CMD18_READ_MULTIPLE_BLOCK     18
#define RA8P_SDMMC_CMD24_WRITE_SINGLE_BLOCK      24
#define RA8P_SDMMC_CMD25_WRITE_MULTIPLE_BLOCK    25
#define RA8P_SDMMC_CMD55_APP_CMD                 55
#define RA8P_SDMMC_ACMD41_SD_SEND_OP_COND        41

/* CMD register response types */
#define RA8P_SDMMC_CMD_RSP_TYPE_MASK               (0x03 << 6)
#define RA8P_SDMMC_CMD_RSP_TYPE_NONE               (0x00 << 6) /* No response */
#define RA8P_SDMMC_CMD_RSP_TYPE_136                (0x01 << 6) /* Response with 136 bits */
#define RA8P_SDMMC_CMD_RSP_TYPE_48                 (0x02 << 6) /* Response with 48 bits */
#define RA8P_SDMMC_CMD_RSP_TYPE_48_CHK             (0x03 << 6) /* Response with 48 bits and check */

/* Interrupt numbers */
#define RA8P_IRQ_SDHC0                           92
#define RA8P_IRQ_SDHC1                           93

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* SDHC card information structure */
struct ra8p_sdhc_cardinfo_s
{
  uint32_t capacity;                    /* Card capacity in bytes */
  uint8_t ocr[4];                       /* OCR register value */
  uint32_t rca;                         /* Relative Card Address */
  uint8_t csd[16];                      /* CSD register */
  uint8_t cid[16];                      /* CID register */
  uint8_t scr[8];                       /* SCR register */
  bool high_capacity;                   /* High capacity card (SDHC/SDXC) */
  bool sdio;                           /* SDIO card */
  bool mmc;                            /* MMC card */
  bool write_protected;                 /* Write protect switch status */
  uint8_t bus_width;                    /* Current bus width */
  uint32_t max_frequency;               /* Maximum supported frequency */
  uint8_t voltage;                      /* Supported voltage */
  uint8_t card_type;                    /* Card type (SD/MMC/SDHC) */
};

/* SDHC transfer configuration structure */
struct ra8p_sdhc_transfer_s
{
  uint32_t block_addr;                 /* Block address */
  uint8_t *buffer;                     /* Data buffer */
  uint32_t nblocks;                    /* Number of blocks */
  bool read;                           /* Read operation */
  bool use_dma;                        /* Use DMA for transfer */
  bool block_count_enable;              /* Block count enable */
  uint8_t bus_width;                   /* Bus width (1, 4, or 8) */
};

/* SDHC initialization structure */
struct ra8p_sdhc_init_s
{
  uint8_t channel;                     /* SDHC channel (0 or 1) */
  uint32_t max_frequency;              /* Maximum SD clock frequency in Hz */
  bool bus_width_4bit;                 /* Enable 4-bit bus width */
  bool bus_width_8bit;                 /* Enable 8-bit bus width */
  bool high_speed;                     /* Enable high-speed mode */
  bool sdio_enabled;                   /* Enable SDIO support */
  bool mmc_enabled;                    /* Enable MMC support */
  uint8_t power_supply_voltage;         /* Power supply voltage */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_sdhc_initialize
 *
 * Description:
 *   Initialize the SDHC controller based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   config - Pointer to initialization structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_initialize(const struct ra8p_sdhc_init_s *config);

/****************************************************************************
 * Name: ra8p_sdhc_card_inserted
 *
 * Description:
 *   Check if SD card is inserted based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   true if card is inserted, false otherwise
 *
 ****************************************************************************/

bool ra8p_sdhc_card_inserted(uint8_t channel);

/****************************************************************************
 * Name: ra8p_sdhc_read_block
 *
 * Description:
 *   Read a block from SD card based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   block_num - Block number to read
 *   buffer - Buffer to read data into
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_read_block(uint8_t channel, uint32_t block_num, uint8_t *buffer);

/****************************************************************************
 * Name: ra8p_sdhc_write_block
 *
 * Description:
 *   Write a block to SD card based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   block_num - Block number to write
 *   buffer - Buffer containing data to write
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_write_block(uint8_t channel, uint32_t block_num, const uint8_t *buffer);

/****************************************************************************
 * Name: ra8p_sdhc_read_blocks
 *
 * Description:
 *   Read multiple blocks from SD card based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   start_block - Starting block number
 *   buffer - Buffer to read data into
 *   nblocks - Number of blocks to read
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_read_blocks(uint8_t channel, uint32_t start_block, 
                         uint8_t *buffer, uint32_t nblocks);

/****************************************************************************
 * Name: ra8p_sdhc_write_blocks
 *
 * Description:
 *   Write multiple blocks to SD card based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   start_block - Starting block number
 *   buffer - Buffer containing data to write
 *   nblocks - Number of blocks to write
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_write_blocks(uint8_t channel, uint32_t start_block, 
                          const uint8_t *buffer, uint32_t nblocks);

/****************************************************************************
 * Name: ra8p_sdhc_set_bus_width
 *
 * Description:
 *   Set SDHC bus width (1, 4, or 8 bit) based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   width   - Bus width (1, 4, or 8)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_set_bus_width(uint8_t channel, uint8_t width);

/****************************************************************************
 * Name: ra8p_sdhc_set_frequency
 *
 * Description:
 *   Set SDHC clock frequency based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   frequency - Clock frequency in Hz
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_set_frequency(uint8_t channel, uint32_t frequency);

/****************************************************************************
 * Name: ra8p_sdhc_get_card_info
 *
 * Description:
 *   Get SD card information based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   info - Pointer to card info structure to fill
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_get_card_info(uint8_t channel, struct ra8p_sdhc_cardinfo_s *info);

/****************************************************************************
 * Name: ra8p_sdhc_enable_high_speed
 *
 * Description:
 *   Enable/disable high-speed mode based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   enable - true to enable high speed, false for normal speed
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_enable_high_speed(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_sdhc_power_up
 *
 * Description:
 *   Power up SDHC controller based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_power_up(uint8_t channel);

/****************************************************************************
 * Name: ra8p_sdhc_power_down
 *
 * Description:
 *   Power down SDHC controller based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_power_down(uint8_t channel);

/****************************************************************************
 * Name: ra8p_sdhc_is_initialized
 *
 * Description:
 *   Check if SDHC is initialized based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   true if initialized, false otherwise
 *
 ****************************************************************************/

bool ra8p_sdhc_is_initialized(uint8_t channel);

/****************************************************************************
 * Name: ra8p_sdhc_is_enabled
 *
 * Description:
 *   Check if SDHC channel is enabled based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   true if enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_sdhc_is_enabled(uint8_t channel);

/****************************************************************************
 * Name: ra8p_sdhc_get_status
 *
 * Description:
 *   Get SDHC status flags based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   Status flags
 *
 ****************************************************************************/

uint32_t ra8p_sdhc_get_status(uint8_t channel);

/****************************************************************************
 * Name: ra8p_sdhc_reset
 *
 * Description:
 *   Reset SDHC controller based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_reset(uint8_t channel);

/****************************************************************************
 * Name: ra8p_sdhc_set_dma
 *
 * Description:
 *   Enable/disable DMA for SDHC transfers based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   enable - true to enable DMA, false to use PIO
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_set_dma(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_sdhc_is_card_write_protected
 *
 * Description:
 *   Check if SD card is write protected based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   true if write protected, false otherwise
 *
 ****************************************************************************/

bool ra8p_sdhc_is_card_write_protected(uint8_t channel);

/****************************************************************************
 * Name: ra8p_sdhc_set_uhs_mode
 *
 * Description:
 *   Set UHS (Ultra High Speed) mode for SDHC based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   mode - UHS mode (0-3)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_set_uhs_mode(uint8_t channel, uint8_t mode);

/****************************************************************************
 * Name: ra8p_sdhc_get_error_flags
 *
 * Description:
 *   Get SDHC error flags based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   Error flags
 *
 ****************************************************************************/

uint32_t ra8p_sdhc_get_error_flags(uint8_t channel);

/****************************************************************************
 * Name: ra8p_sdhc_clear_errors
 *
 * Description:
 *   Clear SDHC error flags based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_clear_errors(uint8_t channel);

/****************************************************************************
 * Name: ra8p_sdhc_send_cmd
 *
 * Description:
 *   Send command to SD card based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   cmd - Command number
 *   arg - Command argument
 *   resp_type - Response type (0=none, 1=136-bit, 2=48-bit, 3=48-bit with check)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_send_cmd(uint8_t channel, uint8_t cmd, uint32_t arg, uint8_t resp_type);

/****************************************************************************
 * Name: ra8p_sdhc_enable_interrupts
 *
 * Description:
 *   Enable/disable SDHC interrupts based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_enable_interrupts(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_sdhc_is_busy
 *
 * Description:
 *   Check if SDHC is busy based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   true if busy, false otherwise
 *
 ****************************************************************************/

bool ra8p_sdhc_is_busy(uint8_t channel);

/****************************************************************************
 * Name: ra8p_sdhc_get_max_frequency
 *
 * Description:
 *   Get maximum supported frequency for card based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   Max frequency in Hz
 *
 ****************************************************************************/

uint32_t ra8p_sdhc_get_max_frequency(uint8_t channel);

/****************************************************************************
 * Name: ra8p_sdhc_get_bus_width
 *
 * Description:
 *   Get current SDHC bus width based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   Bus width (1, 4, or 8)
 *
 ****************************************************************************/

uint8_t ra8p_sdhc_get_bus_width(uint8_t channel);

/****************************************************************************
 * Name: ra8p_sdhc_select_card
 *
 * Description:
 *   Select/deselect SD card based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   rca - Relative card address
 *   select - true to select, false to deselect
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_select_card(uint8_t channel, uint32_t rca, bool select);

/****************************************************************************
 * Name: ra8p_sdhc_get_response
 *
 * Description:
 *   Get last SD command response based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   response - Array to store response (at least 4 uint32_t elements)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_get_response(uint8_t channel, uint32_t *response);

/****************************************************************************
 * Name: ra8p_sdhc_set_timeout
 *
 * Description:
 *   Set SDHC data timeout value based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   timeout - Timeout value (0-15)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_set_timeout(uint8_t channel, uint8_t timeout);

/****************************************************************************
 * Name: ra8p_sdhc_get_transfer_status
 *
 * Description:
 *   Get SDHC transfer status based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   Transfer status
 *
 ****************************************************************************/

uint32_t ra8p_sdhc_get_transfer_status(uint8_t channel);

/****************************************************************************
 * Name: ra8p_sdhc_is_high_capacity
 *
 * Description:
 *   Check if card is SDHC/SDXC high capacity based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *
 * Returned Value:
 *   true if high capacity card, false otherwise
 *
 ****************************************************************************/

bool ra8p_sdhc_is_high_capacity(uint8_t channel);

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_SDHC_H */