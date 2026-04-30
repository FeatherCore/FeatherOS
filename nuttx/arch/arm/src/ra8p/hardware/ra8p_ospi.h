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

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* OSPI Register Offsets - Based on RA8P Hardware Manual and Zephyr definitions */
#define RA8P_OSPI_OSPICR_OFFSET          0x0000  /* OSPI Control Register */
#define RA8P_OSPI_OSPISCR_OFFSET         0x0004  /* OSPI Status Control Register */
#define RA8P_OSPI_OSPIMDR_OFFSET         0x0008  /* OSPI Mode Register */
#define RA8P_OSPI_OSPICR2_OFFSET         0x000C  /* OSPI Control Register 2 */
#define RA8P_OSPI_OSPICCR_OFFSET         0x0010  /* OSPI Clock Control Register */
#define RA8P_OSPI_OSPITCR_OFFSET         0x0014  /* OSPI Timing Control Register */
#define RA8P_OSPI_OSPIECR_OFFSET         0x0018  /* OSPI Extended Control Register */
#define RA8P_OSPI_OSPISR_OFFSET          0x001C  /* OSPI Status Register */
#define RA8P_OSPI_OSPISR2_OFFSET         0x0020  /* OSPI Status Register 2 */
#define RA8P_OSPI_OSPIINT_OFFSET         0x0024  /* OSPI Interrupt Register */
#define RA8P_OSPI_OSPICMDSR_OFFSET       0x0028  /* OSPI Command Register */
#define RA8P_OSPI_OSPICMDCR_OFFSET       0x002C  /* OSPI Command Control Register */
#define RA8P_OSPI_OSPIMCR_OFFSET         0x0030  /* OSPI Mode Control Register */
#define RA8P_OSPI_OSPIMCR2_OFFSET        0x0034  /* OSPI Mode Control Register 2 */
#define RA8P_OSPI_OSPIBDCR_OFFSET        0x0038  /* OSPI Buffer Data Control Register */
#define RA8P_OSPI_OSPIBUCR_OFFSET        0x003C  /* OSPI Buffer Update Control Register */
#define RA8P_OSPI_OSPIDAR_OFFSET         0x0040  /* OSPI Data Address Register */
#define RA8P_OSPI_OSPIDDR_OFFSET         0x0044  /* OSPI Data Register */
#define RA8P_OSPI_OSPIAMR_OFFSET         0x0048  /* OSPI Address Mode Register */
#define RA8P_OSPI_OSPIBCR_OFFSET         0x004C  /* OSPI Byte Count Register */
#define RA8P_OSPI_OSPIMBSE_OFFSET        0x0050  /* OSPI Master Bus Select Enable Register */
#define RA8P_OSPI_OSPISSCE_OFFSET        0x0054  /* OSPI Slave Select Control Enable Register */
#define RA8P_OSPI_OSPICDR_OFFSET         0x0058  /* OSPI Clock Division Register */
#define RA8P_OSPI_OSPIDFSR_OFFSET        0x005C  /* OSPI Data Format Setting Register */
#define RA8P_OSPI_OSPIPPR_OFFSET         0x0060  /* OSPI Port Pin Register */
#define RA8P_OSPI_OSPIPPR2_OFFSET        0x0064  /* OSPI Port Pin Register 2 */
#define RA8P_OSPI_OSPIPPR3_OFFSET        0x0068  /* OSPI Port Pin Register 3 */
#define RA8P_OSPI_OSPIDLYR_OFFSET        0x0070  /* OSPI Delay Control Register */
#define RA8P_OSPI_OSPIDLYR2_OFFSET       0x0074  /* OSPI Delay Control Register 2 */
#define RA8P_OSPI_OSPIDLYR3_OFFSET       0x0078  /* OSPI Delay Control Register 3 */
#define RA8P_OSPI_OSPIBFDLY_OFFSET       0x007C  /* OSPI Bus Free Delay Register */
#define RA8P_OSPI_OSPIDQS_OFFSET         0x0080  /* OSPI DQS Control Register */
#define RA8P_OSPI_OSPISSL_OFFSET         0x0084  /* OSPI SSL Register */
#define RA8P_OSPI_OSPISSLIE_OFFSET       0x0088  /* OSPI SSL Initial Enable Register */
#define RA8P_OSPI_OSPISSLF_OFFSET        0x008C  /* OSPI SSL Function Register */
#define RA8P_OSPI_OSPIBSC_OFFSET         0x0090  /* OSPI Bus Space Control Register */
#define RA8P_OSPI_OSPIBSC2_OFFSET        0x0094  /* OSPI Bus Space Control Register 2 */
#define RA8P_OSPI_OSPICNT_OFFSET         0x0098  /* OSPI Count Register */
#define RA8P_OSPI_OSPICNT2_OFFSET        0x009C  /* OSPI Count Register 2 */
#define RA8P_OSPI_OSPIECCSR_OFFSET       0x00A0  /* OSPI ECC Status Register */
#define RA8P_OSPI_OSPIECCCR_OFFSET       0x00A4  /* OSPI ECC Control Register */
#define RA8P_OSPI_OSPIECCMCR_OFFSET      0x00A8  /* OSPI ECC Memory Control Register */
#define RA8P_OSPI_OSPICCS_OFFSET         0x00B0  /* OSPI CCS Register */
#define RA8P_OSPI_OSPICCS2_OFFSET        0x00B4  /* OSPI CCS Register 2 */

