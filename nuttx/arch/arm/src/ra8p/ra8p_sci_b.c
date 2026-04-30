/****************************************************************************
 * arch/arm/src/ra8p/ra8p_sci_b.c
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
#include <unistd.h>
#include <string.h>
#include <errno.h>
#include <debug.h>

#include <nuttx/irq.h>
#include <nuttx/arch.h>
#include <nuttx/serial/serial.h>
#include <nuttx/spinlock.h>

#include <arch/board/board.h>

#include "chip.h"
#include "arm_internal.h"
#include "ra8p_config.h"
#include "ra8p_sci_b.h"

#ifdef USE_SERIALDRIVER

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Which SCI_B UARTs are enabled? */

#ifdef CONFIG_RA8P_SCI_B_UART0
#  define SCI0_ENABLED 1
#else
#  define SCI0_ENABLED 0
#endif

#ifdef CONFIG_RA8P_SCI_B_UART1
#  define SCI1_ENABLED 1
#else
#  define SCI1_ENABLED 0
#endif

#ifdef CONFIG_RA8P_SCI_B_UART2
#  define SCI2_ENABLED 1
#else
#  define SCI2_ENABLED 0
#endif

#ifdef CONFIG_RA8P_SCI_B_UART3
#  define SCI3_ENABLED 1
#else
#  define SCI3_ENABLED 0
#endif

#ifdef CONFIG_RA8P_SCI_B_UART4
#  define SCI4_ENABLED 1
#else
#  define SCI4_ENABLED 0
#endif

#define NUARTS (SCI0_ENABLED + SCI1_ENABLED + SCI2_ENABLED + SCI3_ENABLED + SCI4_ENABLED)

/* Which UART will be used as serial console (or -1)? */

#if defined(CONFIG_SCI0_SERIAL_CONSOLE)
#  define CONSOLE_DEV 0
#elif defined(CONFIG_SCI1_SERIAL_CONSOLE)
#  define CONSOLE_DEV 1
#elif defined(CONFIG_SCI2_SERIAL_CONSOLE)
#  define CONSOLE_DEV 2
#elif defined(CONFIG_SCI3_SERIAL_CONSOLE)
#  define CONSOLE_DEV 3
#elif defined(CONFIG_SCI4_SERIAL_CONSOLE)
#  define CONSOLE_DEV 4
#else
#  undef CONSOLE_DEV
#endif

/****************************************************************************
 * Private Types
 ****************************************************************************/

struct ra8p_sci_b_s
{
  uintptr_t base;            /* Base address of SCI_B registers */
  uint32_t baud;             /* Configured baud rate */
  uint32_t pclk;             /* Peripheral clock frequency */
  uint8_t irq;               /* IRQ number */
  uint8_t parity;            /* 0=none, 1=odd, 2=even */
  uint8_t bits;              /* Number of data bits (7 or 8) */
  uint8_t stopbits;          /* Number of stop bits (1 or 2) */
};

/****************************************************************************
 * Private Function Prototypes
 ****************************************************************************/

static int  sci_b_setup(struct uart_dev_s *dev);
static void sci_b_shutdown(struct uart_dev_s *dev);
static int  sci_b_attach(struct uart_dev_s *dev);
static void sci_b_detach(struct uart_dev_s *dev);
static int  sci_b_ioctl(struct file *filep, int cmd, unsigned long arg);
static int  sci_b_receive(struct uart_dev_s *dev, unsigned int *status);
static void sci_b_rxint(struct uart_dev_s *dev, bool enable);
static bool sci_b_rxavailable(struct uart_dev_s *dev);
static void sci_b_send(struct uart_dev_s *dev, int ch);
static void sci_b_txint(struct uart_dev_s *dev, bool enable);
static bool sci_b_txready(struct uart_dev_s *dev);
static bool sci_b_txempty(struct uart_dev_s *dev);

/****************************************************************************
 * Private Data
 ****************************************************************************/

