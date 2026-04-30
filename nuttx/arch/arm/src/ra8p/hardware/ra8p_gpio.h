/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_gpio.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_GPIO_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_GPIO_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* GPIO Register Offsets - Based on RA8P Hardware Manual */
#define RA8P_GPIO_PODR_OFFSET            0x0000  /* Port Output Data Register */
#define RA8P_GPIO_PIDR_OFFSET            0x0004  /* Port Input Data Register */
#define RA8P_GPIO_PDDR_OFFSET            0x0008  /* Port Direction Data Register */
#define RA8P_GPIO_PDRR_OFFSET            0x000C  /* Port Data Read Register */
#define RA8P_GPIO_PMCSR_OFFSET           0x0010  /* Port Mode Control/Status Register */
#define RA8P_GPIO_PEDR_OFFSET            0x0014  /* Port Extended Data Register */
#define RA8P_GPIO_PCOR_OFFSET            0x0018  /* Port Clear Output Register */
#define RA8P_GPIO_PSOR_OFFSET            0x001C  /* Port Set Output Register */
#define RA8P_GPIO_PPOR_OFFSET            0x0020  /* Port Toggle Output Register */
#define RA8P_GPIO_PIBC_OFFSET            0x0024  /* Port Input Buffer Control Register */
#define RA8P_GPIO_POC1_OFFSET            0x0028  /* Port Output Control 1 Register */
#define RA8P_GPIO_POC2_OFFSET            0x002C  /* Port Output Control 2 Register */
#define RA8P_GPIO_PMC_OFFSET             0x0030  /* Port Mode Control Register */
#define RA8P_GPIO_PFCA_OFFSET            0x0034  /* Port Function Control A Register */
#define RA8P_GPIO_PFCE_OFFSET            0x0038  /* Port Function Control E Register */
#define RA8P_GPIO_PFCR_OFFSET            0x003C  /* Port Function Control Register */
#define RA8P_GPIO_PFCE2_OFFSET           0x0040  /* Port Function Control E2 Register */
#define RA8P_GPIO_PDE_OFFSET             0x0044  /* Port Drive Enable Register */
#define RA8P_GPIO_PSELR_OFFSET           0x0048  /* Port Select Register */
#define RA8P_GPIO_PSEL_OFFSET            0x004C  /* Port Select Register */

/* PFS Register Base Address */
#define RA8P_PFS_BASE                    0x40400800

/* GPIO Port Base Addresses */
#define RA8P_GPIO_PORT0_BASE             0x40400000
#define RA8P_GPIO_PORT1_BASE             0x40400020
#define RA8P_GPIO_PORT2_BASE             0x40400040
#define RA8P_GPIO_PORT3_BASE             0x40400060
#define RA8P_GPIO_PORT4_BASE             0x40400080
#define RA8P_GPIO_PORT5_BASE             0x404000A0
#define RA8P_GPIO_PORT6_BASE             0x404000C0
#define RA8P_GPIO_PORT7_BASE             0x404000E0
#define RA8P_GPIO_PORT8_BASE             0x40400100
#define RA8P_GPIO_PORT9_BASE             0x40400120
#define RA8P_GPIO_PORTA_BASE             0x40400140
#define RA8P_GPIO_PORTB_BASE             0x40400160
#define RA8P_GPIO_PORTC_BASE             0x40400180
#define RA8P_GPIO_PORTD_BASE             0x404001A0

/* Maximum number of ports and pins */
#define RA8P_GPIO_NPORTS                 14      /* Port 0-A, B, C, D */
#define RA8P_GPIO_NPINS_PER_PORT         16      /* Pins 0-15 per port */

/* PFS Register Offsets */
#define RA8P_PFS_P000_OFFSET             0x000
#define RA8P_PFS_P001_OFFSET             0x004
#define RA8P_PFS_P002_OFFSET             0x008
#define RA8P_PFS_P003_OFFSET             0x00C
#define RA8P_PFS_P004_OFFSET             0x010
#define RA8P_PFS_P005_OFFSET             0x014
#define RA8P_PFS_P006_OFFSET             0x018
#define RA8P_PFS_P007_OFFSET             0x01C
#define RA8P_PFS_P008_OFFSET             0x020
#define RA8P_PFS_P009_OFFSET             0x024
#define RA8P_PFS_P010_OFFSET             0x028
#define RA8P_PFS_P011_OFFSET             0x02C
#define RA8P_PFS_P012_OFFSET             0x030
#define RA8P_PFS_P013_OFFSET             0x034
#define RA8P_PFS_P014_OFFSET             0x038
#define RA8P_PFS_P015_OFFSET             0x03C
#define RA8P_PFS_P100_OFFSET             0x040
/* ... continuing pattern for other ports */
#define RA8P_PFS_PORT_OFFSET(port, pin)  (0x040 * (port) + 0x04 * (pin))

