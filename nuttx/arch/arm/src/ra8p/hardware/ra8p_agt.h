/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_agt.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_AGT_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_AGT_H_

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* AGT (Asynchronous General-Purpose Timer) Base Addresses */

#define RA8P_AGT0_BASE                         (0x40221000)
#define RA8P_AGT1_BASE                         (0x40221020)

/* AGT Register Offsets */

/* AGT Control Register (AGTCR) */

#define RA8P_AGT_AGTCR_OFFSET                 (0x000)
#define RA8P_AGT_AGTCR_TSTART                (1 << 0)    /* Bit 0: Timer Start */
#define RA8P_AGT_AGTCR_TCPE                  (1 << 1)    /* Bit 1: Timer Stop */
#define RA8P_AGT_AGTCR_TEDGEL               (1 << 2)    /* Bit 2: Trigger Edge Select */
#define RA8P_AGT_AGTCR_TEDGEL_RISING        (0 << 2)    /* Rising edge */
#define RA8P_AGT_AGTCR_TEDGEL_FALLING        (1 << 2)    /* Falling edge */
#define RA8P_AGT_AGTCR_TMOD_MASK             (0x30)      /* Bits 4-5: Timer Mode */
#define RA8P_AGT_AGTCR_TMOD_SHIFT            (4)
#define RA8P_AGT_AGTCR_TMOD_TIMER           (0x0 << 4)  /* Timer mode */
#define RA8P_AGT_AGTCR_TMOD_PULSE           (0x1 << 4)  /* Pulse output mode */
#define RA8P_AGT_AGTCR_TMOD_EVENT           (0x2 << 4)  /* Event counter mode */
#define RA8P_AGT_AGTCR_TMOD_ONE_SHOT        (0x3 << 4)  /* One-shot pulse mode */
#define RA8P_AGT_AGTCR_TFUNC_MASK           (0xC0)      /* Bits 6-7: Timer Function */
#define RA8P_AGT_AGTCR_TFUNC_SHIFT          (6)
#define RA8P_AGT_AGTCR_TFUNC_DIS           (0x0 << 6)  /* Disabled */
#define RA8P_AGT_AGTCR_TFUNC_CLK           (0x1 << 6)  /* Clock counting */
#define RA8P_AGT_AGTCR_TFUNC_CAP           (0x2 << 6)  /* Capture */
#define RA8P_AGT_AGTCR_TFUNC_PULSE          (0x3 << 6)  /* Pulse output */

/* AGT Clock Control Register (AGTMR) */

#define RA8P_AGT_AGTMR_OFFSET                 (0x004)
#define RA8P_AGT_AGTMR_TCK_MASK              (0x7)       /* Bits 0-2: Clock Source */
#define RA8P_AGT_AGTMR_TCK_SHIFT             (0)
#define RA8P_AGT_AGTMR_TCK_PCLKB            (0x0)       /* PCLKB */
#define RA8P_AGT_AGTMR_TCK_PCLKB2           (0x1)       /* PCLKB2 */
#define RA8P_AGT_AGTMR_TCK_SUBOSC          (0x2)       /* Sub-clock oscillator */
#define RA8P_AGT_AGTMR_TCK_LOCO             (0x3)       /* LOCO (32.768 kHz) */
#define RA8P_AGT_AGTMR_TCK_FSUB              (0x4)       /* FSUB (15 kHz) */
#define RA8P_AGT_AGTMR_TCK_AGTSCK          (0x5)       /* AGTSCK (external) */
#define RA8P_AGT_AGTMR_TCK_TRG              (0x6)       /* Trigger input */
#define RA8P_AGT_AGTMR_TPCS_MASK             (0x70)      /* Bits 4-6: Clock Divider */
#define RA8P_AGT_AGTMR_TPCS_SHIFT            (4)
#define RA8P_AGT_AGTMR_TPCS_1               (0x0 << 4)  /* Divider 1 */
#define RA8P_AGT_AGTMR_TPCS_2               (0x1 << 4)  /* Divider 2 */
#define RA8P_AGT_AGTMR_TPCS_4               (0x2 << 4)  /* Divider 4 */
#define RA8P_AGT_AGTMR_TPCS_8               (0x3 << 4)  /* Divider 8 */
#define RA8P_AGT_AGTMR_TPCS_16              (0x4 << 4)  /* Divider 16 */
#define RA8P_AGT_AGTMR_TPCS_32              (0x5 << 4)  /* Divider 32 */
#define RA8P_AGT_AGTMR_TPCS_64              (0x6 << 4)  /* Divider 64 */
#define RA8P_AGT_AGTMR_TPCS_128             (0x7 << 4)  /* Divider 128 */
#define RA8P_AGT_AGTMR_TEDGEL               (1 << 7)    /* Bit 7: Trigger Edge */

/* AGT Compare Match Register (AGTCMPA) */

#define RA8P_AGT_AGTCMPA_OFFSET                (0x008)
#define RA8P_AGT_AGTCMPA_VALUE_MASK           (0xFFFF)    /* Bits 0-15: Compare Match A Value */

/* AGT Compare Match Register (AGTCMPB) */

#define RA8P_AGT_AGTCMPB_OFFSET                (0x00A)
#define RA8P_AGT_AGTCMPB_VALUE_MASK           (0xFFFF)    /* Bits 0-15: Compare Match B Value */

/* AGT Counter Register (AGT) */

#define RA8P_AGT_AGT_OFFSET                    (0x00C)
#define RA8P_AGT_AGT_COUNT_MASK               (0xFFFF)    /* Bits 0-15: Counter Value */

/* AGT Status Register (AGTSR) */

