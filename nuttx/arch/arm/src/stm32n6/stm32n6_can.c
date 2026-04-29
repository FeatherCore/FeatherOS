/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_can.c
 *
 * SPDX-License-Identifier: Apache-2.0
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

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <sys/types.h>
#include <stdint.h>
#include <stdbool.h>
#include <errno.h>

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_memorymap.h"
#include "hardware/stm32n6_rcc.h"
#include "stm32n6_can.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define CAN_TIMEOUT_US             1000000  /* 1 second timeout */

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static inline void stm32n6_can_putreg(uintptr_t canbase,
                                      uint32_t offset, uint32_t value)
{
  putreg32(value, canbase + offset);
}

static inline uint32_t stm32n6_can_getreg(uintptr_t canbase,
                                          uint32_t offset)
{
  return getreg32(canbase + offset);
}

static void stm32n6_can_enable_clock(uintptr_t canbase)
{
  uint32_t regval;

  regval = getreg32(STM32_RCC_APB1ENR);
  if (canbase == STM32_FDCAN1_BASE)
    {
      regval |= (1 << 8);  /* FDCAN1EN */
    }
  else if (canbase == STM32_FDCAN2_BASE)
    {
      regval |= (1 << 9);  /* FDCAN2EN */
    }
  else if (canbase == STM32_FDCAN3_BASE)
    {
      regval |= (1 << 10); /* FDCAN3EN */
    }
  putreg32(regval, STM32_RCC_APB1ENR);
}

