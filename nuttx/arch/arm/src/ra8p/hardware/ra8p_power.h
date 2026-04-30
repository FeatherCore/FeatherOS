/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_power.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_POWER_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_POWER_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Power Control Base Address */

#define RA8P_SYSC_BASE           0x4001E000

/* System Control Registers */

#define RA8P_SYSC_PWRMGMTCR      0x0040    /* Power Management Control Register */
#define RA8P_SYSC_PWRMGMTSTAT    0x0044    /* Power Management Status Register */
#define RA8P_SYSC_SBYCR          0x0048    /* Standby Control Register */
#define RA8P_SYSC_SBYSR          0x004C    /* Standby Status Register */
#define RA8P_SYSC_DPSBYCR        0x0050    /* Deep Standby Control Register */
#define RA8P_SYSC_DPSBYSR        0x0054    /* Deep Standby Status Register */
#define RA8P_SYSC_DPSIER0        0x0060    /* Deep Standby Interrupt Enable Register 0 */
#define RA8P_SYSC_DPSIER1        0x0064    /* Deep Standby Interrupt Enable Register 1 */
#define RA8P_SYSC_DPSIER2        0x0068    /* Deep Standby Interrupt Enable Register 2 */
#define RA8P_SYSC_DPSIER3        0x006C    /* Deep Standby Interrupt Enable Register 3 */
#define RA8P_SYSC_DPSIER4        0x0070    /* Deep Standby Interrupt Enable Register 4 */
#define RA8P_SYSC_DPSIER5        0x0074    /* Deep Standby Interrupt Enable Register 5 */
#define RA8P_SYSC_DPSIEGR0       0x0080    /* Deep Standby Interrupt Edge Setting Register 0 */
#define RA8P_SYSC_DPSIEGR1       0x0084    /* Deep Standby Interrupt Edge Setting Register 1 */
#define RA8P_SYSC_DPSIEGR2       0x0088    /* Deep Standby Interrupt Edge Setting Register 2 */
#define RA8P_SYSC_DPSIEGR3       0x008C    /* Deep Standby Interrupt Edge Setting Register 3 */
#define RA8P_SYSC_DPSIEGR4       0x0090    /* Deep Standby Interrupt Edge Setting Register 4 */

/* PWRMGMTCR - Power Management Control Register */

#define SYSC_PWRMGMTCR_SBYWC_MASK   (0x0F << 0)   /* Standby Wake Control Mask */
#define SYSC_PWRMGMTCR_SBYWC_SHIFT  0
#define SYSC_PWRMGMTCR_SBYPWC       (1 << 4)      /* Standby Power Supply Control */
#define SYSC_PWRMGMTCR_SBYSW        (1 << 5)      /* Standby Software Standby */
#define SYSC_PWRMGMTCR_DPSBYE       (1 << 6)      /* Deep Standby Mode Enable */
#define SYSC_PWRMGMTCR_DSIDE        (1 << 7)      /* Deep Standby Independent Mode Enable */
#define SYSC_PWRMGMTCR_VBATT        (1 << 8)      /* VBATT Domain Control */

/* PWRMGMTSTAT - Power Management Status Register */

#define SYSC_PWRMGMTSTAT_SBYSTS     (1 << 0)      /* Standby Status */
#define SYSC_PWRMGMTSTAT_DPSBYST    (1 << 1)      /* Deep Standby Status */

/* SBYCR - Standby Control Register */

#define SYSC_SBYCR_SSBY             (1 << 0)      /* Software Standby */
#define SYSC_SBYCR_OPE              (1 << 1)      /* On-chip RAM Pseudo Standby */
#define SYSC_SBYCR_RSCK             (1 << 2)      /* RAM Self Refresh Clock Control */
#define SYSC_SBYCR_LOCOCORE         (1 << 3)      /* LOCO Oscillator Control in Standby */
#define SYSC_SBYCR_MOSCSTOP         (1 << 4)      /* Main Oscillator Stop */
#define SYSC_SBYCR_PLLSTOP          (1 << 5)      /* PLL Stop */
#define SYSC_SBYCR_SOSCCR           (1 << 6)      /* Sub OSC Control */
#define SYSC_SBYCR_MOSCCR           (1 << 7)      /* Main OSC Control */

/* SBYSR - Standby Status Register */

#define SYSC_SBYSR_SSBYST           (1 << 0)      /* Software Standby Status */

/* DPSBYCR - Deep Standby Control Register */

#define SYSC_DPSBYCR_DPSBYPWCR      (1 << 0)      /* Deep Standby Power Control */
#define SYSC_DPSBYCR_DPSBY2F        (1 << 1)      /* Deep Standby Mode 2 */
#define SYSC_DPSBYCR_DPSBYPD        (1 << 2)      /* Deep Standby Port Disable */
#define SYSC_DPSBYCR_DPSBYRVM       (1 << 3)      /* Deep Standby RAM Voltage Monitor */
#define SYSC_DPSBYCR_DPSBYRVS       (1 << 4)      /* Deep Standby RAM Voltage Supply */
#define SYSC_DPSBYCR_DPSBYVBERDY    (1 << 5)      /* Deep Standby VBATT Ready */
#define SYSC_DPSBYCR_DPSBY2FEN      (1 << 6)      /* Deep Standby Mode 2 Enable */

/* Power modes */

#define RA8P_POWER_MODE_RUN        0
#define RA8P_POWER_MODE_SLEEP      1
#define RA8P_POWER_MODE_DEEPSLEEP  2
#define RA8P_POWER_MODE_STANDBY    3
#define RA8P_POWER_MODE_DEEPSTBY   4

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* Power management configuration structure */

struct ra8p_power_cfg_s
{
  uint32_t power_mode;              /* Target power mode */
  uint32_t wake_sources;            /* Wake-up sources */
  uint32_t ram_retention_mask;      /* RAM retention mask */
  uint32_t io_port_state;           /* IO port state */
  uint32_t power_supply_state;      /* Power supply state */
  uint32_t deep_standby_cancel_src; /* Deep standby cancel sources */
  bool tcm_retention;               /* TCM retention */
  bool pll_ldo_enable;              /* PLL LDO enable */
  bool hoco_ldo_enable;             /* HOCO LDO enable */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_power_initialize
 *
 * Description:
 *   Initialize the power management system
 *
 ****************************************************************************/

int ra8p_power_initialize(void);

/****************************************************************************
 * Name: ra8p_power_enter_mode
 *
 * Description:
 *   Enter a specific power mode
 *
 ****************************************************************************/

int ra8p_power_enter_mode(struct ra8p_power_cfg_s *cfg);

/****************************************************************************
 * Name: ra8p_power_sleep
 *
 * Description:
 *   Enter sleep mode
 *
 ****************************************************************************/

void ra8p_power_sleep(void);

/****************************************************************************
 * Name: ra8p_power_deepsleep
 *
 * Description:
 *   Enter deep sleep mode
 *
 ****************************************************************************/

void ra8p_power_deepsleep(void);

/****************************************************************************
 * Name: ra8p_power_standby
 *
 * Description:
 *   Enter standby mode
 *
 ****************************************************************************/

void ra8p_power_standby(void);

/****************************************************************************
 * Name: ra8p_power_deepstandby
 *
 * Description:
 *   Enter deep standby mode
 *
 ****************************************************************************/

void ra8p_power_deepstandby(void);

/****************************************************************************
 * Name: ra8p_power_get_status
 *
 * Description:
 *   Get current power status
 *
 ****************************************************************************/

uint32_t ra8p_power_get_status(void);

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_POWER_H */