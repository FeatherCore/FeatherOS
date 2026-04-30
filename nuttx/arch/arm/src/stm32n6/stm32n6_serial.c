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
#include <stdbool.h>
#include <stdint.h>
#include <errno.h>
#include <termios.h>

#include <nuttx/arch.h>
#include <nuttx/fs/ioctl.h>
#include <nuttx/irq.h>
#include <nuttx/serial/serial.h>

#include "arm_internal.h"
#include "chip.h"
#include "hardware/stm32n6_memorymap.h"
#include "hardware/stm32n6_rcc.h"
#include "hardware/stm32n6_uart.h"
#include "stm32n6_gpio.h"
#include "stm32n6_serial.h"

#include <arch/board/board.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

#if defined(CONFIG_USART1_SERIAL_CONSOLE)
#  define CONSOLE_DEV     g_usart1_dev
#  define CONSOLE_BASE    STM32_USART1_BASE
#elif defined(CONFIG_USART2_SERIAL_CONSOLE)
#  define CONSOLE_DEV     g_usart2_dev
#  define CONSOLE_BASE    STM32_USART2_BASE
#endif

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

#ifdef USE_SERIALDRIVER
static int stm32n6_setup(struct uart_dev_s *dev);
static void stm32n6_shutdown(struct uart_dev_s *dev);
static int stm32n6_attach(struct uart_dev_s *dev);
static void stm32n6_detach(struct uart_dev_s *dev);
static int stm32n6_ioctl(struct file *filep, int cmd, unsigned long arg);
static int stm32n6_receive(struct uart_dev_s *dev, unsigned int *status);
static void stm32n6_rxint(struct uart_dev_s *dev, bool enable);
static bool stm32n6_rxavailable(struct uart_dev_s *dev);
static void stm32n6_send(struct uart_dev_s *dev, int ch);
static void stm32n6_txint(struct uart_dev_s *dev, bool enable);
static bool stm32n6_txready(struct uart_dev_s *dev);
static bool stm32n6_txempty(struct uart_dev_s *dev);
#endif

/****************************************************************************
 * Private Data
 ****************************************************************************/

#ifdef USE_SERIALDRIVER
static const struct uart_ops_s g_uart_ops =
{
  .setup       = stm32n6_setup,
  .shutdown    = stm32n6_shutdown,
  .attach      = stm32n6_attach,
  .detach      = stm32n6_detach,
  .ioctl       = stm32n6_ioctl,
  .receive     = stm32n6_receive,
  .rxint       = stm32n6_rxint,
  .rxavailable = stm32n6_rxavailable,
  .send        = stm32n6_send,
  .txint       = stm32n6_txint,
  .txready     = stm32n6_txready,
  .txempty     = stm32n6_txempty,
};
#endif

#ifdef CONFIG_STM32N6_USART1
static struct stm32n6_uart_s g_usart1_priv =
{
  .uartbase  = STM32_USART1_BASE,
  .baud      = CONFIG_USART1_BAUD,
  .bits      = CONFIG_USART1_BITS,
  .parity    = CONFIG_USART1_PARITY,
  .stopbits2 = CONFIG_USART1_2STOP,
  .irq       = STM32N6_IRQ_USART1,
};
#endif

#ifdef CONFIG_STM32N6_USART2
static struct stm32n6_uart_s g_usart2_priv =
{
  .uartbase  = STM32_USART2_BASE,
  .baud      = CONFIG_USART2_BAUD,
  .bits      = CONFIG_USART2_BITS,
  .parity    = CONFIG_USART2_PARITY,
  .stopbits2 = CONFIG_USART2_2STOP,
  .irq       = STM32N6_IRQ_USART2,
};
#endif

#if defined(USE_SERIALDRIVER) && defined(CONFIG_USART1_SERIALDRIVER)
static char g_usart1_rxbuffer[CONFIG_USART1_RXBUFSIZE];
static char g_usart1_txbuffer[CONFIG_USART1_TXBUFSIZE];

