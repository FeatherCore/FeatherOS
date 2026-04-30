/****************************************************************************
 * arch/arm/src/stm32u5/stm32u5xx_rcc.c
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

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <arch/stm32u5/chip.h>
#include <arch/board/board.h>

#include "stm32_pwr.h"
#include "stm32_flash.h"
#include "stm32_rcc.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Allow up to 100 milliseconds for the high speed clock to become ready.
 * that is a very long delay, but if the clock does not become ready we are
 * hosed anyway.  Normally this is very fast, but I have seen at least one
 * board that required this long, long timeout for the HSE to be ready.
 */

#define HSERDY_TIMEOUT (100 * CONFIG_BOARD_LOOPSPERMSEC)

/* Same for HSI and MSI */

#define HSIRDY_TIMEOUT HSERDY_TIMEOUT
#define MSISRDY_TIMEOUT HSERDY_TIMEOUT

/* HSE divisor to yield ~1MHz RTC clock */

#define HSE_DIVISOR (STM32_HSE_FREQUENCY + 500000) / 1000000

/* Determine if board wants to use HSI48 as 48 MHz oscillator. */

#if defined(CONFIG_STM32U5_HAVE_HSI48) && defined(STM32_USE_CLK48)
#  if STM32_CLK48_SEL == RCC_CCIPR_CLK48SEL_HSI48
#    define STM32_USE_HSI48
#  endif
#endif

/****************************************************************************
 * Private Data
 ****************************************************************************/

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: rcc_enableahb1
 *
 * Description:
 *   Enable selected AHB1 peripherals
 *
 ****************************************************************************/

static inline void rcc_enableahb1(void)
{
  uint32_t regval;

  /* Set the appropriate bits in the AHB1ENR register to enabled the clocks
   * of selected AHB1 peripherals.
   */

  regval = getreg32(STM32_RCC_AHB1ENR);

#ifdef CONFIG_STM32U5_GPDMA1
  regval |= RCC_AHB1ENR_GPDMA1EN;
#endif

#ifdef CONFIG_STM32U5_CORDIC
  regval |= RCC_AHB1ENR_CORDIC;
#endif

#ifdef CONFIG_STM32U5_FMAC
  regval |= RCC_AHB1ENR_FMACEN;
#endif

#ifdef CONFIG_STM32U5_MDF1
  regval |= RCC_AHB1ENR_MDF1EN;
#endif

#ifdef CONFIG_STM32U5_FLASH
  regval |= RCC_AHB1ENR_FLASHEN;
#endif

#ifdef CONFIG_STM32U5_CRC
  regval |= RCC_AHB1ENR_CRCEN;
#endif

#ifdef CONFIG_STM32U5_TSC
  regval |= RCC_AHB1ENR_TSCEN;
#endif

#ifdef CONFIG_STM32U5_RAMCFG
  regval |= RCC_AHB1ENR_RAMCFGEN;
#endif

#ifdef CONFIG_STM32U5_DMA2D
  regval |= RCC_AHB1ENR_DMA2DEN;
#endif

#ifdef CONFIG_STM32U5_GTZC1
  regval |= RCC_AHB1ENR_GTZC1EN;
#endif

#ifdef CONFIG_STM32U5_BKPSRAM
  regval |= RCC_AHB1ENR_BKPSRAMEN;
#endif

#ifdef CONFIG_STM32U5_DCACHE1
  regval |= RCC_AHB1ENR_DCACHE1EN;
#endif

#ifdef CONFIG_STM32U5_SRAM1
  regval |= RCC_AHB1ENR_SRAM1EN;
#endif

  putreg32(regval, STM32_RCC_AHB1ENR);   /* Enable peripherals */
}

/****************************************************************************
 * Name: rcc_enableahb2
 *
 * Description:
 *   Enable selected AHB2 peripherals
 *
 ****************************************************************************/

