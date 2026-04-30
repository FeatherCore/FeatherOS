/****************************************************************************
 * arch/arm/src/ra8p/ra8p_sysinit.c
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
#include <nuttx/init.h>
#include <arch/board/board.h>

#include "ra8p_config.h"
#include "ra8p_peripherals.h"

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_chip_init
 *
 * Description:
 *   Initialize all RA8P1 specific peripherals based on configuration options
 *
 ****************************************************************************/

int ra8p_chip_init(void)
{
  int ret = OK;

#ifdef CONFIG_RA8P_SCI_B_UART
  /* Initialize UART early for debug/console output */
  ret = ra8p_sci_b_initialize();
  if (ret < 0)
    {
      return ret;
    }
#endif

#ifdef CONFIG_RA8P_GPIO
  /* Initialize GPIO */
  ret = ra8p_gpio_initialize();
  if (ret < 0)
    {
      return ret;
    }
#endif

#ifdef CONFIG_RA8P_SPI_B
  /* Initialize SPI */
  ret = ra8p_spi_b_initialize();
  if (ret < 0)
    {
      return ret;
    }
#endif

#ifdef CONFIG_RA8P_IIC
  /* Initialize I2C */
  ret = ra8p_iic_initialize();
  if (ret < 0)
    {
      return ret;
    }
#endif

#ifdef CONFIG_RA8P_GPT_PWM
  /* Initialize GPT (PWM) */
  ret = ra8p_gpt_initialize();
  if (ret < 0)
    {
      return ret;
    }
#endif

#ifdef CONFIG_RA8P_ICU
  /* Initialize ICU (External Interrupts) */
  ret = ra8p_icu_initialize();
  if (ret < 0)
    {
      return ret;
    }
#endif

#ifdef CONFIG_RA8P_DMA
  /* Initialize DMA */
  ret = ra8p_dmac_initialize();
  if (ret < 0)
    {
      return ret;
    }
#endif

#ifdef CONFIG_RA8P_ADC0
  /* Initialize ADC */
  ret = ra8p_adc_initialize();
  if (ret < 0)
    {
      return ret;
    }
#endif

#ifdef CONFIG_RA8P_DAC0
  /* Initialize DAC */
  ret = ra8p_dac_initialize();
  if (ret < 0)
    {
      return ret;
    }
#endif

#ifdef CONFIG_RA8P_CRC
  /* Initialize CRC */
  ret = ra8p_crc_initialize();
  if (ret < 0)
    {
      return ret;
    }
#endif

#ifdef CONFIG_RA8P_ENTROPY
  /* Initialize Entropy (TRNG) */
  ret = ra8p_entropy_initialize();
  if (ret < 0)
    {
      return ret;
    }
#endif

#ifdef CONFIG_RA8P_I2S0
  /* Initialize I2S */
  ret = ra8p_i2s_initialize();
  if (ret < 0)
    {
      return ret;
    }
#endif

#ifdef CONFIG_RA8P_COMPARATOR
  /* Initialize Comparator */
  ret = ra8p_comparator_initialize();
  if (ret < 0)
    {
      return ret;
    }
#endif

#ifdef CONFIG_RA8P_GLCDC
  /* Initialize GLCDC */
  ret = ra8p_glcdc_initialize();
  if (ret < 0)
    {
      return ret;
    }
#endif

#ifdef CONFIG_RA8P_MIPI_DSI
  /* Initialize MIPI DSI */
  ret = ra8p_dsi_initialize();
  if (ret < 0)
    {
      return ret;
    }
#endif

#ifdef CONFIG_RA8P_OSPI0
  /* Initialize OSPI */
  ret = ra8p_ospi_initialize();
  if (ret < 0)
    {
      return ret;
    }
#endif

#ifdef CONFIG_RA8P_ETHERNET
  /* Initialize Ethernet */
  ret = ra8p_ethernet_initialize();
  if (ret < 0)
    {
      return ret;
    }
#endif

#ifdef CONFIG_RA8P_I3C0
  /* Initialize I3C */
  ret = ra8p_i3c_initialize();
  if (ret < 0)
    {
      return ret;
    }
#endif

#ifdef CONFIG_RA8P_RTC
  /* Initialize RTC */
  ret = ra8p_rtc_initialize();
  if (ret < 0)
    {
      return ret;
    }
#endif

#ifdef CONFIG_RA8P_WDT
  /* Initialize WDT */
  ret = ra8p_wdt_initialize();
  if (ret < 0)
    {
      return ret;
    }
#endif

#ifdef CONFIG_RA8P_HWINFO
  /* Initialize Hardware Info */
  ra8p_hwinfo_get_unique_id(NULL, 0); /* Just to trigger initialization */
#endif

  return ret;
}

/****************************************************************************
 * Name: ra8p_board_init
 *
 * Description:
 *   Board-specific initialization (if needed)
 *
 ****************************************************************************/

int ra8p_board_init(void)
{
  /* Board-specific initialization goes here */

  return OK;
}

/****************************************************************************
 * Name: ra8p_system_init
 *
 * Description:
 *   Initialize the entire RA8P1 system
 *
 ****************************************************************************/

int ra8p_system_init(void)
{
  int ret;

  /* Initialize RA8P1 specific peripherals */
  ret = ra8p_chip_init();
  if (ret < 0)
    {
      return ret;
    }

  /* Board-specific initialization */
  ret = ra8p_board_init();
  if (ret < 0)
    {
      return ret;
    }

  return OK;
}