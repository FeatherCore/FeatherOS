/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_serial.c
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
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include <assert.h>
#include <debug.h>
#include <errno.h>

#include <nuttx/irq.h>
#include <nuttx/arch.h>
#include <nuttx/semaphore.h>
#include <nuttx/fs/ioctl.h>
#include <nuttx/serial/serial.h>

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_memorymap.h"
#include "hardware/stm32n6_uart.h"
#include "hardware/stm32n6_rcc.h"
#include "stm32n6_gpio.h"
#include "stm32n6_serial.h"

/****************************************************************************
 * Private Data
 ****************************************************************************/

#ifdef CONFIG_STM32N6_UART1
static struct stm32n6_uart_s g_uart1_priv =
{
  .uartbase   = STM32_USART1_BASE,
  .baudrate   = CONFIG_STM32N6_UART1_BAUD,
  .irq        = STM32N6_IRQ_USART1,
};
#endif

#ifdef CONFIG_STM32N6_UART2
static struct stm32n6_uart_s g_uart2_priv =
{
  .uartbase   = STM32_USART2_BASE,
  .baudrate   = CONFIG_STM32N6_UART2_BAUD,
  .irq        = STM32N6_IRQ_USART2,
};
#endif

/****************************************************************************
 * Private Functions
 ****************************************************************************/

static inline void stm32n6_uart_putreg(uintptr_t uartbase,
                                       uint32_t offset, uint32_t value)
{
  putreg32(value, uartbase + offset);
}

static inline uint32_t stm32n6_uart_getreg(uintptr_t uartbase,
                                           uint32_t offset)
{
  return getreg32(uartbase + offset);
}

