/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_npu.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_NPU_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_NPU_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include "chip.h"

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Ethos-U55 NPU Base Address */

#define RA8P_NPU_BASE                          (0x40140000)

/* NPU Register Offsets */

/* NPU Control Register (NPUCR) */

#define RA8P_NPU_NPUCR_OFFSET                  (0x000)
#define RA8P_NPU_NPUCR_NPUE                   (1 << 0)    /* Bit 0: NPU Enable */
#define RA8P_NPU_NPUCR_NPURESET               (1 << 1)    /* Bit 1: NPU Reset */
#define RA8P_NPU_NPUCR_NPUINTEN               (1 << 2)    /* Bit 2: NPU Interrupt Enable */
#define RA8P_NPU_NPUCR_NPUDBGDIS              (1 << 3)    /* Bit 3: NPU Debug Disable */

/* NPU Status Register (NPUSR) */

#define RA8P_NPU_NPUSR_OFFSET                  (0x004)
#define RA8P_NPU_NPUSR_NPUSTATUS              (1 << 0)    /* Bit 0: NPU Status */
#define RA8P_NPU_NPUSR_NPUIDLE                (1 << 1)    /* Bit 1: NPU Idle */
#define RA8P_NPU_NPUSR_NPUINT                 (1 << 2)    /* Bit 2: NPU Interrupt */
#define RA8P_NPU_NPUSR_NPUERROR               (1 << 3)    /* Bit 3: NPU Error */

/* NPU Interrupt Enable Register (NPUINTENR) */

#define RA8P_NPU_NPUINTENR_OFFSET              (0x008)
#define RA8P_NPU_NPUINTENR_DONE               (1 << 0)    /* Bit 0: Operation Done Interrupt Enable */
#define RA8P_NPU_NPUINTENR_ERROR              (1 << 1)    /* Bit 1: Error Interrupt Enable */

/* NPU Interrupt Status Register (NPUINTSR) */

#define RA8P_NPU_NPUINTSR_OFFSET               (0x00C)
#define RA8P_NPU_NPUINTSR_DONE                (1 << 0)    /* Bit 0: Operation Done */
#define RA8P_NPU_NPUINTSR_ERROR               (1 << 1)    /* Bit 1: Error Occurred */

/* NPU Command Register (NPUCOMR) */

#define RA8P_NPU_NPUCOMR_OFFSET               (0x010)
#define RA8P_NPU_NPUCOMR_COMMAND_MASK         (0xFF)      /* Bits 0-7: Command */
#define RA8P_NPU_NPUCOMR_COMMAND_SHIFT        (0)
#define RA8P_NPU_NPUCOMR_CMD_EXECUTE          (0x01)      /* Execute command */
#define RA8P_NPU_NPUCOMR_CMD_RESET            (0x02)      /* Reset command */
#define RA8P_NPU_NPUCOMR_CMD_CONFIGURE        (0x03)      /* Configure command */

/* NPU Configuration Register (NPUCFGR) */

#define RA8P_NPU_NPUCFGR_OFFSET               (0x014)
#define RA8P_NPU_NPUCFGR_MACS_MASK            (0xF)       /* Bits 0-3: MAC Units */
#define RA8P_NPU_NPUCFGR_MACS_SHIFT           (0)
#define RA8P_NPU_NPUCFGR_MACS_256             (0x0)       /* 256 MAC units */
#define RA8P_NPU_NPUCFGR_MACS_512             (0x1)       /* 512 MAC units */
#define RA8P_NPU_NPUCFGR_MACS_1024            (0x2)       /* 1024 MAC units */
#define RA8P_NPU_NPUCFGR_PLANES_MASK          (0x30)      /* Bits 4-5: Planes */
#define RA8P_NPU_NPUCFGR_PLANES_SHIFT         (4)
#define RA8P_NPU_NPUCFGR_PLANES_1             (0x0 << 4)  /* 1 Plane */
#define RA8P_NPU_NPUCFGR_PLANES_2             (0x1 << 4)  /* 2 Planes */
#define RA8P_NPU_NPUCFGR_PLANES_4             (0x2 << 4)  /* 4 Planes */
#define RA8P_NPU_NPUCFGR_SHRAM_MASK           (0xC0)      /* Bits 6-7: SHRAM Size */
#define RA8P_NPU_NPUCFGR_SHRAM_SHIFT          (6)
#define RA8P_NPU_NPUCFGR_SHRAM_64KB           (0x0 << 6)  /* 64KB SHRAM */
#define RA8P_NPU_NPUCFGR_SHRAM_128KB          (0x1 << 6)  /* 128KB SHRAM */
#define RA8P_NPU_NPUCFGR_SHRAM_256KB          (0x2 << 6)  /* 256KB SHRAM */
#define RA8P_NPU_NPUCFGR_SHRAM_512KB          (0x3 << 6)  /* 512KB SHRAM */

