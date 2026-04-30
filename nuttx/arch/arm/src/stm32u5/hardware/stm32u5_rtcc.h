/****************************************************************************
 * arch/arm/src/stm32u5/hardware/stm32u5_rtcc.h
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

#ifndef __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_RTCC_H
#define __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_RTCC_H

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Register Offsets *********************************************************/

#define STM32U5_RTC_TR_OFFSET        0x0000 /* RTC time register */
#define STM32U5_RTC_DR_OFFSET        0x0004 /* RTC date register */
#define STM32U5_RTC_CR_OFFSET        0x0008 /* RTC control register */
#define STM32U5_RTC_ISR_OFFSET       0x000c /* RTC initialization and status register */
#define STM32U5_RTC_PRER_OFFSET      0x0010 /* RTC prescaler register */
#define STM32U5_RTC_WUTR_OFFSET      0x0014 /* RTC wakeup timer register */
#define STM32U5_RTC_ALRMAR_OFFSET    0x001c /* RTC alarm A register */
#define STM32U5_RTC_ALRMBR_OFFSET    0x0020 /* RTC alarm B register */
#define STM32U5_RTC_WPR_OFFSET       0x0024 /* RTC write protection register */
#define STM32U5_RTC_SSR_OFFSET       0x0028 /* RTC sub second register */
#define STM32U5_RTC_SHIFTR_OFFSET    0x002c /* RTC shift control register */
#define STM32U5_RTC_TSTR_OFFSET      0x0030 /* RTC time stamp time register */
#define STM32U5_RTC_TSDR_OFFSET      0x0034 /* RTC time stamp date register */
#define STM32U5_RTC_TSSSR_OFFSET     0x0038 /* RTC timestamp sub second register */
#define STM32U5_RTC_CALR_OFFSET      0x003c /* RTC calibration register */
#define STM32U5_RTC_TAMPCR_OFFSET    0x0040 /* RTC tamper configuration register */
#define STM32U5_RTC_ALRMASSR_OFFSET  0x0044 /* RTC alarm A sub second register */
#define STM32U5_RTC_ALRMBSSR_OFFSET  0x0048 /* RTC alarm B sub second register */
#define STM32U5_RTC_OR_OFFSET        0x004c /* RTC option register */
#define STM32U5_RTC_BKPXR_OFFSET(n)  (0x0050+((n)<<2)) /* RTC backup registers */
#define STM32U5_RTC_BK0R_OFFSET      0x0050 /* RTC backup register 0 */
#define STM32U5_RTC_BK1R_OFFSET      0x0054 /* RTC backup register 1 */
#define STM32U5_RTC_BK2R_OFFSET      0x0058 /* RTC backup register 2 */
#define STM32U5_RTC_BK3R_OFFSET      0x005c /* RTC backup register 3 */
#define STM32U5_RTC_BK4R_OFFSET      0x0060 /* RTC backup register 4 */
#define STM32U5_RTC_BK5R_OFFSET      0x0064 /* RTC backup register 5 */
#define STM32U5_RTC_BK6R_OFFSET      0x0068 /* RTC backup register 6 */
#define STM32U5_RTC_BK7R_OFFSET      0x006c /* RTC backup register 7 */
#define STM32U5_RTC_BK8R_OFFSET      0x0070 /* RTC backup register 8 */
#define STM32U5_RTC_BK9R_OFFSET      0x0074 /* RTC backup register 9 */
#define STM32U5_RTC_BK10R_OFFSET     0x0078 /* RTC backup register 10 */
#define STM32U5_RTC_BK11R_OFFSET     0x007c /* RTC backup register 11 */
#define STM32U5_RTC_BK12R_OFFSET     0x0080 /* RTC backup register 12 */
#define STM32U5_RTC_BK13R_OFFSET     0x0084 /* RTC backup register 13 */
#define STM32U5_RTC_BK14R_OFFSET     0x0088 /* RTC backup register 14 */
#define STM32U5_RTC_BK15R_OFFSET     0x008c /* RTC backup register 15 */
#define STM32U5_RTC_BK16R_OFFSET     0x0090 /* RTC backup register 16 */
#define STM32U5_RTC_BK17R_OFFSET     0x0094 /* RTC backup register 17 */
#define STM32U5_RTC_BK18R_OFFSET     0x0098 /* RTC backup register 18 */
#define STM32U5_RTC_BK19R_OFFSET     0x009c /* RTC backup register 19 */
#define STM32U5_RTC_BK20R_OFFSET     0x00a0 /* RTC backup register 20 */
#define STM32U5_RTC_BK21R_OFFSET     0x00a4 /* RTC backup register 21 */
#define STM32U5_RTC_BK22R_OFFSET     0x00a8 /* RTC backup register 22 */
#define STM32U5_RTC_BK23R_OFFSET     0x00ac /* RTC backup register 23 */
#define STM32U5_RTC_BK24R_OFFSET     0x00b0 /* RTC backup register 24 */
#define STM32U5_RTC_BK25R_OFFSET     0x00b4 /* RTC backup register 25 */
#define STM32U5_RTC_BK26R_OFFSET     0x00b8 /* RTC backup register 26 */
#define STM32U5_RTC_BK27R_OFFSET     0x00bc /* RTC backup register 27 */
#define STM32U5_RTC_BK28R_OFFSET     0x00c0 /* RTC backup register 28 */
#define STM32U5_RTC_BK29R_OFFSET     0x00c4 /* RTC backup register 29 */
#define STM32U5_RTC_BK30R_OFFSET     0x00c8 /* RTC backup register 30 */
#define STM32U5_RTC_BK31R_OFFSET     0x00cc /* RTC backup register 31 */

