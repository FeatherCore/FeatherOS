/****************************************************************************
 * boards/arm/stm32n6/stm32n6570-dk/include/board.h
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

#ifndef __BOARDS_ARM_STM32N6_STM32N6570_DK_INCLUDE_BOARD_H
#define __BOARDS_ARM_STM32N6_STM32N6570_DK_INCLUDE_BOARD_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#ifndef __ASSEMBLY__
#  include <stdint.h>
#endif

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define BOARD_XTAL_FREQUENCY      48000000ul
#define BOARD_SYSCLK_FREQUENCY    CONFIG_STM32N6_SYSCLK_FREQUENCY

#define GPIO_USART1_TX            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_PULLUP | \
                                   GPIO_AF(7) | GPIO_PORTE | GPIO_PIN5)
#define GPIO_USART1_RX            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_PULLUP | \
                                   GPIO_AF(7) | GPIO_PORTE | GPIO_PIN6)

#define GPIO_USART2_TX            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_PULLUP | \
                                   GPIO_AF(7) | GPIO_GPIOD | GPIO_PIN5)
#define GPIO_USART2_RX            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_PULLUP | \
                                   GPIO_AF(7) | GPIO_GPIOF | GPIO_PIN6)

#define GPIO_LED1                 (GPIO_OUTPUT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_LOW | GPIO_OUTPUT_CLEAR | \
                                   GPIO_GPIOO | GPIO_PIN1)
#define GPIO_LED2                 (GPIO_OUTPUT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_LOW | GPIO_OUTPUT_SET | \
                                   GPIO_GPIOG | GPIO_PIN10)

#define GPIO_BTN_USER             (GPIO_INPUT | GPIO_PULLDOWN | \
                                   GPIO_GPIOC | GPIO_PIN13)

/* The STM32N6570-DK board wiring below is taken from Zephyr's
 * stm32n6570_dk_common.dtsi.  The matching STM32N657x0HxQ pinctrl include is
 * not present in the local Zephyr tree, so non-console alternate-function
 * numbers are kept in one place for later validation against ST pin data.
 */

#define GPIO_AF_I2C               GPIO_AF(4)
#define GPIO_AF_SPI5              GPIO_AF(5)
#define GPIO_AF_FDCAN             GPIO_AF(9)
#define GPIO_AF_SDMMC             GPIO_AF(12)
#define GPIO_AF_TIM1              GPIO_AF(1)
#define GPIO_AF_TIM15             GPIO_AF(4)
#define GPIO_AF_XSPI              GPIO_AF(9)
#define GPIO_AF_ETH               GPIO_AF(11)
#define GPIO_AF_LTDC              GPIO_AF(14)

#define GPIO_I2C1_SCL             (GPIO_ALT | GPIO_OPENDRAIN | \
                                   GPIO_SPEED_HIGH | GPIO_PULLUP | \
                                   GPIO_AF_I2C | GPIO_GPIOH | GPIO_PIN9)
#define GPIO_I2C1_SDA             (GPIO_ALT | GPIO_OPENDRAIN | \
                                   GPIO_SPEED_HIGH | GPIO_PULLUP | \
                                   GPIO_AF_I2C | GPIO_GPIOC | GPIO_PIN1)

#define GPIO_I2C2_SCL             (GPIO_ALT | GPIO_OPENDRAIN | \
                                   GPIO_SPEED_HIGH | GPIO_PULLUP | \
                                   GPIO_AF_I2C | GPIO_GPIOD | GPIO_PIN14)
#define GPIO_I2C2_SDA             (GPIO_ALT | GPIO_OPENDRAIN | \
                                   GPIO_SPEED_HIGH | GPIO_PULLUP | \
                                   GPIO_AF_I2C | GPIO_GPIOD | GPIO_PIN4)

