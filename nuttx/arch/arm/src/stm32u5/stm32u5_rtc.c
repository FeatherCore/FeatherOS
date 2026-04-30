/****************************************************************************
 * arch/arm/src/stm32u5/stm32u5_rtc.c
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

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <sys/types.h>
#include <stdbool.h>
#include <time.h>
#include <errno.h>
#include <debug.h>

#include <nuttx/arch.h>
#include <nuttx/irq.h>
#include <nuttx/timers/rtc.h>

#include "arm_internal.h"
#include "chip.h"
#include "stm32_rcc.h"
#include "stm32_pwr.h"
#include "hardware/stm32u5_rtcc.h"
#include "stm32u5_rtc.h"

#ifdef CONFIG_RTC

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Configuration ************************************************************/

#ifndef CONFIG_RTC_DATETIME
#  error "CONFIG_RTC_DATETIME must be provided"
#endif

#ifdef CONFIG_RTC_ALARM
#  ifndef CONFIG_RTC_ALARM
#    error "CONFIG_RTC_ALARM must be provided"
#  endif
#endif

/* If CONFIG_RTC_ABSOLUTE_MAXDATE is defined, then check if the time to be
 * set is reasonable. The check is performed at the granularity of seconds.
 * Since the check is performed at the granularity of seconds, the
 * configuration really should be larger than 1 second to be effective.
 */

#ifdef CONFIG_RTC_ABSOLUTE_MAXDATE
#  define RTC_ABSOLUTE_MAXDATE_TS (time_t)((CONFIG_RTC_ABSOLUTE_MAXDATE_YEAR << 16) | \
                                       (CONFIG_RTC_ABSOLUTE_MAXDATE_MONTH << 8) | \
                                       (CONFIG_RTC_ABSOLUTE_MAXDATE_DAY))
#  define RTC_ABSOLUTE_MAXDATE_SEC (mktime(gmtime(&RTC_ABSOLUTE_MAXDATE_TS)))
#endif

/* If CONFIG_RTC_ABSOLUTE_MINDATE is defined, then check if the time to be
 * set is reasonable. The check is performed at the granularity of seconds.
 * Since the check is performed at the granularity of seconds, the
 * configuration really should be larger than 1 second to be effective.
 */

#ifdef CONFIG_RTC_ABSOLUTE_MINDATE
#  define RTC_ABSOLUTE_MINDATE_TS (time_t)((CONFIG_RTC_ABSOLUTE_MINDATE_YEAR << 16) | \
                                       (CONFIG_RTC_ABSOLUTE_MINDATE_MONTH << 8) | \
                                       (CONFIG_RTC_ABSOLUTE_MINDATE_DAY))
#  define RTC_ABSOLUTE_MINDATE_SEC (mktime(gmtime(&RTC_ABSOLUTE_MINDATE_TS)))
#endif

/* Debug ******************************************************************************/

#ifdef CONFIG_DEBUG_RTC_INFO
#  define rtcinfo(format, ...)       _info(format, ##__VA_ARGS__)
#else
#  define rtcinfo(format, ...)       (void)0
#endif

#ifdef CONFIG_DEBUG_RTC_ERROR
#  define rtcerr(format, ...)        _err(format, ##__VA_ARGS__)
#else
#  define rtcerr(format, ...)        (void)0
#endif

/****************************************************************************
 * Private Data
 ****************************************************************************/

/* Watchdog private data used to manage the RTC initialization and alarm
 * callbacks.
 */

static struct stm32u5_rtc_dev_s
{
  volatile bool init;        /* True: RTC has been initialized */
  volatile bool have_dtime;  /* True: A valid date time has been set */
#ifdef CONFIG_RTC_ALARM
  volatile bool alen[RTC_ALARM_LAST]; /* True: Alarm is enabled */
  alm_callback_t cb[RTC_ALARM_LAST];  /* Callback when the alarm expires */
  void *priv[RTC_ALARM_LAST];         /* Private data passed with the callback */
#endif
#ifdef CONFIG_RTC_PERIODIC
  wakeupcb_t periodic_cb;             /* Callback when the periodic wakes up */
#endif
} g_rtc_dev;

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32_rtc_sync
 *
 * Description:
 *   Wait for synchronisation between the RTC registers and the APB domain.
 *
 ****************************************************************************/

static inline int stm32_rtc_sync(void)
{
  volatile int timeout = 0xfff;

  /* Clear the RSF flag */

  modifyreg32(STM32U5_RTC_ISR, 0, RTC_ISR_RSF);

  /* Loop while the register is not synchronized */

  while (!(getreg32(STM32U5_RTC_ISR) & RTC_ISR_RSF) && timeout-- > 0)
    {
      up_udelay(1);
    }

  return timeout > 0 ? OK : -ETIMEDOUT;
}