/* Register Addresses *******************************************************/

#define STM32U5_RTC_TR               (STM32U5_RTC_BASE+STM32U5_RTC_TR_OFFSET)
#define STM32U5_RTC_DR               (STM32U5_RTC_BASE+STM32U5_RTC_DR_OFFSET)
#define STM32U5_RTC_CR               (STM32U5_RTC_BASE+STM32U5_RTC_CR_OFFSET)
#define STM32U5_RTC_ISR              (STM32U5_RTC_BASE+STM32U5_RTC_ISR_OFFSET)
#define STM32U5_RTC_PRER             (STM32U5_RTC_BASE+STM32U5_RTC_PRER_OFFSET)
#define STM32U5_RTC_WUTR             (STM32U5_RTC_BASE+STM32U5_RTC_WUTR_OFFSET)
#define STM32U5_RTC_ALRMAR           (STM32U5_RTC_BASE+STM32U5_RTC_ALRMAR_OFFSET)
#define STM32U5_RTC_ALRMBR           (STM32U5_RTC_BASE+STM32U5_RTC_ALRMBR_OFFSET)
#define STM32U5_RTC_WPR              (STM32U5_RTC_BASE+STM32U5_RTC_WPR_OFFSET)
#define STM32U5_RTC_SSR              (STM32U5_RTC_BASE+STM32U5_RTC_SSR_OFFSET)
#define STM32U5_RTC_SHIFTR           (STM32U5_RTC_BASE+STM32U5_RTC_SHIFTR_OFFSET)
#define STM32U5_RTC_TSTR             (STM32U5_RTC_BASE+STM32U5_RTC_TSTR_OFFSET)
#define STM32U5_RTC_TSDR             (STM32U5_RTC_BASE+STM32U5_RTC_TSDR_OFFSET)
#define STM32U5_RTC_TSSSR            (STM32U5_RTC_BASE+STM32U5_RTC_TSSSR_OFFSET)
#define STM32U5_RTC_CALR             (STM32U5_RTC_BASE+STM32U5_RTC_CALR_OFFSET)
#define STM32U5_RTC_TAMPCR           (STM32U5_RTC_BASE+STM32U5_RTC_TAMPCR_OFFSET)
#define STM32U5_RTC_ALRMASSR         (STM32U5_RTC_BASE+STM32U5_RTC_ALRMASSR_OFFSET)
#define STM32U5_RTC_ALRMBSSR         (STM32U5_RTC_BASE+STM32U5_RTC_ALRMBSSR_OFFSET)
#define STM32U5_RTC_OR               (STM32U5_RTC_BASE+STM32U5_RTC_OR_OFFSET)
#define STM32U5_RTC_BK0R             (STM32U5_RTC_BASE+STM32U5_RTC_BK0R_OFFSET)
#define STM32U5_RTC_BK1R             (STM32U5_RTC_BASE+STM32U5_RTC_BK1R_OFFSET)
#define STM32U5_RTC_BK2R             (STM32U5_RTC_BASE+STM32U5_RTC_BK2R_OFFSET)
#define STM32U5_RTC_BK3R             (STM32U5_RTC_BASE+STM32U5_RTC_BK3R_OFFSET)
#define STM32U5_RTC_BK4R             (STM32U5_RTC_BASE+STM32U5_RTC_BK4R_OFFSET)
#define STM32U5_RTC_BK5R             (STM32U5_RTC_BASE+STM32U5_RTC_BK5R_OFFSET)
#define STM32U5_RTC_BK6R             (STM32U5_RTC_BASE+STM32U5_RTC_BK6R_OFFSET)
#define STM32U5_RTC_BK7R             (STM32U5_RTC_BASE+STM32U5_RTC_BK7R_OFFSET)
#define STM32U5_RTC_BK8R             (STM32U5_RTC_BASE+STM32U5_RTC_BK8R_OFFSET)
#define STM32U5_RTC_BK9R             (STM32U5_RTC_BASE+STM32U5_RTC_BK9R_OFFSET)
#define STM32U5_RTC_BK10R             (STM32U5_RTC_BASE+STM32U5_RTC_BK10R_OFFSET)
#define STM32U5_RTC_BK11R             (STM32U5_RTC_BASE+STM32U5_RTC_BK11R_OFFSET)
#define STM32U5_RTC_BK12R             (STM32U5_RTC_BASE+STM32U5_RTC_BK12R_OFFSET)
#define STM32U5_RTC_BK13R             (STM32U5_RTC_BASE+STM32U5_RTC_BK13R_OFFSET)
#define STM32U5_RTC_BK14R             (STM32U5_RTC_BASE+STM32U5_RTC_BK14R_OFFSET)
#define STM32U5_RTC_BK15R             (STM32U5_RTC_BASE+STM32U5_RTC_BK15R_OFFSET)
#define STM32U5_RTC_BK16R             (STM32U5_RTC_BASE+STM32U5_RTC_BK16R_OFFSET)
#define STM32U5_RTC_BK17R             (STM32U5_RTC_BASE+STM32U5_RTC_BK17R_OFFSET)
#define STM32U5_RTC_BK18R             (STM32U5_RTC_BASE+STM32U5_RTC_BK18R_OFFSET)
#define STM32U5_RTC_BK19R             (STM32U5_RTC_BASE+STM32U5_RTC_BK19R_OFFSET)
#define STM32U5_RTC_BK20R             (STM32U5_RTC_BASE+STM32U5_RTC_BK20R_OFFSET)
#define STM32U5_RTC_BK21R             (STM32U5_RTC_BASE+STM32U5_RTC_BK21R_OFFSET)
#define STM32U5_RTC_BK22R             (STM32U5_RTC_BASE+STM32U5_RTC_BK22R_OFFSET)
#define STM32U5_RTC_BK23R             (STM32U5_RTC_BASE+STM32U5_RTC_BK23R_OFFSET)
#define STM32U5_RTC_BK24R             (STM32U5_RTC_BASE+STM32U5_RTC_BK24R_OFFSET)
#define STM32U5_RTC_BK25R             (STM32U5_RTC_BASE+STM32U5_RTC_BK25R_OFFSET)
#define STM32U5_RTC_BK26R             (STM32U5_RTC_BASE+STM32U5_RTC_BK26R_OFFSET)
#define STM32U5_RTC_BK27R             (STM32U5_RTC_BASE+STM32U5_RTC_BK27R_OFFSET)
#define STM32U5_RTC_BK28R             (STM32U5_RTC_BASE+STM32U5_RTC_BK28R_OFFSET)
#define STM32U5_RTC_BK29R             (STM32U5_RTC_BASE+STM32U5_RTC_BK29R_OFFSET)
#define STM32U5_RTC_BK30R             (STM32U5_RTC_BASE+STM32U5_RTC_BK30R_OFFSET)
#define STM32U5_RTC_BK31R             (STM32U5_RTC_BASE+STM32U5_RTC_BK31R_OFFSET)

