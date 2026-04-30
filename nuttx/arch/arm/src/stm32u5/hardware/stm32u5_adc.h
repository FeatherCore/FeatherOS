/****************************************************************************
 * arch/arm/src/stm32u5/hardware/stm32u5_adc.h
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

#ifndef __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_ADC_H
#define __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_ADC_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Register Offsets for ADC1 (0x42028000) **********************************/

#define STM32U5_ADC_ISR_OFFSET         0x0000  /* ADC interrupt and status register */
#define STM32U5_ADC_IER_OFFSET         0x0004  /* ADC interrupt enable register */
#define STM32U5_ADC_CR_OFFSET          0x0008  /* ADC control register */
#define STM32U5_ADC_CFGR_OFFSET        0x000c  /* ADC configuration register */
#define STM32U5_ADC_CFGR2_OFFSET       0x0010  /* ADC configuration register 2 */
#define STM32U5_ADC_CFGR3_OFFSET       0x0014  /* ADC configuration register 3 */
#define STM32U5_ADC_SMPR1_OFFSET       0x0018  /* ADC sample time register 1 */
#define STM32U5_ADC_SMPR2_OFFSET       0x001c  /* ADC sample time register 2 */
#define STM32U5_ADC_PCSETR_OFFSET     0x0020  /* ADC pre channel selection register */
#define STM32U5_ADC_TR1_OFFSET         0x0024  /* ADC watchdog threshold register 1 */
#define STM32U5_ADC_TR2_OFFSET         0x0028  /* ADC watchdog threshold register 2 */
#define STM32U5_ADC_TR3_OFFSET         0x002c  /* ADC watchdog threshold register 3 */
#define STM32U5_ADC_SQR1_OFFSET        0x0030  /* ADC regular sequence register 1 */
#define STM32U5_ADC_SQR2_OFFSET        0x0034  /* ADC regular sequence register 2 */
#define STM32U5_ADC_SQR3_OFFSET        0x0038  /* ADC regular sequence register 3 */
#define STM32U5_ADC_SQR4_OFFSET        0x003c  /* ADC regular sequence register 4 */
#define STM32U5_ADC_DR_OFFSET          0x0040  /* ADC regular data register */
#define STM32U5_ADC_JSQR_OFFSET        0x004c  /* ADC injected sequence register */
#define STM32U5_ADC_OFR1_OFFSET        0x0060  /* ADC offset register 1 */
#define STM32U5_ADC_OFR2_OFFSET        0x0064  /* ADC offset register 2 */
#define STM32U5_ADC_OFR3_OFFSET        0x0068  /* ADC offset register 3 */
#define STM32U5_ADC_OFR4_OFFSET        0x006c  /* ADC offset register 4 */
#define STM32U5_ADC_JDR1_OFFSET        0x0080  /* ADC injected data register 1 */
#define STM32U5_ADC_JDR2_OFFSET        0x0084  /* ADC injected data register 2 */
#define STM32U5_ADC_JDR3_OFFSET        0x0088  /* ADC injected data register 3 */
#define STM32U5_ADC_JDR4_OFFSET        0x008c  /* ADC injected data register 4 */
#define STM32U5_ADC_AWD2CR_OFFSET      0x00a0  /* ADC analog watchdog 2 configuration register */
#define STM32U5_ADC_AWD3CR_OFFSET      0x00a4  /* ADC analog watchdog 3 configuration register */
#define STM32U5_ADC_AWDFILT_OFFSET     0x00a8  /* ADC watchdog filter register */
#define STM32U5_ADC_CALFACT_OFFSET     0x00b4  /* ADC calibration factors */
#define STM32U5_ADC_CALFACT2_OFFSET   0x00b8  /* ADC calibration factors 2 */
#define STM32U5_ADC_OR_OFFSET          0x00d0  /* ADC option register */
#define STM32U5_ADC_VMIDB_OFFSET       0x00d4  /* ADC Vbat/Intref monitoring register */
#define STM32U5_ADC_DIFSEL_OFFSET      0x00e0  /* ADC differential mode selection register */
#define STM32U5_ADC_DLYR_OFFSET       0x00f0  /* ADC delay selection register */

/* ADC Common Registers (for ADC1/ADC4) */

