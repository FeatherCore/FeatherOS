/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_usbphy.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_USBPHY_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_USBPHY_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* USBPHY Register Offsets */
#define RA8P_USBPHY_PHYCONF_OFFSET        0x0000  /* PHY Configuration Register */
#define RA8P_USBPHY_PHYTIM0_OFFSET        0x0004  /* PHY Timing Register 0 */
#define RA8P_USBPHY_PHYTIM1_OFFSET        0x0008  /* PHY Timing Register 1 */
#define RA8P_USBPHY_PHYPADCTL_OFFSET      0x000C  /* PHY Pad Control Register */
#define RA8P_USBPHY_PLLCTRL_OFFSET        0x0010  /* PLL Control Register */
#define RA8P_USBPHY_PHYSTAT_OFFSET        0x0014  /* PHY Status Register */
#define RA8P_USBPHY_PLLSTAT_OFFSET        0x0018  /* PLL Status Register */
#define RA8P_USBPHY_PHYADDITION_OFFSET    0x001C  /* PHY Additional Register */
#define RA8P_USBPHY_PHYPLLCTRL_OFFSET     0x0020  /* PHY PLL Control Register */
#define RA8P_USBPHY_PHYERR_OFFSET         0x0024  /* PHY Error Register */
#define RA8P_USBPHY_PHYERRCLR_OFFSET      0x0028  /* PHY Error Clear Register */
#define RA8P_USBPHY_PHYCNT_OFFSET         0x002C  /* PHY Counter Register */
#define RA8P_USBPHY_PHYCNTCLR_OFFSET      0x0030  /* PHY Counter Clear Register */
#define RA8P_USBPHY_PHYCNT2_OFFSET        0x0034  /* PHY Counter Register 2 */
#define RA8P_USBPHY_PHYCNTCLR2_OFFSET     0x0038  /* PHY Counter Clear Register 2 */
#define RA8P_USBPHY_PHYCNT3_OFFSET        0x003C  /* PHY Counter Register 3 */
#define RA8P_USBPHY_PHYCNTCLR3_OFFSET     0x0040  /* PHY Counter Clear Register 3 */

/* PHYCONF - PHY Configuration Register */
#define RA8P_USBPHY_PHYCONF_DIRPD         (1 << 0)   /* DP/DM Pulldown Disable */
#define RA8P_USBPHY_PHYCONF_PLLRESET      (1 << 1)   /* PLL Reset */
#define RA8P_USBPHY_PHYCONF_XCVRSELDPS    (1 << 4)   /* XCVR Select D+ Pull-up */
#define RA8P_USBPHY_PHYCONF_XCVRSELDMS    (1 << 5)   /* XCVR Select D- Pull-up */
#define RA8P_USBPHY_PHYCONF_RESMI         (1 << 6)   /* Resume Interrupt */
#define RA8P_USBPHY_PHYCONF_USBPWDN       (1 << 7)   /* USB Power Down */
#define RA8P_USBPHY_PHYCONF_PLLWAIT       (1 << 8)   /* PLL Wait */

/* PHYTIM0 - PHY Timing Register 0 */
#define RA8P_USBPHY_PHYTIM0_USBDMPD       (1 << 0)   /* USB D- Pulldown */
#define RA8P_USBPHY_PHYTIM0_USBDPPD       (1 << 1)   /* USB D+ Pulldown */
#define RA8P_USBPHY_PHYTIM0_USBSIGMD      (1 << 2)   /* USB Signal Mode */
#define RA8P_USBPHY_PHYTIM0_USBSUSPEND    (1 << 3)   /* USB Suspend */
#define RA8P_USBPHY_PHYTIM0_USBSLPMD      (1 << 4)   /* USB Sleep Mode */
#define RA8P_USBPHY_PHYTIM0_USBRSMD       (1 << 5)   /* USB Reset Mode */

