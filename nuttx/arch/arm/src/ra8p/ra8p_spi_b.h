/****************************************************************************
 * arch/arm/src/ra8p/ra8p_spi_b.h
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

#ifndef __ARCH_ARM_SRC_RA8P_RA8P_SPI_B_H
#define __ARCH_ARM_SRC_RA8P_RA8P_SPI_B_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <sys/types.h>
#include <stdint.h>
#include <stdbool.h>

#include "hardware/ra8p_spi_b.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* SPI_B base addresses */
#define RA8P_SPI0_BASE                   RA8P_SPI0_BASE
#define RA8P_SPI1_BASE                   RA8P_SPI1_BASE
#define RA8P_SPI2_BASE                   RA8P_SPI2_BASE
#define RA8P_SPI3_BASE                   RA8P_SPI3_BASE

/* SPI_B timeout in milliseconds */
#define RA8P_SPI_B_TIMEOUT_MS            1000

/* Maximum SPI clock divider */
#define RA8P_SPI_B_MAX_DIVIDER           255

/* SPI_B minimum data bits */
#define RA8P_SPI_B_MIN_BITS              4

/* SPI_B maximum data bits */
#define RA8P_SPI_B_MAX_BITS              32

/* Default SPI clock frequency */
#define RA8P_SPI_B_DEFAULT_FREQ          1000000  /* 1 MHz */

/* SPI_B slave select values */
#define RA8P_SPI_B_SSL_0                 0
#define RA8P_SPI_B_SSL_1                 1
#define RA8P_SPI_B_SSL_2                 2
#define RA8P_SPI_B_SSL_3                 3
#define RA8P_SPI_B_SSL_4                 4
#define RA8P_SPI_B_SSL_5                 5
#define RA8P_SPI_B_SSL_6                 6
#define RA8P_SPI_B_SSL_7                 7

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* SPI_B configuration structure */
struct ra8p_spi_b_config_s
{
  uint8_t channel;                   /* SPI_B channel (0-3) */
  uint32_t frequency;                /* SPI clock frequency in Hz */
  uint8_t mode;                      /* SPI mode (0-3) */
  uint8_t bits;                      /* Data bits (4-32) */
  uint8_t ssl;                       /* Slave select (0-7) */
  bool lsb_first;                    /* LSB first if true, MSB first if false */
  bool enable_dma;                   /* Enable DMA for transfers */
  bool use_dtc;                      /* Use DTC (Data Transfer Controller) */
  bool loopback;                     /* Enable loopback mode */
  uint8_t clock_divider;             /* Clock divider value */
  bool bidirectional;                /* Bidirectional mode */
};

/* SPI_B transfer structure */
struct ra8p_spi_b_transfer_s
{
  uint8_t *txbuffer;                /* Transmit buffer */
  uint8_t *rxbuffer;                /* Receive buffer */
  size_t nwords;                    /* Number of words to transfer */
  uint8_t width;                    /* Word width in bytes (1, 2, or 4) */
  bool keep_cs_active;              /* Keep CS active after transfer */
  bool bidirectional;               /* Bidirectional transfer */
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
 *   config - Pointer to configuration structure
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
 *   channel - SPI_B channel (0-3)
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
 *   channel - SPI_B channel (0-3)
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
 *   channel - SPI_B channel (0-3)
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
 *   Set SPI_B mode based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI_B channel (0-3)
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
 *   channel - SPI_B channel (0-3)
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
 *   channel - SPI_B channel (0-3)
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
 *   channel - SPI_B channel (0-3)
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
 *   channel - SPI_B channel (0-3)
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
 *   channel - SPI_B channel (0-3)
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
 *   channel - SPI_B channel (0-3)
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
 *   channel - SPI_B channel (0-3)
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
 *   channel - SPI_B channel (0-3)
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
 *   channel - SPI_B channel (0-3)
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
 *   channel - SPI_B channel (0-3)
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
 *   channel - SPI_B channel (0-3)
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
 *   channel - SPI_B channel (0-3)
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
 *   channel - SPI_B channel (0-3)
 *   enable - true to enable DMA, false to use PIO
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_set_dma(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_spi_b_set_loopback
 *
 * Description:
 *   Enable/disable loopback mode based on Nuttx SPI driver implementation.
 *
 * Input Parameters:
 *   channel - SPI_B channel (0-3)
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_spi_b_set_loopback(uint8_t channel, bool enable);

#endif /* __ARCH_ARM_SRC_RA8P_RA8P_SPI_B_H */