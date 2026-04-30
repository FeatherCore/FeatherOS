/****************************************************************************
 * arch/arm/src/ra8p/ra8p_gpio.h
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

#ifndef __ARCH_ARM_SRC_RA8P_RA8P_GPIO_H
#define __ARCH_ARM_SRC_RA8P_RA8P_GPIO_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

#include <stdint.h>
#include <stdbool.h>

#include "hardware/ra8p_gpio.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* GPIO Port Base Addresses */
#define RA8P_GPIO_PORT0_BASE             RA8P_GPIO_PORT0_BASE
#define RA8P_GPIO_PORT1_BASE             RA8P_GPIO_PORT1_BASE
#define RA8P_GPIO_PORT2_BASE             RA8P_GPIO_PORT2_BASE
#define RA8P_GPIO_PORT3_BASE             RA8P_GPIO_PORT3_BASE
#define RA8P_GPIO_PORT4_BASE             RA8P_GPIO_PORT4_BASE
#define RA8P_GPIO_PORT5_BASE             RA8P_GPIO_PORT5_BASE
#define RA8P_GPIO_PORT6_BASE             RA8P_GPIO_PORT6_BASE
#define RA8P_GPIO_PORT7_BASE             RA8P_GPIO_PORT7_BASE
#define RA8P_GPIO_PORT8_BASE             RA8P_GPIO_PORT8_BASE
#define RA8P_GPIO_PORT9_BASE             RA8P_GPIO_PORT9_BASE
#define RA8P_GPIO_PORTA_BASE             RA8P_GPIO_PORTA_BASE
#define RA8P_GPIO_PORTB_BASE             RA8P_GPIO_PORTB_BASE
#define RA8P_GPIO_PORTC_BASE             RA8P_GPIO_PORTC_BASE
#define RA8P_GPIO_PORTD_BASE             RA8P_GPIO_PORTD_BASE

/* GPIO Port Definitions */
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

/* GPIO Pin Definitions */
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

/* GPIO pin configuration bits */
#define GPIO_INPUT                       (0 << 0)   /* Input mode */
#define GPIO_OUTPUT                      (1 << 0)   /* Output mode */
#define GPIO_OUTPUT_ONE                  (1 << 1)   /* Output high level */
#define GPIO_OUTPUT_ZERO                 (0 << 1)   /* Output low level */
#define GPIO_PULLUP                      (1 << 2)   /* Pull-up enabled */
#define GPIO_PULLDOWN                    (1 << 3)   /* Pull-down enabled */
#define GPIO_OPEN_DRAIN                  (1 << 4)   /* Open drain mode */
#define GPIO_PUSH_PULL                   (0 << 4)   /* Push-pull mode */
#define GPIO_DRIVE_STRENGTH_LOW          (0 << 5)   /* Low drive strength */
#define GPIO_DRIVE_STRENGTH_HIGH         (1 << 5)   /* High drive strength */
#define GPIO_INTERRUPT                   (1 << 6)   /* Enable interrupt */

/* Maximum number of GPIO ports and pins */
#define RA8P_GPIO_NPORTS                 14         /* Port 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, A, B, C, D */
#define RA8P_GPIO_NPINS_PER_PORT         16         /* Pins 0-15 per port */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* RA8P1 GPIO pinset definition - combines port, pin, and configuration */
typedef uint32_t gpio_pinset_t;

/* GPIO configuration structure */
struct ra8p_gpio_config_s
{
  uint8_t port;                     /* GPIO port (0-13, or 'A'-'D') */
  uint8_t pin;                      /* GPIO pin (0-15) */
  bool direction;                   /* Direction: true=output, false=input */
  bool initial_value;               /* Initial value for output */
  bool pull_up;                     /* Pull-up enable for input */
  bool pull_down;                   /* Pull-down enable for input */
  bool open_drain;                  /* Open-drain mode */
  bool interrupt_enable;            /* Enable interrupt */
  uint8_t interrupt_trigger;        /* Interrupt trigger (0=low, 1=high, 2=rising, 3=falling, 4=both) */
  bool drive_strength;              /* Drive strength: true=high, false=low */
};

/* GPIO interrupt callback */
typedef void (*ra8p_gpio_callback_t)(uint8_t port, uint8_t pin, void *arg);

/* GPIO interrupt handler structure */
struct ra8p_gpio_handler_s
{
  uint8_t port;                     /* Port number */
  uint8_t pin;                      /* Pin number */
  ra8p_gpio_callback_t callback;   /* Callback function */
  void *arg;                       /* Callback argument */
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
 * Name: ra8p_gpio_get_value
 *
 * Description:
 *   Get GPIO pin value by port and pin number based on Nuttx GPIO driver implementation.
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
 * Name: ra8p_gpio_set_value
 *
 * Description:
 *   Set GPIO pin value by port and pin number based on Nuttx GPIO driver implementation.
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
 * Name: ra8p_gpio_set_interrupt
 *
 * Description:
 *   Enable/disable GPIO interrupt based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *   trigger - Interrupt trigger type (0=low, 1=high, 2=rising, 3=falling, 4=both)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_set_interrupt(uint8_t port, uint8_t pin, uint8_t trigger);

/****************************************************************************
 * Name: ra8p_gpio_attach_interrupt
 *
 * Description:
 *   Attach a GPIO interrupt handler based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *   handler - Interrupt handler
 *   arg - Handler argument
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_attach_interrupt(uint8_t port, uint8_t pin, 
                              ra8p_gpio_callback_t handler, void *arg);

/****************************************************************************
 * Name: ra8p_gpio_detach_interrupt
 *
 * Description:
 *   Detach a GPIO interrupt handler based on Nuttx GPIO driver implementation.
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
 *   Check if GPIO is initialized and enabled based on Nuttx GPIO driver implementation.
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
 * Name: ra8p_gpio_make_pinset
 *
 * Description:
 *   Create a GPIO pinset from port and pin based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *
 * Returned Value:
 *   Pinset value
 *
 ****************************************************************************/

gpio_pinset_t ra8p_gpio_make_pinset(uint8_t port, uint8_t pin);

/****************************************************************************
 * Name: ra8p_gpio_decode_pinset
 *
 * Description:
 *   Decode a GPIO pinset to port and pin based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   pinset - GPIO pinset
 *   port - Pointer to store port
 *   pin - Pointer to store pin
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_decode_pinset(gpio_pinset_t pinset, uint8_t *port, uint8_t *pin);

/****************************************************************************
 * Name: ra8p_gpio_enable_output
 *
 * Description:
 *   Enable output for GPIO pin based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_enable_output(uint8_t port, uint8_t pin);

/****************************************************************************
 * Name: ra8p_gpio_enable_input
 *
 * Description:
 *   Enable input for GPIO pin based on Nuttx GPIO driver implementation.
 *
 * Input Parameters:
 *   port - GPIO port (0-13, or 'A'-'D')
 *   pin - GPIO pin (0-15)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_gpio_enable_input(uint8_t port, uint8_t pin);

#endif /* __ARCH_ARM_SRC_RA8P_RA8P_GPIO_H */