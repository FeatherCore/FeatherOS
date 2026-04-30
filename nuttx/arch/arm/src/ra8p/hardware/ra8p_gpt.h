/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_gpt.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_GPT_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_GPT_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* GPT Register Offsets - Based on Zephyr RA8P1 device tree */
#define RA8P_GPT_GTPR_OFFSET         0x000   /* Period Register */
#define RA8P_GPT_GTPSR_OFFSET        0x004   /* Period Save Register */
#define RA8P_GPT_GTCR_OFFSET         0x008   /* Count Control Register */
#define RA8P_GPT_GTSSR_OFFSET        0x00C   /* Start Source Register */
#define RA8P_GPT_GTPSR_OFFSET        0x010   /* Pulse Save Register */
#define RA8P_GPT_GTUPSR_OFFSET       0x014   /* Up Count Source Register */
#define RA8P_GPT_GTDNSR_OFFSET       0x018   /* Down Count Source Register */
#define RA8P_GPT_GTITCOR_OFFSET      0x020   /* Initial Transition Count Register */
#define RA8P_GPT_GTITC_OFFSET        0x024   /* Transition Count Register */
#define RA8P_GPT_GTDBLR_OFFSET       0x028   /* Double Buffer Register */
#define RA8P_GPT_GTICASR_OFFSET      0x030   /* Input Capture A Source Register */
#define RA8P_GPT_GTICBSR_OFFSET      0x034   /* Input Capture B Source Register */
#define RA8P_GPT_GTCCRA_OFFSET       0x038   /* Capture/Compare A Register */
#define RA8P_GPT_GTCCRB_OFFSET       0x03C   /* Capture/Compare B Register */
#define RA8P_GPT_GTCCRC_OFFSET       0x040   /* Capture/Compare C Register */
#define RA8P_GPT_GTCCRD_OFFSET       0x044   /* Capture/Compare D Register */
#define RA8P_GPT_GTCCRE_OFFSET       0x048   /* Capture/Compare E Register */
#define RA8P_GPT_GTCCRF_OFFSET       0x04C   /* Capture/Compare F Register */
#define RA8P_GPT_GTCUAR_OFFSET       0x050   /* Compare Match A Unmatched Register */
#define RA8P_GPT_GTCUBR_OFFSET       0x054   /* Compare Match B Unmatched Register */
#define RA8P_GPT_GTCUCR_OFFSET       0x058   /* Compare Match C Unmatched Register */
#define RA8P_GPT_GTCUDR_OFFSET       0x05C   /* Compare Match D Unmatched Register */
#define RA8P_GPT_GTCUER_OFFSET       0x060   /* Compare Match E Unmatched Register */
#define RA8P_GPT_GTCUFR_OFFSET       0x064   /* Compare Match F Unmatched Register */
#define RA8P_GPT_GTDVU_OFFSET        0x068   /* Division Value Upper Register */
#define RA8P_GPT_GTDVD_OFFSET        0x06C   /* Division Value Lower Register */
#define RA8P_GPT_GTPBR_OFFSET(n)     (0x080 + ((n) * 4))  /* Buffer Register A-D */
#define RA8P_GPT_GTSSTR_OFFSET       0x0A0   /* Software Start Register */
#define RA8P_GPT_GTSCLR_OFFSET       0x0A4   /* Software Clear Register */
#define RA8P_GPT_GTSTR_OFFSET        0x0A8   /* Status Register */
#define RA8P_GPT_GTSTP_OFFSET        0x0AC   /* Stop Register */
#define RA8P_GPT_GTCLR_OFFSET        0x0B0   /* Clear Register */
#define RA8P_GPT_GTICCR_OFFSET       0x0B4   /* Input Capture Control Register */
#define RA8P_GPT_GTICASR_OFFSET      0x0B8   /* Input Capture A Status Register */
#define RA8P_GPT_GTICBSR_OFFSET      0x0BC   /* Input Capture B Status Register */
#define RA8P_GPT_GTDADSR_OFFSET      0x0C0   /* Duty Cycle A/D Selection Register */
#define RA8P_GPT_GTDBADSR_OFFSET     0x0C4   /* Double Buffer A/D Selection Register */
#define RA8P_GPT_GTEVFSR_OFFSET      0x0C8   /* External Event Filter Select Register */
#define RA8P_GPT_GTEVCR_OFFSET       0x0CC   /* External Event Control Register */
#define RA8P_GPT_GTADCSR_OFFSET      0x0D0   /* A/D Converter Start Select Register */
#define RA8P_GPT_GTADTGR_OFFSET      0x0D4   /* A/D Converter Start Timing Register */
#define RA8P_GPT_GTADTGPR_OFFSET     0x0D8   /* A/D Converter Start Timing Buffer Register */
#define RA8P_GPT_GTDTCR_OFFSET       0x0DC   /* Dead Time Control Register */
#define RA8P_GPT_GTDTRA_OFFSET       0x0E0   /* Dead Time Rising A Register */
#define RA8P_GPT_GTDTRB_OFFSET       0x0E4   /* Dead Time Rising B Register */
#define RA8P_GPT_GTDTRC_OFFSET       0x0E8   /* Dead Time Rising C Register */
#define RA8P_GPT_GTDTRD_OFFSET       0x0EC   /* Dead Time Rising D Register */
#define RA8P_GPT_GTDTFA_OFFSET       0x0F0   /* Dead Time Falling A Register */
#define RA8P_GPT_GTDTFB_OFFSET       0x0F4   /* Dead Time Falling B Register */
#define RA8P_GPT_GTDTFC_OFFSET       0x0F8   /* Dead Time Falling C Register */
#define RA8P_GPT_GTDTFD_OFFSET       0x0FC   /* Dead Time Falling D Register */
#define RA8P_GPT_GTWFR_OFFSET        0x100   /* Waveform Function Select Register */
#define RA8P_GPT_GTOSR_OFFSET        0x104   /* Output Select Register */
#define RA8P_GPT_GTUPSR_OFFSET       0x108   /* Up Count Source Register */
#define RA8P_GPT_GTDNSR_OFFSET       0x10C   /* Down Count Source Register */
#define RA8P_GPT_GTCNT_OFFSET        0x110   /* Counter Register */
#define RA8P_GPT_GTUDDTYC_OFFSET     0x114   /* Up/Down Duty Cycle Register */
#define RA8P_GPT_GTPINTSR_OFFSET     0x118   /* Period Interrupt Source Register */
#define RA8P_GPT_GTCINTSR_OFFSET     0x11C   /* Compare Match Interrupt Source Register */
#define RA8P_GPT_GTINTAD_OFFSET      0x120   /* Interrupt A/D Register */

