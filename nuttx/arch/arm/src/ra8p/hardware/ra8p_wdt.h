/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_wdt.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_WDT_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_WDT_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* WDT Register Offsets - Based on RA8P Hardware Manual and Zephyr definitions */
#define RA8P_WDT_WDTRR_OFFSET            0x0000  /* Watchdog Refresh Register */
#define RA8P_WDT_WDTCR_OFFSET            0x0004  /* Watchdog Control Register */
#define RA8P_WDT_WDTRCR_OFFSET           0x0008  /* Watchdog Reset Control Register */
#define RA8P_WDT_WDTCSTPR_OFFSET         0x000C  /* Watchdog Count Stop Control Register */
#define RA8P_WDT_WDTOVF_OFFSET           0x0010  /* Watchdog Overflow Register */
#define RA8P_WDT_WDTOVFCLR_OFFSET        0x0010  /* Watchdog Overflow Clear Register */
#define RA8P_WDT_WDTSR_OFFSET            0x0014  /* Watchdog Status Register */
#define RA8P_WDT_WDTULOCK_OFFSET         0x0018  /* Watchdog Unlock Register */
#define RA8P_WDT_WDTOVFR_OFFSET          0x001C  /* Watchdog Overflow Flag Register */
#define RA8P_WDT_WDTOVFSTS_OFFSET        0x0020  /* Watchdog Overflow Status Register */
#define RA8P_WDT_WDTINT_OFFSET           0x0024  /* Watchdog Interrupt Register */
#define RA8P_WDT_WDTPR_OFFSET            0x0028  /* Watchdog Prescaler Register */
#define RA8P_WDT_WDTM_OFFSET             0x002C  /* Watchdog Mode Register */
#define RA8P_WDT_WDTC_OFFSET             0x0030  /* Watchdog Count Register */
#define RA8P_WDT_WDTCLK_OFFSET           0x0034  /* Watchdog Clock Control Register */

