# Renesas RA8P1 移植记录

> 基于 Zephyr RTOS 实现，为 FeatherOS/NuttX 添加 RA8P1 支持

## 概述

RA8P1 (R7KA8P1KFLCAC) 是 Renesas 基于 ARM Cortex-M85 的高性能微控制器系列。本文档记录了将其移植到 NuttX RTOS 的过程。

## NuttX 代码风格

本文档中所有 NuttX 代码遵循以下风格规范：

### 文件头

```c
/****************************************************************************
 * arch/arm/src/ra8p/xxx.c
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
```

### 函数注释

```c
/****************************************************************************
 * Name: ra8p_clockconfig
 *
 * Description:
 *   Configure clock settings based on Zephyr soc/renesas/ra/ra8p1/soc.c
 *
 * Reference: Zephyr soc/renesas/ra/ra8p1/power.c
 *
 ****************************************************************************/
```

### 寄存器访问

使用 `getreg32()`/`putreg32()` 宏：

```c
/* 读取 CPACR 寄存器 */
uint32_t cpacr = getreg32(RA8P_CPACR);

/* 写入寄存器 */
putreg32(cpacr | RA8P_CPACR_CP10_FULL | RA8P_CPACR_CP11_FULL,
         RA8P_CPACR);
```

### 命名约定

- 寄存器基址：`RA8P_SCI_B2_BASE`
- 寄存器偏移量：`RA8P_SCIF_SCR_OFFSET`
- 中断号：`RA8P_IRQ_SCI2_TXI`
- 时钟频率配置：`CONFIG_RA8P_ICLK_FREQUENCY`
- 电源状态：`RA8P_PM_STATE_SLEEP`

## 概述

RA8P1 (R7KA8P1KFLCAC) 是 Renesas 基于 ARM Cortex-M85 的高性能微控制器系列。本文档记录了将其移植到 NuttX RTOS 的过程。

## RA8P1 规格

| 特性 | 规格 |
|------|------|
| CPU | ARM Cortex-M85 |
| 架构 | ARMv8.1-M Mainline |
| 最高频率 | 1 GHz |
| DSP 扩展 | 是 |
| MVE-F | 是 (M-Profile Vector Extension) |
| FPU | FPv5-D16 (双精度浮点) |
| I-Cache | 32 字节缓存行 |
| D-Cache | 32 字节缓存行 |
| TrustZone | 是 |
| PMU | 8 事件计数器 |
| MRAM | 768 KB |
| SRAM | 1 MB |
| NPU | Ethos-U55 (可选) |

## 移植来源

移植基于 Zephyr RTOS 的 RA8P1 实现：

| Zephyr 路径 | 描述 |
|-------------|------|
| `soc/renesas/ra/ra8p1/` | SOC 配置 |
| `soc/renesas/ra/ra8p1/Kconfig` | RA8P1 Kconfig 定义 |
| `soc/renesas/ra/ra8p1/Kconfig.soc` | SOC 系列选择 |
| `soc/renesas/ra/ra8p1/Kconfig.defconfig` | 默认配置 |
| `soc/renesas/ra/ra8p1/power.c` | 电源管理实现 |
| `soc/renesas/ra/ra8p1/sections.ld` | 链接段定义 |
| `dts/arm/renesas/ra/ra8/` | 设备树定义 |
| `dts/arm/renesas/ra/ra8/r7ka8p1kflcac.dtsi` | RA8P1 设备树 |
| `dts/arm/renesas/ra/ra8/r7ka8p1kflcac_cm85.dtsi` | CM85 核心设备树 |
| `dts/arm/renesas/ra/ra8/r7ka8p1xf.dtsi` | RA8P1 系列基础设备树 |
| `dts/arm/renesas/ra/ra8/ra8x2.dtsi` | RA8x2 基础设备树 |
| `boards/renesas/ek_ra8p1/` | EK-RA8P1 开发板 |
| `boards/renesas/ek_ra8p1/ek_ra8p1.dtsi` | 开发板设备树 |
| `boards/renesas/ek_ra8p1/ek_ra8p1-pinctrl.dtsi` | 引脚配置 |
| `boards/renesas/ek_ra8p1/ek_ra8p1_r7ka8p1kflcac_cm85.dts` | CM85 核心配置 |
| `boards/renesas/ek_ra8p1/ek_ra8p1_r7ka8p1kflcac_cm85_defconfig` | 默认编译配置 |

### Zephyr RA8P1 Kconfig 参考

**`soc/renesas/ra/ra8p1/Kconfig`**:

```kconfig
config SOC_SERIES_RA8P1
    select ARM
    select CPU_HAS_ARM_SAU
    select CPU_HAS_ARM_MPU
    select CPU_HAS_FPU
    select FPU
    select CPU_CORTEX_M_HAS_SYSTICK
    select CPU_CORTEX_M_HAS_DWT
    select ARMV8_M_DSP
    select HAS_SWO
    select XIP
    select CLOCK_CONTROL_RENESAS_RA_CGC if CLOCK_CONTROL
    select HAS_RENESAS_RA_FSP
    select HAS_PM
    select SOC_RA_DYNAMIC_INTERRUPT_NUMBER
    select ARM_MPU
    select CPU_RA_HAS_DCACHE_WRITETHROUGH if CPU_CORTEX_M85

config SOC_R7KA8P1KFLCAC_CM85
    select CPU_CORTEX_M85
    select GPIO_RA_HAS_VBTICTLR

config SOC_R7KA8P1KFLCAC_CM33
    select CPU_CORTEX_M33
    select SOC_RA_SECOND_CORE_BUILD
```

**`soc/renesas/ra/ra8p1/Kconfig.defconfig`**:

```kconfig
if SOC_SERIES_RA8P1

config NUM_IRQS
    default 96

config SYS_CLOCK_HW_CYCLES_PER_SEC
    default $(dt_node_int_prop_int,$(DT_CPUCLK0_PATH),clock-frequency) if SOC_R7KA8P1KFLCAC_CM85

config CORTEX_M_SYSTICK
    default n if RENESAS_RA_ULPT_TIMER

config BUILD_OUTPUT_HEX
    default y

config CLOCK_CONTROL
    default y

config DCACHE
    default n

config CACHE_MANAGEMENT
    default n

if ETHOS_U
choice ETHOS_U_NPU_CONFIG
    default ETHOS_U55_256
endchoice
endif # ETHOS_U

endif # SOC_SERIES_RA8P1
```