/* PFS Register Bit Definitions */
#define RA8P_PFS_PSEL_MASK               (0x1F << 24)  /* Port Select Mask */
#define RA8P_PFS_PSEL_SHIFT              24
#define RA8P_PFS_PSEL_GPIO               0            /* GPIO Mode */
#define RA8P_PFS_PSEL_PERIPHERAL(n)      (n)          /* Peripheral Functions 1-31 */
#define RA8P_PFS_ISEL                    (1 << 22)     /* Interrupt Select */
#define RA8P_PFS_PDR                      (1 << 0)      /* Port Direction */
#define RA8P_PFS_PDR_INPUT                (0 << 0)      /* Input Direction */
#define RA8P_PFS_PDR_OUTPUT               (1 << 0)      /* Output Direction */
#define RA8P_PFS_PODR                     (1 << 1)      /* Port Output Data */
#define RA8P_PFS_PODR_LOW                 (0 << 1)      /* Output Low */
#define RA8P_PFS_PODR_HIGH                (1 << 1)      /* Output High */
#define RA8P_PFS_PCR                      (1 << 2)      /* Pull Control */
#define RA8P_PFS_PCR_DISABLE              (0 << 2)      /* Pull Disable */
#define RA8P_PFS_PCR_ENABLE               (1 << 2)      /* Pull Enable */
#define RA8P_PFS_DSCR                     (1 << 3)      /* Drive Strength Control */
#define RA8P_PFS_DSCR_LOW                 (0 << 3)      /* Low Drive Strength */
#define RA8P_PFS_DSCR_HIGH                (1 << 3)      /* High Drive Strength */
#define RA8P_PFS_ASEL                     (1 << 9)      /* Analog Select */
#define RA8P_PFS_ASEL_DISABLE             (0 << 9)      /* Digital Mode */
#define RA8P_PFS_ASEL_ENABLE              (1 << 9)      /* Analog Mode */

/* Port Direction Data Register bits */
#define RA8P_GPIO_PDDR_PIN_MASK(pin)     (1 << (pin))
#define RA8P_GPIO_PDDR_PIN_INPUT(pin)   (0 << (pin))
#define RA8P_GPIO_PDDR_PIN_OUTPUT(pin)  (1 << (pin))

/* Port Output Data Register bits */
#define RA8P_GPIO_PODR_PIN_MASK(pin)     (1 << (pin))
#define RA8P_GPIO_PODR_PIN_CLEAR(pin)   (0 << (pin))
#define RA8P_GPIO_PODR_PIN_SET(pin)     (1 << (pin))

/* Port Input Data Register bits */
#define RA8P_GPIO_PIDR_PIN_MASK(pin)     (1 << (pin))

/* GPIO pin definitions */
#define GPIO_PORT0                       0
#define GPIO_PORT1                       1
#define GPIO_PORT2                       2
#define GPIO_PORT3                       3
#define GPIO_PORT4                       4
#define GPIO_PORT5                       5
#define GPIO_PORT6                       6
#define GPIO_PORT7                       7
#define GPIO_PORT8                       8
#define GPIO_PORT9                       9
#define GPIO_PORTA                       10
#define GPIO_PORTB                       11
#define GPIO_PORTC                       12
#define GPIO_PORTD                       13

#define GPIO_PIN0                        0
#define GPIO_PIN1                        1
#define GPIO_PIN2                        2
#define GPIO_PIN3                        3
#define GPIO_PIN4                        4
#define GPIO_PIN5                        5
#define GPIO_PIN6                        6
#define GPIO_PIN7                        7
#define GPIO_PIN8                        8
#define GPIO_PIN9                        9
#define GPIO_PIN10                       10
#define GPIO_PIN11                       11
#define GPIO_PIN12                       12
#define GPIO_PIN13                       13
#define GPIO_PIN14                       14
#define GPIO_PIN15                       15