static void stm32n6_uart_enable_clock(uintptr_t uartbase)
{
  uint32_t regval;

  regval = getreg32(STM32_RCC_APB2ENR);
  if (uartbase == STM32_USART1_BASE)
    {
      regval |= RCC_APB2ENR_USART1EN;
    }
  else if (uartbase == STM32_USART6_BASE)
    {
      regval |= RCC_APB2ENR_USART6EN;
    }
  putreg32(regval, STM32_RCC_APB2ENR);

  regval = getreg32(STM32_RCC_APB1ENR);
  if (uartbase == STM32_USART2_BASE)
    {
      regval |= RCC_APB1ENR_USART2EN;
    }
  else if (uartbase == STM32_USART3_BASE)
    {
      regval |= RCC_APB1ENR_USART3EN;
    }
  else if (uartbase == STM32_UART4_BASE)
    {
      regval |= RCC_APB1ENR_UART4EN;
    }
  else if (uartbase == STM32_UART5_BASE)
    {
      regval |= RCC_APB1ENR_UART5EN;
    }
  putreg32(regval, STM32_RCC_APB1ENR);
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

void stm32n6_uart_enable(uintptr_t uartbase)
{
  uint32_t regval;

  stm32n6_uart_enable_clock(uartbase);

  regval = stm32n6_uart_getreg(uartbase, STM32_USART_CR1_OFFSET);
  regval |= USART_CR1_UE;
  stm32n6_uart_putreg(uartbase, STM32_USART_CR1_OFFSET, regval);
}

void stm32n6_uart_disable(uintptr_t uartbase)
{
  uint32_t regval;

  regval = stm32n6_uart_getreg(uartbase, STM32_USART_CR1_OFFSET);
  regval &= ~USART_CR1_UE;
  stm32n6_uart_putreg(uartbase, STM32_USART_CR1_OFFSET, regval);
}

int stm32n6_uart_configure(uintptr_t uartbase, uint32_t baudrate,
                           uint32_t parity, uint32_t stopbits)
{
  uint32_t regval;
  uint32_t brr;
  uint32_t clock;

  clock = STM32N6_PCLK1_FREQUENCY;

  stm32n6_uart_disable(uartbase);

  regval = 0;
  regval |= USART_CR1_TE | USART_CR1_RE;

  if (parity == 1)
    {
      regval |= USART_CR1_PCE | USART_CR1_PS;
    }
  else if (parity == 2)
    {
      regval |= USART_CR1_PCE;
    }

  stm32n6_uart_putreg(uartbase, STM32_USART_CR1_OFFSET, regval);

  regval = 0;
  if (stopbits == 1)
    {
      regval |= USART_CR2_STOP_1;
    }
  else if (stopbits == 2)
    {
      regval |= USART_CR2_STOP_2;
    }
  stm32n6_uart_putreg(uartbase, STM32_USART_CR2_OFFSET, regval);

  brr = (clock + (baudrate / 2)) / baudrate;
  stm32n6_uart_putreg(uartbase, STM32_USART_BRR_OFFSET, brr);

  stm32n6_uart_enable(uartbase);

  return OK;
}

void stm32n6_uart_send(uintptr_t uartbase, uint8_t ch)
{
  while (!stm32n6_uart_txready(uartbase));

  stm32n6_uart_putreg(uartbase, STM32_USART_TDR_OFFSET, (uint32_t)ch);
}

uint8_t stm32n6_uart_receive(uintptr_t uartbase)
{
  while (!stm32n6_uart_rxavailable(uartbase));

  return (uint8_t)stm32n6_uart_getreg(uartbase, STM32_USART_RDR_OFFSET);
}

bool stm32n6_uart_txready(uintptr_t uartbase)
{
  uint32_t regval;

  regval = stm32n6_uart_getreg(uartbase, STM32_USART_ISR_OFFSET);
  return (regval & USART_ISR_TXE) != 0;
}

bool stm32n6_uart_rxavailable(uintptr_t uartbase)
{
  uint32_t regval;

  regval = stm32n6_uart_getreg(uartbase, STM32_USART_ISR_OFFSET);
  return (regval & USART_ISR_RXNE) != 0;
}

/****************************************************************************
 * Name: stm32n6_serial_setup
 *
 * Description:
 *   Configure the UART baud, bits, parity, etc.
 *
 ****************************************************************************/

#ifdef USE_SERIALDRIVER
static int stm32n6_serial_setup(struct uart_dev_s *dev)
{
  struct stm32n6_uart_s *priv = (struct stm32n6_uart_s *)dev->priv;

  return stm32n6_uart_configure(priv->uartbase, priv->baudrate, 0, 1);
}

static int stm32n6_serial_puts(struct uart_dev_s *dev, int count,
                               const uint8_t *buffer)
{
  struct stm32n6_uart_s *priv = (struct stm32n6_uart_s *)dev->priv;
  int i;

  for (i = 0; i < count; i++)
    {
      stm32n6_uart_send(priv->uartbase, buffer[i]);
    }

  return count;
}

static int stm32n6_serial_gets(struct uart_dev_s *dev, int count,
                               uint8_t *buffer)
{
  struct stm32n6_uart_s *priv = (struct stm32n6_uart_s *)dev->priv;
  int i;

  for (i = 0; i < count; i++)
    {
      buffer[i] = stm32n6_uart_receive(priv->uartbase);
    }

  return count;
}

static void stm32n6_serial_txint(struct uart_dev_s *dev, bool enable)
{
  struct stm32n6_uart_s *priv = (struct stm32n6_uart_s *)dev->priv;
  uint32_t regval;

  regval = stm32n6_uart_getreg(priv->uartbase, STM32_USART_CR1_OFFSET);
  if (enable)
    {
      regval |= USART_CR1_TXEIE;
    }
  else
    {
      regval &= ~USART_CR1_TXEIE;
    }
  stm32n6_uart_putreg(priv->uartbase, STM32_USART_CR1_OFFSET, regval);
}

static void stm32n6_serial_rxint(struct uart_dev_s *dev, bool enable)
{
  struct stm32n6_uart_s *priv = (struct stm32n6_uart_s *)dev->priv;
  uint32_t regval;

  regval = stm32n6_uart_getreg(priv->uartbase, STM32_USART_CR1_OFFSET);
  if (enable)
    {
      regval |= USART_CR1_RXNEIE;
    }
  else
    {
      regval &= ~USART_CR1_RXNEIE;
    }
  stm32n6_uart_putreg(priv->uartbase, STM32_USART_CR1_OFFSET, regval);
}

static bool stm32n6_serial_txready(struct uart_dev_s *dev)
{
  struct stm32n6_uart_s *priv = (struct stm32n6_uart_s *)dev->priv;
  return stm32n6_uart_txready(priv->uartbase);
}

static bool stm32n6_serial_rxavailable(struct uart_dev_s *dev)
{
  struct stm32n6_uart_s *priv = (struct stm32n6_uart_s *)dev->priv;
  return stm32n6_uart_rxavailable(priv->uartbase);
}

static int stm32n6_serial_ioctl(struct uart_dev_s *dev, int cmd,
                                unsigned long arg)
{
  struct stm32n6_uart_s *priv = (struct stm32n6_uart_s *)dev->priv;
  int ret = OK;

  switch (cmd)
    {
      case TCIOSETSPEED:
        {
          uint32_t baudrate = (uint32_t)arg;
          ret = stm32n6_uart_configure(priv->uartbase, baudrate, 0, 1);
          if (ret == OK)
            {
              priv->baudrate = baudrate;
            }
        }
        break;

      default:
        ret = -ENOTTY;
        break;
    }

  return ret;
}

/****************************************************************************
 * Serial Driver Operations
 ****************************************************************************/

static const struct uart_ops_s g_uart_ops =
{
  .setup       = stm32n6_serial_setup,
  .shutdown    = NULL,
  .attach      = NULL,
  .detach      = NULL,
  .txint       = stm32n6_serial_txint,
  .rxint       = stm32n6_serial_rxint,
  .txready     = stm32n6_serial_txready,
  .rxavailable = stm32n6_serial_rxavailable,
  .ioctl       = stm32n6_serial_ioctl,
  .receive     = stm32n6_serial_gets,
  .send        = stm32n6_serial_puts,
};

#ifdef CONFIG_STM32N6_UART1_SERIALDRIVER
static struct uart_dev_s g_uart1_dev =
{
  .ops  = &g_uart_ops,
  .priv = &g_uart1_priv,
};
#endif

#ifdef CONFIG_STM32N6_UART2_SERIALDRIVER
static struct uart_dev_s g_uart2_dev =
{
  .ops  = &g_uart_ops,
  .priv = &g_uart2_priv,
};
#endif

/****************************************************************************
 * Name: stm32n6_serial_initialize
 *
 * Description:
 *   Register serial devices
 *
 ****************************************************************************/

void stm32n6_serial_initialize(void)
{
#ifdef CONFIG_STM32N6_UART1_SERIALDRIVER
  uart_register("/dev/ttyS0", &g_uart1_dev);
#endif

#ifdef CONFIG_STM32N6_UART2_SERIALDRIVER
  uart_register("/dev/ttyS1", &g_uart2_dev);
#endif
}

#endif /* USE_SERIALDRIVER */