/* OSPICR - OSPI Control Register */
#define RA8P_OSPI_OSPICR_SPIMS            (1 << 0)   /* SPI Mode Select */
#define RA8P_OSPI_OSPICR_SSLP             (1 << 1)   /* SSL Polarity */
#define RA8P_OSPI_OSPICR_MOIFV            (1 << 2)   /* MOI Flag Valid */
#define RA8P_OSPI_OSPICR_MOIFE            (1 << 3)   /* MOI Flag Enable */
#define RA8P_OSPI_OSPICR_SCKDL_MASK       (0x03 << 4) /* SCK Delay */
#define RA8P_OSPI_OSPICR_SCKDL_SHIFT      4
#define RA8P_OSPI_OSPICR_PCS_MASK         (0x07 << 6) /* Pin Control Selection */
#define RA8P_OSPI_OSPICR_PCS_SHIFT        6
#define RA8P_OSPI_OSPICR_SSLFV            (1 << 8)   /* SSL Flag Valid */
#define RA8P_OSPI_OSPICR_SSLFE            (1 << 9)   /* SSL Flag Enable */
#define RA8P_OSPI_OSPICR_SPRIE            (1 << 10)  /* SPI Receive Data Full Interrupt Enable */
#define RA8P_OSPI_OSPICR_SPTIE            (1 << 11)  /* SPI Transmit Data Empty Interrupt Enable */
#define RA8P_OSPI_OSPICR_TENDIE           (1 << 12)  /* Transmission End Interrupt Enable */
#define RA8P_OSPI_OSPICR_SPE              (1 << 15)  /* SPI Enable */

/* OSPISCR - OSPI Status Control Register */
#define RA8P_OSPI_OSPISCR_SPIRST          (1 << 0)   /* SPI Reset */
#define RA8P_OSPI_OSPISCR_SSLN_MASK       (0x07 << 1) /* SSL Signal Selection */
#define RA8P_OSPI_OSPISCR_SSLN_SHIFT      1
#define RA8P_OSPI_OSPISCR_SSLF            (1 << 4)   /* SSL Flag */
#define RA8P_OSPI_OSPISCR_SPSSL           (1 << 5)   /* SPI SSL */
#define RA8P_OSPI_OSPISCR_MOIF            (1 << 6)   /* MOI Flag */
#define RA8P_OSPI_OSPISCR_MOIFE           (1 << 7)   /* MOI Flag Enable */

/* OSPIMDR - OSPI Mode Register */
#define RA8P_OSPI_OSPIMDR_SSLKP           (1 << 0)   /* SSL Keep */
#define RA8P_OSPI_OSPIMDR_BRDV_MASK       (0x07 << 1) /* Bit Rate Division */
#define RA8P_OSPI_OSPIMDR_BRDV_SHIFT      1
#define RA8P_OSPI_OSPIMDR_SPB_MASK        (0x0F << 4) /* SPI Bit Count */
#define RA8P_OSPI_OSPIMDR_SPB_SHIFT       4
#define RA8P_OSPI_OSPIMDR_LSBF            (1 << 8)   /* LSB First */
#define RA8P_OSPI_OSPIMDR_SPNDL_MASK      (0x03 << 9) /* SPI Next Delay */
#define RA8P_OSPI_OSPIMDR_SPNDL_SHIFT     9
#define RA8P_OSPI_OSPIMDR_SLNDL_MASK      (0x03 << 11) /* SSL Next Delay */
#define RA8P_OSPI_OSPIMDR_SLNDL_SHIFT     11
#define RA8P_OSPI_OSPIMDR_SCKPL           (1 << 13)  /* SCK Pin Polarity */
#define RA8P_OSPI_OSPIMDR_INV           (1 << 14)  /* Inverse */
#define RA8P_OSPI_OSPIMDR_DME             (1 << 15)  /* Dummy Enable */

