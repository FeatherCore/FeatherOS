/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_elc.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_ELC_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_ELC_H_

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* ELC (Event Link Controller) Base Address */

#define RA8P_ELC_BASE                          (0x4000e000)

/* ELC Register Offsets */

/* Event Link Control Register (ELCCR) */

#define RA8P_ELC_ELCCR_OFFSET                 (0x000)
#define RA8P_ELC_ELCCR_ELCEN                (1 << 0)    /* Bit 0: ELC Enable */
#define RA8P_ELC_ELCCR_ELCRST               (1 << 1)    /* Bit 1: ELC Reset */

/* Event Link Status Register (ELCSR) */

#define RA8P_ELC_ELCSR_OFFSET                 (0x004)
#define RA8P_ELC_ELCSR_ELCBSY              (1 << 0)    /* Bit 0: ELC Busy */

/* Event Link Table Register (ELCTBL) */

#define RA8P_ELC_ELCTBL_OFFSET                (0x008)
#define RA8P_ELC_ELCTBL_EVTSEL_MASK          (0xFF)      /* Bits 0-7: Event Select */
#define RA8P_ELC_ELCTBL_EVTSEL_SHIFT         (0)
#define RA8P_ELC_ELCTBL_ModSELR_MASK         (0xFF00)    /* Bits 8-15: Module Select R */
#define RA8P_ELC_ELCTBL_ModSELR_SHIFT        (8)

/* Event Link Trigger Register (ELCTRG) */

#define RA8P_ELC_ELCTRG_OFFSET                (0x00C)
#define RA8P_ELC_ELCTRG_TRG                  (1 << 0)    /* Bit 0: Trigger */

/* Event Link Interrupt Enable Register (ELCIER) */

#define RA8P_ELC_ELCIER_OFFSET                (0x010)
#define RA8P_ELC_ELCIER_EVTIE               (1 << 0)    /* Bit 0: Event Interrupt Enable */

/* Event Link Interrupt Status Register (ELCISR) */

#define RA8P_ELC_ELCISR_OFFSET                (0x014)
#define RA8P_ELC_ELCISR_EVTIF               (1 << 0)    /* Bit 0: Event Interrupt Flag */

/* Event Link Edge Control Register (ELCECR) */

#define RA8P_ELC_ELCECR_OFFSET                (0x018)
#define RA8P_ELC_ELCECR_ELCPE               (1 << 0)    /* Bit 0: Event Polarity Select */
#define RA8P_ELC_ELCECR_ELCET_MASK           (0x6)       /* Bits 1-2: Event Type */
#define RA8P_ELC_ELCECR_ELCET_SHIFT          (1)
#define RA8P_ELC_ELCECR_ELCET_RISING        (0x0 << 1)  /* Rising edge */
#define RA8P_ELC_ELCECR_ELCET_FALLING       (0x1 << 1)  /* Falling edge */
#define RA8P_ELC_ELCECR_ELCET_BOTH          (0x2 << 1)  /* Both edges */
#define RA8P_ELC_ELCECR_ELCET_LOW           (0x3 << 1)  /* Low level */
#define RA8P_ELC_ELCECR_ELCET_HIGH          (0x4 << 1)  /* High level */

/* Event Link Table Size Register (ELCTSZ) */

#define RA8P_ELC_ELCTSZ_OFFSET                (0x01C)
#define RA8P_ELC_ELCTSZ_ELCTSZ_MASK         (0xFF)      /* Bits 0-7: Table Size */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* Event number */

enum ra8p_elc_event_e
{
  RA8P_ELC_EVENT_NONE = 0,          /* No event */
  RA8P_ELC_EVENT_IRQ0,             /* IRQ0 event */
  RA8P_ELC_EVENT_IRQ1,             /* IRQ1 event */
  RA8P_ELC_EVENT_IRQ2,             /* IRQ2 event */
  /* ... up to IRQ31 */
  RA8P_ELC_EVENT_DMAC0,            /* DMAC0 event */
  RA8P_ELC_EVENT_DMAC1,            /* DMAC1 event */
  /* ... more events */
  RA8P_ELC_EVENT_GPT0_CMPA,        /* GPT0 Compare Match A */
  RA8P_ELC_EVENT_GPT0_CMPB,        /* GPT0 Compare Match B */
  /* ... more GPT events */
  RA8P_ELC_EVENT_RTC_ALARM,        /* RTC Alarm event */
  RA8P_ELC_EVENT_RTC_PERIOD,       /* RTC Periodic event */
  RA8P_ELC_EVENT_AGT0_CMP,          /* AGT0 Compare Match */
  RA8P_ELC_EVENT_AGT1_CMP,          /* AGT1 Compare Match */
  RA8P_ELC_EVENT_MAX = 255,          /* Maximum event number */
};

/* Module number */

enum ra8p_elc_module_e
{
  RA8P_ELC_MODULE_NONE = 0,        /* No module */
  RA8P_ELC_MODULE_DMAC0,          /* DMAC0 */
  RA8P_ELC_MODULE_DMAC1,          /* DMAC1 */
  RA8P_ELC_MODULE_GPT0,           /* GPT0 */
  RA8P_ELC_MODULE_GPT1,           /* GPT1 */
  /* ... more modules */
  RA8P_ELC_MODULE_ADC0,           /* ADC0 */
  RA8P_ELC_MODULE_ADC1,           /* ADC1 */
  RA8P_ELC_MODULE_RTC,            /* RTC */
  RA8P_ELC_MODULE_AGT0,           /* AGT0 */
  RA8P_ELC_MODULE_AGT1,           /* AGT1 */
  RA8P_ELC_MODULE_MAX = 255,        /* Maximum module number */
};

/* Event type/polarity */

enum ra8p_elc_event_type_e
{
  RA8P_ELC_TYPE_RISING = 0,        /* Rising edge */
  RA8P_ELC_TYPE_FALLING,       /* Falling edge */
  RA8P_ELC_TYPE_BOTH,           /* Both edges */
  RA8P_ELC_TYPE_LOW,            /* Low level */
  RA8P_ELC_TYPE_HIGH,           /* High level */
};

/* ELC event link configuration */

struct ra8p_elc_link_s
{
  enum ra8p_elc_event_e event;        /* Event number */
  enum ra8p_elc_module_e module;       /* Target module */
  enum ra8p_elc_event_type_e type;   /* Event type/polarity */
  bool interrupt_enable;                /* Enable interrupt on event */
};

/* ELC configuration */

struct ra8p_elc_config_s
{
  uint32_t base;                    /* ELC base address */
  int irq;                          /* ELC interrupt number */
  struct ra8p_elc_link_s links[32];  /* Event link table */
  uint8_t num_links;                 /* Number of configured links */
  bool initialized;                 /* True if initialized */
};

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_ELC_H */