/* Register Bitfield Definitions *********************************************/

/* RTC Time Register */

#define RTC_TR_PM                      (1 << 22) /* Bit 22: AM/PM notation */
#define RTC_TR_HT_SHIFT                (20)      /* Bits 20-21: Hour tens in BCD format */
#define RTC_TR_HT_MASK                 (3 << RTC_TR_HT_SHIFT)
#define RTC_TR_HU_SHIFT                (16)      /* Bits 16-19: Hour units in BCD format */
#define RTC_TR_HU_MASK                 (0x0f << RTC_TR_HU_SHIFT)
#define RTC_TR_MNT_SHIFT               (12)      /* Bits 12-14: Minute tens in BCD format */
#define RTC_TR_MNT_MASK                (7 << RTC_TR_MNT_SHIFT)
#define RTC_TR_MNU_SHIFT               (8)       /* Bits 8-11: Minute units in BCD format */
#define RTC_TR_MNU_MASK                (0x0f << RTC_TR_MNU_SHIFT)
#define RTC_TR_ST_SHIFT                (4)       /* Bits 4-6: Second tens in BCD format */
#define RTC_TR_ST_MASK                 (7 << RTC_TR_ST_SHIFT)
#define RTC_TR_SU_SHIFT                (0)       /* Bits 0-3: Second units in BCD format */
#define RTC_TR_SU_MASK                 (0x0f << RTC_TR_SU_SHIFT)