/* WDTCR - Watchdog Control Register */
#define RA8P_WDT_WDTCR_TME               (1 << 15)  /* Watchdog Module Enable */
#define RA8P_WDT_WDTCR_RPES_MASK         (0x0F << 11) /* Reset Pulse Extension Select Mask */
#define RA8P_WDT_WDTCR_RPES_SHIFT        11
#define RA8P_WDT_WDTCR_RPES_1CYCLE       (0x00 << 11) /* 1 PCLKB cycle */
#define RA8P_WDT_WDTCR_RPES_2CYCLES      (0x01 << 11) /* 2 PCLKB cycles */
#define RA8P_WDT_WDTCR_RPES_4CYCLES      (0x02 << 11) /* 4 PCLKB cycles */
#define RA8P_WDT_WDTCR_RPES_8CYCLES      (0x03 << 11) /* 8 PCLKB cycles */
#define RA8P_WDT_WDTCR_RPES_16CYCLES     (0x04 << 11) /* 16 PCLKB cycles */
#define RA8P_WDT_WDTCR_RPES_32CYCLES     (0x05 << 11) /* 32 PCLKB cycles */
#define RA8P_WDT_WDTCR_RPES_64CYCLES     (0x06 << 11) /* 64 PCLKB cycles */
#define RA8P_WDT_WDTCR_RPES_128CYCLES    (0x07 << 11) /* 128 PCLKB cycles */
#define RA8P_WDT_WDTCR_RPES_256CYCLES    (0x08 << 11) /* 256 PCLKB cycles */
#define RA8P_WDT_WDTCR_RPES_512CYCLES    (0x09 << 11) /* 512 PCLKB cycles */
#define RA8P_WDT_WDTCR_RPES_1024CYCLES   (0x0A << 11) /* 1024 PCLKB cycles */
#define RA8P_WDT_WDTCR_RPES_2048CYCLES   (0x0B << 11) /* 2048 PCLKB cycles */
#define RA8P_WDT_WDTCR_RPES_4096CYCLES   (0x0C << 11) /* 4096 PCLKB cycles */
#define RA8P_WDT_WDTCR_RPES_8192CYCLES   (0x0D << 11) /* 8192 PCLKB cycles */
#define RA8P_WDT_WDTCR_RPES_16384CYCLES  (0x0E << 11) /* 16384 PCLKB cycles */
#define RA8P_WDT_WDTCR_RPES_32768CYCLES  (0x0F << 11) /* 32768 PCLKB cycles */
#define RA8P_WDT_WDTCR_TOPS_MASK         (0x03 << 13) /* Timeout Period Select Mask */
#define RA8P_WDT_WDTCR_TOPS_SHIFT        13
#define RA8P_WDT_WDTCR_TOPS_128          (0x00 << 13) /* 128 cycles */
#define RA8P_WDT_WDTCR_TOPS_512          (0x01 << 13) /* 512 cycles */
#define RA8P_WDT_WDTCR_TOPS_1024         (0x02 << 13) /* 1024 cycles */
#define RA8P_WDT_WDTCR_TOPS_2048         (0x03 << 13) /* 2048 cycles */
#define RA8P_WDT_WDTCR_CKS_MASK          (0x0F << 8)  /* Clock Select Mask */
#define RA8P_WDT_WDTCR_CKS_SHIFT         8
#define RA8P_WDT_WDTCR_CKS_DIV1          (0x00 << 8)  /* PCLKB / 1 */
#define RA8P_WDT_WDTCR_CKS_DIV2          (0x01 << 8)  /* PCLKB / 2 */
#define RA8P_WDT_WDTCR_CKS_DIV4          (0x02 << 8)  /* PCLKB / 4 */
#define RA8P_WDT_WDTCR_CKS_DIV8          (0x03 << 8)  /* PCLKB / 8 */
#define RA8P_WDT_WDTCR_CKS_DIV16         (0x04 << 8)  /* PCLKB / 16 */
#define RA8P_WDT_WDTCR_CKS_DIV32         (0x05 << 8)  /* PCLKB / 32 */
#define RA8P_WDT_WDTCR_CKS_DIV64         (0x06 << 8)  /* PCLKB / 64 */
#define RA8P_WDT_WDTCR_CKS_DIV128        (0x07 << 8)  /* PCLKB / 128 */
#define RA8P_WDT_WDTCR_CKS_DIV256        (0x08 << 8)  /* PCLKB / 256 */
#define RA8P_WDT_WDTCR_CKS_DIV512        (0x09 << 8)  /* PCLKB / 512 */
#define RA8P_WDT_WDTCR_CKS_DIV1024       (0x0A << 8)  /* PCLKB / 1024 */
#define RA8P_WDT_WDTCR_CKS_DIV2048       (0x0B << 8)  /* PCLKB / 2048 */
#define RA8P_WDT_WDTCR_CKS_DIV4096       (0x0C << 8)  /* PCLKB / 4096 */
#define RA8P_WDT_WDTCR_CKS_DIV8192       (0x0D << 8)  /* PCLKB / 8192 */
#define RA8P_WDT_WDTCR_CKS_DIV16384      (0x0E << 8)  /* PCLKB / 16384 */
#define RA8P_WDT_WDTCR_CKS_DIV32768      (0x0F << 8)  /* PCLKB / 32768 */

/* WDTRCR - Watchdog Reset Control Register */
#define RA8P_WDT_WDTRCR_RSTIRQS_MASK     (0x03 << 0)  /* Reset Interrupt Request Select Mask */
#define RA8P_WDT_WDTRCR_RSTIRQS_SHIFT    0
#define RA8P_WDT_WDTRCR_RSTIRQS_DIS      (0x00 << 0)  /* Disable */
#define RA8P_WDT_WDTRCR_RSTIRQS_RST      (0x01 << 0)  /* Generate reset */
#define RA8P_WDT_WDTRCR_RSTIRQS_IRQ      (0x02 << 0)  /* Generate interrupt */
#define RA8P_WDT_WDTRCR_WDTRF            (1 << 7)   /* WDT Reset Flag */

/* WDTULOCK - Watchdog Unlock Register */
#define RA8P_WDT_WDTULOCK_UNLOCK_CODE    0xA5

/* WDTOVFCLR - Watchdog Overflow Clear Register */
#define RA8P_WDT_WDTOVFCLR_WDTOVFCLR     0xA5

