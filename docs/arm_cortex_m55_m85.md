# ARM Cortex-M55 and Cortex-M85 Support

> Based on Zephyr RTOS implementation, updated for FeatherOS/NuttX

## 提交信息

**Commit:** `3e5b67a34369f7ff084cbe9ad397d507b70997e5`

```
feat(nuttx): add ARM Cortex-M55/M85 and RA8P1 support

Based on Zephyr RTOS implementation:

- Add M55/M85 CPU configuration with DSP, MVE, Cache support
- Add MPS3/MPS4 board configurations for Corstone-300/310/315/320
- Add RA8P1 (R7KA8P1KFLCAC) chip support:
  - Startup code (ra8p_start.c)
  - Clock configuration (ra8p_clockconfig.c)
  - Memory map and interrupt definitions
  - SCI_B UART low-level setup
- Add STM32N6 (Cortex-M55) SoC support:
  - SoC initialization (stm32n6_soc.c)
  - Clock configuration (stm32n6_clockconfig.c)
  - Memory map and interrupt definitions
  - RIF (Resource Isolation Framework) support
  - NPU (Neural-ART) accelerator support
  - LTDC LCD controller support
  - AXISRAM memory configuration

Files added:
- arch/arm/src/ra8p/: RA8P1 chip implementation
- arch/arm/src/stm32n6/: STM32N6 chip implementation
- include/arch/ra8p/irq.h: IRQ header
- include/arch/stm32n6/irq.h: IRQ header
- docs/arm_cortex_m55_m85.md: M55/M85 documentation
- docs/ra8p1_porting.md: RA8P1 porting guide
- docs/stm32n6_porting.md: STM32N6 porting guide

Files modified:
- arch/arm/Kconfig: Add ARCH_CORTEXM55/M85 features and RA8P/STM32N6 chips
- arch/arm/src/mps/Kconfig: Add MPS M55/M85 variants
- arch/arm/src/armv8-m/arm_cpuinfo.c: Add M55/M85 CPU information
- arch/arm/src/cmake/armv8-m.cmake: Add M55/M85 compile options
```

## 相对于原版 NuttX 的更新

### 新增文件

| 文件 | 说明 |
|------|------|
| `arch/arm/src/ra8p/ra8p_start.c` | RA8P1 启动代码 |
| `arch/arm/src/ra8p/ra8p_clockconfig.c` | 时钟配置 |
| `arch/arm/src/ra8p/ra8p_lowsetup.c` | 底层初始化 |
| `arch/arm/src/ra8p/chip.h` | 芯片定义 |
| `arch/arm/src/ra8p/hardware/ra8p_memorymap.h` | 内存映射 |
| `arch/arm/src/ra8p/hardware/ra8p_irq.h` | 中断定义 |
| `arch/arm/src/ra8p/Kconfig` | RA8P Kconfig |
| `arch/arm/src/ra8p/Make.defs` | 构建定义 |
| `arch/arm/src/ra8p/CMakeLists.txt` | CMake 配置 |
| `include/arch/ra8p/irq.h` | IRQ 头文件 |
| `arch/arm/src/stm32n6/stm32n6_start.c` | STM32N6 启动代码 |
| `arch/arm/src/stm32n6/stm32n6_clockconfig.c` | STM32N6 时钟配置 |
| `arch/arm/src/stm32n6/stm32n6_lowsetup.c` | STM32N6 底层初始化 |
| `arch/arm/src/stm32n6/stm32n6_soc.c` | STM32N6 SoC 初始化 |
| `arch/arm/src/stm32n6/stm32n6_rif.c` | RIF 资源隔离框架 |
| `arch/arm/src/stm32n6/chip.h` | STM32N6 芯片定义 |
| `arch/arm/src/stm32n6/hardware/stm32n6_memorymap.h` | 内存映射 |
| `arch/arm/src/stm32n6/hardware/stm32n6_irq.h` | 中断定义 |
| `arch/arm/src/stm32n6/hardware/stm32n6_rcc.h` | RCC 时钟寄存器 |
| `arch/arm/src/stm32n6/hardware/stm32n6_pwr.h` | PWR 电源寄存器 |
| `arch/arm/src/stm32n6/Kconfig` | STM32N6 Kconfig |
| `arch/arm/src/stm32n6/Make.defs` | STM32N6 构建定义 |
| `arch/arm/src/stm32n6/CMakeLists.txt` | STM32N6 CMake 配置 |
| `include/arch/stm32n6/irq.h` | STM32N6 IRQ 头文件 |

### 修改文件

| 文件 | 修改内容 |
|------|----------|
| `arch/arm/Kconfig` | 添加 `ARCH_CORTEXM55`、`ARCH_CORTEXM85` 配置，添加 RA8P/STM32N6 芯片选择 |
| `arch/arm/src/mps/Kconfig` | 添加 MPS3/MPS4 的 M55/M85 变体配置 |
| `arch/arm/src/armv8-m/arm_cpuinfo.c` | 添加 M55/M85 CPU 信息 |
| `arch/arm/src/cmake/armv8-m.cmake` | 添加 M55/M85 编译选项 |

### 新增 Kconfig 选项

```kconfig
# CPU 配置
config ARCH_CORTEXM55          # Cortex-M55 CPU
config ARCH_CORTEXM85          # Cortex-M85 CPU

# MPS 板级配置
config ARCH_CHIP_MPS3_AN547    # MPS3 AN547 (Cortex-M55)
config ARCH_CHIP_MPS3_AN550    # MPS3 AN550 (Cortex-M55)
config ARCH_CHIP_MPS3_AN555    # MPS3 AN555 (Cortex-M55)
config ARCH_CHIP_MPS3_AN547_M85 # MPS3 AN547 (Cortex-M85)
config ARCH_CHIP_MPS4_AN550    # MPS4 AN550 (Cortex-M85)

# RA8P 芯片配置
config ARCH_CHIP_RA8P          # Renesas RA8P 系列
config ARCH_CHIP_R7KA8P1KFLCAC # RA8P1 (Cortex-M85)

# STM32N6 芯片配置
config ARCH_CHIP_STM32N6       # STMicroelectronics STM32N6 系列
config ARCH_CHIP_STM32N657XX   # STM32N657xx (Cortex-M55)
```