/* RTC Date Register */

#define RTC_DR_YT_SHIFT                (28)      /* Bits 28-31: Year tens in BCD format */
#define RTC_DR_YT_MASK                 (0x0f << RTC_DR_YT_SHIFT)
#define RTC_DR_YU_SHIFT                (24)      /* Bits 24-27: Year units in BCD format */
#define RTC_DR_YU_MASK                 (0x0f << RTC_DR_YU_SHIFT)
#define RTC_DR_WDU_SHIFT               (13)      /* Bits 13-15: Week day units */
#define RTC_DR_WDU_MASK                (7 << RTC_DR_WDU_SHIFT)
#define RTC_DR_MT                      (1 << 12) /* Bit 12: Month tens in BCD format */
#define RTC_DR_MU_SHIFT                (8)       /* Bits 8-11: Month units in BCD format */
#define RTC_DR_MU_MASK                 (0x0f << RTC_DR_MU_SHIFT)
#define RTC_DR_DT_SHIFT                (4)       /* Bits 4-5: Date tens in BCD format */
#define RTC_DR_DT_MASK                 (3 << RTC_DR_DT_SHIFT)
#define RTC_DR_DU_SHIFT                (0)       /* Bits 0-3: Date units in BCD format */
#define RTC_DR_DU_MASK                 (0x0f << RTC_DR_DU_SHIFT)

/* RTC Control Register */

