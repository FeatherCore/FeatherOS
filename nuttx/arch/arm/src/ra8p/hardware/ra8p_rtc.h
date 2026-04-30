/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_rtc.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_RTC_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_RTC_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* RTC Register Offsets - Based on RA8P Hardware Manual */
#define RA8P_RTC_RYR_OFFSET              0x0000  /* Year Register */
#define RA8P_RTC_RMONR_OFFSET            0x0002  /* Month Register */
#define RA8P_RTC_RDAYR_OFFSET            0x0004  /* Day of Month Register */
#define RA8P_RTC_RWKR_OFFSET             0x0006  /* Weekday Register */
#define RA8P_RTC_RHR_OFFSET              0x0008  /* Hour Register */
#define RA8P_RTC_RMINR_OFFSET            0x000A  /* Minute Register */
#define RA8P_RTC_RSECR_OFFSET            0x000C  /* Second Register */
#define RA8P_RTC_RYAR_OFFSET             0x0010  /* Year Alarm Register */
#define RA8P_RTC_RMONAR_OFFSET           0x0012  /* Month Alarm Register */
#define RA8P_RTC_RDAYAR_OFFSET           0x0014  /* Day of Month Alarm Register */
#define RA8P_RTC_RHRAR_OFFSET            0x0016  /* Hour Alarm Register */
#define RA8P_RTC_RMINAR_OFFSET           0x0018  /* Minute Alarm Register */
#define RA8P_RTC_RSECAR_OFFSET           0x001A  /* Second Alarm Register */
#define RA8P_RTC_RCR1_OFFSET             0x001C  /* Control Register 1 */
#define RA8P_RTC_RCR2_OFFSET             0x001E  /* Control Register 2 */
#define RA8P_RTC_RSR_OFFSET              0x0020  /* Status Register */
#define RA8P_RTC_RCCR_OFFSET             0x0022  /* Carry Control Register */
#define RA8P_RTC_RCCLR_OFFSET            0x0024  /* Carry Clear Register */
#define RA8P_RTC_RFCR_OFFSET             0x0026  /* Frequency Control Register */
#define RA8P_RTC_RVSR_OFFSET             0x0028  /* Valid Status Register */
#define RA8P_RTC_RCR3_OFFSET             0x002A  /* Control Register 3 */
#define RA8P_RTC_RCR4_OFFSET             0x002C  /* Control Register 4 */
#define RA8P_RTC_RCR5_OFFSET             0x002E  /* Control Register 5 */
#define RA8P_RTC_RCR6_OFFSET             0x0030  /* Control Register 6 */
#define RA8P_RTC_RCR7_OFFSET             0x0032  /* Control Register 7 */
#define RA8P_RTC_RCR8_OFFSET             0x0034  /* Control Register 8 */
#define RA8P_RTC_RIVR_OFFSET             0x0036  /* Interval Register */
#define RA8P_RTC_RICE_OFFSET             0x0038  /* Interval Control Register */
#define RA8P_RTC_RCRV_OFFSET             0x003A  /* Calibration Register */

/* RCR1 - Control Register 1 */
#define RA8P_RTC_RCR1_CF                 (1 << 0)   /* Carry Flag */
#define RA8P_RTC_RCR1_CIE                (1 << 1)   /* Carry Interrupt Enable */
#define RA8P_RTC_RCR1_AIE                (1 << 2)   /* Alarm Interrupt Enable */
#define RA8P_RTC_RCR1_PIE                (1 << 3)   /* Periodic Interrupt Enable */
#define RA8P_RTC_RCR1_PES_MASK           (0x0F << 4) /* Periodic Interrupt Select */
#define RA8P_RTC_RCR1_PES_SHIFT          4
#define RA8P_RTC_RCR1_PES_1SEC           (0x07 << 4) /* 1 second interval */
#define RA8P_RTC_RCR1_PES_1_2SEC         (0x06 << 4) /* 1/2 second interval */
#define RA8P_RTC_RCR1_PES_1_4SEC         (0x05 << 4) /* 1/4 second interval */
#define RA8P_RTC_RCR1_PES_1_8SEC         (0x04 << 4) /* 1/8 second interval */
#define RA8P_RTC_RCR1_PES_1_16SEC        (0x03 << 4) /* 1/16 second interval */
#define RA8P_RTC_RCR1_PES_1_32SEC        (0x02 << 4) /* 1/32 second interval */
#define RA8P_RTC_RCR1_PES_1_64SEC        (0x01 << 4) /* 1/64 second interval */
#define RA8P_RTC_RCR1_PES_1_128SEC       (0x00 << 4) /* 1/128 second interval */

