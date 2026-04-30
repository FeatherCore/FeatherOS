/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_comparator.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_COMPARATOR_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_COMPARATOR_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Comparator Base Address */

#define RA8P_COMP_BASE                          (0x40228000)

/* Comparator Register Offsets */

/* Comparator Control Register 0 (COMPDR0) */

#define RA8P_COMP_COMPDR0_OFFSET                (0x000)
#define RA8P_COMP_COMPDR0_COMPSEL               (1 << 0)    /* Bit 0: Comparator Select */
#define RA8P_COMP_COMPDR0_CMPSEL_MASK           (0x7)       /* Bits 1-3: Compare Input Select */
#define RA8P_COMP_COMPDR0_CMPSEL_SHIFT          (1)
#define RA8P_COMP_COMPDR0_CMPSEL_IN0            (0x0 << 1)  /* Input 0 */
#define RA8P_COMP_COMPDR0_CMPSEL_IN1            (0x1 << 1)  /* Input 1 */
#define RA8P_COMP_COMPDR0_CMPSEL_IN2            (0x2 << 1)  /* Input 2 */
#define RA8P_COMP_COMPDR0_CMPSEL_IN3            (0x3 << 1)  /* Input 3 */
#define RA8P_COMP_COMPDR0_CMPSEL_IN4            (0x4 << 1)  /* Input 4 */
#define RA8P_COMP_COMPDR0_CMPSEL_IN5            (0x5 << 1)  /* Input 5 */
#define RA8P_COMP_COMPDR0_CMPSEL_IN6            (0x6 << 1)  /* Input 6 */
#define RA8P_COMP_COMPDR0_CMPSEL_IN7            (0x7 << 1)  /* Input 7 */
#define RA8P_COMP_COMPDR0_REFSEL_MASK           (0x30)      /* Bits 4-5: Reference Input Select */
#define RA8P_COMP_COMPDR0_REFSEL_SHIFT          (4)
#define RA8P_COMP_COMPDR0_REFSEL_IN0            (0x0 << 4)  /* Reference Input 0 */
#define RA8P_COMP_COMPDR0_REFSEL_IN1            (0x1 << 4)  /* Reference Input 1 */
#define RA8P_COMP_COMPDR0_REFSEL_IN2            (0x2 << 4)  /* Reference Input 2 */
#define RA8P_COMP_COMPDR0_REFSEL_IN3            (0x3 << 4)  /* Reference Input 3 */
#define RA8P_COMP_COMPDR0_OUTEN                 (1 << 6)    /* Bit 6: Output Enable */
#define RA8P_COMP_COMPDR0_OUTMON                (1 << 7)    /* Bit 7: Output Monitor */
#define RA8P_COMP_COMPDR0_INTSEL                (1 << 8)    /* Bit 8: Interrupt Select */
#define RA8P_COMP_COMPDR0_INTEN                 (1 << 9)    /* Bit 9: Interrupt Enable */
#define RA8P_COMP_COMPDR0_COMPON                (1 << 15)   /* Bit 15: Comparator On */

/* Comparator Control Register 1 (COMPDR1) */