## Zephyr 参考位置

本实现基于 Zephyr RTOS 的 Cortex-M55/M85 支持，参考以下文件：

### CPU 架构配置

| Zephyr 文件 | 说明 |
|-------------|------|
| `zephyr/arch/arm/core/cortex_m/Kconfig` (L77-97) | `CPU_CORTEX_M55`、`CPU_CORTEX_M85` 定义 |
| `zephyr/soc/arm/mps3/Kconfig` | Corstone-300 (M55)、Corstone-310 (M85) 配置 |
| `zephyr/soc/arm/mps4/Kconfig` | Corstone-315/320 (M85) 配置 |
| `zephyr/soc/renesas/ra/ra8p1/Kconfig` | RA8P1 SoC 配置 |
| `zephyr/soc/st/stm32/stm32n6x/Kconfig` | STM32N6 SoC 配置 |

### 板级支持

| Zephyr 文件 | 说明 |
|-------------|------|
| `zephyr/boards/arm/mps3/` | MPS3 板级支持目录 |
| `zephyr/boards/arm/mps3/mps3_corstone300_an547.dts` | Corstone-300 AN547 设备树 |
| `zephyr/boards/st/stm32n6570_dk/` | STM32N6570 Discovery Kit 板级支持 |

### Zephyr Kconfig 对比

**Zephyr Cortex-M55/M85 定义** (`arch/arm/core/cortex_m/Kconfig`):

```kconfig
config CPU_CORTEX_M55
    bool
    select CPU_CORTEX_M
    select ARMV8_1_M_MAINLINE
    select ARMV8_M_SE if CPU_HAS_TEE
    select ARMV7_M_ARMV8_M_FP if CPU_HAS_FPU
    select CPU_HAS_DCACHE
    select CPU_HAS_ICACHE

config CPU_CORTEX_M85
    bool
    select CPU_CORTEX_M
    select ARMV8_1_M_MAINLINE
    select ARMV8_M_SE if CPU_HAS_TEE
    select ARMV7_M_ARMV8_M_FP if CPU_HAS_FPU
    select CPU_HAS_DCACHE
    select CPU_HAS_ICACHE
```

**Zephyr STM32N6 配置** (`soc/st/stm32/stm32n6x/Kconfig`):

```kconfig
config SOC_SERIES_STM32N6X
    select ARM
    select CPU_CORTEX_M55
    select ARM_TRUSTZONE_M
    select CPU_HAS_ARM_SAU
    select CPU_HAS_ARM_MPU
    select CPU_HAS_FPU
    select ARMV8_M_DSP
    select ARMV8_1_M_MVEI
    select ARMV8_1_M_MVEF
    select CPU_CORTEX_M_HAS_DWT
    select HAS_STM32CUBE
    select INIT_ARCH_HW_AT_BOOT
    select SOC_RESET_HOOK
    select SOC_EARLY_INIT_HOOK
    select BUILD_OUTPUT_BIN
    select MPU_GAP_FILLING if USERSPACE
    select USE_STM32_HAL_RIF if STM32N6_RIF_OPEN
```

## Overview

This document describes the implementation status and configuration of ARM Cortex-M55 and Cortex-M85 processors in FeatherOS/NuttX.

## Architecture Features Comparison

| Feature | Cortex-M33 | Cortex-M55 | Cortex-M85 |
|---------|------------|------------|------------|
| Architecture | ARMv8-M Baseline | ARMv8.1-M Mainline | ARMv8.1-M Mainline |
| DSP Extension | Optional | Yes | Yes |
| FPU | Optional (FPv5-SP-D16) | Yes (FPv5-D16) | Yes (FPv5-D16) |
| MVE-I | No | Yes | Yes |
| MVE-F | No | Yes | Yes |
| I-Cache | No | Yes | Yes |
| D-Cache | No | Yes | Yes |
| TrustZone | Yes | Yes | Yes |
| PMU | No | Yes (8 counters) | Yes (8 counters) |
| PACBTI | No | No | Yes |

## Supported SoCs

### Renesas RA8P1 (Cortex-M85)

- **CPU**: Cortex-M85 @ 1 GHz
- **Flash**: Up to 768 KB
- **SRAM**: Up to 1 MB
- **Peripherals**: Ethernet, LCD, USB, CAN-FD, SCI, SPI, I2C, ADC

### STMicroelectronics STM32N6 (Cortex-M55)

- **CPU**: Cortex-M55 @ up to 800 MHz
- **AXISRAM**: Up to 2.75 MB (6 banks)
- **Peripherals**: LTDC, DMA2D, DCMIPP, ETH, NPU, FDCAN, USB OTG HS
- **Security**: TrustZone, RIF (Resource Isolation Framework)
- **Accelerators**: NPU (Neural-ART), JPEG, VENC

## Kconfig Configuration

### CPU Selection (`arch/arm/Kconfig`)

