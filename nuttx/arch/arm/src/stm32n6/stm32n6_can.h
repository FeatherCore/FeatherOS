/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_can.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_CAN_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_CAN_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <stdint.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_FDCAN1_BASE           (STM32N6_PERIPH_BASE + 0x0000A000)
#define STM32_FDCAN2_BASE           (STM32N6_PERIPH_BASE + 0x0000A400)
#define STM32_FDCAN3_BASE           (STM32N6_PERIPH_BASE + 0x0000E800)

#define FDCAN_CREL_OFFSET          0x000
#define FDCAN_ENDN_OFFSET          0x004
#define FDCAN_MRCFG_OFFSET         0x008
#define FDCAN_DBTP_OFFSET          0x00C
#define FDCAN_TEST_OFFSET          0x010
#define FDCAN_RWD_OFFSET           0x014
#define FDCAN_CCCR_OFFSET          0x018
#define FDCAN_NBTP_OFFSET          0x01C
#define FDCAN_TSCC_OFFSET          0x020
#define FDCAN_TSCV_OFFSET          0x024
#define FDCAN_TOCC_OFFSET          0x028
#define FDCAN_TOCV_OFFSET          0x02C
#define FDCAN_ECR_OFFSET           0x040
#define FDCAN_PSR_OFFSET           0x044
#define FDCAN_TDCR_OFFSET          0x048
#define FDCAN_IR_OFFSET            0x050
#define FDCAN_IE_OFFSET            0x054
#define FDCAN_ILS_OFFSET           0x058
#define FDCAN_ILE_OFFSET           0x05C

/* Message RAM Configuration */
#define FDCAN_MRSCFG_OFFSET        0x0C0
#define FDCAN_SIDFC_OFFSET         0x0C4
#define FDCAN_XIDFC_OFFSET         0x0C8
#define FDCAN_XIDAM_OFFSET         0x0D0
#define FDCAN_HPMS_OFFSET          0x0D4
#define FDCAN_RXF0C_OFFSET         0x0D8
#define FDCAN_RXF0S_OFFSET         0x0DC
#define FDCAN_RXF0A_OFFSET         0x0E0
#define FDCAN_RXBC_OFFSET          0x0E4
#define FDCAN_RXF1C_OFFSET         0x0E8
#define FDCAN_RXF1S_OFFSET         0x0EC
#define FDCAN_RXF1A_OFFSET         0x0F0
#define FDCAN_RXESC_OFFSET         0x0F4
#define FDCAN_TXESC_OFFSET         0x0F8
#define FDCAN_TXBGC_OFFSET         0x0FC
#define FDCAN_TXBC_OFFSET          0x100
#define FDCAN_TXFQS_OFFSET         0x104
#define FDCAN_TXEFC_OFFSET         0x108
#define FDCAN_TXEFS_OFFSET         0x10C
#define FDCAN_TXEFA_OFFSET         0x110

/* CAN Control and Status Register */
#define FDCAN_CCCR_INIT            (1 << 0)
#define FDCAN_CCCR_CCE             (1 << 1)
#define FDCAN_CCCR_ASM             (1 << 2)
#define FDCAN_CCCR_CSA             (1 << 3)
#define FDCAN_CCCR_CSR             (1 << 4)
#define FDCAN_CCCR_MON             (1 << 5)
#define FDCAN_CCCR_DAR             (1 << 6)
#define FDCAN_CCCR_TEST            (1 << 7)
#define FDCAN_CCCR_FDOE            (1 << 8)
#define FDCAN_CCCR_BSE             (1 << 9)
#define FDCAN_CCCR_PXHD            (1 << 12)
#define FDCAN_CCCR_EFBI            (1 << 13)
#define FDCAN_CCCR_TXP             (1 << 14)
#define FDCAN_CCCR_NISO            (1 << 15)

/* Protocol Status Register */
#define FDCAN_PSR_LEC_SHIFT        0
#define FDCAN_PSR_LEC_MASK         (7 << FDCAN_PSR_LEC_SHIFT)
#define FDCAN_PSR_ACT_SHIFT        3
#define FDCAN_PSR_ACT_MASK         (3 << FDCAN_PSR_ACT_SHIFT)
#define FDCAN_PSR_EP               (1 << 5)
#define FDCAN_PSR_EW               (1 << 6)
#define FDCAN_PSR_BO               (1 << 7)
#define FDCAN_PSR_DLEC_SHIFT       8
#define FDCAN_PSR_DLEC_MASK        (7 << FDCAN_PSR_DLEC_SHIFT)
#define FDCAN_PSR_RESI_SHIFT       11
#define FDCAN_PSR_RESI_MASK        (1 << FDCAN_PSR_RESI_SHIFT)
#define FDCAN_PSR_RBRS_SHIFT       12
#define FDCAN_PSR_RBRS_MASK        (1 << FDCAN_PSR_RBRS_SHIFT)
#define FDCAN_PSR_RFDF_SHIFT       13
#define FDCAN_PSR_RFDF_MASK        (1 << FDCAN_PSR_RFDF_SHIFT)
#define FDCAN_PSR_PED_SHIFT        14
#define FDCAN_PSR_PED_MASK         (1 << FDCAN_PSR_PED_SHIFT)
#define FDCAN_PSR_PXE_SHIFT        15
#define FDCAN_PSR_PXE_MASK         (1 << FDCAN_PSR_PXE_SHIFT)

