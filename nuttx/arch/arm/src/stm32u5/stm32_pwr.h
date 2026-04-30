/****************************************************************************
 * arch/arm/src/stm32u5/stm32_pwr.h
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

#ifndef __ARCH_ARM_SRC_STM32U5_STM32_PWR_H
#define __ARCH_ARM_SRC_STM32U5_STM32_PWR_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <stdbool.h>

#include "chip.h"
#include "hardware/stm32_pwr.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#ifndef __ASSEMBLY__

#undef EXTERN
#if defined(__cplusplus)
#define EXTERN extern "C"
extern "C"
{
#else
#define EXTERN extern
#endif

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: enableclk
 *
 * Description:
 *   Enable/disable the clock to the power control peripheral.  Enabling must
 *   be done after the APB1 clock is validly configured, and prior to using
 *   any functionality controlled by the PWR block (i.e. much of anything
 *   else provided by this module).
 *
 * Input Parameters:
 *   enable - True: enable the clock to the Power control (PWR) block.
 *
 * Returned Value:
 *   True:  the PWR block was previously enabled.
 *
 ****************************************************************************/

bool stm32_pwr_enableclk(bool enable);

/****************************************************************************
 * Name: stm32_pwr_enablebkp
 *
 * Description:
 *   Enables write access to the backup domain (RTC registers, RTC backup
 *   data registers and backup SRAM). Compare [RM0456], section 10.4.7
 *   Battery Backup domain, Backup domain access.
 *
 * Input Parameters:
 *   writable - True: enable ability to write to backup domain registers
 *
 * Returned Value:
 *   True: The backup domain was previously writable.
 *
 ****************************************************************************/

bool stm32_pwr_enablebkp(bool writable);

/****************************************************************************
 * Name stm32_pwr_adjustvcore
 *
 * Description:
 *   Adjusts the voltage used for digital peripherals (V_CORE) before
 *   raising or after decreasing the system clock frequency.  Compare
 *   [RM0456], section 10.5.4 Dynamic voltage scaling management.
 *
 * Input Parameters:
 *   sysclock - The frequency in Hertz the system clock will be raised to.
 *
 ****************************************************************************/

void stm32_pwr_adjustvcore(unsigned sysclock);

/****************************************************************************
 * Name stm32_pwr_enable_smps
 *
 * Description:
 *   Select between the Low-Drop Out (LDO) or Switched Mode Power Supply
 *   (SMPS) regulator.  Compare [RM0456], section 10.5.1 SMPS and LDO
 *   embedded regulators.
 *
 * Input Parameters:
 *   enable - If true, the SMPS regulator will be enabled, otherwise the LDO,
 *
 ****************************************************************************/

void stm32_pwr_enablesmps(bool enable);

/****************************************************************************
 * Name: stm32_pwr_enter_stop_mode
 *
 * Description:
 *   Enter specified STOP mode. In Stop mode, the HSI16 and HSE oscillators are 
 *   stopped. The PLL, the HSI16 and the HSE RC oscillators can be retained in 
 *   low-power mode to save startup time.
 *
 * Input Parameters:
 *   stop_mode - Which STOP mode to enter (0-3)
 *
 ****************************************************************************/

void stm32_pwr_enter_stop_mode(uint8_t stop_mode);

/****************************************************************************
 * Name: stm32_pwr_enter_standby_mode
 *
 * Description:
 *   Enter STANDBY mode. In Standby mode, the PLL, the HSI16 RC and the HSE 
 *   crystal oscillators are stopped. The voltage regulator is disabled and 
 *   the V_CORE supply domain is consequently powered off. RTC registers and 
 *   backup registers are still powered from the V_BAT supply.
 *
 ****************************************************************************/

void stm32_pwr_enter_standby_mode(void);

/****************************************************************************
 * Name: stm32_pwr_set_wakeup_clock
 *
 * Description:
 *   Set the wakeup clock selection after wake from Stop mode.
 *
 * Input Parameters:
 *   clk_source - Wakeup clock source (refer to reference manual)
 *
 ****************************************************************************/

void stm32_pwr_set_wakeup_clock(uint32_t clk_source);

/****************************************************************************
 * Name: stm32_pwr_enable_sram_retention
 *
 * Description:
 *   Enable SRAM retention in Stop 3 and Standby modes.
 *
 * Input Parameters:
 *   sram_bitmap - Bitmap of SRAM banks to retain (bit 0: SRAM2 page 1, bit 1: SRAM2 page 2)
 *
 ****************************************************************************/

void stm32_pwr_enable_sram_retention(uint8_t sram_bitmap);

# undef EXTERN
# if defined(__cplusplus)
}
# endif

#endif /* __ASSEMBLY__ */
#endif /* __ARCH_ARM_SRC_STM32U5_STM32_PWR_H */