/* PHYPADCTL - PHY Pad Control Register */
#define RA8P_USBPHY_PHYPADCTL_HSEDM       (1 << 0)   /* Hi-Speed Enable DM */
#define RA8P_USBPHY_PHYPADCTL_HSEDPP      (1 << 1)   /* Hi-Speed Enable D+ */
#define RA8P_USBPHY_PHYPADCTL_DMRPD       (1 << 2)   /* D- Resistor Pull-down */
#define RA8P_USBPHY_PHYPADCTL_DPRPU       (1 << 3)   /* D+ Resistor Pull-up */
#define RA8P_USBPHY_PHYPADCTL_DVS         (1 << 4)   /* Data Validation Sequence */
#define RA8P_USBPHY_PHYPADCTL_DMS         (1 << 5)   /* D- Selection */
#define RA8P_USBPHY_PHYPADCTL_DPS         (1 << 6)   /* D+ Selection */
#define RA8P_USBPHY_PHYPADCTL_PCIEN       (1 << 7)   /* PC Input Enable */

/* PLLCTRL - PLL Control Register */
#define RA8P_USBPHY_PLLCTRL_PLL_EN        (1 << 0)   /* PLL Enable */
#define RA8P_USBPHY_PLLCTRL_PLL_RESET     (1 << 1)   /* PLL Reset */
#define RA8P_USBPHY_PLLCTRL_PLL_CLKSEL    (1 << 2)   /* PLL Clock Select */
#define RA8P_USBPHY_PLLCTRL_PLL_STABLE    (1 << 3)   /* PLL Stable */

/* PLLSTAT - PLL Status Register */
#define RA8P_USBPHY_PLLSTAT_PLL_LOCK      (1 << 0)   /* PLL Lock */

/* PHYSTAT - PHY Status Register */
#define RA8P_USBPHY_PHYSTAT_PLL_LOCK_STS  (1 << 0)   /* PLL Lock Status */
#define RA8P_USBPHY_PHYSTAT_VBUS_STS      (1 << 1)   /* VBUS Status */
#define RA8P_USBPHY_PHYSTAT_SUSPEND_STS   (1 << 2)   /* Suspend Status */
#define RA8P_USBPHY_PHYSTAT_OVRCUR_STS    (1 << 3)   /* Overcurrent Status */

/* PHYERR - PHY Error Register */
#define RA8P_USBPHY_PHYERR_PLL_LOCK_ERR   (1 << 0)   /* PLL Lock Error */
#define RA8P_USBPHY_PHYERR_VBUS_ERR       (1 << 1)   /* VBUS Error */
#define RA8P_USBPHY_PHYERR_SUSPEND_ERR    (1 << 2)   /* Suspend Error */
#define RA8P_USBPHY_PHYERR_OVERCUR_ERR    (1 << 3)   /* Overcurrent Error */

/* PHYCNTCLR - PHY Counter Clear Register */
#define RA8P_USBPHY_PHYCNTCLR_PHYCNTCL    (1 << 0)   /* PHY Counter Clear */

/* USBPHY Base Address */
#define RA8P_USBPHY_BASE                  0x40254000
#define RA8P_USBPHY_SIZE                  0x100

/* USBPHY interrupt number */
#define RA8P_IRQ_USBPHY                   96

/* USBPHY modes */
#define RA8P_USBPHY_MODE_FS               0x00      /* Full Speed Mode */
#define RA8P_USBPHY_MODE_HS               0x01      /* High Speed Mode */
#define RA8P_USBPHY_MODE_HSIC             0x02      /* High Speed IC Mode */

/* USBPHY clock sources */
#define RA8P_USBPHY_CLK_SRC_XTAL          0x00      /* Crystal clock source */
#define RA8P_USBPHY_CLK_SRC_EXT           0x01      /* External clock source */
#define RA8P_USBPHY_CLK_SRC_PLL           0x02      /* PLL clock source */