/* WDT Base Address */
#define RA8P_WDT_BASE                    0x40202600
#define RA8P_WDT_SIZE                    0x0020

/* WDT interrupt numbers */
#define RA8P_IRQ_WDT                     77

/* WDT timeout values in milliseconds */
#define RA8P_WDT_TIMEOUT_MIN_MS          1        /* Minimum timeout in ms */
#define RA8P_WDT_TIMEOUT_MAX_MS          32768    /* Maximum timeout in ms */

/* Maximum clock divisor values */
#define RA8P_WDT_MAX_CLK_DIVIDER         32768    /* Maximum clock divider */

/* WDT timeout periods */
#define RA8P_WDT_TIMEOUT_128CYCLES       128      /* 128 cycles */
#define RA8P_WDT_TIMEOUT_512CYCLES       512      /* 512 cycles */
#define RA8P_WDT_TIMEOUT_1024CYCLES      1024     /* 1024 cycles */
#define RA8P_WDT_TIMEOUT_2048CYCLES      2048     /* 2048 cycles */

/* WDT refresh sequence values */
#define RA8P_WDT_REFRESH_SEQ1            0xAC
#define RA8P_WDT_REFRESH_SEQ2            0x53

/* Default timeout setting */
#define RA8P_WDT_DEFAULT_TIMEOUT_MS      1000     /* 1 second default */

/* WDT interrupt enable flags */
#define RA8P_WDT_INT_OVERFLOW            (1 << 0)   /* Overflow interrupt */

/* WDT status register bits */
#define RA8P_WDT_WDTSR_CSTS              (1 << 0)   /* Carry Status */
#define RA8P_WDT_WDTSR_ASTS              (1 << 1)   /* Alarm Status */
#define RA8P_WDT_WDTSR_PSTS              (1 << 2)   /* Periodic Status */
#define RA8P_WDT_WDTSR_ADJSTS            (1 << 3)   /* Adjustment Status */
#define RA8P_WDT_WDTSR_OVSTS             (1 << 4)   /* Overflow Status */
#define RA8P_WDT_WDTSR_WDTRF             (1 << 7)   /* WDT Reset Flag */

/* WDT control register bits */
#define RA8P_WDT_WDTCR_RPTEN             (1 << 15)  /* Reset Enable */

/* Maximum timeout periods */
#define RA8P_WDT_MAX_TIMEOUT_PERIOD      15

/* Clock sources for WDT */
#define RA8P_WDT_CLK_SRC_SUBCLK          0
#define RA8P_WDT_CLK_SRC_LOCO            1
#define RA8P_WDT_CLK_SRC_HOCO            2

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* WDT configuration structure */
struct ra8p_wdt_config_s
{
  uint32_t timeout_ms;                /* Timeout period in milliseconds */
  bool reset_enabled;                 /* Enable reset on timeout */
  bool interrupt_only;                /* Interrupt only mode (no reset) */
  uint8_t clock_divider;              /* Clock divider */
  uint8_t timeout_setting;            /* Timeout setting */
  bool auto_refresh;                  /* Automatic refresh */
  bool clock_select_subclk;           /* Use sub-clock instead of PCLKB */
  uint8_t prescaler;                  /* Prescaler value */
  bool low_power_mode;                /* Enable low power mode */
};

