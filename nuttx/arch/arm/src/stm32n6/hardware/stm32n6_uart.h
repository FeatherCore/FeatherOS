/****************************************************************************
 * arch/arm/src/stm32n6/hardware/stm32n6_uart.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_UART_H
#define __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_UART_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "stm32n6_memorymap.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_USART_CR1_OFFSET         0x00
#define STM32_USART_CR2_OFFSET         0x04
#define STM32_USART_CR3_OFFSET         0x08
#define STM32_USART_BRR_OFFSET         0x0C
#define STM32_USART_GTPR_OFFSET        0x10
#define STM32_USART_RTOR_OFFSET        0x14
#define STM32_USART_RQR_OFFSET         0x18
#define STM32_USART_ISR_OFFSET         0x1C
#define STM32_USART_ICR_OFFSET         0x20
#define STM32_USART_RDR_OFFSET         0x24
#define STM32_USART_TDR_OFFSET         0x28
#define STM32_USART_PRESC_OFFSET       0x2C

#define USART_CR1_UE                   (1 << 0)
#define USART_CR1_UESM                 (1 << 1)
#define USART_CR1_RE                   (1 << 2)
#define USART_CR1_TE                   (1 << 3)
#define USART_CR1_IDLEIE               (1 << 4)
#define USART_CR1_RXNEIE_RXFNEIE       (1 << 5)
#define USART_CR1_TCIE                 (1 << 6)
#define USART_CR1_TXEIE_TXFNFIE        (1 << 7)
#define USART_CR1_PEIE                 (1 << 8)
#define USART_CR1_PS                   (1 << 9)
#define USART_CR1_PCE                  (1 << 10)
#define USART_CR1_WAKE                 (1 << 11)
#define USART_CR1_M0                   (1 << 12)
#define USART_CR1_MME                  (1 << 13)
#define USART_CR1_CMIE                 (1 << 14)
#define USART_CR1_OVER8                (1 << 15)
#define USART_CR1_DEDT_SHIFT           16
#define USART_CR1_DEDT_MASK            (0x1f << USART_CR1_DEDT_SHIFT)
#define USART_CR1_DEAT_SHIFT           21
#define USART_CR1_DEAT_MASK            (0x1f << USART_CR1_DEAT_SHIFT)
#define USART_CR1_RTOIE                (1 << 26)
#define USART_CR1_EOBIE                (1 << 27)
#define USART_CR1_TXEIE                (1 << 28)
#define USART_CR1_M1                   (1 << 28)
#define USART_CR1_FIFOEN               (1 << 29)
#define USART_CR1_RXFFIE               (1 << 30)
#define USART_CR1_TXFEIE               (1 << 31)

#define USART_CR2_SLVEN                (1 << 0)
#define USART_CR2_DIS_NSS              (1 << 3)
#define USART_CR2_ADDM7                (1 << 4)
#define USART_CR2_LBDIE                (1 << 6)
#define USART_CR2_LBDL                 (1 << 5)
#define USART_CR2_STOP_SHIFT           12
#define USART_CR2_STOP_MASK            (3 << USART_CR2_STOP_SHIFT)
#define USART_CR2_STOP_1               (0 << USART_CR2_STOP_SHIFT)
#define USART_CR2_STOP_0_5             (1 << USART_CR2_STOP_SHIFT)
#define USART_CR2_STOP_2               (2 << USART_CR2_STOP_SHIFT)
#define USART_CR2_STOP_1_5             (3 << USART_CR2_STOP_SHIFT)
#define USART_CR2_LINEN                (1 << 14)
#define USART_CR2_SWAP                (1 << 15)
#define USART_CR2_RXINV                (1 << 16)
#define USART_CR2_TXINV                (1 << 17)
#define USART_CR2_DATAINV              (1 << 18)
#define USART_CR2_MSBFIRST             (1 << 19)
#define USART_CR2_AUTOFLOW             (1 << 20)
#define USART_CR2_RTOEN                (1 << 23)
#define USART_CR2_ADD_SHIFT            24
#define USART_CR2_ADD_MASK             (0xff << USART_CR2_ADD_SHIFT)

#define USART_CR3_EIE                  (1 << 0)
#define USART_CR3_IREN                 (1 << 1)
#define USART_CR3_IRLP                 (1 << 2)
#define USART_CR3_HDSEL                (1 << 3)
#define USART_CR3_NACK                 (1 << 4)
#define USART_CR3_SCEN                 (1 << 5)
#define USART_CR3_DMAR                 (1 << 6)
#define USART_CR3_DMAT                 (1 << 7)
#define USART_CR3_RTSE                 (1 << 8)
#define USART_CR3_CTSE                 (1 << 9)
#define USART_CR3_CTSIE                (1 << 10)
#define USART_CR3_ONEBIT               (1 << 11)
#define USART_CR3_OVRDIS               (1 << 12)
#define USART_CR3_DEM                  (1 << 14)
#define USART_CR3_DEP                  (1 << 15)
#define USART_CR3_SCARCNT_SHIFT        17
#define USART_CR3_SCARCNT_MASK         (7 << USART_CR3_SCARCNT_SHIFT)
#define USART_CR3_WUS_SHIFT            20
#define USART_CR3_WUS_MASK             (3 << USART_CR3_WUS_SHIFT)
#define USART_CR3_WUFIE                (1 << 22)
#define USART_CR3_TXFTCFG_SHIFT        25
#define USART_CR3_TXFTCFG_MASK         (7 << USART_CR3_TXFTCFG_SHIFT)
#define USART_CR3_RXFTCFG_SHIFT        28
#define USART_CR3_RXFTCFG_MASK         (7 << USART_CR3_RXFTCFG_SHIFT)

#define USART_ISR_PE                   (1 << 0)
#define USART_ISR_FE                   (1 << 1)
#define USART_ISR_NF                   (1 << 2)
#define USART_ISR_ORE                  (1 << 3)
#define USART_ISR_IDLE                 (1 << 4)
#define USART_ISR_RXNE_RXFNE           (1 << 5)
#define USART_ISR_TC                   (1 << 6)
#define USART_ISR_TXE_TXFNF            (1 << 7)
#define USART_ISR_LBDF                 (1 << 8)
#define USART_ISR_CTSIF                (1 << 9)
#define USART_ISR_CTS                  (1 << 10)
#define USART_ISR_RTOF                 (1 << 11)
#define USART_ISR_EOBF                 (1 << 12)
#define USART_ISR_UDR                  (1 << 13)
#define USART_ISR_ABRE                 (1 << 14)
#define USART_ISR_ABRF                 (1 << 15)
#define USART_ISR_BUSY                 (1 << 16)
#define USART_ISR_CMF                  (1 << 17)
#define USART_ISR_SBKF                 (1 << 18)
#define USART_ISR_WUF                  (1 << 19)
#define USART_ISR_TEACK                (1 << 21)
#define USART_ISR_RWU                  (1 << 22)
#define USART_ISR_RXFF                 (1 << 24)
#define USART_ISR_TXFE                 (1 << 25)
#define USART_ISR_RXFT                 (1 << 26)
#define USART_ISR_TXFT                 (1 << 27)

#define USART_ICR_PECF                 (1 << 0)
#define USART_ICR_FECF                 (1 << 1)
#define USART_ICR_NECF                 (1 << 2)
#define USART_ICR_ORECF                (1 << 3)
#define USART_ICR_IDLECF               (1 << 4)
#define USART_ICR_TXFECF               (1 << 5)
#define USART_ICR_TCCF                 (1 << 6)
#define USART_ICR_TCBGTCF              (1 << 7)
#define USART_ICR_LBDCF                (1 << 8)
#define USART_ICR_CTSCF                (1 << 9)
#define USART_ICR_RTOCF                (1 << 11)
#define USART_ICR_EOBCF                (1 << 12)
#define USART_ICR_UDRCF                (1 << 13)
#define USART_ICR_CMCF                 (1 << 17)
#define USART_ICR_SBKCF                (1 << 18)
#define USART_ICR_WUCF                 (1 << 19)

#endif /* __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_UART_H */