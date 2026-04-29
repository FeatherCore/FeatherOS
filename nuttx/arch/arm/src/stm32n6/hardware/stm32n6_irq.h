/****************************************************************************
 * arch/arm/src/stm32n6/hardware/stm32n6_irq.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_IRQ_H
#define __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_IRQ_H

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32N6_IRQ_NIRQS              196

#define STM32N6_IRQ_RESERVED           0
#define STM32N6_IRQ_NMI                2
#define STM32N6_IRQ_HARDFAULT          3
#define STM32N6_IRQ_MEMMANAGE          4
#define STM32N6_IRQ_BUSFAULT           5
#define STM32N6_IRQ_USAGEFAULT         6
#define STM32N6_IRQ_SVCALL             11
#define STM32N6_IRQ_DEBUGMONITOR       12
#define STM32N6_IRQ_PENDSV             14
#define STM32N6_IRQ_SYSTICK            15

#define STM32N6_IRQ_WWDG               16
#define STM32N6_IRQ_PVD                17
#define STM32N6_IRQ_TAMP               18
#define STM32N6_IRQ_RTC                19
#define STM32N6_IRQ_EXTI0              20
#define STM32N6_IRQ_EXTI1              21
#define STM32N6_IRQ_EXTI2              22
#define STM32N6_IRQ_EXTI3              23
#define STM32N6_IRQ_EXTI4              24
#define STM32N6_IRQ_DMA1_CHANNEL1      25
#define STM32N6_IRQ_DMA1_CHANNEL2      26
#define STM32N6_IRQ_DMA1_CHANNEL3      27
#define STM32N6_IRQ_DMA1_CHANNEL4      28
#define STM32N6_IRQ_DMA1_CHANNEL5      29
#define STM32N6_IRQ_DMA1_CHANNEL6      30
#define STM32N6_IRQ_DMA1_CHANNEL7      31
#define STM32N6_IRQ_ADC1_2             46
#define STM32N6_IRQ_DCMIPP             47
#define STM32N6_IRQ_DCMIPP_CSI         48
#define STM32N6_IRQ_FDCAN1_IT0         180
#define STM32N6_IRQ_FDCAN1_IT1         181
#define STM32N6_IRQ_FDCAN2_IT0         182
#define STM32N6_IRQ_FDCAN2_IT1         183
#define STM32N6_IRQ_FDCAN3_IT0         184
#define STM32N6_IRQ_FDCAN3_IT1         185
#define STM32N6_IRQ_TIM1_BRK           112
#define STM32N6_IRQ_TIM1_UP            113
#define STM32N6_IRQ_TIM1_TRG_COM       114
#define STM32N6_IRQ_TIM1_CC            115
#define STM32N6_IRQ_TIM2               116
#define STM32N6_IRQ_TIM3               117
#define STM32N6_IRQ_TIM4               118
#define STM32N6_IRQ_TIM5               119
#define STM32N6_IRQ_TIM6               120
#define STM32N6_IRQ_TIM7               121
#define STM32N6_IRQ_TIM8_BRK           122
#define STM32N6_IRQ_TIM8_UP            123
#define STM32N6_IRQ_TIM8_TRG_COM       124
#define STM32N6_IRQ_TIM8_CC            125
#define STM32N6_IRQ_I2C1_EV            100
#define STM32N6_IRQ_I2C1_ER            101
#define STM32N6_IRQ_I2C2_EV            102
#define STM32N6_IRQ_I2C2_ER            103
#define STM32N6_IRQ_I2C3_EV            104
#define STM32N6_IRQ_I2C3_ER            105
#define STM32N6_IRQ_I2C4_EV            106
#define STM32N6_IRQ_I2C4_ER            107
#define STM32N6_IRQ_SPI1               153
#define STM32N6_IRQ_SPI2               154
#define STM32N6_IRQ_SPI3               155
#define STM32N6_IRQ_SPI4               156
#define STM32N6_IRQ_SPI5               157
#define STM32N6_IRQ_SPI6               158
#define STM32N6_IRQ_USART1             159
#define STM32N6_IRQ_USART2             160
#define STM32N6_IRQ_USART3             161
#define STM32N6_IRQ_UART4              162
#define STM32N6_IRQ_UART5              163
#define STM32N6_IRQ_USART6             164
#define STM32N6_IRQ_UART7              165
#define STM32N6_IRQ_UART8              166
#define STM32N6_IRQ_UART9              167
#define STM32N6_IRQ_USART10            168
#define STM32N6_IRQ_I3C1_EV            108
#define STM32N6_IRQ_I3C1_ER            109
#define STM32N6_IRQ_I3C2_EV            110
#define STM32N6_IRQ_I3C2_ER            111
#define STM32N6_IRQ_XSPI1              170
#define STM32N6_IRQ_XSPI2              171
#define STM32N6_IRQ_XSPI3              172
#define STM32N6_IRQ_SDMMC1             174
#define STM32N6_IRQ_SDMMC2             175
#define STM32N6_IRQ_ETH                179
#define STM32N6_IRQ_USB_OTG_HS1        177
#define STM32N6_IRQ_USB_OTG_HS2        178
#define STM32N6_IRQ_LTDC               193
#define STM32N6_IRQ_LTDC_ER            194
#define STM32N6_IRQ_JPEG               61
#define STM32N6_IRQ_VENC               62
#define STM32N6_IRQ_RNG                40
#define STM32N6_IRQ_NPU                195

#endif /* __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_IRQ_H */