static const struct uart_ops_s g_sci_b_ops =
{
  .setup       = sci_b_setup,
  .shutdown    = sci_b_shutdown,
  .attach      = sci_b_attach,
  .detach      = sci_b_detach,
  .ioctl       = sci_b_ioctl,
  .receive     = sci_b_receive,
  .rxint       = sci_b_rxint,
  .rxavailable = sci_b_rxavailable,
#ifdef CONFIG_SERIAL_IFLOWCONTROL
  .rxflowcontrol = NULL,
#endif
  .send        = sci_b_send,
  .txint       = sci_b_txint,
  .txready     = sci_b_txready,
  .txempty     = sci_b_txempty,
};

/* SCI_B UART state structures */

static struct ra8p_sci_b_s g_sci_b_priv[NUARTS];
static char g_sci_b_rxbuf[NUARTS][CONFIG_UART_RXBUFSIZE];
static char g_sci_b_txbuf[NUARTS][CONFIG_UART_TXBUFSIZE];

static struct uart_dev_s g_sci_b_dev[NUARTS] =
{
#if SCI0_ENABLED
  [0] =
  {
    .recv =
    {
      .buffer = g_sci_b_rxbuf[0],
      .bufsize = CONFIG_UART_RXBUFSIZE,
    },
    .xmit =
    {
      .buffer = g_sci_b_txbuf[0],
      .bufsize = CONFIG_UART_TXBUFSIZE,
    },
    .ops = &g_sci_b_ops,
    .priv = &g_sci_b_priv[0],
  },
#endif
#if SCI1_ENABLED
  [1] =
  {
    .recv =
    {
      .buffer = g_sci_b_rxbuf[1],
      .bufsize = CONFIG_UART_RXBUFSIZE,
    },
    .xmit =
    {
      .buffer = g_sci_b_txbuf[1],
      .bufsize = CONFIG_UART_TXBUFSIZE,
    },
    .ops = &g_sci_b_ops,
    .priv = &g_sci_b_priv[1],
  },
#endif
#if SCI2_ENABLED
  [2] =
  {
    .recv =
    {
      .buffer = g_sci_b_rxbuf[2],
      .bufsize = CONFIG_UART_RXBUFSIZE,
    },
    .xmit =
    {
      .buffer = g_sci_b_txbuf[2],
      .bufsize = CONFIG_UART_TXBUFSIZE,
    },
    .ops = &g_sci_b_ops,
    .priv = &g_sci_b_priv[2],
  },
#endif
#if SCI3_ENABLED
  [3] =
  {
    .recv =
    {
      .buffer = g_sci_b_rxbuf[3],
      .bufsize = CONFIG_UART_RXBUFSIZE,
    },
    .xmit =
    {
      .buffer = g_sci_b_txbuf[3],
      .bufsize = CONFIG_UART_TXBUFSIZE,
    },
    .ops = &g_sci_b_ops,
    .priv = &g_sci_b_priv[3],
  },
#endif
#if SCI4_ENABLED
  [4] =
  {
    .recv =
    {
      .buffer = g_sci_b_rxbuf[4],
      .bufsize = CONFIG_UART_RXBUFSIZE,
    },
    .xmit =
    {
      .buffer = g_sci_b_txbuf[4],
      .bufsize = CONFIG_UART_TXBUFSIZE,
    },
    .ops = &g_sci_b_ops,
    .priv = &g_sci_b_priv[4],
  },
#endif
};

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: sci_b_getreg
 ****************************************************************************/

static inline uint32_t sci_b_getreg(struct ra8p_sci_b_s *priv,
                                    uint32_t offset)
{
  return getreg32(priv->base + offset);
}

/****************************************************************************
 * Name: sci_b_putreg
 ****************************************************************************/

static inline void sci_b_putreg(struct ra8p_sci_b_s *priv,
                                uint32_t offset, uint32_t value)
{
  putreg32(value, priv->base + offset);
}