### Zephyr RA8P1 设备树参考

**`dts/arm/renesas/ra/ra8/r7ka8p1kflcac.dtsi`** - 内存和外设定义:

```dts
/ {
    soc {
        /* MRAM (Code Flash) - CM85 核心 768KB */
        mram-controller@4013c000 {
            code_mram_cm85: mram@2000000 {
                compatible = "renesas,ra-nv-mram", "soc-nv-flash";
                reg = <0x2000000 DT_SIZE_K(768)>;
                write-block-size = <1>;
                erase-block-size = <32>;
            };

            /* CM33 核心 256KB */
            code_mram_cm33: mram@20c0000 {
                reg = <0x20c0000 DT_SIZE_K(256)>;
            };
        };

        /* SRAM - 1MB */
        sram0: memory@22000000 {
            compatible = "mmio-sram";
            reg = <0x22000000 DT_SIZE_M(1)>;
        };

        /* 额外 SRAM - 640KB */
        sram1: memory@22100000 {
            reg = <0x22100000 DT_SIZE_K(640)>;
        };

        /* LCD 控制器 */
        lcdif: display-controller@40342000 {
            compatible = "renesas,ra-glcdc";
            reg = <0x40342000 0x1454>;
            clocks = <&lcdclk MSTPC 4>;
        };

        /* MIPI DSI */
        mipi_dsi: dsihost@40346000 {
            compatible = "renesas,ra-mipi-dsi";
            reg = <0x40346000 0x2000>;
            clocks = <&lcdclk MSTPC 10>;
        };
    };
};
```

**`dts/arm/renesas/ra/ra8/r7ka8p1xf.dtsi`** - 时钟配置:

```dts
/ {
    clocks: clocks {
        /* 外部晶振 24MHz */
        xtal: clock-main-osc {
            compatible = "renesas,ra-cgc-external-clock";
            clock-frequency = <DT_FREQ_M(24)>;
        };

        /* HOCO - 48MHz */
        hoco: clock-hoco {
            compatible = "fixed-clock";
            clock-frequency = <DT_FREQ_M(48)>;
        };

        /* LOCO - 32.768kHz */
        loco: clock-loco {
            compatible = "fixed-clock";
            clock-frequency = <32768>;
        };

        /* PLL1 配置 */
        pll: pll {
            compatible = "renesas,ra-cgc-pll";
            clocks = <&xtal>;
            div = <3>;
            mul = <250 0>;  /* 24MHz * 250 / 3 = 2GHz */

            pllp: pllp {
                div = <2>;
                freq = <DT_FREQ_M(1000)>;  /* 1GHz CPU */
            };
            pllq: pllq {
                div = <6>;
                freq = <333333333>;
            };
            pllr: pllr {
                div = <5>;
                freq = <DT_FREQ_M(400)>;
            };
        };

        /* PLL2 配置 */
        pll2: pll2 {
            pll2p: pll2p {
                div = <4>;
                freq = <DT_FREQ_M(600)>;
            };
            pll2q: pll2q {
                div = <3>;
                freq = <DT_FREQ_M(800)>;
            };
            pll2r: pll2r {
                div = <5>;
                freq = <DT_FREQ_M(480)>;
            };
        };

        /* CPU 时钟 */
        cpuclk0: cpuclk0 {
            compatible = "renesas,ra-cgc-pclk";
            clock-frequency = <1000000000>;  /* 1GHz */
            div = <1>;
        };

        /* 外设时钟 */
        pclka: pclka { div = <8>; };   /* 125MHz */
        pclkb: pclkb { div = <16>; };  /* 62.5MHz */
        pclkc: pclkc { div = <8>; };
        pclkd: pclkd { div = <4>; };   /* 250MHz */
    };
};
```

### Zephyr EK-RA8P1 开发板配置

**`boards/renesas/ek_ra8p1/ek_ra8p1.dtsi`** - 开发板设备树:

```dts
/ {
    leds {
        compatible = "gpio-leds";
        led1: led1 { gpios = <&ioport6 0 GPIO_ACTIVE_HIGH>; };
        led2: led2 { gpios = <&ioport3 3 GPIO_ACTIVE_HIGH>; };
        led3: led3 { gpios = <&ioporta 7 GPIO_ACTIVE_HIGH>; };
    };

    buttons {
        compatible = "gpio-keys";
        button0: s1 { gpios = <&ioport0 9 (GPIO_PULL_UP | GPIO_ACTIVE_LOW)>; };
        button1: s2 { gpios = <&ioport0 8 (GPIO_PULL_UP | GPIO_ACTIVE_LOW)>; };
    };

    /* SDRAM - 64MB */
    sdram1: sdram@68000000 {
        compatible = "zephyr,memory-region", "mmio-sram";
        reg = <0x68000000 DT_SIZE_M(64)>;
        zephyr,memory-region = "SDRAM";
    };
};

/* SCI8 UART - 控制台 */
&sci8 {
    pinctrl-0 = <&sci8_default>;
    interrupts = <0 1>, <1 1>, <2 1>, <3 1>;
    interrupt-names = "rxi", "txi", "tei", "eri";
    status = "okay";

    uart8: uart {
        current-speed = <115200>;
        status = "okay";
    };
};

/* Ethernet */
&eth1 {
    phy-connection-type = "rgmii";
    local-mac-address = [74 90 50 01 02 03];
    phy-handle = <&phy0>;
    status = "okay";
};

&mdio1 {
    phy0: phy@0 {
        compatible = "maxlinear,gpy111";
        maxlinear,interface-type = "rgmii";
        reg = <0>;
    };
};

/* MRAM 分区 */
&code_mram_cm85 {
    partitions {
        boot_partition: partition@0 {
            reg = <0x0 DT_SIZE_K(64)>;  /* Bootloader */
        };
        slot0_partition: partition@10000 {
            reg = <0x10000 DT_SIZE_K(344)>;  /* Application */
        };
        slot1_partition: partition@66000 {
            reg = <0x66000 DT_SIZE_K(344)>;  /* OTA */
        };
        storage_partition: partition@bc000 {
            reg = <0xbc000 DT_SIZE_K(16)>;  /* Storage */
        };
    };
};
```