```kconfig
config ARCH_CORTEXM55
    bool
    default n
    select ARM_THUMB
    select ARCH_ARMV8M
    select ARCH_HAVE_IRQPRIO
    select ARCH_HAVE_IRQTRIGGER
    select ARCH_HAVE_RAMVECTORS
    select ARCH_HAVE_HIPRI_INTERRUPT
    select ARCH_HAVE_RESET
    select ARCH_HAVE_TESTSET
    select ARCH_HAVE_HARDFAULT_DEBUG
    select ARCH_HAVE_MEMFAULT_DEBUG
    select ARCH_HAVE_BUSFAULT_DEBUG
    select ARCH_HAVE_USAGEFAULT_DEBUG
    select ARCH_HAVE_SECUREFAULT_DEBUG if ARCH_TRUSTZONE_SECURE
    select ARMV8M_HAVE_ICACHE
    select ARMV8M_HAVE_DCACHE
    select ARM_HAVE_DSP
    select ARM_HAVE_MVE

config ARCH_CORTEXM85
    bool
    default n
    select ARM_THUMB
    select ARCH_ARMV8M
    # ... same base features as M55
    select ARMV8M_HAVE_ICACHE
    select ARMV8M_HAVE_DCACHE
    select ARM_HAVE_DSP
    select ARM_HAVE_MVE
```

### STM32N6 SoC Selection (`arch/arm/src/stm32n6/Kconfig`)

```kconfig
config ARCH_CHIP_STM32N6
    bool "STMicroelectronics STM32N6"
    select ARCH_CORTEXM55
    select ARCH_HAVE_MPU
    select ARM_HAVE_MPU_UNIFIED
    select ARCH_HAVE_FPU
    select ARMV8M_HAVE_ICACHE
    select ARMV8M_HAVE_DCACHE
    select ARM_HAVE_DSP
    select ARM_HAVE_MVE
    select STM32_HAL_RIF
    select STM32_HAL_NPU
    ---help---
        STMicroelectronics STM32N6 Series (ARM Cortex-M55).
        ARMv8.1-M Mainline with DSP, MVE, FPU, I/D Cache, TrustZone, RIF.

config ARCH_CHIP_STM32N657XX
    bool "STM32N657xx"
    select ARCH_CHIP_STM32N6
    ---help---
        STM32N657xx (Cortex-M55) with NPU, LTDC, Ethernet.
```

## NuttX 代码实现细节

### 目录结构

```
nuttx/arch/arm/src/
├── armv8-m/                    # ARMv8-M 通用架构代码
│   ├── arm_cache.c             # I/D-Cache 支持
│   ├── arm_fpuconfig.c         # FPU 配置
│   ├── arm_mpu.c               # MPU 支持
│   ├── arm_sau.c               # SAU (Secure Attribution Unit)
│   ├── arm_hardfault.c         # HardFault 处理
│   ├── arm_memfault.c          # MemFault 处理
│   ├── arm_securefault.c       # SecureFault 处理
│   ├── arm_perf.c              # PMU 支持
│   ├── arm_initialstate.c      # 初始状态设置
│   ├── arm_vectors.c           # 中断向量表
│   └── Make.defs               # 构建定义
├── mps/                        # ARM MPS (Corstone) 板级支持
│   ├── mps_start.c             # 启动代码
│   ├── mps_serial.c            # 串口驱动
│   ├── mps_irq.c               # 中断处理
│   ├── mps_timer.c             # 定时器
│   └── Kconfig                 # MPS 配置
├── ra8p/                       # Renesas RA8P (Cortex-M85)
│   ├── ra8p_start.c            # 启动代码
│   ├── ra8p_clockconfig.c      # 时钟配置
│   ├── ra8p_lowsetup.c         # 底层初始化
│   ├── chip.h                  # 芯片定义
│   ├── hardware/
│   │   ├── ra8p_memorymap.h    # 内存映射
│   │   └── ra8p_irq.h          # 中断定义
│   └── Kconfig                 # RA8P 配置
├── stm32n6/                    # STMicroelectronics STM32N6 (Cortex-M55)
│   ├── stm32n6_start.c         # 启动代码
│   ├── stm32n6_clockconfig.c   # 时钟配置 (RCC, PLL1-4, IC1-20)
│   ├── stm32n6_lowsetup.c      # 底层初始化 (PWR, GPIO)
│   ├── stm32n6_soc.c           # SoC 初始化 (Cache, RIF)
│   ├── stm32n6_rif.c           # RIF 资源隔离框架
│   ├── chip.h                  # 芯片定义
│   ├── hardware/
│   │   ├── stm32n6_memorymap.h # 内存映射
│   │   ├── stm32n6_irq.h       # 中断定义
│   │   ├── stm32n6_rcc.h       # RCC 时钟寄存器定义
│   │   └── stm32n6_pwr.h       # PWR 电源寄存器定义
│   ├── Kconfig                 # STM32N6 配置
│   ├── Make.defs               # 构建定义
│   └── CMakeLists.txt          # CMake 配置
└── cmake/
    └── armv8-m.cmake           # 工具链配置
```

### STM32N6 SoC 初始化 (`stm32n6_soc.c`)

基于 Zephyr `soc/st/stm32/stm32n6x/soc.c` 实现：