#define RTC_CR_ITSE                    (1 << 24) /* Bit 24: Timestamp on internal event output enable */
#define RTC_CR_COE                     (1 << 23) /* Bit 23: Calibration output enable */
#define RTC_CR_OSEL_SHIFT              (21)      /* Bits 21-22: Output selection */
#define RTC_CR_OSEL_MASK               (3 << RTC_CR_OSEL_SHIFT)
#  define RTC_CR_OSEL_DISABLED         (0 << RTC_CR_OSEL_SHIFT) /* Output disabled */
#  define RTC_CR_OSEL_ALARMA           (1 << RTC_CR_OSEL_SHIFT) /* Alarm A output enabled */
#  define RTC_CR_OSEL_ALARMB           (2 << RTC_CR_OSEL_SHIFT) /* Alarm B output enabled */
#  define RTC_CR_OSEL_WAKEUP           (3 << RTC_CR_OSEL_SHIFT) /* Wakeup output enabled */
#define RTC_CR_POL                     (1 << 20) /* Bit 20: Output polarity */
#define RTC_CR_COSEL                   (1 << 19) /* Bit 19: Calibration output selection */
#define RTC_CR_BCK                     (1 << 18) /* Bit 18: Backup */
#define RTC_CR_SUB1H                   (1 << 17) /* Bit 17: Subtract 1 hour (winter time change) */
#define RTC_CR_ADD1H                   (1 << 16) /* Bit 16: Add 1 hour (summer time change) */
#define RTC_CR_TSIE                    (1 << 15) /* Bit 15: Time-stamp interrupt enable */
#define RTC_CR_WUTIE                   (1 << 14) /* Bit 14: Wakeup timer interrupt enable */
#define RTC_CR_ALRBIE                  (1 << 13) /* Bit 13: Alarm B interrupt enable */
#define RTC_CR_ALRAIE                  (1 << 12) /* Bit 12: Alarm A interrupt enable */
#define RTC_CR_TSE                     (1 << 11) /* Bit 11: Time stamp enable */
#define RTC_CR_WUTE                    (1 << 10) /* Bit 10: Wakeup timer enable */
#define RTC_CR_ALRBE                   (1 << 9)  /* Bit 9: Alarm B enable */
#define RTC_CR_ALRAE                   (1 << 8)  /* Bit 8: Alarm A enable */
#define RTC_CR_FMT                     (1 << 6)  /* Bit 6: Hour format */
#define RTC_CR_BYPSHAD                 (1 << 5)  /* Bit 5: Bypass the shadow registers */
#define RTC_CR_REFCKON                 (1 << 4)  /* Bit 4: Reference clock detection enable */
#define RTC_CR_TSEDGE                  (1 << 3)  /* Bit 3: Time-stamp event active edge */
#define RTC_CR_WUCKSEL_SHIFT           (0)       /* Bits 0-2: Wakeup clock selection */
#define RTC_CR_WUCKSEL_MASK            (7 << RTC_CR_WUCKSEL_SHIFT)
#  define RTC_CR_WUCKSEL_RTC_DIV16     (0 << RTC_CR_WUCKSEL_SHIFT) /* RTC/16 */
#  define RTC_CR_WUCKSEL_RTC_DIV8      (1 << RTC_CR_WUCKSEL_SHIFT) /* RTC/8 */
#  define RTC_CR_WUCKSEL_RTC_DIV4      (2 << RTC_CR_WUCKSEL_SHIFT) /* RTC/4 */
#  define RTC_CR_WUCKSEL_RTC_DIV2      (3 << RTC_CR_WUCKSEL_SHIFT) /* RTC/2 */
#  define RTC_CR_WUCKSEL_CK_SPRE_16BIT (4 << RTC_CR_WUCKSEL_SHIFT) /* CK_SPRE with a counter period of 16-bit */
#  define RTC_CR_WUCKSEL_CK_SPRE_17BIT (6 << RTC_CR_WUCKSEL_SHIFT) /* CK_SPRE with a counter period of 17-bit */

/* RTC Initialization and Status Register */

