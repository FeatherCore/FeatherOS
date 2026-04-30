/****************************************************************************
 * arch/arm/src/ra8p/ra8p_peripherals.h
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

#ifndef __ARCH_ARM_SRC_RA8P_RA8P_PERIPHERALS_H
#define __ARCH_ARM_SRC_RA8P_RA8P_PERIPHERALS_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/* Include all RA8P peripheral driver headers based on Zephyr RA8P1 implementation */

#ifdef CONFIG_ARCH_CHIP_RA8P
#include "hardware/ra8p_memorymap.h"
#include "hardware/ra8p_irq.h"

/* Basic drivers */
#include "ra8p_clockconfig.h"
#include "ra8p_lowsetup.h"
#include "ra8p_gpio.h"

/* Advanced drivers */
#ifdef CONFIG_RA8P_SCI_B_UART
#include "ra8p_sci_b.h"
#endif

#ifdef CONFIG_RA8P_SPI_B
#include "ra8p_spi_b.h"
#endif

#ifdef CONFIG_RA8P_IIC
#include "ra8p_iic.h"
#endif

#ifdef CONFIG_RA8P_GPT_PWM
#include "ra8p_gpt.h"
#endif

#ifdef CONFIG_RA8P_ICU
#include "hardware/ra8p_icu.h"
#endif

#ifdef CONFIG_RA8P_RTC
#include "hardware/ra8p_rtc.h"
#endif

#ifdef CONFIG_RA8P_USBFS || CONFIG_RA8P_USBHS
#include "hardware/ra8p_usb.h"
#endif

#ifdef CONFIG_RA8P_WDT
#include "hardware/ra8p_wdt.h"
#endif

#ifdef CONFIG_RA8P_HAVE_CANFD
#include "hardware/ra8p_canfd.h"
#endif

#ifdef CONFIG_RA8P_HAVE_SDHC
#include "hardware/ra8p_sdhc.h"
#endif

#ifdef CONFIG_RA8P_POWER
#include "ra8p_power.h"
#endif

#ifdef CONFIG_RA8P_HAVE_DMA
#include "ra8p_dmac.h"
#endif

#ifdef CONFIG_RA8P_HAVE_PMSC
#include "ra8p_pinctrl.h"
#endif

#ifdef CONFIG_RA8P_I3C0
#include "hardware/ra8p_i3c.h"
#endif

#ifdef CONFIG_RA8P_OSPI0
#include "hardware/ra8p_ospi.h"
#endif

#ifdef CONFIG_RA8P_ETHERNET
#include "hardware/ra8p_eth.h"
#endif

#ifdef CONFIG_RA8P_ADC0
#include "hardware/ra8p_adc.h"
#endif

#ifdef CONFIG_RA8P_DAC0
#include "hardware/ra8p_dac.h"
#endif

#ifdef CONFIG_RA8P_MIPI_DSI
#include "hardware/ra8p_mipi_dsi.h"
#endif

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* RA8P1-specific constants */
#define RA8P_NUM_GPIO_PORTS         14      /* P0-PD */
#define RA8P_NUM_UART_CHANNELS      10      /* SCI0-SCI9 */
#define RA8P_NUM_SPI_CHANNELS       4       /* SPI0-SPI3 */
#define RA8P_NUM_IIC_CHANNELS       3       /* IIC0-IIC2 */
#define RA8P_NUM_GPT_CHANNELS       14      /* GPT0-GPT13 */
#define RA8P_NUM_IRQ_CHANNELS       32      /* External IRQ0-IRQ31 */
#define RA8P_NUM_CANFD_CHANNELS     2       /* CANFD0, CANFD1 */
#define RA8P_NUM_SDHC_CHANNELS      2       /* SDHC0, SDHC1 */

/* Clock sources */
#define RA8P_CLOCK_LOCO_FREQ        32768   /* LOCO: 32.768 kHz */
#define RA8P_CLOCK_MOCO_FREQ        8000000 /* MOCO: 8 MHz */
#define RA8P_CLOCK_HOCO_FREQ        48000000 /* HOCO: 48 MHz */
#define RA8P_CLOCK_XTAL_FREQ        24000000 /* XTAL: 24 MHz */

/****************************************************************************
 * Public Data
 ****************************************************************************/

/* External declarations for RA8P1-specific implementations */

extern const char *g_ra8p_chip_name;
extern const uint32_t g_ra8p_package_pins[];

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_initialize
 *
 * Description:
 *   Initialize all RA8P1 peripherals based on configuration options.
 *   This function calls all individual peripheral initialization functions
 *   based on the Kconfig options.
 *
 ****************************************************************************/

int ra8p_initialize(void);

/****************************************************************************
 * Name: ra8p_get_chip_info
 *
 * Description:
 *   Get RA8P1 chip information.
 *
 * Input Parameters:
 *   info - Pointer to structure to fill with chip information
 *
 ****************************************************************************/

struct ra8p_chip_info_s
{
  const char *name;           /* Chip name (e.g., "R7KA8P1KFLCAC") */
  uint8_t package_type;       /* Package type (1=AB, 2=AC, 3=AJ) */
  uint8_t num_cores;          /* Number of cores (1 or 2) */
  uint32_t mram_size;         /* MRAM size in bytes */
  uint32_t sram_size;         /* SRAM size in bytes */
  uint32_t feature_set;       /* Feature set flags */
};

int ra8p_get_chip_info(struct ra8p_chip_info_s *info);

/****************************************************************************
 * Name: ra8p_get_clock_freq
 *
 * Description:
 *   Get the frequency of a specific clock domain.
 *
 * Input Parameters:
 *   clock_id - Clock identifier (e.g., "iclk", "pclka", etc.)
 *
 * Return Value:
 *   Clock frequency in Hz, or 0 if invalid clock_id
 *
 ****************************************************************************/

uint32_t ra8p_get_clock_freq(uint8_t clock_id);

/****************************************************************************
 * Name: ra8p_reset_peripheral
 *
 * Description:
 *   Reset a specific peripheral module.
 *
 * Input Parameters:
 *   peripheral_id - Peripheral identifier
 *
 ****************************************************************************/

int ra8p_reset_peripheral(uint8_t peripheral_id);

/****************************************************************************
 * Name: ra8p_enable_peripheral
 *
 * Description:
 *   Enable clock for a specific peripheral module.
 *
 * Input Parameters:
 *   peripheral_id - Peripheral identifier
 *
 ****************************************************************************/

int ra8p_enable_peripheral(uint8_t peripheral_id);

/****************************************************************************
 * Name: ra8p_disable_peripheral
 *
 * Description:
 *   Disable clock for a specific peripheral module.
 *
 * Input Parameters:
 *   peripheral_id - Peripheral identifier
 *
 ****************************************************************************/

int ra8p_disable_peripheral(uint8_t peripheral_id);

#endif /* CONFIG_ARCH_CHIP_RA8P */

#endif /* __ARCH_ARM_SRC_RA8P_RA8P_PERIPHERALS_H */