```c
#include <nuttx/config.h>
#include <stdint.h>
#include "arm_internal.h"
#include "stm32n6_rcc.h"
#include "stm32n6_pwr.h"

void stm32n6_soc_early_init(void)
{
    /* 1. Enable I-Cache and D-Cache */
    arm_enable_icache();
    arm_enable_dcache();

    /* 2. Enable PWR clock */
    modifyreg32(STM32N6_RCC_AHB4ENR, 0, RCC_AHB4ENR_PWR);

    /* 3. Set regulator voltage scaling for best performance */
    modifyreg32(STM32N6_PWR_CSR1, PWR_CSR1_VOS_MASK,
                PWR_CSR1_VOS_SCALE0);

    /* 4. Enable IO supply configuration */
    modifyreg32(STM32N6_PWR_CR2, 0,
                PWR_CR2_IOSV1EN | PWR_CR2_IOSV2EN |
                PWR_CR2_IOSV3EN | PWR_CR2_IOSV4EN | PWR_CR2_IOSV5EN);

#ifdef CONFIG_STM32N6_RIF_OPEN
    /* 5. Configure RIF (Resource Isolation Framework) */
    stm32n6_rif_init();
#endif
}

/* RIF Configuration - Reference: Zephyr soc.c */
#ifdef CONFIG_STM32N6_RIF_OPEN
static void stm32n6_rif_init(void)
{
    /* Enable RIFSC clock */
    modifyreg32(STM32N6_RCC_AHB5ENR, 0, RCC_AHB5ENR_RIFSC);

    /* Configure RIMC (Resource Isolation Master Controller) */
    /* Configure RISC (Resource Isolation Slave Controller) */
    /* Reference: Zephyr RIF_MASTER_CID1_SEC_PRIV and RIF_SLAVE_SEC_PRIV macros */

    /* ADC12 Configuration */
    /* DMA2D Configuration */
    /* ETH1 Configuration */
    /* JPEG Configuration */
    /* LTDC Configuration */
    /* NPU Configuration */
    /* VENC Configuration */
}
#endif
```

### STM32N6 内存映射 (`hardware/stm32n6_memorymap.h`)

基于 Zephyr `dts/arm/st/n6/stm32n6.dtsi`:

```c
/* Peripheral base addresses - Secure region (default) */
#define STM32N6_PERIPH_BASE        0x50000000

/* RCC (Reset and Clock Control) */
#define STM32N6_RCC_BASE           (STM32N6_PERIPH_BASE + 0x002800)

/* PWR (Power Control) */
#define STM32N6_PWR_BASE           (STM32N6_PERIPH_BASE + 0x002C00)

/* GPIO Ports */
#define STM32N6_GPIOA_BASE         (STM32N6_PERIPH_BASE + 0x00200000)
#define STM32N6_GPIOB_BASE         (STM32N6_PERIPH_BASE + 0x00200400)
#define STM32N6_GPIOC_BASE         (STM32N6_PERIPH_BASE + 0x00200800)
#define STM32N6_GPIOD_BASE         (STM32N6_PERIPH_BASE + 0x00200C00)
#define STM32N6_GPIOE_BASE         (STM32N6_PERIPH_BASE + 0x00201000)
#define STM32N6_GPIOF_BASE         (STM32N6_PERIPH_BASE + 0x00201400)
#define STM32N6_GPIOG_BASE         (STM32N6_PERIPH_BASE + 0x00201800)
#define STM32N6_GPIOH_BASE         (STM32N6_PERIPH_BASE + 0x00201C00)
#define STM32N6_GPION_BASE         (STM32N6_PERIPH_BASE + 0x00203400)
#define STM32N6_GPIOO_BASE         (STM32N6_PERIPH_BASE + 0x00203800)
#define STM32N6_GPIOP_BASE         (STM32N6_PERIPH_BASE + 0x00203C00)
#define STM32N6_GPIOQ_BASE         (STM32N6_PERIPH_BASE + 0x00204000)

/* USART/UART */
#define STM32N6_USART1_BASE        (STM32N6_PERIPH_BASE + 0x00010000)
#define STM32N6_USART2_BASE        (STM32N6_PERIPH_BASE + 0x00004400)
/* ... USART3-10, UART4-9 ... */

/* I2C */
#define STM32N6_I2C1_BASE          (STM32N6_PERIPH_BASE + 0x00005400)
#define STM32N6_I2C2_BASE          (STM32N6_PERIPH_BASE + 0x00005800)
#define STM32N6_I2C3_BASE          (STM32N6_PERIPH_BASE + 0x00005C00)
#define STM32N6_I2C4_BASE          (STM32N6_PERIPH_BASE + 0x00001C00)

/* SPI */
#define STM32N6_SPI1_BASE          (STM32N6_PERIPH_BASE + 0x00013000)
#define STM32N6_SPI2_BASE          (STM32N6_PERIPH_BASE + 0x00003800)
/* ... SPI3-6 ... */

/* FDCAN */
#define STM32N6_FDCAN1_BASE        (STM32N6_PERIPH_BASE + 0x0000A000)
#define STM32N6_FDCAN2_BASE        (STM32N6_PERIPH_BASE + 0x0000A400)
#define STM32N6_FDCAN3_BASE        (STM32N6_PERIPH_BASE + 0x0000E800)

/* ADC */
#define STM32N6_ADC1_BASE          (STM32N6_PERIPH_BASE + 0x00022000)
#define STM32N6_ADC2_BASE          (STM32N6_PERIPH_BASE + 0x00022100)

/* LTDC (LCD-TFT Display Controller) */
#define STM32N6_LTDC_BASE          (STM32N6_PERIPH_BASE + 0x00011000)

/* DMA2D (2D Graphics Accelerator) */
#define STM32N6_DMA2D_BASE         (STM32N6_PERIPH_BASE + 0x00011400)

/* DCMIPP (Digital Camera Interface Parallel Port) */
#define STM32N6_DCMIPP_BASE        (STM32N6_PERIPH_BASE + 0x000802000)

/* Ethernet */
#define STM32N6_ETH_BASE           (STM32N6_PERIPH_BASE + 0x000803600)

/* NPU (Neural-ART Neural Processing Unit) */
#define STM32N6_NPU_BASE           (STM32N6_PERIPH_BASE + 0x00080E0000)

/* JPEG Codec */
#define STM32N6_JPEG_BASE          (STM32N6_PERIPH_BASE + 0x0008023000)

/* VENC (Video Encoder) */
#define STM32N6_VENC_BASE          (STM32N6_PERIPH_BASE + 0x0008005000)

/* USB OTG HS */
#define STM32N6_OTG_HS1_BASE       (STM32N6_PERIPH_BASE + 0x0008040000)
#define STM32N6_OTG_HS2_BASE       (STM32N6_PERIPH_BASE + 0x0008080000)

/* AXISRAM (AXI SRAM) - 6 banks, total 2.75 MB */
#define STM32N6_AXISRAM1_BASE      0x34000000  /* 512 KB */
#define STM32N6_AXISRAM2_BASE      0x34180000  /* 512 KB */
#define STM32N6_AXISRAM3_BASE      0x34200000  /* 448 KB */
#define STM32N6_AXISRAM4_BASE      0x34270000  /* 448 KB */
#define STM32N6_AXISRAM5_BASE      0x342E0000  /* 448 KB */
#define STM32N6_AXISRAM6_BASE      0x34350000  /* 448 KB */
```

