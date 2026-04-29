/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_pwr.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_PWR_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_PWR_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <stdint.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define PWR_CR1_LPMS_SHIFT          0
#define PWR_CR1_LPMS_MASK           (7 << PWR_CR1_LPMS_SHIFT)
#define PWR_CR1_LPMS_STOP0          (0 << PWR_CR1_LPMS_SHIFT)
#define PWR_CR1_LPMS_STOP1          (1 << PWR_CR1_LPMS_SHIFT)
#define PWR_CR1_LPMS_STOP2          (2 << PWR_CR1_LPMS_SHIFT)
#define PWR_CR1_LPMS_STANDBY        (3 << PWR_CR1_LPMS_SHIFT)
#define PWR_CR1_LPMS_SHUTDOWN       (4 << PWR_CR1_LPMS_SHIFT)

#define PWR_CR1_FPDR               (1 << 4)
#define PWR_CR1_FPDS               (1 << 5)
#define PWR_CR1_DBP                (1 << 8)
#define PWR_CR1_VOS_SHIFT          14
#define PWR_CR1_VOS_MASK           (3 << PWR_CR1_VOS_SHIFT)
#define PWR_CR1_VOS_SCALE0         (3 << PWR_CR1_VOS_SHIFT)
#define PWR_CR1_VOS_SCALE1         (2 << PWR_CR1_VOS_SHIFT)
#define PWR_CR1_VOS_SCALE2         (1 << PWR_CR1_VOS_SHIFT)
#define PWR_CR1_VOS_SCALE3         (0 << PWR_CR1_VOS_SHIFT)

#define PWR_CR2_IOSV               (1 << 9)
#define PWR_CR2_USV                (1 << 10)

#define PWR_CR3_LPR                (1 << 16)

#define PWR_SCR_CSBF               (1 << 3)
#define PWR_SCR_CWUF1              (1 << 0)
#define PWR_SCR_CWUF2              (1 << 1)
#define PWR_SCR_CWUF3              (1 << 2)

#define PWR_SR1_WUFI               (1 << 15)
#define PWR_SR1_SBF                (1 << 8)
#define PWR_SR1_WUF3               (1 << 2)
#define PWR_SR1_WUF2               (1 << 1)
#define PWR_SR1_WUF1               (1 << 0)

#define PWR_WUCR1_WUPF1            (1 << 0)
#define PWR_WUCR1_WUPF2            (1 << 1)
#define PWR_WUCR1_WUPF3            (1 << 2)

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

void stm32n6_pwr_init(void);
void stm32n6_pwr_set_voltage_scaling(uint8_t scale);
void stm32n6_pwr_enter_stop_mode(uint8_t mode);
void stm32n6_pwr_enter_standby_mode(void);
void stm32n6_pwr_clear_standby_flag(void);
void stm32n6_pwr_clear_wakeup_flags(void);
void stm32n6_pwr_enable_backup_domain_access(bool enable);
bool stm32n6_pwr_is_backup_domain_access_enabled(void);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_PWR_H */