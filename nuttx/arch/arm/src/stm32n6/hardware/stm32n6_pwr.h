/****************************************************************************
 * arch/arm/src/stm32n6/hardware/stm32n6_pwr.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_PWR_H
#define __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_PWR_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "stm32n6_memorymap.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_PWR_CR1_OFFSET           0x00
#define STM32_PWR_CSR1_OFFSET          0x04
#define STM32_PWR_CR2_OFFSET           0x08
#define STM32_PWR_CR3_OFFSET           0x0C
#define STM32_PWR_CR4_OFFSET           0x10
#define STM32_PWR_CPUCR_OFFSET         0x14
#define STM32_PWR_D3CR_OFFSET          0x18
#define STM32_PWR_WUCR_OFFSET          0x20
#define STM32_PWR_WUSCR_OFFSET         0x24
#define STM32_PWR_WUSR_OFFSET          0x28
#define STM32_PWR_SECCFGR_OFFSET       0x30
#define STM32_PWR_PRIVCFGR_OFFSET      0x34

#define PWR_CR1_VOS_SHIFT              14
#define PWR_CR1_VOS_MASK               (3 << PWR_CR1_VOS_SHIFT)
#define PWR_CR1_VOS_SCALE0             (3 << PWR_CR1_VOS_SHIFT)
#define PWR_CR1_VOS_SCALE1             (2 << PWR_CR1_VOS_SHIFT)
#define PWR_CR1_VOS_SCALE2             (1 << PWR_CR1_VOS_SHIFT)
#define PWR_CR1_VOS_SCALE3             (0 << PWR_CR1_VOS_SHIFT)

#define PWR_CR2_VDDIO2EN               (1 << 0)
#define PWR_CR2_VDDIO3EN               (1 << 1)
#define PWR_CR2_VDDIO4EN               (1 << 2)
#define PWR_CR2_VDDIO5EN               (1 << 3)

#define PWR_CSR1_VOSRDY                (1 << 13)

#endif /* __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_PWR_H */