#define GPIO_I2C4_SCL             (GPIO_ALT | GPIO_OPENDRAIN | \
                                   GPIO_SPEED_HIGH | GPIO_PULLUP | \
                                   GPIO_AF_I2C | GPIO_GPIOE | GPIO_PIN13)
#define GPIO_I2C4_SDA             (GPIO_ALT | GPIO_OPENDRAIN | \
                                   GPIO_SPEED_HIGH | GPIO_PULLUP | \
                                   GPIO_AF_I2C | GPIO_GPIOE | GPIO_PIN14)

#define GPIO_GT911_RESET          (GPIO_OUTPUT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_LOW | GPIO_OUTPUT_SET | \
                                   GPIO_GPIOE | GPIO_PIN1)
#define GPIO_GT911_IRQ            (GPIO_INPUT | GPIO_PULLDOWN | \
                                   GPIO_GPIOQ | GPIO_PIN4)

#define GPIO_SPI5_NSS             (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_PULLUP | \
                                   GPIO_AF_SPI5 | GPIO_GPIOA | GPIO_PIN3)
#define GPIO_SPI5_SCK             (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_SPI5 | GPIO_GPIOE | GPIO_PIN15)
#define GPIO_SPI5_MISO            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_SPI5 | GPIO_GPIOH | GPIO_PIN8)
#define GPIO_SPI5_MOSI            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_SPI5 | GPIO_GPIOG | GPIO_PIN2)

#define GPIO_FDCAN1_RX            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_PULLUP | \
                                   GPIO_AF_FDCAN | GPIO_GPIOD | GPIO_PIN0)
#define GPIO_FDCAN1_TX            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_PULLUP | \
                                   GPIO_AF_FDCAN | GPIO_GPIOH | GPIO_PIN2)

#define GPIO_SDMMC2_D0            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_PULLUP | \
                                   GPIO_AF_SDMMC | GPIO_GPIOC | GPIO_PIN4)
#define GPIO_SDMMC2_D1            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_PULLUP | \
                                   GPIO_AF_SDMMC | GPIO_GPIOC | GPIO_PIN5)
#define GPIO_SDMMC2_D2            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_PULLUP | \
                                   GPIO_AF_SDMMC | GPIO_GPIOC | GPIO_PIN0)
#define GPIO_SDMMC2_D3            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_PULLUP | \
                                   GPIO_AF_SDMMC | GPIO_GPIOE | GPIO_PIN4)
#define GPIO_SDMMC2_CK            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_SDMMC | GPIO_GPIOC | GPIO_PIN2)
#define GPIO_SDMMC2_CMD           (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_PULLUP | \
                                   GPIO_AF_SDMMC | GPIO_GPIOC | GPIO_PIN3)
#define GPIO_SDMMC2_CD            (GPIO_INPUT | GPIO_PULLUP | \
                                   GPIO_GPION | GPIO_PIN12)
#define GPIO_SDMMC2_PWR           (GPIO_OUTPUT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_LOW | GPIO_OUTPUT_CLEAR | \
                                   GPIO_GPIOQ | GPIO_PIN7)

#define GPIO_ADC1_INP10           (GPIO_ANALOG | GPIO_FLOAT | \
                                   GPIO_GPIOA | GPIO_PIN9)
#define GPIO_ADC1_INP11           (GPIO_ANALOG | GPIO_FLOAT | \
                                   GPIO_GPIOA | GPIO_PIN10)

#define GPIO_TIM1_CH1             (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_TIM1 | GPIO_GPIOE | GPIO_PIN9)
#define GPIO_TIM15_CH1            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_TIM15 | GPIO_GPIOC | GPIO_PIN12)

#define GPIO_XSPI1_NCS            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_PULLUP | \
                                   GPIO_AF_XSPI | GPIO_GPIOO | GPIO_PIN0)