static struct uart_dev_s g_usart1_dev =
{
  .recv =
  {
    .size   = CONFIG_USART1_RXBUFSIZE,
    .buffer = g_usart1_rxbuffer,
  },
  .xmit =
  {
    .size   = CONFIG_USART1_TXBUFSIZE,
    .buffer = g_usart1_txbuffer,
  },
  .ops     = &g_uart_ops,
  .priv    = &g_usart1_priv,
};
#endif

#if defined(USE_SERIALDRIVER) && defined(CONFIG_USART2_SERIALDRIVER)
static char g_usart2_rxbuffer[CONFIG_USART2_RXBUFSIZE];
static char g_usart2_txbuffer[CONFIG_USART2_TXBUFSIZE];

static struct uart_dev_s g_usart2_dev =
{
  .recv =
  {
    .size   = CONFIG_USART2_RXBUFSIZE,
    .buffer = g_usart2_rxbuffer,
  },
  .xmit =
  {
    .size   = CONFIG_USART2_TXBUFSIZE,
    .buffer = g_usart2_txbuffer,
  },
  .ops     = &g_uart_ops,
  .priv    = &g_usart2_priv,
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

  if (uartbase == STM32_USART1_BASE)
    {
      regval = getreg32(STM32_RCC_APB2ENR);
      regval |= RCC_APB2ENR_USART1EN;
      putreg32(regval, STM32_RCC_APB2ENR);
    }
  else if (uartbase == STM32_USART6_BASE)
    {
      regval = getreg32(STM32_RCC_APB2ENR);
      regval |= RCC_APB2ENR_USART6EN;
      putreg32(regval, STM32_RCC_APB2ENR);
    }
  else
    {
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
}

static void stm32n6_uart_pins(uintptr_t uartbase)
{
#if defined(CONFIG_STM32N6_USART1)
  if (uartbase == STM32_USART1_BASE)
    {
      stm32n6_configgpio(GPIO_USART1_TX);
      stm32n6_configgpio(GPIO_USART1_RX);
    }
#endif

#if defined(CONFIG_STM32N6_USART2)
  if (uartbase == STM32_USART2_BASE)
    {
      stm32n6_configgpio(GPIO_USART2_TX);
      stm32n6_configgpio(GPIO_USART2_RX);
    }
#endif
}

#ifdef USE_SERIALDRIVER
static int stm32n6_interrupt(int irq, void *context, void *arg)
{
  struct uart_dev_s *dev = arg;
  struct stm32n6_uart_s *priv = dev->priv;
  uint32_t status;

  status = stm32n6_uart_getreg(priv->uartbase, STM32_USART_ISR_OFFSET);
  if ((status & USART_ISR_RXNE_RXFNE) != 0)
    {
      uart_recvchars(dev);
    }

  if ((status & USART_ISR_TXE_TXFNF) != 0)
    {
      uart_xmitchars(dev);
    }

  return OK;
}

static int stm32n6_setup(struct uart_dev_s *dev)
{
  struct stm32n6_uart_s *priv = dev->priv;

  stm32n6_uart_pins(priv->uartbase);

  return stm32n6_uart_configure(priv->uartbase, priv->baud, priv->bits,
                                priv->parity, priv->stopbits2);
}

static void stm32n6_shutdown(struct uart_dev_s *dev)
{
  struct stm32n6_uart_s *priv = dev->priv;
  uint32_t regval;

  regval = stm32n6_uart_getreg(priv->uartbase, STM32_USART_CR1_OFFSET);
  regval &= ~USART_CR1_UE;
  stm32n6_uart_putreg(priv->uartbase, STM32_USART_CR1_OFFSET, regval);
}

static int stm32n6_attach(struct uart_dev_s *dev)
{
  struct stm32n6_uart_s *priv = dev->priv;
  int ret;

  ret = irq_attach(priv->irq, stm32n6_interrupt, dev);
  if (ret == OK)
    {
      up_enable_irq(priv->irq);
    }

  return ret;
}

static void stm32n6_detach(struct uart_dev_s *dev)
{
  struct stm32n6_uart_s *priv = dev->priv;

  up_disable_irq(priv->irq);
  irq_detach(priv->irq);
}

static int stm32n6_ioctl(struct file *filep, int cmd, unsigned long arg)
{
  struct inode *inode = filep->f_inode;
  struct uart_dev_s *dev = inode->i_private;
  struct stm32n6_uart_s *priv = dev->priv;
  int ret = OK;

  switch (cmd)
    {
      case TCSETS:
        {
          struct termios *term = (struct termios *)arg;

          priv->baud = term->c_speed;
          priv->parity = 0;
          priv->stopbits2 = (term->c_cflag & CSTOPB) != 0;
          ret = stm32n6_uart_configure(priv->uartbase, priv->baud,
                                       priv->bits, priv->parity,
                                       priv->stopbits2);
        }
        break;

      default:
        ret = -ENOTTY;
        break;
    }

  return ret;
}

static int stm32n6_receive(struct uart_dev_s *dev, unsigned int *status)
{
  struct stm32n6_uart_s *priv = dev->priv;

  *status = stm32n6_uart_getreg(priv->uartbase, STM32_USART_ISR_OFFSET);

  return stm32n6_uart_receive(priv->uartbase);
}

static void stm32n6_rxint(struct uart_dev_s *dev, bool enable)
{
  struct stm32n6_uart_s *priv = dev->priv;
  uint32_t regval;

  regval = stm32n6_uart_getreg(priv->uartbase, STM32_USART_CR1_OFFSET);
  if (enable)
    {
      regval |= USART_CR1_RXNEIE_RXFNEIE;
    }
  else
    {
      regval &= ~USART_CR1_RXNEIE_RXFNEIE;
    }

  stm32n6_uart_putreg(priv->uartbase, STM32_USART_CR1_OFFSET, regval);
}

static bool stm32n6_rxavailable(struct uart_dev_s *dev)
{
  struct stm32n6_uart_s *priv = dev->priv;

  return stm32n6_uart_rxavailable(priv->uartbase);
}

static void stm32n6_send(struct uart_dev_s *dev, int ch)
{
  struct stm32n6_uart_s *priv = dev->priv;

  stm32n6_uart_send(priv->uartbase, ch);
}

static void stm32n6_txint(struct uart_dev_s *dev, bool enable)
{
  struct stm32n6_uart_s *priv = dev->priv;
  uint32_t regval;

  regval = stm32n6_uart_getreg(priv->uartbase, STM32_USART_CR1_OFFSET);
  if (enable)
    {
      regval |= USART_CR1_TXEIE_TXFNFIE;
    }
  else
    {
      regval &= ~USART_CR1_TXEIE_TXFNFIE;
    }

  stm32n6_uart_putreg(priv->uartbase, STM32_USART_CR1_OFFSET, regval);
}

static bool stm32n6_txready(struct uart_dev_s *dev)
{
  struct stm32n6_uart_s *priv = dev->priv;

  return stm32n6_uart_txready(priv->uartbase);
}

static bool stm32n6_txempty(struct uart_dev_s *dev)
{
  struct stm32n6_uart_s *priv = dev->priv;

  return stm32n6_uart_txempty(priv->uartbase);
}
#endif

/****************************************************************************
 * Public Functions
 ****************************************************************************/

int stm32n6_uart_configure(uintptr_t uartbase, uint32_t baud, uint8_t bits,
                           uint8_t parity, uint8_t stopbits2)
{
  uint32_t regval;
  uint32_t brr;
  uint32_t clock;

  stm32n6_uart_enable_clock(uartbase);

  regval = stm32n6_uart_getreg(uartbase, STM32_USART_CR1_OFFSET);
  regval &= ~USART_CR1_UE;
  stm32n6_uart_putreg(uartbase, STM32_USART_CR1_OFFSET, regval);

  regval = USART_CR1_TE | USART_CR1_RE;

  if (bits == 7)
    {
      regval |= USART_CR1_M1;
    }

  if (parity == 1)
    {
      regval |= USART_CR1_PCE | USART_CR1_PS;
    }
  else if (parity == 2)
    {
      regval |= USART_CR1_PCE;
    }

  stm32n6_uart_putreg(uartbase, STM32_USART_CR1_OFFSET, regval);

  if (stopbits2 != 0)
    {
      stm32n6_uart_putreg(uartbase, STM32_USART_CR2_OFFSET,
                          USART_CR2_STOP_2);
    }
  else
    {
      stm32n6_uart_putreg(uartbase, STM32_USART_CR2_OFFSET,
                          USART_CR2_STOP_1);
    }

  clock = uartbase == STM32_USART1_BASE ? STM32N6_PCLK2_FREQUENCY :
                                          STM32N6_PCLK1_FREQUENCY;
  brr = (clock + (baud / 2)) / baud;
  stm32n6_uart_putreg(uartbase, STM32_USART_BRR_OFFSET, brr);

  stm32n6_uart_putreg(uartbase, STM32_USART_ICR_OFFSET, 0xffffffff);

  regval = stm32n6_uart_getreg(uartbase, STM32_USART_CR1_OFFSET);
  regval |= USART_CR1_UE;
  stm32n6_uart_putreg(uartbase, STM32_USART_CR1_OFFSET, regval);

  return OK;
}

void stm32n6_uart_send(uintptr_t uartbase, int ch)
{
  while (!stm32n6_uart_txready(uartbase));

  stm32n6_uart_putreg(uartbase, STM32_USART_TDR_OFFSET, (uint32_t)ch);
}

int stm32n6_uart_receive(uintptr_t uartbase)
{
  return (int)(stm32n6_uart_getreg(uartbase, STM32_USART_RDR_OFFSET) & 0xff);
}

bool stm32n6_uart_txready(uintptr_t uartbase)
{
  return (stm32n6_uart_getreg(uartbase, STM32_USART_ISR_OFFSET) &
          USART_ISR_TXE_TXFNF) != 0;
}

bool stm32n6_uart_txempty(uintptr_t uartbase)
{
  return (stm32n6_uart_getreg(uartbase, STM32_USART_ISR_OFFSET) &
          USART_ISR_TC) != 0;
}

bool stm32n6_uart_rxavailable(uintptr_t uartbase)
{
  return (stm32n6_uart_getreg(uartbase, STM32_USART_ISR_OFFSET) &
          USART_ISR_RXNE_RXFNE) != 0;
}

#ifdef USE_EARLYSERIALINIT
void arm_earlyserialinit(void)
{
#ifdef CONFIG_USART1_SERIAL_CONSOLE
  g_usart1_dev.isconsole = true;
  stm32n6_setup(&g_usart1_dev);
#elif defined(CONFIG_USART2_SERIAL_CONSOLE)
  g_usart2_dev.isconsole = true;
  stm32n6_setup(&g_usart2_dev);
#endif
}
#endif

#ifdef USE_SERIALDRIVER
void arm_serialinit(void)
{
#ifdef CONSOLE_DEV
  uart_register("/dev/console", &CONSOLE_DEV);
#endif

#ifdef CONFIG_USART1_SERIALDRIVER
  uart_register("/dev/ttyS0", &g_usart1_dev);
#endif

#ifdef CONFIG_USART2_SERIALDRIVER
  uart_register("/dev/ttyS1", &g_usart2_dev);
#endif
}
#endif

void up_putc(int ch)
{
#ifdef CONSOLE_BASE
  stm32n6_uart_send(CONSOLE_BASE, ch);
#endif
}
