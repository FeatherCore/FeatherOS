/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_spi_b.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_SPI_B_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_SPI_B_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* SPI_B Register Offsets - Based on RA8P Hardware Manual and Zephyr definitions */
#define RA8P_SPI_B_SPSR_OFFSET           0x0000  /* SPI Status Register */
#define RA8P_SPI_B_SPDR_OFFSET           0x0004  /* SPI Data Register */
#define RA8P_SPI_B_SPCR_OFFSET           0x0008  /* SPI Control Register */
#define RA8P_SPI_B_SPCR2_OFFSET          0x000C  /* SPI Control Register 2 */
#define RA8P_SPI_B_SSLP_OFFSET           0x0010  /* SSL Polarity Register */
#define RA8P_SPI_B_SPPR_OFFSET           0x0014  /* SPI Pin Control Register */
#define RA8P_SPI_B_SPBR_OFFSET           0x0018  /* SPI Bit Rate Register */
#define RA8P_SPI_B_SPDCR_OFFSET          0x001C  /* SPI Data Control Register */
#define RA8P_SPI_B_SPCKD_OFFSET          0x0020  /* SPI Clock Delay Register */
#define RA8P_SPI_B_SSLND_OFFSET          0x0024  /* SSL Negation Delay Register */
#define RA8P_SPI_B_SPND_OFFSET           0x0028  /* SPI Next-Access Delay Register */
#define RA8P_SPI_B_SPTD_OFFSET           0x0030  /* SPI Transfer Delay Register */
#define RA8P_SPI_B_SPBFC_OFFSET          0x0034  /* SPI Buffer Counter Register */
#define RA8P_SPI_B_SPCMD_OFFSET(n)       (0x0040 + ((n) * 4))  /* SPI Command Register n */
#define RA8P_SPI_B_SPBF_OFFSET           0x0060  /* SPI Buffer Flag Register */
#define RA8P_SPI_B_SPFC_OFFSET           0x0064  /* SPI Frame Counter Register */
#define RA8P_SPI_B_SPIIAR_OFFSET         0x0070  /* SPI Internal Interrupt Address Register */
#define RA8P_SPI_B_SFBER_OFFSET          0x0074  /* SPI Frequency Division Error Register */
#define RA8P_SPI_B_SPBDC_OFFSET          0x0078  /* SPI Buffer Counter Clear Register */
#define RA8P_SPI_B_SPBST_OFFSET          0x007C  /* SPI Buffer Status Register */
#define RA8P_SPI_B_SPBCR_OFFSET          0x0080  /* SPI Buffer Counter Register 2 */
#define RA8P_SPI_B_SPBFC2_OFFSET         0x0084  /* SPI Buffer Counter Register 2 */
#define RA8P_SPI_B_SPBR2_OFFSET          0x0088  /* SPI Bit Rate Register 2 */
#define RA8P_SPI_B_SPCSR_OFFSET          0x008C  /* SPI Command Status Register */
#define RA8P_SPI_B_SPCST_OFFSET          0x0090  /* SPI Command Status Register 2 */
#define RA8P_SPI_B_SPBCC_OFFSET          0x0094  /* SPI Buffer Counter Clear Register 2 */
#define RA8P_SPI_B_SPBCS_OFFSET          0x0098  /* SPI Buffer Counter Status Register */
#define RA8P_SPI_B_SPBCD_OFFSET          0x009C  /* SPI Buffer Counter Difference Register */
#define RA8P_SPI_B_SPCMD0_OFFSET         0x0040  /* SPI Command Register 0 */
#define RA8P_SPI_B_SPCMD1_OFFSET         0x0044  /* SPI Command Register 1 */
#define RA8P_SPI_B_SPCMD2_OFFSET         0x0048  /* SPI Command Register 2 */
#define RA8P_SPI_B_SPCMD3_OFFSET         0x004C  /* SPI Command Register 3 */
#define RA8P_SPI_B_SPCMD4_OFFSET         0x0050  /* SPI Command Register 4 */
#define RA8P_SPI_B_SPCMD5_OFFSET         0x0054  /* SPI Command Register 5 */
#define RA8P_SPI_B_SPCMD6_OFFSET         0x0058  /* SPI Command Register 6 */
#define RA8P_SPI_B_SPCMD7_OFFSET         0x005C  /* SPI Command Register 7 */