/****************************************************************************
 * Name: stm32_rtc_enter_init
 *
 * Description:
 *   Enter initialization mode.
 *
 ****************************************************************************/

static inline int stm32_rtc_enter_init(void)
{
  int timeout = 0xfff;

  /* Enter init mode */

  modifyreg32(STM32U5_RTC_ISR, 0, RTC_ISR_INIT);

  /* Wait until init mode is entered */

  while (!(getreg32(STM32U5_RTC_ISR) & RTC_ISR_INITF) && timeout-- > 0)
    {
      up_udelay(1);
    }

  return timeout > 0 ? OK : -ETIMEDOUT;
}

/****************************************************************************
 * Name: stm32_rtc_exit_init
 *
 * Description:
 *   Exit initialization mode.
 *
 ****************************************************************************/

static inline void stm32_rtc_exit_init(void)
{
  /* Clear INIT bit to exit initialization mode */

  modifyreg32(STM32U5_RTC_ISR, RTC_ISR_INIT, 0);
}

/****************************************************************************
 * Name: stm32_rtc_bin2bcd
 *
 * Description:
 *   Convert a 2 digit binary to BCD format
 *
 ****************************************************************************/

static uint32_t stm32_rtc_bin2bcd(int value)
{
  uint32_t tens = value / 10;
  return (tens << 4) | (value - tens * 10);
}

/****************************************************************************
 * Name: stm32_rtc_bcd2bin
 *
 * Description:
 *   Convert a 2 digit BCD to binary format
 *
 ****************************************************************************/