/****************************************************************************
 * Name: sci_b_setbaud
 ****************************************************************************/

static void sci_b_setbaud(struct ra8p_sci_b_s *priv, uint32_t baudrate)
{
  uint32_t brr;
  uint32_t pclk = priv->pclk;

  /* Calculate BRR value: BRR = (PCLK / (16 * baudrate)) - 1 */

  brr = (pclk / (16 * baudrate)) - 1;
  if (brr > 255)
    {
      brr = 255;
    }

  sci_b_putreg(priv, RA8P_SCI_B_BRR_OFFSET, brr);
}

/****************************************************************************
 * Name: sci_b_setup
 ****************************************************************************/

static int sci_b_setup(struct uart_dev_s *dev)
{
  struct ra8p_sci_b_s *priv = dev->priv;
  uint32_t regval;

  /* Disable transmit and receive */

  sci_b_putreg(priv, RA8P_SCI_B_CCR0_OFFSET, 0);

  /* Wait for any transmission to complete */

  while ((sci_b_getreg(priv, RA8P_SCI_B_CSR_OFFSET) &
          RA8P_SCI_B_CSR_TEND) == 0);

  /* Configure serial mode register */

  regval = 0;
  if (priv->bits == 7)
    {
      regval |= RA8P_SCI_B_SCMR_CHR_7;
    }

  if (priv->parity == 1)
    {
      regval |= RA8P_SCI_B_SCMR_PE | RA8P_SCI_B_SCMR_PM_ODD;
    }
  else if (priv->parity == 2)
    {
      regval |= RA8P_SCI_B_SCMR_PE | RA8P_SCI_B_SCMR_PM_EVEN;
    }

  if (priv->stopbits == 2)
    {
      regval |= RA8P_SCI_B_SCMR_STOP_2;
    }

  sci_b_putreg(priv, RA8P_SCI_B_SCMR_OFFSET, regval);

  /* Configure extended mode register */

  regval = RA8P_SCI_B_SEMR_BRME;
  sci_b_putreg(priv, RA8P_SCI_B_SEMR_OFFSET, regval);

  /* Set baud rate */

  sci_b_setbaud(priv, priv->baud);

  /* Enable transmit and receive */

  regval = RA8P_SCI_B_CCR0_TE | RA8P_SCI_B_CCR0_RE;
  sci_b_putreg(priv, RA8P_SCI_B_CCR0_OFFSET, regval);

  return OK;
}

/****************************************************************************
 * Name: sci_b_shutdown
 ****************************************************************************/

static void sci_b_shutdown(struct uart_dev_s *dev)
{
  struct ra8p_sci_b_s *priv = dev->priv;

  /* Disable transmit and receive */

  sci_b_putreg(priv, RA8P_SCI_B_CCR0_OFFSET, 0);
}

/****************************************************************************
 * Name: sci_b_attach
 ****************************************************************************/

static int sci_b_attach(struct uart_dev_s *dev)
{
  struct ra8p_sci_b_s *priv = dev->priv;
  int ret;

  /* Attach the IRQ */

  ret = irq_attach(priv->irq, sci_b_interrupt, dev);
  if (ret == OK)
    {
      /* Enable the IRQ */

      up_enable_irq(priv->irq);
    }

  return ret;
}

/****************************************************************************
 * Name: sci_b_detach
 ****************************************************************************/

static void sci_b_detach(struct uart_dev_s *dev)
{
  struct ra8p_sci_b_s *priv = dev->priv;

  /* Disable the IRQ */

  up_disable_irq(priv->irq);

  /* Detach the IRQ */

  irq_detach(priv->irq);
}

/****************************************************************************
 * Name: sci_b_ioctl
 ****************************************************************************/