/* NPU Input Buffer Register (NPUINBR) */

#define RA8P_NPU_NPUINBR_OFFSET               (0x018)
#define RA8P_NPU_NPUINBR_IBA_MASK             (0xFFFFFFFF) /* Bits 0-31: Input Buffer Address */

/* NPU Output Buffer Register (NPUOUTBR) */

#define RA8P_NPU_NPUOUTBR_OFFSET              (0x01C)
#define RA8P_NPU_NPUOUTBR_OBA_MASK            (0xFFFFFFFF) /* Bits 0-31: Output Buffer Address */

/* NPU Weight Buffer Register (NPUWEIBR) */

#define RA8P_NPU_NPUWEIBR_OFFSET              (0x020)
#define RA8P_NPU_NPUWEIBR_WBA_MASK            (0xFFFFFFFF) /* Bits 0-31: Weight Buffer Address */

/* NPU Control Buffer Register (NPUCTLBR) */

#define RA8P_NPU_NPUCTLBR_OFFSET              (0x024)
#define RA8P_NPU_NPUCTLBR_CBA_MASK            (0xFFFFFFFF) /* Bits 0-31: Control Buffer Address */

/* NPU Timing Register (NPUTIMR) */

#define RA8P_NPU_NPUTIMR_OFFSET               (0x028)
#define RA8P_NPU_NPUTIMR_CLKEN                (1 << 0)    /* Bit 0: Clock Enable */
#define RA8P_NPU_NPUTIMR_PWRGATE              (1 << 1)    /* Bit 1: Power Gate */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* NPU configuration */

enum ra8p_npu_macs_e
{
  RA8P_NPU_MACS_256 = 0,            /* 256 MAC units */
  RA8P_NPU_MACS_512,                /* 512 MAC units */
  RA8P_NPU_MACS_1024,               /* 1024 MAC units */
};

enum ra8p_npu_planes_e
{
  RA8P_NPU_PLANES_1 = 0,            /* 1 Plane */
  RA8P_NPU_PLANES_2,                /* 2 Planes */
  RA8P_NPU_PLANES_4,                /* 4 Planes */
};

enum ra8p_npu_shram_e
{
  RA8P_NPU_SHRAM_64KB = 0,          /* 64KB SHRAM */
  RA8P_NPU_SHRAM_128KB,             /* 128KB SHRAM */
  RA8P_NPU_SHRAM_256KB,             /* 256KB SHRAM */
  RA8P_NPU_SHRAM_512KB,             /* 512KB SHRAM */
};

struct ra8p_npu_config_s
{
  uint32_t base;                    /* NPU base address */
  int irq;                          /* NPU interrupt number */
  enum ra8p_npu_macs_e macs;        /* Number of MAC units */
  enum ra8p_npu_planes_e planes;    /* Number of planes */
  enum ra8p_npu_shram_e shram;      /* SHRAM size */
  void *input_buffer;               /* Input buffer address */
  void *output_buffer;              /* Output buffer address */
  void *weight_buffer;              /* Weight buffer address */
  void *control_buffer;             /* Control buffer address */
  bool clock_enable;                /* Enable clock */
  bool power_gate;                  /* Power gate control */
};

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_NPU_H */