#define STM32U5_ADC_CSR_OFFSET         0x0000  /* ADC Common status register */
#define STM32U5_ADC_CCR_OFFSET         0x0008  /* ADC Common control register */
#define STM32U5_ADC_CDR_OFFSET         0x000c  /* ADC Common regular data register for dual mode */

/* Register Addresses for ADC1 **********************************************/

#define STM32U5_ADC1_ISR              (STM32_ADC1_BASE+STM32U5_ADC_ISR_OFFSET)
#define STM32U5_ADC1_IER              (STM32_ADC1_BASE+STM32U5_ADC_IER_OFFSET)
#define STM32U5_ADC1_CR                (STM32_ADC1_BASE+STM32U5_ADC_CR_OFFSET)
#define STM32U5_ADC1_CFGR             (STM32_ADC1_BASE+STM32U5_ADC_CFGR_OFFSET)
#define STM32U5_ADC1_CFGR2            (STM32_ADC1_BASE+STM32U5_ADC_CFGR2_OFFSET)
#define STM32U5_ADC1_CFGR3            (STM32_ADC1_BASE+STM32U5_ADC_CFGR3_OFFSET)
#define STM32U5_ADC1_SMPR1            (STM32_ADC1_BASE+STM32U5_ADC_SMPR1_OFFSET)
#define STM32U5_ADC1_SMPR2            (STM32_ADC1_BASE+STM32U5_ADC_SMPR2_OFFSET)
#define STM32U5_ADC1_PCSETR           (STM32_ADC1_BASE+STM32U5_ADC_PCSETR_OFFSET)
#define STM32U5_ADC1_TR1              (STM32_ADC1_BASE+STM32U5_ADC_TR1_OFFSET)
#define STM32U5_ADC1_TR2              (STM32_ADC1_BASE+STM32U5_ADC_TR2_OFFSET)
#define STM32U5_ADC1_TR3              (STM32_ADC1_BASE+STM32U5_ADC_TR3_OFFSET)
#define STM32U5_ADC1_SQR1             (STM32_ADC1_BASE+STM32U5_ADC_SQR1_OFFSET)
#define STM32U5_ADC1_SQR2             (STM32_ADC1_BASE+STM32U5_ADC_SQR2_OFFSET)
#define STM32U5_ADC1_SQR3             (STM32_ADC1_BASE+STM32U5_ADC_SQR3_OFFSET)
#define STM32U5_ADC1_SQR4             (STM32_ADC1_BASE+STM32U5_ADC_SQR4_OFFSET)
#define STM32U5_ADC1_DR               (STM32_ADC1_BASE+STM32U5_ADC_DR_OFFSET)
#define STM32U5_ADC1_JSQR             (STM32_ADC1_BASE+STM32U5_ADC_JSQR_OFFSET)
#define STM32U5_ADC1_OFR1             (STM32_ADC1_BASE+STM32U5_ADC_OFR1_OFFSET)
#define STM32U5_ADC1_OFR2             (STM32_ADC1_BASE+STM32U5_ADC_OFR2_OFFSET)
#define STM32U5_ADC1_OFR3             (STM32_ADC1_BASE+STM32U5_ADC_OFR3_OFFSET)
#define STM32U5_ADC1_OFR4             (STM32_ADC1_BASE+STM32U5_ADC_OFR4_OFFSET)
#define STM32U5_ADC1_JDR1             (STM32_ADC1_BASE+STM32U5_ADC_JDR1_OFFSET)
#define STM32U5_ADC1_JDR2             (STM32_ADC1_BASE+STM32U5_ADC_JDR2_OFFSET)
#define STM32U5_ADC1_JDR3             (STM32_ADC1_BASE+STM32U5_ADC_JDR3_OFFSET)
#define STM32U5_ADC1_JDR4             (STM32_ADC1_BASE+STM32U5_ADC_JDR4_OFFSET)
#define STM32U5_ADC1_AWD2CR           (STM32_ADC1_BASE+STM32U5_ADC_AWD2CR_OFFSET)
#define STM32U5_ADC1_AWD3CR           (STM32_ADC1_BASE+STM32U5_ADC_AWD3CR_OFFSET)
#define STM32U5_ADC1_AWDFILT          (STM32_ADC1_BASE+STM32U5_ADC_AWDFILT_OFFSET)
#define STM32U5_ADC1_CALFACT          (STM32_ADC1_BASE+STM32U5_ADC_CALFACT_OFFSET)
#define STM32U5_ADC1_CALFACT2         (STM32_ADC1_BASE+STM32U5_ADC_CALFACT2_OFFSET)
#define STM32U5_ADC1_OR               (STM32_ADC1_BASE+STM32U5_ADC_OR_OFFSET)
#define STM32U5_ADC1_VMIDB            (STM32_ADC1_BASE+STM32U5_ADC_VMIDB_OFFSET)
#define STM32U5_ADC1_DIFSEL           (STM32_ADC1_BASE+STM32U5_ADC_DIFSEL_OFFSET)
#define STM32U5_ADC1_DLYR             (STM32_ADC1_BASE+STM32U5_ADC_DLYR_OFFSET)