#define GPIO_XSPI1_DQS0           (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPIOO | GPIO_PIN2)
#define GPIO_XSPI1_DQS1           (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPIOO | GPIO_PIN3)
#define GPIO_XSPI1_CLK            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPIOO | GPIO_PIN4)
#define GPIO_XSPI1_IO0            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPIOP | GPIO_PIN0)
#define GPIO_XSPI1_IO1            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPIOP | GPIO_PIN1)
#define GPIO_XSPI1_IO2            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPIOP | GPIO_PIN2)
#define GPIO_XSPI1_IO3            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPIOP | GPIO_PIN3)
#define GPIO_XSPI1_IO4            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPIOP | GPIO_PIN4)
#define GPIO_XSPI1_IO5            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPIOP | GPIO_PIN5)
#define GPIO_XSPI1_IO6            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPIOP | GPIO_PIN6)
#define GPIO_XSPI1_IO7            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPIOP | GPIO_PIN7)
#define GPIO_XSPI1_IO8            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPIOP | GPIO_PIN8)
#define GPIO_XSPI1_IO9            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPIOP | GPIO_PIN9)
#define GPIO_XSPI1_IO10           (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPIOP | GPIO_PIN10)
#define GPIO_XSPI1_IO11           (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPIOP | GPIO_PIN11)
#define GPIO_XSPI1_IO12           (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPIOP | GPIO_PIN12)
#define GPIO_XSPI1_IO13           (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPIOP | GPIO_PIN13)
#define GPIO_XSPI1_IO14           (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPIOP | GPIO_PIN14)
#define GPIO_XSPI1_IO15           (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPIOP | GPIO_PIN15)

#define GPIO_XSPI2_NCS            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_PULLUP | \
                                   GPIO_AF_XSPI | GPIO_GPION | GPIO_PIN1)
#define GPIO_XSPI2_DQS            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPION | GPIO_PIN0)
#define GPIO_XSPI2_CLK            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPION | GPIO_PIN6)
#define GPIO_XSPI2_IO0            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPION | GPIO_PIN2)
#define GPIO_XSPI2_IO1            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPION | GPIO_PIN3)
#define GPIO_XSPI2_IO2            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPION | GPIO_PIN4)
#define GPIO_XSPI2_IO3            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPION | GPIO_PIN5)
#define GPIO_XSPI2_IO4            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPION | GPIO_PIN8)
#define GPIO_XSPI2_IO5            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPION | GPIO_PIN9)
#define GPIO_XSPI2_IO6            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPION | GPIO_PIN10)
#define GPIO_XSPI2_IO7            (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_XSPI | GPIO_GPION | GPIO_PIN11)

#define GPIO_ETH_GTX_CLK          (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_ETH | GPIO_GPIOF | GPIO_PIN0)
#define GPIO_ETH_MDC              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_ETH | GPIO_GPIOD | GPIO_PIN1)
#define GPIO_ETH_MDIO             (GPIO_ALT | GPIO_OPENDRAIN | \
                                   GPIO_SPEED_HIGH | GPIO_PULLUP | \
                                   GPIO_AF_ETH | GPIO_GPIOD | GPIO_PIN12)
#define GPIO_ETH_CLK125           (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_ETH | GPIO_GPIOF | GPIO_PIN2)
#define GPIO_ETH_RX_CLK           (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_ETH | GPIO_GPIOF | GPIO_PIN7)
#define GPIO_ETH_RXD2             (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_ETH | GPIO_GPIOF | GPIO_PIN8)
#define GPIO_ETH_RXD3             (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_ETH | GPIO_GPIOF | GPIO_PIN9)
#define GPIO_ETH_RX_CTL           (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_ETH | GPIO_GPIOF | GPIO_PIN10)
#define GPIO_ETH_TX_CTL           (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_ETH | GPIO_GPIOF | GPIO_PIN11)
#define GPIO_ETH_TXD0             (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_ETH | GPIO_GPIOF | GPIO_PIN12)
#define GPIO_ETH_TXD1             (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_ETH | GPIO_GPIOF | GPIO_PIN13)
#define GPIO_ETH_RXD0             (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_ETH | GPIO_GPIOF | GPIO_PIN14)
#define GPIO_ETH_RXD1             (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_ETH | GPIO_GPIOF | GPIO_PIN15)
#define GPIO_ETH_TXD2             (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_ETH | GPIO_GPIOG | GPIO_PIN3)
#define GPIO_ETH_TXD3             (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_VERYHIGH | GPIO_FLOAT | \
                                   GPIO_AF_ETH | GPIO_GPIOG | GPIO_PIN4)
