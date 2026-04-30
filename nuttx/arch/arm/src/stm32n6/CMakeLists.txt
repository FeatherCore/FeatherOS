# ##############################################################################
# arch/arm/src/stm32n6/CMakeLists.txt
#
# SPDX-License-Identifier: Apache-2.0
#
# Licensed to the Apache Software Foundation (ASF) under one or more contributor
# license agreements.  See the NOTICE file distributed with this work for
# additional information regarding copyright ownership.  The ASF licenses this
# file to you under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License.  You may obtain a copy of
# the License at
#
# http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
# WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.  See the
# License for the specific language governing permissions and limitations under
# the License.
#
# ##############################################################################

set(SRCS stm32n6_allocateheap.c stm32n6_clockconfig.c stm32n6_idle.c
         stm32n6_irq.c stm32n6_lowsetup.c stm32n6_soc.c stm32n6_start.c
         stm32n6_timerisr.c)

if(CONFIG_STM32N6_NPU)
  list(APPEND SRCS stm32n6_npu.c)
endif()

if(CONFIG_STM32N6_RIF)
  list(APPEND SRCS stm32n6_rif.c)
endif()

if(CONFIG_STM32N6_AXISRAM3 OR CONFIG_STM32N6_AXISRAM4
   OR CONFIG_STM32N6_AXISRAM5 OR CONFIG_STM32N6_AXISRAM6)
  list(APPEND SRCS stm32n6_axisram.c)
endif()

if(CONFIG_STM32N6_GPIO)
  list(APPEND SRCS stm32n6_gpio.c)
endif()

if(CONFIG_STM32N6_USART)
  list(APPEND SRCS stm32n6_serial.c)
endif()

if(CONFIG_STM32N6_SPI)
  list(APPEND SRCS stm32n6_spi.c)
endif()

if(CONFIG_STM32N6_I2C)
  list(APPEND SRCS stm32n6_i2c.c)
endif()

if(CONFIG_STM32N6_ETH)
  list(APPEND SRCS stm32n6_eth.c)
endif()

if(CONFIG_STM32N6_LTDC)
  list(APPEND SRCS stm32n6_ltdc.c)
endif()

if(CONFIG_STM32N6_USB)
  list(APPEND SRCS stm32n6_usb.c)
endif()

if(CONFIG_STM32N6_XSPI)
  list(APPEND SRCS stm32n6_xspi.c)
endif()

if(CONFIG_STM32N6_JPEG)
  list(APPEND SRCS stm32n6_jpeg.c)
endif()

if(CONFIG_STM32N6_VENC)
  list(APPEND SRCS stm32n6_venc.c)
endif()

if(CONFIG_STM32N6_DMA)
  list(APPEND SRCS stm32n6_dma.c)
endif()

if(CONFIG_STM32N6_ADC)
  list(APPEND SRCS stm32n6_adc.c)
endif()

if(CONFIG_STM32N6_TIMERS)
  list(APPEND SRCS stm32n6_tim.c)
endif()

if(CONFIG_STM32N6_PWM)
  list(APPEND SRCS stm32n6_pwm.c)
endif()

if(CONFIG_STM32N6_EXTI)
  list(APPEND SRCS stm32n6_exti.c)
endif()

if(CONFIG_STM32N6_HSEM)
  list(APPEND SRCS stm32n6_hsem.c)
endif()

if(CONFIG_STM32N6_LPTIM)
  list(APPEND SRCS stm32n6_lptim.c)
endif()

if(CONFIG_STM32N6_FLASH)
  list(APPEND SRCS stm32n6_flash.c)
endif()

if(CONFIG_STM32N6_PWR)
  list(APPEND SRCS stm32n6_pwr.c)
endif()

if(CONFIG_STM32N6_WWDG)
  list(APPEND SRCS stm32n6_wdg.c)
endif()

if(CONFIG_STM32N6_FDCAN)
  list(APPEND SRCS stm32n6_can.c)
endif()

if(CONFIG_STM32N6_SDMMC)
  list(APPEND SRCS stm32n6_sdmmc.c)
endif()

target_sources(arch PRIVATE ${SRCS})