/* SPI Base Addresses */
#define RA8P_SPI0_BASE                   0x4035C000
#define RA8P_SPI1_BASE                   0x4035C100
#define RA8P_SPI2_BASE                   0x4035C200
#define RA8P_SPI3_BASE                   0x4035C300

/* SPSR - SPI Status Register */
#define RA8P_SPI_B_SPSR_SPTEF            (1 << 0)   /* SPI Transmit Buffer Empty Flag */
#define RA8P_SPI_B_SPSR_SPTEND           (1 << 1)   /* SPI Transfer End Flag */
#define RA8P_SPI_B_SPSR_SPRF             (1 << 2)   /* SPI Receive Buffer Full Flag */
#define RA8P_SPI_B_SPSR_RDRF             (1 << 3)   /* Receive Data Ready Flag */
#define RA8P_SPI_B_SPSR_PERF             (1 << 4)   /* Parity Error Flag */
#define RA8P_SPI_B_SPSR_OVRF             (1 << 5)   /* Overflow Error Flag */
#define RA8P_SPI_B_SPSR_MODF             (1 << 6)   /* Mode Fault Error Flag */
#define RA8P_SPI_B_SPSR_IDLNF            (1 << 7)   /* Idle Noise Flag */
#define RA8P_SPI_B_SPSR_DCMF             (1 << 8)   /* Data Collision Error Flag */

/* SPCR - SPI Control Register */
#define RA8P_SPI_B_SPCR_MSTR             (1 << 0)   /* Master/Slave Select */
#define RA8P_SPI_B_SPCR_SPIE             (1 << 1)   /* SPI Operation Enable */
#define RA8P_SPI_B_SPCR_PSE              (1 << 2)   /* Pin Select */
#define RA8P_SPI_B_SPCR_TXMD             (1 << 3)   /* SPI Transmit Mode Select */
#define RA8P_SPI_B_SPCR_SPIRST           (1 << 4)   /* SPI Reset */
#define RA8P_SPI_B_SPCR_SPSSL            (1 << 5)   /* SPI SSL */
#define RA8P_SPI_B_SPCR_INV              (1 << 6)   /* Inverse */
#define RA8P_SPI_B_SPCR_DSS_MASK         (0x1F << 7) /* Data Size Select */
#define RA8P_SPI_B_SPCR_DSS_SHIFT        7

/* SPCR2 - SPI Control Register 2 */
#define RA8P_SPI_B_SPCR2_SPMS            (1 << 0)   /* SPI Multi Slave Control */
#define RA8P_SPI_B_SPCR2_SCKASE           (1 << 1)   /* SPI SCK Automatic Stop Enable */
#define RA8P_SPI_B_SPCR2_SLNDVE           (1 << 2)   /* SPI SSL Negation Delay Violation Detection Enable */
#define RA8P_SPI_B_SPCR2_SPNDVE           (1 << 3)   /* SPI Next-access Delay Violation Detection Enable */
#define RA8P_SPI_B_SPCR2_RXDMDS           (1 << 4)   /* SPI Receive Data Multiplexed Enable */
#define RA8P_SPI_B_SPCR2_DCOMSE           (1 << 5)   /* Data Collision Detection Enable */

/* SPDCR - SPI Data Control Register */
#define RA8P_SPI_B_SPDCR_SPBYT            (1 << 0)   /* SPI Byte Access Enable */
#define RA8P_SPI_B_SPDCR_SPIIE            (1 << 1)   /* SPI Idle Interrupt Enable */
#define RA8P_SPI_B_SPDCR_SPDIIE           (1 << 2)   /* SPI Data Interrupt Enable */
#define RA8P_SPI_B_SPDCR_SPOAE            (1 << 3)   /* SPI Overcurrent Detection A Enable */
#define RA8P_SPI_B_SPDCR_SPOBE            (1 << 4)   /* SPI Overcurrent Detection B Enable */

