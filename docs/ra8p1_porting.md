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
| CPU | ARM Cortex-M85 (主核) + ARM Cortex-M33 (副核) |
| 架构 | ARMv8.1-M Mainline |
| 最高频率 | 1 GHz (CM85) / 250 MHz (CM33) |
| DSP 扩展 | 是 |
| MVE-F | 是 (M-Profile Vector Extension) |
| FPU | FPv5-D16 (双精度浮点) |
| I-Cache | 32 字节缓存行 |
| D-Cache | 32 字节缓存行 |
| TrustZone | 是 |
| PMU | 8 事件计数器 |
| MRAM | 768 KB (CM85) + 256 KB (CM33) |
| SRAM | 1 MB (主SRAM) + 1404 KB (SRAM0) + 468 KB (SRAM1) |
| NPU | Ethos-U55 (可选) |
| GPIO | 14 个端口 (P0-PD) |
| UART | 10 个 SCI_B 通道 |
| SPI | 4 个 SPI_B 通道 |
| I2C | 3 个 IIC 通道 |
| PWM | 14 个 GPT 通道 |
| USB | 1 个 FS + 1 个 HS |
| CAN | 2 个 CANFD 通道 |
| SDHC | 2 个 SDHC 通道 |
| 显示 | GLCDC + MIPI DSI |

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
├── ra8p_gpio.c                # GPIO 驱动实现
├── ra8p_gpio.h                # GPIO 驱动 API
├── ra8p_sci_b.c               # SCI_B UART 驱动
├── ra8p_spi_b.c               # SPI_B 驱动
├── ra8p_icu.c                 # 中断控制器驱动
├── ra8p_dmac.c                # DMA 控制器驱动
├── ra8p_power.c               # 电源管理驱动
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
- 默认使用内部 RC 振荡器 (LOCO ~ 32.768 kHz)
- 外部晶振 (XTAL) 24 MHz
- PLL 配置可达 1 GHz (CM85) / 250 MHz (CM33)
- 外设时钟：PCLKA (125 MHz), PCLKB (62.5 MHz), PCLKC (125 MHz), PCLKD (250 MHz)

```c
void ra8p_clockconfig(void)
{
  /* RA8P starts with internal RC oscillator (LOCO ~ 32.768 kHz).
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

基于 Zephyr 设备树定义的外设内存映射：

| 外设 | 基地址 | 描述 |
|------|--------|------|
| Code Flash | 0x00000000 | 4 MB |
| SRAM | 0x20000000 | 2 MB |
| SRAM0 | 0x22000000 | 1404 KB |
| SRAM1 | 0x2215F000 | 468 KB |
| SDRAM | 0x68000000 | 64 MB (外部) |
| GPIO0-9 | 0x40400000+ | 14 个 GPIO 端口 |
| GPIOA-D | 0x40400140+ | 扩展 GPIO 端口 |
| SCI0-9 | 0x40358000+ | 10 个 UART 通道 |
| SPI0-3 | 0x4035C000+ | 4 个 SPI 通道 |
| IIC0-2 | 0x4025E000+ | 3 个 I2C 通道 |
| GPT0-13 | 0x40322000+ | 14 个 PWM 定时器 |
| USB FS | 0x40250000 | USB Full Speed |
| USB HS | 0x40351000 | USB High Speed |
| CANFD | 0x40380000 | CAN FD 控制器 |
| SDHC0-1 | 0x40252000+ | SD/SDIO 控制器 |
| GLCDC | 0x40342000 | 显示控制器 |
| MIPI DSI | 0x40346000 | MIPI DSI 接口 |
| NPU | 0x40140000 | Ethos-U55 NPU |
| I3C0 | 0x4035F000 | I3C 控制器 |
| RTC | 0x40202000 | 实时时钟 |
| WDT | 0x40202600 | 看门狗定时器 |
| DMAC | 0x4000A000 | DMA 控制器 |
| ICU | 0x40024000 | 中断控制器 |

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

外设中断 (16-95)：
```c
#define RA8P_IRQ_DMAC0         16   /* DMA Controller 0 */
#define RA8P_IRQ_SCI0_TXI0     28   /* SCI0 TX Interrupt */
#define RA8P_IRQ_SCI0_RXI0     29   /* SCI0 RX Interrupt */
#define RA8P_IRQ_SCI2_TXI2     36   /* SCI2 TX Interrupt */
#define RA8P_IRQ_SCI2_RXI2     37   /* SCI2 RX Interrupt */
#define RA8P_IRQ_SPI0          68   /* SPI0 */
#define RA8P_IRQ_IIC0          72   /* IIC0 */
#define RA8P_IRQ_USBFS         76   /* USB Full Speed */
#define RA8P_IRQ_USBHS         77   /* USB High Speed */
#define RA8P_IRQ_GPT0          78   /* GPT0 */
#define RA8P_IRQ_CANFD         94   /* CANFD */
#define RA8P_IRQ_I3C0          95   /* I3C0 */
```

端口中断 (0-31)：
```c
#define RA8P_IRQ_PORT0         0    /* Port IRQ 0 */
#define RA8P_IRQ_PORT1         1    /* Port IRQ 1 */
...
#define RA8P_IRQ_PORT31        31   /* Port IRQ 31 */
```

### 6. GPIO 实现

**`ra8p_gpio.c`**

支持 14 个 GPIO 端口 (P0-PD)，每个端口最多 16 个引脚：

```c
/* GPIO 端口基地址 */
static const uintptr_t g_port_bases[] =
{
  RA8P_GPIO0_BASE,   /* Port 0 */
  RA8P_GPIO1_BASE,   /* Port 1 */
  ...
  RA8P_GPIOD_BASE,   /* Port D (13) */
};

