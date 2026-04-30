/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_dac.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_DAC_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_DAC_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* DAC Base Addresses */

#define RA8P_DAC0_BASE                         (0x40294000) /* DAC0 base address */
#define RA8P_DAC1_BASE                         (0x40294200) /* DAC1 base address */

/* DAC Register Offsets */

/* DAC Data Register A (DADR) */

#define RA8P_DAC_DADR_OFFSET                   (0x000)
#define RA8P_DAC_DADR_DA_MASK                  (0xFFF)     /* Bits 0-11: DAC Data */
#define RA8P_DAC_DADR_DA_SHIFT                 (0)

/* DAC Data Register B (DADRB) */

#define RA8P_DAC_DADRB_OFFSET                  (0x002)
#define RA8P_DAC_DADRB_DB_MASK                 (0xFFF)     /* Bits 0-11: DAC Data */
#define RA8P_DAC_DADRB_DB_SHIFT                (0)

/* DAC Control Register (DACR) */

#define RA8P_DAC_DACR_OFFSET                   (0x004)
#define RA8P_DAC_DACR_DAOE                     (1 << 0)    /* Bit 0: DAC Output Enable */
#define RA8P_DAC_DACR_DAOE1                    (1 << 1)    /* Bit 1: DAC Output Enable 1 */
#define RA8P_DAC_DACR_DACMOD                   (1 << 2)    /* Bit 2: DAC Mode */
#define RA8P_DAC_DACR_DACMOD_NORMAL            (0 << 2)    /* Normal mode */
#define RA8P_DAC_DACR_DACMOD_SINE              (1 << 2)    /* Sine wave mode */
#define RA8P_DAC_DACR_CHGR                     (1 << 3)    /* Bit 3: Channel Gain */
#define RA8P_DAC_DACR_CHGR_NORMAL              (0 << 3)    /* Normal gain */
#define RA8P_DAC_DACR_CHGR_HIGH                (1 << 3)    /* High gain */

/* DAC Comparison Reference Register (DADRVR) */

#define RA8P_DAC_DADRVR_OFFSET                 (0x006)
#define RA8P_DAC_DADRVR_REFC_MASK              (0x7)       /* Bits 0-2: Reference Voltage */
#define RA8P_DAC_DADRVR_REFC_SHIFT             (0)
#define RA8P_DAC_DADRVR_REFC_INT_VREF          (0)        /* Internal reference voltage */
#define RA8P_DAC_DADRVR_REFC_AVCC              (1)        /* AVCC */
#define RA8P_DAC_DADRVR_REFC_EXT_VREF          (2)        /* External reference voltage */

/* DAC Interrupt Enable Register (DAIINT) */

#define RA8P_DAC_DAIINT_OFFSET                 (0x008)
#define RA8P_DAC_DAIINT_DACIE                  (1 << 0)    /* Bit 0: DAC Interrupt Enable */
#define RA8P_DAC_DAIINT_DACMIE                 (1 << 1)    /* Bit 1: DAC Mode Interrupt Enable */

/* DAC Status Register (DASTR) */

#define RA8P_DAC_DASTR_OFFSET                  (0x00A)
#define RA8P_DAC_DASTR_DACIF                   (1 << 0)    /* Bit 0: DAC Interrupt Flag */
#define RA8P_DAC_DASTR_DACMIF                  (1 << 1)    /* Bit 1: DAC Mode Interrupt Flag */

/* DAC Amplifier Control Register (DACAAMP) */

#define RA8P_DAC_DACAAMP_OFFSET                (0x00C)
#define RA8P_DAC_DACAAMP_AOF                   (1 << 0)    /* Bit 0: Amplifier Offset */
#define RA8P_DAC_DACAAMP_AGAIN_MASK            (0x70)      /* Bits 4-6: Amplifier Gain */
#define RA8P_DAC_DACAAMP_AGAIN_SHIFT           (4)

/* DAC Power Control Register (DACPCON) */

#define RA8P_DAC_DACPCON_OFFSET                (0x00E)
#define RA8P_DAC_DACPCON_DACEN                 (1 << 0)    /* Bit 0: DAC Power Enable */
#define RA8P_DAC_DACPCON_DACENST               (1 << 1)    /* Bit 1: DAC Power Status */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* DAC channel configuration */

struct ra8p_dac_channel_s
{
  uint8_t channel;                  /* Channel number (0 or 1) */
  uint16_t resolution;              /* Resolution (12 bits for RA8P) */
  uint8_t reference;                /* Reference voltage source */
  uint8_t gain;                     /* Amplifier gain setting */
  bool output_enabled;              /* True if output is enabled */
};

/* DAC device configuration */

struct ra8p_dac_config_s
{
  uint32_t base;                    /* DAC base address */
  uint8_t channels;                 /* Number of available channels */
  uint16_t resolution;              /* Resolution in bits */
  uint32_t frequency;               /* Output frequency (for waveform modes) */
};

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_DAC_H */