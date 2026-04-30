/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_icu.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_ICU_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_ICU_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* ICU Register Offsets - Based on Zephyr RA8P1 implementation */
#define RA8P_ICU_IELSR_OFFSET(n)    ((n) * 4)     /* Interrupt Event Link Select Register */

/* IRQCR Register Offsets - Based on Zephyr renesas_ra_external_interrupt.c */
#define RA8P_ICU_IRQCR_OFFSET(n)    (0x06000 + (n))

/* IRQCR bit definitions */
#define RA8P_ICU_IRQCR_IRQMD_MASK   (0x03 << 0)   /* IRQ Mode Mask */
#define RA8P_ICU_IRQCR_IRQMD_SHIFT  0
#define RA8P_ICU_IRQCR_FLTEN        (1 << 2)      /* Digital Filter Enable */
#define RA8P_ICU_IRQCR_ISEL         (1 << 3)      /* IRQ Enable */

/* IRQMD values - Based on Zephyr enum ext_irq_trigger */
#define RA8P_ICU_IRQMD_FALLING      0             /* Falling edge */
#define RA8P_ICU_IRQMD_RISING       1             /* Rising edge */
#define RA8P_ICU_IRQMD_BOTH         2             /* Both edges */
#define RA8P_ICU_IRQMD_LOW_LEVEL    3             /* Low level */

/* Sample clock divider values - Based on Zephyr enum ext_irq_sample_clock */
#define RA8P_ICU_FCLKSEL_DIV1       0             /* Divide by 1 */
#define RA8P_ICU_FCLKSEL_DIV8       1             /* Divide by 8 */
#define RA8P_ICU_FCLKSEL_DIV32      2             /* Divide by 32 */
#define RA8P_ICU_FCLKSEL_DIV64      3             /* Divide by 64 */

/* Event Link Numbers - Based on RA8P1 hardware manual */
#define RA8P_EVENT_ICU_IRQ0         0
#define RA8P_EVENT_ICU_IRQ1         1
#define RA8P_EVENT_ICU_IRQ2         2
#define RA8P_EVENT_ICU_IRQ3         3
#define RA8P_EVENT_ICU_IRQ4         4
#define RA8P_EVENT_ICU_IRQ5         5
#define RA8P_EVENT_ICU_IRQ6         6
#define RA8P_EVENT_ICU_IRQ7         7
#define RA8P_EVENT_ICU_IRQ8         8
#define RA8P_EVENT_ICU_IRQ9         9
#define RA8P_EVENT_ICU_IRQ10        10
#define RA8P_EVENT_ICU_IRQ11        11
#define RA8P_EVENT_ICU_IRQ12        12
#define RA8P_EVENT_ICU_IRQ13        13
#define RA8P_EVENT_ICU_IRQ14        14
#define RA8P_EVENT_ICU_IRQ15        15

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* External interrupt trigger mode */
enum ra8p_ext_irq_trigger_e
{
  RA8P_EXT_IRQ_FALLING = 0,     /* Falling edge trigger */
  RA8P_EXT_IRQ_RISING,          /* Rising edge trigger */
  RA8P_EXT_IRQ_BOTH,            /* Both edges trigger */
  RA8P_EXT_IRQ_LOW_LEVEL,       /* Low level trigger */
};

/* External interrupt callback */
typedef void (*ra8p_ext_irq_callback_t)(uint8_t channel, void *arg);

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_icu_init
 *
 * Description:
 *   Initialize the ICU (Interrupt Controller Unit).
 *
 ****************************************************************************/

void ra8p_icu_init(void);

/****************************************************************************
 * Name: ra8p_icu_configure_irq
 *
 * Description:
 *   Configure an external interrupt channel.
 *   Based on Zephyr gpio_ra_interrupt_set implementation.
 *
 * Input Parameters:
 *   channel     - IRQ channel (0-15)
 *   trigger     - Trigger mode
 *   filter      - Enable digital filter
 *   sample_div  - Sample clock divider
 *
 ****************************************************************************/

int ra8p_icu_configure_irq(uint8_t channel, 
                           enum ra8p_ext_irq_trigger_e trigger,
                           bool filter, uint8_t sample_div);

/****************************************************************************
 * Name: ra8p_icu_enable_irq
 *
 * Description:
 *   Enable an external interrupt channel.
 *
 * Input Parameters:
 *   channel - IRQ channel (0-15)
 *
 ****************************************************************************/

void ra8p_icu_enable_irq(uint8_t channel);

/****************************************************************************
 * Name: ra8p_icu_disable_irq
 *
 * Description:
 *   Disable an external interrupt channel.
 *
 * Input Parameters:
 *   channel - IRQ channel (0-15)
 *
 ****************************************************************************/

void ra8p_icu_disable_irq(uint8_t channel);

/****************************************************************************
 * Name: ra8p_icu_clear_irq
 *
 * Description:
 *   Clear an external interrupt flag.
 *
 * Input Parameters:
 *   channel - IRQ channel (0-15)
 *
 ****************************************************************************/

void ra8p_icu_clear_irq(uint8_t channel);

/****************************************************************************
 * Name: ra8p_icu_set_callback
 *
 * Description:
 *   Set callback for an external interrupt.
 *
 * Input Parameters:
 *   channel  - IRQ channel (0-15)
 *   callback - Callback function
 *   arg      - User argument
 *
 ****************************************************************************/

int ra8p_icu_set_callback(uint8_t channel, 
                          ra8p_ext_irq_callback_t callback,
                          void *arg);

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_ICU_H */