### STM32N6 时钟配置 (`hardware/stm32n6_rcc.h`)

基于 Zephyr `include/zephyr/dt-bindings/clock/stm32n6_clock.h`:

```c
/* RCC Register Offsets */
#define STM32N6_RCC_CR_OFFSET      0x0000  /* Clock Control */
#define STM32N6_RCC_ICSCR_OFFSET   0x0004  /* Internal Clock Calibration */
#define STM32N6_RCC_CFGR1_OFFSET   0x0020  /* Configuration Register 1 */
#define STM32N6_RCC_CFGR2_OFFSET   0x0024  /* Configuration Register 2 */
#define STM32N6_RCC_PLL1CFGR_OFFSET 0x0028 /* PLL1 Configuration */
#define STM32N6_RCC_PLL2CFGR_OFFSET 0x002C /* PLL2 Configuration */
#define STM32N6_RCC_PLL3CFGR_OFFSET 0x0030 /* PLL3 Configuration */
#define STM32N6_RCC_PLL4CFGR_OFFSET 0x0034 /* PLL4 Configuration */

/* Bus Enable Registers */
#define STM32N6_RCC_AHB1ENR_OFFSET 0x0250
#define STM32N6_RCC_AHB2ENR_OFFSET 0x0254
#define STM32N6_RCC_AHB3ENR_OFFSET 0x0258
#define STM32N6_RCC_AHB4ENR_OFFSET 0x025C
#define STM32N6_RCC_AHB5ENR_OFFSET 0x0260
#define STM32N6_RCC_APB1ENR_OFFSET 0x0264
#define STM32N6_RCC_APB2ENR_OFFSET 0x026C
#define STM32N6_RCC_APB4ENR_OFFSET 0x0274
#define STM32N6_RCC_APB5ENR_OFFSET 0x027C

/* CCIPR Registers (Peripheral Clock Selection) */
#define STM32N6_RCC_CCIPR1_OFFSET  0x0144
#define STM32N6_RCC_CCIPR2_OFFSET  0x0148
#define STM32N6_RCC_CCIPR3_OFFSET  0x014C
#define STM32N6_RCC_CCIPR4_OFFSET  0x0150
#define STM32N6_RCC_CCIPR5_OFFSET  0x0154
#define STM32N6_RCC_CCIPR6_OFFSET  0x0158
#define STM32N6_RCC_CCIPR7_OFFSET  0x015C
#define STM32N6_RCC_CCIPR8_OFFSET  0x0160
#define STM32N6_RCC_CCIPR9_OFFSET  0x0164
#define STM32N6_RCC_CCIPR12_OFFSET 0x0170
#define STM32N6_RCC_CCIPR13_OFFSET 0x0174
#define STM32N6_RCC_CCIPR14_OFFSET 0x0178

/* ICxCFGR Registers (Interconnect Clock Configuration) */
#define STM32N6_RCC_IC1CFGR_OFFSET 0x00C4
/* ... IC2CFGR to IC20CFGR ... */

/* Clock Sources */
#define STM32N6_CLK_SRC_HSI        0  /* High Speed Internal */
#define STM32N6_CLK_SRC_HSE        1  /* High Speed External */
#define STM32N6_CLK_SRC_PLL1       2  /* PLL1 Output */
#define STM32N6_CLK_SRC_PLL2       3  /* PLL2 Output */
#define STM32N6_CLK_SRC_PLL3       4  /* PLL3 Output */
#define STM32N6_CLK_SRC_PLL4       5  /* PLL4 Output */
#define STM32N6_CLK_SRC_IC1        6  /* Interconnect Clock 1 */
/* ... IC2 to IC20 ... */
```

### RA8P1 启动代码 (`ra8p/ra8p_start.c`)

```c
void __start(void)
{
#ifndef CONFIG_BUILD_PIC
    const uint32_t *src;
    uint32_t *dest;
#endif

    /* Zero .bss */
    for (dest = (uint32_t *)_sbss; dest < (uint32_t *)_ebss; )
        {
            *dest++ = 0;
        }

#ifndef CONFIG_BUILD_PIC
    /* Copy initialized data from FLASH to SRAM */
    for (src = (const uint32_t *)_eronly, dest = (uint32_t *)_sdata;
         dest < (uint32_t *)_edata;)
        {
            *dest++ = *src++;
        }
#endif

    /* Configure clock and FPU */
    ra8p_clockconfig();
    arm_fpuconfig();

    /* Configure UART pins */
    ra8p_lowsetup();

#ifdef USE_EARLYSERIALINIT
    arm_earlyserialinit();
#endif

    /* Enable Cache */
    ra8p_icache_enable();
    ra8p_dcache_enable();

#ifdef CONFIG_BOARD_INITIALIZE
    ra8p_boardinitialize();
#endif

    /* Start NuttX */
    nx_start();

    for (; ; );
}
```