/* WDT timeout periods */
enum ra8p_wdt_timeout_period_e
{
  RA8P_WDT_TIMEOUT_PERIOD_128,
  RA8P_WDT_TIMEOUT_PERIOD_512,
  RA8P_WDT_TIMEOUT_PERIOD_1024,
  RA8P_WDT_TIMEOUT_PERIOD_2048,
  RA8P_WDT_TIMEOUT_PERIOD_4096,
  RA8P_WDT_TIMEOUT_PERIOD_8192,
  RA8P_WDT_TIMEOUT_PERIOD_16384,
  RA8P_WDT_TIMEOUT_PERIOD_32768
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_wdt_initialize
 *
 * Description:
 *   Initialize the WDT based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   config - Pointer to WDT configuration structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_initialize(const struct ra8p_wdt_config_s *config);

/****************************************************************************
 * Name: ra8p_wdt_enable
 *
 * Description:
 *   Enable the WDT based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_enable(void);

/****************************************************************************
 * Name: ra8p_wdt_disable
 *
 * Description:
 *   Disable the WDT based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_disable(void);

/****************************************************************************
 * Name: ra8p_wdt_refresh
 *
 * Description:
 *   Refresh the WDT based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_refresh(void);

/****************************************************************************
 * Name: ra8p_wdt_set_timeout
 *
 * Description:
 *   Set WDT timeout period based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   timeout_ms - Timeout period in milliseconds
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_set_timeout(uint32_t timeout_ms);

/****************************************************************************
 * Name: ra8p_wdt_is_enabled
 *
 * Description:
 *   Check if WDT is enabled based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_wdt_is_enabled(void);

/****************************************************************************
 * Name: ra8p_wdt_is_overflow_occurred
 *
 * Description:
 *   Check if WDT overflow has occurred based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if overflow occurred, false otherwise
 *
 ****************************************************************************/

bool ra8p_wdt_is_overflow_occurred(void);

/****************************************************************************
 * Name: ra8p_wdt_clear_overflow_flag
 *
 * Description:
 *   Clear WDT overflow flag based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_clear_overflow_flag(void);

/****************************************************************************
 * Name: ra8p_wdt_get_timeout
 *
 * Description:
 *   Get current WDT timeout period based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Current timeout period in milliseconds
 *
 ****************************************************************************/

uint32_t ra8p_wdt_get_timeout(void);

/****************************************************************************
 * Name: ra8p_wdt_set_reset_mode
 *
 * Description:
 *   Configure WDT reset mode (reset vs interrupt) based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   reset - true to enable reset on timeout, false for interrupt only
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_set_reset_mode(bool reset);

/****************************************************************************
 * Name: ra8p_wdt_is_reset_enabled
 *
 * Description:
 *   Check if WDT reset on timeout is enabled based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if reset enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_wdt_is_reset_enabled(void);

/****************************************************************************
 * Name: ra8p_wdt_set_clock_divider
 *
 * Description:
 *   Set WDT clock divider based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   divider - Clock divider (0-15)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_set_clock_divider(uint8_t divider);

/****************************************************************************
 * Name: ra8p_wdt_get_clock_divider
 *
 * Description:
 *   Get current WDT clock divider based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Current clock divider
 *
 ****************************************************************************/

uint8_t ra8p_wdt_get_clock_divider(void);

/****************************************************************************
 * Name: ra8p_wdt_feed
 *
 * Description:
 *   Alias for wdt_refresh based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_feed(void);

/****************************************************************************
 * Name: ra8p_wdt_start
 *
 * Description:
 *   Start WDT operation based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_start(void);

/****************************************************************************
 * Name: ra8p_wdt_stop
 *
 * Description:
 *   Stop WDT operation based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_stop(void);

/****************************************************************************
 * Name: ra8p_wdt_reset_counter
 *
 * Description:
 *   Reset the watchdog counter based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_reset_counter(void);

/****************************************************************************
 * Name: ra8p_wdt_get_status
 *
 * Description:
 *   Get WDT status flags based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Status flags
 *
 ****************************************************************************/

uint32_t ra8p_wdt_get_status(void);

/****************************************************************************
 * Name: ra8p_wdt_enable_interrupt
 *
 * Description:
 *   Enable WDT interrupts based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   enable - true to enable interrupt, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_enable_interrupt(bool enable);

/****************************************************************************
 * Name: ra8p_wdt_calculate_timeout
 *
 * Description:
 *   Calculate WDT timeout parameters from desired timeout in milliseconds.
 *
 * Input Parameters:
 *   timeout_ms - Desired timeout in milliseconds
 *   divider - Pointer to store calculated clock divider
 *   setting - Pointer to store calculated timeout setting
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_calculate_timeout(uint32_t timeout_ms, uint8_t *divider, uint8_t *setting);

/****************************************************************************
 * Name: ra8p_wdt_unlock_registers
 *
 * Description:
 *   Unlock WDT registers for write access based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_unlock_registers(void);

/****************************************************************************
 * Name: ra8p_wdt_get_error_flags
 *
 * Description:
 *   Get WDT error flags based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Error flags
 *
 ****************************************************************************/

uint32_t ra8p_wdt_get_error_flags(void);

/****************************************************************************
 * Name: ra8p_wdt_clear_errors
 *
 * Description:
 *   Clear WDT error flags based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   flags - Error flags to clear
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_clear_errors(uint32_t flags);

/****************************************************************************
 * Name: ra8p_wdt_get_current_count
 *
 * Description:
 *   Get current WDT counter value based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Current counter value
 *
 ****************************************************************************/

uint32_t ra8p_wdt_get_current_count(void);

/****************************************************************************
 * Name: ra8p_wdt_set_timeout_period
 *
 * Description:
 *   Set WDT timeout period selection based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   period - Timeout period code (0-3)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_set_timeout_period(uint8_t period);

/****************************************************************************
 * Name: ra8p_wdt_get_overflow_count
 *
 * Description:
 *   Get the number of WDT overflow events based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Overflow count
 *
 ****************************************************************************/

uint32_t ra8p_wdt_get_overflow_count(void);

/****************************************************************************
 * Name: ra8p_wdt_set_clock_source
 *
 * Description:
 *   Set WDT clock source (SUBCLK vs PCLKB) based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   subclk - true to use sub-clock, false to use PCLKB
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_set_clock_source(bool subclk);

/****************************************************************************
 * Name: ra8p_wdt_get_timeout_ms
 *
 * Description:
 *   Get calculated timeout in milliseconds based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   divider - Clock divider
 *   setting - Timeout setting
 *
 * Returned Value:
 *   Calculated timeout in milliseconds
 *
 ****************************************************************************/

uint32_t ra8p_wdt_get_timeout_ms(uint8_t divider, uint8_t setting);

/****************************************************************************
 * Name: ra8p_wdt_enable_low_power_mode
 *
 * Description:
 *   Enable/disable WDT low power mode based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   enable - true to enable low power mode, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_enable_low_power_mode(bool enable);

/****************************************************************************
 * Name: ra8p_wdt_is_low_power_mode
 *
 * Description:
 *   Check if WDT is in low power mode based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if in low power mode, false otherwise
 *
 ****************************************************************************/

bool ra8p_wdt_is_low_power_mode(void);

/****************************************************************************
 * Name: ra8p_wdt_set_prescaler
 *
 * Description:
 *   Set WDT prescaler value based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   prescaler - Prescaler value
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_set_prescaler(uint8_t prescaler);

/****************************************************************************
 * Name: ra8p_wdt_get_prescaler
 *
 * Description:
 *   Get WDT prescaler value based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   Prescaler value
 *
 ****************************************************************************/

uint8_t ra8p_wdt_get_prescaler(void);

/****************************************************************************
 * Name: ra8p_wdt_is_stopped
 *
 * Description:
 *   Check if WDT is stopped based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if stopped, false otherwise
 *
 ****************************************************************************/

bool ra8p_wdt_is_stopped(void);

/****************************************************************************
 * Name: ra8p_wdt_get_reset_flag
 *
 * Description:
 *   Get WDT reset flag based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if reset occurred due to WDT, false otherwise
 *
 ****************************************************************************/

bool ra8p_wdt_get_reset_flag(void);

/****************************************************************************
 * Name: ra8p_wdt_clear_reset_flag
 *
 * Description:
 *   Clear WDT reset flag based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_clear_reset_flag(void);

/****************************************************************************
 * Name: ra8p_wdt_set_interrupt_only
 *
 * Description:
 *   Set WDT to interrupt-only mode (no reset) based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   enable - true to enable interrupt-only mode, false for reset mode
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_wdt_set_interrupt_only(bool enable);

/****************************************************************************
 * Name: ra8p_wdt_is_interrupt_only
 *
 * Description:
 *   Check if WDT is in interrupt-only mode based on Nuttx WDT driver implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if interrupt-only mode, false otherwise
 *
 ****************************************************************************/

bool ra8p_wdt_is_interrupt_only(void);

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_WDT_H */