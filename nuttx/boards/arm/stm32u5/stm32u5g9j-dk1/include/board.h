/****************************************************************************
 * boards/arm/stm32u5/stm32u5g9j-dk1/include/board.h
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

#ifndef __BOARDS_ARM_STM32U5_STM32U5G9J_DK1_INCLUDE_BOARD_H
#define __BOARDS_ARM_STM32U5_STM32U5G9J_DK1_INCLUDE_BOARD_H

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

/* Clocking *****************************************************************/

/* The STM32U5G9J-DK1 board supports both HSE and LSE crystals (X1 and X2).
 *   X1:  16 MHz oscillator for STM32U5G9NJH6Q microcontroller
 *   X2:  32.768 KHz crystal for STM32U5G9NJH6Q embedded RTC
 *
 *   System Clock source : PLL (HSE)    16MHz
 *   SYSCLK(Hz)          : 160000000    Determined by PLL configuration
 *   HCLK(Hz)            : 160000000
 *   AHB Prescaler       : 1            (STM32_RCC_CFGR2_HPRE)  (160MHz)
 *   APB1 Prescaler      : 1            (STM32_RCC_CFGR2_PPRE1) (160MHz)
 *   APB2 Prescaler      : 1            (STM32_RCC_CFGR2_PPRE2) (160MHz)
 *   APB3 Prescaler      : 1            (STM32_RCC_CFGR3_PPRE3) (160MHz)
 *   MSIS Frequency(Hz)  : 4000000      (nominal)
 *   MSIK Frequency(Hz)  : 4000000      (nominal)
 *   PLL_MBOOST          : 1            (Embedded power distribution booster)
 *   PLLM                : 1            (STM32_PLLCFG_PLLM)
 *   PLLN                : 10           (STM32_PLLCFG_PLLN)
 *   PLLP                : 2            (STM32_PLLCFG_PLLP)
 *   PLLQ                : 2            (STM32_PLLCFG_PLLQ)
 *   PLLR                : 1            (STM32_PLLCFG_PLLR)
 *   Flash Latency(WS)   : 4
 */

/* HSI - 16 MHz RC factory-trimmed
 * LSI - 32 KHz RC
 * MSI - 4 MHz, autotrimmed via LSE
 * HSE - 16 MHz installed
 * LSE - 32.768 kHz installed
 */

#define STM32_BOARD_XTAL        16000000ul

#define STM32_HSI_FREQUENCY     16000000ul
#define STM32_LSI_FREQUENCY     32000
#define STM32_HSE_FREQUENCY     STM32_BOARD_XTAL
#define STM32_LSE_FREQUENCY     32768

/* Enable HSE */

#define STM32_USE_HSE           1

#ifndef STM32_USE_HSE
#define STM32_BOARD_USEMSIS     1
#define STM32_BOARD_MSISRANGE   RCC_ICSCR1_MSISRANGE_4MHZ
#define STM32_BOARD_MSIKRANGE   RCC_ICSCR1_MSIKRANGE_4MHZ
#endif // !STM32_USE_HSE

/* PLL1 config; we use this to generate our system clock */

// RCC_PLL1CFGR     RCC PLL1 configuration register
// RCC_PLL1DIVR     RCC PLL1 dividers register

// RCC_PLL1CFGR[11: 8]  ->  PLL1M[ 3: 0]：Prescaler for PLL1
#define STM32_RCC_PLL1CFGR_PLL1M          RCC_PLL1CFGR_PLL1M(1)
// RCC_PLL1DIVR[ 8: 0]  ->  PLL1N[ 8: 0]: Multiplication factor for PLL1 VCO
#define STM32_RCC_PLL1DIVR_PLL1N          RCC_PLL1DIVR_PLL1N(10)
// RCC_PLL1DIVR[15: 9]  ->  PLL1P[ 6: 0]: PLL1 DIVP division factor
#define STM32_RCC_PLL1DIVR_PLL1P          0
// RCC_PLL1CFGR[16]     ->  PLL1PEN: PLL1 DIVP divider output disable
#undef  STM32_RCC_PLL1CFGR_PLL1P_ENABLED
// RCC_PLL1DIVR[22:16]  ->  PLL1Q[ 6: 0]: PLL1 DIVQ division factor
#define STM32_RCC_PLL1DIVR_PLL1Q          0
// RCC_PLL1CFGR[17]     ->  PLL1QEN: PLL1 DIVQ divider output disable
#undef  STM32_RCC_PLL1CFGR_PLL1Q_ENABLED
// RCC_PLL1DIVR[30:24]  ->  PLL1R[ 6: 0]: PLL1 DIVR division factor
#define STM32_RCC_PLL1DIVR_PLL1R          RCC_PLL1DIVR_PLL1R(1)
// RCC_PLL1CFGR[18]     ->  PLL1REN: PLL1 DIVR divider output enable
#define STM32_RCC_PLL1CFGR_PLL1R_ENABLED

#define STM32_SYSCLK_FREQUENCY  160000000ul

/* Enable LSE (for the RTC and for MSIS autotrimming) */

