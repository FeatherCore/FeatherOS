/****************************************************************************
 * arch/arm/src/stm32n6/hardware/stm32n6_rcc.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_RCC_H
#define __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_RCC_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "stm32n6_memorymap.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_RCC_CR_OFFSET            0x00
#define STM32_RCC_ICSCR_OFFSET         0x04
#define STM32_RCC_CFGR1_OFFSET         0x20
#define STM32_RCC_CFGR2_OFFSET         0x24
#define STM32_RCC_PLL1CFGR_OFFSET      0x28
#define STM32_RCC_PLL2CFGR_OFFSET      0x2C
#define STM32_RCC_PLL3CFGR_OFFSET      0x30
#define STM32_RCC_PLL4CFGR_OFFSET      0x34
#define STM32_RCC_AHB1ENR_OFFSET       0x100
#define STM32_RCC_AHB2ENR_OFFSET       0x104
#define STM32_RCC_AHB3ENR_OFFSET       0x108
#define STM32_RCC_AHB4ENR_OFFSET       0x10C
#define STM32_RCC_AHB5ENR_OFFSET       0x110
#define STM32_RCC_APB1ENR_OFFSET       0x180
#define STM32_RCC_APB2ENR_OFFSET       0x184
#define STM32_RCC_APB3ENR_OFFSET       0x188
#define STM32_RCC_APB4ENR_OFFSET       0x18C
#define STM32_RCC_APB5ENR_OFFSET       0x190
#define STM32_RCC_AHB1RSTR_OFFSET      0x200
#define STM32_RCC_AHB2RSTR_OFFSET      0x204
#define STM32_RCC_AHB3RSTR_OFFSET      0x208
#define STM32_RCC_AHB4RSTR_OFFSET      0x20C
#define STM32_RCC_AHB5RSTR_OFFSET      0x210
#define STM32_RCC_APB1RSTR_OFFSET      0x240
#define STM32_RCC_APB2RSTR_OFFSET      0x244
#define STM32_RCC_APB3RSTR_OFFSET      0x248
#define STM32_RCC_APB4RSTR_OFFSET      0x24C
#define STM32_RCC_APB5RSTR_OFFSET      0x250

#define RCC_CR_HSION                   (1 << 0)
#define RCC_CR_HSIRDY                  (1 << 2)
#define RCC_CR_HSEON                   (1 << 16)
#define RCC_CR_HSERDY                  (1 << 17)
#define RCC_CR_HSEBYP                  (1 << 18)
#define RCC_CR_PLL1ON                  (1 << 24)
#define RCC_CR_PLL1RDY                 (1 << 25)
#define RCC_CR_PLL2ON                  (1 << 26)
#define RCC_CR_PLL2RDY                 (1 << 27)
#define RCC_CR_PLL3ON                  (1 << 28)
#define RCC_CR_PLL3RDY                 (1 << 29)
#define RCC_CR_PLL4ON                  (1 << 30)
#define RCC_CR_PLL4RDY                 (1 << 31)

#define RCC_CFGR_SW_MASK               0x00000003
#define RCC_CFGR_SW_HSI                0x00000000
#define RCC_CFGR_SW_HSE                0x00000001
#define RCC_CFGR_SW_PLL1               0x00000002
#define RCC_CFGR_SWS_MASK              0x0000000c
#define RCC_CFGR_SWS_HSI               0x00000000
#define RCC_CFGR_SWS_HSE               0x00000004
#define RCC_CFGR_SWS_PLL1              0x00000008

#define RCC_AHB1ENR_GPDMA1EN           (1 << 4)
#define RCC_AHB1ENR_ADC12EN            (1 << 5)

#define RCC_AHB2ENR_RAMCFGEN           (1 << 12)

#define RCC_AHB4ENR_GPIOAEN            (1 << 0)
#define RCC_AHB4ENR_GPIOBEN            (1 << 1)
#define RCC_AHB4ENR_GPIOCEN            (1 << 2)
#define RCC_AHB4ENR_GPIODEN            (1 << 3)
#define RCC_AHB4ENR_GPIOEEN            (1 << 4)
#define RCC_AHB4ENR_GPIOFEN            (1 << 5)
#define RCC_AHB4ENR_GPIOGEN            (1 << 6)
#define RCC_AHB4ENR_GPIOHEN            (1 << 7)
#define RCC_AHB4ENR_GPIONEN            (1 << 13)
#define RCC_AHB4ENR_GPIOOEN            (1 << 14)
#define RCC_AHB4ENR_GPIOPEN            (1 << 15)
#define RCC_AHB4ENR_GPIOQEN            (1 << 16)
#define RCC_AHB4ENR_PWREN              (1 << 28)

#define RCC_APB1ENR_USART2EN           (1 << 17)
#define RCC_APB1ENR_USART3EN           (1 << 18)
#define RCC_APB1ENR_UART4EN            (1 << 19)
#define RCC_APB1ENR_UART5EN            (1 << 20)
#define RCC_APB1ENR_I2C1EN             (1 << 21)
#define RCC_APB1ENR_I2C2EN             (1 << 22)
#define RCC_APB1ENR_I2C3EN             (1 << 23)

#define RCC_APB2ENR_USART1EN           (1 << 4)
#define RCC_APB2ENR_USART6EN           (1 << 5)
#define RCC_APB2ENR_UART9EN            (1 << 6)
#define RCC_APB2ENR_USART10EN          (1 << 7)
#define RCC_APB2ENR_SPI1EN             (1 << 12)
#define RCC_APB2ENR_SPI4EN             (1 << 13)

#define RCC_APB4ENR_I2C4EN             (1 << 7)

#define RCC_APB5ENR_LTDCEN             (1 << 1)
#define RCC_APB5ENR_DCMIPPEN           (1 << 2)
#define RCC_APB5ENR_VENCEN             (1 << 5)

#define RCC_AHB5ENR_JPEGEN             (1 << 3)
#define RCC_AHB5ENR_XSPI1EN            (1 << 5)
#define RCC_AHB5ENR_SDMMC2EN           (1 << 7)
#define RCC_AHB5ENR_SDMMC1EN           (1 << 8)
#define RCC_AHB5ENR_XSPI2EN            (1 << 12)
#define RCC_AHB5ENR_XSPIMEN            (1 << 13)
#define RCC_AHB5ENR_XSPI3EN            (1 << 17)
#define RCC_AHB5ENR_ETH1EN             (1 << 22)
#define RCC_AHB5ENR_ETH1TXEN           (1 << 23)
#define RCC_AHB5ENR_ETH1RXEN           (1 << 24)
#define RCC_AHB5ENR_USBPHYC1EN         (1 << 27)
#define RCC_AHB5ENR_USB1OTGEN          (1 << 26)
#define RCC_AHB5ENR_USBPHYC2EN         (1 << 28)
#define RCC_AHB5ENR_USB2OTGEN          (1 << 29)
#define RCC_AHB5ENR_NPUCACHEEN         (1 << 30)
#define RCC_AHB5ENR_NPUEN              (1 << 31)

#endif /* __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_RCC_H */
