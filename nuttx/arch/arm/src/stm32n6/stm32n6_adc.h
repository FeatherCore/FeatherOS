/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_adc.h
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

#ifndef __ARCH_ARM_SRC_STM32N6_STM32N6_ADC_H
#define __ARCH_ARM_SRC_STM32N6_STM32N6_ADC_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>
#include <nuttx/analog/adc.h>
#include <nuttx/analog/ioctl.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define STM32_ADC1_BASE             (STM32N6_PERIPH_BASE + 0x00022000)
#define STM32_ADC2_BASE             (STM32N6_PERIPH_BASE + 0x00022100)

/* Register Offsets */
#define STM32_ADC_ISR_OFFSET        0x00
#define STM32_ADC_IER_OFFSET        0x04
#define STM32_ADC_CR_OFFSET         0x08
#define STM32_ADC_CFGR1_OFFSET      0x0C
#define STM32_ADC_CFGR2_OFFSET      0x10
#define STM32_ADC_SMPR1_OFFSET      0x14
#define STM32_ADC_SMPR2_OFFSET      0x18
#define STM32_ADC_PCSEL_OFFSET      0x1C
#define STM32_ADC_AWD1TR_OFFSET     0x20
#define STM32_ADC_AWD2TR_OFFSET     0x24
#define STM32_ADC_CHSELR_OFFSET     0x28
#define STM32_ADC_AWD3TR_OFFSET     0x2C
#define STM32_ADC_DR_OFFSET         0x40
#define STM32_ADC_AWD2CR_OFFSET     0x34
#define STM32_ADC_AWD3CR_OFFSET     0x38
#define STM32_ADC_CALFACT_OFFSET    0x44
#define STM32_ADC_CCR_OFFSET        0x308

/* ADC Interrupt and Status Register (ISR) */
#define ADC_ISR_ADRDY               (1 << 0)   /* ADC Ready */
#define ADC_ISR_EOSMP               (1 << 1)   /* End of Sampling */
#define ADC_ISR_EOC                 (1 << 2)   /* End of Conversion */
#define ADC_ISR_EOS                 (1 << 3)   /* End of Sequence */
#define ADC_ISR_OVR                 (1 << 4)   /* Overrun */
#define ADC_ISR_AWD1                (1 << 7)   /* Analog watchdog 1 */
#define ADC_ISR_AWD2                (1 << 8)   /* Analog watchdog 2 */
#define ADC_ISR_AWD3                (1 << 9)   /* Analog watchdog 3 */
#define ADC_ISR_EOCAL               (1 << 11)  /* End of calibration */

/* ADC Interrupt Enable Register (IER) */
#define ADC_IER_ADRDYIE             (1 << 0)   /* Ready interrupt enable */
#define ADC_IER_EOSMPIE             (1 << 1)   /* End of sampling interrupt */
#define ADC_IER_EOCIE               (1 << 2)   /* End of conversion interrupt */
#define ADC_IER_EOSIE               (1 << 3)   /* End of sequence interrupt */
#define ADC_IER_OVRIE               (1 << 4)   /* Overrun interrupt enable */
#define ADC_IER_AWD1IE              (1 << 7)   /* Analog watchdog 1 interrupt */
#define ADC_IER_AWD2IE              (1 << 8)   /* Analog watchdog 2 interrupt */
#define ADC_IER_AWD3IE              (1 << 9)   /* Analog watchdog 3 interrupt */
#define ADC_IER_EOCALIE             (1 << 11)  /* End of calibration interrupt */

/* ADC Control Register (CR) */
#define ADC_CR_ADEN                 (1 << 0)   /* ADC enable */
#define ADC_CR_ADDIS                (1 << 1)   /* ADC disable */
#define ADC_CR_ADSTART              (1 << 2)   /* ADC start conversion */
#define ADC_CR_ADSTP                (1 << 3)   /* ADC stop conversion */
#define ADC_CR_ADVREGEN             (1 << 28)  /* Voltage regulator enable */

/* ADC Configuration Register 1 (CFGR1) */
#define ADC_CFGR1_DMAEN             (1 << 0)   /* DMA enable */
#define ADC_CFGR1_DMACFG            (1 << 1)   /* DMA configuration */
#define ADC_CFGR1_SCANDIR           (1 << 2)   /* Scan sequence direction */
#define ADC_CFGR1_RES_SHIFT         3          /* Resolution shift */
#define ADC_CFGR1_RES_MASK          (3 << 3)   /* Resolution mask */
#define ADC_CFGR1_RES_12BIT         (0 << 3)   /* 12-bit resolution */
#define ADC_CFGR1_RES_10BIT         (1 << 3)   /* 10-bit resolution */
#define ADC_CFGR1_RES_8BIT          (2 << 3)   /* 8-bit resolution */
#define ADC_CFGR1_RES_6BIT          (3 << 3)   /* 6-bit resolution */
#define ADC_CFGR1_ALIGN             (1 << 5)   /* Data align */
#define ADC_CFGR1_EXTSEL_SHIFT      6          /* External trigger selection */
#define ADC_CFGR1_EXTSEL_MASK       (0x1F << 6)
#define ADC_CFGR1_EXTEN_SHIFT       10         /* External trigger enable */
#define ADC_CFGR1_EXTEN_MASK        (3 << 10)
#define ADC_CFGR1_EXTEN_DISABLED    (0 << 10)  /* Trigger disabled */
#define ADC_CFGR1_EXTEN_RISING      (1 << 10)  /* Rising edge trigger */
#define ADC_CFGR1_EXTEN_FALLING     (2 << 10)  /* Falling edge trigger */
#define ADC_CFGR1_EXTEN_BOTH        (3 << 10)  /* Both edges trigger */
#define ADC_CFGR1_OVRMOD            (1 << 12)  /* Overrun mode */
#define ADC_CFGR1_CONT              (1 << 13)  /* Continuous mode */
#define ADC_CFGR1_WAIT              (1 << 14)  /* Wait conversion mode */
#define ADC_CFGR1_AUTOFF            (1 << 15)  /* Auto poweroff */
#define ADC_CFGR1_DISCEN            (1 << 16)  /* Discontinuous mode */
#define ADC_CFGR1_DISCNUM_SHIFT     17         /* Discontinuous number shift */
#define ADC_CFGR1_DISCNUM_MASK      (7 << 17)  /* Discontinuous number mask */
#define ADC_CFGR1_AWD1SGL           (1 << 22)  /* Analog watchdog 1 single */
#define ADC_CFGR1_AWD1EN            (1 << 23)  /* Analog watchdog 1 enable */
#define ADC_CFGR1_AWD1CH_SHIFT      26         /* Analog watchdog 1 channel */
#define ADC_CFGR1_AWD1CH_MASK       (0x1F << 26)