/* GPIO interrupt numbers */
#define RA8P_IRQ_PORT0                   0
#define RA8P_IRQ_PORT1                   1
#define RA8P_IRQ_PORT2                   2
#define RA8P_IRQ_PORT3                   3
#define RA8P_IRQ_PORT4                   4
#define RA8P_IRQ_PORT5                   5
#define RA8P_IRQ_PORT6                   6
#define RA8P_IRQ_PORT7                   7
#define RA8P_IRQ_PORT8                   8
#define RA8P_IRQ_PORT9                   9
#define RA8P_IRQ_PORT10                  10
#define RA8P_IRQ_PORT11                  11
#define RA8P_IRQ_PORT12                  12
#define RA8P_IRQ_PORT13                  13
#define RA8P_IRQ_PORT14                  14
#define RA8P_IRQ_PORT15                  15
#define RA8P_IRQ_PORT16                  16
#define RA8P_IRQ_PORT17                  17
#define RA8P_IRQ_PORT18                  18
#define RA8P_IRQ_PORT19                  19
#define RA8P_IRQ_PORT20                  20
#define RA8P_IRQ_PORT21                  21
#define RA8P_IRQ_PORT22                  22
#define RA8P_IRQ_PORT23                  23
#define RA8P_IRQ_PORT24                  24
#define RA8P_IRQ_PORT25                  25
#define RA8P_IRQ_PORT26                  26
#define RA8P_IRQ_PORT27                  27
#define RA8P_IRQ_PORT28                  28
#define RA8P_IRQ_PORT29                  29
#define RA8P_IRQ_PORT30                  30
#define RA8P_IRQ_PORT31                  31

/* Maximum number of GPIO interrupts */
#define RA8P_GPIO_MAX_IRQS               32

/* GPIO configuration flags */
#define GPIO_INPUT                       (0 << 8)
#define GPIO_OUTPUT                      (1 << 8)
#define GPIO_OUTPUT_ZERO                 (0 << 9)
#define GPIO_OUTPUT_ONE                  (1 << 9)
#define GPIO_PULL_NONE                   (0 << 10)
#define GPIO_PULL_UP                     (1 << 10)
#define GPIO_PULL_DOWN                   (2 << 10)
#define GPIO_DRIVE_STRENGTH_LOW          (0 << 12)
#define GPIO_DRIVE_STRENGTH_HIGH         (1 << 12)
#define GPIO_INT_NONE                    (0 << 13)
#define GPIO_INT_RISING                  (1 << 13)
#define GPIO_INT_FALLING                 (2 << 13)
#define GPIO_INT_BOTH                    (3 << 13)
#define GPIO_ANALOG_DISABLE              (0 << 15)
#define GPIO_ANALOG_ENABLE               (1 << 15)

/* Create GPIO pinset from port and pin */
#define GPIO_PINS(port, pin)             (((uint32_t)(port) << 16) | (pin))

/* GPIO port extraction macros */
#define GPIO_PORT(p)                     (((p) >> 16) & 0xFF)
#define GPIO_PIN(p)                      (((p) >> 0) & 0xFF)

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* GPIO configuration structure */
struct ra8p_gpio_config_s
{
  uint8_t port;                       /* GPIO port (0-13, or 'A'-'D') */
  uint8_t pin;                        /* GPIO pin (0-15) */
  bool input;                         /* Direction: true=input, false=output */
  bool initial_value;                 /* Initial value for output (true=high, false=low) */
  bool pullup;                        /* Enable pull-up */
  bool pulldown;                      /* Enable pull-down */
  bool high_drive;                    /* High drive strength */
  bool interrupt_enabled;             /* Interrupt enabled */
  uint8_t interrupt_trigger;          /* Interrupt trigger (0=none, 1=rising, 2=falling, 3=both) */
  bool analog_mode;                   /* Analog mode */
  bool open_drain;                    /* Open drain mode */
  bool debounce_enabled;              /* Debounce enabled */
  uint8_t debounce_cycles;            /* Debounce clock cycles */
};

/* GPIO interrupt callback function type */
typedef void (*ra8p_gpio_handler_t)(void *arg);