/* SPCMD - SPI Command Register */
#define RA8P_SPI_B_SPCMD_SCKDEN           (1 << 0)   /* SCK Delay Enable */
#define RA8P_SPI_B_SPCMD_SLNDEN           (1 << 1)   /* SSL Negation Delay Enable */
#define RA8P_SPI_B_SPCMD_SPNDEN           (1 << 2)   /* SPI Next-access Delay Enable */
#define RA8P_SPI_B_SPCMD_LSBF             (1 << 3)   /* LSB First */
#define RA8P_SPI_B_SPCMD_SPB_MASK         (0x1F << 4) /* SPI Data Size */
#define RA8P_SPI_B_SPCMD_SPB_SHIFT        4
#define RA8P_SPI_B_SPCMD_SPISSL_MASK      (0x03 << 8) /* SPI SSL Signal */
#define RA8P_SPI_B_SPCMD_SPISSL_SHIFT     8
#define RA8P_SPI_B_SPCMD_SPISSL_0         0x00       /* SSL0 */
#define RA8P_SPI_B_SPCMD_SPISSL_1         0x01       /* SSL1 */
#define RA8P_SPI_B_SPCMD_SPISSL_2         0x02       /* SSL2 */
#define RA8P_SPI_B_SPCMD_SPISSL_3         0x03       /* SSL3 */
#define RA8P_SPI_B_SPCMD_SSLKP            (1 << 10)  /* SSL Keep */
#define RA8P_SPI_B_SPCMD_BRDV_MASK        (0x03 << 12) /* Bit Rate Division */
#define RA8P_SPI_B_SPCMD_BRDV_SHIFT       12
#define RA8P_SPI_B_SPCMD_BRDV_DIV1        0x00       /* Divide by 1 */
#define RA8P_SPI_B_SPCMD_BRDV_DIV2        0x01       /* Divide by 2 */
#define RA8P_SPI_B_SPCMD_BRDV_DIV4        0x02       /* Divide by 4 */
#define RA8P_SPI_B_SPCMD_BRDV_DIV8        0x03       /* Divide by 8 */

/* SPBR - SPI Bit Rate Register */
#define RA8P_SPI_B_SPBR_SPBR_MASK         (0xFF << 0) /* SPI Bit Rate */
#define RA8P_SPI_B_SPBR_SPBR_SHIFT        0

/* Maximum number of SPI channels */
#define RA8P_SPI_MAX_CHANNELS             4

/* Maximum command registers */
#define RA8P_SPI_MAX_CMD_REGS             8

/* Maximum number of slaves per SPI controller */
#define RA8P_SPI_MAX_SLAVES               4

/* Default SPI clock frequency */
#define RA8P_SPI_DEFAULT_FREQ             1000000  /* 1 MHz */

/* SPI modes */
#define RA8P_SPI_MODE_0                   0        /* CPOL=0, CPHA=0 */
#define RA8P_SPI_MODE_1                   1        /* CPOL=0, CPHA=1 */
#define RA8P_SPI_MODE_2                   2        /* CPOL=1, CPHA=0 */
#define RA8P_SPI_MODE_3                   3        /* CPOL=1, CPHA=1 */

/* SPI bit rates */
#define RA8P_SPI_B_BITRATE_DIV1           0
#define RA8P_SPI_B_BITRATE_DIV2           1
#define RA8P_SPI_B_BITRATE_DIV4           2
#define RA8P_SPI_B_BITRATE_DIV8           3

/* SPI data sizes */
#define RA8P_SPI_B_DSS_4BIT               (4 - 1)  /* 4-bit */
#define RA8P_SPI_B_DSS_5BIT               (5 - 1)  /* 5-bit */
#define RA8P_SPI_B_DSS_6BIT               (6 - 1)  /* 6-bit */
#define RA8P_SPI_B_DSS_7BIT               (7 - 1)  /* 7-bit */
#define RA8P_SPI_B_DSS_8BIT               (8 - 1)  /* 8-bit */
#define RA8P_SPI_B_DSS_16BIT              (16 - 1) /* 16-bit */
#define RA8P_SPI_B_DSS_32BIT              (32 - 1) /* 32-bit */