/* Interrupt Register */
#define FDCAN_IR_RF0N              (1 << 0)
#define FDCAN_IR_RF0W              (1 << 1)
#define FDCAN_IR_RF0F              (1 << 2)
#define FDCAN_IR_RF0L              (1 << 3)
#define FDCAN_IR_RF1N              (1 << 4)
#define FDCAN_IR_RF1W              (1 << 5)
#define FDCAN_IR_RF1F              (1 << 6)
#define FDCAN_IR_RF1L              (1 << 7)
#define FDCAN_IR_HPM               (1 << 8)
#define FDCAN_IR_TC                (1 << 9)
#define FDCAN_IR_TCF               (1 << 10)
#define FDCAN_IR_TFE               (1 << 11)
#define FDCAN_IR_TEFN              (1 << 12)
#define FDCAN_IR_TEFW              (1 << 13)
#define FDCAN_IR_TEFF              (1 << 14)
#define FDCAN_IR_TEFL              (1 << 15)
#define FDCAN_IR_TSW               (1 << 16)
#define FDCAN_IR_MRAF              (1 << 17)
#define FDCAN_IR_TOO               (1 << 18)
#define FDCAN_IR_DRX               (1 << 19)
#define FDCAN_IR_BEC               (1 << 20)
#define FDCAN_IR_BEU               (1 << 21)
#define FDCAN_IR_ELO               (1 << 22)
#define FDCAN_IR_EP                (1 << 23)
#define FDCAN_IR_EW                (1 << 24)
#define FDCAN_IR_BO                (1 << 25)
#define FDCAN_IR_WDI               (1 << 26)
#define FDCAN_IR_PEA               (1 << 27)
#define FDCAN_IR_PED               (1 << 28)
#define FDCAN_IR_ARA               (1 << 29)

/* CAN Bit Timing and Prescaler Register */
#define FDCAN_DBTP_DSJW_SHIFT      0
#define FDCAN_DBTP_DSJW_MASK       (0xF << FDCAN_DBTP_DSJW_SHIFT)
#define FDCAN_DBTP_DTSEG2_SHIFT    4
#define FDCAN_DBTP_DTSEG2_MASK     (0xF << FDCAN_DBTP_DTSEG2_SHIFT)
#define FDCAN_DBTP_DTSEG1_SHIFT    8
#define FDCAN_DBTP_DTSEG1_MASK     (0x1F << FDCAN_DBTP_DTSEG1_SHIFT)
#define FDCAN_DBTP_DBRP_SHIFT      16
#define FDCAN_DBTP_DBRP_MASK       (0x1F << FDCAN_DBTP_DBRP_SHIFT)
#define FDCAN_DBTP_TDCO_SHIFT      23
#define FDCAN_DBTP_TDCO_MASK       (0x7F << FDCAN_DBTP_TDCO_SHIFT)

#define FDCAN_NBTP_NTSEG2_SHIFT    0
#define FDCAN_NBTP_NTSEG2_MASK     (0x7F << FDCAN_NBTP_NTSEG2_SHIFT)
#define FDCAN_NBTP_NTSEG1_SHIFT    8
#define FDCAN_NBTP_NTSEG1_MASK     (0xFF << FDCAN_NBTP_NTSEG1_SHIFT)
#define FDCAN_NBTP_NBRP_SHIFT      16
#define FDCAN_NBTP_NBRP_MASK       (0x1FF << FDCAN_NBTP_NBRP_SHIFT)
#define FDCAN_NBTP_NSJW_SHIFT      25
#define FDCAN_NBTP_NSJW_MASK       (0x7F << FDCAN_NBTP_NSJW_SHIFT)

/****************************************************************************
 * Public Types
 ****************************************************************************/

struct stm32n6_can_s
{
  uintptr_t canbase;
  uint32_t bitrate;
  uint32_t dbitrate;
  uint8_t protocol_exception;
  int irq0;
  int irq1;
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

int stm32n6_can_initialize(uintptr_t canbase, uint32_t bitrate, uint32_t dbitrate);
void stm32n6_can_enable(uintptr_t canbase);
void stm32n6_can_disable(uintptr_t canbase);
int stm32n6_can_transmit(uintptr_t canbase, uint32_t id, bool extended, 
                         bool fd, bool brs, uint8_t *data, uint8_t dlc);
int stm32n6_can_receive(uintptr_t canbase, uint32_t *id, bool *extended,
                        uint8_t *data, uint8_t *dlc);
bool stm32n6_can_available(uintptr_t canbase);
void stm32n6_can_reset_error_counters(uintptr_t canbase);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_CAN_H */