/* GPIO interrupt configuration structure */
struct ra8p_gpio_intconfig_s
{
  uint8_t port;                       /* GPIO port */
  uint8_t pin;                        /* GPIO pin */
  uint8_t trigger;                    /* Interrupt trigger type */
  ra8p_gpio_handler_t handler;        /* Interrupt handler */
  void *arg;                          /* Handler argument */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_gpio_config
 *
 * Description:
 *   Configure a GPIO pin based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   pinset - GPIO pinset configuration
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_config(gpio_pinset_t pinset);

/****************************************************************************
 * Name: ra8p_gpio_write
 *
 * Description:
 *   Write a value to a GPIO pin based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   pinset - GPIO pinset configuration
 *   value - true for high, false for low
 *
 * Returned Value:
 *   None
 *
 ****************************************************************************/

void ra8p_gpio_write(gpio_pinset_t pinset, bool value);

/****************************************************************************
 * Name: ra8p_gpio_read
 *
 * Description:
 *   Read a value from a GPIO pin based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   pinset - GPIO pinset configuration
 *
 * Returned Value:
 *   true for high, false for low
 *
 ****************************************************************************/

bool ra8p_gpio_read(gpio_pinset_t pinset);

/****************************************************************************
 * Name: ra8p_gpio_set_direction
 *
 * Description:
 *   Set GPIO pin direction based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *   input - true for input, false for output
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_set_direction(uint8_t port, uint8_t pin, bool input);

/****************************************************************************
 * Name: ra8p_gpio_set_value
 *
 * Description:
 *   Set GPIO pin value based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *   value - true for high, false for low
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_set_value(uint8_t port, uint8_t pin, bool value);

/****************************************************************************
 * Name: ra8p_gpio_get_value
 *
 * Description:
 *   Get GPIO pin value based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *
 * Returned Value:
 *   true for high, false for low
 *
 ****************************************************************************/

bool ra8p_gpio_get_value(uint8_t port, uint8_t pin);

/****************************************************************************
 * Name: ra8p_gpio_port_read
 *
 * Description:
 *   Read all pins of a GPIO port based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *
 * Returned Value:
 *   Port value
 *
 ****************************************************************************/

uint16_t ra8p_gpio_port_read(uint8_t port);

/****************************************************************************
 * Name: ra8p_gpio_port_write
 *
 * Description:
 *   Write all pins of a GPIO port based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   value - Port value
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_port_write(uint8_t port, uint16_t value);

/****************************************************************************
 * Name: ra8p_gpio_enable_pullup
 *
 * Description:
 *   Enable/disable pull-up resistor for GPIO pin based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_enable_pullup(uint8_t port, uint8_t pin, bool enable);

/****************************************************************************
 * Name: ra8p_gpio_enable_pulldown
 *
 * Description:
 *   Enable/disable pull-down resistor for GPIO pin based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_enable_pulldown(uint8_t port, uint8_t pin, bool enable);

/****************************************************************************
 * Name: ra8p_gpio_set_drive_strength
 *
 * Description:
 *   Set GPIO pin drive strength based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *   high - true for high drive, false for low drive
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_set_drive_strength(uint8_t port, uint8_t pin, bool high);

/****************************************************************************
 * Name: ra8p_gpio_enable_interrupt
 *
 * Description:
 *   Enable/disable GPIO interrupt based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *   trigger - Interrupt trigger type (0=none, 1=rising, 2=falling, 3=both)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_enable_interrupt(uint8_t port, uint8_t pin, uint8_t trigger);

/****************************************************************************
 * Name: ra8p_gpio_disable_interrupt
 *
 * Description:
 *   Disable GPIO interrupt based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_disable_interrupt(uint8_t port, uint8_t pin);

/****************************************************************************
 * Name: ra8p_gpio_clear_interrupt
 *
 * Description:
 *   Clear GPIO interrupt flag based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_clear_interrupt(uint8_t port, uint8_t pin);

/****************************************************************************
 * Name: ra8p_gpio_is_enabled
 *
 * Description:
 *   Check if GPIO is enabled based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *
 * Returned Value:
 *   true if enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_gpio_is_enabled(uint8_t port);

/****************************************************************************
 * Name: ra8p_gpio_is_output
 *
 * Description:
 *   Check if GPIO pin is configured as output based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *
 * Returned Value:
 *   true if output, false if input
 *
 ****************************************************************************/

bool ra8p_gpio_is_output(uint8_t port, uint8_t pin);

/****************************************************************************
 * Name: ra8p_gpio_set_output_type
 *
 * Description:
 *   Set GPIO output type (push-pull or open-drain) based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *   open_drain - true for open drain, false for push-pull
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_set_output_type(uint8_t port, uint8_t pin, bool open_drain);

/****************************************************************************
 * Name: ra8p_gpio_set_analog_mode
 *
 * Description:
 *   Set GPIO pin to analog mode based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *   enable - true for analog mode, false for digital
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_set_analog_mode(uint8_t port, uint8_t pin, bool enable);

/****************************************************************************
 * Name: ra8p_gpio_get_port_base
 *
 * Description:
 *   Get base address for a GPIO port based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *
 * Returned Value:
 *   Base address of port registers
 *
 ****************************************************************************/

uint32_t ra8p_gpio_get_port_base(uint8_t port);

/****************************************************************************
 * Name: ra8p_gpio_set_initial_value
 *
 * Description:
 *   Set initial value for GPIO output pin based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *   value - Initial value (true=high, false=low)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_set_initial_value(uint8_t port, uint8_t pin, bool value);

/****************************************************************************
 * Name: ra8p_gpio_is_analog
 *
 * Description:
 *   Check if GPIO pin is in analog mode based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *
 * Returned Value:
 *   true if in analog mode, false if in digital mode
 *
 ****************************************************************************/

bool ra8p_gpio_is_analog(uint8_t port, uint8_t pin);

/****************************************************************************
 * Name: ra8p_gpio_set_debounce
 *
 * Description:
 *   Set GPIO debounce for input pin based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *   cycles - Debounce clock cycles (0-15)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_set_debounce(uint8_t port, uint8_t pin, uint8_t cycles);

/****************************************************************************
 * Name: ra8p_gpio_attach_interrupt
 *
 * Description:
 *   Attach GPIO interrupt handler based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *   handler - Interrupt handler function
 *   arg - Handler argument
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_attach_interrupt(uint8_t port, uint8_t pin, 
                               ra8p_gpio_handler_t handler, void *arg);

/****************************************************************************
 * Name: ra8p_gpio_detach_interrupt
 *
 * Description:
 *   Detach GPIO interrupt handler based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_detach_interrupt(uint8_t port, uint8_t pin);

/****************************************************************************
 * Name: ra8p_gpio_set_function
 *
 * Description:
 *   Set GPIO pin function (GPIO or alternate) based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *   func - Function selection (0=GPIO, 1-31=peripheral)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_set_function(uint8_t port, uint8_t pin, uint8_t func);

/****************************************************************************
 * Name: ra8p_gpio_is_interrupt_enabled
 *
 * Description:
 *   Check if GPIO interrupt is enabled based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *
 * Returned Value:
 *   true if interrupt enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_gpio_is_interrupt_enabled(uint8_t port, uint8_t pin);

/****************************************************************************
 * Name: ra8p_gpio_get_interrupt_trigger
 *
 * Description:
 *   Get current GPIO interrupt trigger configuration based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *
 * Returned Value:
 *   Trigger configuration (0=none, 1=rising, 2=falling, 3=both)
 *
 ****************************************************************************/

uint8_t ra8p_gpio_get_interrupt_trigger(uint8_t port, uint8_t pin);

/****************************************************************************
 * Name: ra8p_gpio_get_config
 *
 * Description:
 *   Get current GPIO pin configuration based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *   config - Pointer to configuration structure to fill
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_get_config(uint8_t port, uint8_t pin, struct ra8p_gpio_config_s *config);

/****************************************************************************
 * Name: ra8p_gpio_reset_port
 *
 * Description:
 *   Reset GPIO port to default state based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_reset_port(uint8_t port);

/****************************************************************************
 * Name: ra8p_gpio_set_pullup_pulldown
 *
 * Description:
 *   Set both pull-up and pull-down for GPIO pin based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *   pullup - true to enable pull-up
 *   pulldown - true to enable pull-down
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_set_pullup_pulldown(uint8_t port, uint8_t pin, bool pullup, bool pulldown);

/****************************************************************************
 * Name: ra8p_gpio_get_drive_strength
 *
 * Description:
 *   Get current GPIO pin drive strength based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *
 * Returned Value:
 *   true if high drive, false if low drive
 *
 ****************************************************************************/

bool ra8p_gpio_get_drive_strength(uint8_t port, uint8_t pin);

/****************************************************************************
 * Name: ra8p_gpio_get_function
 *
 * Description:
 *   Get current GPIO pin function based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *
 * Returned Value:
 *   Function selection (0=GPIO, 1-31=peripheral)
 *
 ****************************************************************************/

uint8_t ra8p_gpio_get_function(uint8_t port, uint8_t pin);

/****************************************************************************
 * Name: ra8p_gpio_set_port_mode
 *
 * Description:
 *   Set GPIO port mode (multiple pins at once) based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   mask - Bit mask for pins to configure
 *   func - Function to set for selected pins
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_set_port_mode(uint8_t port, uint16_t mask, uint8_t func);

/****************************************************************************
 * Name: ra8p_gpio_port_get_config
 *
 * Description:
 *   Get GPIO port configuration for multiple pins based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   mask - Bit mask for pins to query
 *   config - Array to fill with configuration data (one per pin)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_port_get_config(uint8_t port, uint16_t mask, 
                              struct ra8p_gpio_config_s *config);

/****************************************************************************
 * Name: ra8p_gpio_get_interrupt_status
 *
 * Description:
 *   Get GPIO interrupt status flags based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *
 * Returned Value:
 *   true if interrupt occurred, false otherwise
 *
 ****************************************************************************/

bool ra8p_gpio_get_interrupt_status(uint8_t port, uint8_t pin);

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_GPIO_H */