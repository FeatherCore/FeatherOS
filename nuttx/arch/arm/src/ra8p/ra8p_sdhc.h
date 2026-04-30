/****************************************************************************
 * arch/arm/src/ra8p/ra8p_sdhc.h
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

#ifndef __ARCH_ARM_SRC_RA8P_RA8P_SDHC_H
#define __ARCH_ARM_SRC_RA8P_RA8P_SDHC_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <sys/types.h>
#include <stdint.h>
#include <stdbool.h>
#include <time.h>

#include "hardware/ra8p_sdhc.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* SDHC timeout in milliseconds */
#define RA8P_SDHC_TIMEOUT_MS             5000
#define RA8P_SDHC_CMD_TIMEOUT_MS         1000

/* SDHC block size (always 512 bytes for SD) */
#define RA8P_SDHC_BLOCK_SIZE             512

/* Maximum number of SDHC channels */
#define RA8P_SDHC_MAX_CHANNELS           2

/* SDHC maximum number of blocks per transfer */
#define RA8P_SDHC_MAX_TRANSFER_BLOCKS    128

/* Default card detection timeout */
#define RA8P_SDHC_CARD_DETECT_TIMEOUT_MS 100

/* SDHC clock divider values */
#define RA8P_SDHC_CLOCK_DIV_1            0
#define RA8P_SDHC_CLOCK_DIV_2            1
#define RA8P_SDHC_CLOCK_DIV_4            2
#define RA8P_SDHC_CLOCK_DIV_8            3
#define RA8P_SDHC_CLOCK_DIV_16           4
#define RA8P_SDHC_CLOCK_DIV_32           5
#define RA8P_SDHC_CLOCK_DIV_64           6
#define RA8P_SDHC_CLOCK_DIV_128          7
#define RA8P_SDHC_CLOCK_DIV_256          8
#define RA8P_SDHC_CLOCK_DIV_512          9

/* SDHC bus width values */
#define RA8P_SDHC_BUS_WIDTH_1BIT         1
#define RA8P_SDHC_BUS_WIDTH_4BIT         4
#define RA8P_SDHC_BUS_WIDTH_8BIT         8

/* SDHC speed modes */
#define RA8P_SDHC_SPEED_MODE_NORMAL      0
#define RA8P_SDHC_SPEED_MODE_HIGH        1
#define RA8P_SDHC_SPEED_MODE_SDR12       2
#define RA8P_SDHC_SPEED_MODE_SDR25       3
#define RA8P_SDHC_SPEED_MODE_SDR50       4
#define RA8P_SDHC_SPEED_MODE_SDR104      5
#define RA8P_SDHC_SPEED_MODE_DDR50       6

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* SDHC configuration structure */
struct ra8p_sdhc_config_s
{
  uint8_t channel;                   /* SDHC channel (0 or 1) */
  uint32_t max_frequency;            /* Maximum SD clock frequency in Hz */
  bool bus_width_4bit;               /* Enable 4-bit bus width */
  bool bus_width_8bit;               /* Enable 8-bit bus width */
  bool high_speed;                   /* Enable high-speed mode */
  bool sdio_enabled;                 /* Enable SDIO support */
  bool mmc_enabled;                  /* Enable MMC support */
  uint8_t power_supply_voltage;      /* Power supply voltage */
};

/* SDHC card information structure */
struct ra8p_sdhc_cardinfo_s
{
  uint32_t capacity;                 /* Card capacity in bytes */
  uint32_t block_count;              /* Number of blocks */
  uint8_t block_size;                /* Block size in bytes (usually 512) */
  bool high_capacity;                /* High capacity card (SDHC/SDXC) */
  bool sdio;                         /* SDIO card */
  bool mmc;                          /* MMC card */
  bool wp_switch;                    /* Write protect switch */
  bool card_detected;                /* Card detected flag */
  uint8_t speed_class;               /* Speed class */
  uint8_t bus_width;                 /* Current bus width */
};

/* SDHC transfer structure */
struct ra8p_sdhc_transfer_s
{
  uint32_t block_addr;               /* Block address */
  uint8_t *buffer;                   /* Data buffer */
  uint32_t nblocks;                  /* Number of blocks to transfer */
  bool read;                         /* true=read, false=write */
  bool use_dma;                      /* Use DMA for transfer */
  bool multi_block;                  /* Multi-block transfer */
  bool block_count_enable;           /* Enable block count */
};

/* SDHC interrupt callback */
typedef void (*ra8p_sdhc_callback_t)(uint8_t channel, void *arg);

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
 *   config - Pointer to configuration structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_sdhc_initialize(const struct ra8p_sdhc_config_s *config);

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
 *   Set SDHC bus width based on Nuttx SDIO driver implementation.
 *
 * Input Parameters:
 *   channel - SDHC channel (0 or 1)
 *   width - Bus width (1, 4, or 8)
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

#endif /* __ARCH_ARM_SRC_RA8P_RA8P_SDHC_H */