static int stm32n6_can_wait_sync(uintptr_t canbase, uint32_t mask)
{
  uint32_t regval;
  uint32_t timeout = CAN_TIMEOUT_US;

  do
    {
      regval = stm32n6_can_getreg(canbase, FDCAN_CCCR_OFFSET);
      if (!(regval & mask))
        {
          return OK;
        }
      up_udelay(1);
      timeout--;
    }
  while (timeout > 0);

  return -ETIMEDOUT;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int stm32n6_can_initialize(uintptr_t canbase, uint32_t bitrate, uint32_t dbitrate)
{
  uint32_t regval;
  uint32_t nbtp_reg = 0;
  uint32_t dbtp_reg = 0;

  stm32n6_can_enable_clock(canbase);

  /* Enter initialization mode */
  regval = stm32n6_can_getreg(canbase, FDCAN_CCCR_OFFSET);
  regval |= FDCAN_CCCR_INIT;
  stm32n6_can_putreg(canbase, FDCAN_CCCR_OFFSET, regval);

  /* Wait for acknowledge */
  if (stm32n6_can_wait_sync(canbase, FDCAN_CCCR_INIT) != OK)
    {
      return -ETIMEDOUT;
    }

  /* Enable Configuration Change Enable */
  regval |= FDCAN_CCCR_CCE;
  stm32n6_can_putreg(canbase, FDCAN_CCCR_OFFSET, regval);

  /* Wait for CCE acknowledge */
  if (stm32n6_can_wait_sync(canbase, FDCAN_CCCR_CCE) != OK)
    {
      return -ETIMEDOUT;
    }

  /* Calculate NBTP register value (Nominal Bit Timing and Prescaler) */
  /* Using default values for demonstration purposes */
  nbtp_reg = (15 << FDCAN_NBTP_NBRP_SHIFT) |    /* Baud rate prescaler */
             (3 << FDCAN_NBTP_NSJW_SHIFT) |     /* (Re)Sync Jump Width */
             (14 << FDCAN_NBTP_NTSEG1_SHIFT) |  /* Time segment 1 */
             (4 << FDCAN_NBTP_NTSEG2_SHIFT);    /* Time segment 2 */

  stm32n6_can_putreg(canbase, FDCAN_NBTP_OFFSET, nbtp_reg);

  /* Calculate DBTP register value (Data Bit Timing and Prescaler) */
  /* Used for CAN FD at data phase */
  if (dbitrate > 0)
    {
      dbtp_reg = (4 << FDCAN_DBTP_DBRP_SHIFT) |    /* Data baud rate prescaler */
                 (1 << FDCAN_DBTP_DSJW_SHIFT) |    /* Data (Re)Sync Jump Width */
                 (6 << FDCAN_DBTP_DTSEG1_SHIFT) |  /* Data time segment 1 */
                 (3 << FDCAN_DBTP_DTSEG2_SHIFT);   /* Data time segment 2 */

      stm32n6_can_putreg(canbase, FDCAN_DBTP_OFFSET, dbtp_reg);

      /* Enable CAN FD mode */
      regval = stm32n6_can_getreg(canbase, FDCAN_CCCR_OFFSET);
      regval |= FDCAN_CCCR_FDOE;  /* Enable CAN FD operation */
      regval |= FDCAN_CCCR_BSE;   /* Bit rate switching */
      stm32n6_can_putreg(canbase, FDCAN_CCCR_OFFSET, regval);
    }

  /* Configure TX Buffer/FIFO/Queue elements */
  stm32n6_can_putreg(canbase, FDCAN_TXBC_OFFSET, 0x00000000);  /* No Tx buffers yet */

  /* Configure Rx FIFO 0 elements */
  stm32n6_can_putreg(canbase, FDCAN_RXF0C_OFFSET, 0x00000000); /* No Rx FIFO 0 yet */

  /* Configure Rx FIFO 1 elements */
  stm32n6_can_putreg(canbase, FDCAN_RXF1C_OFFSET, 0x00000000); /* No Rx FIFO 1 yet */

  /* Exit initialization mode */
  regval = stm32n6_can_getreg(canbase, FDCAN_CCCR_OFFSET);
  regval &= ~FDCAN_CCCR_INIT;
  stm32n6_can_putreg(canbase, FDCAN_CCCR_OFFSET, regval);

  /* Wait for acknowledge */
  if (stm32n6_can_wait_sync(canbase, FDCAN_CCCR_INIT) == OK)
    {
      return -ETIMEDOUT;
    }

  return OK;
}

void stm32n6_can_enable(uintptr_t canbase)
{
  uint32_t regval;

  regval = stm32n6_can_getreg(canbase, FDCAN_CCCR_OFFSET);
  regval &= ~FDCAN_CCCR_INIT;
  stm32n6_can_putreg(canbase, FDCAN_CCCR_OFFSET, regval);

  /* Wait for acknowledge */
  while ((stm32n6_can_getreg(canbase, FDCAN_CCCR_OFFSET) & FDCAN_CCCR_INIT) != 0);
}

void stm32n6_can_disable(uintptr_t canbase)
{
  uint32_t regval;

  regval = stm32n6_can_getreg(canbase, FDCAN_CCCR_OFFSET);
  regval |= FDCAN_CCCR_INIT;
  stm32n6_can_putreg(canbase, FDCAN_CCCR_OFFSET, regval);

  /* Wait for acknowledge */
  while ((stm32n6_can_getreg(canbase, FDCAN_CCCR_OFFSET) & FDCAN_CCCR_INIT) == 0);
}

int stm32n6_can_transmit(uintptr_t canbase, uint32_t id, bool extended, 
                         bool fd, bool brs, uint8_t *data, uint8_t dlc)
{
  uint32_t txbc_reg;
  uint32_t txbtie_reg;
  uint32_t timeout = CAN_TIMEOUT_US;
  uint32_t msg_word0, msg_word1;
  uint32_t i;
  uint32_t *data_ptr;

  /* Check if transmission queue is full */
  uint32_t txqs_reg = stm32n6_can_getreg(canbase, FDCAN_TXFQS_OFFSET);
  if ((txqs_reg & 0x1F) == 0)  /* No free Tx element */
    {
      return -EBUSY;
    }

  /* Prepare the message header */
  msg_word0 = (id & 0x1FFFFFFF);  /* Standard/Extended ID */
  if (extended)
    {
      msg_word0 |= (1 << 29);  /* XTD bit - Extended ID */
    }
  else
    {
      msg_word0 &= ~(1 << 29);  /* STD ID */
    }

  if (fd)
    {
      msg_word0 |= (1 << 30);  /* FDF bit - CAN FD frame */
      if (brs)
        {
          msg_word0 |= (1 << 31);  /* BRS bit - Bit Rate Switch */
        }
    }
  else
    {
      msg_word0 &= ~((1 << 30) | (1 << 31));  /* Classic CAN frame */
    }

  msg_word1 = (dlc & 0xF);  /* Data Length Code */

  /* Write message header */
  uint32_t tx_buffer_base = 0x00000000; /* This would be calculated based on TXFQS */
  putreg32(msg_word0, canbase + 0x00000000 + tx_buffer_base);
  putreg32(msg_word1, canbase + 0x00000004 + tx_buffer_base);

  /* Write data */
  data_ptr = (uint32_t *)(canbase + 0x00000008 + tx_buffer_base);
  for (i = 0; i < (dlc + 3) / 4; i++)
    {
      if (i * 4 + 3 < dlc)
        {
          *data_ptr = ((uint32_t)data[i*4+3] << 24) |
                      ((uint32_t)data[i*4+2] << 16) |
                      ((uint32_t)data[i*4+1] << 8)  |
                      (uint32_t)data[i*4];
        }
      else if (i * 4 + 2 < dlc)
        {
          *data_ptr = ((uint32_t)data[i*4+2] << 16) |
                      ((uint32_t)data[i*4+1] << 8)  |
                      (uint32_t)data[i*4];
        }
      else if (i * 4 + 1 < dlc)
        {
          *data_ptr = ((uint32_t)data[i*4+1] << 8) |
                      (uint32_t)data[i*4];
        }
      else if (i * 4 < dlc)
        {
          *data_ptr = (uint32_t)data[i*4];
        }
      else
        {
          *data_ptr = 0;
        }
      data_ptr++;
    }

  /* Request transmission */
  uint32_t tx_queue_elem = (txqs_reg >> 16) & 0x1F;  /* Get first available index */
  uint32_t txbc_reg_new = (1 << tx_queue_elem);      /* Request transmission of element */
  stm32n6_can_putreg(canbase, FDCAN_TXBC_OFFSET, txbc_reg_new);

  /* Wait for transmission completion */
  while (timeout--)
    {
      uint32_t ir_reg = stm32n6_can_getreg(canbase, FDCAN_IR_OFFSET);
      if (ir_reg & FDCAN_IR_TC)  /* Transmission Completed */
        {
          /* Clear interrupt flag */
          stm32n6_can_putreg(canbase, FDCAN_IR_OFFSET, FDCAN_IR_TC);
          return OK;
        }
      up_udelay(1);
    }

  return -ETIMEDOUT;
}

int stm32n6_can_receive(uintptr_t canbase, uint32_t *id, bool *extended,
                        uint8_t *data, uint8_t *dlc)
{
  uint32_t rxf0s_reg = stm32n6_can_getreg(canbase, FDCAN_RXF0S_OFFSET);
  
  if ((rxf0s_reg & 0x7F) == 0)  /* No new messages in FIFO 0 */
    {
      return -EAGAIN;
    }

  /* Get the next message from FIFO 0 */
  uint32_t rx_fifo0_base = 0x00000000; /* This would be calculated based on RAM configuration */
  uint32_t msg_word0 = getreg32(canbase + 0x00000000 + rx_fifo0_base);
  uint32_t msg_word1 = getreg32(canbase + 0x00000004 + rx_fifo0_base);

  /* Extract ID */
  *id = msg_word0 & 0x1FFFFFFF;
  *extended = (msg_word0 & (1 << 29)) != 0;

  /* Extract DLC */
  *dlc = msg_word1 & 0xF;

  /* Extract data */
  uint32_t *data_ptr = (uint32_t *)(canbase + 0x00000008 + rx_fifo0_base);
  uint32_t data_word;
  for (int i = 0; i < (*dlc + 3) / 4; i++)
    {
      data_word = *data_ptr++;
      data[i*4] = data_word & 0xFF;
      if (i*4+1 < *dlc) data[i*4+1] = (data_word >> 8) & 0xFF;
      if (i*4+2 < *dlc) data[i*4+2] = (data_word >> 16) & 0xFF;
      if (i*4+3 < *dlc) data[i*4+3] = (data_word >> 24) & 0xFF;
    }

  /* Release FIFO element */
  uint32_t rx_fifo0_get_idx = (rxf0s_reg >> 8) & 0x3F;
  uint32_t new_get_idx = (rx_fifo0_get_idx + 1) & 0x3F;
  stm32n6_can_putreg(canbase, FDCAN_RXF0A_OFFSET, new_get_idx);

  return OK;
}

bool stm32n6_can_available(uintptr_t canbase)
{
  uint32_t rxf0s_reg = stm32n6_can_getreg(canbase, FDCAN_RXF0S_OFFSET);
  return (rxf0s_reg & 0x7F) != 0;  /* Messages pending in FIFO 0 */
}

void stm32n6_can_reset_error_counters(uintptr_t canbase)
{
  /* Write to Error Counter Register to reset error counters */
  stm32n6_can_putreg(canbase, FDCAN_ECR_OFFSET, 0);
}