static int sci_b_ioctl(struct file *filep, int cmd, unsigned long arg)
{
  struct inode *inode = filep->f_inode;
  struct uart_dev_s *dev = inode->i_private;
  struct ra8p_sci_b_s *priv = dev->priv;
  int ret = OK;

  switch (cmd)
    {
      case TCGETS:
        {
          struct termios *termiosp = (struct termios *)arg;
          if (termiosp)
            {
              termiosp->c_cflag = priv->baud;
            }
        }
        break;

      case TCSETS:
        {
          struct termios *termiosp = (struct termios *)arg;
          if (termiosp)
            {
              priv->baud = cfgetispeed(termiosp);
              sci_b_setbaud(priv, priv->baud);
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
 * Name: sci_b_receive
 ****************************************************************************/

static int sci_b_receive(struct uart_dev_s *dev, unsigned int *status)
{
  struct ra8p_sci_b_s *priv = dev->priv;
  uint32_t rdr;

  /* Get the received data */

  rdr = sci_b_getreg(priv, RA8P_SCI_B_RDR_OFFSET);

  /* Get the status */

  if (status)
    {
      *status = sci_b_getreg(priv, RA8P_SCI_B_CSR_OFFSET);
    }

  return (int)(rdr & 0xff);
}

/****************************************************************************
 * Name: sci_b_rxint
 ****************************************************************************/

static void sci_b_rxint(struct uart_dev_s *dev, bool enable)
{
  struct ra8p_sci_b_s *priv = dev->priv;
  uint32_t regval;

  regval = sci_b_getreg(priv, RA8P_SCI_B_CCR0_OFFSET);

  if (enable)
    {
      regval |= RA8P_SCI_B_CCR0_RIE;
    }
  else
    {
      regval &= ~RA8P_SCI_B_CCR0_RIE;
    }

  sci_b_putreg(priv, RA8P_SCI_B_CCR0_OFFSET, regval);
}

/****************************************************************************
 * Name: sci_b_rxavailable
 ****************************************************************************/

static bool sci_b_rxavailable(struct uart_dev_s *dev)
{
  struct ra8p_sci_b_s *priv = dev->priv;
  uint32_t csr;

  csr = sci_b_getreg(priv, RA8P_SCI_B_CSR_OFFSET);

  return (csr & RA8P_SCI_B_CSR_RDRF) != 0;
}

/****************************************************************************
 * Name: sci_b_send
 ****************************************************************************/

static void sci_b_send(struct uart_dev_s *dev, int ch)
{
  struct ra8p_sci_b_s *priv = dev->priv;

  /* Wait for transmit data register to be empty */

  while ((sci_b_getreg(priv, RA8P_SCI_B_CSR_OFFSET) &
          RA8P_SCI_B_CSR_TDRE) == 0);

  /* Write the data */

  sci_b_putreg(priv, RA8P_SCI_B_TDR_OFFSET, (uint32_t)ch);
}

/****************************************************************************
 * Name: sci_b_txint
 ****************************************************************************/

static void sci_b_txint(struct uart_dev_s *dev, bool enable)
{
  struct ra8p_sci_b_s *priv = dev->priv;
  uint32_t regval;

  regval = sci_b_getreg(priv, RA8P_SCI_B_CCR0_OFFSET);

  if (enable)
    {
      regval |= RA8P_SCI_B_CCR0_TIE;
    }
  else
    {
      regval &= ~RA8P_SCI_B_CCR0_TIE;
    }

  sci_b_putreg(priv, RA8P_SCI_B_CCR0_OFFSET, regval);
}

/****************************************************************************
 * Name: sci_b_txready
 ****************************************************************************/

static bool sci_b_txready(struct uart_dev_s *dev)
{
  struct ra8p_sci_b_s *priv = dev->priv;
  uint32_t csr;

  csr = sci_b_getreg(priv, RA8P_SCI_B_CSR_OFFSET);

  return (csr & RA8P_SCI_B_CSR_TDRE) != 0;
}

/****************************************************************************
 * Name: sci_b_txempty
 ****************************************************************************/

static bool sci_b_txempty(struct uart_dev_s *dev)
{
  struct ra8p_sci_b_s *priv = dev->priv;
  uint32_t csr;

  csr = sci_b_getreg(priv, RA8P_SCI_B_CSR_OFFSET);

  return (csr & RA8P_SCI_B_CSR_TEND) != 0;
}

/****************************************************************************
 * Name: sci_b_interrupt
 ****************************************************************************/

static int sci_b_interrupt(int irq, void *context, void *arg)
{
  struct uart_dev_s *dev = arg;
  struct ra8p_sci_b_s *priv = dev->priv;
  uint32_t csr;

  csr = sci_b_getreg(priv, RA8P_SCI_B_CSR_OFFSET);

  /* Check for receive interrupt */

  if ((csr & RA8P_SCI_B_CSR_RDRF) != 0)
    {
      uart_recvchars(dev);
    }

  /* Check for transmit interrupt */

  if ((csr & RA8P_SCI_B_CSR_TDRE) != 0)
    {
      uart_xmitchars(dev);
    }

  /* Clear error flags */

  if ((csr & (RA8P_SCI_B_CSR_ORER | RA8P_SCI_B_CSR_FER |
              RA8P_SCI_B_CSR_PER)) != 0)
    {
      sci_b_putreg(priv, RA8P_SCI_B_CFCLR_OFFSET,
                   RA8P_SCI_B_CFCLR_ORERC | RA8P_SCI_B_CFCLR_FERC |
                   RA8P_SCI_B_CFCLR_PERC);
    }

  return OK;
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: arm_earlyserialinit
 ****************************************************************************/

#ifdef USE_EARLYSERIALINIT
void arm_earlyserialinit(void)
{
  unsigned int i;

  /* Initialize each SCI_B UART */

  for (i = 0; i < NUARTS; i++)
    {
      g_sci_b_priv[i].base = RA8P_SCI_B0_BASE + (i * 0x1000);
      g_sci_b_priv[i].baud = 115200;
      g_sci_b_priv[i].pclk = RA_PCLKB_FREQUENCY;
      g_sci_b_priv[i].bits = 8;
      g_sci_b_priv[i].parity = 0;
      g_sci_b_priv[i].stopbits = 1;

      sci_b_setup(&g_sci_b_dev[i]);
    }

#ifdef CONSOLE_DEV
  /* Register the console */

  uart_register("/dev/console", &g_sci_b_dev[CONSOLE_DEV]);
#endif
}
#endif

/****************************************************************************
 * Name: arm_serialinit
 ****************************************************************************/

void arm_serialinit(void)
{
  unsigned int i;
  char devname[16];

  for (i = 0; i < NUARTS; i++)
    {
#ifdef CONSOLE_DEV
      if (i != CONSOLE_DEV)
#endif
        {
          snprintf(devname, sizeof(devname), "/dev/ttySC%d", i);
          uart_register(devname, &g_sci_b_dev[i]);
        }
    }
}

/****************************************************************************
 * Name: up_putc
 ****************************************************************************/

void up_putc(int ch)
{
#ifdef CONSOLE_DEV
  struct ra8p_sci_b_s *priv = g_sci_b_dev[CONSOLE_DEV].priv;

  /* Wait for transmit data register to be empty */

  while ((sci_b_getreg(priv, RA8P_SCI_B_CSR_OFFSET) &
          RA8P_SCI_B_CSR_TDRE) == 0);

  /* Write the character */

  sci_b_putreg(priv, RA8P_SCI_B_TDR_OFFSET, (uint32_t)ch);

  /* Wait for transmission to complete */

  while ((sci_b_getreg(priv, RA8P_SCI_B_CSR_OFFSET) &
          RA8P_SCI_B_CSR_TEND) == 0);
#endif
}

#endif /* USE_SERIALDRIVER */