/* SPI timeout in milliseconds */
#define RA8P_SPI_TIMEOUT_MS               1000

/* SPI interrupt numbers */
#define RA8P_IRQ_SPI0                     68
#define RA8P_IRQ_SPI1                     69
#define RA8P_IRQ_SPI2                     70
#define RA8P_IRQ_SPI3                     71

/* Maximum SPI transfer size */
#define RA8P_SPI_MAX_TRANSFER_SIZE        4096

/* SPI clock divider settings */
#define RA8P_SPI_CLK_DIV_MIN              1
#define RA8P_SPI_CLK_DIV_MAX              255

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* SPI_B configuration structure */
struct ra8p_spi_b_config_s
{
  uint8_t channel;                  /* SPI channel (0-3) */
  uint32_t frequency;               /* SPI clock frequency in Hz */
  uint8_t mode;                     /* SPI mode (0-3) */
  uint8_t bits;                     /* Number of bits per transfer (4-32) */
  uint8_t ssl;                      /* Slave select pin (0-3) */
  bool lsb_first;                   /* LSB first if true, MSB first if false */
  bool enable_dma;                  /* Enable DMA for transfers */
  bool bidirectional;               /* Bidirectional mode */
  bool multi_slave;                 /* Multi-slave mode */
  bool keep_ssl_active;             /* Keep SSL active after transfer */
  uint8_t clock_divider;            /* Clock divider */
  bool byte_access;                 /* Byte access mode */
  bool idle_interrupt;              /* Idle interrupt enable */
};

/* SPI_B transfer structure */
struct ra8p_spi_b_transfer_s
{
  uint8_t *txbuffer;               /* Transmit buffer */
  uint8_t *rxbuffer;               /* Receive buffer */
  size_t nwords;                   /* Number of words to transfer */
  uint8_t width;                   /* Word width in bytes (1, 2, or 4) */
  bool keep_cs_active;             /* Keep CS active after transfer */
  bool bidirectional;              /* Bidirectional transfer */
  bool blocking;                   /* Blocking vs non-blocking */
  unsigned int timeout;            /* Timeout in ms */
};