#define RTC_ISR_ITSF                   (1 << 17) /* Bit 17: Internal time-stamp flag */
#define RTC_ISR_RECALPF                (1 << 16) /* Bit 16: Recalibration pending flag */
#define RTC_ISR_TAMP3F                 (1 << 15) /* Bit 15: Tamper 3 detection flag */
#define RTC_ISR_TAMP2F                 (1 << 14) /* Bit 14: Tamper 2 detection flag */
#define RTC_ISR_TAMP1F                 (1 << 13) /* Bit 13: Tamper 1 detection flag */
#define RTC_ISR_TSOVF                  (1 << 12) /* Bit 12: Time-stamp overflow flag */
#define RTC_ISR_TSF                    (1 << 11) /* Bit 11: Time-stamp flag */
#define RTC_ISR_WUTF                   (1 << 10) /* Bit 10: Wakeup timer flag */
#define RTC_ISR_ALRBF                  (1 << 9)  /* Bit 9: Alarm B flag */
#define RTC_ISR_ALRAF                  (1 << 8)  /* Bit 8: Alarm A flag */
#define RTC_ISR_INIT                   (1 << 7)  /* Bit 7: Initialization mode */
#define RTC_ISR_INITF                  (1 << 6)  /* Bit 6: Initialization flag */
#define RTC_ISR_RSF                    (1 << 5)  /* Bit 5: Registers synchronization flag */
#define RTC_ISR_INITS                  (1 << 4)  /* Bit 4: Initialization status flag */
#define RTC_ISR_SHPF                   (1 << 3)  /* Bit 3: Shift operation pending flag */
#define RTC_ISR_WUTWF                  (1 << 2)  /* Bit 2: Wakeup timer write flag */
#define RTC_ISR_ALRBWF                 (1 << 1)  /* Bit 1: Alarm B write flag */
#define RTC_ISR_ALRAWF                 (1 << 0)  /* Bit 0: Alarm A write flag */

/* RTC Prescaler Register */

#define RTC_PRER_PREDIV_A_SHIFT        (16)      /* Bits 16-23: Asynchronous prescaler factor */
#define RTC_PRER_PREDIV_A_MASK         (0xff << RTC_PRER_PREDIV_A_SHIFT)
#define RTC_PRER_PREDIV_S_SHIFT        (0)       /* Bits 0-14: Synchronous prescaler factor */
#define RTC_PRER_PREDIV_S_MASK         (0x7fff << RTC_PRER_PREDIV_S_SHIFT)

/* RTC Wakeup Timer Register */

#define RTC_WUTR_WUT_SHIFT             (0)       /* Bits 0-15: Wakeup auto-reload value */
#define RTC_WUTR_WUT_MASK              (0xffff << RTC_WUTR_WUT_SHIFT)

/* RTC Alarm Register */

#define RTC_ALRMXR_MSK4                (1 << 31) /* Bit 31: Alarm X date mask */
#define RTC_ALRMXR_WDSEL               (1 << 30) /* Bit 30: Week day selection */
#define RTC_ALRMXR_DT_SHIFT            (28)      /* Bits 28-29: Date tens in BCD format */
#define RTC_ALRMXR_DT_MASK             (3 << RTC_ALRMXR_DT_SHIFT)
#define RTC_ALRMXR_DU_SHIFT            (24)      /* Bits 24-27: Date units or week day in BCD format */
#define RTC_ALRMXR_DU_MASK             (0x0f << RTC_ALRMXR_DU_SHIFT)
#define RTC_ALRMXR_MSK3                (1 << 23) /* Bit 23: Alarm X hours mask */
#define RTC_ALRMXR_PM                  (1 << 22) /* Bit 22: AM/PM notation */
#define RTC_ALRMXR_HT_SHIFT            (20)      /* Bits 20-21: Hour tens in BCD format */
#define RTC_ALRMXR_HT_MASK             (3 << RTC_ALRMXR_HT_SHIFT)
#define RTC_ALRMXR_HU_SHIFT            (16)      /* Bits 16-19: Hour units in BCD format */
#define RTC_ALRMXR_HU_MASK             (0x0f << RTC_ALRMXR_HU_SHIFT)
#define RTC_ALRMXR_MSK2                (1 << 15) /* Bit 15: Alarm X minutes mask */
#define RTC_ALRMXR_MNT_SHIFT           (12)      /* Bits 12-14: Minute tens in BCD format */
#define RTC_ALRMXR_MNT_MASK            (7 << RTC_ALRMXR_MNT_SHIFT)
#define RTC_ALRMXR_MNU_SHIFT           (8)       /* Bits 8-11: Minute units in BCD format */
#define RTC_ALRMXR_MNU_MASK            (0x0f << RTC_ALRMXR_MNU_SHIFT)
#define RTC_ALRMXR_MSK1                (1 << 7)  /* Bit 7: Alarm X seconds mask */
#define RTC_ALRMXR_ST_SHIFT            (4)       /* Bits 4-6: Second tens in BCD format */
#define RTC_ALRMXR_ST_MASK             (7 << RTC_ALRMXR_ST_SHIFT)
#define RTC_ALRMXR_SU_SHIFT            (0)       /* Bits 0-3: Second units in BCD format */
#define RTC_ALRMXR_SU_MASK             (0x0f << RTC_ALRMXR_SU_SHIFT)

