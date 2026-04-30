/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_adc.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_ADC_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_ADC_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* ADC Base Addresses */

#define RA8P_ADC0_BASE                         (0x40290000) /* ADC0 base address */
#define RA8P_ADC1_BASE                         (0x40291000) /* ADC1 base address */

/* ADC Register Offsets */

/* A/D Control Register (ADCR) */

#define RA8P_ADC_ADCR_OFFSET                   (0x000)
#define RA8P_ADC_ADCR_ADST                     (1 << 0)    /* Bit 0: A/D Start */
#define RA8P_ADC_ADCR_ADCS_MASK                (0x6)       /* Bits 1-2: A/D Conversion Select */
#define RA8P_ADC_ADCR_ADCS_SHIFT               (1)
#define RA8P_ADC_ADCR_ADCS_SINGLE              (0x0 << 1)  /* Single scan mode */
#define RA8P_ADC_ADCR_ADCS_CONTINUOUS          (0x1 << 1)  /* Continuous scan mode */
#define RA8P_ADC_ADCR_ADCS_GROUP               (0x2 << 1)  /* Group scan mode */
#define RA8P_ADC_ADCR_TRGE                     (1 << 3)    /* Bit 3: Trigger Enable */
#define RA8P_ADC_ADCR_EXTRGSEL_MASK            (0x30)      /* Bits 4-5: External Trigger Select */
#define RA8P_ADC_ADCR_EXTRGSEL_SHIFT           (4)
#define RA8P_ADC_ADCR_OVWE                     (1 << 6)    /* Bit 6: Overwrite Enable */
#define RA8P_ADC_ADCR_GADCE                    (1 << 7)    /* Bit 7: Group A/D Conversion Enable */

/* A/D Status Register (ADSSTR) */

#define RA8P_ADC_ADSSTR_OFFSET                 (0x004)
#define RA8P_ADC_ADSSTR_ADSST_MASK             (0xFF)      /* Bits 0-7: A/D Sampling State */
#define RA8P_ADC_ADSST_MIN                     (0x04)      /* Minimum sampling state */
#define RA8P_ADC_ADSST_MAX                     (0xFF)      /* Maximum sampling state */

/* A/D Channel Select Register A (ADANSA) */

#define RA8P_ADC_ADANSA_OFFSET                 (0x008)
#define RA8P_ADC_ADANSA_CH_MASK                (0xFFFF)    /* Bits 0-15: Channel Select A */

/* A/D Channel Select Register B (ADANSB) */

#define RA8P_ADC_ADANSB_OFFSET                 (0x00A)
#define RA8P_ADC_ADANSB_CH_MASK                (0xFFFF)    /* Bits 0-15: Channel Select B */

/* A/D Extended Input Control Register (ADEXCR) */

#define RA8P_ADC_ADEXCR_OFFSET                 (0x00C)
#define RA8P_ADC_ADEXCR_SSSEL_MASK             (0x3)       /* Bits 0-1: Sample State Select */
#define RA8P_ADC_ADEXCR_SSSEL_SHIFT            (0)
#define RA8P_ADC_ADEXCR_SSSEL_DEFAULT          (0x0)       /* Default sample state */
#define RA8P_ADC_ADEXCR_SSSEL_CHANNEL          (0x1)       /* Channel-specific sample state */
#define RA8P_ADC_ADEXCR_SSSEL_EXTENDED         (0x2)       /* Extended sample state */

/* A/D Data Register (ADDR) */

#define RA8P_ADC_ADDR_OFFSET(n)                (0x020 + ((n) * 2))
#define RA8P_ADC_ADDR_MASK                     (0xFFFF)    /* Bits 0-15: A/D Data */
#define RA8P_ADC_ADDR_OVF                      (1 << 16)   /* Bit 16: Overflow */
#define RA8P_ADC_ADDR_ADST                     (1 << 31)   /* Bit 31: A/D Status */

/* A/D Conversion Time Setting Register (ADTSET) */

#define RA8P_ADC_ADTSET_OFFSET                 (0x040)
#define RA8P_ADC_ADTSET_ADSTS_MASK             (0x7)       /* Bits 0-2: A/D Sampling Time Select */
#define RA8P_ADC_ADTSET_ADSTS_SHIFT            (0)
#define RA8P_ADC_ADTSET_ADSTS_4                (0x0)       /* 4 states */
#define RA8P_ADC_ADTSET_ADSTS_8                (0x1)       /* 8 states */
#define RA8P_ADC_ADTSET_ADSTS_16               (0x2)       /* 16 states */
#define RA8P_ADC_ADTSET_ADSTS_32               (0x3)       /* 32 states */
#define RA8P_ADC_ADTSET_ADSTS_64               (0x4)       /* 64 states */
#define RA8P_ADC_ADTSET_ADSTS_128              (0x5)       /* 128 states */
#define RA8P_ADC_ADTSET_ADSTS_256              (0x6)       /* 256 states */

/* A/D Compare Control Register (ADCMPCR) */

#define RA8P_ADC_ADCMPCR_OFFSET                (0x044)
#define RA8P_ADC_ADCMPCR_CMPAE                 (1 << 0)    /* Bit 0: Compare A Enable */
#define RA8P_ADC_ADCMPCR_CMPBE                 (1 << 1)    /* Bit 1: Compare B Enable */
#define RA8P_ADC_ADCMPCR_CMPIE                 (1 << 2)    /* Bit 2: Compare Interrupt Enable */
#define RA8P_ADC_ADCMPCR_CMPCOND_MASK          (0x30)      /* Bits 4-5: Compare Condition */
#define RA8P_ADC_ADCMPCR_CMPCOND_SHIFT         (4)
#define RA8P_ADC_ADCMPCR_CMPCOND_BELOW         (0x0 << 4)  /* Below threshold */
#define RA8P_ADC_ADCMPCR_CMPCOND_ABOVE         (0x1 << 4)  /* Above threshold */
#define RA8P_ADC_ADCMPCR_CMPCOND_MATCH         (0x2 << 4)  /* Match threshold */

