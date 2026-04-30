/****************************************************************************
 * arch/arm/src/ra8p/ra8p_rtc.c
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

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <sys/types.h>
#include <stdint.h>
#include <stdbool.h>
#include <time.h>
#include <errno.h>
#include <debug.h>

#include "chip.h"
#include "arm_internal.h"
#include "hardware/ra8p_rtc.h"
#include "hardware/ra8p_memorymap.h"

#ifdef CONFIG_RA8P_RTC

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#define RA8P_RTC_BASE                   RA8P_RTC_BASE

/* RTC timeout in milliseconds */
#define RA8P_RTC_TIMEOUT_MS             1000

/* Maximum year supported by RTC (2000-2099) */
#define RA8P_RTC_MIN_YEAR               2000
#define RA8P_RTC_MAX_YEAR               2099

/* Reference year for calculation (2000) */
#define RA8P_RTC_YEAR_REF               2000

/* Sub-clock frequency (32.768 kHz) */
#define RA8P_RTC_SUBCLK_FREQ            32768

/****************************************************************************
 * Private Types
 ****************************************************************************/

/* RA8P1 RTC driver state structure */

struct ra8p_rtc_priv_s
{
  uint32_t base;                              /* Base address of RTC registers */
  bool initialized;                           /* Initialization flag */
  bool enabled;                               /* Enable flag */
  bool clock_source_subclk;                  /* Clock source is sub-clock */
  bool alarm_enabled;                         /* Alarm enabled flag */
  bool periodic_irq_enabled;                  /* Periodic interrupt enabled */
  uint8_t periodic_interval;                  /* Periodic interrupt interval */
  bool calibration_enabled;                   /* Calibration enabled */
  uint8_t calibration_value;                  /* Calibration value */
  struct tm current_time;                     /* Current time value */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int rtc_wait_ready(struct ra8p_rtc_priv_s *priv);
static int rtc_convert_tm_to_regs(const struct tm *time, uint8_t *regs);
static int rtc_convert_regs_to_tm(const uint8_t *regs, struct tm *time);
static void rtc_putreg32(struct ra8p_rtc_priv_s *priv, uint32_t offset, uint32_t value);
static uint32_t rtc_getreg32(struct ra8p_rtc_priv_s *priv, uint32_t offset);
static void rtc_putreg16(struct ra8p_rtc_priv_s *priv, uint32_t offset, uint16_t value);
static uint16_t rtc_getreg16(struct ra8p_rtc_priv_s *priv, uint32_t offset);
static void rtc_putreg8(struct ra8p_rtc_priv_s *priv, uint32_t offset, uint8_t value);
static uint8_t rtc_getreg8(struct ra8p_rtc_priv_s *priv, uint32_t offset);
static int rtc_set_alarm_regs(struct ra8p_rtc_priv_s *priv, const struct tm *time);
static int rtc_get_alarm_regs(struct ra8p_rtc_priv_s *priv, struct tm *time);
static int rtc_enable_alarm(struct ra8p_rtc_priv_s *priv, bool enable);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static struct ra8p_rtc_priv_s g_rtc;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: rtc_putreg32
 ****************************************************************************/

static inline void rtc_putreg32(struct ra8p_rtc_priv_s *priv, uint32_t offset, uint32_t value)
{
  putreg32(value, priv->base + offset);
}

/****************************************************************************
 * Name: rtc_getreg32
 ****************************************************************************/

static inline uint32_t rtc_getreg32(struct ra8p_rtc_priv_s *priv, uint32_t offset)
{
  return getreg32(priv->base + offset);
}

/****************************************************************************
 * Name: rtc_putreg16
 ****************************************************************************/

static inline void rtc_putreg16(struct ra8p_rtc_priv_s *priv, uint32_t offset, uint16_t value)
{
  putreg16(value, priv->base + offset);
}

/****************************************************************************
 * Name: rtc_getreg16
 ****************************************************************************/

static inline uint16_t rtc_getreg16(struct ra8p_rtc_priv_s *priv, uint32_t offset)
{
  return getreg16(priv->base + offset);
}

/****************************************************************************
 * Name: rtc_putreg8
 ****************************************************************************/

static inline void rtc_putreg8(struct ra8p_rtc_priv_s *priv, uint32_t offset, uint8_t value)
{
  putreg8(value, priv->base + offset);
}

/****************************************************************************
 * Name: rtc_getreg8
 ****************************************************************************/

static inline uint8_t rtc_getreg8(struct ra8p_rtc_priv_s *priv, uint32_t offset)
{
  return getreg8(priv->base + offset);
}

/****************************************************************************
 * Name: rtc_wait_ready
 ****************************************************************************/

static int rtc_wait_ready(struct ra8p_rtc_priv_s *priv)
{
  volatile int timeout = RA8P_RTC_TIMEOUT_MS * 1000;  /* microseconds */

  /* Wait for register write to be ready */
  while ((rtc_getreg16(priv, RA8P_RTC_RCR2_OFFSET) & RA8P_RTC_RCR2_RTCSTP) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  return OK;
}

/****************************************************************************
 * Name: rtc_bcd2bin
 ****************************************************************************/

static uint8_t rtc_bcd2bin(uint8_t bcd)
{
  return ((bcd >> 4) * 10) + (bcd & 0x0F);
}

/****************************************************************************
 * Name: rtc_bin2bcd
 ****************************************************************************/

static uint8_t rtc_bin2bcd(uint8_t bin)
{
  return ((bin / 10) << 4) | (bin % 10);
}

/****************************************************************************
 * Name: rtc_convert_tm_to_regs
 ****************************************************************************/

static int rtc_convert_tm_to_regs(const struct tm *time, uint8_t *regs)
{
  if (time == NULL || regs == NULL)
    {
      return -EINVAL;
    }

  /* Validate year range (2000-2099) */
  if (time->tm_year + 1900 < RA8P_RTC_MIN_YEAR || 
      time->tm_year + 1900 > RA8P_RTC_MAX_YEAR)
    {
      return -EINVAL;
    }

  /* Validate month (0-11, converted to 1-12) */
  if (time->tm_mon < 0 || time->tm_mon > 11)
    {
      return -EINVAL;
    }

  /* Validate day (1-31) */
  if (time->tm_mday < 1 || time->tm_mday > 31)
    {
      return -EINVAL;
    }

  /* Validate hour (0-23) */
  if (time->tm_hour < 0 || time->tm_hour > 23)
    {
      return -EINVAL;
    }

  /* Validate minute (0-59) */
  if (time->tm_min < 0 || time->tm_min > 59)
    {
      return -EINVAL;
    }

  /* Validate second (0-59) */
  if (time->tm_sec < 0 || time->tm_sec > 59)
    {
      return -EINVAL;
    }

  /* Convert time structure to BCD registers */
  regs[0] = rtc_bin2bcd(time->tm_sec);       /* Second */
  regs[1] = rtc_bin2bcd(time->tm_min);       /* Minute */
  regs[2] = rtc_bin2bcd(time->tm_hour);      /* Hour */
  regs[3] = time->tm_wday & 0x07;           /* Weekday (0-6, 3 bits) */
  regs[4] = rtc_bin2bcd(time->tm_mday);      /* Day of month */
  regs[5] = rtc_bin2bcd(time->tm_mon + 1);   /* Month (0-11 -> 1-12) */
  regs[6] = rtc_bin2bcd((time->tm_year + 1900) - RA8P_RTC_YEAR_REF);  /* Year (relative to 2000) */

  return OK;
}

/****************************************************************************
 * Name: rtc_convert_regs_to_tm
 ****************************************************************************/

static int rtc_convert_regs_to_tm(const uint8_t *regs, struct tm *time)
{
  if (time == NULL || regs == NULL)
    {
      return -EINVAL;
    }

  /* Convert BCD registers to time structure */
  time->tm_sec = rtc_bcd2bin(regs[0] & 0x7F);  /* Second */
  time->tm_min = rtc_bcd2bin(regs[1] & 0x7F);  /* Minute */
  time->tm_hour = rtc_bcd2bin(regs[2] & 0x3F); /* Hour */
  time->tm_wday = regs[3] & 0x07;              /* Weekday */
  time->tm_mday = rtc_bcd2bin(regs[4] & 0x3F); /* Day of month */
  time->tm_mon = rtc_bcd2bin(regs[5] & 0x1F) - 1; /* Month (0-11) */
  time->tm_year = rtc_bcd2bin(regs[6] & 0xFF) + (RA8P_RTC_YEAR_REF - 1900); /* Year */

  /* Clear unsupported fields */
  time->tm_yday = -1;  /* Day of year not supported */
  time->tm_isdst = -1; /* DST info not supported */

  return OK;
}

/****************************************************************************
 * Name: rtc_enable_alarm
 ****************************************************************************/

static int rtc_enable_alarm(struct ra8p_rtc_priv_s *priv, bool enable)
{
  uint8_t rcr1;

  rcr1 = rtc_getreg8(priv, RA8P_RTC_RCR1_OFFSET);

  if (enable)
    {
      rcr1 |= RA8P_RTC_RCR1_AIE;  /* Enable alarm interrupt */
    }
  else
    {
      rcr1 &= ~RA8P_RTC_RCR1_AIE; /* Disable alarm interrupt */
    }

  rtc_putreg8(priv, RA8P_RTC_RCR1_OFFSET, rcr1);

  priv->alarm_enabled = enable;

  return OK;
}

/****************************************************************************
 * Name: rtc_set_alarm_regs
 ****************************************************************************/

static int rtc_set_alarm_regs(struct ra8p_rtc_priv_s *priv, const struct tm *time)
{
  uint8_t alarm_regs[6];
  uint8_t rcr2;

  if (time == NULL)
    {
      return -EINVAL;
    }

  /* Convert alarm time to BCD registers */
  alarm_regs[0] = (time->tm_year < 0) ? 0xFF : rtc_bin2bcd((time->tm_year + 1900) - RA8P_RTC_YEAR_REF);
  alarm_regs[1] = (time->tm_mon < 0) ? 0xFF : rtc_bin2bcd(time->tm_mon + 1);
  alarm_regs[2] = (time->tm_mday < 0) ? 0xFF : rtc_bin2bcd(time->tm_mday);
  alarm_regs[3] = rtc_bin2bcd(time->tm_hour);
  alarm_regs[4] = rtc_bin2bcd(time->tm_min);
  alarm_regs[5] = rtc_bin2bcd(time->tm_sec);

  /* Wait for ready */
  int ret = rtc_wait_ready(priv);
  if (ret != OK)
    {
      return ret;
    }

  /* Stop RTC temporarily to set alarm */
  rcr2 = rtc_getreg8(priv, RA8P_RTC_RCR2_OFFSET);
  rcr2 |= RA8P_RTC_RCR2_STOP;  /* Stop RTC */
  rtc_putreg8(priv, RA8P_RTC_RCR2_OFFSET, rcr2);

  /* Set alarm registers */
  rtc_putreg8(priv, RA8P_RTC_RSECAR_OFFSET, alarm_regs[5]);  /* Alarm Second */
  rtc_putreg8(priv, RA8P_RTC_RMINAR_OFFSET, alarm_regs[4]);  /* Alarm Minute */
  rtc_putreg8(priv, RA8P_RTC_RHRAR_OFFSET, alarm_regs[3]);   /* Alarm Hour */
  rtc_putreg8(priv, RA8P_RTC_RDAYAR_OFFSET, alarm_regs[2]);  /* Alarm Day */
  rtc_putreg8(priv, RA8P_RTC_RMONAR_OFFSET, alarm_regs[1]);  /* Alarm Month */
  rtc_putreg8(priv, RA8P_RTC_RYAR_OFFSET, alarm_regs[0]);    /* Alarm Year */

  /* Restart RTC */
  rcr2 &= ~RA8P_RTC_RCR2_STOP;  /* Clear stop bit */
  rtc_putreg8(priv, RA8P_RTC_RCR2_OFFSET, rcr2);

  return OK;
}

/****************************************************************************
 * Public Functions
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

int ra8p_rtc_initialize(void)
{
  struct ra8p_rtc_priv_s *priv = &g_rtc;
  uint8_t rcr1;
  uint8_t rcr2;
  uint8_t rcr3;

  priv->base = RA8P_RTC_BASE;
  priv->initialized = false;
  priv->enabled = false;
  priv->clock_source_subclk = true;  /* Use sub-clock by default */
  priv->alarm_enabled = false;
  priv->periodic_irq_enabled = false;
  priv->periodic_interval = 7;  /* Default to 1 second */
  priv->calibration_enabled = false;
  priv->calibration_value = 0;

  /* Configure clock source */
  rcr3 = rtc_getreg8(priv, RA8P_RTC_RCR3_OFFSET);
  if (priv->clock_source_subclk)
    {
      rcr3 &= ~RA8P_RTC_RCR3_RCKSEL;  /* Select sub-clock */
    }
  else
    {
      rcr3 |= RA8P_RTC_RCR3_RCKSEL;   /* Select LOCO */
    }
  rtc_putreg8(priv, RA8P_RTC_RCR3_OFFSET, rcr3);

  /* Configure interrupt settings - disable all interrupts initially */
  rcr1 = rtc_getreg8(priv, RA8P_RTC_RCR1_OFFSET);
  rcr1 &= ~(RA8P_RTC_RCR1_CIE |      /* Clear carry interrupt */
            RA8P_RTC_RCR1_AIE |      /* Clear alarm interrupt */
            RA8P_RTC_RCR1_PIE);      /* Clear periodic interrupt */
  rcr1 |= (priv->periodic_interval << RA8P_RTC_RCR1_PES_SHIFT);  /* Set periodic interval */
  rtc_putreg8(priv, RA8P_RTC_RCR1_OFFSET, rcr1);

  /* Configure control register */
  rcr2 = rtc_getreg8(priv, RA8P_RTC_RCR2_OFFSET);
  rcr2 |= RA8P_RTC_RCR2_START;        /* Start RTC */
  rcr2 &= ~RA8P_RTC_RCR2_STOP;        /* Clear stop bit */
  rcr2 |= RA8P_RTC_RCR2_RTCEN;        /* Enable RTC */
  rcr2 |= RA8P_RTC_RCR2_RTCE;         /* Enable RTC clock */
  rcr2 &= ~RA8P_RTC_RCR2_ADJ;         /* Clear adjustment */
  rcr2 &= ~RA8P_RTC_RCR2_AADJ;        /* Clear automatic adjustment */
  rtc_putreg8(priv, RA8P_RTC_RCR2_OFFSET, rcr2);

  /* Wait for RTC to be ready */
  volatile int timeout = RA8P_RTC_TIMEOUT_MS * 1000;
  while ((rtc_getreg8(priv, RA8P_RTC_RCR2_OFFSET) & RA8P_RTC_RCR2_START) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  priv->initialized = true;
  priv->enabled = true;

  rtcinfo("RTC initialized at 0x%08x\n", priv->base);
  return OK;
}

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

int ra8p_rtc_get_time(struct tm *time)
{
  struct ra8p_rtc_priv_s *priv = &g_rtc;
  uint8_t time_regs[7];
  int ret;

  if (!priv->initialized || time == NULL)
    {
      return -EAGAIN;
    }

  /* Check if RTC is running */
  uint8_t rcr2 = rtc_getreg8(priv, RA8P_RTC_RCR2_OFFSET);
  if (rcr2 & RA8P_RTC_RCR2_STOP)  /* RTC is stopped */
    {
      return -EAGAIN;
    }

  /* Get time registers */
  time_regs[0] = rtc_getreg8(priv, RA8P_RTC_RSECR_OFFSET);  /* Second */
  time_regs[1] = rtc_getreg8(priv, RA8P_RTC_RMINR_OFFSET);  /* Minute */
  time_regs[2] = rtc_getreg8(priv, RA8P_RTC_RHOUR_OFFSET);   /* Hour */
  time_regs[3] = rtc_getreg8(priv, RA8P_RTC_RWKR_OFFSET);    /* Weekday */
  time_regs[4] = rtc_getreg8(priv, RA8P_RTC_RDAYR_OFFSET);   /* Day */
  time_regs[5] = rtc_getreg8(priv, RA8P_RTC_RMONR_OFFSET);   /* Month */
  time_regs[6] = rtc_getreg8(priv, RA8P_RTC_RYR_OFFSET);     /* Year */

  /* Convert registers to time structure */
  ret = rtc_convert_regs_to_tm(time_regs, time);
  if (ret != OK)
    {
      return ret;
    }

  /* Convert relative year to full year */
  time->tm_year += RA8P_RTC_YEAR_REF - 1900;

  rtcinfo("RTC time: %04d-%02d-%02d %02d:%02d:%02d\n",
          time->tm_year + 1900, time->tm_mon + 1, time->tm_mday,
          time->tm_hour, time->tm_min, time->tm_sec);
  return OK;
}

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

int ra8p_rtc_set_time(const struct tm *time)
{
  struct ra8p_rtc_priv_s *priv = &g_rtc;
  uint8_t time_regs[7];
  uint8_t rcr2;
  int ret;

  if (!priv->initialized || time == NULL)
    {
      return -EAGAIN;
    }

  /* Convert time to registers */
  ret = rtc_convert_tm_to_regs(time, time_regs);
  if (ret != OK)
    {
      return ret;
    }

  /* Wait for ready */
  ret = rtc_wait_ready(priv);
  if (ret != OK)
    {
      return ret;
    }

  /* Stop RTC temporarily to set time */
  rcr2 = rtc_getreg8(priv, RA8P_RTC_RCR2_OFFSET);
  rcr2 |= RA8P_RTC_RCR2_STOP;  /* Stop RTC */
  rtc_putreg8(priv, RA8P_RTC_RCR2_OFFSET, rcr2);

  /* Wait for stop to take effect */
  volatile int timeout = RA8P_RTC_TIMEOUT_MS * 1000;
  while ((rtc_getreg8(priv, RA8P_RTC_RCR2_OFFSET) & RA8P_RTC_RCR2_START) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Set time registers */
  rtc_putreg8(priv, RA8P_RTC_RSECR_OFFSET, time_regs[0]);  /* Second */
  rtc_putreg8(priv, RA8P_RTC_RMINR_OFFSET, time_regs[1]);  /* Minute */
  rtc_putreg8(priv, RA8P_RTC_RHOUR_OFFSET, time_regs[2]);   /* Hour */
  rtc_putreg8(priv, RA8P_RTC_RWKR_OFFSET, time_regs[3]);    /* Weekday */
  rtc_putreg8(priv, RA8P_RTC_RDAYR_OFFSET, time_regs[4]);   /* Day */
  rtc_putreg8(priv, RA8P_RTC_RMONR_OFFSET, time_regs[5]);   /* Month */
  rtc_putreg8(priv, RA8P_RTC_RYR_OFFSET, time_regs[6]);     /* Year */

  /* Restart RTC */
  rcr2 &= ~RA8P_RTC_RCR2_STOP;  /* Clear stop bit */
  rcr2 |= RA8P_RTC_RCR2_START;  /* Start RTC */
  rtc_putreg8(priv, RA8P_RTC_RCR2_OFFSET, rcr2);

  /* Update internal time */
  memcpy(&priv->current_time, time, sizeof(struct tm));

  rtcinfo("RTC time set to %04d-%02d-%02d %02d:%02d:%02d\n",
          time->tm_year + 1900, time->tm_mon + 1, time->tm_mday,
          time->tm_hour, time->tm_min, time->tm_sec);
  return OK;
}

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

int ra8p_rtc_set_alarm(const struct tm *alarm)
{
  struct ra8p_rtc_priv_s *priv = &g_rtc;
  int ret;

  if (!priv->initialized || alarm == NULL)
    {
      return -EAGAIN;
    }

  /* Set alarm registers */
  ret = rtc_set_alarm_regs(priv, alarm);
  if (ret != OK)
    {
      return ret;
    }

  /* Enable alarm interrupt */
  ret = rtc_enable_alarm(priv, true);
  if (ret != OK)
    {
      return ret;
    }

  priv->alarm_enabled = true;

  rtcinfo("RTC alarm set to %04d-%02d-%02d %02d:%02d:%02d\n",
          alarm->tm_year + 1900, alarm->tm_mon + 1, alarm->tm_mday,
          alarm->tm_hour, alarm->tm_min, alarm->tm_sec);
  return OK;
}

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

int ra8p_rtc_enable_alarm(bool enable)
{
  struct ra8p_rtc_priv_s *priv = &g_rtc;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  return rtc_enable_alarm(priv, enable);
}

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

int ra8p_rtc_set_calibration(int8_t adjustment)
{
  struct ra8p_rtc_priv_s *priv = &g_rtc;
  uint8_t rcrv;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  if (adjustment < -63 || adjustment > 63)
    {
      return -EINVAL;
    }

  /* Set calibration register */
  rcrv = (adjustment & 0x3F);
  if (adjustment < 0)
    {
      rcrv |= 0x40;  /* Set sign bit for negative values */
    }

  rtc_putreg8(priv, RA8P_RTC_RCRV_OFFSET, rcrv);

  priv->calibration_value = adjustment;
  priv->calibration_enabled = (adjustment != 0);

  rtcinfo("RTC calibration set to %d\n", adjustment);
  return OK;
}

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

int ra8p_rtc_enable_periodic_irq(bool enable)
{
  struct ra8p_rtc_priv_s *priv = &g_rtc;
  uint8_t rcr1;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  rcr1 = rtc_getreg8(priv, RA8P_RTC_RCR1_OFFSET);

  if (enable)
    {
      rcr1 |= RA8P_RTC_RCR1_PIE;  /* Enable periodic interrupt */
    }
  else
    {
      rcr1 &= ~RA8P_RTC_RCR1_PIE; /* Disable periodic interrupt */
    }

  rtc_putreg8(priv, RA8P_RTC_RCR1_OFFSET, rcr1);

  priv->periodic_irq_enabled = enable;

  rtcinfo("RTC periodic IRQ %s\n", enable ? "enabled" : "disabled");
  return OK;
}

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

int ra8p_rtc_set_periodic_interval(uint8_t interval)
{
  struct ra8p_rtc_priv_s *priv = &g_rtc;
  uint8_t rcr1;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  if (interval > 15)
    {
      return -EINVAL;
    }

  rcr1 = rtc_getreg8(priv, RA8P_RTC_RCR1_OFFSET);
  rcr1 &= ~RA8P_RTC_RCR1_PES_MASK;  /* Clear periodic interval */
  rcr1 |= (interval << RA8P_RTC_RCR1_PES_SHIFT);  /* Set new interval */
  rtc_putreg8(priv, RA8P_RTC_RCR1_OFFSET, rcr1);

  priv->periodic_interval = interval;

  rtcinfo("RTC periodic interval set to code %u\n", interval);
  return OK;
}

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

bool ra8p_rtc_is_running(void)
{
  struct ra8p_rtc_priv_s *priv = &g_rtc;
  uint8_t rcr2;

  if (!priv->initialized)
    {
      return false;
    }

  rcr2 = rtc_getreg8(priv, RA8P_RTC_RCR2_OFFSET);
  return !(rcr2 & RA8P_RTC_RCR2_STOP);  /* If STOP bit is clear, RTC is running */
}

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

bool ra8p_rtc_is_initialized(void)
{
  return g_rtc.initialized;
}

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

uint8_t ra8p_rtc_get_status(void)
{
  struct ra8p_rtc_priv_s *priv = &g_rtc;

  if (!priv->initialized)
    {
      return 0;
    }

  return rtc_getreg8(priv, RA8P_RTC_RSR_OFFSET);
}

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

int ra8p_rtc_clear_status(uint8_t flags)
{
  struct ra8p_rtc_priv_s *priv = &g_rtc;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  /* Clear status flags by writing to RSR register */
  rtc_putreg8(priv, RA8P_RTC_RSR_OFFSET, flags);

  return OK;
}

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

int8_t ra8p_rtc_get_calibration(void)
{
  struct ra8p_rtc_priv_s *priv = &g_rtc;
  uint8_t rcrv;
  int8_t calibration;

  if (!priv->initialized)
    {
      return 0;
    }

  rcrv = rtc_getreg8(priv, RA8P_RTC_RCRV_OFFSET);
  calibration = rcrv & 0x3F;

  if (rcrv & 0x40)  /* Negative value */
    {
      calibration = -calibration;
    }

  return calibration;
}

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

int ra8p_rtc_reset_counter(void)
{
  struct ra8p_rtc_priv_s *priv = &g_rtc;
  uint8_t rcr2;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  /* Reset RTC */
  rcr2 = rtc_getreg8(priv, RA8P_RTC_RCR2_OFFSET);
  rcr2 |= RA8P_RTC_RCR2_RESET;  /* Set reset */
  rtc_putreg8(priv, RA8P_RTC_RCR2_OFFSET, rcr2);

  /* Wait for reset to complete */
  volatile int timeout = RA8P_RTC_TIMEOUT_MS * 1000;
  while ((rtc_getreg8(priv, RA8P_RTC_RCR2_OFFSET) & RA8P_RTC_RCR2_RESET) && timeout--)
    {
      up_udelay(1);
    }

  if (timeout <= 0)
    {
      return -ETIMEDOUT;
    }

  /* Clear reset */
  rcr2 &= ~RA8P_RTC_RCR2_RESET;
  rtc_putreg8(priv, RA8P_RTC_RCR2_OFFSET, rcr2);

  return OK;
}

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

int ra8p_rtc_set_clock_source(bool subclk)
{
  struct ra8p_rtc_priv_s *priv = &g_rtc;
  uint8_t rcr3;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  rcr3 = rtc_getreg8(priv, RA8P_RTC_RCR3_OFFSET);
  rcr3 &= ~RA8P_RTC_RCR3_RCKSEL;  /* Clear clock select */
  if (!subclk)
    {
      rcr3 |= RA8P_RTC_RCR3_RCKSEL;  /* Select LOCO if not subclk */
    }
  rtc_putreg8(priv, RA8P_RTC_RCR3_OFFSET, rcr3);

  priv->clock_source_subclk = subclk;

  rtcinfo("RTC clock source set to %s\n", subclk ? "SUBCLK" : "LOCO");
  return OK;
}

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

bool ra8p_rtc_get_clock_source(void)
{
  struct ra8p_rtc_priv_s *priv = &g_rtc;
  uint8_t rcr3;

  if (!priv->initialized)
    {
      return true;  /* Default to sub-clock */
    }

  rcr3 = rtc_getreg8(priv, RA8P_RTC_RCR3_OFFSET);
  return !(rcr3 & RA8P_RTC_RCR3_RCKSEL);  /* If RCKSEL is clear, SUBCLK is selected */
}

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

bool ra8p_rtc_get_alarm_status(void)
{
  struct ra8p_rtc_priv_s *priv = &g_rtc;
  uint8_t rcr1;

  if (!priv->initialized)
    {
      return false;
    }

  rcr1 = rtc_getreg8(priv, RA8P_RTC_RCR1_OFFSET);
  return (rcr1 & RA8P_RTC_RCR1_AF) != 0;  /* Alarm flag */
}

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

int ra8p_rtc_clear_alarm_status(void)
{
  struct ra8p_rtc_priv_s *priv = &g_rtc;
  uint8_t rcr1;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  rcr1 = rtc_getreg8(priv, RA8P_RTC_RCR1_OFFSET);
  rcr1 &= ~RA8P_RTC_RCR1_AF;  /* Clear alarm flag */
  rtc_putreg8(priv, RA8P_RTC_RCR1_OFFSET, rcr1);

  return OK;
}

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

bool ra8p_rtc_is_alarm_enabled(void)
{
  struct ra8p_rtc_priv_s *priv = &g_rtc;

  if (!priv->initialized)
    {
      return false;
    }

  return priv->alarm_enabled;
}

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

int ra8p_rtc_enable_power_save(bool enable)
{
  struct ra8p_rtc_priv_s *priv = &g_rtc;
  uint8_t rcr2;

  if (!priv->initialized)
    {
      return -EAGAIN;
    }

  rcr2 = rtc_getreg8(priv, RA8P_RTC_RCR2_OFFSET);

  if (enable)
    {
      rcr2 |= RA8P_RTC_RCR2_PSW;  /* Enable power save */
    }
  else
    {
      rcr2 &= ~RA8P_RTC_RCR2_PSW; /* Disable power save */
    }

  rtc_putreg8(priv, RA8P_RTC_RCR2_OFFSET, rcr2);

  return OK;
}

#endif /* CONFIG_RA8P_RTC */