/* OSPICCR - OSPI Clock Control Register */
#define RA8P_OSPI_OSPICCR_CKDV_MASK       (0x0F << 0)  /* Clock Division */
#define RA8P_OSPI_OSPICCR_CKDV_SHIFT      0
#define RA8P_OSPI_OSPICCR_MSTR            (1 << 4)   /* Master Mode */
#define RA8P_OSPI_OSPICCR_MOI             (1 << 5)   /* MOI Flag */
#define RA8P_OSPI_OSPICCR_MOIFE           (1 << 6)   /* MOI Flag Enable */

/* OSPITCR - OSPI Timing Control Register */
#define RA8P_OSPI_OSPITCR_SSLAHL_MASK     (0x0F << 0)  /* SSL Assertion Hold Length */
#define RA8P_OSPI_OSPITCR_SSLAHL_SHIFT    0
#define RA8P_OSPI_OSPITCR_SSLDL_MASK      (0x0F << 4)  /* SSL Delay */
#define RA8P_OSPI_OSPITCR_SSLDL_SHIFT     4
#define RA8P_OSPI_OSPITCR_BFDLY_MASK      (0x0F << 8)  /* Bus Free Delay */
#define RA8P_OSPI_OSPITCR_BFDLY_SHIFT     8

/* OSPIECR - OSPI Extended Control Register */
#define RA8P_OSPI_OSPIECR_DME2            (1 << 0)   /* Dummy Enable 2 */
#define RA8P_OSPI_OSPIECR_BSIZE_MASK      (0x07 << 1)  /* Bus Size */
#define RA8P_OSPI_OSPIECR_BSIZE_SHIFT     1
#define RA8P_OSPI_OSPIECR_BSIZE_8BIT      0          /* 8-bit */
#define RA8P_OSPI_OSPIECR_BSIZE_16BIT     1          /* 16-bit */
#define RA8P_OSPI_OSPIECR_BSIZE_32BIT     2          /* 32-bit */
#define RA8P_OSPI_OSPIECR_BSIZE_64BIT     3          /* 64-bit */
#define RA8P_OSPI_OSPIECR_CSHW_MASK       (0x0F << 4)  /* Chip Select High Width */
#define RA8P_OSPI_OSPIECR_CSHW_SHIFT      4
#define RA8P_OSPI_OSPIECR_CSSW_MASK       (0x0F << 8)  /* Chip Select Setup Width */
#define RA8P_OSPI_OSPIECR_CSSW_SHIFT      8
#define RA8P_OSPI_OSPIECR_CSDW_MASK       (0x0F << 12) /* Chip Select Data Width */
#define RA8P_OSPI_OSPIECR_CSDW_SHIFT      12

/* OSPI Base Addresses */
#define RA8P_OSPI0_BASE                   0x4026C000
#define RA8P_OSPI1_BASE                   0x4026D000

/* OSPI Size */
#define RA8P_OSPI_SIZE                    0x1000

/* OSPI Interrupt Numbers */
#define RA8P_IRQ_OSPI0                    102
#define RA8P_IRQ_OSPI1                    103

/* OctaSPI modes */
#define RA8P_OSPI_MODE_SPI               0x00      /* Standard SPI mode */
#define RA8P_OSPI_MODE_OCTAL             0x01      /* Octal mode */
#define RA8P_OSPI_MODE_DUAL              0x02      /* Dual mode */
#define RA8P_OSPI_MODE_QUAD              0x03      /* Quad mode */

/* Data widths */
#define RA8P_OSPI_WIDTH_1                0x00      /* 1 bit */
#define RA8P_OSPI_WIDTH_2                0x01      /* 2 bits */
#define RA8P_OSPI_WIDTH_4                0x02      /* 4 bits */
#define RA8P_OSPI_WIDTH_8                0x03      /* 8 bits */

/* SSL Polarity values */
#define RA8P_OSPI_SSL_LOW                0x00      /* Active low */
#define RA8P_OSPI_SSL_HIGH               0x01      /* Active high */