/* GPIO 配置 API */
int ra8p_gpio_config(gpio_pinset_t pinset);
void ra8p_gpio_write(gpio_pinset_t pinset, bool value);
bool ra8p_gpio_read(gpio_pinset_t pinset);
```

### 7. 外设支持

#### 7.1 SCI_B UART

支持 10 个 SCI_B UART 通道 (SCI0-SCI9)，每个通道支持：
- 异步串行通信
- 可配置波特率
- 中断驱动传输

#### 7.2 SPI_B

支持 4 个 SPI_B 通道 (SPI0-SPI3)，特性：
- 主/从模式
- 可配置时钟极性和相位
- DMA 支持

#### 7.3 IIC (I2C)

支持 3 个 IIC 通道 (IIC0-IIC2)，特性：
- 主/从模式
- 7位/10位地址
- 高速模式 (400 kHz)

#### 7.4 GPT PWM

支持 14 个 GPT PWM 通道 (GPT0-GPT13)，特性：
- 32位定时器
- 可配置频率和占空比
- 多种计数模式

#### 7.5 USB

支持 USB FS 和 USB HS：
- USB FS: 全速 (12 Mbps)
- USB HS: 高速 (480 Mbps)
- Device 和 Host 模式

#### 7.6 CANFD

支持 2 个 CANFD 通道：
- CAN 2.0 和 CAN FD 协议
- 高达 8 Mbps 数据速率

#### 7.7 SDHC

支持 2 个 SDHC 通道：
- SD/SDIO/SDHC 卡支持
- 4位数据总线
- 高达 52 MHz 时钟

#### 7.8 显示

- GLCDC: 图形显示控制器
- MIPI DSI: 高速显示接口

#### 7.9 NPU

- Ethos-U55: AI/ML 加速器 (可选)

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
| GPIO | ✅ 完成 | 14 个端口 (P0-PD) |
| SCI_B UART | ✅ 基础支持 | 10 个通道，SCI2 默认控制台 |
| SPI_B | 🔄 待完善 | 4 个通道 |
| IIC (I2C) | 🔄 待完善 | 3 个通道 |
| GPT PWM | 🔄 待实现 | 14 个通道 |
| USB FS/HS | 🔄 待实现 | Device/Host 模式 |
| CANFD | 🔄 待实现 | 2 个通道 |
| SDHC | 🔄 待实现 | 2 个通道 |
| I3C | 🔄 待实现 | 1 个通道 |
| Ethernet (RMAC) | 🔄 待实现 | |
| GLCDC | 🔄 待实现 | 显示控制器 |
| MIPI DSI | 🔄 待实现 | 显示接口 |
| RTC | 🔄 待实现 | 实时时钟 |
| WDT | 🔄 待实现 | 看门狗定时器 |
| ADC | 🔄 待实现 | |
| DAC | 🔄 待实现 | |
| DMAC | 🔄 待完善 | 8 通道 DMA |
| NPU (Ethos-U55) | 🔄 待实现 | AI/ML 加速器 |
| SDRAMC | 🔄 待实现 | SDRAM 控制器 |

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
#       GPIO Support
#       SPI_B Support
#       IIC Support
#       ...

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
        ├── ek_ra8p1_bringup.c
        ├── ek_ra8p1_boot.c
        ├── ek_ra8p1_leds.c
        └── ek_ra8p1_buttons.c
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
   - PWM 输出测试
   - USB 设备测试
   - SD 卡读写测试

## 已知问题

1. **时钟配置**: 当前使用默认配置，需要添加 PLL 配置支持
2. **引脚配置**: 需要完善端口复用器配置
3. **中断处理**: 需要添加完整的中断处理程序
4. **双核支持**: 需要添加 CM33 核心的启动和通信支持

## 参考资料

1. [Renesas RA8P1 数据手册](https://www.renesas.com/ra8p1)
2. [ARM Cortex-M85 技术参考手册](https://developer.arm.com/Processors/Cortex-M85)
3. [ARMv8-M 架构参考手册](https://developer.arm.com/documentation/ddi0553/latest)
4. [Zephyr RA8P1 实现](https://github.com/zephyrproject-rtos/zephyr/tree/main/soc/renesas/ra/ra8p1)
5. [NuttX ARMv8-M 架构](https://nuttx.apache.org/docs/latest/)
6. [EK-RA8P1 开发板用户手册](https://www.renesas.com/ek-ra8p1)

## 变更历史

| 日期 | 描述 |
|------|------|
| 2026-04-23 | 初始移植，基于 Zephyr RA8P1 实现 |
| 2026-04-23 | 添加启动代码、时钟配置、内存映射、中断定义 |
| 2026-04-24 | 补充 Zephyr RA8P1 实现细节（设备树、Kconfig、电源管理） |
| 2026-04-24 | 更新 NuttX 代码风格示例（Apache 2.0 许可证头、函数注释、寄存器访问宏） |
| 2026-04-24 | 添加 Zephyr 源码参考路径清单 |
| 2026-04-24 | 添加 NuttX 风格的电源管理代码示例 |
| 2026-04-29 | 完善内存映射，添加更多外设支持 |
| 2026-04-29 | 更新 GPIO 实现，支持 14 个端口 |
| 2026-04-29 | 添加完整的 Kconfig 配置选项 |
| 2026-04-29 | 添加双核、MIPI DSI、NPU 支持 |
| 2026-04-29 | 添加 pinctrl 驱动、CGC 时钟控制、GPT PWM 驱动 |
| 2026-05-01 | 添加 ICU 外部中断、RTC、USB、WDT、CANFD、SDHC 驱动框架 |

## 新增文件清单

基于 Zephyr RTOS RA8P1 实现，以下是新增的文件：

### 硬件抽象层 (HAL)

| 文件 | 描述 |
|------|------|
| `arch/arm/src/ra8p/hardware/ra8p_cgc.h` | CGC (时钟生成控制) 寄存器定义 |
| `arch/arm/src/ra8p/hardware/ra8p_pinctrl.h` | PFS (端口功能选择) 寄存器定义 |
| `arch/arm/src/ra8p/hardware/ra8p_gpt.h` | GPT (通用 PWM 定时器) 寄存器定义 |
| `arch/arm/src/ra8p/hardware/ra8p_iic.h` | IIC (I2C) 寄存器定义 |
| `arch/arm/src/ra8p/hardware/ra8p_icu.h` | ICU (外部中断控制器) 寄存器定义 |
| `arch/arm/src/ra8p/hardware/ra8p_rtc.h` | RTC (实时时钟) 寄存器定义 |
| `arch/arm/src/ra8p/hardware/ra8p_usb.h` | USB 控制器寄存器定义 |
| `arch/arm/src/ra8p/hardware/ra8p_wdt.h` | WDT (看门狗定时器) 寄存器定义 |
| `arch/arm/src/ra8p/hardware/ra8p_canfd.h` | CANFD (CAN FD) 寄存器定义 |
| `arch/arm/src/ra8p/hardware/ra8p_sdhc.h` | SDHC (SD卡控制器) 寄存器定义 |

### 驱动实现

| 文件 | 描述 |
|------|------|
| `arch/arm/src/ra8p/ra8p_cgc.c` | CGC 时钟控制驱动 |
| `arch/arm/src/ra8p/ra8p_pinctrl.c` | Pinmux/PFS 驱动 |
| `arch/arm/src/ra8p/ra8p_gpt.c` | GPT PWM 驱动 |
| `arch/arm/src/ra8p/ra8p_icu.c` | ICU 外部中断驱动 |
| `arch/arm/src/ra8p/ra8p_power.c` | 电源管理驱动 |

### Zephyr 到 NuttX 映射 (新增)

| Zephyr 路径 | NuttX 文件 | 描述 |
|--------------|-----------|------|
| `dts/arm/renesas/ra/ra8/ra8x2.dtsi` | `hardware/ra8p_memorymap.h` | 双核设备树映射 |
| `dts/arm/renesas/ra/ra8/r7ka8p1xf.dtsi` | `hardware/ra8p_cgc.h` | 时钟定义映射 |
| `boards/renesas/ek_ra8p1/ek_ra8p1-pinctrl.dtsi` | `ra8p_pinctrl.c` | 引脚配置映射 |
| `dts/arm/renesas/ra/ra8/ra8x1.dtsi` (GPT) | `ra8p_gpt.c` | GPT PWM 映射 |
| `drivers/misc/renesas_ra_external_interrupt/` | `ra8p_icu.c` | ICU 外部中断映射 |
| `drivers/rtc/rtc_renesas_ra.c` | `ra8p_rtc.h` | RTC 驱动框架映射 |
| `drivers/usb/usb_dc_ra.c` | `ra8p_usb.h` | USB 驱动框架映射 |
| `drivers/watchdog/wdt_renesas_ra.c` | `ra8p_wdt.h` | WDT 驱动框架映射 |
| `drivers/can/can_renesas_ra.c` | `ra8p_canfd.h` | CANFD 驱动框架映射 |
| `drivers/sdhc/sdhc_renesas_ra.c` | `ra8p_sdhc.h` | SDHC 驱动框架映射 |