/* ADC Configuration Register 2 (CFGR2) */
#define ADC_CFGR2_OVSE              (1 << 0)   /* Oversampler enable */
#define ADC_CFGR2_OVSR_SHIFT        2          /* Oversampling ratio */
#define ADC_CFGR2_OVSR_MASK         (7 << 2)
#define ADC_CFGR2_OVSS_SHIFT        5          /* Oversampling shift */
#define ADC_CFGR2_OVSS_MASK         (0x0F << 5)
#define ADC_CFGR2_TOVS              (1 << 9)   /* Triggered oversampling */
#define ADC_CFGR2_LFTRIG            (1 << 29)  /* Low frequency trigger mode */
#define ADC_CFGR2_CKMODE_SHIFT      30         /* Clock mode */
#define ADC_CFGR2_CKMODE_MASK       (3 << 30)

/* ADC Sample Time Register 1 (SMPR1) */
#define ADC_SMPR1_SMP0_SHIFT        0          /* Channel 0 sample time */
#define ADC_SMPR1_SMP0_MASK         (7 << 0)
#define ADC_SMPR1_SMP1_SHIFT        3          /* Channel 1 sample time */
#define ADC_SMPR1_SMP1_MASK         (7 << 3)
#define ADC_SMPR1_SMP2_SHIFT        6          /* Channel 2 sample time */
#define ADC_SMPR1_SMP2_MASK         (7 << 6)
#define ADC_SMPR1_SMP9_SHIFT        27         /* Channel 9 sample time */
#define ADC_SMPR1_SMP9_MASK         (7 << 27)

/* ADC Sample Time Register 2 (SMPR2) */
#define ADC_SMPR2_SMP10_SHIFT       0          /* Channel 10 sample time */
#define ADC_SMPR2_SMP10_MASK        (7 << 0)
#define ADC_SMPR2_SMP18_SHIFT       24         /* Channel 18 sample time */
#define ADC_SMPR2_SMP18_MASK        (7 << 24)

/* Common Control Register (CCR) */
#define ADC_CCR_CKMODE_SHIFT        16         /* ADC clock mode */
#define ADC_CCR_CKMODE_MASK         (3 << 16)
#define ADC_CCR_PRESC_SHIFT         18         /* Prescaler */
#define ADC_CCR_PRESC_MASK          (0x0F << 18)

/* ADC Channel Selection Register (CHSELR) */
#define ADC_CHSELR_CHSEL(n)         (1 << (n)) /* Channel selection */

/* ADC resolution definitions */
#define ADC_RESOLUTION_12BIT        12
#define ADC_RESOLUTION_10BIT        10
#define ADC_RESOLUTION_8BIT         8
#define ADC_RESOLUTION_6BIT         6

/* ADC sampling time definitions */
#define ADC_SMPR_SMP_1_5CYCLES      0  /* 1.5 cycles */
#define ADC_SMPR_SMP_3_5CYCLES      1  /* 3.5 cycles */
#define ADC_SMPR_SMP_7_5CYCLES      2  /* 7.5 cycles */
#define ADC_SMPR_SMP_12_5CYCLES     3  /* 12.5 cycles */
#define ADC_SMPR_SMP_19_5CYCLES     4  /* 19.5 cycles */
#define ADC_SMPR_SMP_39_5CYCLES     5  /* 39.5 cycles */
#define ADC_SMPR_SMP_79_5CYCLES     6  /* 79.5 cycles */
#define ADC_SMPR_SMP_160_5CYCLES    7  /* 160.5 cycles */

#define STM32N6_ADC_MAX_CHANNELS    20

/****************************************************************************
 * Public Types
 ****************************************************************************/

struct stm32n6_adc_s
{
  const struct adc_ops_s *ops;    /* Arch-specific ADC operations */
  uint32_t base;                  /* ADC register base address */
  uint8_t irq;                    /* ADC interrupt number */
  uint8_t chanlist[STM32N6_ADC_MAX_CHANNELS];
  uint8_t nchannels;              /* Number of channels in sequence */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

int stm32n6_adc_initialize(void);
FAR struct adc_dev_s *stm32n6_adcinitialize(int intf,
                                            FAR const uint8_t *chanlist,
                                            int nchannels);

#endif /* __ARCH_ARM_SRC_STM32N6_STM32N6_ADC_H */
