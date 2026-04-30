/****************************************************************************
 * arch/arm/src/ra8p/ra8p_rtc.h
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

#ifndef __ARCH_ARM_SRC_RA8P_RA8P_RTC_H
#define __ARCH_ARM_SRC_RA8P_RA8P_RTC_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <stdint.h>
#include <stdbool.h>
#include <time.h>

#include "hardware/ra8p_rtc.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* RTC timeout in milliseconds */
#define RA8P_RTC_TIMEOUT_MS              1000

/* RTC block size */
#define RA8P_RTC_BLOCK_SIZE              512

/* RTC calendar range */
#define RA8P_RTC_MIN_YEAR                2000  /* Minimum supported year */
#define RA8P_RTC_MAX_YEAR                2099  /* Maximum supported year */

/* Default time zone (UTC) */
#define RA8P_RTC_DEFAULT_TIMEZONE        0

/* RTC alarm index */
#define RA8P_RTC_ALARM_INDEX             0     /* Only one alarm supported */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* RA8P1 RTC time structure */
struct ra8p_rtc_time_s
{
  uint16_t year;                     /* Calendar year (2000-2099) */
  uint8_t month;                     /* Month (1-12) */
  uint8_t day;                       /* Day of month (1-31) */
  uint8_t hour;                      /* Hour (0-23) */
  uint8_t minute;                    /* Minute (0-59) */
  uint8_t second;                    /* Second (0-59) */
  uint8_t weekday;                   /* Weekday (0=Sunday, 1=Monday, ..., 6=Saturday) */
};

/* RA8P1 RTC alarm structure */
struct ra8p_rtc_alarm_s
{
  uint8_t month;                     /* Month (1-12, 0xFF to disable) */
  uint8_t day;                       /* Day of month (1-31, 0xFF to disable) */
  uint8_t hour;                      /* Hour (0-23, 0xFF to disable) */
  uint8_t minute;                    /* Minute (0-59, 0xFF to disable) */
  uint8_t second;                    /* Second (0-59, 0xFF to disable) */
  uint8_t weekday;                   /* Weekday (0-6, 0xFF to disable) */
  bool enabled;                      /* Alarm enabled */
  bool repeat_daily;                 /* Daily repeat enabled */
  uint8_t index;                     /* Alarm index */
};

/* RTC calibration structure */
struct ra8p_rtc_calibration_s
{
  int8_t adjustment;                /* Adjustment value (-63 to +63) */
  bool automatic;                   /* Automatic adjustment */
  bool enabled;                     /* Calibration enabled */
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
 * Name: ra8p_rtc_get_time
 *
 * Description:
 *   Get the RTC time based on Nuttx RTC driver implementation.
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
 * Name: ra8p_rtc_set_alarm
 *
 * Description:
 *   Set the RTC alarm based on Nuttx RTC driver implementation.
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
 *   Enable/disable the RTC alarm based on Nuttx RTC driver implementation.
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
 * Name: ra8p_rtc_get_alarm
 *
 * Description:
 *   Get the RTC alarm configuration based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   alarm - Pointer to alarm structure to fill
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_get_alarm(struct ra8p_rtc_alarm_s *alarm);

/****************************************************************************
 * Name: ra8p_rtc_set_calibration
 *
 * Description:
 *   Set RTC calibration based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   cal_val - Calibration value (-63 to +63)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_set_calibration(int8_t cal_val);

/****************************************************************************
 * Name: ra8p_rtc_get_calibration
 *
 * Description:
 *   Get RTC calibration value based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Calibration value (-63 to +63)
 *
 ****************************************************************************/

int8_t ra8p_rtc_get_calibration(void);

/****************************************************************************
 * Name: ra8p_rtc_enable_periodic
 *
 * Description:
 *   Enable periodic interrupt based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_enable_periodic(bool enable);

/****************************************************************************
 * Name: ra8p_rtc_set_periodic_interval
 *
 * Description:
 *   Set periodic interrupt interval based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   interval - Interval code (0-15)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_set_periodic_interval(uint8_t interval);

/****************************************************************************
 * Name: ra8p_rtc_is_initialized
 *
 * Description:
 *   Check if RTC is properly initialized based on Nuttx RTC driver implementation.
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
 * Name: ra8p_rtc_clear_overflow_flag
 *
 * Description:
 *   Clear RTC overflow flag based on Nuttx RTC driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_rtc_clear_overflow_flag(void);

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

uint32_t ra8p_rtc_get_status(void);

#endif /* __ARCH_ARM_SRC_RA8P_RA8P_RTC_H */