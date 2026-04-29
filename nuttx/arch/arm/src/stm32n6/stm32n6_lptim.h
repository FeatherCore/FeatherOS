/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_lptim.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_LPTIM_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_LPTIM_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <stdint.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_LPTIM1_BASE           (STM32N6_PERIPH_BASE + 0x04000400)
#define STM32_LPTIM2_BASE           (STM32N6_PERIPH_BASE + 0x04000800)
#define STM32_LPTIM3_BASE           (STM32N6_PERIPH_BASE + 0x04000C00)
#define STM32_LPTIM4_BASE           (STM32N6_PERIPH_BASE + 0x04001000)
#define STM32_LPTIM5_BASE           (STM32N6_PERIPH_BASE + 0x04001400)

#define LPTIM_CR_ENABLE            (1 << 0)
#define LPTIM_CR_SUSPEN           (1 << 1)
#define LPTIM_CR_RSTARE           (1 << 2)

#define LPTIM_CFGR_CKSEL          (1 << 0)
#define LPTIM_CFGR_CKPOL          (1 << 1)
#define LPTIM_CFGR_CKFLT_SHIFT    2
#define LPTIM_CFGR_CKFLT_MASK     (3 << LPTIM_CFGR_CKFLT_SHIFT)
#define LPTIM_CFGR_TRGFLT_SHIFT   6
#define LPTIM_CFGR_TRGFLT_MASK    (3 << LPTIM_CFGR_TRGFLT_SHIFT)
#define LPTIM_CFGR_PRESC_SHIFT    9
#define LPTIM_CFGR_PRESC_MASK     (7 << LPTIM_CFGR_PRESC_SHIFT)
#define LPTIM_CFGR_TRIGSEL_SHIFT  13
#define LPTIM_CFGR_TRIGSEL_MASK   (7 << LPTIM_CFGR_TRIGSEL_SHIFT)
#define LPTIM_CFGR_TRIGEN_SHIFT   17
#define LPTIM_CFGR_TRIGEN_MASK    (3 << LPTIM_CFGR_TRIGEN_SHIFT)
#define LPTIM_CFGR_TIMOUT        (1 << 20)
#define LPTIM_CFGR_WAVPOL        (1 << 21)
#define LPTIM_CFGR_WAVE          (1 << 22)
#define LPTIM_CFGR_PRELOAD       (1 << 23)
#define LPTIM_CFGR_COUNTMODE      (1 << 24)
#define LPTIM_CFGR_ENCODER       (1 << 25)

#define LPTIM_ISR_CMPM           (1 << 0)
#define LPTIM_ISR_CMPOK          (1 << 1)
#define LPTIM_ISR_ARRM           (1 << 2)
#define LPTIM_ISR_ARROK          (1 << 3)
#define LPTIM_ISR_EXTTRIG        (1 << 4)
#define LPTIM_ISR_AREPU         (1 << 5)

#define LPTIM_IER_CMPMIE         (1 << 0)
#define LPTIM_IER_CMPOKIE        (1 << 1)
#define LPTIM_IER_ARRMIE        (1 << 2)
#define LPTIM_IER_ARROKIE        (1 << 3)
#define LPTIM_IER_EXTTRIGIE      (1 << 4)

/****************************************************************************
 * Public Types
 ****************************************************************************/

struct stm32n6_lptim_s
{
  uintptr_t lptimbase;
  uint32_t frequency;
  uint8_t prescaler;
  int irq;
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

int stm32n6_lptim_init(uintptr_t lptimbase, uint32_t frequency, uint8_t prescaler);
void stm32n6_lptim_enable(uintptr_t lptimbase);
void stm32n6_lptim_disable(uintptr_t lptimbase);
void stm32n6_lptim_setautoreload(uintptr_t lptimbase, uint32_t arr);
uint32_t stm32n6_lptim_getautoreload(uintptr_t lptimbase);
uint32_t stm32n6_lptim_getcounter(uintptr_t lptimbase);
void stm32n6_lptim_start(uintptr_t lptimbase);
void stm32n6_lptim_stop(uintptr_t lptimbase);
void stm32n6_lptim_int_enable(uintptr_t lptimbase, uint32_t sources);
void stm32n6_lptim_int_disable(uintptr_t lptimbase, uint32_t sources);
bool stm32n6_lptim_int_status(uintptr_t lptimbase, uint32_t source);
void stm32n6_lptim_int_ack(uintptr_t lptimbase, uint32_t sources);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_LPTIM_H */