/* RCR2 - Control Register 2 */
#define RA8P_RTC_RCR2_START              (1 << 0)   /* Start */
#define RA8P_RTC_RCR2_STOP               (1 << 1)   /* Stop */
#define RA8P_RTC_RCR2_RESET              (1 << 2)   /* Reset */
#define RA8P_RTC_RCR2_RTCEN              (1 << 3)   /* RTC Enable */
#define RA8P_RTC_RCR2_ADJ                (1 << 4)   /* Adjustment */
#define RA8P_RTC_RCR2_RTCE               (1 << 5)   /* RTC Clock Enable */
#define RA8P_RTC_RCR2_ADJ30              (1 << 6)   /* 30-second Adjustment */
#define RA8P_RTC_RCR2_AADJ               (1 << 7)   /* Automatic Adjustment */

/* RSR - Status Register */
#define RA8P_RTC_RSR_CSTS                (1 << 0)   /* Carry Status */
#define RA8P_RTC_RSR_ASTS                (1 << 1)   /* Alarm Status */
#define RA8P_RTC_RSR_PSTS                (1 << 2)   /* Periodic Status */
#define RA8P_RTC_RSR_ADJSTS              (1 << 3)   /* Adjustment Status */

/* RCR3 - Control Register 3 */
#define RA8P_RTC_RCR3_RCKSEL_SUBCLK      (0 << 0)   /* Sub-clock selection */
#define RA8P_RTC_RCR3_RCKSEL_LOCO        (1 << 0)   /* LOCO selection */
#define RA8P_RTC_RCR3_RCKSEL             (1 << 0)   /* Clock source selection */

/* RTC Base Address */
#define RA8P_RTC_BASE                    0x40202000
#define RA8P_RTC_SIZE                    0x0040

/* RTC interrupt numbers */
#define RA8P_IRQ_RTC                     76

/* RTC constants */
#define RA8P_RTC_YEAR_BASE               2000       /* Year reference (RTC stores 0-99) */
#define RA8P_RTC_YEAR_MAX                2099       /* Maximum supported year */
#define RA8P_RTC_YEAR_MIN                2000       /* Minimum supported year */

/* Calibration range */
#define RA8P_RTC_CALIBRATION_MIN         -63
#define RA8P_RTC_CALIBRATION_MAX         63

/* Maximum RTC timeout in milliseconds */
#define RA8P_RTC_TIMEOUT_MS              1000

/* RTC clock sources */
#define RA8P_RTC_CLK_SRC_SUBCLK          0          /* Sub-clock (32.768 kHz) */
#define RA8P_RTC_CLK_SRC_LOCO            1          /* Low-speed on-chip oscillator */

/* RTC mode definitions */
#define RA8P_RTC_MODE_NORMAL             0          /* Normal mode */
#define RA8P_RTC_MODE_CALIBRATION        1          /* Calibration mode */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* RTC time structure */
struct ra8p_rtc_time_s
{
  uint16_t year;                      /* Calendar year (2000-2099) */
  uint8_t month;                      /* Month (1-12) */
  uint8_t day;                        /* Day of month (1-31) */
  uint8_t hour;                       /* Hour (0-23) */
  uint8_t minute;                     /* Minute (0-59) */
  uint8_t second;                     /* Second (0-59) */
  uint8_t weekday;                    /* Weekday (0-6, 0=Sunday, 1=Monday, ..., 6=Saturday) */
};

/* RTC alarm structure */
struct ra8p_rtc_alarm_s
{
  uint8_t year;                       /* Year (0-99) - Use 0xFF to disable */
  uint8_t month;                      /* Month (1-12) - Use 0xFF to disable */
  uint8_t day;                        /* Day of month (1-31) - Use 0xFF to disable */
  uint8_t hour;                       /* Hour (0-23) */
  uint8_t minute;                     /* Minute (0-59) */
  uint8_t second;                     /* Second (0-59) */
  uint8_t weekday;                    /* Weekday (0-6) - Use 0xFF to disable */
  bool enabled;                       /* Alarm enabled */
  bool repeat_daily;                  /* Daily repeat flag */
};

