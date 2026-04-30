/****************************************************************************
 * boards/arm/stm32n6/stm32n6570-dk/src/stm32_bringup.c
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with this
 * work for additional information regarding copyright ownership.  The ASF
 * licenses this file to you under the Apache License, Version 2.0 (the
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

#include <errno.h>
#include <syslog.h>

#include <arch/board/board.h>

#include "hardware/stm32n6_memorymap.h"
#include "stm32n6_adc.h"
#include "stm32n6_can.h"
#include "stm32n6_eth.h"
#include "stm32n6_gpio.h"
#include "stm32n6_i2c.h"
#include "stm32n6_ltdc.h"
#include "stm32n6_pwm.h"
#include "stm32n6_sdmmc.h"
#include "stm32n6_spi.h"
#include "stm32n6_xspi.h"

#include "stm32n6570-dk.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#if defined(CONFIG_STM32N6570_DK_I2C1) || \
    defined(CONFIG_STM32N6570_DK_I2C2) || \
    defined(CONFIG_STM32N6570_DK_I2C4) || \
    defined(CONFIG_STM32N6570_DK_GT911) || \
    defined(CONFIG_STM32N6570_DK_SPI5) || \
    defined(CONFIG_STM32N6570_DK_FDCAN1) || \
    defined(CONFIG_STM32N6570_DK_SDMMC2) || \
    defined(CONFIG_STM32N6570_DK_ADC1) || \
    defined(CONFIG_STM32N6570_DK_TIM1_PWM) || \
    defined(CONFIG_STM32N6570_DK_TIM15_PWM) || \
    defined(CONFIG_STM32N6570_DK_ETH) || \
    defined(CONFIG_STM32N6570_DK_LTDC) || \
    defined(CONFIG_STM32N6570_DK_XSPI1_PSRAM) || \
    defined(CONFIG_STM32N6570_DK_XSPI2_FLASH)
#  define HAVE_STM32N6570_DK_BRINGUP 1
#endif

/****************************************************************************
 * Private Functions
 ****************************************************************************/

#ifdef HAVE_STM32N6570_DK_BRINGUP
static void stm32_bringup_report(FAR const char *name, int ret)
{
  if (ret < 0)
    {
      syslog(LOG_WARNING, "WARNING: %s bring-up deferred: %d\n", name, ret);
    }
}
#endif

/****************************************************************************
 * Name: stm32_i2c_setup
 ****************************************************************************/

#if defined(CONFIG_STM32N6570_DK_I2C1) || \
    defined(CONFIG_STM32N6570_DK_I2C2) || \
    defined(CONFIG_STM32N6570_DK_I2C4)
static int stm32_i2c_setup(int bus)
{
  FAR struct i2c_master_s *i2c;

  if (bus == 1)
    {
      stm32n6_configgpio(GPIO_I2C1_SCL);
      stm32n6_configgpio(GPIO_I2C1_SDA);
    }
  else if (bus == 2)
    {
      stm32n6_configgpio(GPIO_I2C2_SCL);
      stm32n6_configgpio(GPIO_I2C2_SDA);
    }
  else if (bus == 4)
    {
      stm32n6_configgpio(GPIO_I2C4_SCL);
      stm32n6_configgpio(GPIO_I2C4_SDA);
    }
  else
    {
      return -EINVAL;
    }

  i2c = stm32n6_i2cbus_initialize(bus);
  return i2c == NULL ? -ENOSYS : OK;
}
#endif

/****************************************************************************
 * Name: stm32_spi5_setup
 ****************************************************************************/

#ifdef CONFIG_STM32N6570_DK_SPI5
static int stm32_spi5_setup(void)
{
  FAR struct spi_dev_s *spi;

  stm32n6_configgpio(GPIO_SPI5_NSS);
  stm32n6_configgpio(GPIO_SPI5_SCK);
  stm32n6_configgpio(GPIO_SPI5_MISO);
  stm32n6_configgpio(GPIO_SPI5_MOSI);

  spi = stm32n6_spibus_initialize(5);
  return spi == NULL ? -ENOSYS : OK;
}
#endif

/****************************************************************************
 * Name: stm32_fdcan1_setup
 ****************************************************************************/

#ifdef CONFIG_STM32N6570_DK_FDCAN1
static int stm32_fdcan1_setup(void)
{
  FAR struct can_dev_s *can;

  stm32n6_configgpio(GPIO_FDCAN1_RX);
  stm32n6_configgpio(GPIO_FDCAN1_TX);

  can = stm32n6_caninitialize(1);
  return can == NULL ? -ENOSYS : OK;
}
#endif