/* RTC Write Protection Register */

#define RTC_WPR_KEY_SHIFT              (0)       /* Bits 0-7: Write protection key */
#define RTC_WPR_KEY_MASK               (0xff << RTC_WPR_KEY_SHIFT)

/* RTC Sub Second Register */

#define RTC_SSR_SS_SHIFT               (0)       /* Bits 0-15: Sub second value */
#define RTC_SSR_SS_MASK                (0xffff << RTC_SSR_SS_SHIFT)

/* RTC Shift Control Register */

#define RTC_SHIFTR_ADD1S               (1 << 31) /* Bit 31: Add one second */
#define RTC_SHIFTR_SUBFS_SHIFT         (0)       /* Bits 0-14: Subtract a fraction of a second */
#define RTC_SHIFTR_SUBFS_MASK          (0x7fff << RTC_SHIFTR_SUBFS_SHIFT)

/* RTC Timestamp Time Register */

#define RTC_TSTR_PM                    (1 << 22) /* Bit 22: AM/PM notation */
#define RTC_TSTR_HT_SHIFT              (20)      /* Bits 20-21: Hour tens in BCD format */
#define RTC_TSTR_HT_MASK               (3 << RTC_TSTR_HT_SHIFT)
#define RTC_TSTR_HU_SHIFT              (16)      /* Bits 16-19: Hour units in BCD format */
#define RTC_TSTR_HU_MASK               (0x0f << RTC_TSTR_HU_SHIFT)
#define RTC_TSTR_MNT_SHIFT             (12)      /* Bits 12-14: Minute tens in BCD format */
#define RTC_TSTR_MNT_MASK              (7 << RTC_TSTR_MNT_SHIFT)
#define RTC_TSTR_MNU_SHIFT             (8)       /* Bits 8-11: Minute units in BCD format */
#define RTC_TSTR_MNU_MASK              (0x0f << RTC_TSTR_MNU_SHIFT)
#define RTC_TSTR_ST_SHIFT              (4)       /* Bits 4-6: Second tens in BCD format */
#define RTC_TSTR_ST_MASK               (7 << RTC_TSTR_ST_SHIFT)
#define RTC_TSTR_SU_SHIFT              (0)       /* Bits 0-3: Second units in BCD format */
#define RTC_TSTR_SU_MASK               (0x0f << RTC_TSTR_SU_SHIFT)

/* RTC Timestamp Date Register */

#define RTC_TSDR_WDU_SHIFT             (13)      /* Bits 13-15: Week day units */
#define RTC_TSDR_WDU_MASK              (7 << RTC_TSDR_WDU_SHIFT)
#define RTC_TSDR_MT                    (1 << 12) /* Bit 12: Month tens in BCD format */
#define RTC_TSDR_MU_SHIFT              (8)       /* Bits 8-11: Month units in BCD format */
#define RTC_TSDR_MU_MASK               (0x0f << RTC_TSDR_MU_SHIFT)
#define RTC_TSDR_DT_SHIFT              (4)       /* Bits 4-5: Date tens in BCD format */
#define RTC_TSDR_DT_MASK               (3 << RTC_TSDR_DT_SHIFT)
#define RTC_TSDR_DU_SHIFT              (0)       /* Bits 0-3: Date units in BCD format */
#define RTC_TSDR_DU_MASK               (0x0f << RTC_TSDR_DU_SHIFT)