/* SPI_B slave configuration structure */
struct ra8p_spi_b_slave_s
{
  uint8_t ssl_pin;                  /* Slave select pin */
  uint32_t frequency;               /* Clock frequency for this slave */
  uint8_t mode;                     /* SPI mode */
  uint8_t bits;                     /* Data bits */
  bool active_high;                 /* Active high or low */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_spi_b_initialize
 *
 * Description:
 *   Initialize the SPI_B controller based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   config - Pointer to SPI configuration structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_initialize(const struct ra8p_spi_b_config_s *config);

/****************************************************************************
 * Name: ra8p_spi_b_enable
 *
 * Description:
 *   Enable the SPI_B controller based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_enable(uint8_t channel);

/****************************************************************************
 * Name: ra8p_spi_b_disable
 *
 * Description:
 *   Disable the SPI_B controller based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_disable(uint8_t channel);

/****************************************************************************
 * Name: ra8p_spi_b_set_frequency
 *
 * Description:
 *   Set SPI_B clock frequency based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   frequency - Clock frequency in Hz
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_set_frequency(uint8_t channel, uint32_t frequency);

/****************************************************************************
 * Name: ra8p_spi_b_set_mode
 *
 * Description:
 *   Set SPI_B mode (0-3) based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   mode - SPI mode (0-3)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_set_mode(uint8_t channel, uint8_t mode);

/****************************************************************************
 * Name: ra8p_spi_b_set_bits
 *
 * Description:
 *   Set SPI_B data bits per transfer based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   nbits - Number of bits per transfer (4-32)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_set_bits(uint8_t channel, uint8_t nbits);

/****************************************************************************
 * Name: ra8p_spi_b_exchange
 *
 * Description:
 *   Exchange data on SPI_B bus based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   txbuffer - Transmit buffer
 *   rxbuffer - Receive buffer
 *   nwords - Number of words to exchange
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_exchange(uint8_t channel, const void *txbuffer, void *rxbuffer, size_t nwords);

/****************************************************************************
 * Name: ra8p_spi_b_send
 *
 * Description:
 *   Send data on SPI_B bus based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   buffer - Data buffer to send
 *   nwords - Number of words to send
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_send(uint8_t channel, const void *buffer, size_t nwords);

/****************************************************************************
 * Name: ra8p_spi_b_receive
 *
 * Description:
 *   Receive data from SPI_B bus based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   buffer - Buffer to receive data into
 *   nwords - Number of words to receive
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_receive(uint8_t channel, void *buffer, size_t nwords);

/****************************************************************************
 * Name: ra8p_spi_b_lock
 *
 * Description:
 *   Lock SPI_B for exclusive access based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   lock - true to lock, false to unlock
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_lock(uint8_t channel, bool lock);

/****************************************************************************
 * Name: ra8p_spi_b_select
 *
 * Description:
 *   Select/deselect SPI slave based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   devid - Device ID (used for slave select)
 *   selected - true to select, false to deselect
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_select(uint8_t channel, uint32_t devid, bool selected);

/****************************************************************************
 * Name: ra8p_spi_b_set_databits
 *
 * Description:
 *   Set SPI_B data bits (alias for ra8p_spi_b_set_bits) based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   nbits - Number of bits per transfer (4-32)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_set_databits(uint8_t channel, uint8_t nbits);

/****************************************************************************
 * Name: ra8p_spi_b_set_msbfirst
 *
 * Description:
 *   Set SPI_B bit order to MSB first based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   msbfirst - true for MSB first, false for LSB first
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_set_msbfirst(uint8_t channel, bool msbfirst);

/****************************************************************************
 * Name: ra8p_spi_b_get_status
 *
 * Description:
 *   Get SPI_B status flags based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *
 * Returned Value:
 *   Status flags
 *
 ****************************************************************************/

uint32_t ra8p_spi_b_get_status(uint8_t channel);

/****************************************************************************
 * Name: ra8p_spi_b_is_initialized
 *
 * Description:
 *   Check if SPI_B is initialized based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *
 * Returned Value:
 *   true if initialized, false otherwise
 *
 ****************************************************************************/

bool ra8p_spi_b_is_initialized(uint8_t channel);

/****************************************************************************
 * Name: ra8p_spi_b_is_enabled
 *
 * Description:
 *   Check if SPI_B is enabled based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *
 * Returned Value:
 *   true if enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_spi_b_is_enabled(uint8_t channel);

/****************************************************************************
 * Name: ra8p_spi_b_reset
 *
 * Description:
 *   Reset SPI_B controller based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_reset(uint8_t channel);

/****************************************************************************
 * Name: ra8p_spi_b_set_dma
 *
 * Description:
 *   Enable/disable DMA for SPI_B transfers based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   enable - true to enable DMA, false to use PIO
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_set_dma(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_spi_b_configure_command
 *
 * Description:
 *   Configure SPI command register based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   cmd_idx - Command register index (0-7)
 *   ssl - Slave select (0-3)
 *   bits - Data bits (4-32)
 *   mode - SPI mode
 *   lsb_first - LSB first flag
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_configure_command(uint8_t channel, uint8_t cmd_idx, uint8_t ssl, 
                                uint8_t bits, uint8_t mode, bool lsb_first);

/****************************************************************************
 * Name: ra8p_spi_b_set_lsbfirst
 *
 * Description:
 *   Set SPI_B bit order to LSB first based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   lsbfirst - true for LSB first, false for MSB first
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_set_lsbfirst(uint8_t channel, bool lsbfirst);

/****************************************************************************
 * Name: ra8p_spi_b_set_clock_divider
 *
 * Description:
 *   Set SPI_B clock divider based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   divider - Clock divider value (1-255)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_set_clock_divider(uint8_t channel, uint8_t divider);

/****************************************************************************
 * Name: ra8p_spi_b_get_frequency
 *
 * Description:
 *   Get current SPI_B frequency based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *
 * Returned Value:
 *   Current frequency in Hz, or 0 if error
 *
 ****************************************************************************/

uint32_t ra8p_spi_b_get_frequency(uint8_t channel);

/****************************************************************************
 * Name: ra8p_spi_b_set_ssl_polarity
 *
 * Description:
 *   Set SPI_B SSL polarity based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   ssl - SSL pin (0-3)
 *   active_high - true for active high, false for active low
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_set_ssl_polarity(uint8_t channel, uint8_t ssl, bool active_high);

/****************************************************************************
 * Name: ra8p_spi_b_set_ssl_keep
 *
 * Description:
 *   Set SPI_B SSL keep active after transfer based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   ssl - SSL pin (0-3)
 *   keep - true to keep active, false for normal
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_set_ssl_keep(uint8_t channel, uint8_t ssl, bool keep);

/****************************************************************************
 * Name: ra8p_spi_b_wait_ready
 *
 * Description:
 *   Wait for SPI_B to be ready for transfer based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   timeout - Timeout in milliseconds
 *
 * Returned Value:
 *   OK on success, negated errno on timeout
 *
 ****************************************************************************/

int ra8p_spi_b_wait_ready(uint8_t channel, unsigned int timeout);

/****************************************************************************
 * Name: ra8p_spi_b_is_slave_mode
 *
 * Description:
 *   Check if SPI_B is in slave mode based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *
 * Returned Value:
 *   true if in slave mode, false if in master mode
 *
 ****************************************************************************/

bool ra8p_spi_b_is_slave_mode(uint8_t channel);

/****************************************************************************
 * Name: ra8p_spi_b_set_slave_mode
 *
 * Description:
 *   Set SPI_B to slave mode based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   enable - true to enable slave mode, false for master
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_set_slave_mode(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_spi_b_set_bidirectional
 *
 * Description:
 *   Set bidirectional mode for SPI_B based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   enable - true for bidirectional mode, false for standard
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_set_bidirectional(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_spi_b_get_transfer_count
 *
 * Description:
 *   Get number of bytes transferred on SPI_B based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *
 * Returned Value:
 *   Number of bytes transferred
 *
 ****************************************************************************/

uint32_t ra8p_spi_b_get_transfer_count(uint8_t channel);

/****************************************************************************
 * Name: ra8p_spi_b_clear_status
 *
 * Description:
 *   Clear SPI_B status flags based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   flags - Flags to clear
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_clear_status(uint8_t channel, uint32_t flags);

/****************************************************************************
 * Name: ra8p_spi_b_enable_interrupts
 *
 * Description:
 *   Enable/disable SPI_B interrupts based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   enable - true to enable interrupts, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_enable_interrupts(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_spi_b_set_slave_config
 *
 * Description:
 *   Configure a SPI slave device based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   slave_config - Pointer to slave configuration structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_set_slave_config(uint8_t channel, const struct ra8p_spi_b_slave_s *slave_config);

/****************************************************************************
 * Name: ra8p_spi_b_get_clock_divider
 *
 * Description:
 *   Get current SPI_B clock divider based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *
 * Returned Value:
 *   Current clock divider, or 0 if error
 *
 ****************************************************************************/

uint8_t ra8p_spi_b_get_clock_divider(uint8_t channel);

/****************************************************************************
 * Name: ra8p_spi_b_set_multi_slave
 *
 * Description:
 *   Enable/disable multi-slave mode based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *   enable - true to enable multi-slave, false for single slave
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_set_multi_slave(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_spi_b_get_error_flags
 *
 * Description:
 *   Get SPI_B error flags based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *
 * Returned Value:
 *   Error flags
 *
 ****************************************************************************/

uint32_t ra8p_spi_b_get_error_flags(uint8_t channel);

/****************************************************************************
 * Name: ra8p_spi_b_clear_errors
 *
 * Description:
 *   Clear SPI_B error flags based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI channel (0-3)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_clear_errors(uint8_t channel);

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_SPI_B_H */