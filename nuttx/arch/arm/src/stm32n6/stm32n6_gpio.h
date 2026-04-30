/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_gpio.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_GPIO_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_GPIO_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <stdbool.h>
#include <stdint.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32N6_NGPIO             17

#define GPIO_PIN_SHIFT            0
#define GPIO_PIN_MASK             (15 << GPIO_PIN_SHIFT)
#define GPIO_PIN(n)               ((uint32_t)(n) << GPIO_PIN_SHIFT)
#define GPIO_PIN0                 GPIO_PIN(0)
#define GPIO_PIN1                 GPIO_PIN(1)
#define GPIO_PIN2                 GPIO_PIN(2)
#define GPIO_PIN3                 GPIO_PIN(3)
#define GPIO_PIN4                 GPIO_PIN(4)
#define GPIO_PIN5                 GPIO_PIN(5)
#define GPIO_PIN6                 GPIO_PIN(6)
#define GPIO_PIN7                 GPIO_PIN(7)
#define GPIO_PIN8                 GPIO_PIN(8)
#define GPIO_PIN9                 GPIO_PIN(9)
#define GPIO_PIN10                GPIO_PIN(10)
#define GPIO_PIN11                GPIO_PIN(11)
#define GPIO_PIN12                GPIO_PIN(12)
#define GPIO_PIN13                GPIO_PIN(13)
#define GPIO_PIN14                GPIO_PIN(14)
#define GPIO_PIN15                GPIO_PIN(15)

#define GPIO_PORT_SHIFT           4
#define GPIO_PORT_MASK            (31 << GPIO_PORT_SHIFT)
#define GPIO_PORT(n)              ((uint32_t)(n) << GPIO_PORT_SHIFT)
#define GPIO_PORTA                GPIO_PORT(0)
#define GPIO_PORTB                GPIO_PORT(1)
#define GPIO_PORTC                GPIO_PORT(2)
#define GPIO_PORTD                GPIO_PORT(3)
#define GPIO_PORTE                GPIO_PORT(4)
#define GPIO_PORTF                GPIO_PORT(5)
#define GPIO_PORTG                GPIO_PORT(6)
#define GPIO_PORTH                GPIO_PORT(7)
#define GPIO_PORTN                GPIO_PORT(13)
#define GPIO_PORTO                GPIO_PORT(14)
#define GPIO_PORTP                GPIO_PORT(15)
#define GPIO_PORTQ                GPIO_PORT(16)

#define GPIO_GPIOA                GPIO_PORTA
#define GPIO_GPIOB                GPIO_PORTB
#define GPIO_GPIOC                GPIO_PORTC
#define GPIO_GPIOD                GPIO_PORTD
#define GPIO_GPIOE                GPIO_PORTE
#define GPIO_GPIOF                GPIO_PORTF
#define GPIO_GPIOG                GPIO_PORTG
#define GPIO_GPIOH                GPIO_PORTH
#define GPIO_GPION                GPIO_PORTN
#define GPIO_GPIOO                GPIO_PORTO
#define GPIO_GPIOP                GPIO_PORTP
#define GPIO_GPIOQ                GPIO_PORTQ

#define GPIO_OUTPUT_SET           (1 << 9)
#define GPIO_OUTPUT_CLEAR         (0)

#define GPIO_OTYPE_SHIFT          10
#define GPIO_OTYPE_MASK           (1 << GPIO_OTYPE_SHIFT)
#define GPIO_PUSHPULL             (0)
#define GPIO_OPENDRAIN            GPIO_OTYPE_MASK

#define GPIO_SPEED_SHIFT          11
#define GPIO_SPEED_MASK           (3 << GPIO_SPEED_SHIFT)
#define GPIO_SPEED_LOW            (0 << GPIO_SPEED_SHIFT)
#define GPIO_SPEED_MEDIUM         (1 << GPIO_SPEED_SHIFT)
#define GPIO_SPEED_HIGH           (2 << GPIO_SPEED_SHIFT)
#define GPIO_SPEED_VERYHIGH       (3 << GPIO_SPEED_SHIFT)

#define GPIO_PUPD_SHIFT           13
#define GPIO_PUPD_MASK            (3 << GPIO_PUPD_SHIFT)
#define GPIO_FLOAT                (0 << GPIO_PUPD_SHIFT)
#define GPIO_PULLUP               (1 << GPIO_PUPD_SHIFT)
#define GPIO_PULLDOWN             (2 << GPIO_PUPD_SHIFT)

#define GPIO_AF_SHIFT             15
#define GPIO_AF_MASK              (15 << GPIO_AF_SHIFT)
#define GPIO_AF(n)                ((uint32_t)(n) << GPIO_AF_SHIFT)

#define GPIO_MODE_SHIFT           19
#define GPIO_MODE_MASK            (3 << GPIO_MODE_SHIFT)
#define GPIO_INPUT                (0 << GPIO_MODE_SHIFT)
#define GPIO_OUTPUT               (1 << GPIO_MODE_SHIFT)
#define GPIO_ALT                  (2 << GPIO_MODE_SHIFT)
#define GPIO_ANALOG               (3 << GPIO_MODE_SHIFT)

#define GPIO_PIN_SET              true
#define GPIO_PIN_RESET            false

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

void stm32n6_gpioinit(void);
int stm32n6_configgpio(uint32_t cfgset);
int stm32n6_unconfiggpio(uint32_t cfgset);
void stm32n6_gpiowrite(uint32_t pinset, bool value);
bool stm32n6_gpioread(uint32_t pinset);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_GPIO_H */