/* RTC Timestamp Sub Second Register */

#define RTC_TSSSR_SS_SHIFT             (0)       /* Bits 0-15: Sub second value */
#define RTC_TSSSR_SS_MASK              (0xffff << RTC_TSSSR_SS_SHIFT)

/* RTC Calibration Register */

#define RTC_CALR_CALP                  (1 << 15) /* Bit 15: Increase frequency of RTC by 488.5 ppm */
#define RTC_CALR_CALW8                 (1 << 14) /* Bit 14: Use an 8-second calibration cycle period */
#define RTC_CALR_CALW16                (1 << 13) /* Bit 13: Use a 16-second calibration cycle period */
#define RTC_CALR_CALM_SHIFT            (0)       /* Bits 0-8: Calibration minus */
#define RTC_CALR_CALM_MASK             (0x1ff << RTC_CALR_CALM_SHIFT)

/* RTC Tamper Configuration Register */

#define RTC_TAMPCR_TAMP3MF             (1 << 24) /* Bit 24: Tamper 3 mask flag */
#define RTC_TAMPCR_TAMP3NOERASE        (1 << 23) /* Bit 23: Tamper 3 no erase */
#define RTC_TAMPCR_TAMP3IE             (1 << 22) /* Bit 22: Tamper 3 interrupt enable */
#define RTC_TAMPCR_TAMP2MF             (1 << 21) /* Bit 21: Tamper 2 mask flag */
#define RTC_TAMPCR_TAMP2NOERASE        (1 << 20) /* Bit 20: Tamper 2 no erase */
#define RTC_TAMPCR_TAMP2IE             (1 << 19) /* Bit 19: Tamper 2 interrupt enable */
#define RTC_TAMPCR_TAMP1MF             (1 << 18) /* Bit 18: Tamper 1 mask flag */
#define RTC_TAMPCR_TAMP1NOERASE        (1 << 17) /* Bit 17: Tamper 1 no erase */
#define RTC_TAMPCR_TAMP1IE             (1 << 16) /* Bit 16: Tamper 1 interrupt enable */
#define RTC_TAMPCR_TAMPTS              (1 << 15) /* Bit 15: Time stamp on tamper detection */
#define RTC_TAMPCR_TAMPFREQ_SHIFT      (11)      /* Bits 11-13: Tamper sampling frequency */
#define RTC_TAMPCR_TAMPFREQ_MASK       (7 << RTC_TAMPCR_TAMPFREQ_SHIFT)
#define RTC_TAMPCR_TAMPFLT_SHIFT       (8)       /* Bits 8-9: Tamper filter count */
#define RTC_TAMPCR_TAMPFLT_MASK        (3 << RTC_TAMPCR_TAMPFLT_SHIFT)
#define RTC_TAMPCR_TAMPPRCH_SHIFT      (6)       /* Bits 6-7: Tamper precharge duration */
#define RTC_TAMPCR_TAMPPRCH_MASK       (3 << RTC_TAMPCR_TAMPPRCH_SHIFT)
#define RTC_TAMPCR_TAMPPUDIS           (1 << 5)  /* Bit 5: Tamper pull-up disable */
#define RTC_TAMPCR_TAMPIE              (1 << 2)  /* Bit 2: Tamper interrupt enable */
#define RTC_TAMPCR_TAMP1TRG            (1 << 1)  /* Bit 1: Active level for tamper 1 */
#define RTC_TAMPCR_TAMP1E              (1 << 0)  /* Bit 0: Tamper 1 detection enable */

/* RTC Option Register */

#define RTC_OR_OUT_RMP_SHIFT           (2)       /* Bits 2-3: RTC_OUT remap */
#define RTC_OR_OUT_RMP_MASK            (3 << RTC_OR_OUT_RMP_SHIFT)
#define RTC_OR_ALARMOUTTYPE            (1 << 0)  /* Bit 0: AFO_ALARM output type */

#endif /* __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_RTCC_H */