/* Register Addresses for ADC4 **********************************************/

#define STM32U5_ADC4_ISR              (STM32_ADC4_BASE+STM32U5_ADC_ISR_OFFSET)
#define STM32U5_ADC4_IER              (STM32_ADC4_BASE+STM32U5_ADC_IER_OFFSET)
#define STM32U5_ADC4_CR               (STM32_ADC4_BASE+STM32U5_ADC_CR_OFFSET)
#define STM32U5_ADC4_CFGR             (STM32_ADC4_BASE+STM32U5_ADC_CFGR_OFFSET)
#define STM32U5_ADC4_CFGR2            (STM32_ADC4_BASE+STM32U5_ADC_CFGR2_OFFSET)
#define STM32U5_ADC4_CFGR3            (STM32_ADC4_BASE+STM32U5_ADC_CFGR3_OFFSET)
#define STM32U5_ADC4_SMPR1            (STM32_ADC4_BASE+STM32U5_ADC_SMPR1_OFFSET)
#define STM32U5_ADC4_SMPR2            (STM32_ADC4_BASE+STM32U5_ADC_SMPR2_OFFSET)
#define STM32U5_ADC4_PCSETR           (STM32_ADC4_BASE+STM32U5_ADC_PCSETR_OFFSET)
#define STM32U5_ADC4_TR1              (STM32_ADC4_BASE+STM32U5_ADC_TR1_OFFSET)
#define STM32U5_ADC4_TR2              (STM32_ADC4_BASE+STM32U5_ADC_TR2_OFFSET)
#define STM32U5_ADC4_TR3              (STM32_ADC4_BASE+STM32U5_ADC_TR3_OFFSET)
#define STM32U5_ADC4_SQR1             (STM32_ADC4_BASE+STM32U5_ADC_SQR1_OFFSET)
#define STM32U5_ADC4_SQR2             (STM32_ADC4_BASE+STM32U5_ADC_SQR2_OFFSET)
#define STM32U5_ADC4_SQR3             (STM32_ADC4_BASE+STM32U5_ADC_SQR3_OFFSET)
#define STM32U5_ADC4_SQR4             (STM32_ADC4_BASE+STM32U5_ADC_SQR4_OFFSET)
#define STM32U5_ADC4_DR               (STM32_ADC4_BASE+STM32U5_ADC_DR_OFFSET)
#define STM32U5_ADC4_JSQR             (STM32_ADC4_BASE+STM32U5_ADC_JSQR_OFFSET)
#define STM32U5_ADC4_OFR1             (STM32_ADC4_BASE+STM32U5_ADC_OFR1_OFFSET)
#define STM32U5_ADC4_OFR2             (STM32_ADC4_BASE+STM32U5_ADC_OFR2_OFFSET)
#define STM32U5_ADC4_OFR3             (STM32_ADC4_BASE+STM32U5_ADC_OFR3_OFFSET)
#define STM32U5_ADC4_OFR4             (STM32_ADC4_BASE+STM32U5_ADC_OFR4_OFFSET)
#define STM32U5_ADC4_JDR1             (STM32_ADC4_BASE+STM32U5_ADC_JDR1_OFFSET)
#define STM32U5_ADC4_JDR2             (STM32_ADC4_BASE+STM32U5_ADC_JDR2_OFFSET)
#define STM32U5_ADC4_JDR3             (STM32_ADC4_BASE+STM32U5_ADC_JDR3_OFFSET)
#define STM32U5_ADC4_JDR4             (STM32_ADC4_BASE+STM32U5_ADC_JDR4_OFFSET)
#define STM32U5_ADC4_AWD2CR           (STM32_ADC4_BASE+STM32U5_ADC_AWD2CR_OFFSET)
#define STM32U5_ADC4_AWD3CR           (STM32_ADC4_BASE+STM32U5_ADC_AWD3CR_OFFSET)
#define STM32U5_ADC4_AWDFILT          (STM32_ADC4_BASE+STM32U5_ADC_AWDFILT_OFFSET)
#define STM32U5_ADC4_CALFACT          (STM32_ADC4_BASE+STM32U5_ADC_CALFACT_OFFSET)
#define STM32U5_ADC4_CALFACT2         (STM32_ADC4_BASE+STM32U5_ADC_CALFACT2_OFFSET)
#define STM32U5_ADC4_OR               (STM32_ADC4_BASE+STM32U5_ADC_OR_OFFSET)
#define STM32U5_ADC4_VMIDB            (STM32_ADC4_BASE+STM32U5_ADC_VMIDB_OFFSET)
#define STM32U5_ADC4_DIFSEL           (STM32_ADC4_BASE+STM32U5_ADC_DIFSEL_OFFSET)
#define STM32U5_ADC4_DLYR             (STM32_ADC4_BASE+STM32U5_ADC_DLYR_OFFSET)

