/****************************************************************************
 * arch/arm/src/stm32u5/stm32u5_lptim.h
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

#ifndef __ARCH_ARM_SRC_STM32U5_STM32U5_LPTIM_H
#define __ARCH_ARM_SRC_STM32U5_STM32U5_LPTIM_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* LPTIM device identification */

#define STM32U5_LPTIM1   1
#define STM32U5_LPTIM2   2
#define STM32U5_LPTIM3   3
#define STM32U5_LPTIM4   4

/* LPTIM clock sources */

#define LPTIM_CLKSRC_APBCLOCK      0  /* APB clock */
#define LPTIM_CLKSRC_LSE           1  /* LSE (32.768 kHz) */
#define LPTIM_CLKSRC_LSI           2  /* LSI */
#define LPTIM_CLKSRC_APBCLOCK_RMP  3  /* APB clock remapped */

/* LPTIM clock prescaler */

#define LPTIM_PRESC_DIV1      0
#define LPTIM_PRESC_DIV2      1
#define LPTIM_PRESC_DIV4      2
#define LPTIM_PRESC_DIV8      3
#define LPTIM_PRESC_DIV16     4
#define LPTIM_PRESC_DIV32     5
#define LPTIM_PRESC_DIV64     6
#define LPTIM_PRESC_DIV128    7

/* LPTIM trigger sources */

#define LPTIM_TRIGSRC_GPIO      0  /* GPIO */
#define LPTIM_TRIGSRC_RTCAL0   1  /* RTC Alarm */
#define LPTIM_TRIGSRC_RTCTAMP0 2  /* RTC Tamper 0 */
#define LPTIM_TRIGSRC_COMP1    3  /* COMP1 */
#define LPTIM_TRIGSRC_COMP2    4  /* COMP2 */

/* LPTIM operating modes */

#define LPTIM_MODE_CONTINUOUS   0  /* Continuous mode */
#define LPTIM_MODE_SINGLE      1  /* Single mode */

/* LPTIM clock polarity */

#define LPTIM_POL_RISING        0  /* Rising edge */
#define LPTIM_POL_FALLING       1  /* Falling edge */
#define LPTIM_POL_BOTH          2  /* Both edges */

/****************************************************************************
 * Public Types
 ****************************************************************************/

#ifndef __ASSEMBLY__

enum stm32u5_lptim_mode_e
{
  LPTIM_MODE_UNUSED = 0,
  LPTIM_MODE_COUNTER,
  LPTIM_MODE_PWM,
  LPTIM_MODE_PULSE,
  LPTIM_MODE_ENCODER
};

enum stm32u5_lptim_clksrc_e
{
  LPTIM_CLKSRC_INTERNAL = 0,
  LPTIM_CLKSRC_EXTERNAL
};

enum stm32u5_lptim_pol_e
{
  LPTIM_POL_RISING = 0,
  LPTIM_POL_FALLING,
  LPTIM_POL_BOTH
};

enum stm32u5_lptim_trig_e
{
  LPTIM_TRIG_SOFTWARE = 0,
  LPTIM_TRIG_INTERNAL,
  LPTIM_TRIG_EXTERNAL
};

struct stm32u5_lptim_dev_s
{
  uint8_t                     lptim_id;
  uint32_t                    base;
  enum stm32u5_lptim_mode_e   mode;
  enum stm32u5_lptim_clksrc_e clksrc;
  uint8_t                     prescaler;
  enum stm32u5_lptim_pol_e    polarity;
  enum stm32u5_lptim_trig_e   trigger;
  uint16_t                    period;
  uint16_t                    cmp;
};

/****************************************************************************
 * Public Data
 ****************************************************************************/