#define RA8P_COMP_COMPDR1_OFFSET                (0x004)
#define RA8P_COMP_COMPDR1_CMP1SEL_MASK          (0x7)       /* Bits 0-2: CMP1 Input Select */
#define RA8P_COMP_COMPDR1_CMP1SEL_SHIFT         (0)
#define RA8P_COMP_COMPDR1_CMP1SEL_IN0           (0x0 << 0)  /* Input 0 */
#define RA8P_COMP_COMPDR1_CMP1SEL_IN1           (0x1 << 0)  /* Input 1 */
#define RA8P_COMP_COMPDR1_CMP1SEL_IN2           (0x2 << 0)  /* Input 2 */
#define RA8P_COMP_COMPDR1_CMP1SEL_IN3           (0x3 << 0)  /* Input 3 */
#define RA8P_COMP_COMPDR1_CMP1SEL_IN4           (0x4 << 0)  /* Input 4 */
#define RA8P_COMP_COMPDR1_CMP1SEL_IN5           (0x5 << 0)  /* Input 5 */
#define RA8P_COMP_COMPDR1_CMP1SEL_IN6           (0x6 << 0)  /* Input 6 */
#define RA8P_COMP_COMPDR1_CMP1SEL_IN7           (0x7 << 0)  /* Input 7 */
#define RA8P_COMP_COMPDR1_CMP2SEL_MASK          (0x70)      /* Bits 4-6: CMP2 Input Select */
#define RA8P_COMP_COMPDR1_CMP2SEL_SHIFT         (4)
#define RA8P_COMP_COMPDR1_CMP2SEL_IN0           (0x0 << 4)  /* Input 0 */
#define RA8P_COMP_COMPDR1_CMP2SEL_IN1           (0x1 << 4)  /* Input 1 */
#define RA8P_COMP_COMPDR1_CMP2SEL_IN2           (0x2 << 4)  /* Input 2 */
#define RA8P_COMP_COMPDR1_CMP2SEL_IN3           (0x3 << 4)  /* Input 3 */
#define RA8P_COMP_COMPDR1_CMP2SEL_IN4           (0x4 << 4)  /* Input 4 */
#define RA8P_COMP_COMPDR1_CMP2SEL_IN5           (0x5 << 4)  /* Input 5 */
#define RA8P_COMP_COMPDR1_CMP2SEL_IN6           (0x6 << 4)  /* Input 6 */
#define RA8P_COMP_COMPDR1_CMP2SEL_IN7           (0x7 << 4)  /* Input 7 */
#define RA8P_COMP_COMPDR1_REF1SEL_MASK          (0x300)     /* Bits 8-9: Reference 1 Input Select */
#define RA8P_COMP_COMPDR1_REF1SEL_SHIFT         (8)
#define RA8P_COMP_COMPDR1_REF2SEL_MASK          (0xC00)     /* Bits 10-11: Reference 2 Input Select */
#define RA8P_COMP_COMPDR1_REF2SEL_SHIFT         (10)
#define RA8P_COMP_COMPDR1_OUT1EN                (1 << 12)   /* Bit 12: Output 1 Enable */
#define RA8P_COMP_COMPDR1_OUT2EN                (1 << 13)   /* Bit 13: Output 2 Enable */
#define RA8P_COMP_COMPDR1_OUT1MON               (1 << 14)   /* Bit 14: Output 1 Monitor */
#define RA8P_COMP_COMPDR1_OUT2MON               (1 << 15)   /* Bit 15: Output 2 Monitor */

/* Comparator Status Register (COMPSTR) */

#define RA8P_COMP_COMPSTR_OFFSET                (0x008)
#define RA8P_COMP_COMPSTR_CMPOUT                (1 << 0)    /* Bit 0: Comparator Output */
#define RA8P_COMP_COMPSTR_CMPOUT1               (1 << 1)    /* Bit 1: Comparator Output 1 */
#define RA8P_COMP_COMPSTR_CMPOUT2               (1 << 2)    /* Bit 2: Comparator Output 2 */
#define RA8P_COMP_COMPSTR_CMPMON                (1 << 8)    /* Bit 8: Comparator Monitor */
#define RA8P_COMP_COMPSTR_CMP1MON               (1 << 9)    /* Bit 9: Comparator Monitor 1 */
#define RA8P_COMP_COMPSTR_CMP2MON               (1 << 10)   /* Bit 10: Comparator Monitor 2 */
#define RA8P_COMP_COMPSTR_CMPCOND               (1 << 15)   /* Bit 15: Compare Condition */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* Comparator input channel */

enum ra8p_comp_input_e
{
  RA8P_COMP_INPUT_IN0 = 0,          /* Input 0 */
  RA8P_COMP_INPUT_IN1,             /* Input 1 */
  RA8P_COMP_INPUT_IN2,             /* Input 2 */
  RA8P_COMP_INPUT_IN3,             /* Input 3 */
  RA8P_COMP_INPUT_IN4,             /* Input 4 */
  RA8P_COMP_INPUT_IN5,             /* Input 5 */
  RA8P_COMP_INPUT_IN6,             /* Input 6 */
  RA8P_COMP_INPUT_IN7,             /* Input 7 */
};

/* Comparator reference input */

enum ra8p_comp_reference_e
{
  RA8P_COMP_REFERENCE_IN0 = 0,      /* Reference Input 0 */
  RA8P_COMP_REFERENCE_IN1,         /* Reference Input 1 */
  RA8P_COMP_REFERENCE_IN2,         /* Reference Input 2 */
  RA8P_COMP_REFERENCE_IN3,         /* Reference Input 3 */
};

/* Comparator configuration */

struct ra8p_comparator_config_s
{
  uint8_t comparator_num;           /* Comparator number (0-2) */
  enum ra8p_comp_input_e input;     /* Input channel */
  enum ra8p_comp_reference_e reference; /* Reference input */
  bool output_enable;               /* Enable output */
  bool interrupt_enable;            /* Enable interrupt */
  bool monitor_enable;              /* Enable monitor */
};

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_COMPARATOR_H */