/* ADC Common Register Addresses ******************************************/

#define STM32U5_ADC12_CSR            (STM32_ADC1_BASE+0x300+STM32U5_ADC_CSR_OFFSET)
#define STM32U5_ADC12_CCR            (STM32_ADC1_BASE+0x300+STM32U5_ADC_CCR_OFFSET)
#define STM32U5_ADC12_CDR            (STM32_ADC1_BASE+0x300+STM32U5_ADC_CDR_OFFSET)

/* Register Bitfield Definitions *********************************************/

/* ADC Interrupt and Status Register (ISR) */

#define ADC_ISR_ADRDY                (1 << 0)   /* Bit 0: ADC ready */
#define ADC_ISR_EOSMP                (1 << 1)   /* Bit 1: End of sampling flag */
#define ADC_ISR_EOC                  (1 << 2)   /* Bit 2: End of regular conversion flag */
#define ADC_ISR_EOS                  (1 << 3)   /* Bit 3: End of regular sequence flag */
#define ADC_ISR_OVR                  (1 << 4)   /* Bit 4: Overrun flag */
#define ADC_ISR_JEOC                 (1 << 5)   /* Bit 5: End of injected conversion flag */
#define ADC_ISR_JEOS                 (1 << 6)   /* Bit 6: End of injected sequence flag */
#define ADC_ISR_AWD1                 (1 << 7)   /* Bit 7: Analog watchdog 1 flag */
#define ADC_ISR_AWD2                 (1 << 8)   /* Bit 8: Analog watchdog 2 flag */
#define ADC_ISR_AWD3                 (1 << 9)   /* Bit 9: Analog watchdog 3 flag */
#define ADC_ISR_EOSMP3               (1 << 12)  /* Bit 12: End of sampling phase of channel 3 flag */
#define ADC_ISR_EOC3                 (1 << 13)  /* Bit 13: End of conversion of channel 3 flag */
#define ADC_ISR_AWD1_3               (1 << 14)  /* Bit 14: Analog watchdog 1 flag on channel 3 only */

/* ADC Interrupt Enable Register (IER) */

#define ADC_IER_ADRDYIE              (1 << 0)   /* Bit 0: ADC ready interrupt enable */
#define ADC_IER_EOSMPIE              (1 << 1)   /* Bit 1: End of sampling interrupt enable */
#define ADC_IER_EOCIE                (1 << 2)   /* Bit 2: End of regular conversion interrupt enable */
#define ADC_IER_EOSIE                (1 << 3)   /* Bit 3: End of regular sequence interrupt enable */
#define ADC_IER_OVRIE                (1 << 4)   /* Bit 4: Overrun interrupt enable */
#define ADC_IER_JEOCIE              (1 << 5)   /* Bit 5: End of injected conversion interrupt enable */
#define ADC_IER_JEOSIE              (1 << 6)   /* Bit 6: End of injected sequence interrupt enable */
#define ADC_IER_AWD1IE              (1 << 7)   /* Bit 7: Analog watchdog 1 interrupt enable */
#define ADC_IER_AWD2IE              (1 << 8)   /* Bit 8: Analog watchdog 2 interrupt enable */
#define ADC_IER_AWD3IE              (1 << 9)   /* Bit 9: Analog watchdog 3 interrupt enable */