/* GTCR - Count Control Register */
#define RA8P_GPT_GTCR_MD_MASK        (0x0F << 0)   /* Operating Mode Mask */
#define RA8P_GPT_GTCR_MD_SHIFT       0
#define RA8P_GPT_GTCR_MD_STOP        (0 << 0)      /* Stop */
#define RA8P_GPT_GTCR_MD_CONT        (1 << 0)      /* Continuous count */
#define RA8P_GPT_GTCR_MD_ONESHOT     (2 << 0)      /* One-shot count */
#define RA8P_GPT_GTCR_MD_TRIANGLE    (4 << 0)      /* Triangle wave */
#define RA8P_GPT_GTCR_MD_SAWTOOTH    (8 << 0)      /* Sawtooth wave */

#define RA8P_GPT_GTCR_CST            (1 << 4)      /* Count Start */
#define RA8P_GPT_GTCR_BFE            (1 << 5)      /* Buffer Enable */
#define RA8P_GPT_GTCR_OADGE          (1 << 6)      /* Output A/D Group Enable */
#define RA8P_GPT_GTCR_OAE            (1 << 7)      /* Output A Enable */
#define RA8P_GPT_GTCR_OBE            (1 << 8)      /* Output B Enable */
#define RA8P_GPT_GTCR_OCE            (1 << 9)      /* Output C Enable */
#define RA8P_GPT_GTCR_ODE            (1 << 10)     /* Output D Enable */
#define RA8P_GPT_GTCR_UDDNC          (1 << 12)     /* Up/Down Count Select */
#define RA8P_GPT_GTCR_UDF            (1 << 13)     /* Up/Down Flag */
#define RA8P_GPT_GTCR_TPCS_MASK      (0x07 << 16)  /* Timer Prescaler Mask */
#define RA8P_GPT_GTCR_TPCS_SHIFT     16
#define RA8P_GPT_GTCR_TPCS_DIV1      (0 << 16)     /* Divide by 1 */
#define RA8P_GPT_GTCR_TPCS_DIV2      (1 << 16)     /* Divide by 2 */
#define RA8P_GPT_GTCR_TPCS_DIV4      (2 << 16)     /* Divide by 4 */
#define RA8P_GPT_GTCR_TPCS_DIV8      (3 << 16)     /* Divide by 8 */
#define RA8P_GPT_GTCR_TPCS_DIV16     (4 << 16)     /* Divide by 16 */
#define RA8P_GPT_GTCR_TPCS_DIV32     (5 << 16)     /* Divide by 32 */
#define RA8P_GPT_GTCR_TPCS_DIV64     (6 << 16)     /* Divide by 64 */
#define RA8P_GPT_GTCR_TPCS_DIV128    (7 << 16)     /* Divide by 128 */

/* GTSTR - Status Register */
#define RA8P_GPT_GTSTR_SST           (1 << 0)      /* Start Status */
#define RA8P_GPT_GTSTR_CSTT          (1 << 1)      /* Count Start Trigger Status */
#define RA8P_GPT_GTSTR_CSTF          (1 << 2)      /* Count Start Factor Status */

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_gpt_init
 *
 * Description:
 *   Initialize a GPT PWM channel.
 *
 * Input Parameters:
 *   channel - GPT channel (0-13)
 *   frequency - PWM frequency in Hz
 *
 ****************************************************************************/

int ra8p_gpt_init(uint8_t channel, uint32_t frequency);

/****************************************************************************
 * Name: ra8p_gpt_set_duty
 *
 * Description:
 *   Set the duty cycle for a GPT PWM channel.
 *
 * Input Parameters:
 *   channel - GPT channel (0-13)
 *   duty - Duty cycle (0-100%)
 *
 ****************************************************************************/

int ra8p_gpt_set_duty(uint8_t channel, uint8_t duty);

/****************************************************************************
 * Name: ra8p_gpt_start
 *
 * Description:
 *   Start a GPT PWM channel.
 *
 * Input Parameters:
 *   channel - GPT channel (0-13)
 *
 ****************************************************************************/

void ra8p_gpt_start(uint8_t channel);

/****************************************************************************
 * Name: ra8p_gpt_stop
 *
 * Description:
 *   Stop a GPT PWM channel.
 *
 * Input Parameters:
 *   channel - GPT channel (0-13)
 *
 ****************************************************************************/

void ra8p_gpt_stop(uint8_t channel);

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_GPT_H */