/****************************************************************************
 * Name: stm32_sdmmc2_setup
 ****************************************************************************/

#ifdef CONFIG_STM32N6570_DK_SDMMC2
static int stm32_sdmmc2_setup(void)
{
  FAR struct sdio_dev_s *sdio;

  stm32n6_configgpio(GPIO_SDMMC2_D0);
  stm32n6_configgpio(GPIO_SDMMC2_D1);
  stm32n6_configgpio(GPIO_SDMMC2_D2);
  stm32n6_configgpio(GPIO_SDMMC2_D3);
  stm32n6_configgpio(GPIO_SDMMC2_CK);
  stm32n6_configgpio(GPIO_SDMMC2_CMD);
  stm32n6_configgpio(GPIO_SDMMC2_CD);
  stm32n6_configgpio(GPIO_SDMMC2_PWR);

  sdio = sdio_initialize(0);
  return sdio == NULL ? -ENOSYS : OK;
}
#endif

/****************************************************************************
 * Name: stm32_adc1_setup
 ****************************************************************************/

#ifdef CONFIG_STM32N6570_DK_ADC1
static int stm32_adc1_setup(void)
{
  static const uint8_t chanlist[] =
  {
    10, 11
  };

  FAR struct adc_dev_s *adc;

  stm32n6_configgpio(GPIO_ADC1_INP10);
  stm32n6_configgpio(GPIO_ADC1_INP11);

  adc = stm32n6_adcinitialize(1, chanlist,
                              sizeof(chanlist) / sizeof(chanlist[0]));
  return adc == NULL ? -ENOSYS : OK;
}
#endif

/****************************************************************************
 * Name: stm32_pwm_setup
 ****************************************************************************/

#if defined(CONFIG_STM32N6570_DK_TIM1_PWM) || \
    defined(CONFIG_STM32N6570_DK_TIM15_PWM)
static int stm32_pwm_setup(int timer)
{
  FAR struct pwm_lowerhalf_s *pwm;

  if (timer == 1)
    {
      stm32n6_configgpio(GPIO_TIM1_CH1);
    }
  else if (timer == 15)
    {
      stm32n6_configgpio(GPIO_TIM15_CH1);
    }
  else
    {
      return -EINVAL;
    }

  pwm = stm32n6_pwminitialize(timer);
  return pwm == NULL ? -ENOSYS : OK;
}
#endif

/****************************************************************************
 * Name: stm32_eth_setup
 ****************************************************************************/

#ifdef CONFIG_STM32N6570_DK_ETH
static int stm32_eth_setup(void)
{
  stm32n6_configgpio(GPIO_ETH_GTX_CLK);
  stm32n6_configgpio(GPIO_ETH_CLK125);
  stm32n6_configgpio(GPIO_ETH_RX_CLK);
  stm32n6_configgpio(GPIO_ETH_RXD2);
  stm32n6_configgpio(GPIO_ETH_RXD3);
  stm32n6_configgpio(GPIO_ETH_RX_CTL);
  stm32n6_configgpio(GPIO_ETH_TX_CTL);
  stm32n6_configgpio(GPIO_ETH_TXD1);
  stm32n6_configgpio(GPIO_ETH_TXD0);
  stm32n6_configgpio(GPIO_ETH_RXD0);
  stm32n6_configgpio(GPIO_ETH_RXD1);
  stm32n6_configgpio(GPIO_ETH_TXD2);
  stm32n6_configgpio(GPIO_ETH_TXD3);
  stm32n6_configgpio(GPIO_ETH_MDIO);
  stm32n6_configgpio(GPIO_ETH_MDC);
  stm32n6_configgpio(GPIO_ETH_PHY_INT);

  return stm32n6_ethinitialize(0);
}
#endif

/****************************************************************************
 * Name: stm32_ltdc_setup
 ****************************************************************************/