/* ADC Control Register (CR) */

#define ADC_CR_ADSTART               (1 << 2)   /* Bit 2: ADC start regular conversion */
#define ADC_CR_JADSTART              (1 << 3)   /* Bit 3: ADC start injected conversion */
#define ADC_CR_ADSTP                (1 << 5)   /* Bit 5: ADC stop regular conversion */
#define ADC_CR_JADSTP               (1 << 6)   /* Bit 6: ADC stop injected conversion */
#define ADC_CR_ADDISO               (1 << 8)   /* Bit 8: ADC voltage regulator enable */
#define ADC_CR_DEEPPWD              (1 << 9)   /* Bit 9: ADC deep power down enable */
#define ADC_CR_ADCAL                (1 << 11)  /* Bit 11: ADC calibration */
#define ADC_CR_ADCALDIF             (1 << 13)  /* Bit 13: ADC differential mode calibration */
#define ADC_CR_LINCALRDY1           (1 << 22)  /* Bit 22: Linearity calibration ready flag 1 */
#define ADC_CR_LINCALRDY2           (1 << 23)  /* Bit 23: Linearity calibration ready flag 2 */
#define ADC_CR_LINCALRDY3           (1 << 24)  /* Bit 24: Linearity calibration ready flag 3 */
#define ADC_CR_LINCALRDY4           (1 << 25)  /* Bit 25: Linearity calibration ready flag 4 */
#define ADC_CR_LINCALRDY5           (1 << 26)  /* Bit 26: Linearity calibration ready flag 5 */
#define ADC_CR_LINCALRDY6           (1 << 27)  /* Bit 27: Linearity calibration ready flag 6 */
#define ADC_CR_SHSEL_SHIFT          (28)        /* Bits 28-29: Sampling time selection */
#define ADC_CR_SHSEL_MASK           (3 << ADC_CR_SHSEL_SHIFT)
#define ADC_CR_SHSEL_CH6_CH0        (0 << ADC_CR_SHSEL_SHIFT)  /* CH6 selected as sampling time */
#define ADC_CR_SHSEL_CH7_CH1        (1 << ADC_CR_SHSEL_SHIFT)  /* CH7 selected as sampling time */
#define ADC_CR_SHSEL_CH8_CH2        (2 << ADC_CR_SHSEL_SHIFT)  /* CH8 selected as sampling time */
#define ADC_CR_SHSEL_CH9_CH3        (3 << ADC_CR_SHSEL_SHIFT)  /* CH9 selected as sampling time */
#define ADC_CR_BOOSTE               (1 << 30)  /* Bit 30: Boost mode for high impedance sources */

/* ADC Configuration Register (CFGR) */