#define RA8P_AGT_AGTSR_OFFSET                 (0x010)
#define RA8P_AGT_AGTSR_CCMFA                 (1 << 0)    /* Bit 0: Compare Match A Flag */
#define RA8P_AGT_AGTSR_CCMFB                 (1 << 1)    /* Bit 1: Compare Match B Flag */
#define RA8P_AGT_AGTSR_OVF                   (1 << 2)    /* Bit 2: Overflow Flag */
#define RA8P_AGT_AGTSR_UNF                   (1 << 3)    /* Bit 3: Underflow Flag */
#define RA8P_AGT_AGTSR_TRGFF                 (1 << 4)    /* Bit 4: Trigger Flag */

/* AGT Interrupt Enable Register (AGTIER) */

#define RA8P_AGT_AGTIER_OFFSET                (0x014)
#define RA8P_AGT_AGTIER_CCMAE                (1 << 0)    /* Bit 0: Compare Match A Interrupt Enable */
#define RA8P_AGT_AGTIER_CCMBE                (1 << 1)    /* Bit 1: Compare Match B Interrupt Enable */
#define RA8P_AGT_AGTIER_OVIE                 (1 << 2)    /* Bit 2: Overflow Interrupt Enable */
#define RA8P_AGT_AGTIER_UNIE                 (1 << 3)    /* Bit 3: Underflow Interrupt Enable */
#define RA8P_AGT_AGTIER_TRFIE                (1 << 4)    /* Bit 4: Trigger Interrupt Enable */

/* AGT Event Control Register (AGTECR) */

#define RA8P_AGT_AGTECR_OFFSET                (0x018)
#define RA8P_AGT_AGTECR_CCMAE                (1 << 0)    /* Bit 0: Compare Match A Event Enable */
#define RA8P_AGT_AGTECR_CCMBE                (1 << 1)    /* Bit 1: Compare Match B Event Enable */
#define RA8P_AGT_AGTECR_OVE                  (1 << 2)    /* Bit 2: Overflow Event Enable */
#define RA8P_AGT_AGTECR_UNE                  (1 << 3)    /* Bit 3: Underflow Event Enable */
#define RA8P_AGT_AGTECR_TRFE                 (1 << 4)    /* Bit 4: Trigger Event Enable */

/* AGT Option Control Register (AGTOCR) */

#define RA8P_AGT_AGTOCR_OFFSET                (0x01C)
#define RA8P_AGT_AGTOCR_TOE                  (1 << 0)    /* Bit 0: Timer Output Enable */
#define RA8P_AGT_AGTOCR_TOPOL                (1 << 1)    /* Bit 1: Timer Output Polarity */
#define RA8P_AGT_AGTOCR_TSTARTE               (1 << 2)    /* Bit 2: Timer Start Enable */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* AGT clock source */

enum ra8p_agt_clock_source_e
{
  RA8P_AGT_CLOCK_PCLKB = 0,       /* PCLKB */
  RA8P_AGT_CLOCK_PCLKB2,           /* PCLKB2 */
  RA8P_AGT_CLOCK_SUBOSC,          /* Sub-clock oscillator (32.768 kHz) */
  RA8P_AGT_CLOCK_LOCO,            /* LOCO (32.768 kHz) */
  RA8P_AGT_CLOCK_FSUB,            /* FSUB (15 kHz) */
  RA8P_AGT_CLOCK_AGTSCK,          /* AGTSCK (external clock) */
  RA8P_AGT_CLOCK_TRIGGER,          /* Trigger input */
};

/* AGT timer mode */

enum ra8p_agt_timer_mode_e
{
  RA8P_AGT_MODE_TIMER = 0,        /* Timer mode */
  RA8P_AGT_MODE_PULSE,           /* Pulse output mode */
  RA8P_AGT_MODE_EVENT,            /* Event counter mode */
  RA8P_AGT_MODE_ONE_SHOT,        /* One-shot pulse mode */
};

/* AGT timer function */

enum ra8p_agt_timer_function_e
{
  RA8P_AGT_FUNC_DISABLE = 0,       /* Disabled */
  RA8P_AGT_FUNC_CLOCK_COUNT,     /* Clock counting */
  RA8P_AGT_FUNC_CAPTURE,         /* Capture */
  RA8P_AGT_FUNC_PULSE_OUTPUT,    /* Pulse output */
};

/* AGT clock divider */

enum ra8p_agt_clock_divider_e
{
  RA8P_AGT_DIV_1 = 0,             /* Divider 1 */
  RA8P_AGT_DIV_2,                 /* Divider 2 */
  RA8P_AGT_DIV_4,                 /* Divider 4 */
  RA8P_AGT_DIV_8,                 /* Divider 8 */
  RA8P_AGT_DIV_16,                /* Divider 16 */
  RA8P_AGT_DIV_32,                /* Divider 32 */
  RA8P_AGT_DIV_64,                /* Divider 64 */
  RA8P_AGT_DIV_128,               /* Divider 128 */
};

/* AGT configuration */

struct ra8p_agt_config_s
{
  uint32_t base;                    /* AGT base address */
  int irq;                          /* AGT interrupt number */
  enum ra8p_agt_clock_source_e clock; /* Clock source */
  enum ra8p_agt_timer_mode_e mode;   /* Timer mode */
  enum ra8p_agt_timer_function_e func; /* Timer function */
  enum ra8p_agt_clock_divider_e div; /* Clock divider */
  uint16_t period;                 /* Timer period (for periodic mode) */
  bool interrupt_enable;            /* Enable interrupts */
  bool output_enable;              /* Enable timer output */
  bool output_polarity;             /* Output polarity (true = active high) */
};

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_AGT_H */