**`boards/renesas/ek_ra8p1/ek_ra8p1_r7ka8p1kflcac_cm85_defconfig`**:

```kconfig
# Enable GPIO
CONFIG_GPIO=y

# Enable Console
CONFIG_SERIAL=y
CONFIG_UART_CONSOLE=y
CONFIG_UART_INTERRUPT_DRIVEN=y
CONFIG_CONSOLE=y
```

### Zephyr RA8P1 电源管理

**`soc/renesas/ra/ra8p1/power.c`**:

```c
#include <zephyr/pm/pm.h>
#include <r_lpm.h>

static lpm_instance_ctrl_t pm_state_ctrl;

/* Runtime Idle 配置 */
const lpm_cfg_t pm_state_runtime_idle_cfg = {
    .low_power_mode = LPM_MODE_SLEEP,
    .standby_wake_sources = LPM_STANDBY_WAKE_SOURCE_ULP0U,
    .output_port_enable = LPM_OUTPUT_PORT_ENABLE_RETAIN,
    .io_port_state = LPM_IO_PORT_NO_CHANGE,
    .power_supply_state = LPM_POWER_SUPPLY_DEEP_STANDBY_MODE1,
    .ram_retention_cfg.ram_retention = (uint16_t)(0x7F),
    .ram_retention_cfg.tcm_retention = true,
};

/* Standby 配置 */
const lpm_cfg_t pm_state_standby_cfg = {
    .low_power_mode = LPM_MODE_STANDBY,
    .standby_wake_sources = LPM_STANDBY_WAKE_SOURCE_ULP0U,
    .ram_retention_cfg.ram_retention = (uint16_t)(0x7F),
    .ram_retention_cfg.tcm_retention = true,
};

void pm_state_set(enum pm_state state, uint8_t substate_id)
{
    switch (state) {
    case PM_STATE_RUNTIME_IDLE:
        R_LPM_Open(&pm_state_ctrl, &pm_state_runtime_idle_cfg);
        __disable_irq();
        __set_BASEPRI(0);
        __ISB();
        R_LPM_LowPowerModeEnter(&pm_state_ctrl);
        __enable_irq();
        break;

    case PM_STATE_STANDBY:
        R_LPM_Open(&pm_state_ctrl, &pm_state_standby_cfg);
        __disable_irq();
        R_LPM_LowPowerModeEnter(&pm_state_ctrl);
        __enable_irq();
        break;
    }
}

void pm_state_exit_post_ops(enum pm_state state, uint8_t substate_id)
{
    R_LPM_Close(&pm_state_ctrl);
    irq_unlock(0);
}
```

## 文件结构

### 新增目录结构

```
arch/arm/src/ra8p/
├── Kconfig                    # Kconfig 芯片选择配置
├── CMakeLists.txt             # CMake 构建配置
├── Make.defs                  # Makefile 源文件定义
├── chip.h                     # 芯片定义和中断数
├── ra8p_start.c               # 启动代码
├── ra8p_clockconfig.c         # 时钟配置实现
├── ra8p_clockconfig.h         # 时钟配置 API
├── ra8p_lowsetup.c            # 底层初始化 (UART 引脚)
├── ra8p_lowsetup.h            # 底层初始化 API
└── hardware/
    ├── ra8p_memorymap.h       # 外设内存映射
    └── ra8p_irq.h             # 中断号定义

include/arch/ra8p/
└── irq.h                      # IRQ 头文件入口
```

### 修改的文件

| 文件 | 修改内容 |
|------|---------|
| `arch/arm/Kconfig` | 添加 `ARCH_CHIP_RA8P` 芯片族选择 |

## 详细实现

### 1. Kconfig 配置

**`arch/arm/src/ra8p/Kconfig`**

```kconfig
config ARCH_CHIP_R7KA8P1KFLCAC
    bool "R7KA8P1KFLCAC (RA8P1)"
    select ARCH_CORTEXM85        # Cortex-M85 CPU
    select ARCH_HAVE_FPU         # FPU 支持
    select ARCH_HAVE_MPU         # MPU 支持
    select ARM_HAVE_MPU_UNIFIED  # 统一 MPU
    select ARM_HAVE_MVE          # MVE 向量扩展
    select ARMV8M_HAVE_ICACHE    # I-Cache
    select ARMV8M_HAVE_DCACHE    # D-Cache
    select ARMV8M_SYSTICK        # SysTick 定时器
```

**`arch/arm/Kconfig` 修改**

```kconfig
config ARCH_CHIP_RA8P
    bool "Renesas RA8P"
    select ARCH_HAVE_MPU
    select ARM_HAVE_MPU_UNIFIED
    select ARCH_HAVE_FPU
    select ARMV8M_HAVE_ICACHE
    select ARMV8M_HAVE_DCACHE
    ---help---
        Renesas RA8P1 Series (ARM Cortex-M85).
        ARMv8.1-M Mainline with DSP, MVE-F, FPU, I/D Cache.
```

### 2. 启动代码

**`ra8p_start.c`**

启动流程：
1. 清零 BSS 段
2. 复制数据段从 Flash 到 SRAM
3. 配置时钟 (`ra8p_clockconfig`)
4. 配置 FPU (`arm_fpuconfig`)
5. 配置串口引脚 (`ra8p_lowsetup`)
6. 使能 I-Cache 和 D-Cache
7. 启动 NuttX (`nx_start`)

关键代码：
```c
void __start(void)
{
  /* Clear .bss */
  for (dest = (uint32_t *)_sbss; dest < (uint32_t *)_ebss; )
    *dest++ = 0;

  /* Copy .data from FLASH to SRAM */
  for (src = (const uint32_t *)_eronly, dest = (uint32_t *)_sdata;
       dest < (uint32_t *)_edata;)
    *dest++ = *src++;

  /* Configure clocks and FPU */
  ra8p_clockconfig();
  arm_fpuconfig();

  /* Configure serial */
  ra8p_lowsetup();

  /* Enable caches */
  ra8p_icache_enable();
  ra8p_dcache_enable();

  /* Start NuttX */
  nx_start();
}
```