#define ADC_CFGR_DMAEN              (1 << 0)   /* Bit 0: Direct memory access enable */
#define ADC_CFGR_DMACFG             (1 << 1)   /* Bit 1: DMA configuration */
#define ADC_CFGR_RES_SHIFT          (2)        /* Bits 2-3: Data resolution */
#define ADC_CFGR_RES_MASK           (3 << ADC_CFGR_RES_SHIFT)
#  define ADC_CFGR_RES_12BIT        (0 << ADC_CFGR_RES_SHIFT)  /* 12-bit */
#  define ADC_CFGR_RES_10BIT        (1 << ADC_CFGR_RES_SHIFT)  /* 10-bit */
#  define ADC_CFGR_RES_8BIT         (2 << ADC_CFGR_RES_SHIFT)  /* 8-bit */
#  define ADC_CFGR_RES_6BIT         (3 << ADC_CFGR_RES_SHIFT)  /* 6-bit */
#define ADC_CFGR_ALIGN              (1 << 5)   /* Bit 5: Data alignment */
#define ADC_CFGR_EXTSEL_SHIFT       (6)        /* Bits 6-9: External trigger selection for regular */
#define ADC_CFGR_EXTSEL_MASK        (0xf << ADC_CFGR_EXTSEL_SHIFT)
#define ADC_CFGR_EXTEN_SHIFT        (10)       /* Bits 10-11: External trigger enable and polarity */
#define ADC_CFGR_EXTEN_MASK         (3 << ADC_CFGR_EXTEN_SHIFT)
#  define ADC_CFGR_EXTEN_DISABLED   (0 << ADC_CFGR_EXTEN_SHIFT)  /* Trigger disabled */
#  define ADC_CFGR_EXTEN_RISING     (1 << ADC_CFGR_EXTEN_SHIFT)  /* Trigger on rising edge */
#  define ADC_CFGR_EXTEN_FALLING    (2 << ADC_CFGR_EXTEN_SHIFT)  /* Trigger on falling edge */
#  define ADC_CFGR_EXTEN_BOTH       (3 << ADC_CFGR_EXTEN_SHIFT)  /* Trigger on both edges */
#define ADC_CFGR_OVRMOD            (1 << 12)  /* Bit 12: Overrun mode */
#define ADC_CFGR_DISCEN            (1 << 13)  /* Bit 13: Discontinuous mode on regular channels */
#define ADC_CFGR_JDISCEN           (1 << 14)  /* Bit 14: Discontinuous mode on injected channels */
#define ADC_CFGR_AWD1SGL          (1 << 22)  /* Bit 22: Enable watchdog on a single channel */
#define ADC_CFGR_AWD1EN            (1 << 23)  /* Bit 23: Analog watchdog 1 enable on regular channels */
#define ADC_CFGR_JAWD1EN           (1 << 24)  /* Bit 24: Analog watchdog 1 enable on injected channels */
#define ADC_CFGR_JQDIS             (1 << 25)  /* Bit 25: Injected queue disable */
#define ADC_CFGR_AWDCH1CH_SHIFT     (26)       /* Bits 26-30: Analog watchdog 1 channel selection */
#define ADC_CFGR_AWDCH1CH_MASK     (0x1f << ADC_CFGR_AWDCH1CH_SHIFT)

/* ADC Sample Time Register 1 (SMPR1) */

#define ADC_SMPR1_SMP0_SHIFT       (0)        /* Bits 0-2: Channel 0 sample time selection */
#define ADC_SMPR1_SMP0_MASK        (7 << ADC_SMPR1_SMP0_SHIFT)
#define ADC_SMPR1_SMP1_SHIFT       (3)        /* Bits 3-5: Channel 1 sample time selection */
#define ADC_SMPR1_SMP1_MASK        (7 << ADC_SMPR1_SMP1_SHIFT)
#define ADC_SMPR1_SMP2_SHIFT       (6)        /* Bits 6-8: Channel 2 sample time selection */
#define ADC_SMPR1_SMP2_MASK        (7 << ADC_SMPR1_SMP2_SHIFT)
#define ADC_SMPR1_SMP3_SHIFT       (9)        /* Bits 9-11: Channel 3 sample time selection */
#define ADC_SMPR1_SMP3_MASK        (7 << ADC_SMPR1_SMP3_SHIFT)
#define ADC_SMPR1_SMP4_SHIFT       (12)       /* Bits 12-14: Channel 4 sample time selection */
#define ADC_SMPR1_SMP4_MASK        (7 << ADC_SMPR1_SMP4_SHIFT)
#define ADC_SMPR1_SMP5_SHIFT       (15)       /* Bits 15-17: Channel 5 sample time selection */
#define ADC_SMPR1_SMP5_MASK        (7 << ADC_SMPR1_SMP5_SHIFT)
#define ADC_SMPR1_SMP6_SHIFT       (18)       /* Bits 18-20: Channel 6 sample time selection */
#define ADC_SMPR1_SMP6_MASK        (7 << ADC_SMPR1_SMP6_SHIFT)
#define ADC_SMPR1_SMP7_SHIFT       (21)       /* Bits 21-23: Channel 7 sample time selection */
#define ADC_SMPR1_SMP7_MASK        (7 << ADC_SMPR1_SMP7_SHIFT)
#define ADC_SMPR1_SMP8_SHIFT       (24)       /* Bits 24-26: Channel 8 sample time selection */
#define ADC_SMPR1_SMP8_MASK        (7 << ADC_SMPR1_SMP8_SHIFT)
#define ADC_SMPR1_SMP9_SHIFT       (27)       /* Bits 27-29: Channel 9 sample time selection */
#define ADC_SMPR1_SMP9_MASK        (7 << ADC_SMPR1_SMP9_SHIFT)

/* Sample time values (cycles) */