static inline void rcc_enableahb2(void)
{
  uint32_t regval;

  /* Set the appropriate bits in the AHB2ENR1 and AHB2ENR2 registers to
   * enable the clocks of selected AHB2 peripherals.
   */

  regval = getreg32(STM32_RCC_AHB2ENR1);

#if STM32_NPORTS > 0
  regval |= (RCC_AHB2ENR1_GPIOAEN
#if STM32_NPORTS > 1
             | RCC_AHB2ENR1_GPIOBEN
#endif
#if STM32_NPORTS > 2
             | RCC_AHB2ENR1_GPIOCEN
#endif
#if STM32_NPORTS > 3
             | RCC_AHB2ENR1_GPIODEN
#endif
#if STM32_NPORTS > 4
             | RCC_AHB2ENR1_GPIOEEN
#endif
#if STM32_NPORTS > 5
             | RCC_AHB2ENR1_GPIOFEN
#endif
#if STM32_NPORTS > 6
             | RCC_AHB2ENR1_GPIOGEN
#endif
#if STM32_NPORTS > 7
             | RCC_AHB2ENR1_GPIOHEN
#endif
#if STM32_NPORTS > 8
             | RCC_AHB2ENR1_GPIOIEN
#endif
       );
#endif

#if defined(CONFIG_STM32U5_ADC1)
  regval |= RCC_AHB2ENR1_ADC1EN;
#endif

#if defined(CONFIG_STM32U5_DCMI_PSSI)
  regval |= RCC_AHB2ENR1_DCMI_PSSIEN;
#endif

#ifdef CONFIG_STM32U5_OTGHS
  regval |= RCC_AHB2ENR1_OTGEN;
#endif

#ifdef CONFIG_STM32U5_OTGHS
  regval |= RCC_AHB2ENR1_OTGPHYEN;
#endif

#ifdef CONFIG_STM32U5_AES
  regval |= RCC_AHB2ENR1_AESEN;
#endif

#ifdef CONFIG_STM32U5_HASH
  regval |= RCC_AHB2ENR1_HASHEN
#endif

#ifdef CONFIG_STM32U5_RNG
  regval |= RCC_AHB2ENR1_RNGEN;
#endif

#ifdef CONFIG_STM32U5_PKA
  regval |= RCC_AHB2ENR_PKAEN;
#endif

#ifdef CONFIG_STM32U5_SAES
  regval |= RCC_AHB2ENR1_SAES;
#endif

#ifdef CONFIG_STM32U5_OCTOSPIM
  regval |= RCC_AHB2ENR1_OCTOSPIM;
#endif

#ifdef CONFIG_STM32U5_OTFDEC1
  regval |= RCC_AHB2ENR1_OTFDEC1;
#endif

#ifdef CONFIG_STM32U5_OTFDEC2
  regval |= RCC_AHB2ENR1_OTFDEC2;
#endif

#ifdef CONFIG_STM32U5_SDMMC1EN
  regval |= RCC_AHB2ENR1_SDMMC1EN;
#endif

#ifdef CONFIG_STM32U5_SDMMC2EN
  regval |= RCC_AHB2ENR1_SDMMC2EN;
#endif

#ifdef CONFIG_STM32U5_SRAM2
  regval |= RCC_AHB2ENR1_SRAM2EN;
#endif

#ifdef CONFIG_STM32U5_SRAM3
  regval |= RCC_AHB2ENR1_SRAM3EN;
#endif

  putreg32(regval, STM32_RCC_AHB2ENR1);

  regval = getreg32(STM32_RCC_AHB2ENR2);

#ifdef CONFIG_STM32U5_FSMC
  regval |= RCC_AHB2ENR2_FSMCEN;
#endif

#ifdef CONFIG_STM32U5_OCTOSPI1
  regval |= RCC_AHB2ENR2_OCTOSPI1EN;
#endif

#ifdef CONFIG_STM32U5_OCTOSPI2
  regval |= RCC_AHB2ENR2_OCTOSPI2EN;
#endif

#ifdef CONFIG_STM32U5_SRAM5
  regval |= RCC_AHB2ENR2_SRAM5EN;
#endif

#ifdef CONFIG_STM32U5_SRAM6
  regval |= RCC_AHB2ENR2_SRAM6EN;
#endif

  putreg32(regval, STM32_RCC_AHB2ENR2);
}

/****************************************************************************
 * Name: rcc_enableahb3
 *
 * Description:
 *   Enable selected AHB3 peripherals
 *
 ****************************************************************************/

static inline void rcc_enableahb3(void)
{
  uint32_t regval;

  /* Set the appropriate bits in the AHB3ENR register to enabled the clocks
   * of selected AHB3 peripherals.
   */

  regval = getreg32(STM32_RCC_AHB3ENR);

#ifdef CONFIG_STM32U5_LPGPIO1
  regval |= RCC_AHB3ENR_LPGPIO1EN;
#endif

#ifdef CONFIG_STM32U5_PWR
  regval |= RCC_AHB3ENR_PWREN;
#endif

#ifdef CONFIG_STM32U5_ADC4
  regval |= RCC_AHB3ENR_ADC4EN;
#endif

#ifdef CONFIG_STM32U5_DAC1
  regval |= RCC_AHB3ENR_DAC1EN;
#endif

#ifdef CONFIG_STM32U5_LPDMA1
  regval |= RCC_AHB3ENR_LPDMA1EN;
#endif

#ifdef CONFIG_STM32U5_ADF1
  regval |= RCC_AHB3ENR_ADF1EN;
#endif

#ifdef CONFIG_STM32U5_GTZC2
  regval |= RCC_AHB3ENR_GTZC2EN;
#endif

#ifdef CONFIG_STM32U5_SRAM4
  regval |= RCC_AHB3ENR_SRAM4EN;
#endif

  putreg32(regval, STM32_RCC_AHB3ENR);   /* Enable peripherals */
}

/****************************************************************************
 * Name: rcc_enableapb1
 *
 * Description:
 *   Enable selected APB1 peripherals
 *
 ****************************************************************************/