#ifdef CONFIG_STM32N6570_DK_LTDC
static int stm32_ltdc_setup(void)
{
  stm32n6_configgpio(GPIO_LTDC_R0);
  stm32n6_configgpio(GPIO_LTDC_R1);
  stm32n6_configgpio(GPIO_LTDC_R2);
  stm32n6_configgpio(GPIO_LTDC_R3);
  stm32n6_configgpio(GPIO_LTDC_R4);
  stm32n6_configgpio(GPIO_LTDC_R5);
  stm32n6_configgpio(GPIO_LTDC_R6);
  stm32n6_configgpio(GPIO_LTDC_R7);
  stm32n6_configgpio(GPIO_LTDC_G0);
  stm32n6_configgpio(GPIO_LTDC_G1);
  stm32n6_configgpio(GPIO_LTDC_G2);
  stm32n6_configgpio(GPIO_LTDC_G3);
  stm32n6_configgpio(GPIO_LTDC_G4);
  stm32n6_configgpio(GPIO_LTDC_G5);
  stm32n6_configgpio(GPIO_LTDC_G6);
  stm32n6_configgpio(GPIO_LTDC_G7);
  stm32n6_configgpio(GPIO_LTDC_B0);
  stm32n6_configgpio(GPIO_LTDC_B1);
  stm32n6_configgpio(GPIO_LTDC_B2);
  stm32n6_configgpio(GPIO_LTDC_B3);
  stm32n6_configgpio(GPIO_LTDC_B4);
  stm32n6_configgpio(GPIO_LTDC_B5);
  stm32n6_configgpio(GPIO_LTDC_B6);
  stm32n6_configgpio(GPIO_LTDC_B7);
  stm32n6_configgpio(GPIO_LTDC_DE);
  stm32n6_configgpio(GPIO_LTDC_CLK);
  stm32n6_configgpio(GPIO_LTDC_HSYNC);
  stm32n6_configgpio(GPIO_LTDC_VSYNC);

#ifdef CONFIG_STM32N6570_DK_DISPLAY
  stm32n6_configgpio(GPIO_DISPLAY_ON);
  stm32n6_configgpio(GPIO_DISPLAY_BL);
#endif

  return stm32n6_ltdc_initialize();
}
#endif

/****************************************************************************
 * Name: stm32_gt911_setup
 ****************************************************************************/

#ifdef CONFIG_STM32N6570_DK_GT911
static int stm32_gt911_setup(void)
{
  stm32n6_configgpio(GPIO_GT911_RESET);
  stm32n6_configgpio(GPIO_GT911_IRQ);

  return -ENOSYS;
}
#endif

/****************************************************************************
 * Name: stm32_xspi1_psram_setup
 ****************************************************************************/

#ifdef CONFIG_STM32N6570_DK_XSPI1_PSRAM
static int stm32_xspi1_psram_setup(void)
{
  FAR struct qspi_dev_s *qspi;

  qspi = stm32n6_xspi_initialize(1);
  return qspi == NULL ? -ENODEV : -ENOSYS;
}
#endif

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32_bringup
 ****************************************************************************/

int stm32_bringup(void)
{
#ifdef CONFIG_STM32N6570_DK_I2C1
  stm32_bringup_report("I2C1", stm32_i2c_setup(1));
#endif

#ifdef CONFIG_STM32N6570_DK_I2C2
  stm32_bringup_report("I2C2", stm32_i2c_setup(2));
#endif

#ifdef CONFIG_STM32N6570_DK_I2C4
  stm32_bringup_report("I2C4", stm32_i2c_setup(4));
#endif

#ifdef CONFIG_STM32N6570_DK_GT911
  stm32_bringup_report("GT911", stm32_gt911_setup());
#endif

#ifdef CONFIG_STM32N6570_DK_SPI5
  stm32_bringup_report("SPI5", stm32_spi5_setup());
#endif

#ifdef CONFIG_STM32N6570_DK_FDCAN1
  stm32_bringup_report("FDCAN1", stm32_fdcan1_setup());
#endif

#ifdef CONFIG_STM32N6570_DK_SDMMC2
  stm32_bringup_report("SDMMC2", stm32_sdmmc2_setup());
#endif

#ifdef CONFIG_STM32N6570_DK_ADC1
  stm32_bringup_report("ADC1", stm32_adc1_setup());
#endif

#ifdef CONFIG_STM32N6570_DK_TIM1_PWM
  stm32_bringup_report("TIM1 PWM", stm32_pwm_setup(1));
#endif

#ifdef CONFIG_STM32N6570_DK_TIM15_PWM
  stm32_bringup_report("TIM15 PWM", stm32_pwm_setup(15));
#endif

#ifdef CONFIG_STM32N6570_DK_ETH
  stm32_bringup_report("Ethernet", stm32_eth_setup());
#endif

#ifdef CONFIG_STM32N6570_DK_LTDC
  stm32_bringup_report("LTDC", stm32_ltdc_setup());
#endif

#ifdef CONFIG_STM32N6570_DK_XSPI1_PSRAM
  stm32_bringup_report("XSPI1 PSRAM", stm32_xspi1_psram_setup());
#endif

#ifdef CONFIG_STM32N6570_DK_XSPI2_FLASH
  stm32_bringup_report("XSPI2 Flash", stm32_xspi_initialize());
#endif

  return OK;
}