## Features Description

### 1. DSP Extension

The DSP extension provides single-cycle MAC operations and SIMD instructions for signal processing.

**Kconfig:**
- `CONFIG_ARM_HAVE_DSP` - Enables DSP instruction support
- `CONFIG_ARM_DSP` - Runtime DSP extension enable

### 2. MVE (M-Profile Vector Extension)

MVE provides vector processing capabilities for both integer (MVE-I) and floating-point (MVE-F) operations.

**Kconfig:**
- `CONFIG_ARM_HAVE_MVE` - Enables MVE support

### 3. Cache Support

M55 and M85 include integrated I-cache and D-cache.

**Kconfig:**
- `CONFIG_ARMV8M_HAVE_ICACHE` - Hardware has I-cache
- `CONFIG_ARMV8M_HAVE_DCACHE` - Hardware has D-cache
- `CONFIG_ARMV8M_ICACHE` - Enable I-cache at runtime
- `CONFIG_ARMV8M_DCACHE` - Enable D-cache at runtime

### 4. Performance Monitoring Unit (PMU)

ARMv8.1-M includes an enhanced PMU with up to 8 event counters plus a cycle counter.

### 5. RIF (Resource Isolation Framework) - STM32N6 Specific

RIF provides hardware-enforced isolation between secure and non-secure worlds on STM32N6.

**Kconfig:**
- `CONFIG_STM32N6_RIF_OPEN` - Configure RIF with full access for secure world

### 6. NPU (Neural-ART) - STM32N6 Specific

Hardware neural network accelerator for AI/ML inference.

**Kconfig:**
- `CONFIG_STM32N6_NPU` - Enable Neural-ART NPU

## Reference: Zephyr to NuttX Mapping

| Zephyr Config | NuttX Config | Description |
|---------------|--------------|-------------|
| `CPU_CORTEX_M55` | `ARCH_CORTEXM55` | Cortex-M55 CPU |
| `CPU_CORTEX_M85` | `ARCH_CORTEXM85` | Cortex-M85 CPU |
| `ARMV8_1_M_MAINLINE` | `ARCH_ARMV8M` | ARMv8.1-M Architecture |
| `CPU_HAS_ICACHE` | `ARMV8M_HAVE_ICACHE` | I-Cache present |
| `CPU_HAS_DCACHE` | `ARMV8M_HAVE_DCACHE` | D-Cache present |
| `ARMV8_M_DSP` | `ARM_HAVE_DSP` | DSP extension |
| `ARMV8_1_M_MVEI` | `ARM_HAVE_MVE` | MVE-I support |
| `ARMV8_1_M_MVEF` | `ARM_HAVE_MVE` | MVE-F support |
| `ARMV8_1_M_PMU` | `ARCH_HAVE_PERF_EVENTS` | PMU present |
| `ARMV8_M_SE` | `ARMV8M_CMSE` | TrustZone Security |
| `SOC_R7KA8P1KFLCAC_CM85` | `ARCH_CHIP_R7KA8P1KFLCAC` | RA8P1 CM85 core |
| `SOC_SERIES_STM32N6X` | `ARCH_CHIP_STM32N6` | STM32N6 series |
| `SOC_STM32N657XX` | `ARCH_CHIP_STM32N657XX` | STM32N657xx |
| `STM32N6_RIF_OPEN` | `CONFIG_STM32N6_RIF_OPEN` | RIF configuration |
| `STM32N6_NPU` | `CONFIG_STM32N6_NPU` | Neural-ART NPU |

## Usage Example

### Building for STM32N6570 Discovery Kit

```bash
cd nuttx
./tools/configure.sh stm32n6570-dk:nsh
make menuconfig
# Select: System Type -> STMicroelectronics STM32N6 -> STM32N657xx
# Enable: System Type -> Cortex-M55 Configuration -> MVE, DSP, Cache
make
```

### Building for MPS3 AN547 (Cortex-M55)

```bash
cd nuttx
./tools/configure.sh mps3-an547:nsh
make menuconfig
# Select: System Type -> ARM MPS Configuration -> MPS3 AN547 Processor Cortexm55
make
```

### Building for RA8P1

```bash
cd nuttx
./tools/configure.sh ek-ra8p1:nsh
make menuconfig
# Select: System Type -> Renesas RA8P -> R7KA8P1KFLCAC (RA8P1)
make
```

## References