/* RTC calibration structure */
struct ra8p_rtc_calibration_s
{
  int8_t adjustment;                  /* Adjustment value (-63 to +63) */
  bool automatic;                     /* Automatic adjustment enabled */
  uint8_t frequency;                  /* Adjustment frequency */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_rtc_initialize
 *
 * Description:
 *   Initialize the RTC based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_initialize(void);

/****************************************************************************
 * Name: ra8p_rtc_get_time
 *
 * Description:
 *   Get the current RTC time based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   time - Pointer to time structure to fill
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_get_time(struct ra8p_rtc_time_s *time);

/****************************************************************************
 * Name: ra8p_rtc_set_time
 *
 * Description:
 *   Set the RTC time based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   time - Pointer to time structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_set_time(const struct ra8p_rtc_time_s *time);

/****************************************************************************
 * Name: ra8p_rtc_set_alarm
 *
 * Description:
 *   Set RTC alarm based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   alarm - Pointer to alarm structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_set_alarm(const struct ra8p_rtc_alarm_s *alarm);

/****************************************************************************
 * Name: ra8p_rtc_enable_alarm
 *
 * Description:
 *   Enable/disable RTC alarm based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_enable_alarm(bool enable);

/****************************************************************************
 * Name: ra8p_rtc_set_calibration
 *
 * Description:
 *   Set RTC calibration based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   adjustment - Calibration value (-63 to +63)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_set_calibration(int8_t adjustment);

/****************************************************************************
 * Name: ra8p_rtc_enable_periodic_irq
 *
 * Description:
 *   Enable/disable RTC periodic interrupt based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_enable_periodic_irq(bool enable);

/****************************************************************************
 * Name: ra8p_rtc_set_periodic_interval
 *
 * Description:
 *   Set RTC periodic interrupt interval based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   interval - Interval code (0-15, see RCR1_PES values)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_set_periodic_interval(uint8_t interval);

/****************************************************************************
 * Name: ra8p_rtc_is_running
 *
 * Description:
 *   Check if RTC is running based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if running, false otherwise
 *
 ****************************************************************************/

bool ra8p_rtc_is_running(void);

/****************************************************************************
 * Name: ra8p_rtc_is_initialized
 *
 * Description:
 *   Check if RTC is initialized based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if initialized, false otherwise
 *
 ****************************************************************************/

bool ra8p_rtc_is_initialized(void);

/****************************************************************************
 * Name: ra8p_rtc_get_status
 *
 * Description:
 *   Get RTC status flags based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Status flags
 *
 ****************************************************************************/

uint8_t ra8p_rtc_get_status(void);

/****************************************************************************
 * Name: ra8p_rtc_clear_status
 *
 * Description:
 *   Clear RTC status flags based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   flags - Status flags to clear
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_clear_status(uint8_t flags);

/****************************************************************************
 * Name: ra8p_rtc_get_calibration
 *
 * Description:
 *   Get current RTC calibration value based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Current calibration value
 *
 ****************************************************************************/

int8_t ra8p_rtc_get_calibration(void);

/****************************************************************************
 * Name: ra8p_rtc_reset_counter
 *
 * Description:
 *   Reset RTC counter based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_reset_counter(void);

/****************************************************************************
 * Name: ra8p_rtc_enable_interrupts
 *
 * Description:
 *   Enable RTC interrupts based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_enable_interrupts(void);

/****************************************************************************
 * Name: ra8p_rtc_disable_interrupts
 *
 * Description:
 *   Disable RTC interrupts based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_disable_interrupts(void);

/****************************************************************************
 * Name: ra8p_rtc_get_alarm_status
 *
 * Description:
 *   Get RTC alarm status based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if alarm occurred, false otherwise
 *
 ****************************************************************************/

bool ra8p_rtc_get_alarm_status(void);

/****************************************************************************
 * Name: ra8p_rtc_clear_alarm_status
 *
 * Description:
 *   Clear RTC alarm status based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_clear_alarm_status(void);

/****************************************************************************
 * Name: ra8p_rtc_is_alarm_enabled
 *
 * Description:
 *   Check if RTC alarm is enabled based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if alarm enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_rtc_is_alarm_enabled(void);

/****************************************************************************
 * Name: ra8p_rtc_set_clock_source
 *
 * Description:
 *   Set RTC clock source (SUBCLK or LOCO) based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   subclk - true for sub-clock, false for LOCO
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_set_clock_source(bool subclk);

/****************************************************************************
 * Name: ra8p_rtc_get_clock_source
 *
 * Description:
 *   Get current RTC clock source based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if SUBCLK, false if LOCO
 *
 ****************************************************************************/

bool ra8p_rtc_get_clock_source(void);

/****************************************************************************
 * Name: ra8p_rtc_enable_power_save
 *
 * Description:
 *   Enable/disable RTC power saving mode based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   enable - true to enable power save, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_enable_power_save(bool enable);

/****************************************************************************
 * Name: ra8p_rtc_get_alarm_info
 *
 * Description:
 *   Get RTC alarm information based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   alarm - Pointer to alarm structure to fill
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_get_alarm_info(struct ra8p_rtc_alarm_s *alarm);

/****************************************************************************
 * Name: ra8p_rtc_get_error_flags
 *
 * Description:
 *   Get RTC error flags based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Error flags
 *
 ****************************************************************************/

uint8_t ra8p_rtc_get_error_flags(void);

/****************************************************************************
 * Name: ra8p_rtc_clear_errors
 *
 * Description:
 *   Clear RTC error flags based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   flags - Error flags to clear
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_clear_errors(uint8_t flags);

/****************************************************************************
 * Name: ra8p_rtc_get_frequency
 *
 * Description:
 *   Get RTC clock frequency based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   RTC clock frequency in Hz
 *
 ****************************************************************************/

uint32_t ra8p_rtc_get_frequency(void);

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_RTC_H */