### 3. 时钟配置

**`ra8p_clockconfig.c`**

RA8P1 时钟系统：
- 默认使用内部 RC 振荡器 (LOCO ~ 4 MHz)
- 可配置 PLL 达到最高 1 GHz
- 外设时钟：PCLKA (120 MHz), PCLKB (60 MHz)

```c
void ra8p_clockconfig(void)
{
  /* RA8P starts with internal RC oscillator (LOCO ~ 4 MHz).
   * For now, keep default configuration set by boot ROM.
   */

#ifdef CONFIG_ARCH_FPU
  /* Ensure FPU is enabled */
  volatile uint32_t cpacr = getreg32(0xe000ed88);
  cpacr |= (0xf << 20);  /* Set CP10 and CP11 full access */
  putreg32(cpacr, 0xe000ed88);
#endif
}
```

### 4. 内存映射

**`hardware/ra8p_memorymap.h`**

| 外设 | 基地址 | 描述 |
|------|--------|------|
| Code Flash | 0x00000000 | 4 MB |
| SRAM | 0x20000000 | 2 MB |
| CCM SRAM | 0x21000000 | 16 KB |
| SCI_B0 | 0x40070000 | UART 0 |
| SCI_B1 | 0x40071000 | UART 1 |
| SCI_B2 | 0x40072000 | UART 2 (默认控制台) |
| SPI_B0 | 0x40080000 | SPI 0 |
| I2C_B0 | 0x40050000 | I2C 0 |
| Ethernet | 0x40110000 | RMAC |
| USB FS | 0x40090000 | USB Full Speed |
| USB HS | 0x40091000 | USB High Speed |
| LCD | 0x400cc000 | GLCDC |
| GPT0 | 0x40038000 | 通用 PWM 定时器 |
| OSTM0 | 0x40040000 | OS 定时器 |

### 5. 中断定义

**`hardware/ra8p_irq.h`**

核心中断 (0-15)：
```c
#define RA8P_IRQ_RESET         1    /* Reset */
#define RA8P_IRQ_NMI           2    /* NMI */
#define RA8P_IRQ_HARDFAULT     3    /* Hard Fault */
#define RA8P_IRQ_SVCALL        11   /* SVCall */
#define RA8P_IRQ_PENDSV        14   /* PendSV */
#define RA8P_IRQ_SYSTICK       15   /* SysTick */
```

外设中断 (16-111)：
```c
#define RA8P_IRQ_DMAC0         16   /* DMA Controller 0 */
#define RA8P_IRQ_SCI0_TXI0     28   /* SCI0 TX Interrupt */
#define RA8P_IRQ_SCI0_RXI0     29   /* SCI0 RX Interrupt */
#define RA8P_IRQ_SCI2_TXI2     36   /* SCI2 TX Interrupt */
#define RA8P_IRQ_SCI2_RXI2     37   /* SCI2 RX Interrupt */
#define RA8P_IRQ_SPI0          68   /* SPI0 */
#define RA8P_IRQ_I2C0          72   /* I2C0 */
#define RA8P_IRQ_GPT0          78   /* GPT0 */
```

### 6. 底层初始化

**`ra8p_lowsetup.c`**

配置 SCI_B UART 引脚：
- SCI_B2: P110 (TX), P111 (RX) - 默认控制台

```c
void ra8p_lowsetup(void)
{
#ifdef CONFIG_RA8P_SCI_B_UART2
  /* SCI_B2 is typically used as console on RA8P EK boards */
  /* Pin configuration via port multiplexer registers */
#endif
}
```

## Zephyr 到 NuttX 映射

| Zephyr 符号 | NuttX 符号 | 描述 |
|-------------|-----------|------|
| `SOC_SERIES_RA8P1` | `ARCH_CHIP_RA8P` | RA8P 系列 |
| `SOC_R7KA8P1KFLCAC_CM85` | `ARCH_CHIP_R7KA8P1KFLCAC` | RA8P1 CM85 核心 |
| `CPU_CORTEX_M85` | `ARCH_CORTEXM85` | Cortex-M85 CPU |
| `ARMV8_1_M_MAINLINE` | `ARCH_ARMV8M` | ARMv8.1-M 架构 |
| `ARMV8_1_M_MVEI` | `ARM_HAVE_MVE` | MVE-I 支持 |
| `ARMV8_1_M_MVEF` | `ARM_HAVE_MVE` | MVE-F 支持 |
| `ARMV8_M_DSP` | `ARM_HAVE_DSP` | DSP 扩展 |
| `CPU_HAS_ICACHE` | `ARMV8M_HAVE_ICACHE` | I-Cache |
| `CPU_HAS_DCACHE` | `ARMV8M_HAVE_DCACHE` | D-Cache |
| `NUM_IRQS` | `CONFIG_RA8P_NR_IRQS` | 中断数量 (96) |

## 工具链配置

### GCC 编译选项

```bash
-mtune=cortex-m85
-march=armv8.1-m.main+mve.fp+fp.dp+dsp
-mfpu=fpv5-d16
-mfloat-abi=hard
```

### LLVM/Clang 配置

```bash
-target thumbv8.1m.main
-mfpu=fp-armv8-fullfp16-d16
```

## 外设支持状态

| 外设 | 状态 | 备注 |
|------|------|------|
| SCI_B UART | ✅ 基础支持 | SCI_B2 默认控制台 |
| SPI_B | 🔄 待实现 | |
| I2C_B | 🔄 待实现 | |
| Ethernet (RMAC) | 🔄 待实现 | |
| USB FS/HS | 🔄 待实现 | |
| GPT (PWM) | 🔄 待实现 | |
| GPIO | 🔄 待实现 | |
| ADC | 🔄 待实现 | |
| DAC | 🔄 待实现 | |
| LCD (GLCDC) | 🔄 待实现 | |
| Ethos-U55 NPU | 🔄 待实现 | |

## 使用方法

### 配置和编译