static int stm32_rtc_bcd2bin(uint32_t value)
{
  uint32_t tens = ((value & 0xf0) >> 4) * 10;
  return tens + (value & 0x0f);
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32u5_rtc_is_initialized
 *
 * Description:
 *    Returns 'true' if the RTC has been initialized
 *    Returns 'false' if the RTC has never been initialized since first time
 *    power up, and the counters are stopped until it is first initialized.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Returns true if RTC has been initialized.
 *
 ****************************************************************************/

bool stm32u5_rtc_is_initialized(void)
{
  return g_rtc_dev.init;
}

/****************************************************************************
 * Name: stm32u5_rtc_havesettime
 *
 * Description:
 *   Check if RTC time has been set.
 *
 * Returned Value:
 *   Returns true if RTC date-time have been previously set.
 *
 ****************************************************************************/

bool stm32u5_rtc_havesettime(void)
{
  return g_rtc_dev.have_dtime;
}

/****************************************************************************
 * Name: stm32u5_rtc_setdatetime
 *
 * Description:
 *   Set the RTC to the provided time. RTC implementations which provide
 *   up_rtc_getdatetime() (CONFIG_RTC_DATETIME is selected) should provide
 *   this function.
 *
 * Input Parameters:
 *   tp - the time to use
 *
 * Returned Value:
 *   Zero (OK) on success; a negated errno on failure
 *
 ****************************************************************************/

#ifdef CONFIG_RTC_DATETIME
int stm32u5_rtc_setdatetime(const struct tm *tp)
{
  uint32_t tr;
  uint32_t dr;
  uint32_t cr;
  int ret;

  rtcinfo("Setting time to %" PRId32 ":%" PRId32 ":%" PRId32 "\n",
          tp->tm_hour, tp->tm_min, tp->tm_sec);

  /* Sample the time and date registers to ensure that they are stable */

  ret = stm32_rtc_sync();
  if (ret < 0)
    {
      rtcerr("ERROR: RTC sync failed: %d\n", ret);
      return ret;
    }

  /* Get current control register and disable calendar update */

  cr = getreg32(STM32U5_RTC_CR) & ~RTC_CR_BYPSHAD;

  /* Convert the struct tm format to RTC time and date formats */

  tr = (stm32_rtc_bin2bcd(tp->tm_sec)  << RTC_TR_SU_SHIFT) |
       (stm32_rtc_bin2bcd(tp->tm_min)  << RTC_TR_MNU_SHIFT) |
       (stm32_rtc_bin2bcd(tp->tm_hour) << RTC_TR_HU_SHIFT);

  dr = (stm32_rtc_bin2bcd(tp->tm_year - 100) << RTC_DR_YU_SHIFT) |
       (stm32_rtc_bin2bcd(tp->tm_mon + 1)    << RTC_DR_MU_SHIFT) |
       (stm32_rtc_bin2bcd(tp->tm_mday)       << RTC_DR_DU_SHIFT);

  /* The value of the year field is limited to the range [0,99]. */

  if (tp->tm_year > 199 || tp->tm_year < 100)
    {
      rtcerr("ERROR: Year must be in range 2000-2099.  Setting %d\n",
             tp->tm_year + 1900);
      return -EINVAL;
    }

  /* Disable the write protection before accessing the RTC registers */

  stm32_rtc_writeprotect(false);

  /* Enter initialization mode */

  ret = stm32_rtc_enter_init();
  if (ret < 0)
    {
      rtcerr("ERROR: Failed to enter init mode: %d\n", ret);
      goto errout_with_protect;
    }

  /* Set the TR and DR registers */

  putreg32(tr, STM32U5_RTC_TR);
  putreg32(dr, STM32U5_RTC_DR);

  /* Exit initialization mode */

  stm32_rtc_exit_init();

  /* Re-enable the write protection */

  stm32_rtc_writeprotect(true);

  /* Remember that the RTC has been initialized */

  g_rtc_dev.init        = true;
  g_rtc_dev.have_dtime  = true;

  rtcinfo("  tr: %08" PRIx32 " dr: %08" PRIx32 "\n", tr, dr);
  return OK;

errout_with_protect:
  stm32_rtc_writeprotect(true);
  return ret;
}
#endif

/****************************************************************************
 * Name: stm32u5_rtc_getdatetime_with_subseconds
 *
 * Description:
 *   Get the current date and time from the date/time RTC.  This interface
 *   is only supported by the date/time RTC hardware implementation.
 *   It is used to replace the system timer.  It is only used by the RTOS
 *   during initialization to set up the system time when CONFIG_RTC and
 *   CONFIG_RTC_DATETIME are selected (and CONFIG_RTC_HIRES is not).
 *
 *   NOTE: The sub-second accuracy is returned through 'nsec'.
 *
 * Input Parameters:
 *   tp - The location to return the high resolution time value.
 *   nsec - The location to return the subsecond time value.
 *
 * Returned Value:
 *   Zero (OK) on success; a negated errno on failure
 *
 ****************************************************************************/

#ifdef CONFIG_STM32U5_HAVE_RTC_SUBSECONDS
int stm32u5_rtc_getdatetime_with_subseconds(struct tm *tp, long *nsec)
{
  uint32_t ssr;
  uint32_t tr;
  uint32_t dr;
  uint32_t tmp;
  int ret;

  /* Sample the time and date registers to ensure that they are stable */

  ret = stm32_rtc_sync();
  if (ret < 0)
    {
      rtcerr("ERROR: RTC sync failed: %d\n", ret);
      return ret;
    }

  /* Get the current date and time */

  tr = getreg32(STM32U5_RTC_TR);
  dr = getreg32(STM32U5_RTC_DR);
  ssr = getreg32(STM32U5_RTC_SSR);

  /* Convert the RTC time and date structures to the struct tm format */

  tmp = (tr & RTC_TR_SU_MASK) >> RTC_TR_SU_SHIFT;
  tp->tm_sec = stm32_rtc_bcd2bin(tmp);

  tmp = (tr & RTC_TR_MNU_MASK) >> RTC_TR_MNU_SHIFT;
  tp->tm_min = stm32_rtc_bcd2bin(tmp);

  tmp = (tr & RTC_TR_HU_MASK) >> RTC_TR_HU_SHIFT;
  tp->tm_hour = stm32_rtc_bcd2bin(tmp);

  tmp = (dr & RTC_DR_DU_MASK) >> RTC_DR_DU_SHIFT;
  tp->tm_mday = stm32_rtc_bcd2bin(tmp);

  tmp = (dr & RTC_DR_MU_MASK) >> RTC_DR_MU_SHIFT;
  tp->tm_mon = stm32_rtc_bcd2bin(tmp) - 1;

  tmp = (dr & RTC_DR_YU_MASK) >> RTC_DR_YU_SHIFT;
  tp->tm_year = stm32_rtc_bcd2bin(tmp) + 100;

  /* Get sub-seconds in nanoseconds.
   * The range of the sub second register is [0, 2^15-1] where 2^15 = 32768
   */

  if (nsec)
    {
      *nsec = (((uint32_t)32767 - ssr) * NSEC_PER_SEC) / 32768;
    }

  rtcinfo("  tr: %08" PRIx32 " dr: %08" PRIx32 " ssr: %08" PRIx32
          " "
          "year: %d month: %d mday: %d "
          "hour: %d min: %d sec: %d\n",
          tr, dr, ssr,
          tp->tm_year + 1900, tp->tm_mon + 1, tp->tm_mday,
          tp->tm_hour, tp->tm_min, tp->tm_sec);

  return OK;
}
#endif

/****************************************************************************
 * Name: stm32_rtc_writeprotect
 *
 * Description:
 *   Enable or disable write protection to the RTC registers
 *
 ****************************************************************************/

void stm32_rtc_writeprotect(bool en)
{
  uint32_t regval;

  if (en)
    {
      /* Enable write protection */

      regval = getreg32(STM32U5_RTC_WPR);
      regval = (regval & ~RTC_WPR_KEY_MASK) | (0xff << RTC_WPR_KEY_SHIFT);
      putreg32(regval, STM32U5_RTC_WPR);
    }
  else
    {
      /* Disable write protection sequence:
       * Write 0xca to RTC_WPR
       * Write 0x53 to RTC_WPR
       */

      putreg32(0xca, STM32U5_RTC_WPR);
      putreg32(0x53, STM32U5_RTC_WPR);
    }
}

/****************************************************************************
 * Name: stm32_rtc_rinit
 *
 * Description:
 *   Initialize the hardware RTC per the selected configuration.  This
 *   function is called once during the OS initialization sequence.
 *
 ****************************************************************************/

int stm32_rtc_rinit(void)
{
  uint32_t regval;
  int ret;

  /* Enable the backup domain access */

  stm32_pwr_enablebkp(true);

  /* Check if the RTC has already been initialized */

  if (g_rtc_dev.init)
    {
      /* Already initialized... Just return */

      return OK;
    }

  /* Enable the LSE oscillator and select it as the RTC clock */

#if defined(CONFIG_STM32U5_RTC_HSECLOCK)
  /* Use HSE clock */

  /* Enable the external high-speed oscillator (HSE) */

  regval  = getreg32(STM32_RCC_CR);
  regval |= RCC_CR_HSEON;           /* Enable HSE */
  putreg32(regval, STM32_RCC_CR);

  /* Wait until the HSE is ready */

  while ((getreg32(STM32_RCC_CR) & RCC_CR_HSERDY) == 0)
    {
      /* Loop until HSE is ready */
    }

  /* Set the RTC clock source */

  regval = getreg32(STM32_RCC_BDCR);
  regval &= ~RCC_BDCR_RTCSEL_MASK;
  regval |= RCC_BDCR_RTCSEL_HSEDIV; /* Use HSE/128 */
  putreg32(regval, STM32_RCC_BDCR);

#elif defined(CONFIG_STM32U5_RTC_LSICLOCK)
  /* Use LSI clock */

  /* Enable the internal low-speed oscillator (LSI) */

  stm32_rcc_enablelsi();

  /* Set the RTC clock source */

  regval = getreg32(STM32_RCC_BDCR);
  regval &= ~RCC_BDCR_RTCSEL_MASK;
  regval |= RCC_BDCR_RTCSEL_LSI;
  putreg32(regval, STM32_RCC_BDCR);

#else
  /* Use LSE clock (most common) */

  /* Enable the external low-speed oscillator (LSE) */

  stm32_rcc_enablelse();

  /* Set the RTC clock source */

  regval = getreg32(STM32_RCC_BDCR);
  regval &= ~RCC_BDCR_RTCSEL_MASK;
  regval |= RCC_BDCR_RTCSEL_LSE;
  putreg32(regval, STM32_RCC_BDCR);

#endif

  /* Enable the RTC clock */

  regval = getreg32(STM32_RCC_BDCR);
  regval |= RCC_BDCR_RTCEN;
  putreg32(regval, STM32_RCC_BDCR);

  /* Disable the write protection before accessing the RTC registers */

  stm32_rtc_writeprotect(false);

  /* Enter initialization mode */

  ret = stm32_rtc_enter_init();
  if (ret < 0)
    {
      rtcerr("ERROR: Failed to enter init mode: %d\n", ret);
      goto errout_with_protect;
    }

  /* Set the prescaler for the RTC.
   * Use the most common 32.768 KHz LSE oscillator:
   * RTC period = RTCCLK / (PREDIV_A + 1) / (PREDIV_S + 1)
   * For RTCCLK = 32.768 KHz, PREDIV_A = 0x7f, PREDIV_S = 0x00ff
   * gives a RTC period of 1 second.
   */

  putreg32((RTC_PRER_PREDIV_A_MASK & (0x7f << RTC_PRER_PREDIV_A_SHIFT)) |
           (RTC_PRER_PREDIV_S_MASK & (0x00ff << RTC_PRER_PREDIV_S_SHIFT)),
           STM32U5_RTC_PRER);

  /* Exit initialization mode */

  stm32_rtc_exit_init();

  /* Enable the write protection */

  stm32_rtc_writeprotect(true);

  /* Remember that the RTC has been initialized */

  g_rtc_dev.init        = true;
  g_rtc_dev.have_dtime  = false;

  return OK;

errout_with_protect:
  stm32_rtc_writeprotect(true);
  return ret;
}

/****************************************************************************
 * Name: stm32u5_rtc_setperiodic
 *
 * Description:
 *   Set a periodic RTC wakeup
 *
 * Input Parameters:
 *  period   - Time to sleep between wakeups
 *  callback - Function to call when the period expires.
 *
 * Returned Value:
 *   Zero (OK) on success; a negated errno on failure
 *
 ****************************************************************************/

#ifdef CONFIG_RTC_PERIODIC
int stm32u5_rtc_setperiodic(const struct timespec *period,
                             wakeupcb_t callback)
{
  uint32_t wutr;
  uint32_t cr;
  int ret;

  /* Sample the time and date registers to ensure that they are stable */

  ret = stm32_rtc_sync();
  if (ret < 0)
    {
      rtcerr("ERROR: RTC sync failed: %d\n", ret);
      return ret;
    }

  /* Calculate the wakeup timer value. Since the period is given in
   * seconds, and we are using a 1-second prescaler, the WUTR value is
   * the same as the number of seconds.
   */

  if (period->tv_sec < 1 || period->tv_sec > 0xffff)
    {
      return -ERANGE;
    }

  wutr = period->tv_sec - 1;

  /* Disable the write protection before accessing the RTC registers */

  stm32_rtc_writeprotect(false);

  /* Enter initialization mode */

  ret = stm32_rtc_enter_init();
  if (ret < 0)
    {
      rtcerr("ERROR: Failed to enter init mode: %d\n", ret);
      goto errout_with_protect;
    }

  /* Set the wakeup timer register */

  putreg32(wutr, STM32U5_RTC_WUTR);

  /* Exit initialization mode */

  stm32_rtc_exit_init();

  /* Save the callback */

  g_rtc_dev.periodic_cb = callback;

  /* Configure the control register */

  cr = getreg32(STM32U5_RTC_CR);

  /* Clear the WUTIE and WUTE bits */

  cr &= ~(RTC_CR_WUTIE | RTC_CR_WUTE);

  /* Select the desired clock source (currently defaulting to CK_SPRE) */

  cr &= ~RTC_CR_WUCKSEL_MASK;
  cr |= RTC_CR_WUCKSEL_CK_SPRE_16BIT;

  /* Set the WUTE bit to enable the wakeup timer */

  cr |= RTC_CR_WUTE;

#ifdef CONFIG_RTC_PERIODIC_HIRES
  /* Enable the wakeup timer interrupt if a callback is provided */

  if (callback != NULL)
    {
      cr |= RTC_CR_WUTIE;
    }
#endif

  putreg32(cr, STM32U5_RTC_CR);

  /* Re-enable the write protection */

  stm32_rtc_writeprotect(true);

  return OK;

errout_with_protect:
  stm32_rtc_writeprotect(true);
  return ret;
}

/****************************************************************************
 * Name: stm32u5_rtc_cancelperiodic
 *
 * Description:
 *   Cancel a periodic wakeup
 *
 * Input Parameters:
 *
 * Returned Value:
 *   Zero (OK) on success; a negated errno on failure
 *
 ****************************************************************************/

int stm32u5_rtc_cancelperiodic(void)
{
  uint32_t cr;

  /* Disable the write protection before accessing the RTC registers */

  stm32_rtc_writeprotect(false);

  /* Clear the WUTIE and WUTE bits */

  cr = getreg32(STM32U5_RTC_CR);
  cr &= ~(RTC_CR_WUTIE | RTC_CR_WUTE);
  putreg32(cr, STM32U5_RTC_CR);

  /* Clear the callback */

  g_rtc_dev.periodic_cb = NULL;

  /* Re-enable the write protection */

  stm32_rtc_writeprotect(true);

  return OK;
}
#endif /* CONFIG_RTC_PERIODIC */

#ifdef CONFIG_RTC_ALARM

/****************************************************************************
 * Name: stm32u5_rtc_setalarm
 *
 * Description:
 *   Set an alarm to an absolute time using associated hardware.
 *
 * Input Parameters:
 *  alminfo - Information about the alarm configuration.
 *
 * Returned Value:
 *   Zero (OK) on success; a negated errno on failure
 *
 ****************************************************************************/

int stm32u5_rtc_setalarm(struct alm_setalarm_s *alminfo)
{
  uint32_t tr;
  uint32_t dr;
  uint32_t tmp;
  uint32_t cr;
  irqstate_t flags;

  flags = enter_critical_section();

  switch (alminfo->as_id)
    {
      case RTC_ALARMA:
        {
          /* Clear flag to indicate that the alarm is no longer pending */

          g_rtc_dev.alen[RTC_ALARMA] = false;

          /* Disable the write protection before accessing the RTC registers */

          stm32_rtc_writeprotect(false);

          /* Set the alarm interrupt callback and argument */

          g_rtc_dev.cb[RTC_ALARMA]   = alminfo->as_cb;
          g_rtc_dev.priv[RTC_ALARMA] = alminfo->as_arg;

          /* Calculate the alarm time register values */

          dr = (stm32_rtc_bin2bcd(alminfo->as_time.tm_year - 100) << RTC_DR_YU_SHIFT) |
               (stm32_rtc_bin2bcd(alminfo->as_time.tm_mon + 1)    << RTC_DR_MU_SHIFT) |
               (stm32_rtc_bin2bcd(alminfo->as_time.tm_mday)       << RTC_DR_DU_SHIFT);

          tr = (stm32_rtc_bin2bcd(alminfo->as_time.tm_sec)  << RTC_ALRMXR_SU_SHIFT) |
               (stm32_rtc_bin2bcd(alminfo->as_time.tm_min)  << RTC_ALRMXR_MNU_SHIFT) |
               (stm32_rtc_bin2bcd(alminfo->as_time.tm_hour) << RTC_ALRMXR_HU_SHIFT);

          /* Store the alarm registers */

          putreg32(dr, STM32U5_RTC_ALRMADR);
          putreg32(tr, STM32U5_RTC_ALRMATR);

          /* Clear the alarm interrupt enable and pending bits in the control and status registers */

          modifyreg32(STM32U5_RTC_CR, (RTC_CR_ALRAIE), 0);
          modifyreg32(STM32U5_RTC_ISR, RTC_ISR_ALRAF, 0);

          /* Set the alarm interrupt enable bit */

          modifyreg32(STM32U5_RTC_CR, 0, RTC_CR_ALRAIE);

          /* Enable the alarm */

          modifyreg32(STM32U5_RTC_CR, 0, RTC_CR_ALRAE);

          /* Re-enable the write protection */

          stm32_rtc_writeprotect(true);

          /* Remember that the alarm is set */

          g_rtc_dev.alen[RTC_ALARMA] = true;
          break;
        }

      case RTC_ALARMB:
        {
          /* Clear flag to indicate that the alarm is no longer pending */

          g_rtc_dev.alen[RTC_ALARMB] = false;

          /* Disable the write protection before accessing the RTC registers */

          stm32_rtc_writeprotect(false);

          /* Set the alarm interrupt callback and argument */

          g_rtc_dev.cb[RTC_ALARMB]   = alminfo->as_cb;
          g_rtc_dev.priv[RTC_ALARMB] = alminfo->as_arg;

          /* Calculate the alarm time register values */

          dr = (stm32_rtc_bin2bcd(alminfo->as_time.tm_year - 100) << RTC_DR_YU_SHIFT) |
               (stm32_rtc_bin2bcd(alminfo->as_time.tm_mon + 1)    << RTC_DR_MU_SHIFT) |
               (stm32_rtc_bin2bcd(alminfo->as_time.tm_mday)       << RTC_DR_DU_SHIFT);

          tr = (stm32_rtc_bin2bcd(alminfo->as_time.tm_sec)  << RTC_ALRMXR_SU_SHIFT) |
               (stm32_rtc_bin2bcd(alminfo->as_time.tm_min)  << RTC_ALRMXR_MNU_SHIFT) |
               (stm32_rtc_bin2bcd(alminfo->as_time.tm_hour) << RTC_ALRMXR_HU_SHIFT);

          /* Store the alarm registers */

          putreg32(dr, STM32U5_RTC_ALRMBDR);
          putreg32(tr, STM32U5_RTC_ALRMBTR);

          /* Clear the alarm interrupt enable and pending bits in the control and status registers */

          modifyreg32(STM32U5_RTC_CR, (RTC_CR_ALRBIE), 0);
          modifyreg32(STM32U5_RTC_ISR, RTC_ISR_ALRBF, 0);

          /* Set the alarm interrupt enable bit */

          modifyreg32(STM32U5_RTC_CR, 0, RTC_CR_ALRBIE);

          /* Enable the alarm */

          modifyreg32(STM32U5_RTC_CR, 0, RTC_CR_ALRBE);

          /* Re-enable the write protection */

          stm32_rtc_writeprotect(true);

          /* Remember that the alarm is set */

          g_rtc_dev.alen[RTC_ALARMB] = true;
          break;
        }

      default:
        {
          leave_critical_section(flags);
          return -EINVAL;
        }
    }

  leave_critical_section(flags);
  return OK;
}

/****************************************************************************
 * Name: stm32u5_rtc_rdalarm
 *
 * Description:
 *   Query an alarm configured in hardware.
 *
 * Input Parameters:
 *  alminfo - Information about the alarm configuration.
 *
 * Returned Value:
 *   Zero (OK) on success; a negated errno on failure
 *
 ****************************************************************************/

int stm32u5_rtc_rdalarm(struct alm_rdalarm_s *alminfo)
{
  uint32_t tr;
  uint32_t dr;
  uint32_t tmp;

  switch (alminfo->ar_id)
    {
      case RTC_ALARMA:
        {
          if (!g_rtc_dev.alen[RTC_ALARMA])
            {
              return -ENOENT;
            }

          /* Sample the alarm register values */

          tr = getreg32(STM32U5_RTC_ALRMATR);
          dr = getreg32(STM32U5_RTC_ALRMADR);

          /* Convert the RTC alarm time and date structures to the struct tm format */

          tmp = (tr & RTC_ALRMXR_SU_MASK) >> RTC_ALRMXR_SU_SHIFT;
          alminfo->ar_time->tm_sec = stm32_rtc_bcd2bin(tmp);

          tmp = (tr & RTC_ALRMXR_MNU_MASK) >> RTC_ALRMXR_MNU_SHIFT;
          alminfo->ar_time->tm_min = stm32_rtc_bcd2bin(tmp);

          tmp = (tr & RTC_ALRMXR_HU_MASK) >> RTC_ALRMXR_HU_SHIFT;
          alminfo->ar_time->tm_hour = stm32_rtc_bcd2bin(tmp);

          tmp = (dr & RTC_DR_DU_MASK) >> RTC_DR_DU_SHIFT;
          alminfo->ar_time->tm_mday = stm32_rtc_bcd2bin(tmp);

          tmp = (dr & RTC_DR_MU_MASK) >> RTC_DR_MU_SHIFT;
          alminfo->ar_time->tm_mon = stm32_rtc_bcd2bin(tmp) - 1;

          tmp = (dr & RTC_DR_YU_MASK) >> RTC_DR_YU_SHIFT;
          alminfo->ar_time->tm_year = stm32_rtc_bcd2bin(tmp) + 100;

          break;
        }

      case RTC_ALARMB:
        {
          if (!g_rtc_dev.alen[RTC_ALARMB])
            {
              return -ENOENT;
            }

          /* Sample the alarm register values */

          tr = getreg32(STM32U5_RTC_ALRMBTR);
          dr = getreg32(STM32U5_RTC_ALRMBDR);

          /* Convert the RTC alarm time and date structures to the struct tm format */

          tmp = (tr & RTC_ALRMXR_SU_MASK) >> RTC_ALRMXR_SU_SHIFT;
          alminfo->ar_time->tm_sec = stm32_rtc_bcd2bin(tmp);

          tmp = (tr & RTC_ALRMXR_MNU_MASK) >> RTC_ALRMXR_MNU_SHIFT;
          alminfo->ar_time->tm_min = stm32_rtc_bcd2bin(tmp);

          tmp = (tr & RTC_ALRMXR_HU_MASK) >> RTC_ALRMXR_HU_SHIFT;
          alminfo->ar_time->tm_hour = stm32_rtc_bcd2bin(tmp);

          tmp = (dr & RTC_DR_DU_MASK) >> RTC_DR_DU_SHIFT;
          alminfo->ar_time->tm_mday = stm32_rtc_bcd2bin(tmp);

          tmp = (dr & RTC_DR_MU_MASK) >> RTC_DR_MU_SHIFT;
          alminfo->ar_time->tm_mon = stm32_rtc_bcd2bin(tmp) - 1;

          tmp = (dr & RTC_DR_YU_MASK) >> RTC_DR_YU_SHIFT;
          alminfo->ar_time->tm_year = stm32_rtc_bcd2bin(tmp) + 100;

          break;
        }

      default:
        {
          return -EINVAL;
        }
    }

  return OK;
}

/****************************************************************************
 * Name: stm32u5_rtc_cancelalarm
 *
 * Description:
 *   Cancel an alarm.
 *
 * Input Parameters:
 *  alarmid - Identifies the alarm to be cancelled
 *
 * Returned Value:
 *   Zero (OK) on success; a negated errno on failure
 *
 ****************************************************************************/

int stm32u5_rtc_cancelalarm(enum alm_id_e alarmid)
{
  irqstate_t flags;

  flags = enter_critical_section();

  switch (alarmid)
    {
      case RTC_ALARMA:
        {
          /* Disable the write protection before accessing the RTC registers */

          stm32_rtc_writeprotect(false);

          /* Disable the alarm */

          modifyreg32(STM32U5_RTC_CR, RTC_CR_ALRAE | RTC_CR_ALRAIE, 0);

          /* Clear the alarm pending flag */

          modifyreg32(STM32U5_RTC_ISR, RTC_ISR_ALRAF, 0);

          /* Re-enable the write protection */

          stm32_rtc_writeprotect(true);

          /* Clear the alarm callback and clear the alarm pending flag */

          g_rtc_dev.cb[RTC_ALARMA]   = NULL;
          g_rtc_dev.priv[RTC_ALARMA] = NULL;
          g_rtc_dev.alen[RTC_ALARMA] = false;
          break;
        }

      case RTC_ALARMB:
        {
          /* Disable the write protection before accessing the RTC registers */

          stm32_rtc_writeprotect(false);

          /* Disable the alarm */

          modifyreg32(STM32U5_RTC_CR, RTC_CR_ALRBE | RTC_CR_ALRBIE, 0);

          /* Clear the alarm pending flag */

          modifyreg32(STM32U5_RTC_ISR, RTC_ISR_ALRBF, 0);

          /* Re-enable the write protection */

          stm32_rtc_writeprotect(true);

          /* Clear the alarm callback and clear the alarm pending flag */

          g_rtc_dev.cb[RTC_ALARMB]   = NULL;
          g_rtc_dev.priv[RTC_ALARMB] = NULL;
          g_rtc_dev.alen[RTC_ALARMB] = false;
          break;
        }

      default:
        {
          leave_critical_section(flags);
          return -EINVAL;
        }
    }

  leave_critical_section(flags);
  return OK;
}

/****************************************************************************
 * Name: stm32_rtc_alarm_handler
 *
 * Description:
 *   RTC interrupt handler
 *
 ****************************************************************************/

int stm32_rtc_alarm_handler(int irq, void *context, void *arg)
{
  uint32_t isr;

  /* Get the status of the interrupts */

  isr = getreg32(STM32U5_RTC_ISR);

  /* Check for Alarm A interrupt */

  if ((isr & RTC_ISR_ALRAF) && g_rtc_dev.cb[RTC_ALARMA])
    {
      /* Acknowledge the source of the alarm */

      putreg32(RTC_ISR_ALRAF, STM32U5_RTC_ISR);

      /* Disable the write protection before accessing the RTC registers */

      stm32_rtc_writeprotect(false);

      /* Disable the alarm */

      modifyreg32(STM32U5_RTC_CR, RTC_CR_ALRAE, 0);

      /* Re-enable the write protection */

      stm32_rtc_writeprotect(true);

      /* Clear the alarm pending flag */

      g_rtc_dev.alen[RTC_ALARMA] = false;

      /* Call the alarm callback */

      g_rtc_dev.cb[RTC_ALARMA](g_rtc_dev.priv[RTC_ALARMA], RTC_ALARMA);
    }

  /* Check for Alarm B interrupt */

  if ((isr & RTC_ISR_ALRBF) && g_rtc_dev.cb[RTC_ALARMB])
    {
      /* Acknowledge the source of the alarm */

      putreg32(RTC_ISR_ALRBF, STM32U5_RTC_ISR);

      /* Disable the write protection before accessing the RTC registers */

      stm32_rtc_writeprotect(false);

      /* Disable the alarm */

      modifyreg32(STM32U5_RTC_CR, RTC_CR_ALRBE, 0);

      /* Re-enable the write protection */

      stm32_rtc_writeprotect(true);

      /* Clear the alarm pending flag */

      g_rtc_dev.alen[RTC_ALARMB] = false;

      /* Call the alarm callback */

      g_rtc_dev.cb[RTC_ALARMB](g_rtc_dev.priv[RTC_ALARMB], RTC_ALARMB);
    }

  return OK;
}
#endif /* CONFIG_RTC_ALARM */

/****************************************************************************
 * Name: stm32u5_rtc_lowerhalf
 *
 * Description:
 *   Instantiate the RTC lower half driver for the STM32U5.
 *
 ****************************************************************************/

#ifdef CONFIG_RTC_DRIVER
struct rtc_lowerhalf_s *stm32u5_rtc_lowerhalf(void)
{
  /* Implementation would go here if needed */

  return NULL;
}
#endif

#endif /* CONFIG_RTC */