#define ADC_SMPR_SMP_1CYCLE5        0         /* 1.5 cycles */
#define ADC_SMPR_SMP_3CYCLES5      1         /* 3.5 cycles */
#define ADC_SMPR_SMP_7CYCLES5      2         /* 7.5 cycles */
#define ADC_SMPR_SMP_12CYCLES5     3         /* 12.5 cycles */
#define ADC_SMPR_SMP_19CYCLES5     4         /* 19.5 cycles */
#define ADC_SMPR_SMP_39CYCLES5     5         /* 39.5 cycles */
#define ADC_SMPR_SMP_79CYCLES5     6         /* 79.5 cycles */
#define ADC_SMPR_SMP_160CYCLES5    7         /* 160.5 cycles */

/* ADC Regular Sequence Register 1 (SQR1) */

#define ADC_SQR1_L_SHIFT            (0)        /* Bits 0-3: Regular channel sequence length */
#define ADC_SQR1_L_MASK            (0xf << ADC_SQR1_L_SHIFT)
#define ADC_SQR1_SQ1_SHIFT         (6)        /* Bits 6-10: Regular channel 1 */
#define ADC_SQR1_SQ1_MASK         (0x1f << ADC_SQR1_SQ1_SHIFT)
#define ADC_SQR1_SQ2_SHIFT         (12)       /* Bits 12-16: Regular channel 2 */
#define ADC_SQR1_SQ2_MASK         (0x1f << ADC_SQR1_SQ2_SHIFT)
#define ADC_SQR1_SQ3_SHIFT         (18)       /* Bits 18-22: Regular channel 3 */
#define ADC_SQR1_SQ3_MASK         (0x1f << ADC_SQR1_SQ3_SHIFT)
#define ADC_SQR1_SQ4_SHIFT         (24)       /* Bits 24-28: Regular channel 4 */
#define ADC_SQR1_SQ4_MASK         (0x1f << ADC_SQR1_SQ4_SHIFT)

/* ADC Regular Data Register (DR) */

#define ADC_DR_DATA_SHIFT          (0)        /* Bits 0-15: Regular conversion data */
#define ADC_DR_DATA_MASK          (0xffff << ADC_DR_DATA_SHIFT)

/* ADC Common Control Register (CCR) */

#define ADC_CCR_PRESC_SHIFT        (18)       /* Bits 18-21: ADC prescaler */
#define ADC_CCR_PRESC_MASK        (0xf << ADC_CCR_PRESC_SHIFT)
#  define ADC_CCR_PRESC_DIV1       (0 << ADC_CCR_PRESC_SHIFT)   /* div1 */
#  define ADC_CCR_PRESC_DIV2       (1 << ADC_CCR_PRESC_SHIFT)   /* div2 */
#  define ADC_CCR_PRESC_DIV4       (2 << ADC_CCR_PRESC_SHIFT)   /* div4 */
#  define ADC_CCR_PRESC_DIV6       (3 << ADC_CCR_PRESC_SHIFT)   /* div6 */
#  define ADC_CCR_PRESC_DIV8       (4 << ADC_CCR_PRESC_SHIFT)   /* div8 */
#  define ADC_CCR_PRESC_DIV10      (5 << ADC_CCR_PRESC_SHIFT)   /* div10 */
#  define ADC_CCR_PRESC_DIV12      (6 << ADC_CCR_PRESC_SHIFT)   /* div12 */
#  define ADC_CCR_PRESC_DIV16      (7 << ADC_CCR_PRESC_SHIFT)   /* div16 */
#  define ADC_CCR_PRESC_DIV32      (8 << ADC_CCR_PRESC_SHIFT)   /* div32 */
#  define ADC_CCR_PRESC_DIV64      (9 << ADC_CCR_PRESC_SHIFT)   /* div64 */
#  define ADC_CCR_PRESC_DIV128     (10 << ADC_CCR_PRESC_SHIFT)  /* div128 */
#  define ADC_CCR_PRESC_DIV256     (11 << ADC_CCR_PRESC_SHIFT)  /* div256 */

#define ADC_CCR_VREFEN            (1 << 22)  /* Bit 22: Temperature sensor and Vrefint enable */
#define ADC_CCR_TSEN              (1 << 23)  /* Bit 23: Temperature sensor enable */
#define ADC_CCR_VBATEN            (1 << 24)  /* Bit 24: VBAT enable */

#endif /* __ARCH_ARM_SRC_STM32U5_HARDWARE_STM32U5_ADC_H */