/* SPI Modes (Clock polarity/phase) */
#define RA8P_OSPI_MODE_0                 0x00      /* CPOL=0, CPHA=0 */
#define RA8P_OSPI_MODE_1                 0x01      /* CPOL=0, CPHA=1 */
#define RA8P_OSPI_MODE_2                 0x02      /* CPOL=1, CPHA=0 */
#define RA8P_OSPI_MODE_3                 0x03      /* CPOL=1, CPHA=1 */

/* OSPI Status Register bits */
#define RA8P_OSPI_OSPISR_SPTEF            (1 << 0)   /* SPI Transmit Buffer Empty Flag */
#define RA8P_OSPI_OSPISR_SPRFF            (1 << 1)   /* SPI Receive Buffer Full Flag */
#define RA8P_OSPI_OSPISR_SPTE             (1 << 2)   /* SPI Transfer Empty */
#define RA8P_OSPI_OSPISR_SPTEND           (1 << 3)   /* SPI Transfer End */
#define RA8P_OSPI_OSPISR_SPSTS            (1 << 4)   /* SPI Status */
#define RA8P_OSPI_OSPISR_SSLF             (1 << 5)   /* SSL Status */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* OSPI configuration structure */
struct ra8p_ospi_config_s
{
  uint8_t channel;                    /* OSPI channel (0 or 1) */
  uint8_t mode;                       /* Operating mode (SPI/Octal/Quad/Dual) */
  uint8_t width;                      /* Data width (1/2/4/8) */
  uint8_t bits;                       /* Data bits per transfer (8/16/32) */
  uint32_t frequency;                 /* Clock frequency in Hz */
  uint8_t spi_mode;                   /* SPI mode (0-3) */
  bool lsb_first;                     /* LSB first (true) or MSB first (false) */
  bool ssl_active_high;               /* SSL active high (true) or low (false) */
  uint8_t ssl_delay;                  /* SSL delay in clock cycles */
  bool dummy_enabled;                 /* Enable dummy cycles */
  uint8_t dummy_cycles;               /* Number of dummy cycles */
};

/* OSPI transfer structure */
struct ra8p_ospi_transfer_s
{
  uint8_t *tx_buffer;                /* Transmit buffer */
  uint8_t *rx_buffer;                /* Receive buffer */
  uint32_t tx_length;                /* Transmit length */
  uint32_t rx_length;                /* Receive length */
  uint8_t ssl;                       /* Slave select (0-7) */
  uint8_t width;                     /* Transfer width */
  bool keep_ssl_active;              /* Keep SSL active after transfer */
};