#undef EXTERN
#if defined(__cplusplus)
#define EXTERN extern "C"
extern "C"
{
#else
#define EXTERN extern
#endif

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32u5_lptiminitialize
 *
 * Description:
 *   Initialize the LPTIM peripheral.
 *
 * Input Parameters:
 *   timer - The timer number (1-4)
 *
 * Returned Value:
 *   A pointer to the LPTIM driver structure, or NULL on failure.
 *
 ****************************************************************************/

struct stm32u5_lptim_dev_s *stm32u5_lptiminitialize(int timer);

/****************************************************************************
 * Name: stm32u5_lptimuninitialize
 *
 * Description:
 *   Uninitialize the LPTIM peripheral.
 *
 * Input Parameters:
 *   dev - Pointer to the LPTIM driver structure
 *
 * Returned Value:
 *   None
 *
 ****************************************************************************/

void stm32u5_lptimuninitialize(struct stm32u5_lptim_dev_s *dev);

/****************************************************************************
 * Name: stm32u5_lptimenable
 *
 * Description:
 *   Enable the LPTIM peripheral.
 *
 * Input Parameters:
 *   dev - Pointer to the LPTIM driver structure
 *
 * Returned Value:
 *   Zero on success; a negated errno value on failure.
 *
 ****************************************************************************/

int stm32u5_lptimenable(struct stm32u5_lptim_dev_s *dev);

/****************************************************************************
 * Name: stm32u5_lptimdisable
 *
 * Description:
 *   Disable the LPTIM peripheral.
 *
 * Input Parameters:
 *   dev - Pointer to the LPTIM driver structure
 *
 * Returned Value:
 *   Zero on success; a negated errno value on failure.
 *
 ****************************************************************************/

int stm32u5_lptimdisable(struct stm32u5_lptim_dev_s *dev);

/****************************************************************************
 * Name: stm32u5_lptimsetclock
 *
 * Description:
 *   Set the LPTIM clock source.
 *
 * Input Parameters:
 *   dev - Pointer to the LPTIM driver structure
 *   clksrc - Clock source (internal/external)
 *
 * Returned Value:
 *   Zero on success; a negated errno value on failure.
 *
 ****************************************************************************/

int stm32u5_lptimsetclock(struct stm32u5_lptim_dev_s *dev,
                          enum stm32u5_lptim_clksrc_e clksrc);

/****************************************************************************
 * Name: stm32u5_lptimsetprescaler
 *
 * Description:
 *   Set the LPTIM clock prescaler.
 *
 * Input Parameters:
 *   dev - Pointer to the LPTIM driver structure
 *   prescaler - Prescaler value (0-7)
 *
 * Returned Value:
 *   Zero on success; a negated errno value on failure.
 *
 ****************************************************************************/

int stm32u5_lptimsetprescaler(struct stm32u5_lptim_dev_s *dev,
                               uint8_t prescaler);

/****************************************************************************
 * Name: stm32u5_lptimsetperiod
 *
 * Description:
 *   Set the LPTIM autoreload period.
 *
 * Input Parameters:
 *   dev - Pointer to the LPTIM driver structure
 *   period - Period value (16-bit)
 *
 * Returned Value:
 *   Zero on success; a negated errno value on failure.
 *
 ****************************************************************************/

int stm32u5_lptimsetperiod(struct stm32u5_lptim_dev_s *dev,
                           uint16_t period);

/****************************************************************************
 * Name: stm32u5_lptimgetcounter
 *
 * Description:
 *   Get the current LPTIM counter value.
 *
 * Input Parameters:
 *   dev - Pointer to the LPTIM driver structure
 *
 * Returned Value:
 *   Counter value.
 *
 ****************************************************************************/

uint16_t stm32u5_lptimgetcounter(struct stm32u5_lptim_dev_s *dev);

/****************************************************************************
 * Name: stm32u5_lptimsetcompare
 *
 * Description:
 *   Set the LPTIM compare value.
 *
 * Input Parameters:
 *   dev - Pointer to the LPTIM driver structure
 *   cmp - Compare value
 *
 * Returned Value:
 *   Zero on success; a negated errno value on failure.
 *
 ****************************************************************************/

int stm32u5_lptimsetcompare(struct stm32u5_lptim_dev_s *dev,
                            uint16_t cmp);

/****************************************************************************
 * Name: stm32u5_lptimstart
 *
 * Description:
 *   Start the LPTIM in the specified mode.
 *
 * Input Parameters:
 *   dev - Pointer to the LPTIM driver structure
 *   mode - Operating mode (continuous/single)
 *
 * Returned Value:
 *   Zero on success; a negated errno value on failure.
 *
 ****************************************************************************/

int stm32u5_lptimstart(struct stm32u5_lptim_dev_s *dev,
                        enum stm32u5_lptim_mode_e mode);

/****************************************************************************
 * Name: stm32u5_lptimstop
 *
 * Description:
 *   Stop the LPTIM.
 *
 * Input Parameters:
 *   dev - Pointer to the LPTIM driver structure
 *
 * Returned Value:
 *   Zero on success; a negated errno value on failure.
 *
 ****************************************************************************/

int stm32u5_lptimstop(struct stm32u5_lptim_dev_s *dev);

#undef EXTERN
#if defined(__cplusplus)
}
#endif

#endif /* __ASSEMBLY__ */
#endif /* __ARCH_ARM_SRC_STM32U5_STM32U5_LPTIM_H */