```bash
cd nuttx

# 配置 RA8P1
make menuconfig

# 选择:
#   System Type ->
#     ARM MCU selection ->
#       Renesas RA8P
#     RA8P Chip Selection ->
#       R7KA8P1KFLCAC (RA8P1)
#     RA8P Peripheral Support ->
#       SCI_B UART 2 (控制台)

# 编译
make
```

### 预期目录结构

```
boards/arm/ra8p/
└── ek-ra8p1/                  # EK-RA8P1 开发板
    ├── Kconfig
    ├── include/
    │   └── board.h
    ├── scripts/
    │   └── flash.ld           # 链接脚本
    ├── configs/
    │   └── nsh/
    │       └── defconfig      # 默认配置
    └── src/
        ├── CMakeLists.txt
        └── ek_ra8p1_bringup.c
```

## 测试计划

1. **基础启动测试**
   - 验证启动代码执行
   - 验证控制台 UART 输出

2. **时钟测试**
   - 验证系统时钟频率
   - 验证外设时钟分频

3. **Cache 测试**
   - 验证 I-Cache 使能
   - 验证 D-Cache 使能

4. **外设测试**
   - UART 收发测试
   - GPIO 测试
   - SPI/I2C 测试

## 已知问题

1. **时钟配置**: 当前使用默认配置，需要添加 PLL 配置支持
2. **引脚配置**: 需要完善端口复用器配置
3. **中断处理**: 需要添加完整的中断处理程序

## 参考资料