/* OSPI flash memory configuration */
struct ra8p_ospi_flash_config_s
{
  uint32_t capacity;                  /* Flash capacity in bytes */
  uint32_t sector_size;              /* Sector size in bytes */
  uint32_t page_size;                /* Page size in bytes */
  uint8_t addr_width;                /* Address width (24 or 32 bits) */
  bool quad_enable;                  /* Quad mode enable */
  bool octal_enable;                 /* Octal mode enable */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_ospi_initialize
 *
 * Description:
 *   Initialize the OSPI controller based on Zephyr flash_renesas_ra_ospi_b.c implementation.
 *
 * Input Parameters:
 *   config - Pointer to OSPI configuration structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ospi_initialize(const struct ra8p_ospi_config_s *config);

/****************************************************************************
 * Name: ra8p_ospi_enable
 *
 * Description:
 *   Enable OSPI controller based on Zephyr flash_renesas_ra_ospi_b.c implementation.
 *
 * Input Parameters:
 *   channel - OSPI channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ospi_enable(uint8_t channel);

/****************************************************************************
 * Name: ra8p_ospi_disable
 *
 * Description:
 *   Disable OSPI controller based on Zephyr flash_renesas_ra_ospi_b.c implementation.
 *
 * Input Parameters:
 *   channel - OSPI channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ospi_disable(uint8_t channel);

/****************************************************************************
 * Name: ra8p_ospi_set_frequency
 *
 * Description:
 *   Set OSPI clock frequency based on Zephyr flash_renesas_ra_ospi_b.c implementation.
 *
 * Input Parameters:
 *   channel - OSPI channel (0 or 1)
 *   frequency - Clock frequency in Hz
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ospi_set_frequency(uint8_t channel, uint32_t frequency);

/****************************************************************************
 * Name: ra8p_ospi_set_mode
 *
 * Description:
 *   Set OSPI operating mode (SPI/Octal/Quad/Dual) based on Zephyr flash_renesas_ra_ospi_b.c implementation.
 *
 * Input Parameters:
 *   channel - OSPI channel (0 or 1)
 *   mode - Operating mode
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ospi_set_mode(uint8_t channel, uint8_t mode);

/****************************************************************************
 * Name: ra8p_ospi_transfer
 *
 * Description:
 *   Perform OSPI transfer based on Zephyr flash_renesas_ra_ospi_b.c implementation.
 *
 * Input Parameters:
 *   channel - OSPI channel (0 or 1)
 *   transfer - Pointer to transfer structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ospi_transfer(uint8_t channel, struct ra8p_ospi_transfer_s *transfer);

/****************************************************************************
 * Name: ra8p_ospi_flash_read
 *
 * Description:
 *   Read from OSPI flash memory based on Zephyr flash_renesas_ra_ospi_b.c implementation.
 *
 * Input Parameters:
 *   channel - OSPI channel (0 or 1)
 *   addr - Address to read from
 *   buffer - Buffer to read data into
 *   len - Number of bytes to read
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ospi_flash_read(uint8_t channel, uint32_t addr, uint8_t *buffer, size_t len);

/****************************************************************************
 * Name: ra8p_ospi_flash_write
 *
 * Description:
 *   Write to OSPI flash memory based on Zephyr flash_renesas_ra_ospi_b.c implementation.
 *
 * Input Parameters:
 *   channel - OSPI channel (0 or 1)
 *   addr - Address to write to
 *   buffer - Buffer containing data to write
 *   len - Number of bytes to write
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ospi_flash_write(uint8_t channel, uint32_t addr, const uint8_t *buffer, size_t len);

/****************************************************************************
 * Name: ra8p_ospi_flash_erase_sector
 *
 * Description:
 *   Erase OSPI flash sector based on Zephyr flash_renesas_ra_ospi_b.c implementation.
 *
 * Input Parameters:
 *   channel - OSPI channel (0 or 1)
 *   addr - Address within sector to erase
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ospi_flash_erase_sector(uint8_t channel, uint32_t addr);

/****************************************************************************
 * Name: ra8p_ospi_flash_erase_chip
 *
 * Description:
 *   Erase entire OSPI flash chip based on Zephyr flash_renesas_ra_ospi_b.c implementation.
 *
 * Input Parameters:
 *   channel - OSPI channel (0 or 1)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ospi_flash_erase_chip(uint8_t channel);

/****************************************************************************
 * Name: ra8p_ospi_flash_get_config
 *
 * Description:
 *   Get OSPI flash configuration based on Zephyr flash_renesas_ra_ospi_b.c implementation.
 *
 * Input Parameters:
 *   channel - OSPI channel (0 or 1)
 *   config - Pointer to flash config structure to fill
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ospi_flash_get_config(uint8_t channel, struct ra8p_ospi_flash_config_s *config);

/****************************************************************************
 * Name: ra8p_ospi_flash_quad_enable
 *
 * Description:
 *   Enable quad mode for OSPI flash based on Zephyr flash_renesas_ra_ospi_b.c implementation.
 *
 * Input Parameters:
 *   channel - OSPI channel (0 or 1)
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ospi_flash_quad_enable(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_ospi_flash_octal_enable
 *
 * Description:
 *   Enable octal mode for OSPI flash based on Zephyr flash_renesas_ra_ospi_b.c implementation.
 *
 * Input Parameters:
 *   channel - OSPI channel (0 or 1)
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ospi_flash_octal_enable(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_ospi_wait_ready
 *
 * Description:
 *   Wait for OSPI flash to be ready based on Zephyr flash_renesas_ra_ospi_b.c implementation.
 *
 * Input Parameters:
 *   channel - OSPI channel (0 or 1)
 *   timeout_ms - Timeout in milliseconds
 *
 * Returned Value:
 *   OK on success, negated errno on timeout
 *
 ****************************************************************************/

int ra8p_ospi_wait_ready(uint8_t channel, uint32_t timeout_ms);

/****************************************************************************
 * Name: ra8p_ospi_is_enabled
 *
 * Description:
 *   Check if OSPI controller is enabled.
 *
 * Input Parameters:
 *   channel - OSPI channel (0 or 1)
 *
 * Returned Value:
 *   true if enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_ospi_is_enabled(uint8_t channel);

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_OSPI_H */