/* USBPHY timeout in milliseconds */
#define RA8P_USBPHY_TIMEOUT_MS            100

/* USBPHY refresh sequences */
#define RA8P_USBPHY_REFRESH_SEQ1          0xAC
#define RA8P_USBPHY_REFRESH_SEQ2          0x53

/* USBPHY unlock code */
#define RA8P_USBPHY_UNLOCK_CODE           0xA5

/* USBPHY reset sequence */
#define RA8P_USBPHY_RESET_SEQ             0x01

/* USBPHY overcurrent protection thresholds */
#define RA8P_USBPHY_OC_THRESHOLD_4_5MA    0x00
#define RA8P_USBPHY_OC_THRESHOLD_9_5MA    0x01
#define RA8P_USBPHY_OC_THRESHOLD_18_5MA   0x02
#define RA8P_USBPHY_OC_THRESHOLD_36_5MA   0x03

/* USBPHY drive strength */
#define RA8P_USBPHY_DRIVE_STRENGTH_LOW    0x00      /* Low drive strength */
#define RA8P_USBPHY_DRIVE_STRENGTH_HIGH   0x01      /* High drive strength */

/* USBPHY differential output current */
#define RA8P_USBPHY_DIFOUT_CURRENT_850UA  0x00      /* 850uA (Default) */
#define RA8P_USBPHY_DIFOUT_CURRENT_1700UA 0x01      /* 1700uA */
#define RA8P_USBPHY_DIFOUT_CURRENT_2550UA 0x02      /* 2550uA */
#define RA8P_USBPHY_DIFOUT_CURRENT_3400UA 0x03      /* 3400uA */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* USBPHY configuration structure */
struct ra8p_usbphy_config_s
{
  uint8_t mode;                       /* PHY mode (FS/HS) */
  uint8_t clk_source;                /* Clock source */
  bool pullup_dp_enable;              /* D+ pull-up enable */
  bool pulldown_dm_enable;            /* D- pull-down enable */
  bool pll_enable;                    /* PLL enable */
  bool high_speed_mode;               /* High-speed mode */
  uint32_t frequency;                 /* Clock frequency */
  bool low_power_mode;                /* Low power mode */
  uint8_t oc_threshold;               /* Overcurrent threshold */
  uint8_t drive_strength;             /* Drive strength */
  uint8_t difout_current;             /* Differential output current */
};