static inline void rcc_enableapb1(void)
{
  uint32_t regval;

  /* Set the appropriate bits in the APB1ENR register to enabled the clocks
   * of selected APB1 peripherals.
   */

  regval = getreg32(STM32_RCC_APB1ENR1);

#ifdef CONFIG_STM32U5_TIM2
  regval |= RCC_APB1ENR1_TIM2EN;
#endif

#ifdef CONFIG_STM32U5_TIM3
  regval |= RCC_APB1ENR1_TIM3EN;
#endif

#ifdef CONFIG_STM32U5_TIM4
  regval |= RCC_APB1ENR1_TIM4EN;
#endif

#ifdef CONFIG_STM32U5_TIM5
  regval |= RCC_APB1ENR1_TIM5EN;
#endif

#ifdef CONFIG_STM32U5_TIM6
  regval |= RCC_APB1ENR1_TIM6EN;
#endif

#ifdef CONFIG_STM32U5_TIM7
  regval |= RCC_APB1ENR1_TIM7EN;
#endif

#ifdef CONFIG_STM32U5_WWDG
  regval |= RCC_APB1ENR1_WWDGEN;
#endif

#ifdef CONFIG_STM32U5_SPI2
  regval |= RCC_APB1ENR1_SPI2EN;
#endif

#ifdef CONFIG_STM32U5_USART2
  regval |= RCC_APB1ENR1_USART2EN;
#endif

#ifdef CONFIG_STM32U5_USART3
  regval |= RCC_APB1ENR1_USART3EN;
#endif

#ifdef CONFIG_STM32U5_UART4
  regval |= RCC_APB1ENR1_UART4EN;
#endif

#ifdef CONFIG_STM32U5_UART5
  regval |= RCC_APB1ENR1_UART5EN;
#endif

#ifdef CONFIG_STM32U5_I2C1
  regval |= RCC_APB1ENR1_I2C1EN;
#endif

#ifdef CONFIG_STM32U5_I2C2
  regval |= RCC_APB1ENR1_I2C2EN;
#endif

#ifdef CONFIG_STM32U5_CRS
  regval |= RCC_APB1ENR1_CRSEN;
#endif

  putreg32(regval, STM32_RCC_APB1ENR1);   /* Enable peripherals */

  /* Second APB1 register */

  regval = getreg32(STM32_RCC_APB1ENR2);

#ifdef CONFIG_STM32U5_I2C4
  regval |= RCC_APB1ENR2_I2C4EN;
#endif

#ifdef CONFIG_STM32U5_LPTIM2
  regval |= RCC_APB1ENR2_LPTIM2EN;
#endif

#ifdef CONFIG_STM32U5_FDCAN1
  regval |= RCC_APB1ENR2_FDCAN1EN;
#endif

#ifdef CONFIG_STM32U5_UCPD1
  regval |= RCC_APB1ENR2_UCPD1EN;
#endif

  putreg32(regval, STM32_RCC_APB1ENR2);
}

/****************************************************************************
 * Name: rcc_enableapb2
 *
 * Description:
 *   Enable selected APB2 peripherals
 *
 ****************************************************************************/

static inline void rcc_enableapb2(void)
{
  uint32_t regval;

  /* Set the appropriate bits in the APB2ENR register to enabled the clocks
   * of selected APB2 peripherals.
   */

  regval = getreg32(STM32_RCC_APB2ENR);

#ifdef CONFIG_STM32U5_TIM1
  regval |= RCC_APB2ENR_TIM1EN;
#endif

#ifdef CONFIG_STM32U5_SPI1
  regval |= RCC_APB2ENR_SPI1EN;
#endif

#ifdef CONFIG_STM32U5_TIM8
  regval |= RCC_APB2ENR_TIM8EN;
#endif

#ifdef CONFIG_STM32U5_USART1
  regval |= RCC_APB2ENR_USART1EN;
#endif

#ifdef CONFIG_STM32U5_TIM15
  regval |= RCC_APB2ENR_TIM15EN;
#endif

#ifdef CONFIG_STM32U5_TIM16
  regval |= RCC_APB2ENR_TIM16EN;
#endif

#ifdef CONFIG_STM32U5_TIM17
  regval |= RCC_APB2ENR_TIM17EN;
#endif

#ifdef CONFIG_STM32U5_SAI1
  regval |= RCC_APB2ENR_SAI1EN;
#endif

#ifdef CONFIG_STM32U5_SAI2
  regval |= RCC_APB2ENR_SAI2EN;
#endif

  putreg32(regval, STM32_RCC_APB2ENR);
}

/****************************************************************************
 * Name: rcc_enableapb3
 *
 * Description:
 *   Enable selected APB3 peripherals
 *
 ****************************************************************************/

static inline void rcc_enableapb3(void)
{
  uint32_t regval;

  /* Set the appropriate bits in the APB3ENR register to enabled the clocks
   * of selected APB3 peripherals.
   */

  regval = getreg32(STM32_RCC_APB3ENR);

#ifdef CONFIG_STM32U5_SYSCFG
  regval |= RCC_APB3ENR_SYSCFGEN;
#endif

#ifdef CONFIG_STM32U5_SPI3
  regval |= RCC_APB3ENR_SPI3EN;
#endif

#ifdef CONFIG_STM32U5_LPUART1
  regval |= RCC_APB3ENR_LPUART1EN;
#endif

#ifdef CONFIG_STM32U5_I2C3EN
  regval |= RCC_APB3ENR_I2C3EN;
#endif

#ifdef CONFIG_STM32U5_LPTIM1
  regval |= RCC_APB3ENR_LPTIM1EN;
#endif

#ifdef CONFIG_STM32U5_LPTIM3
  regval |= RCC_APB3ENR_LPTIM3EN;
#endif

#ifdef CONFIG_STM32U5_LPTIM4
  regval |= RCC_APB3ENR_LPTIM4EN;
#endif

#ifdef CONFIG_STM32U5_OPAMP
  regval |= RCC_APB3ENR_OPAMPEN;
#endif

#ifdef CONFIG_STM32U5_COMP
  regval |= RCC_APB3ENR_COMPEN;
#endif

#ifdef CONFIG_STM32U5_VREF
  regval |= RCC_APB3ENR_VREFEN;
#endif

#ifdef CONFIG_STM32U5_RTCAPB
  regval |= RCC_APB3ENR_RTCAPBEN;
#endif

  putreg32(regval, STM32_RCC_APB3ENR);
}