#define STM32_USE_LSE           1

/* Configure the HCLK divisor (for the AHB bus, core, memory, and DMA */

// RCC_CFGR2        RCC clock configuration register 2

// RCC_CFGR2[ 3: 0]     ->  HPRE[ 3: 0] :AHB prescaler
#define STM32_RCC_CFGR2_HPRE    RCC_CFGR2_HPRE_SYSCLK     /* HCLK  = SYSCLK / 1 */

#define STM32_HCLK_FREQUENCY    STM32_SYSCLK_FREQUENCY

/* Configure the APB1 prescaler */

// RCC_CFGR2[ 6: 4]     ->  PPRE1[ 2: 0] :APB1 prescaler

#define STM32_RCC_CFGR2_PPRE1   RCC_CFGR2_PPRE1_HCLK      /* PCLK1 = HCLK / 1 */
#define STM32_PCLK1_FREQUENCY   (STM32_HCLK_FREQUENCY / 1)

#define STM32_APB1_TIM2_CLKIN   (STM32_PCLK1_FREQUENCY)
#define STM32_APB1_TIM3_CLKIN   (STM32_PCLK1_FREQUENCY)
#define STM32_APB1_TIM4_CLKIN   (STM32_PCLK1_FREQUENCY)
#define STM32_APB1_TIM5_CLKIN   (STM32_PCLK1_FREQUENCY)
#define STM32_APB1_TIM6_CLKIN   (STM32_PCLK1_FREQUENCY)
#define STM32_APB1_TIM7_CLKIN   (STM32_PCLK1_FREQUENCY)

/* Configure the APB2 prescaler */

// RCC_CFGR2[10: 8]     ->  PPRE2[ 2: 0] :APB2 prescaler

#define STM32_RCC_CFGR2_PPRE2   RCC_CFGR2_PPRE2_HCLK       /* PCLK2 = HCLK / 1 */
#define STM32_PCLK2_FREQUENCY   (STM32_HCLK_FREQUENCY / 1)

#define STM32_APB2_TIM1_CLKIN   (STM32_PCLK2_FREQUENCY)
#define STM32_APB2_TIM15_CLKIN  (STM32_PCLK2_FREQUENCY)
#define STM32_APB2_TIM16_CLKIN  (STM32_PCLK2_FREQUENCY)

/* Configure the APB3 prescaler */

// RCC_CFGR3        RCC clock configuration register 3

// RCC_CFGR3[ 6: 4]     ->  PPRE3[ 2: 0] :APB3 prescaler

#define STM32_RCC_CFGR3_PPRE3   RCC_CFGR3_PPRE3_HCLK       /* PCLK3 = HCLK / 1 */
#define STM32_PCLK3_FREQUENCY   (STM32_HCLK_FREQUENCY / 1)

/* The timer clock frequencies are automatically defined by hardware.  If the
 * APB prescaler equals 1, the timer clock frequencies are set to the same
 * frequency as that of the APB domain. Otherwise they are set to twice.
 * Note: TIM1,15,16 are on APB2, others on APB1
 */

#define BOARD_TIM1_FREQUENCY    STM32_HCLK_FREQUENCY
#define BOARD_TIM2_FREQUENCY    STM32_HCLK_FREQUENCY
#define BOARD_TIM3_FREQUENCY    STM32_HCLK_FREQUENCY
#define BOARD_TIM4_FREQUENCY    STM32_HCLK_FREQUENCY
#define BOARD_TIM5_FREQUENCY    STM32_HCLK_FREQUENCY
#define BOARD_TIM6_FREQUENCY    STM32_HCLK_FREQUENCY
#define BOARD_TIM7_FREQUENCY    STM32_HCLK_FREQUENCY
#define BOARD_TIM15_FREQUENCY   STM32_HCLK_FREQUENCY
#define BOARD_TIM16_FREQUENCY   STM32_HCLK_FREQUENCY
#define BOARD_LPTIM1_FREQUENCY  STM32_HCLK_FREQUENCY
#define BOARD_LPTIM2_FREQUENCY  STM32_HCLK_FREQUENCY

/* DMA Channel/Stream Selections ********************************************/

/* Alternate function pin selections ****************************************/

/* USART1: Connected to STLink VCP and to CN10 with small rework of pcb. */

#define GPIO_USART1_RX   GPIO_USART1_RX_1    /* PA10 */
#define GPIO_USART1_TX   GPIO_USART1_TX_1    /* PA9  */

/****************************************************************************
 * Public Data
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
 * Name: stm32_board_initialize
 *
 * Description:
 *   All STM32 architectures must provide the following entry point.
 *   This entry point is called early in the initialization -- after all
 *   memory has been configured and mapped but before any devices
 *   have been initialized.
 *
 ****************************************************************************/

void stm32_board_initialize(void);

#undef EXTERN
#if defined(__cplusplus)
}
#endif

#endif /* __ASSEMBLY__ */
#endif  /* __BOARDS_ARM_STM32U5_STM32U5G9J_DK1_INCLUDE_BOARD_H */