/* A/D Compare Value Register A (ADCMPDR0) */

#define RA8P_ADC_ADCMPDR0_OFFSET               (0x048)
#define RA8P_ADC_ADCMPDR0_CMP_MASK             (0xFFFF)    /* Bits 0-15: Compare Value A */

/* A/D Compare Value Register B (ADCMPDR1) */

#define RA8P_ADC_ADCMPDR1_OFFSET               (0x04A)
#define RA8P_ADC_ADCMPDR1_CMP_MASK             (0xFFFF)    /* Bits 0-15: Compare Value B */

/* A/D Sample State Register (ADSSTRn) */

#define RA8P_ADC_ADSSTR0_OFFSET                (0x060)
#define RA8P_ADC_ADSSTR1_OFFSET                (0x061)
/* ... up to ADSSTR31 */

/* A/D Control Register 2 (ADCR2) */

#define RA8P_ADC_ADCR2_OFFSET                  (0x080)
#define RA8P_ADC_ADCR2_ADSTP                   (1 << 0)    /* Bit 0: A/D Stop */
#define RA8P_ADC_ADCR2_ADSTPC                  (1 << 1)    /* Bit 1: A/D Stop Control */
#define RA8P_ADC_ADCR2_ADBIG                   (1 << 2)    /* Bit 2: A/D Big Endian */
#define RA8P_ADC_ADCR2_ADCSMP                  (1 << 3)    /* Bit 3: A/D Sample Mode */
#define RA8P_ADC_ADCR2_ADREFE                  (1 << 4)    /* Bit 4: A/D Reference Enable */

/* A/D Extended Control Register (ADEXICR) */

#define RA8P_ADC_ADEXICR_OFFSET                (0x084)
#define RA8P_ADC_ADEXICR_OVSS_MASK             (0x7)       /* Bits 0-2: Oversampling Select */
#define RA8P_ADC_ADEXICR_OVSS_SHIFT            (0)
#define RA8P_ADC_ADEXICR_OVIE                  (1 << 3)    /* Bit 3: Overflow Interrupt Enable */
#define RA8P_ADC_ADEXICR_ADHWE                 (1 << 4)    /* Bit 4: A/D Hardware Trigger Enable */
#define RA8P_ADC_ADEXICR_ADHWSEL_MASK          (0x70)      /* Bits 4-6: A/D Hardware Trigger Select */
#define RA8P_ADC_ADEXICR_ADHWSEL_SHIFT         (4)

/* A/D Interrupt Enable Register (ADIER) */

#define RA8P_ADC_ADIER_OFFSET                  (0x088)
#define RA8P_ADC_ADIER_ADIE                    (1 << 0)    /* Bit 0: A/D Interrupt Enable */
#define RA8P_ADC_ADIER_CMPIE                   (1 << 1)    /* Bit 1: Compare Interrupt Enable */
#define RA8P_ADC_ADIER_GADIE                   (1 << 2)    /* Bit 2: Group A/D Interrupt Enable */

/* A/D Interrupt Status Register (ADISR) */

#define RA8P_ADC_ADISR_OFFSET                  (0x08C)
#define RA8P_ADC_ADISR_ADIF                    (1 << 0)    /* Bit 0: A/D Interrupt Flag */
#define RA8P_ADC_ADISR_CMPIF                   (1 << 1)    /* Bit 1: Compare Interrupt Flag */
#define RA8P_ADC_ADISR_GADIF                   (1 << 2)    /* Bit 2: Group A/D Interrupt Flag */

/* A/D Calibration Control Register (ADCALCR) */

#define RA8P_ADC_ADCALCR_OFFSET                (0x090)
#define RA8P_ADC_ADCALCR_ADCAL                 (1 << 0)    /* Bit 0: A/D Calibration */
#define RA8P_ADC_ADCALCR_ADCALST               (1 << 1)    /* Bit 1: A/D Calibration Status */

/* A/D Self-Diagnosis Control Register (ADDICR) */

#define RA8P_ADC_ADDICR_OFFSET                 (0x094)
#define RA8P_ADC_ADDICR_ADDIC                  (1 << 0)    /* Bit 0: A/D Self-Diagnosis */
#define RA8P_ADC_ADDICR_ADDICST                (1 << 1)    /* Bit 1: A/D Self-Diagnosis Status */

/* A/D Power Control Register (ADPWR) */

#define RA8P_ADC_ADPWR_OFFSET                  (0x098)
#define RA8P_ADC_ADPWR_ADPC                    (1 << 0)    /* Bit 0: A/D Power Control */
#define RA8P_ADC_ADPWR_ADPCST                  (1 << 1)    /* Bit 1: A/D Power Control Status */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* ADC channel configuration */

struct ra8p_adc_channel_s
{
  uint8_t channel;                  /* Channel number (0-31) */
  uint8_t reference;                /* Reference voltage source */
  uint8_t sampling_time;            /* Sampling time in ADC clocks */
  uint8_t resolution;               /* Resolution (12 or 16 bits) */
};

/* ADC device configuration */

struct ra8p_adc_config_s
{
  uint32_t base;                    /* ADC base address */
  uint8_t irq;                      /* ADC interrupt number */
  uint8_t channels;                 /* Number of available channels */
  uint8_t resolution;               /* Default resolution */
  uint32_t frequency;               /* ADC clock frequency */
};

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_ADC_H */