#define GPIO_ETH_PHY_INT          (GPIO_INPUT | GPIO_PULLUP | \
                                   GPIO_GPIOD | GPIO_PIN3)

#define GPIO_LTDC_R0              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOG | GPIO_PIN0)
#define GPIO_LTDC_R1              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOD | GPIO_PIN9)
#define GPIO_LTDC_R2              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOD | GPIO_PIN15)
#define GPIO_LTDC_R3              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOB | GPIO_PIN4)
#define GPIO_LTDC_R4              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOH | GPIO_PIN4)
#define GPIO_LTDC_R5              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOA | GPIO_PIN15)
#define GPIO_LTDC_R6              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOG | GPIO_PIN11)
#define GPIO_LTDC_R7              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOD | GPIO_PIN8)
#define GPIO_LTDC_G0              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOG | GPIO_PIN12)
#define GPIO_LTDC_G1              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOG | GPIO_PIN1)
#define GPIO_LTDC_G2              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOA | GPIO_PIN1)
#define GPIO_LTDC_G3              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOA | GPIO_PIN0)
#define GPIO_LTDC_G4              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOB | GPIO_PIN15)
#define GPIO_LTDC_G5              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOB | GPIO_PIN12)
#define GPIO_LTDC_G6              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOB | GPIO_PIN11)
#define GPIO_LTDC_G7              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOG | GPIO_PIN8)
#define GPIO_LTDC_B0              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOG | GPIO_PIN15)
#define GPIO_LTDC_B1              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOA | GPIO_PIN7)
#define GPIO_LTDC_B2              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOB | GPIO_PIN2)
#define GPIO_LTDC_B3              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOG | GPIO_PIN6)
#define GPIO_LTDC_B4              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOH | GPIO_PIN3)
#define GPIO_LTDC_B5              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOH | GPIO_PIN6)
#define GPIO_LTDC_B6              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOA | GPIO_PIN8)
#define GPIO_LTDC_B7              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOA | GPIO_PIN2)
#define GPIO_LTDC_DE              (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOG | GPIO_PIN13)
#define GPIO_LTDC_CLK             (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOB | GPIO_PIN13)
#define GPIO_LTDC_HSYNC           (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOB | GPIO_PIN14)
#define GPIO_LTDC_VSYNC           (GPIO_ALT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_HIGH | GPIO_FLOAT | \
                                   GPIO_AF_LTDC | GPIO_GPIOE | GPIO_PIN11)
#define GPIO_DISPLAY_ON           (GPIO_OUTPUT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_LOW | GPIO_OUTPUT_CLEAR | \
                                   GPIO_GPIOQ | GPIO_PIN3)
#define GPIO_DISPLAY_BL           (GPIO_OUTPUT | GPIO_PUSHPULL | \
                                   GPIO_SPEED_LOW | GPIO_OUTPUT_CLEAR | \
                                   GPIO_GPIOQ | GPIO_PIN6)

#define LED_STARTED               0
#define LED_HEAPALLOCATE          0
#define LED_IRQSENABLED           0
#define LED_STACKCREATED          1
#define LED_INIRQ                 1
#define LED_SIGNAL                1
#define LED_ASSERTION             1
#define LED_PANIC                 1

#endif /* __BOARDS_ARM_STM32N6_STM32N6570_DK_INCLUDE_BOARD_H */