/****************************************************************************
 * Name: rcc_enableccip
 *
 * Description:
 *   Set peripherals independent clock configuration.
 *
 ****************************************************************************/

static inline void rcc_enableccip(void)
{
  /* Certain peripherals have no clock selected even when their enable bit is
   * set. Set some defaults in the CCIPR register so those peripherals
   * will at least have a clock.
   */
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32_rcc_enableperipherals
 ****************************************************************************/

void stm32_rcc_enableperipherals(void)
{
  rcc_enableccip();
  rcc_enableahb1();
  rcc_enableahb2();
  rcc_enableahb3();
  rcc_enableapb1();
  rcc_enableapb2();
  rcc_enableapb3();
}

/****************************************************************************
 * Name: stm32_stdclockconfig
 *
 * Description:
 *   Called to change to new clock based on settings in board.h
 *
 *   NOTE:  This logic would need to be extended if you need to select low-
 *   power clocking modes!
 ****************************************************************************/

#ifndef CONFIG_ARCH_BOARD_STM32U5_CUSTOM_CLOCKCONFIG
void stm32_stdclockconfig(void)
{
  uint32_t regval;
  volatile int32_t timeout;

#if defined(STM32_BOARD_USEMSIS)
  /* Enable Internal Multi-Speed Clock (MSIS) */

  /* Wait until the MSIS is either off or ready (or until timeout elapses) */

  for (timeout = MSISRDY_TIMEOUT; timeout > 0; timeout--)
    {
      if ((regval = getreg32(STM32_RCC_CR)),
          (regval & RCC_CR_MSISRDY) || ~(regval & RCC_CR_MSISON))
        {
          /* If so, then break-out with timeout > 0 */

          break;
        }
    }

  /* setting MSISRANGE */

  putreg32((STM32_BOARD_MSISRANGE |
            STM32_BOARD_MSIKRANGE |
            RCC_ICSCR1_MSIRGSEL_ICSCR1),
           STM32_RCC_ICSCR1);

  regval  = getreg32(STM32_RCC_CR);
  regval |= RCC_CR_MSISON;
  putreg32(regval, STM32_RCC_CR);

  /* Wait until the MSI is ready (or until a timeout elapsed) */

  for (timeout = MSISRDY_TIMEOUT; timeout > 0; timeout--)
    {
      /* Check if the MSIRDY flag is the set in the CR */

      if ((getreg32(STM32_RCC_CR) & RCC_CR_MSISRDY) != 0)
        {
          /* If so, then break-out with timeout > 0 */

          break;
        }
    }

#else

    // RCC_CR         RCC clock control register

    /* Enable High-speed external Clock (HSE) */

    // RCC_CR[16]  ->  HSEON: HSE clock enable
    regval = getreg32(STM32_RCC_CR);
    regval |= RCC_CR_HSEON;
    putreg32(regval, STM32_RCC_CR);

  /* Wait until the HSE is either off or ready (or until timeout elapses) */

  for (timeout = HSERDY_TIMEOUT; timeout > 0; timeout--)
    {
      // RCC_CR[17]  ->  HSERDY: HSE clock ready flag
      if ((getreg32(STM32_RCC_CR) & RCC_CR_HSERDY) != 0)
        {
          /* If so, then break-out with timeout > 0 */

          break;
        }
    }
#endif

  /* Check for a timeout.  If this timeout occurs, then we are hosed.  We
   * have no real back-up plan, although the following logic makes it look
   * as though we do.
   */

  if (timeout > 0)
    {
      /* Select main regulator voltage range according to system clock
       * frequency.
       */

      /* Ensure Power control is enabled before modifying it. */

      stm32_pwr_enableclk(true);

      /* Generate an EPOD booster clock frequency of 4 MHz.  FIXME: This must
       * be computed based on the MSIS clock to yield a frequency between 4
       * and 16 MHz.  Also, the EPOD booster clock is required only for
       * SYSCLK frequencies greater than 55MHz.
       */

#if defined(STM32_BOARD_USEMSIS)
      regval = getreg32(STM32_RCC_PLL1CFGR);
      regval &= ~(RCC_PLL1CFGR_PLL1SRC_MASK | RCC_PLL1CFGR_PLL1MBOOST_MASK);
      regval |= RCC_PLL1CFGR_PLL1SRC_MSIS | RCC_PLL1CFGR_PLL1MBOOST_DIV_1;
      putreg32(regval, STM32_RCC_PLL1CFGR);
#else
      // RCC_PLL1CFGR     RCC PLL1 configuration register

      regval = getreg32(STM32_RCC_PLL1CFGR);
      regval &= ~(RCC_PLL1CFGR_PLL1SRC_MASK | RCC_PLL1CFGR_PLL1MBOOST_MASK);

      // RCC_PLL1CFGR[ 1: 0]  ->  PLL1SRC[ 1: 0]: PLL1 entry clock source
      // RCC_PLL1CFGR[15:12]  ->  PLL1MBOOST[3:0]: Prescaler for EPOD booster input clock
      regval |= RCC_PLL1CFGR_PLL1SRC_HSE | RCC_PLL1CFGR_PLL1MBOOST_DIV_4; // 16MHz / 4
      putreg32(regval, STM32_RCC_PLL1CFGR);
#endif

      /* Select correct main regulator range */

      // PWR_VOSR       PWR voltage scaling register

      // PWR_VOSR[17:16]    -> VOS[ 1: 0]: Voltage scaling range selection
      regval = getreg32(STM32_PWR_VOSR);
      regval &= ~PWR_VOSR_VOS_MASK;

      if (STM32_SYSCLK_FREQUENCY > 110000000)
        {
          regval |= PWR_VOSR_VOS_RANGE1;
        }
      else if (STM32_SYSCLK_FREQUENCY > 55000000)
        {
          regval |= PWR_VOSR_VOS_RANGE2;
        }
      else if (STM32_SYSCLK_FREQUENCY > 25000000)
        {
          regval |= PWR_VOSR_VOS_RANGE3;
        }
      else
        {
          regval |= PWR_VOSR_VOS_RANGE4;
        }

      // PWR_VOSR[18]       -> BOOSTEN: EPOD booster enable
      regval |= PWR_VOSR_BOOSTEN;

      putreg32(regval, STM32_PWR_VOSR);

      /* Wait for voltage regulator to stabilize */

      // PWR_VOSR[14]       -> BOOSTRDY: EPOD booster ready
      while ((getreg32(STM32_PWR_VOSR) &
              (PWR_VOSR_VOSRDY | PWR_VOSR_BOOSTRDY)) !=
             (PWR_VOSR_VOSRDY | PWR_VOSR_BOOSTRDY))
        {
        }

      /* Configure 4 wait states and prefetch for FLASH access. FIXME: Flash
       * wait states must be computed based on SYSCLK frequency.
       */

      // FLASH_ACR      FLASH access control register

      // FLASH_ACR[ 3: 0]   ->  LATENCY[3:0]: Latency
      // FLASH_ACR[ 8]      ->  PRFTEN: Prefetch enable
      regval = FLASH_ACR_LATENCY_4 | FLASH_ACR_PRFTEN;
      putreg32(regval, STM32_FLASH_ACR);

      /* Set the HCLK, PCLK1 and PCLK2 dividers */

      // RCC_CFGR2      RCC clock configuration register 2

      // RCC_CFGR2[ 3: 0]   ->  HPRE[ 3: 0]: AHB prescaler
      // RCC_CFGR2[ 6: 4]   ->  PPRE1[ 2: 0]: APB1 prescaler
      // RCC_CFGR2[10: 8]   ->  PPRE2[ 2: 0]: APB2 prescaler
      regval  = getreg32(STM32_RCC_CFGR2);
      regval &= ~(RCC_CFGR2_HPRE_MASK  |
                  RCC_CFGR2_PPRE1_MASK |
                  RCC_CFGR2_PPRE2_MASK);
      regval |= STM32_RCC_CFGR2_HPRE  |
                STM32_RCC_CFGR2_PPRE1 |
                STM32_RCC_CFGR2_PPRE2;
      putreg32(regval, STM32_RCC_CFGR2);

      /* Set the PCLK3 divider */

      // RCC_CFGR3      RCC clock configuration register 3

      // RCC_CFGR3[ 6: 4]   ->  PPRE3[ 2: 0]: APB3 prescaler
      regval  = getreg32(STM32_RCC_CFGR3);
      regval &= ~RCC_CFGR3_PPRE3_MASK;
      regval |= STM32_RCC_CFGR3_PPRE3;
      putreg32(regval, STM32_RCC_CFGR3);

#ifdef CONFIG_STM32U5_RTC_HSECLOCK

#  error stm32_stdclockconfig() currently doesn not support CONFIG_STM32U5_RTC_HSECLOCK

#endif

      /* Set the PLL1 source, dividers and multipliers */

      // RCC_PLL1DIVR     RCC PLL1 dividers register

      // RCC_PLL1DIVR[ 8: 0]  ->  PLL1N[ 8: 0]: Multiplication factor for PLL1 VCO
      // RCC_PLL1DIVR[15: 9]  ->  PLL1P[ 6: 0]: PLL1 DIVP division factor
      // RCC_PLL1DIVR[22:16]  ->  PLL1Q[ 6: 0]: PLL1 DIVQ division factor
      // RCC_PLL1DIVR[30:24]  ->  PLL1R[ 6: 0]: PLL1 DIVR division factor

      regval = STM32_RCC_PLL1DIVR_PLL1N |
               STM32_RCC_PLL1DIVR_PLL1P |
               STM32_RCC_PLL1DIVR_PLL1Q |
               STM32_RCC_PLL1DIVR_PLL1R;

      putreg32(regval, STM32_RCC_PLL1DIVR);

#if defined(STM32_BOARD_USEMSIS)
      regval = RCC_PLL1CFGR_PLL1SRC_MSIS      |
               RCC_PLL1CFGR_PLL1RGE_4_TO_8MHZ |
               STM32_RCC_PLL1CFGR_PLL1M       |
               RCC_PLL1CFGR_PLL1MBOOST_DIV_1;
#ifdef STM32_RCC_PLL1CFGR_PLL1P_ENABLED
      regval |= RCC_PLL1CFGR_PLL1PEN;
#endif
#ifdef STM32_RCC_PLL1CFGR_PLL1Q_ENABLED
      regval |= RCC_PLL1CFGR_PLL1QEN;
#endif
#ifdef STM32_RCC_PLL1CFGR_PLL1R_ENABLED
      regval |= RCC_PLL1CFGR_PLL1REN;
#endif

      putreg32(regval, STM32_RCC_PLL1CFGR);
#else

      // RCC_PLL1CFGR     RCC PLL1 configuration register

      // RCC_PLL1CFGR[ 1: 0]  ->  PLL1SRC[ 1: 0]: PLL1 entry clock source
      // RCC_PLL1CFGR[ 3: 2]  ->  PLL1RGE[ 1: 0]: PLL1 input frequency range
      // RCC_PLL1CFGR[11: 8]  ->  PLL1M[ 3: 0]：Prescaler for PLL1
      // RCC_PLL1CFGR[15:12]  ->  PLL1MBOOST[3:0]: Prescaler for EPOD booster input clock
      // RCC_PLL1CFGR[18]     ->  PLL1REN: PLL1 DIVR divider output enable
      // RCC_PLL1CFGR[17]     ->  PLL1QEN: PLL1 DIVQ divider output enable
      // RCC_PLL1CFGR[16]     ->  PLL1PEN: PLL1 DIVP divider output enable
      regval = RCC_PLL1CFGR_PLL1SRC_HSE         |
               RCC_PLL1CFGR_PLL1RGE_8_TO_16MHZ  |
               STM32_RCC_PLL1CFGR_PLL1M         |
               RCC_PLL1CFGR_PLL1MBOOST_DIV_4;
#ifdef STM32_RCC_PLL1CFGR_PLL1P_ENABLED
      regval |= RCC_PLL1CFGR_PLL1PEN;
#endif
#ifdef STM32_RCC_PLL1CFGR_PLL1Q_ENABLED
      regval |= RCC_PLL1CFGR_PLL1QEN;
#endif
#ifdef STM32_RCC_PLL1CFGR_PLL1R_ENABLED
      regval |= RCC_PLL1CFGR_PLL1REN;
#endif

      putreg32(regval, STM32_RCC_PLL1CFGR);
#endif

      /* Enable PLL1 */

      // RCC_CR[24]  ->  PLL1ON: PLL1 enable
      regval  = getreg32(STM32_RCC_CR);
      regval |= RCC_CR_PLL1ON;
      putreg32(regval, STM32_RCC_CR);

      /* Wait until PLL1 is ready */

      // RCC_CR[25]  ->  PLL1RDY: PLL1 clock ready flag
      while ((getreg32(STM32_RCC_CR) & RCC_CR_PLL1RDY) == 0)
        {
        }

      /* Select the PLL1 as system clock source */

      // RCC_CFGR1        RCC clock configuration register 1

      // RCC_CFGR1[ 1: 0]     ->  SW[ 3: 0] :system clock switch

      regval  = getreg32(STM32_RCC_CFGR1);
      regval &= ~RCC_CFGR1_SW_MASK;
      regval |= RCC_CFGR1_SW_PLL;
      putreg32(regval, STM32_RCC_CFGR1);

      /* Wait until PLL1 source is used as the system clock source */

      while ((getreg32(STM32_RCC_CFGR1) & RCC_CFGR1_SWS_MASK) !=
             RCC_CFGR1_SWS_PLL)
        {
        }

#if defined(CONFIG_STM32U5_IWDG) || defined(CONFIG_STM32U5_RTC_LSICLOCK)
      /* Low speed internal clock source LSI */

      stm32_rcc_enablelsi();
#endif

#if defined(STM32_USE_LSE)
      /* Low speed external clock source LSE
       *
       * TODO: There is another case where the LSE needs to
       * be enabled: if the MCO1 pin selects LSE as source.
       * XXX and other cases, like automatic trimming of MSI for USB use
       */

      /* ensure Power control is enabled since it is indirectly required
       * to alter the LSE parameters.
       */

      stm32_pwr_enableclk(true);

      /* XXX other LSE settings must be made before turning on the oscillator
       * and we need to ensure it is first off before doing so.
       */

      /* Turn on the LSE oscillator
       * XXX this will almost surely get moved since we also want to use
       * this for automatically trimming MSI, etc.
       */

      stm32_rcc_enablelse();

#  if defined(STM32_BOARD_USEMSI)
      /* Now that LSE is up, auto trim the MSI */

      regval  = getreg32(STM32_RCC_CR);
      regval |= RCC_CR_MSIPLLEN;
      putreg32(regval, STM32_RCC_CR);
#  endif
#endif /* STM32_USE_LSE */
    }
}
#endif

/****************************************************************************
 * Clock Frequency Query Functions
 ****************************************************************************/

/* MSI frequency lookup table (in Hz) */
static const uint32_t g_msi_freq_table[] =
{
  48000000,   /* 0x0: 48 MHz */
  24000000,   /* 0x1: 24 MHz */
  16000000,   /* 0x2: 16 MHz */
  12000000,   /* 0x3: 12 MHz */
  4000000,    /* 0x4: 4 MHz */
  2000000,    /* 0x5: 2 MHz */
  1330000,    /* 0x6: 1.33 MHz (approximate) */
  1000000,    /* 0x7: 1 MHz */
  3072000,    /* 0x8: 3.072 MHz */
  1536000,    /* 0x9: 1.536 MHz */
  1024000,    /* 0xA: 1.024 MHz */
  768000,     /* 0xB: 768 kHz */
  512000,     /* 0xC: 512 kHz */
  384000,     /* 0xD: 384 kHz */
  256000,     /* 0xE: 256 kHz */
  192000      /* 0xF: 192 kHz */
};

/****************************************************************************
 * Name: stm32_get_msis_frequency
 *
 * Description:
 *   Get the frequency of the MSIS clock in Hz.
 *
 ****************************************************************************/

uint32_t stm32_get_msis_frequency(void)
{
  uint32_t regval;
  uint32_t range;

  regval = getreg32(STM32_RCC_ICSCR1);

  /* Check if MSIS range selection is enabled */

  if (regval & RCC_ICSCR1_MSIRGSEL_ICSCR1)
    {
      /* Use MSISRANGE */

      range = (regval & RCC_ICSCR1_MSISRANGE_MASK) >> RCC_ICSCR1_MSISRANGE_SHIFT;
    }
  else
    {
      /* Use MSISRANGE after standby (not typically used in run mode) */

      range = (regval & RCC_ICSCR1_MSISRANGE_MASK) >> RCC_ICSCR1_MSISRANGE_SHIFT;
    }

  if (range < sizeof(g_msi_freq_table) / sizeof(g_msi_freq_table[0]))
    {
      return g_msi_freq_table[range];
    }

  return 0;
}

/****************************************************************************
 * Name: stm32_get_msik_frequency
 *
 * Description:
 *   Get the frequency of the MSIK clock in Hz.
 *
 ****************************************************************************/

uint32_t stm32_get_msik_frequency(void)
{
  uint32_t regval;
  uint32_t range;

  regval = getreg32(STM32_RCC_ICSCR1);
  range = (regval & RCC_ICSCR1_MSIKRANGE_MASK) >> RCC_ICSCR1_MSIKRANGE_SHIFT;

  if (range < sizeof(g_msi_freq_table) / sizeof(g_msi_freq_table[0]))
    {
      return g_msi_freq_table[range];
    }

  return 0;
}

/****************************************************************************
 * Name: stm32_get_pllsrc_frequency
 *
 * Description:
 *   Get the frequency of the PLL source clock in Hz.
 *
 ****************************************************************************/

uint32_t stm32_get_pllsrc_frequency(uint32_t pll_id)
{
  uint32_t regval;
  uint32_t src;

  if (pll_id == 1)
    {
      regval = getreg32(STM32_RCC_PLL1CFGR);
      src = regval & RCC_PLL1CFGR_PLL1SRC_MASK;

      switch (src)
        {
          case RCC_PLL1CFGR_PLL1SRC_HSI:
            return STM32_HSI_FREQUENCY;

          case RCC_PLL1CFGR_PLL1SRC_HSE:
            return STM32_HSE_FREQUENCY;

          case RCC_PLL1CFGR_PLL1SRC_MSIS:
            return stm32_get_msis_frequency();

          default:
            return 0;
        }
    }
  else if (pll_id == 2)
    {
      regval = getreg32(STM32_RCC_PLL2CFGR);
      src = regval & RCC_PLL2CFGR_PLL2SRC_MASK;

      switch (src)
        {
          case RCC_PLL2CFGR_PLL2SRC_HSI:
            return STM32_HSI_FREQUENCY;

          case RCC_PLL2CFGR_PLL2SRC_HSE:
            return STM32_HSE_FREQUENCY;

          case RCC_PLL2CFGR_PLL2SRC_MSIS:
            return stm32_get_msis_frequency();

          default:
            return 0;
        }
    }
  else if (pll_id == 3)
    {
      regval = getreg32(STM32_RCC_PLL3CFGR);
      src = regval & RCC_PLL3CFGR_PLL3SRC_MASK;

      switch (src)
        {
          case RCC_PLL3CFGR_PLL3SRC_HSI:
            return STM32_HSI_FREQUENCY;

          case RCC_PLL3CFGR_PLL3SRC_HSE:
            return STM32_HSE_FREQUENCY;

          case RCC_PLL3CFGR_PLL3SRC_MSIS:
            return stm32_get_msis_frequency();

          default:
            return 0;
        }
    }

  return 0;
}

/****************************************************************************
 * Name: stm32_get_pllout_frequency
 *
 * Description:
 *   Calculate the output frequency of a PLL.
 *
 ****************************************************************************/

uint32_t stm32_get_pllout_frequency(uint32_t pllsrc_freq,
                                   uint32_t pllm_div,
                                   uint32_t plln_mul,
                                   uint32_t plln_frac,
                                   uint32_t pllout_div)
{
  uint64_t vco_freq;
  uint32_t output_freq;

  if (pllm_div == 0 || pllout_div == 0)
    {
      return 0;
    }

  /* Calculate VCO frequency with fractional N support
   * Formula: Fvco = (Fsrc / M) * (N + FRACN / 8192)
   */

  vco_freq = (uint64_t)pllsrc_freq * ((uint64_t)plln_mul * 8192 + plln_frac);
  vco_freq /= (uint64_t)pllm_div * 8192;

  /* Calculate output frequency */

  output_freq = (uint32_t)(vco_freq / pllout_div);

  return output_freq;
}

/****************************************************************************
 * Name: stm32_get_sysclk_frequency
 *
 * Description:
 *   Get the system clock frequency in Hz.
 *
 ****************************************************************************/

uint32_t stm32_get_sysclk_frequency(void)
{
  uint32_t regval;
  uint32_t src;
  uint32_t pllsrc_freq;
  uint32_t pllm_div;
  uint32_t plln_mul;
  uint32_t plln_frac;
  uint32_t pllr_div;

  regval = getreg32(STM32_RCC_CFGR1);
  src = regval & RCC_CFGR1_SWS_MASK;

  switch (src)
    {
      case RCC_CFGR1_SWS_HSI:
        return STM32_HSI_FREQUENCY;

      case RCC_CFGR1_SWS_HSE:
        return STM32_HSE_FREQUENCY;

      case RCC_CFGR1_SWS_MSIS:
        return stm32_get_msis_frequency();

      case RCC_CFGR1_SWS_PLL:
        /* Get PLL1 configuration */

        pllsrc_freq = stm32_get_pllsrc_frequency(1);

        regval = getreg32(STM32_RCC_PLL1CFGR);
        pllm_div = ((regval & RCC_PLL1CFGR_PLL1M_MASK) >> RCC_PLL1CFGR_PLL1M_SHIFT) + 1;

        regval = getreg32(STM32_RCC_PLL1DIVR);
        plln_mul = (regval & RCC_PLL1DIVR_PLL1N_MASK) >> RCC_PLL1DIVR_PLL1N_SHIFT;
        pllr_div = ((regval & RCC_PLL1DIVR_PLL1R_MASK) >> RCC_PLL1DIVR_PLL1R_SHIFT) + 1;

        /* Get fractional N value if enabled */

        regval = getreg32(STM32_RCC_PLL1CFGR);
        if (regval & RCC_PLL1CFGR_PLL1FRACEN)
          {
            plln_frac = getreg32(STM32_RCC_PLL1FRACR) & RCC_PLL1FRACR_PLL1FRACN_MASK;
          }
        else
          {
            plln_frac = 0;
          }

        return stm32_get_pllout_frequency(pllsrc_freq, pllm_div, plln_mul,
                                          plln_frac, pllr_div);

      default:
        return 0;
    }
}
/****************************************************************************
 * Name: stm32_set_epod_booster
 *
 * Description:
 *   Configure the EPOD booster for high-frequency operation.
 *   The EPOD booster clock frequency should be between 4 and 16 MHz.
 *   This function sets the PLL1MBOOST prescaler and enables the booster.
 *
 * Input Parameters:
 *   pllsrc_freq - PLL1 source clock frequency in Hz
 *   pllm_div    - PLL1 M divider value
 *
 ****************************************************************************/

void stm32_set_epod_booster(uint32_t pllsrc_freq, uint32_t pllm_div)
{
  uint32_t epod_freq;
  uint32_t regval;
  volatile int timeout;

  /* Calculate EPOD clock frequency: F_epod = F_pllsrc / M / prescaler */

  if (pllm_div == 0)
    {
      return;
    }

  /* Reset EPOD Prescaler and disable booster first */

  regval = getreg32(STM32_RCC_PLL1CFGR);
  regval &= ~RCC_PLL1CFGR_PLL1MBOOST_MASK;
  regval |= RCC_PLL1CFGR_PLL1MBOOST_DIV_1;
  putreg32(regval, STM32_RCC_PLL1CFGR);

  regval = getreg32(STM32_PWR_VOSR);
  regval &= ~PWR_VOSR_BOOSTEN;
  putreg32(regval, STM32_PWR_VOSR);

  /* Wait for booster to be disabled */

  for (timeout = 1000; timeout > 0; timeout--)
    {
      if ((getreg32(STM32_PWR_VOSR) & PWR_VOSR_BOOSTRDY) == 0)
        {
          break;
        }
      up_udelay(1);
    }

  /* Only configure for system clock > 55 MHz */

  if (STM32_SYSCLK_FREQUENCY <= 55000000)
    {
      return;
    }

  /* Calculate appropriate prescaler to get EPOD clock between 4-16 MHz */

  epod_freq = pllsrc_freq / pllm_div;

  /* Find the best prescaler value */

  if (epod_freq <= 16000000)
    {
      /* No prescaler needed, EPOD clock is already <= 16 MHz */

      regval = getreg32(STM32_RCC_PLL1CFGR);
      regval &= ~RCC_PLL1CFGR_PLL1MBOOST_MASK;
      regval |= RCC_PLL1CFGR_PLL1MBOOST_DIV_1;
      putreg32(regval, STM32_RCC_PLL1CFGR);
    }
  else
    {
      uint32_t div = (epod_freq + 16000000 - 1) / 16000000;

      /* Limit to maximum prescaler value */

      if (div > 16)
        {
          div = 16;
        }

      regval = getreg32(STM32_RCC_PLL1CFGR);
      regval &= ~RCC_PLL1CFGR_PLL1MBOOST_MASK;
      regval |= (div << RCC_PLL1CFGR_PLL1MBOOST_SHIFT);
      putreg32(regval, STM32_RCC_PLL1CFGR);
    }

  /* Enable EPOD booster */

  regval = getreg32(STM32_PWR_VOSR);
  regval |= PWR_VOSR_BOOSTEN;
  putreg32(regval, STM32_PWR_VOSR);

  /* Wait for booster ready */

  for (timeout = 1000; timeout > 0; timeout--)
    {
      if ((getreg32(STM32_PWR_VOSR) & PWR_VOSR_BOOSTRDY) != 0)
        {
          break;
        }
      up_udelay(1);
    }

  DEBUGASSERT(timeout > 0);
}
