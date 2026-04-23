/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_irq.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_IRQ_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_IRQ_H

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* ARM Cortex-M85 core interrupts (0-15) */
#define RA8P_IRQ_RESERVED      0
#define RA8P_IRQ_RESET         1
#define RA8P_IRQ_NMI           2
#define RA8P_IRQ_HARDFAULT     3
#define RA8P_IRQ_MEMFAULT      4
#define RA8P_IRQ_BUSFAULT      5
#define RA8P_IRQ_USAGEFAULT    6
#define RA8P_IRQ_RESERVED1     7
#define RA8P_IRQ_RESERVED2     8
#define RA8P_IRQ_RESERVED3     9
#define RA8P_IRQ_RESERVED4     10
#define RA8P_IRQ_SVCALL        11
#define RA8P_IRQ_DEBUGMON      12
#define RA8P_IRQ_RESERVED5     13
#define RA8P_IRQ_PENDSV        14
#define RA8P_IRQ_SYSTICK       15

/* RA8P peripheral interrupts (16-111) */
#define RA8P_IRQ_DMAC0         16
#define RA8P_IRQ_DMAC1         17
#define RA8P_IRQ_DMAC2         18
#define RA8P_IRQ_DMAC3         19
#define RA8P_IRQ_DMAC4         20
#define RA8P_IRQ_DMAC5         21
#define RA8P_IRQ_DMAC6         22
#define RA8P_IRQ_DMAC7         23
#define RA8P_IRQ_DTC_COMPLETE   24
#define RA8P_IRQ_DTC_ERROR     25
#define RA8P_IRQ_ICU           26
#define RA8P_IRQ_FCU           27
#define RA8P_IRQ_SCI0_TXI0     28
#define RA8P_IRQ_SCI0_RXI0     29
#define RA8P_IRQ_SCI0_TEI0     30
#define RA8P_IRQ_SCI0_ERI0     31
#define RA8P_IRQ_SCI1_TXI1     32
#define RA8P_IRQ_SCI1_RXI1     33
#define RA8P_IRQ_SCI1_TEI1     34
#define RA8P_IRQ_SCI1_ERI1     35
#define RA8P_IRQ_SCI2_TXI2     36
#define RA8P_IRQ_SCI2_RXI2     37
#define RA8P_IRQ_SCI2_TEI2     38
#define RA8P_IRQ_SCI2_ERI2     39
#define RA8P_IRQ_SCI3_TXI3     40
#define RA8P_IRQ_SCI3_RXI3     41
#define RA8P_IRQ_SCI3_TEI3     42
#define RA8P_IRQ_SCI3_ERI3     43
#define RA8P_IRQ_SCI4_TXI4     44
#define RA8P_IRQ_SCI4_RXI4     45
#define RA8P_IRQ_SCI4_TEI4     46
#define RA8P_IRQ_SCI4_ERI4     47
#define RA8P_IRQ_SCI5_TXI5     48
#define RA8P_IRQ_SCI5_RXI5     49
#define RA8P_IRQ_SCI5_TEI5     50
#define RA8P_IRQ_SCI5_ERI5     51
#define RA8P_IRQ_SCI6_TXI6     52
#define RA8P_IRQ_SCI6_RXI6     53
#define RA8P_IRQ_SCI6_TEI6     54
#define RA8P_IRQ_SCI6_ERI6     55
#define RA8P_IRQ_SCI7_TXI7     56
#define RA8P_IRQ_SCI7_RXI7     57
#define RA8P_IRQ_SCI7_TEI7     58
#define RA8P_IRQ_SCI7_ERI7     59
#define RA8P_IRQ_SCI8_TXI8     60
#define RA8P_IRQ_SCI8_RXI8     61
#define RA8P_IRQ_SCI8_TEI8     62
#define RA8P_IRQ_SCI8_ERI8     63
#define RA8P_IRQ_SCI9_TXI9     64
#define RA8P_IRQ_SCI9_RXI9     65
#define RA8P_IRQ_SCI9_TEI9     66
#define RA8P_IRQ_SCI9_ERI9     67
#define RA8P_IRQ_SPI0          68
#define RA8P_IRQ_SPI1          69
#define RA8P_IRQ_SPI2          70
#define RA8P_IRQ_SPI3          71
#define RA8P_IRQ_I2C0          72
#define RA8P_IRQ_I2C1          73
#define RA8P_IRQ_I2C2          74
#define RA8P_IRQ_I2C3          75
#define RA8P_IRQ_USBFS         76
#define RA8P_IRQ_USBHS         77
#define RA8P_IRQ_GPT0          78
#define RA8P_IRQ_GPT1          79
#define RA8P_IRQ_GPT2          80
#define RA8P_IRQ_GPT3          81
#define RA8P_IRQ_ADC0          82
#define RA8P_IRQ_ADC1          83
#define RA8P_IRQ_DAC0          84
#define RA8P_IRQ_DAC1          85
#define RA8P_IRQ_SDHI0         86
#define RA8P_IRQ_SDHI1         87

/* Number of peripheral interrupts */
#define NR_IRQS  CONFIG_RA8P_NR_IRQS

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_IRQ_H */