/* USBPHY status structure */
struct ra8p_usbphy_status_s
{
  bool pll_locked;                    /* PLL lock status */
  bool vbus_detected;                 /* VBUS detection status */
  bool suspend_mode;                  /* Suspend mode status */
  bool overcurrent;                   /* Overcurrent status */
  uint8_t link_state;                 /* Link state */
  uint8_t speed;                      /* Current speed */
  uint8_t error_flags;                /* Error flags */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_usbphy_initialize
 *
 * Description:
 *   Initialize the USBPHY controller based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_initialize(void);

/****************************************************************************
 * Name: ra8p_usbphy_enable
 *
 * Description:
 *   Enable the USBPHY controller based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_enable(void);

/****************************************************************************
 * Name: ra8p_usbphy_disable
 *
 * Description:
 *   Disable the USBPHY controller based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_disable(void);

/****************************************************************************
 * Name: ra8p_usbphy_set_mode
 *
 * Description:
 *   Set USBPHY mode (FS/HS) based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   mode - PHY mode (0=FS, 1=HS)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_mode(uint8_t mode);

/****************************************************************************
 * Name: ra8p_usbphy_set_pullup
 *
 * Description:
 *   Set USB D+ pull-up resistor based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_pullup(bool enable);

/****************************************************************************
 * Name: ra8p_usbphy_set_pulldown
 *
 * Description:
 *   Set USB D- pull-down resistor based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_pulldown(bool enable);

/****************************************************************************
 * Name: ra8p_usbphy_pll_enable
 *
 * Description:
 *   Enable USBPHY PLL based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_pll_enable(void);

/****************************************************************************
 * Name: ra8p_usbphy_pll_disable
 *
 * Description:
 *   Disable USBPHY PLL based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_pll_disable(void);

/****************************************************************************
 * Name: ra8p_usbphy_is_pll_locked
 *
 * Description:
 *   Check if USBPHY PLL is locked based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if PLL is locked, false otherwise
 *
 ****************************************************************************/

bool ra8p_usbphy_is_pll_locked(void);

/****************************************************************************
 * Name: ra8p_usbphy_power_down
 *
 * Description:
 *   Put USBPHY in power-down mode based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   enable - true to enter power-down, false to exit
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_power_down(bool enable);

/****************************************************************************
 * Name: ra8p_usbphy_get_status
 *
 * Description:
 *   Get USBPHY status flags based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Status flags
 *
 ****************************************************************************/

uint8_t ra8p_usbphy_get_status(void);

/****************************************************************************
 * Name: ra8p_usbphy_set_frequency
 *
 * Description:
 *   Set USBPHY frequency based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   frequency - Desired frequency in Hz
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_frequency(uint32_t frequency);

/****************************************************************************
 * Name: ra8p_usbphy_get_frequency
 *
 * Description:
 *   Get USBPHY frequency based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Current frequency in Hz
 *
 ****************************************************************************/

uint32_t ra8p_usbphy_get_frequency(void);

/****************************************************************************
 * Name: ra8p_usbphy_clear_error
 *
 * Description:
 *   Clear USBPHY error flags based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   error_flags - Error flags to clear
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_clear_error(uint32_t error_flags);

/****************************************************************************
 * Name: ra8p_usbphy_get_error_flags
 *
 * Description:
 *   Get USBPHY error flags based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Error flags
 *
 ****************************************************************************/

uint32_t ra8p_usbphy_get_error_flags(void);

/****************************************************************************
 * Name: ra8p_usbphy_is_initialized
 *
 * Description:
 *   Check if USBPHY is initialized based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if initialized, false otherwise
 *
 ****************************************************************************/

bool ra8p_usbphy_is_initialized(void);

/****************************************************************************
 * Name: ra8p_usbphy_is_enabled
 *
 * Description:
 *   Check if USBPHY is enabled based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_usbphy_is_enabled(void);

/****************************************************************************
 * Name: ra8p_usbphy_reset
 *
 * Description:
 *   Reset USBPHY controller based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_reset(void);

/****************************************************************************
 * Name: ra8p_usbphy_enable_interrupts
 *
 * Description:
 *   Enable USBPHY interrupts based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   irq - Interrupt to enable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_enable_interrupts(uint8_t irq);

/****************************************************************************
 * Name: ra8p_usbphy_disable_interrupts
 *
 * Description:
 *   Disable USBPHY interrupts based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   irq - Interrupt to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_disable_interrupts(uint8_t irq);

/****************************************************************************
 * Name: ra8p_usbphy_is_vbus_detected
 *
 * Description:
 *   Check if VBUS is detected based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if VBUS detected, false otherwise
 *
 ****************************************************************************/

bool ra8p_usbphy_is_vbus_detected(void);

/****************************************************************************
 * Name: ra8p_usbphy_is_overcurrent
 *
 * Description:
 *   Check if overcurrent condition occurred based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if overcurrent, false otherwise
 *
 ****************************************************************************/

bool ra8p_usbphy_is_overcurrent(void);

/****************************************************************************
 * Name: ra8p_usbphy_set_pullup_pulldown
 *
 * Description:
 *   Set both D+ pull-up and D- pull-down simultaneously based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   pullup - true to enable D+ pull-up
 *   pulldown - true to enable D- pull-down
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_pullup_pulldown(bool pullup, bool pulldown);

/****************************************************************************
 * Name: ra8p_usbphy_set_high_speed
 *
 * Description:
 *   Set high-speed mode for USBPHY based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   enable - true for high-speed mode, false for full-speed
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_high_speed(bool enable);

/****************************************************************************
 * Name: ra8p_usbphy_get_link_state
 *
 * Description:
 *   Get current USB link state based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Link state value
 *
 ****************************************************************************/

uint8_t ra8p_usbphy_get_link_state(void);

/****************************************************************************
 * Name: ra8p_usbphy_set_loopback
 *
 * Description:
 *   Enable/disable USBPHY loopback mode based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   enable - true to enable loopback, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_loopback(bool enable);

/****************************************************************************
 * Name: ra8p_usbphy_set_external_clock
 *
 * Description:
 *   Configure USBPHY to use external clock source based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   external - true for external clock, false for internal
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_external_clock(bool external);

/****************************************************************************
 * Name: ra8p_usbphy_wait_pll_stable
 *
 * Description:
 *   Wait for USBPHY PLL to be stable based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   timeout_ms - Timeout in milliseconds
 *
 * Returned Value:
 *   OK on success, negated errno on timeout
 *
 ****************************************************************************/

int ra8p_usbphy_wait_pll_stable(uint32_t timeout_ms);

/****************************************************************************
 * Name: ra8p_usbphy_set_squelch_level
 *
 * Description:
 *   Set USBPHY squelch level based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   level - Squelch level (0-3)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_squelch_level(uint8_t level);

/****************************************************************************
 * Name: ra8p_usbphy_set_rx_power
 *
 * Description:
 *   Set USBPHY RX power level based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   level - RX power level (0-3)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_rx_power(uint8_t level);

/****************************************************************************
 * Name: ra8p_usbphy_set_tx_power
 *
 * Description:
 *   Set USBPHY TX power level based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   level - TX power level (0-3)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_tx_power(uint8_t level);

/****************************************************************************
 * Name: ra8p_usbphy_is_suspend_mode
 *
 * Description:
 *   Check if USBPHY is in suspend mode based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if in suspend mode, false otherwise
 *
 ****************************************************************************/

bool ra8p_usbphy_is_suspend_mode(void);

/****************************************************************************
 * Name: ra8p_usbphy_set_clock_divider
 *
 * Description:
 *   Set USBPHY clock divider based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   divider - Clock divider value (0-15)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_clock_divider(uint8_t divider);

/****************************************************************************
 * Name: ra8p_usbphy_get_clock_divider
 *
 * Description:
 *   Get current USBPHY clock divider based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Clock divider value
 *
 ****************************************************************************/

uint8_t ra8p_usbphy_get_clock_divider(void);

/****************************************************************************
 * Name: ra8p_usbphy_set_overcurrent_threshold
 *
 * Description:
 *   Set USBPHY overcurrent threshold based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   threshold - OC threshold value (0-3)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_overcurrent_threshold(uint8_t threshold);

/****************************************************************************
 * Name: ra8p_usbphy_set_drive_strength
 *
 * Description:
 *   Set USBPHY drive strength based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   strength - Drive strength (0=low, 1=high)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_drive_strength(uint8_t strength);

/****************************************************************************
 * Name: ra8p_usbphy_set_differential_output_current
 *
 * Description:
 *   Set USBPHY differential output current based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   current - Differential output current value (0-3)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_differential_output_current(uint8_t current);

/****************************************************************************
 * Name: ra8p_usbphy_enable_auto_calibration
 *
 * Description:
 *   Enable/disable USBPHY automatic calibration based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   enable - true to enable auto calibration, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_enable_auto_calibration(bool enable);

/****************************************************************************
 * Name: ra8p_usbphy_set_impedance_matching
 *
 * Description:
 *   Set USBPHY impedance matching based on Nuttx USB driver implementation.
 *
 * Input Parameters:
 *   matching - Impedance matching setting
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_usbphy_set_impedance_matching(uint8_t matching);

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_USBPHY_H */