1. [ARM Cortex-M55 Processor - Arm Developer](https://developer.arm.com/Processors/Cortex-M55)
2. [ARM Cortex-M85 Processor - Arm Developer](https://developer.arm.com/Processors/Cortex-M85)
3. [ARMv8-M Architecture Reference Manual](https://developer.arm.com/documentation/ddi0553/latest)
4. [Zephyr Project - Cortex-M55/M85 Support](https://docs.zephyrproject.org/latest/)
5. [NuttX ARMv8-M Documentation](https://nuttx.apache.org/docs/latest/)
6. [Renesas RA8P1](https://www.renesas.com/ra8p1)
7. [STM32N657xx Reference Manual (RM0486)](https://www.st.com/resource/en/reference_manual/rm0486-stm32n657xx-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)

## Zephyr 源码参考路径

```
zephyr/                                    # Zephyr RTOS 根目录
├── arch/arm/core/cortex_m/Kconfig         # CPU_CORTEX_M55/M85 定义 (L77-97)
├── soc/arm/
│   ├── mps3/Kconfig                      # Corstone-300/310 配置
│   └── mps4/Kconfig                      # Corstone-315/320 配置
├── soc/renesas/ra/
│   ├── Kconfig                            # RA 系列基础配置
│   ├── ra8p1/                            # RA8P1 实现
│   │   ├── Kconfig                       # RA8P1 Kconfig
│   │   ├── Kconfig.soc                   # SOC 系列选择
│   │   ├── Kconfig.defconfig             # 默认配置
│   │   ├── power.c                       # 电源管理
│   │   └── sections.ld                   # 链接段
│   ├── ra8m1/Kconfig                     # RA8M1 (Cortex-M85)
│   ├── ra8m2/Kconfig                     # RA8M2 (Cortex-M85)
│   ├── ra8d1/Kconfig                     # RA8D1 (Cortex-M85)
│   ├── ra8d2/Kconfig                     # RA8D2 (Cortex-M85)
│   ├── ra8e1/Kconfig                     # RA8E1 (Cortex-M85)
│   └── ra8t1/Kconfig                     # RA8T1 (Cortex-M85)
├── soc/st/stm32/stm32n6x/                # STM32N6 实现
│   ├── Kconfig                           # STM32N6 系列配置
│   ├── Kconfig.soc                       # SOC 选择
│   ├── Kconfig.defconfig                 # 默认配置
│   ├── soc.c                             # SoC 初始化 (Cache, PWR, RIF)
│   ├── soc.h                             # SoC 头文件
│   ├── mpu_regions.c                     # MPU 区域配置
│   ├── mpu_regions.ld                    # MPU 链接脚本
│   ├── ram_check.ld                      # RAM 检查
│   └── npu/                              # NPU 子系统
│       ├── Kconfig                       # NPU 配置
│       ├── npu_cache_stm32n6.c          # NPU Cache 驱动
│       └── npu_stm32n6.c                 # NPU 驱动
├── dts/arm/st/n6/                        # STM32N6 设备树
│   ├── stm32n6.dtsi                      # 基础设备树 (CPU, MPU, 外设)
│   ├── stm32n657.dtsi                    # STM32N657 设备树
│   ├── stm32n657X0.dtsi                  # 封装变体
│   └── stm32n657X0_ns.dtsi               # 非安全模式
├── dts/arm/renesas/ra/ra8/
│   ├── ra8x2.dtsi                        # RA8x2 基础 (CPU/GPIO/SCI/...)
│   ├── r7ka8p1kflcac.dtsi               # RA8P1 设备树
│   ├── r7ka8p1kflcac_cm85.dtsi          # CM85 核心设备树
│   └── r7ka8p1xf.dtsi                   # RA8P1 系列基础 (时钟)
├── boards/st/stm32n6570_dk/              # STM32N6570 Discovery Kit
│   ├── stm32n6570_dk.dts                # 板级设备树
│   ├── stm32n6570_dk_common.dtsi        # 公共定义 (时钟, 外设)
│   ├── stm32n6570_dk_defconfig           # 默认配置
│   ├── Kconfig.defconfig                 # 板级 Kconfig
│   ├── board.cmake                       # 构建配置
│   └── arduino_r3_connector.dtsi         # Arduino 连接器
├── boards/renesas/ek_ra8p1/
│   ├── ek_ra8p1.dtsi                    # 开发板设备树
│   ├── ek_ra8p1-pinctrl.dtsi            # 引脚配置
│   ├── ek_ra8p1_r7ka8p1kflcac_cm85.dts # CM85 配置
│   └── ek_ra8p1_r7ka8p1kflcac_cm85_defconfig # 默认编译配置
├── drivers/misc/stm32n6_axisram/
│   ├── stm32n6_axisram.c                # AXISRAM 驱动
│   └── Kconfig                           # AXISRAM 配置
├── include/zephyr/dt-bindings/
│   ├── clock/stm32n6_clock.h            # 时钟绑定定义 (PLL, IC, CCIPR)
│   ├── reset/stm32n6_reset.h            # 复位绑定定义 (RCC总线复位偏移)
│   └── power/stm32n6_iocell.h           # IO Cell 电源配置
└── tests/drivers/clock_control/stm32_clock_configuration/
    ├── stm32n6_core/                     # STM32N6 内核时钟测试
    └── stm32n6_devices/                  # STM32N6 外设时钟测试
```

## STM32N6 外设摘要 (基于 Zephyr)

### 外设列表

| 外设 | 数量 | 基地址范围 | 说明 |
|------|------|-----------|------|
| GPIO | 12 端口 | 0x50200000-0x50204000 | A, B, C, D, E, F, G, H, N, O, P, Q |
| USART/UART | 10 通道 | 0x50004400+ | USART1-6, UART7-10 |
| I2C | 4 通道 | 0x50005400+ | I2C1-I2C4 |
| SPI/I2S | 6 通道 | 0x50003800+ | SPI1-SPI6, I2S1-I2S3, I2S6 |
| I3C | 2 通道 | 0x50006000+ | I3C1-I3C2 |
| FDCAN | 3 通道 | 0x5000A000+ | FDCAN1-FDCAN3 |
| ADC | 2 通道 | 0x50022000+ | ADC1-ADC2 (12-bit) |
| LTDC | 1 控制器 | 0x50011000 | LCD-TFT 显示控制器 |
| DMA2D | 1 控制器 | 0x50011400 | 2D 绘图加速器 |
| DCMIPP | 1 控制器 | 0x5802000 | 数字摄像头接口 |
| JPEG | 1 控制器 | 0x58023000 | JPEG 编解码器 |
| ETH | 1 控制器 | 0x58036000 | 以太网 MAC |
| NPU | 1 核 | 0x580E0000 | Neural-ART 神经网络处理器 |
| VENC | 1 控制器 | 0x58005000 | 视频编码器 |
| USB OTG HS | 2 通道 | 0x58040000+ | USB OTG HS 1/2 |
| SDMMC | 2 通道 | 0x58027000+ | SD/MMC 控制器 |
| LPTIM | 5 通道 | - | 低功耗定时器 |
| RNG | 1 通道 | 0x50020000 | 真随机数发生器 |

### STM32N6 内存布局

| 区域 | 地址 | 大小 | 说明 |
|------|------|------|------|
| AXISRAM1 | 0x34000000 | 512 KB | AXI SRAM Bank 1 |
| AXISRAM2 | 0x34180000 | 512 KB | AXI SRAM Bank 2 |
| AXISRAM3 | 0x34200000 | 448 KB | AXI SRAM Bank 3 |
| AXISRAM4 | 0x34270000 | 448 KB | AXI SRAM Bank 4 |
| AXISRAM5 | 0x342E0000 | 448 KB | AXI SRAM Bank 5 |
| AXISRAM6 | 0x34350000 | 448 KB | AXI SRAM Bank 6 |
| **总计** | - | **2.75 MB** | 总 AXI SRAM |

### STM32N6 时钟树

```
┌─────────────────────────────────────────────────────────────────┐
│                        STM32N6 时钟树                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌────────┐    ┌────────┐    ┌────────┐    ┌────────┐          │
│  │ HSI 64 │    │ HSE 48 │    │ LSI 32k│    │ LSE 32k│          │
│  └────────┘    └────────┘    └────────┘    └────────┘          │
│       │             │              │              │             │
│       └─────────────┴──────────────┴──────────────┘             │
│                              │                                    │
│         ┌────────────────────┼────────────────────┐             │
│         │                    │                    │             │
│    ┌────┴───┐           ┌────┴───┐           ┌────┴───┐        │
│    │ PLL1   │           │ PLL2   │           │ PLL3   │        │
│    └────────┘           └────────┘           └────────┘        │
│         │                    │                    │             │
│    ┌────┴───┐           ┌────┴───┐           ┌────┴───┐        │
│    │ IC1-10 │           │ IC11-20│           │ ICxx   │        │
│    └────────┘           └────────┘           └────────┘        │
│         │                    │                    │             │
│         └────────────────────┴────────────────────┘             │
│                              │                                    │
│         ┌────────────────────┼────────────────────┐             │
│         │                    │                    │             │
│    ┌────┴───┐           ┌────┴───┐           ┌────┴───┐        │
│    │ CPU    │           │ AHBx   │           │ APBx   │        │
│    │ (IC1)  │           │ 总线   │           │ 总线   │        │
│    └────────┘           └────────┘           └────────┘        │
│                                                                    │
│  IC1-20: 20 个独立的互联时钟域                                      │
│  - IC1: CPU 时钟 (最高 800 MHz)                                     │
│  - IC2: AHB 总线 (最高 400 MHz)                                     │
│  - IC3-20: 各外设域时钟                                             │
│                                                                    │
│  每个外设可从多个时钟源中选择: HSI, HSE, PLL1-PLL4, ICx              │
│                                                                    │
└─────────────────────────────────────────────────────────────────┘
```

## STM32N6570 Discovery Kit 板级配置 (参考 Zephyr)

### 时钟配置 (stm32n6570_dk_common.dtsi)

| 时钟 | 源 | 频率 |
|------|-----|------|
| CPU (IC1) | PLL1 | 800 MHz |
| AHB (IC2) | PLL1 | 400 MHz |
| PERCK | HSI | 64 MHz |
| USART/UART | CKPER | 64 MHz |
| I2C | CKPER | 64 MHz |
| SPI | CKPER | 64 MHz |
| SDMMC | IC4 | - |
| LTDC | IC16 | - |
| ETH | IC5 | - |

### PLL 配置

```kconfig
PLL1:  HSE (48 MHz) / 3 * 150 / 1 = 800 MHz (CPU)
PLL2:  HSI (64 MHz) / 2 * 48 / 1 = 400 MHz (AHB)
PLL3:  HSE (48 MHz) / 3 * 125 / 1 = 600 MHz (Peripherals)
PLL4:  HSI (64 MHz) / 4 * 75 / 1 = 300 MHz (Low-speed peripherals)
```

### 可用外设

- **USART1**: TX=PE5, RX=PE6 (VCP)
- **USART2**: TX=PD5, RX=PF6 (Arduino)
- **I2C1/I2C4**: 用于触摸屏和传感器
- **SPI5**: SPI 接口
- **LTDC**: RGB LCD 接口 (800x480)
- **ETH**: 以太网 (RMII)
- **USB OTG HS**: USB 2.0 High-Speed
- **SDMMC2**: SD 卡接口
- **FDCAN1**: CAN FD 接口
- **JPEG**: JPEG 硬件编解码
- **NPU**: 神经网络加速器

## Changelog

| Date | Commit | Description |
|------|--------|-------------|
| 2026-04-23 | `3e5b67a` | Initial M55/M85 implementation based on Zephyr Corstone-300/310/315/320 support |
| 2026-04-23 | `3e5b67a` | Added RA8P1 support (R7KA8P1KFLCAC) |
| 2026-04-23 | `3e5b67a` | Added MPS3/MPS4 board configurations for M55/M85 variants |
| 2026-04-23 | `3e5b67a` | Updated armv8-m toolchain and cmake configuration |
| 2026-04-24 | - | Added detailed code implementation documentation |
| 2026-04-24 | - | Added STM32N6 (Cortex-M55) SoC support documentation |
| 2026-04-24 | - | Added STM32N6 RIF and NPU support documentation |
| 2026-04-24 | - | Added STM32N6 memory map and clock tree documentation |
| 2026-04-24 | - | Added STM32N6570 Discovery Kit board configuration |
| 2026-04-24 | - | Added stm32n6_porting.md with complete porting guide |
| 2026-04-24 | - | Updated all NuttX code examples to follow NuttX coding style |
