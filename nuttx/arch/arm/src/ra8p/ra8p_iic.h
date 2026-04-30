/****************************************************************************
 * arch/arm/src/ra8p/ra8p_iic.h
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

#ifndef __ARCH_ARM_SRC_RA8P_RA8P_IIC_H
#define __ARCH_ARM_SRC_RA8P_RA8P_IIC_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <sys/types.h>
#include <stdint.h>
#include <stdbool.h>

#include "hardware/ra8p_iic.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* IIC base addresses */
#define RA8P_IIC0_BASE                   RA8P_IIC0_BASE
#define RA8P_IIC1_BASE                   RA8P_IIC1_BASE
#define RA8P_IIC2_BASE                   RA8P_IIC2_BASE

/* IIC timeout in milliseconds */
#define RA8P_IIC_TIMEOUT_MS              1000

/* Maximum IIC channels */
#define RA8P_IIC_MAX_CHANNELS            3

/* IIC clock speeds */
#define RA8P_IIC_STANDARD_SPEED          100000   /* 100 kHz */
#define RA8P_IIC_FAST_SPEED             400000   /* 400 kHz */
#define RA8P_IIC_FAST_PLUS_SPEED        1000000  /* 1 MHz */
#define RA8P_IIC_ULTRA_FAST_SPEED       3400000  /* 3.4 MHz */

/* Default IIC clock speed */
#define RA8P_IIC_DEFAULT_SPEED          100000   /* 100 kHz */

/* Maximum data length per transfer */
#define RA8P_IIC_MAX_TRANSFER_SIZE       255

/* IIC addressing modes */
#define RA8P_IIC_7BIT_ADDR               0
#define RA8P_IIC_10BIT_ADDR              1

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* IIC configuration structure */
struct ra8p_iic_config_s
{
  uint8_t channel;                   /* IIC channel (0-2) */
  uint32_t frequency;                /* I2C clock frequency in Hz */
  uint8_t addr_mode;                 /* Addressing mode (7-bit or 10-bit) */
  bool master_mode;                  /* Master mode if true, slave if false */
  bool enable_dma;                   /* Enable DMA for transfers */
  bool enable_interrupts;            /* Enable interrupts */
  uint8_t scl_pullup;               /* SCL pull-up (0=none, 1=internal, 2=external) */
  uint8_t sda_pullup;               /* SDA pull-up (0=none, 1=internal, 2=external) */
  bool noise_filter_enable;         /* Enable noise filter */
  uint8_t noise_filter_stage;       /* Noise filter stages (0-3) */
  bool timeout_enable;              /* Enable timeout detection */
  uint32_t timeout_cycles;          /* Timeout cycles */
};

/* IIC transfer message structure */
struct ra8p_iic_msg_s
{
  uint16_t addr;                     /* 7-bit or 10-bit slave address */
  uint8_t *buffer;                   /* Data buffer */
  uint32_t length;                   /* Number of bytes to transfer */
  uint32_t flags;                    /* Transfer flags */
};

/* IIC transfer flags */
#define RA8P_IIC_M_READ                 (1 << 0)  /* Read transfer */
#define RA8P_IIC_M_TEN                  (1 << 1)  /* Send START before transfer */
#define RA8P_IIC_M_RESTART              (1 << 2)  /* Send RESTART instead of STOP */
#define RA8P_IIC_M_STOP                 (1 << 3)  /* Send STOP after transfer */
#define RA8P_IIC_M_NOSTART              (1 << 4)  /* Don't send START before transfer */

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_iic_initialize
 *
 * Description:
 *   Initialize the IIC controller based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   config - Pointer to configuration structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_initialize(const struct ra8p_iic_config_s *config);

/****************************************************************************
 * Name: ra8p_iic_start
 *
 * Description:
 *   Start IIC controller operation based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_start(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_stop
 *
 * Description:
 *   Stop IIC controller operation based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_stop(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_transfer
 *
 * Description:
 *   Perform an I2C transfer based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *   msgs - Array of I2C messages
 *   count - Number of messages
 *
 * Returned Value:
 *   Number of messages processed on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_transfer(uint8_t channel, struct ra8p_iic_msg_s *msgs, int count);

/****************************************************************************
 * Name: ra8p_iic_set_frequency
 *
 * Description:
 *   Set IIC clock frequency based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *   frequency - Clock frequency in Hz
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_set_frequency(uint8_t channel, uint32_t frequency);

/****************************************************************************
 * Name: ra8p_iic_set_addr_mode
 *
 * Description:
 *   Set IIC addressing mode based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *   mode - Addressing mode (7-bit or 10-bit)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_set_addr_mode(uint8_t channel, uint8_t mode);

/****************************************************************************
 * Name: ra8p_iic_is_initialized
 *
 * Description:
 *   Check if IIC controller is initialized based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   true if initialized, false otherwise
 *
 ****************************************************************************/

bool ra8p_iic_is_initialized(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_is_enabled
 *
 * Description:
 *   Check if IIC controller is enabled based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   true if enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_iic_is_enabled(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_get_status
 *
 * Description:
 *   Get IIC controller status flags based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   Status flags
 *
 ****************************************************************************/

uint32_t ra8p_iic_get_status(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_enable_interrupts
 *
 * Description:
 *   Enable IIC interrupts based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_enable_interrupts(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_iic_set_dma
 *
 * Description:
 *   Enable/disable DMA for IIC transfers based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *   enable - true to enable DMA, false to use PIO
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_set_dma(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_iic_reset
 *
 * Description:
 *   Reset IIC controller based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_reset(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_clear_status
 *
 * Description:
 *   Clear IIC status flags based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *   flags - Flags to clear
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_clear_status(uint8_t channel, uint32_t flags);

/****************************************************************************
 * Name: ra8p_iic_get_error_flags
 *
 * Description:
 *   Get IIC error flags based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   Error flags
 *
 ****************************************************************************/

uint32_t ra8p_iic_get_error_flags(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_clear_errors
 *
 * Description:
 *   Clear IIC error flags based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_clear_errors(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_enable_timeout
 *
 * Description:
 *   Enable/disable IIC timeout detection based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *   enable - true to enable timeout, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_enable_timeout(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_iic_is_busy
 *
 * Description:
 *   Check if IIC bus is busy based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - IIC channel (0-2)
 *
 * Returned Value:
 *   true if busy, false otherwise
 *
 ****************************************************************************/

bool ra8p_iic_is_busy(uint8_t channel);

#endif /* __ARCH_ARM_SRC_RA8P_RA8P_IIC_H */