1. [Renesas RA8P1 数据手册](https://www.renesas.com/ra8p1)
2. [ARM Cortex-M85 技术参考手册](https://developer.arm.com/Processors/Cortex-M85)
3. [ARMv8-M 架构参考手册](https://developer.arm.com/documentation/ddi0553/latest)
4. [Zephyr RA8P1 实现](https://github.com/zephyrproject-rtos/zephyr/tree/main/soc/renesas/ra/ra8p1)
5. [NuttX ARMv8-M 架构](https://nuttx.apache.org/docs/latest/)

## 变更历史

| 日期 | 描述 |
|------|------|
| 2026-04-23 | 初始移植，基于 Zephyr RA8P1 实现 |
| 2026-04-23 | 添加启动代码、时钟配置、内存映射、中断定义 |
| 2026-04-24 | 补充 Zephyr RA8P1 实现细节（设备树、Kconfig、电源管理） |

## Zephyr 参考文件清单

以下是从 Zephyr 移植到 NuttX 的文件对应关系：

### 配置文件

| Zephyr 文件 | NuttX 文件 | 说明 |
|------------|-----------|------|
| `soc/renesas/ra/ra8p1/Kconfig` | `arch/arm/src/ra8p/Kconfig` | SOC 芯片选择 |
| `soc/renesas/ra/ra8p1/Kconfig.soc` | - | 已在 NuttX Kconfig 中合并 |
| `soc/renesas/ra/ra8p1/Kconfig.defconfig` | - | 运行时配置，需后续支持 |
| `boards/renesas/ek_ra8p1/ek_ra8p1_r7ka8p1kflcac_cm85_defconfig` | `boards/` | 开发板默认配置 |

### 设备树定义

| Zephyr 文件 | NuttX 说明 |
|------------|-----------|
| `dts/arm/renesas/ra/ra8/r7ka8p1kflcac.dtsi` | 内存映射参考 |
| `dts/arm/renesas/ra/ra8/r7ka8p1kflcac_cm85.dtsi` | CM85 核心定义 |
| `dts/arm/renesas/ra/ra8/r7ka8p1xf.dtsi` | 时钟配置参考 |
| `boards/renesas/ek_ra8p1/ek_ra8p1.dtsi` | 开发板外设定义 |

### 源代码

| Zephyr 文件 | NuttX 文件 | 说明 |
|------------|-----------|------|
| `soc/renesas/ra/ra8p1/power.c` | - | 电源管理，待实现 |
| `soc/renesas/ra/ra8p1/sections.ld` | - | 链接脚本，待适配 |

### 待移植功能

以下 Zephyr 功能尚未移植到 NuttX：

1. **电源管理** (`power.c`)
   - 低功耗模式 (Sleep/Standby)
   - TCM/SRAM 保持配置

2. **时钟详细配置**
   - PLL1/PLL2 配置
   - 外设时钟分频器

3. **外设驱动**
   - SCI_UART 完整驱动
   - I2C/SPI 驱动
   - Ethernet RMAC 驱动
   - USB HS/FS 驱动
   - LCD/DSI 显示驱动

4. **开发板支持**
   - EK-RA8P1 完整板级支持
   - 引脚复用配置
   - GPIO 中断支持

## Zephyr RA8P1 外设定义

### CPU 和电源状态

**`dts/arm/renesas/ra/ra8/ra8x2.dtsi`** - CPU 定义:

```dts
cpus {
    #address-cells = <1>;
    #size-cells = <0>;

    /* CM85 核心 - 主 CPU */
    cpu0: cpu@0 {
        device_type = "cpu";
        compatible = "arm,cortex-m85";
        reg = <0>;
        cpu-power-states = <&stop0 &stop1>;

        mpu: mpu@e000ed90 {
            compatible = "arm,armv8.1m-mpu";
            reg = <0xe000ed90 0x40>;
        };
    };

    /* CM33 核心 - 协处理器 */
    cpu1: cpu@1 {
        device_type = "cpu";
        compatible = "arm,cortex-m33";
        reg = <1>;

        mpu: mpu@e000ed90 {
            compatible = "arm,armv8m-mpu";
            reg = <0xe000ed90 0x40>;
        };
    };

    power-states {
        stop0: state0 {
            compatible = "zephyr,power-state";
            power-state-name = "runtime-idle";
            min-residency-us = <100>;
        };

        stop1: state1 {
            compatible = "zephyr,power-state";
            power-state-name = "standby";
            min-residency-us = <5000>;
            exit-latency-us = <3000>;
        };
    };
};
```

### GPIO 端口定义

```dts
/* GPIO 端口 0-13 + A-D */
ioport0: gpio@40400000 {
    compatible = "renesas,ra-gpio-ioport";
    reg = <0x40400000 0x20>;
    port = <0>;
    gpio-controller;
    #gpio-cells = <2>;
    ngpios = <16>;
};

ioport1: gpio@40400020 { port = <1>; ngpios = <16>; };
ioport2: gpio@40400040 { port = <2>; ngpios = <16>; };
ioport3: gpio@40400060 { port = <3>; ngpios = <16>; };
ioport4: gpio@40400080 { port = <4>; ngpios = <16>; vbatts-pins = <2 3 4>; };
ioport5: gpio@404000a0 { port = <5>; ngpios = <16>; };
ioport6: gpio@404000c0 { port = <6>; ngpios = <16>; };
ioport7: gpio@404000e0 { port = <7>; ngpios = <16>; };
ioport8: gpio@40400100 { port = <8>; ngpios = <16>; };
ioport9: gpio@40400120 { port = <9>; ngpios = <16>; };
ioporta: gpio@40400140 { port = <10>; ngpios = <16>; };
ioportb: gpio@40400160 { port = <11>; ngpios = <8>; };
ioportc: gpio@40400180 { port = <12>; ngpios = <16>; };
ioportd: gpio@404001a0 { port = <13>; ngpios = <8>; };
```

### SCI (Serial Communication Interface) 定义

RA8P1 有 10 个 SCI 通道 (SCI0-SCI9)，每个支持 UART/I2C/SPI 模式：

```dts
/* SCI0 - 支持 UART/I2C/SPI */
sci0: sci0@40358000 {
    compatible = "renesas,ra-sci";
    reg = <0x40358000 0x100>;
    clocks = <&sciclk MSTPB 31>;

    uart {
        compatible = "renesas,ra8-uart-sci-b";
        channel = <0>;
    };

    i2c {
        compatible = "renesas,ra-i2c-sci-b";
        #address-cells = <1>;
        #size-cells = <0>;
        channel = <0>;
    };

    spi {
        compatible = "renesas,ra-spi-sci-b";
        #address-cells = <1>;
        #size-cells = <0>;
        channel = <0>;
        overrun-character = <0x00>;
    };
};

/* SCI1-SCI9 类似结构，地址递增 0x100 */
sci1: sci1@40358100 { ... };
sci2: sci2@40358200 { ... };
/* ... */
sci8: sci8@40358800 { ... };  /* EK-RA8P1 控制台 */
sci9: sci9@40358900 { ... };
```

### SPI 控制器定义

```dts
/* 独立 SPI 控制器 (非 SCI) */
spi0: spi@4035c000 {
    compatible = "renesas,ra8-spi-b";
    #address-cells = <1>;
    #size-cells = <0>;
    channel = <0>;
    clocks = <&pclka MSTPB 19>;
    reg = <0x4035c000 0x100>;
};

spi1: spi@4035c100 {
    compatible = "renesas,ra8-spi-b";
    channel = <1>;
    clocks = <&pclka MSTPB 18>;
    reg = <0x4035c100 0x100>;
};
```

### PWM (GPT) 定义

```dts
/* PWM0-13 - 通用 PWM 定时器 */
pwm0: pwm0@40322000 {
    compatible = "renesas,ra-pwm";
    divider = <RA_PWM_SOURCE_DIV_1>;
    channel = <RA_PWM_CHANNEL_0>;
    clocks = <&gptclk MSTPE 31>;
    reg = <0x40322000 0x100>;
    #pwm-cells = <3>;
};

/* PWM1-13 类似，地址递增 0x100 */
pwm1: pwm1@40322100 { channel = <RA_PWM_CHANNEL_1>; };
/* ... */
pwm12: pwm12@40322c00 { channel = <RA_PWM_CHANNEL_12>; };
pwm13: pwm13@40322d00 { channel = <RA_PWM_CHANNEL_13>; };
```

### Ethernet (RMAC) 定义

```dts
/* Ethernet Switch Module */
eswm: eswm@403c8000 {
    compatible = "renesas,ra-eswm";
    reg = <0x403c8000 0x19424>;
    clocks = <&iclk 0 0>,
             <&pclka MSTPC 30>,
             <&eswclk 0 0>,
             <&eswphyclk 0 0>,
             <&ethphyclk MSTPC 28>;
    clock-names = "gwcaclk", "pclk", "eswclk", "eswphyclk", "ethphyclk";

    /* Ethernet MAC 0 */
    eth0: ethernet_mac@403cb000 {
        compatible = "renesas,ra-ethernet-rmac";
        reg = <0x403cb000 0x530>;
        channel = <0>;
    };

    /* Ethernet MAC 1 */
    eth1: ethernet_mac@403cd000 {
        compatible = "renesas,ra-ethernet-rmac";
        reg = <0x403cd000 0x530>;
        channel = <1>;
    };

    /* MDIO 0/1 */
    mdio0: mdio@403cb000 {
        compatible = "renesas,ra-mdio-rmac";
        reg = <0x403cb000 0x4>, <0x403cb004 0x4>;
        reg-names = "mpsm", "mpic";
        #address-cells = <1>;
        #size-cells = <0>;
        channel = <0>;
    };

    mdio1: mdio@403cd000 {
        compatible = "renesas,ra-mdio-rmac";
        channel = <1>;
    };
};
```

### CAN-FD 定义

```dts
canfd_global: canfd_global@40380000 {
    compatible = "renesas,ra-canfd-global";
    clocks = <&pclka 0 0>, <&pclke 0 0>;
    clock-names = "opclk", "ramclk";
    dll-min-freq = <DT_FREQ_M(8)>;
    dll-max-freq = <DT_FREQ_M(80)>;
    reg = <0x40380000 0x4000>;

    canfd0: canfd0 {
        compatible = "renesas,ra-canfd";
        channel = <0>;
        clocks = <&canfdclk MSTPC 27>;
    };

    canfd1: canfd1 {
        compatible = "renesas,ra-canfd";
        channel = <1>;
        clocks = <&canfdclk MSTPC 26>;
    };
};
```

### 其他外设

```dts
/* I2C 控制器 (非 SCI) */
iic0: iic0@4025e000 {
    compatible = "renesas,ra-iic";
    channel = <0>;
    reg = <0x4025e000 0x100>;
    clocks = <&pclkb MSTPB 9>;
};

iic1: iic1@4025e100 { channel = <1>; };
iic2: iic2@4025e200 { channel = <2>; };

/* AGT - 异步通用定时器 */
agt0: agt@40221000 {
    compatible = "renesas,ra-agt";
    channel = <0>;
    reg = <0x40221000 0x100>;
    renesas,count-source = "AGT_CLOCK_LOCO";
    renesas,prescaler = <0>;
    renesas,resolution = <16>;
};

/* ULPT - 超低功耗定时器 */
ulpt0: ulpt@40220000 {
    compatible = "renesas,ra-ulpt";
    reg = <0x40220000 0x100>;
    channel = <0>;

    timer {
        compatible = "renesas,ra-ulpt-timer";
    };
};

/* I3C 控制器 */
i3c0: i3c@4035f000 {
    compatible = "renesas,ra-i3c";
    #address-cells = <3>;
    #size-cells = <0>;
    reg = <0x4035f000 0x3e8>;
    channel = <0>;
    clocks = <&pclka MSTPB 4>, <&i3cclk 0 0>;
};

/* I2S/SSIE 音频接口 */
i2s0: ssie@4025d000 {
    compatible = "renesas,ra-i2s-ssie";
    channel = <0>;
    reg = <0x4025d000 0x28>;
    clocks = <&pclkb MSTPC 8>;
    full-duplex;
};

/* SDRAM 控制器 */
sdram: sdram-controller@40003c00 {
    compatible = "renesas,ra-sdram";
    #address-cells = <1>;
    #size-cells = <0>;
    reg = <0x40003c00 0x54>;
};

/* TRNG - 真随机数生成器 */
trng: trng {
    compatible = "renesas,ra-rsip-e50d-trng";
};
```

## EK-RA8P1 引脚配置

**`boards/renesas/ek_ra8p1/ek_ra8p1-pinctrl.dtsi`**:

### UART 引脚配置

```dts
&pinctrl {
    /* SCI8 - 控制台 UART */
    sci8_default: sci8_default {
        group1 {
            /* TX - P13_2 */
            psels = <RA_PSEL(RA_PSEL_SCI_8, 13, 2)>;
            drive-strength = "medium";
        };

        group2 {
            /* RX - P13_3 */
            psels = <RA_PSEL(RA_PSEL_SCI_8, 13, 3)>;
        };
    };

    /* SCI9 - 备用 UART */
    sci9_default: sci9_default {
        group1 {
            /* TX - P2_9 */
            psels = <RA_PSEL(RA_PSEL_SCI_9, 2, 9)>;
            drive-strength = "medium";
        };

        group2 {
            /* RX - P2_8 */
            psels = <RA_PSEL(RA_PSEL_SCI_9, 2, 8)>;
        };
    };
};
```

### I2C 引脚配置

```dts
    /* SCI1 - I2C 模式 */
    sci1_default: sci1_default {
        group1 {
            /* SDA - P4_0, SCL - P4_1 */
            psels = <RA_PSEL(RA_PSEL_SCI_1, 4, 0)>,
                    <RA_PSEL(RA_PSEL_SCI_1, 4, 1)>;
            drive-strength = "medium";
            drive-open-drain;
        };
    };

    /* IIC1 - 独立 I2C */
    iic1_default: iic1_default {
        group1 {
            /* SCL1 - P5_12, SDA1 - P5_11 */
            psels = <RA_PSEL(RA_PSEL_I2C, 5, 12)>,
                    <RA_PSEL(RA_PSEL_I2C, 5, 11)>;
            drive-strength = "medium";
        };
    };
```

### SPI 引脚配置

```dts
    /* SPI1 */
    spi1_default: spi1_default {
        group1 {
            /* MISO - P1_0, MOSI - P1_1, RSPCK - P1_2, SSL - P1_3 */
            psels = <RA_PSEL(RA_PSEL_SPI, 1, 0)>,
                    <RA_PSEL(RA_PSEL_SPI, 1, 1)>,
                    <RA_PSEL(RA_PSEL_SPI, 1, 2)>,
                    <RA_PSEL(RA_PSEL_SPI, 1, 3)>;
        };
    };
```

### PWM 引脚配置

```dts
    /* PWM1 - GTIOC1A/B */
    pwm1_default: pwm1_default {
        group1 {
            /* GTIOC1A - P1_5 */
            psels = <RA_PSEL(RA_PSEL_GPT1, 1, 5)>;
        };

        group2 {
            /* GTIOC1B - P1_4 */
            psels = <RA_PSEL(RA_PSEL_GPT1, 1, 4)>;
        };
    };

    /* PWM12 - 摄像头时钟 */
    pwm12_default: pwm12_default {
        group1 {
            /* GTIOC12A - P5_1 */
            psels = <RA_PSEL(RA_PSEL_GPT1, 5, 1)>;
            drive-strength = "medium";
        };
    };
```

### Ethernet 引脚配置

```dts
    /* MDIO1 */
    mdio1_default: mdio1_default {
        group1 {
            /* MDC - P4_15, MDIO - P4_14 */
            psels = <RA_PSEL(RA_PSEL_ETH_MII, 4, 15)>,
                    <RA_PSEL(RA_PSEL_ETH_MII, 4, 14)>;
            drive-strength = "medium";
        };
    };

    /* ETH1 - RGMII 模式 */
    eth1_default: eth1_default {
        group1 {
            /* RGMII_TXD0-3, TX_CTL, TX_CLK */
            psels = <RA_PSEL(RA_PSEL_ETH_RGMII, 3, 7)>,  /* TXD0 */
                    <RA_PSEL(RA_PSEL_ETH_RGMII, 3, 6)>,  /* TXD1 */
                    <RA_PSEL(RA_PSEL_ETH_RGMII, 3, 5)>,  /* TXD2 */
                    <RA_PSEL(RA_PSEL_ETH_RGMII, 3, 4)>,  /* TXD3 */
                    <RA_PSEL(RA_PSEL_ETH_RGMII, 3, 10)>, /* TX_CTL */
                    <RA_PSEL(RA_PSEL_ETH_RGMII, 3, 9)>,  /* TX_CLK */
                    /* RGMII_RXD0-3, RX_CTL, RX_CLK */
                    <RA_PSEL(RA_PSEL_ETH_RGMII, 9, 6)>,  /* RXD0 */
                    <RA_PSEL(RA_PSEL_ETH_RGMII, 9, 7)>,  /* RXD1 */
                    <RA_PSEL(RA_PSEL_ETH_RGMII, 9, 8)>,  /* RXD2 */
                    <RA_PSEL(RA_PSEL_ETH_RGMII, 9, 9)>,  /* RXD3 */
                    <RA_PSEL(RA_PSEL_ETH_RGMII, 2, 6)>,  /* RX_CTL */
                    <RA_PSEL(RA_PSEL_ETH_RGMII, 9, 5)>;  /* RX_CLK */
            drive-strength = "high";
        };
    };
```

### USB 引脚配置

```dts
    /* USB HS */
    usbhs_default: usbhs_default {
        group1 {
            /* VBUS - P4_8 */
            psels = <RA_PSEL(RA_PSEL_USBHS, 4, 8)>;
            drive-strength = "high";
        };
    };

    /* USB FS */
    usbfs_default: usbfs_default {
        group1 {
            /* USB_DM - P8_15, USB_DP - P8_14, VBUS - P4_7 */
            psels = <RA_PSEL(RA_PSEL_USBFS, 8, 15)>,
                    <RA_PSEL(RA_PSEL_USBFS, 8, 14)>,
                    <RA_PSEL(RA_PSEL_USBFS, 4, 7)>;
            drive-strength = "high";
        };
    };
```

### SDRAM 引脚配置

```dts
    /* SDRAM - 32 位数据总线 */
    sdram_default: sdram_default {
        group1 {
            /* 地址线 A2-A16 */
            psels = <RA_PSEL(RA_PSEL_BUS, 10, 3)>,  /* A2 */
                    <RA_PSEL(RA_PSEL_BUS, 10, 2)>,  /* A3 */
                    /* ... 更多地址线 ... */
                    <RA_PSEL(RA_PSEL_BUS, 12, 15)>; /* A16 */
            drive-strength = "high";
        };

        group2 {
            /* SDRAM_SDCLK */
            psels = <RA_PSEL(RA_PSEL_BUS, 10, 15)>;
            drive-strength = "highspeed-high";
        };
    ```

## Zephyr 源码参考路径

```
zephyr/                                    # Zephyr RTOS 根目录
├── soc/renesas/ra/ra8p1/                    # RA8P1 SOC 实现
│   ├── Kconfig                           # SOC Kconfig (select CPU_CORTEX_M85)
│   ├── Kconfig.soc                       # SOC 系列选择
│   ├── Kconfig.defconfig                 # 默认配置
│   ├── CMakeLists.txt                     # CMake 构建
│   ├── power.c                          # 电源管理 (Sleep/Standby)
│   └── sections.ld                       # 链接段 (Option Setting OFS)
├── soc/renesas/ra/common/                   # RA 系列通用代码
│   ├── soc.c                            # soc_early_init_hook, soc_late_init_hook
│   ├── pinctrl_soc.h                    # 引脚控制定义
│   └── ram_sections.ld                   # RAM 段链接脚本
├── dts/arm/renesas/ra/ra8/               # RA8 设备树
│   ├── r7ka8p1kflcac.dtsi               # RA8P1 设备树 (MRAM, SRAM)
│   ├── r7ka8p1kflcac_cm85.dtsi          # CM85 核心配置
│   ├── r7ka8p1xf.dtsi                 # RA8P1 时钟定义
│   └── ra8x2.dtsi                       # 通用外设定义 (SCI, GPIO, SPI, I2C...)
├── boards/renesas/ek_ra8p1/                # EK-RA8P1 开发板
│   ├── ek_ra8p1.dtsi                  # 板级设备树
│   ├── ek_ra8p1-pinctrl.dtsi          # 引脚配置
│   ├── ek_ra8p1_r7ka8p1kflcac_cm85.dts # CM85 配置
│   └── ek_ra8p1_r7ka8p1kflcac_cm85_defconfig # 默认配置
└── drivers/                            # 驱动代码
    └── (具体外设驱动)
```

### Zephyr SoC 初始化代码参考

**`soc/renesas/ra/common/soc.c`**:

```c
void soc_early_init_hook(void)
{
  /* NMI handler setup */
  z_arm_nmi_set_handler(NMI_Handler);

  /* DCache enable */
  sys_cache_data_enable();

  /* ICache invalidate after .ram_from_flash init */
  sys_cache_instr_invd_all();
}

void soc_late_init_hook(void)
{
  /* Secondary core start if enabled */
  R_BSP_SecondaryCoreStart();
}
```

### Zephyr 电源管理参考

**`soc/renesas/ra/ra8p1/power.c`**:

```c
/* Sleep mode entry */
void pm_state_set(enum pm_state state, uint8_t substate_id)
{
  switch (state) {
  case PM_STATE_RUNTIME_IDLE:
    R_LPM_Open(&pm_state_ctrl, &pm_state_runtime_idle_cfg);
    __disable_irq();
    __set_BASEPRI(0);
    __ISB();
    R_LPM_LowPowerModeEnter(&pm_state_ctrl);
    __enable_irq();
    __ISB();
    break;

  case PM_STATE_STANDBY:
    R_LPM_Open(&pm_state_ctrl, &pm_state_standby_cfg);
    __disable_irq();
    __set_BASEPRI(0);
    __ISB();
    R_LPM_LowPowerModeEnter(&pm_state_ctrl);
    __enable_irq();
    __ISB();
    break;
  }
}
```

## 变更历史

| 日期 | 描述 |
|------|------|
| 2026-04-23 | 初始移植，基于 Zephyr RA8P1 实现 |
| 2026-04-23 | 添加启动代码、时钟配置、内存映射、中断定义 |
| 2026-04-24 | 补充 Zephyr RA8P1 实现细节（设备树、Kconfig、电源管理） |
| 2026-04-24 | 更新 NuttX 代码风格示例（Apache 2.0 许可证头、函数注释、寄存器访问宏） |
| 2026-04-24 | 添加 Zephyr 源码参考路径清单 |
| 2026-04-24 | 添加 NuttX 风格的电源管理代码示例 |
    };
```
