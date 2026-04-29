/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_tim.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_TIM_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_TIM_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <stdint.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define TIM_CR1_CEN              (1 << 0)
#define TIM_CR1_UDIS            (1 << 1)
#define TIM_CR1_URS             (1 << 2)
#define TIM_CR1_OPM             (1 << 3)
#define TIM_CR1_DIR             (1 << 4)
#define TIM_CR1_CMS_SHIFT       5
#define TIM_CR1_CMS_MASK        (3 << TIM_CR1_CMS_SHIFT)
#define TIM_CR1_ARPE            (1 << 7)
#define TIM_CR1_CKD_SHIFT       8
#define TIM_CR1_CKD_MASK        (3 << TIM_CR1_CKD_SHIFT)

#define TIM_CR2_CCPC            (1 << 0)
#define TIM_CR2_CCDS            (1 << 3)
#define TIM_CR2_MMS_SHIFT       4
#define TIM_CR2_MMS_MASK        (7 << TIM_CR2_MMS_SHIFT)
#define TIM_CR2_TI1S            (1 << 7)
#define TIM_CR2_OIS1            (1 << 8)
#define TIM_CR2_OIS1N           (1 << 9)
#define TIM_CR2_OIS2            (1 << 10)
#define TIM_CR2_OIS2N           (1 << 11)
#define TIM_CR2_OIS3            (1 << 12)
#define TIM_CR2_OIS3N           (1 << 13)
#define TIM_CR2_OIS4            (1 << 14)

#define TIM_DIER_UIE            (1 << 0)
#define TIM_DIER_CC1IE          (1 << 1)
#define TIM_DIER_CC2IE          (1 << 2)
#define TIM_DIER_CC3IE          (1 << 3)
#define TIM_DIER_CC4IE          (1 << 4)
#define TIM_DIER_COMIE          (1 << 5)
#define TIM_DIER_TIE            (1 << 6)
#define TIM_DIER_BIE            (1 << 7)
#define TIM_DIER_UDE            (1 << 8)
#define TIM_DIER_CC1DE          (1 << 9)
#define TIM_DIER_CC2DE          (1 << 10)
#define TIM_DIER_CC3DE          (1 << 11)
#define TIM_DIER_CC4DE          (1 << 12)
#define TIM_DIER_COMDE          (1 << 13)
#define TIM_DIER_TDE            (1 << 14)

#define TIM_SR_UIF              (1 << 0)
#define TIM_SR_CC1IF            (1 << 1)
#define TIM_SR_CC2IF            (1 << 2)
#define TIM_SR_CC3IF            (1 << 3)
#define TIM_SR_CC4IF            (1 << 4)
#define TIM_SR_COMIF            (1 << 5)
#define TIM_SR_TIF              (1 << 6)
#define TIM_SR_BIF              (1 << 7)
#define TIM_SR_CC1OF            (1 << 9)
#define TIM_SR_CC2OF            (1 << 10)
#define TIM_SR_CC3OF            (1 << 11)
#define TIM_SR_CC4OF            (1 << 12)

#define TIM_EGR_UG              (1 << 0)
#define TIM_EGR_CC1G            (1 << 1)
#define TIM_EGR_CC2G            (1 << 2)
#define TIM_EGR_CC3G            (1 << 3)
#define TIM_EGR_CC4G            (1 << 4)
#define TIM_EGR_COMG            (1 << 5)
#define TIM_EGR_TG              (1 << 6)
#define TIM_EGR_BG              (1 << 7)

/****************************************************************************
 * Public Types
 ****************************************************************************/

struct stm32n6_tim_s
{
  uintptr_t timbase;
  uint32_t frequency;
  uint32_t period;
  int irq;
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

int stm32n6_tim_initialize(uintptr_t timbase, uint32_t frequency,
                           uint32_t period);
void stm32n6_tim_enable(uintptr_t timbase);
void stm32n6_tim_disable(uintptr_t timbase);
void stm32n6_tim_set_period(uintptr_t timbase, uint32_t period);
uint32_t stm32n6_tim_get_counter(uintptr_t timbase);
void stm32n6_tim_clear_interrupt(uintptr_t timbase);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_TIM_H */