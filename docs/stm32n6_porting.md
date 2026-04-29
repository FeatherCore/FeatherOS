# STM32N6 移植记录

> 基于 Zephyr RTOS 实现，为 FeatherOS/NuttX 添加 STM32N6 (Cortex-M55) 支持

## 概述

STM32N6 是 STMicroelectronics 基于 ARM Cortex-M55 的高性能微控制器系列，采用 ARMv8.1-M Mainline 架构，支持：
- DSP 扩展指令
- MVE (M-Profile Vector Extension) - 整数和浮点向量运算
- 浮点运算 (FPU FPv5-D16)
- TrustZone 安全扩展
- RIF (Resource Isolation Framework) - 硬件资源隔离
- I-Cache 和 D-Cache
- NPU (Neural-ART) 神经网络加速器

## NuttX 代码风格

本文档中所有 NuttX 代码遵循以下风格规范：

### 文件头

```c
/****************************************************************************
 * arch/arm/src/stm32n6/xxx.c
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
```

### 代码结构

1. **Included Files** - 包含头文件
2. **Pre-processor Definitions** - 宏定义
3. **Private Types** - 私有类型定义
4. **Private Data** - 私有数据
5. **Private Functions** - 私有函数
6. **Public Functions** - 公共函数

### 函数注释

```c
/****************************************************************************
 * Name: function_name
 *
 * Description:
 *   Brief description of what the function does.
 *   Reference: Zephyr source file path
 *
 * Input Parameters:
 *   param1 - Description of parameter 1
 *   param2 - Description of parameter 2
 *
 * Returned Value:
 *   Description of return value.
 *
 ****************************************************************************/
```

### 命名约定

- 宏定义：`STM32_RCC_AHB5ENR` (全大写，下划线分隔)
- 函数：`stm32n6_soc_early_init` (小写，下划线分隔)
- 类型：`struct stm32n6_npu_cfg_s` (小写，下划线分隔，`_s` 后缀)
- 寄存器位：`RCC_AHB5ENR_NPU` (模块前缀)

### 寄存器访问

使用 `modifyreg32()` 和 `getreg32()`/`putreg32()` 宏：

```c
/* 设置位 */
modifyreg32(STM32_RCC_AHB5ENR, 0, RCC_AHB5ENR_NPU);

/* 清除位 */
modifyreg32(STM32_RCC_AHB5RSTR, RCC_AHB5RSTR_NPU, 0);

/* 读取寄存器 */
uint32_t reg = getreg32(STM32_RCC_CR);
```

## 芯片特性对比

| 特性 | STM32N6 |
|------|----------|
| CPU | ARM Cortex-M55 |
| 架构 | ARMv8.1-M Mainline |
| 最高频率 | 800 MHz (CPU), 400 MHz (AHB) |
| DSP 扩展 | 是 |
| MVE | 是 (MVE-I/MVE-F) |
| FPU | 是 (FPv5-D16) |
| TrustZone | 是 |
| RIF | 是 (资源隔离框架) |
| Cache | I-Cache + D-Cache |
| AXISRAM | 高达 2.75 MB (6 banks) |
| NPU | 是 (Neural-ART) |
| LTDC | 是 (LCD 控制器) |
| DMA2D | 是 (2D 图形加速器) |
| DCMIPP | 是 (数字摄像头接口) |
| ETH | 是 (以太网 MAC) |
| JPEG | 是 (JPEG 编解码器) |
| VENC | 是 (视频编码器) |

## 移植来源

移植基于 Zephyr RTOS 的 STM32N6 实现：

| Zephyr 路径 | 描述 |
|-------------|------|
| `soc/st/stm32/stm32n6x/` | STM32N6 SOC 配置和初始化 |
| `soc/st/stm32/stm32n6x/soc.c` | SoC 早期初始化 (Cache, PWR, RIF) |
| `soc/st/stm32/stm32n6x/Kconfig` | SOC Kconfig 配置 |
| `soc/st/stm32/stm32n6x/mpu_regions.c` | MPU 区域配置 |
| `soc/st/stm32/stm32n6x/npu/` | NPU 子系统驱动 |
| `dts/arm/st/n6/stm32n6.dtsi` | STM32N6 基础设备树 |
| `dts/arm/st/n6/stm32n657.dtsi` | STM32N657 设备树 |
| `dts/arm/st/n6/stm32n657X0.dtsi` | STM32N657X0 封装变体 |
| `dts/arm/st/n6/stm32n657X0_ns.dtsi` | 非安全模式设备树 |
| `boards/st/stm32n6570_dk/` | STM32N6570 Discovery Kit 板级支持 |
| `include/zephyr/dt-bindings/clock/stm32n6_clock.h` | 时钟绑定定义 |
| `include/zephyr/dt-bindings/reset/stm32n6_reset.h` | 复位绑定定义 |
| `drivers/misc/stm32n6_axisram/` | AXISRAM 驱动 |

## 目录结构

### Zephyr 源文件结构：

```
soc/st/stm32/stm32n6x/
├── Kconfig                      # SOC 系列配置
├── Kconfig.soc                  # SOC 选择
├── Kconfig.defconfig            # 默认配置
├── soc.c                        # SoC 初始化 (Cache, PWR, RIF)
├── soc.h                        # SoC 头文件 (包含 stm32n6xx.h)
├── mpu_regions.c                # MPU 区域配置
├── mpu_regions.ld               # MPU 链接脚本
├── ram_check.ld                 # RAM 检查链接脚本
├── CMakeLists.txt               # CMake 构建配置
└── npu/                         # NPU 子系统
    ├── Kconfig                  # NPU 配置
    ├── npu_cache_stm32n6.c      # NPU Cache 驱动
    └── npu_stm32n6.c            # NPU 驱动
```

### NuttX 目标结构：

```
arch/arm/src/stm32n6/
├── Kconfig                      # STM32N6 Kconfig
├── Make.defs                    # 构建定义
├── CMakeLists.txt               # CMake 配置
├── chip.h                       # 芯片定义
├── stm32n6_start.c              # 启动代码
├── stm32n6_clockconfig.c        # 时钟配置
├── stm32n6_clockconfig.h        # 时钟 API
├── stm32n6_lowsetup.c           # 底层初始化
├── stm32n6_lowsetup.h           # 底层 API
├── stm32n6_soc.c                # SoC 初始化
├── stm32n6_rif.c                # RIF 配置
├── stm32n6_npu.c                # NPU 驱动
├── stm32n6_gpio.c               # GPIO 驱动
├── stm32n6_uart.c               # UART 驱动
└── hardware/
    ├── stm32n6_memorymap.h      # 内存映射
    ├── stm32n6_irq.h            # 中断定义
    ├── stm32n6_rcc.h            # RCC 时钟寄存器
    ├── stm32n6_pwr.h            # PWR 电源寄存器
    ├── stm32n6_gpio.h           # GPIO 寄存器
    └── stm32n6_uart.h           # UART 寄存器

include/arch/stm32n6/
└── irq.h                        # IRQ 头文件
```

## Kconfig 配置

### `soc/st/stm32/stm32n6x/Kconfig` (Zephyr)

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
	select CPU_HAS_CUSTOM_FIXED_SOC_MPU_REGIONS if USERSPACE && !XIP
	select USE_STM32_HAL_RIF if STM32N6_RIF_OPEN

if SOC_SERIES_STM32N6X

config STM32N6_BOOT_SERIAL
	bool "Serial boot target (USB)"

config STM32N6_RIF_OPEN
	bool "Configure the RIF with all OPEN access"
	default y
	depends on TRUSTED_EXECUTION_SECURE
	help
	  When this option is enabled, the RIMC of all masters and the RISC of all slaves are
	  configured during SoC initialization. Zephyr running with Secure privileges has full
	  access to all SoC resources.

source "soc/st/stm32/stm32n6x/npu/Kconfig"

endif # SOC_SERIES_STM32N6X
```

### `soc/st/stm32/stm32n6x/Kconfig.soc` (Zephyr)

```kconfig
config SOC_SERIES_STM32N6X
	bool
	select SOC_FAMILY_STM32

config SOC_SERIES
	default "stm32n6x" if SOC_SERIES_STM32N6X

config SOC_STM32N657XX
	bool
	select SOC_SERIES_STM32N6X

config SOC
	default "stm32n657xx" if SOC_STM32N657XX
```

### NuttX Kconfig 等效配置 (`arch/arm/src/stm32n6/Kconfig`)

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
	select ARCH_ARMV8M
	---help---
		STMicroelectronics STM32N6 Series (ARM Cortex-M55).
		ARMv8.1-M Mainline with DSP, MVE, FPU, I/D Cache, TrustZone, RIF.

if ARCH_CHIP_STM32N6

config STM32N6_RIF_OPEN
	bool "Configure RIF with full access"
	default y
	depends on ARCH_TRUSTZONE_SECURE
	---help---
		Configure the Resource Isolation Framework to allow full access
		from the secure world to all SoC resources.

config STM32N6_NPU
	bool "Enable Neural-ART NPU"
	default y
	---help---
		Enable the Neural-ART neural network accelerator.

config STM32N6_NPU_CACHE
	bool "Enable NPU Cache (CACHEAXI)"
	default y
	depends on STM32N6_NPU
	---help---
		Enable the NPU cache for improved performance.

endif # ARCH_CHIP_STM32N6

config ARCH_CHIP_STM32N657XX
	bool "STM32N657xx"
	select ARCH_CHIP_STM32N6
	---help---
		STM32N657xx (Cortex-M55) with NPU, LTDC, Ethernet, DCMIPP.
```

## SoC 初始化代码

### `soc/st/stm32/stm32n6x/soc.c` (Zephyr)

```c
#include <zephyr/device.h>
#include <zephyr/init.h>
#include <zephyr/cache.h>
#include <zephyr/logging/log.h>

#include <stm32_ll_bus.h>
#include <stm32_ll_pwr.h>
#include <stm32_ll_icache.h>

#include <cmsis_core.h>

#define LOG_LEVEL CONFIG_SOC_LOG_LEVEL
LOG_MODULE_REGISTER(soc);

extern char _vector_start[];
void *g_pfnVectors = (void *)_vector_start;

#if defined(CONFIG_SOC_RESET_HOOK)
void soc_reset_hook(void)
{
	/* This is provided by STM32Cube HAL */
	SystemInit();
}
#endif

#define RIF_MASTER_CID1_SEC_PRIV(device)	\
	do {										\
		RIMC_MasterConfig_t rimc = {						\
			.MasterCID = RIF_CID_1,						\
			.SecPriv = RIF_ATTRIBUTE_SEC | RIF_ATTRIBUTE_PRIV,		\
		};									\
		HAL_RIF_RIMC_ConfigMasterAttributes(RIF_MASTER_INDEX_##device, &rimc);	\
	} while (0)

#define RIF_SLAVE_SEC_PRIV(device)	\
	HAL_RIF_RISC_SetSlaveSecureAttributes(RIF_RISC_PERIPH_INDEX_##device,		\
					      RIF_ATTRIBUTE_SEC | RIF_ATTRIBUTE_PRIV)

static void soc_rif_config(void)
{
#if defined(CONFIG_TRUSTED_EXECUTION_SECURE)
	/* Enable the clock for the RIFSC (RIF Security Controller) */
	__HAL_RCC_RIFSC_CLK_ENABLE();

	/* ADC */
	RIF_SLAVE_SEC_PRIV(ADC12);
	/* DCMIPP */
	RIF_MASTER_CID1_SEC_PRIV(DCMIPP);
	RIF_SLAVE_SEC_PRIV(DCMIPP);
	/* DMA2D */
	RIF_MASTER_CID1_SEC_PRIV(DMA2D);
	RIF_SLAVE_SEC_PRIV(DMA2D);
	/* ETH */
	RIF_MASTER_CID1_SEC_PRIV(ETH1);
	RIF_SLAVE_SEC_PRIV(ETH1);
	/* JPEG */
	RIF_SLAVE_SEC_PRIV(JPEG);
	/* LTDC Layer 1 */
	RIF_MASTER_CID1_SEC_PRIV(LTDC1);
	RIF_SLAVE_SEC_PRIV(LTDCL1);
#ifdef NPU_PRESENT
	/* NPU */
	RIF_MASTER_CID1_SEC_PRIV(NPU);
	RIF_SLAVE_SEC_PRIV(NPU);
#endif
	/* VENC */
	RIF_MASTER_CID1_SEC_PRIV(VENC);
	RIF_SLAVE_SEC_PRIV(VENC);
#endif /* CONFIG_TRUSTED_EXECUTION_SECURE */
}

/**
 * @brief Perform basic hardware initialization at boot.
 *
 * This needs to be run from the very beginning.
 *
 * @return 0
 */
void soc_early_init_hook(void)
{
	/* Enable caches */
	sys_cache_instr_enable();
	sys_cache_data_enable();

	/* Update CMSIS SystemCoreClock variable (HCLK) */
	/* At reset, system core clock is set to 64 MHz from HSI */
	SystemCoreClock = 64000000;

	/* Enable PWR */
	LL_AHB4_GRP1_EnableClock(LL_AHB4_GRP1_PERIPH_PWR);

	/* Set the main internal Regulator output voltage for best performance */
	LL_PWR_SetRegulVoltageScaling(LL_PWR_REGU_VOLTAGE_SCALE0);

	/* Enable IOs */
	LL_PWR_EnableVddIO2();
	LL_PWR_EnableVddIO3();
	LL_PWR_EnableVddIO4();
	LL_PWR_EnableVddIO5();

	/* RIF configuration */
	if (IS_ENABLED(CONFIG_STM32N6_RIF_OPEN)) {
		soc_rif_config();
	}
}
```

### NuttX 等效实现 (`arch/arm/src/stm32n6/stm32n6_soc.c`)

```c
/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_soc.c
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

#include <stdint.h>

#include <arch/irq.h>

#include "arm_internal.h"
#include "stm32n6.h"
#include "stm32n6_rcc.h"
#include "stm32n6_pwr.h"

#ifdef CONFIG_STM32N6_RIF_OPEN
#  include "stm32n6_rif.h"
#endif

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Reference: Zephyr soc/st/stm32/stm32n6x/soc.c */

/* RIF Master Configuration Macro */
#define RIF_MASTER_CID1_SEC_PRIV(d) \
  do { \
    /* Configure RIMC for device with CID1, Secure, Privileged */ \
    /* Reference: Zephyr RIF_MASTER_CID1_SEC_PRIV macro */ \
  } while (0)

/* RIF Slave Configuration Macro */
#define RIF_SLAVE_SEC_PRIV(d) \
  do { \
    /* Configure RISC for device with Secure, Privileged */ \
    /* Reference: Zephyr RIF_SLAVE_SEC_PRIV macro */ \
  } while (0)

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32n6_rif_config
 *
 * Description:
 *   Configure the Resource Isolation Framework (RIF) for secure access.
 *   Reference: Zephyr soc_rif_config() in soc/st/stm32/stm32n6x/soc.c
 *
 ****************************************************************************/

#ifdef CONFIG_STM32N6_RIF_OPEN
static void stm32n6_rif_config(void)
{
  /* Enable RIFSC clock */

  modifyreg32(STM32_RCC_AHB5ENR, 0, RCC_AHB5ENR_RIFSC);

#ifdef CONFIG_STM32N6_ADC
  /* ADC12 Configuration */
  RIF_SLAVE_SEC_PRIV(ADC12);
#endif

#ifdef CONFIG_STM32N6_DCMIPP
  /* DCMIPP Configuration */
  RIF_MASTER_CID1_SEC_PRIV(DCMIPP);
  RIF_SLAVE_SEC_PRIV(DCMIPP);
#endif

#ifdef CONFIG_STM32N6_DMA2D
  /* DMA2D Configuration */
  RIF_MASTER_CID1_SEC_PRIV(DMA2D);
  RIF_SLAVE_SEC_PRIV(DMA2D);
#endif

#ifdef CONFIG_STM32N6_ETH
  /* ETH1 Configuration */
  RIF_MASTER_CID1_SEC_PRIV(ETH1);
  RIF_SLAVE_SEC_PRIV(ETH1);
#endif

#ifdef CONFIG_STM32N6_JPEG
  /* JPEG Configuration */
  RIF_SLAVE_SEC_PRIV(JPEG);
#endif

#ifdef CONFIG_STM32N6_LTDC
  /* LTDC Configuration */
  RIF_MASTER_CID1_SEC_PRIV(LTDC1);
  RIF_SLAVE_SEC_PRIV(LTDCL1);
#endif

#ifdef CONFIG_STM32N6_NPU
  /* NPU Configuration */
  RIF_MASTER_CID1_SEC_PRIV(NPU);
  RIF_SLAVE_SEC_PRIV(NPU);
#endif

#ifdef CONFIG_STM32N6_VENC
  /* VENC Configuration */
  RIF_MASTER_CID1_SEC_PRIV(VENC);
  RIF_SLAVE_SEC_PRIV(VENC);
#endif
}
#endif /* CONFIG_STM32N6_RIF_OPEN */

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32n6_soc_early_init
 *
 * Description:
 *   Perform early SoC initialization. Called from start-up code.
 *   Reference: Zephyr soc_early_init_hook() in soc/st/stm32/stm32n6x/soc.c
 *
 ****************************************************************************/

void stm32n6_soc_early_init(void)
{
  /* 1. Enable I-Cache and D-Cache
   * Reference: Zephyr sys_cache_instr_enable(), sys_cache_data_enable()
   */

#ifdef CONFIG_ARMV8M_ICACHE
  up_enable_icache();
#endif

#ifdef CONFIG_ARMV8M_DCACHE
  up_enable_dcache();
#endif

  /* 2. Update SystemCoreClock (HSI = 64 MHz at reset)
   * Reference: Zephyr soc_early_init_hook()
   */

  SystemCoreClock = 64000000;

  /* 3. Enable PWR clock
   * Reference: Zephyr LL_AHB4_GRP1_EnableClock(LL_AHB4_GRP1_PERIPH_PWR)
   */

  modifyreg32(STM32_RCC_AHB4ENR, 0, RCC_AHB4ENR_PWR);

  /* 4. Set regulator voltage scaling for best performance
   * Reference: Zephyr LL_PWR_SetRegulVoltageScaling(LL_PWR_REGU_VOLTAGE_SCALE0)
   */

  modifyreg32(STM32_PWR_CSR1, PWR_CSR1_VOS_MASK,
              PWR_CSR1_VOS_SCALE0);

  /* 5. Enable IO supply configuration
   * Reference: Zephyr LL_PWR_EnableVddIOx()
   */

  modifyreg32(STM32_PWR_CR2, 0,
              PWR_CR2_IOSV1EN | PWR_CR2_IOSV2EN |
              PWR_CR2_IOSV3EN | PWR_CR2_IOSV4EN |
              PWR_CR2_IOSV5EN);

#ifdef CONFIG_STM32N6_RIF_OPEN
  /* 6. Configure RIF (Resource Isolation Framework) */

  stm32n6_rif_config();
#endif
}

/****************************************************************************
 * Name: stm32n6_soc_reset_hook
 *
 * Description:
 *   SoC reset hook called before C runtime initialization.
 *   Reference: Zephyr soc_reset_hook() in soc/st/stm32/stm32n6x/soc.c
 *
 ****************************************************************************/

#ifdef CONFIG_SOC_RESET_HOOK
void stm32n6_soc_reset_hook(void)
{
  /* Call STM32Cube HAL SystemInit() */

  SystemInit();
}
#endif
```

## 内存映射 (DTS)

### `dts/arm/st/n6/stm32n6.dtsi` - CPU 定义 (Zephyr)

```dts
#include <arm/armv8.1-m.dtsi>

/ {
    cpus {
        #address-cells = <1>;
        #size-cells = <0>;

        cpu0: cpu@0 {
            device_type = "cpu";
            compatible = "arm,cortex-m55";
            reg = <0>;
            #address-cells = <1>;
            #size-cells = <1>;

            mpu: mpu@e000ed90 {
                compatible = "arm,armv8.1m-mpu";
                reg = <0xe000ed90 0x40>;
            };
        };
    };

    /* AXISRAM - AXI SRAM */
    axisram12: axisram12@24000000 {
        #address-cells = <1>;
        #size-cells = <1>;
        /* FLEXMEM, AXISRAM1 and AXISRAM2 */
        reg = <0x24000000 0x01c00000>;
        ranges = <0x0 0x34000000 DT_SIZE_M(2)>;

        axisram1: memory@0 {
            compatible = "zephyr,memory-region", "mmio-sram";
            zephyr,memory-region = "AXISRAM1";
        };

        axisram2: memory@180400 {
            compatible = "zephyr,memory-region", "mmio-sram";
            zephyr,memory-region = "AXISRAM2";
        };
    };

    /* 时钟定义 */
    clocks {
        clk_hse: clk-hse {
            #clock-cells = <0>;
            compatible = "st,stm32n6-hse-clock";
            status = "disabled";
        };

        clk_hsi: clk-hsi {
            #clock-cells = <0>;
            compatible = "st,stm32h7-hsi-clock";
            clock-frequency = <DT_FREQ_M(64)>;
            status = "disabled";
        };

        /* PLL 锁相环 */
        pll1: pll: pll {
            #clock-cells = <0>;
            compatible = "st,stm32n6-pll-clock";
            status = "disabled";
        };
        pll2: pll2 { ... };
        pll3: pll3 { ... };
        pll4: pll4 { ... };

        /* CPU 时钟开关 */
        cpusw: cpusw {
            #clock-cells = <0>;
            compatible = "st,stm32n6-cpu-clock-mux", "st,stm32-clock-mux";
            status = "disabled";
        };

        /* 外设时钟域 (IC1-IC20) */
        ic1: ic1 { ... };
        /* ... */
    };

    /* 外设总线 */
    soc {
        peripherals: peripherals@40000000 {
            #address-cells = <1>;
            #size-cells = <1>;
            reg = <0x40000000 0x20000000>;
            ranges = <0x0 0x50000000 0x10000000>;

            /* RCC 复位和时钟控制 */
            rcc: rcc@6028000 {
                compatible = "st,stm32n6-rcc";
                clocks-controller;
                #clock-cells = <2>;
                reg = <0x6028000 0x2000>;
            };

            /* EXTI 外部中断控制器 */
            exti: interrupt-controller@6025000 {
                compatible = "st,stm32g0-exti", "st,stm32-exti";
                interrupt-controller;
                #interrupt-cells = <1>;
                reg = <0x6025000 0x400>;
                num-lines = <96>;
            };

            /* GPIO 端口 A-Q */
            pinctrl: pin-controller@6020000 {
                compatible = "st,stm32n6-pinctrl", "st,stm32-pinctrl";
                reg = <0x6020000 0x2000>;

                gpioa: gpio@6020000 { ... }
                gpiob: gpio@6020400 { ... }
                /* ... gpioh, gpion, gpioo, gpiop, gpioq */
            };

            /* RAMCFG SRAM 配置 */
            ramcfg_sram3_axi: ramcfg@2023100 { ... }
            ramcfg_sram4_axi: ramcfg@2023180 { ... }
            ramcfg_sram5_axi: ramcfg@2023200 { ... }
            ramcfg_sram6_axi: ramcfg@2023280 { ... }
        };
    };
};
```

### NuttX 内存映射头文件 (`hardware/stm32n6_memorymap.h`)

```c
#ifndef __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_MEMORYMAP_H
#define __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_MEMORYMAP_H

/****************************************************************************
 * Peripheral Base Addresses
 ****************************************************************************/

/* Reference: Zephyr dts/arm/st/n6/stm32n6.dtsi */
/* Secure region (default): 0x50000000 */

#define STM32N6_PERIPH_BASE        0x50000000

/* RCC (Reset and Clock Control) - Reference: rcc@6028000 */
#define STM32N6_RCC_BASE           (STM32N6_PERIPH_BASE + 0x002800)

/* PWR (Power Control) */
#define STM32N6_PWR_BASE           (STM32N6_PERIPH_BASE + 0x002C00)

/* EXTI (External Interrupt) - Reference: exti@6025000 */
#define STM32N6_EXTI_BASE          (STM32N6_PERIPH_BASE + 0x0025000)

/* GPIO Ports - Reference: pinctrl@6020000 */
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

/* USART/UART - Reference: serial@2001000 etc. */
#define STM32N6_USART1_BASE        (STM32N6_PERIPH_BASE + 0x00010000)
#define STM32N6_USART2_BASE        (STM32N6_PERIPH_BASE + 0x00004400)
#define STM32N6_USART3_BASE        (STM32N6_PERIPH_BASE + 0x00004800)
#define STM32N6_UART4_BASE         (STM32N6_PERIPH_BASE + 0x00004C00)
#define STM32N6_UART5_BASE         (STM32N6_PERIPH_BASE + 0x00005000)
#define STM32N6_USART6_BASE        (STM32N6_PERIPH_BASE + 0x00010400)
#define STM32N6_UART7_BASE         (STM32N6_PERIPH_BASE + 0x00007800)
#define STM32N6_UART8_BASE         (STM32N6_PERIPH_BASE + 0x00007C00)
#define STM32N6_UART9_BASE         (STM32N6_PERIPH_BASE + 0x00010800)
#define STM32N6_USART10_BASE       (STM32N6_PERIPH_BASE + 0x00010C00)

/* I2C - Reference: i2c@5400 etc. */
#define STM32N6_I2C1_BASE          (STM32N6_PERIPH_BASE + 0x00005400)
#define STM32N6_I2C2_BASE          (STM32N6_PERIPH_BASE + 0x00005800)
#define STM32N6_I2C3_BASE          (STM32N6_PERIPH_BASE + 0x00005C00)
#define STM32N6_I2C4_BASE          (STM32N6_PERIPH_BASE + 0x00001C00)

/* SPI - Reference: spi@2003000 etc. */
#define STM32N6_SPI1_BASE          (STM32N6_PERIPH_BASE + 0x00013000)
#define STM32N6_SPI2_BASE          (STM32N6_PERIPH_BASE + 0x00003800)
#define STM32N6_SPI3_BASE          (STM32N6_PERIPH_BASE + 0x00003C00)
#define STM32N6_SPI4_BASE          (STM32N6_PERIPH_BASE + 0x00013400)
#define STM32N6_SPI5_BASE          (STM32N6_PERIPH_BASE + 0x00015000)
#define STM32N6_SPI6_BASE          (STM32N6_PERIPH_BASE + 0x00011400)

/* I3C - Reference: i3c@6000 etc. */
#define STM32N6_I3C1_BASE          (STM32N6_PERIPH_BASE + 0x00006000)
#define STM32N6_I3C2_BASE          (STM32N6_PERIPH_BASE + 0x00006400)

/* FDCAN - Reference: can@a000 etc. */
#define STM32N6_FDCAN1_BASE        (STM32N6_PERIPH_BASE + 0x0000A000)
#define STM32N6_FDCAN2_BASE        (STM32N6_PERIPH_BASE + 0x0000A400)
#define STM32N6_FDCAN3_BASE        (STM32N6_PERIPH_BASE + 0x0000E800)

/* ADC - Reference: adc@22000 etc. */
#define STM32N6_ADC1_BASE          (STM32N6_PERIPH_BASE + 0x00022000)
#define STM32N6_ADC2_BASE          (STM32N6_PERIPH_BASE + 0x00022100)

/* GPDMA - Reference: dma@21000 */
#define STM32N6_GPDMA1_BASE        (STM32N6_PERIPH_BASE + 0x00021000)

/* LTDC (LCD-TFT Display Controller) - Reference: ltdc@8001000 */
#define STM32N6_LTDC_BASE          (STM32N6_PERIPH_BASE + 0x00011000)

/* DMA2D (2D Graphics Accelerator) */
#define STM32N6_DMA2D_BASE         (STM32N6_PERIPH_BASE + 0x00011400)

/* DCMIPP (Digital Camera Interface Parallel Port) - Reference: dcmipp@8002000 */
#define STM32N6_DCMIPP_BASE        (STM32N6_PERIPH_BASE + 0x000802000)

/* Ethernet - Reference: ethernet@8036000 */
#define STM32N6_ETH_BASE           (STM32N6_PERIPH_BASE + 0x000803600)

/* SDMMC - Reference: sdmmc@8027000 etc. */
#define STM32N6_SDMMC1_BASE        (STM32N6_PERIPH_BASE + 0x000802700)
#define STM32N6_SDMMC2_BASE        (STM32N6_PERIPH_BASE + 0x0008026800)

/* XSPI - Reference: xspi@8025000 etc. */
#define STM32N6_XSPI1_BASE         (STM32N6_PERIPH_BASE + 0x000802500)
#define STM32N6_XSPI2_BASE         (STM32N6_PERIPH_BASE + 0x000802A000)
#define STM32N6_XSPI3_BASE         (STM32N6_PERIPH_BASE + 0x000802D000)

/* NPU (Neural-ART Neural Processing Unit) - Reference: npu@80e0000 */
#define STM32N6_NPU_BASE           (STM32N6_PERIPH_BASE + 0x00080E0000)
#define STM32N6_NPU_SIZE           (128 * 1024)  /* 128 KB */

/* NPU Cache - Reference: cache-controller@80dfc00 */
#define STM32N6_NPU_CACHE_BASE     (STM32N6_PERIPH_BASE + 0x00080DFC00)

/* JPEG Codec - Reference: codec@8023000 */
#define STM32N6_JPEG_BASE          (STM32N6_PERIPH_BASE + 0x0008023000)

/* VENC (Video Encoder) - Reference: venc@8005000 */
#define STM32N6_VENC_BASE          (STM32N6_PERIPH_BASE + 0x0008005000)

/* USB OTG HS - Reference: otghs@8040000 etc. */
#define STM32N6_OTG_HS1_BASE       (STM32N6_PERIPH_BASE + 0x0008040000)
#define STM32N6_OTG_HS2_BASE       (STM32N6_PERIPH_BASE + 0x0008080000)

/* USB PHY - Reference: usbphyc@803fc00 etc. */
#define STM32N6_USBPHYC1_BASE      (STM32N6_PERIPH_BASE + 0x000803FC00)
#define STM32N6_USBPHYC2_BASE      (STM32N6_PERIPH_BASE + 0x00080C0000)

/* RNG - Reference: rng@4020000 */
#define STM32N6_RNG_BASE           (STM32N6_PERIPH_BASE + 0x00020000)

/* CRC - Reference: crc@6024c00 */
#define STM32N6_CRC_BASE           (STM32N6_PERIPH_BASE + 0x0024C00)

/* BSEC (Boot Security) - Reference: efuse@6009000 */
#define STM32N6_BSEC_BASE          (STM32N6_PERIPH_BASE + 0x0009000)

/* IWDG/WWWD - Reference: watchdog@6004800 etc. */
#define STM32N6_IWDG_BASE          (STM32N6_PERIPH_BASE + 0x0004800)
#define STM32N6_WWDG_BASE          (STM32N6_PERIPH_BASE + 0x00002C00)

/****************************************************************************
 * AXISRAM Memory (AXI SRAM) - 6 banks, total 2.75 MB
 * Reference: Zephyr axisram12 and ramcfg nodes
 ****************************************************************************/

#define STM32N6_AXISRAM1_BASE      0x34000000  /* 512 KB */
#define STM32N6_AXISRAM1_SIZE      (512 * 1024)

#define STM32N6_AXISRAM2_BASE      0x34180000  /* 512 KB */
#define STM32N6_AXISRAM2_SIZE      (512 * 1024)

#define STM32N6_AXISRAM3_BASE      0x34200000  /* 448 KB */
#define STM32N6_AXISRAM3_SIZE      (448 * 1024)

#define STM32N6_AXISRAM4_BASE      0x34270000  /* 448 KB */
#define STM32N6_AXISRAM4_SIZE      (448 * 1024)

#define STM32N6_AXISRAM5_BASE      0x342E0000  /* 448 KB */
#define STM32N6_AXISRAM5_SIZE      (448 * 1024)

#define STM32N6_AXISRAM6_BASE      0x34350000  /* 448 KB */
#define STM32N6_AXISRAM6_SIZE      (448 * 1024)

/* Total AXISRAM: 2.75 MB */

/****************************************************************************
 * External Memory (via XSPI)
 ****************************************************************************/

/* XSPI1 Memory Mapped Region - PSRAM */
#define STM32N6_XSPI1_MEM_BASE     0x90000000  /* Up to 256 MB */

/* XSPI2 Memory Mapped Region - Flash */
#define STM32N6_XSPI2_MEM_BASE     0x70000000  /* Up to 256 MB */

/* XSPI3 Memory Mapped Region */
#define STM32N6_XSPI3_MEM_BASE     0x80000000  /* Up to 256 MB */

#endif /* __ARCH_ARM_SRC_STM32N6_HARDWARE_STM32N6_MEMORYMAP_H */
```

## 时钟配置

### `include/zephyr/dt-bindings/clock/stm32n6_clock.h` (Zephyr)

```c
/* Domain clocks */

/* System clock - defined in stm32_common_clocks.h */

/* Fixed clocks  */
#define STM32_SRC_HSE       (STM32_SRC_LSI + 1)
#define STM32_SRC_HSI       (STM32_SRC_HSE + 1)
#define STM32_SRC_MSI       (STM32_SRC_HSI + 1)

/* PLL outputs */
#define STM32_SRC_PLL1      (STM32_SRC_MSI + 1)
#define STM32_SRC_PLL2      (STM32_SRC_PLL1 + 1)
#define STM32_SRC_PLL3      (STM32_SRC_PLL2 + 1)
#define STM32_SRC_PLL4      (STM32_SRC_PLL3 + 1)

/* Clock muxes */
#define STM32_SRC_CKPER     (STM32_SRC_PLL4 + 1)
#define STM32_SRC_IC1       (STM32_SRC_CKPER + 1)
#define STM32_SRC_IC2       (STM32_SRC_IC1 + 1)
/* ... IC3 to IC20 ... */

/* Bus clocks */
#define STM32_CLOCK_BUS_MISC    0x248
#define STM32_CLOCK_BUS_MEM     0x24C
#define STM32_CLOCK_BUS_AHB1    0x250
#define STM32_CLOCK_BUS_AHB2    0x254
#define STM32_CLOCK_BUS_AHB3    0x258
#define STM32_CLOCK_BUS_AHB4    0x25C
#define STM32_CLOCK_BUS_AHB5    0x260
#define STM32_CLOCK_BUS_APB1    0x264
#define STM32_CLOCK_BUS_APB1_2  0x268
#define STM32_CLOCK_BUS_APB2    0x26C
#define STM32_CLOCK_BUS_APB3    0x270
#define STM32_CLOCK_BUS_APB4    0x274
#define STM32_CLOCK_BUS_APB4_2  0x278
#define STM32_CLOCK_BUS_APB5    0x27C

/* Peripheral Clock Selection - CCIPRx registers */
#define CCIPR1_REG      0x144
#define CCIPR2_REG      0x148
/* ... */

/* USART Clock Selection */
#define USART1_SEL(val)     STM32_DT_CLOCK_SELECT((val), 2, 0, CCIPR13_REG)
#define USART2_SEL(val)     STM32_DT_CLOCK_SELECT((val), 6, 4, CCIPR13_REG)
/* ... */

/* I2C Clock Selection */
#define I2C1_SEL(val)       STM32_DT_CLOCK_SELECT((val), 2, 0, CCIPR4_REG)
/* ... */

/* SPI Clock Selection */
#define SPI1_SEL(val)       STM32_DT_CLOCK_SELECT((val), 6, 4, CCIPR9_REG)
/* ... */

/* CPU Clock Switch - CFGR1 register */
#define CPU_SEL(val)        STM32_DT_CLOCK_SELECT((val), 17, 16, CFGR1_REG)
```

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
│    │ (HSE)  │           │ (HSI)  │           │ (HSE)  │        │
│    │ 800MHz │           │ 400MHz │           │ 600MHz │        │
│    └────────┘           └────────┘           └────────┘        │
│         │                    │                    │             │
│    ┌────┴───┐           ┌────┴───┐           ┌────┴───┐        │
│    │ PLL4   │           │        │           │        │        │
│    │ (HSI)  │           │        │           │        │        │
│    │ 300MHz │           │        │           │        │        │
│    └────────┘           └────────┘           └────────┘        │
│         │                    │                    │             │
│         └────────────────────┴────────────────────┘             │
│                              │                                    │
│         ┌────────────────────┼────────────────────┐             │
│         │                    │                    │             │
│    ┌────┴───┐           ┌────┴───┐           ┌────┴───┐        │
│    │ IC1-10 │           │ IC11-20│           │ CKPER  │        │
│    │(CPU/外设)│          │(低功耗) │           │(外设)  │        │
│    └────────┘           └────────┘           └────────┘        │
│         │                    │                    │             │
│         └────────────────────┴────────────────────┘             │
│                              │                                    │
│         ┌────────────────────┼────────────────────┐             │
│         │                    │                    │             │
│    ┌────┴───┐           ┌────┴───┐           ┌────┴───┐        │
│    │ CPU    │           │ AHBx   │           │ APBx   │        │
│    │ 800MHz │           │ 400MHz │           │ 200MHz │        │
│    └────────┘           └────────┘           └────────┘        │
│                                                                    │
│  时钟源选择:                                                        │
│  - HSI: 64 MHz 内部 RC 振荡器                                       │
│  - HSE: 外部晶体振荡器 (默认 48 MHz)                                 │
│  - PLL1-4: 4 个独立 PLL                                             │
│  - IC1-20: 20 个独立的互联时钟域                                     │
│  - CKPER: 外设时钟                                                  │
│                                                                    │
│  每个外设可从多个时钟源中选择: HSI, HSE, PLL1-PLL4, ICx, CKPER        │
│                                                                    │
└─────────────────────────────────────────────────────────────────┘
```

## 外设摘要

| 外设 | 数量 | 基地址 | 说明 |
|------|------|--------|------|
| GPIO | 12 端口 | 0x50200000 | A, B, C, D, E, F, G, H, N, O, P, Q |
| USART/UART | 10 通道 | 0x50004400+ | USART1-6, UART7-10 |
| I2C | 4 通道 | 0x50005400+ | I2C1-I2C4 |
| SPI/I2S | 6 通道 | 0x50003800+ | SPI1-SPI6, I2S1-I2S3, I2S6 |
| I3C | 2 通道 | 0x50006000+ | I3C1-I3C2 |
| FDCAN | 3 通道 | 0x5000A000+ | FDCAN1-FDCAN3 |
| ADC | 2 通道 | 0x50022000+ | ADC1-ADC2 (12-bit) |
| GPDMA | 1 控制器 | 0x50021000 | 16 通道 DMA |
| LTDC | 1 控制器 | 0x50011000 | LCD-TFT 显示控制器 |
| DMA2D | 1 控制器 | 0x50011400 | 2D 绘图加速器 |
| DCMIPP | 1 控制器 | 0x5802000 | 数字摄像头接口 |
| JPEG | 1 控制器 | 0x58023000 | JPEG 编解码器 |
| ETH | 1 控制器 | 0x58036000 | 以太网 MAC |
| NPU | 1 核 | 0x580E0000 | 神经网络处理器 (128 KB) |
| VENC | 1 控制器 | 0x58005000 | 视频编码器 |
| USB OTG HS | 2 通道 | 0x58040000+ | USB OTG HS 1/2 |
| SDMMC | 2 通道 | 0x58027000+ | SD/MMC 控制器 |
| XSPI | 3 通道 | 0x58025000+ | Octo-SPI 控制器 |
| RNG | 1 通道 | 0x50020000 | 真随机数发生器 |
| CRC | 1 通道 | 0x5024C00 | CRC 计算单元 |
| IWDG | 1 通道 | 0x5004800 | 独立看门狗 |
| WWDG | 1 通道 | 0x5002C00 | 窗口看门狗 |

## NPU 驱动

### `soc/st/stm32/stm32n6x/npu/npu_stm32n6.c` (Zephyr)

```c
#define DT_DRV_COMPAT st_stm32_npu

#include <errno.h>
#include <zephyr/device.h>
#include <zephyr/drivers/reset.h>
#include <zephyr/init.h>
#include <soc.h>
#include <zephyr/drivers/clock_control/stm32_clock_control.h>

/* Read-only driver configuration */
struct npu_stm32_cfg {
	/* Clock configuration. */
	struct stm32_pclken pclken_npu;
	struct stm32_pclken pclken_cacheaxi;
	/* Reset configuration */
	const struct reset_dt_spec reset_npu;
	const struct reset_dt_spec reset_cacheaxi;
};

static int npu_stm32_init(const struct device *dev)
{
	const struct device *const clk = DEVICE_DT_GET(STM32_CLOCK_CONTROL_NODE);
	const struct npu_stm32_cfg *cfg = dev->config;

	if (!device_is_ready(clk)) {
		return -ENODEV;
	}

	if (clock_control_on(clk, (clock_control_subsys_t)&cfg->pclken_npu) != 0) {
		return -EIO;
	}

	if (clock_control_on(clk, (clock_control_subsys_t)&cfg->pclken_cacheaxi) != 0) {
		return -EIO;
	}

	if (!device_is_ready(cfg->reset_npu.dev)) {
		return -ENODEV;
	}

	/* Reset timer to default state using RCC */
	(void)reset_line_toggle_dt(&cfg->reset_npu);
	(void)reset_line_toggle_dt(&cfg->reset_cacheaxi);

	return 0;
}

static const struct npu_stm32_cfg npu_stm32_cfg = {
	.pclken_npu = STM32_DT_INST_CLOCK_INFO(0),
	.reset_npu = RESET_DT_SPEC_INST_GET(0),
	/*
	 * Even if npu_cache node is disabled, its clocks must be enabled for NPU operation.
	 * This is why we need to get clock and reset line from the npu_cache node.
	 */
	.pclken_cacheaxi = STM32_CLOCK_INFO(0, DT_NODELABEL(npu_cache)),
	.reset_cacheaxi = RESET_DT_SPEC_GET(DT_NODELABEL(npu_cache)),
};

DEVICE_DT_INST_DEFINE(0, npu_stm32_init, NULL, NULL, &npu_stm32_cfg, POST_KERNEL,
		      CONFIG_STM32N6_NPU_INIT_PRIORITY, NULL);
```

### NuttX NPU 驱动 (`arch/arm/src/stm32n6/stm32n6_npu.c`)

```c
/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_npu.c
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

#include <stdint.h>
#include <errno.h>

#include <nuttx/arch.h>

#include "stm32n6.h"
#include "stm32n6_rcc.h"
#include "stm32n6_memorymap.h"
#include "arm_internal.h"

#ifdef CONFIG_STM32N6_NPU

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Reference: Zephyr soc/st/stm32/stm32n6x/npu/npu_stm32n6.c */

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32n6_npu_enable_clocks
 *
 * Description:
 *   Enable NPU and NPU Cache clocks.
 *   Reference: Zephyr npu_stm32_init() clock_control_on() calls
 *
 ****************************************************************************/

static void stm32n6_npu_enable_clocks(void)
{
  /* Enable NPU clock */

  modifyreg32(STM32_RCC_AHB5ENR, 0, RCC_AHB5ENR_NPU);

  /* Enable NPU Cache (CACHEAXI) clock
   * Even if npu_cache node is disabled, its clocks must be enabled
   * for NPU operation.
   */

  modifyreg32(STM32_RCC_AHB5ENR, 0, RCC_AHB5ENR_CACHEAXI);
}

/****************************************************************************
 * Name: stm32n6_npu_reset
 *
 * Description:
 *   Reset NPU and NPU Cache to default state.
 *   Reference: Zephyr reset_line_toggle_dt() calls
 *
 ****************************************************************************/

static void stm32n6_npu_reset(void)
{
  /* Assert NPU reset */

  modifyreg32(STM32_RCC_AHB5RSTR, 0, RCC_AHB5RSTR_NPU);

  /* Deassert NPU reset */

  modifyreg32(STM32_RCC_AHB5RSTR, RCC_AHB5RSTR_NPU, 0);

  /* Assert CACHEAXI reset */

  modifyreg32(STM32_RCC_AHB5RSTR, 0, RCC_AHB5RSTR_CACHEAXI);

  /* Deassert CACHEAXI reset */

  modifyreg32(STM32_RCC_AHB5RSTR, RCC_AHB5RSTR_CACHEAXI, 0);
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32n6_npu_init
 *
 * Description:
 *   Initialize the Neural-ART NPU.
 *   Reference: Zephyr npu_stm32_init() in soc/st/stm32/stm32n6x/npu/npu_stm32n6.c
 *
 * Returned Value:
 *   OK on success; Negated errno on failure.
 *
 ****************************************************************************/

int stm32n6_npu_init(void)
{
  /* Enable clocks */

  stm32n6_npu_enable_clocks();

  /* Reset NPU to default state */

  stm32n6_npu_reset();

  return OK;
}

#endif /* CONFIG_STM32N6_NPU */
```

## AXISRAM 驱动

### `drivers/misc/stm32n6_axisram/stm32n6_axisram.c` (Zephyr)

```c
#include <errno.h>
#include <zephyr/device.h>
#include <zephyr/kernel.h>
#include <zephyr/init.h>
#include <soc.h>
#include <zephyr/drivers/clock_control/stm32_clock_control.h>

#define DT_DRV_COMPAT st_stm32n6_ramcfg

/* Read-only driver configuration */
struct axisram_stm32_cfg {
	/* RAMCFG instance. */
	RAMCFG_TypeDef *base;
	/* SRAM Clock configuration. */
	struct stm32_pclken pclken_axisram;
	/* RAMCFG Clock configuration. */
	struct stm32_pclken pclken_ramcfg;
};

static int axisram_stm32_init(const struct device *dev)
{
	const struct axisram_stm32_cfg *cfg = dev->config;
	RAMCFG_HandleTypeDef ramcfg = {0};
	/* enable clock for subsystem */
	const struct device *const clk = DEVICE_DT_GET(STM32_CLOCK_CONTROL_NODE);

	if (clock_control_on(clk, (clock_control_subsys_t) &cfg->pclken_ramcfg) != 0) {
		return -EIO;
	}

	if (clock_control_on(clk, (clock_control_subsys_t) &cfg->pclken_axisram) != 0) {
		return -EIO;
	}

	ramcfg.Instance = cfg->base;
	HAL_RAMCFG_EnableAXISRAM(&ramcfg);

	return 0;
}

/* ... instantiation macros ... */
```

### NuttX AXISRAM 驱动 (`arch/arm/src/stm32n6/stm32n6_axisram.c`)

```c
/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_axisram.c
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

#include <stdint.h>
#include <errno.h>

#include <nuttx/arch.h>

#include "stm32n6.h"
#include "stm32n6_rcc.h"
#include "stm32n6_memorymap.h"
#include "arm_internal.h"

#ifdef CONFIG_STM32N6_AXISRAM

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Reference: Zephyr drivers/misc/stm32n6_axisram/stm32n6_axisram.c */

/* RAMCFG Register Offsets */
#define STM32_RAMCFG_CR_OFFSET      0x0000  /* Control Register */
#define STM32_RAMCFG_SR_OFFSET      0x0004  /* Status Register */

/* RAMCFG Control Register Bits */
#define RAMCFG_CR_AE                (1 << 0)  /* AXISRAM Enable */

/****************************************************************************
 * Private Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32n6_axisram_enable_clocks
 *
 * Description:
 *   Enable AXISRAM and RAMCFG clocks.
 *   Reference: Zephyr axisram_stm32_init() clock_control_on() calls
 *
 * Input Parameters:
 *   bank - AXISRAM bank number (1-6)
 *
 ****************************************************************************/

static void stm32n6_axisram_enable_clocks(int bank)
{
  /* Enable RAMCFG clock */

  modifyreg32(STM32_RCC_AHB2ENR, 0, RCC_AHB2ENR_RAMCFG);

  /* Enable AXISRAM clock based on bank number */

  switch (bank)
    {
      case 3:
        modifyreg32(STM32_RCC_MEMENR, 0, RCC_MEMENR_AXISRAM3);
        break;
      case 4:
        modifyreg32(STM32_RCC_MEMENR, 0, RCC_MEMENR_AXISRAM4);
        break;
      case 5:
        modifyreg32(STM32_RCC_MEMENR, 0, RCC_MEMENR_AXISRAM5);
        break;
      case 6:
        modifyreg32(STM32_RCC_MEMENR, 0, RCC_MEMENR_AXISRAM6);
        break;
      default:
        break;
    }
}

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32n6_axisram_init
 *
 * Description:
 *   Initialize and enable an AXISRAM bank.
 *   Reference: Zephyr axisram_stm32_init() and HAL_RAMCFG_EnableAXISRAM()
 *
 * Input Parameters:
 *   bank - AXISRAM bank number (1-6)
 *
 * Returned Value:
 *   OK on success; Negated errno on failure.
 *
 ****************************************************************************/

int stm32n6_axisram_init(int bank)
{
  uintptr_t ramcfg_base;

  /* Get RAMCFG base address for the bank */

  switch (bank)
    {
      case 3:
        ramcfg_base = STM32_RAMCFG_SRAM3_BASE;
        break;
      case 4:
        ramcfg_base = STM32_RAMCFG_SRAM4_BASE;
        break;
      case 5:
        ramcfg_base = STM32_RAMCFG_SRAM5_BASE;
        break;
      case 6:
        ramcfg_base = STM32_RAMCFG_SRAM6_BASE;
        break;
      default:
        return -EINVAL;
    }

  /* Enable clocks */

  stm32n6_axisram_enable_clocks(bank);

  /* Enable AXISRAM via RAMCFG */

  modifyreg32(ramcfg_base + STM32_RAMCFG_CR_OFFSET, 0, RAMCFG_CR_AE);

  return OK;
}

#endif /* CONFIG_STM32N6_AXISRAM */
```

## MPU 区域配置

### `soc/st/stm32/stm32n6x/mpu_regions.c` (Zephyr)

```c
#include <zephyr/linker/linker-defs.h>
#include <zephyr/arch/arm/mpu/arm_mpu.h>

extern const uint32_t __rom_region_limit[];
extern const uint32_t __image_ram_limit[];

static const struct arm_mpu_region mpu_regions[] = {
	{
		.base = (uint32_t)__rom_region_start,
		.name = "SRAM_RO",
		.attr = {
			.rbar = RO_Msk | NON_SHAREABLE_Msk,
			.mair_idx = MPU_MAIR_INDEX_FLASH,
			.pxn = !PRIV_EXEC_NEVER,
			.r_limit = (uint32_t)__rom_region_limit,
		},
	},
	{
		.base = (uint32_t)_image_ram_start,
		.name = "SRAM_RW",
		.attr = {
			.rbar = NOT_EXEC | P_RW_U_NA_Msk | NON_SHAREABLE_Msk,
			.mair_idx = MPU_MAIR_INDEX_SRAM,
			.pxn = !PRIV_EXEC_NEVER,
			.r_limit = (uint32_t)__image_ram_limit,
		},
	}
};

const struct arm_mpu_config mpu_config = {
	.num_regions = ARRAY_SIZE(mpu_regions),
	.mpu_regions = mpu_regions,
};
```

### NuttX MPU 区域配置 (`arch/arm/src/stm32n6/stm32n6_mpuinit.c`)

```c
/****************************************************************************
 * arch/arm/src/stm32n6/stm32n6_mpuinit.c
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

#include <assert.h>

#include "arm_mpu.h"
#include "stm32n6_mpuinit.h"

#ifdef CONFIG_ARM_MPU

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Reference: Zephyr soc/st/stm32/stm32n6x/mpu_regions.c */

/* MPU Region Attributes */
#define MPU_RBAR_XN             (1 << 0)   /* Execute Never */
#define MPU_RBAR_AP_SHIFT       1          /* Access Permission shift */
#define MPU_RBAR_AP_RW          (1 << MPU_RBAR_AP_SHIFT)  /* Read/Write */
#define MPU_RBAR_AP_RO          (0 << MPU_RBAR_AP_SHIFT)  /* Read-only */
#define MPU_RBAR_SH_SHIFT       3          /* Shareable shift */
#define MPU_RBAR_SH_NONSHAREABLE (0 << MPU_RBAR_SH_SHIFT)
#define MPU_RBAR_SH_INNERSHARE  (3 << MPU_RBAR_SH_SHIFT)
#define MPU_RBAR_SH_OUTERSHARE  (2 << MPU_RBAR_SH_SHIFT)
#define MPU_RBAR_SH_INNEROUTER  (1 << MPU_RBAR_SH_SHIFT)

/* MPU Region Limit Register */
#define MPU_RLAR_LIMIT_MASK     0xffffffe0  /* 32-byte aligned limit */

/****************************************************************************
 * Public Functions
 ****************************************************************************/

/****************************************************************************
 * Name: stm32n6_mpu_init
 *
 * Description:
 *   Configure the MPU regions for STM32N6.
 *   Reference: Zephyr mpu_regions.c
 *
 ****************************************************************************/

void stm32n6_mpu_init(void)
{
  uintptr_t flash_base;
  uintptr_t flash_limit;
  uintptr_t ram_base;
  uintptr_t ram_limit;

  /* Get memory boundaries from linker script */

  /* Flash region (code/rodata) */
  flash_base  = (uintptr_t)_START_CODE;
  flash_limit = (uintptr_t)_END_CODE - 1;

  /* RAM region (data/bss) */
  ram_base   = (uintptr_t)_START_DATA;
  ram_limit  = (uintptr_t)_END_BSS - 1;

  /* Configure Flash region (Read-only, Execute) */

  arm_mpu_setregion(0, flash_base, flash_limit,
                    MPU_RBAR_AP_RO | MPU_RBAR_SH_NONSHAREABLE,
                    0);  /* MAIR index 0 = Flash */

  /* Configure RAM region (Read/Write, No Execute) */

  arm_mpu_setregion(1, ram_base, ram_limit,
                    MPU_RBAR_AP_RW | MPU_RBAR_XN | MPU_RBAR_SH_NONSHAREABLE,
                    1);  /* MAIR index 1 = SRAM */

  /* Enable MPU */

  arm_mpu_enable(true);
}

#endif /* CONFIG_ARM_MPU */
```

## STM32N6570 Discovery Kit 板级配置

### `boards/st/stm32n6570_dk/stm32n6570_dk_common.dtsi` (Zephyr)

关键配置摘要：

```dts
/ {
    chosen {
        zephyr,console = &usart1;
        zephyr,shell-uart = &usart1;
        zephyr,sram = &axisram2;
        zephyr,canbus = &fdcan1;
        zephyr,display = &ltdc;
        zephyr,touch = &gt911;
    };

    aliases {
        led0 = &green_led_1;
        led1 = &red_led_2;
        sw0 = &user_button;
        watchdog0 = &iwdg;
    };
};

/* Clock Configuration */
&clk_hse {
    hse-div2;
    clock-frequency = <DT_FREQ_M(48)>;
    status = "okay";
};

&clk_hsi {
    hsi-div = <1>;
    status = "okay";
};

&pll1 {
    clocks = <&clk_hse>;
    div-m = <3>;
    mul-n = <150>;
    div-p1 = <1>;
    div-p2 = <1>;
    status = "okay";
};

&cpusw {
    clocks = <&rcc STM32_SRC_IC1 CPU_SEL(3)>;
    clock-frequency = <DT_FREQ_M(800)>;  /* 800 MHz CPU */
    status = "okay";
};

&rcc {
    clocks = <&ic2>;
    clock-frequency = <DT_FREQ_M(400)>;  /* 400 MHz AHB */
    ahb-prescaler = <2>;
    timg-prescaler = <2>;
};

/* USART1 Console */
&usart1 {
    clocks = <&rcc STM32_CLOCK(APB2, 4)>,
             <&rcc STM32_SRC_CKPER USART1_SEL(1)>;
    pinctrl-0 = <&usart1_tx_pe5 &usart1_rx_pe6>;
    pinctrl-names = "default";
    current-speed = <115200>;
    status = "okay";
};

/* LTDC Display */
&ltdc {
    clocks = <&rcc STM32_CLOCK(APB5, 1)>,
             <&rcc STM32_SRC_IC16 LTDC_SEL(2)>;
    width = <800>;
    height = <480>;
    pixel-format = <PANEL_PIXEL_FORMAT_RGB_888>;
    status = "okay";
};

/* Ethernet */
&mac {
    status = "okay";
    phy-connection-type = "rgmii";
    phy-handle = <&eth_phy>;
};

/* NPU */
&npu {
    status = "okay";
};
```

### 板级引脚配置

| 功能 | 引脚 | 说明 |
|------|------|------|
| USART1 TX | PE5 | VCP 串口 |
| USART1 RX | PE6 | VCP 串口 |
| USART2 TX | PD5 | Arduino |
| USART2 RX | PF6 | Arduino |
| I2C2 SCL | PD14 | 触摸屏 |
| I2C2 SDA | PD4 | 触摸屏 |
| SPI5 NSS | PA3 | SPI |
| SPI5 SCK | PE15 | SPI |
| SPI5 MISO | PH8 | SPI |
| SPI5 MOSI | PG2 | SPI |
| FDCAN1 RX | PD0 | CAN FD |
| FDCAN1 TX | PH2 | CAN FD |
| SDMMC2 | PC0-5, PE4 | SD 卡 |
| ETH RGMII | PF0-15, PG3-4 | 以太网 |
| LTDC RGB | PA0-2,7-8,15, PB2,4,11-15, PD8-9,15, PE11, PG0,1,6,8,11-13,15, PH3-6 | LCD |
| LED1 (绿) | PO1 | 用户 LED |
| LED2 (红) | PG10 | 用户 LED |
| Button | PC13 | 用户按钮 |

## 移植计划

### 阶段 1：基础移植 ✅

- [x] SoC 配置 (Kconfig/soc.c)
- [x] 启动代码 (Reset_Handler, 向量表)
- [x] Cache 初始化 (I-Cache, D-Cache)
- [x] PWR 电源配置
- [x] RIF 资源隔离框架
- [ ] 时钟初始化 (RCC, PLL 配置)

### 阶段 2：核心外设

- [ ] GPIO 驱动
- [ ] UART 控制台驱动
- [ ] Timer 驱动
- [ ] 中断控制器 (EXTI)
- [ ] DMA 驱动

### 阶段 3：通信外设

- [ ] I2C 驱动
- [ ] SPI 驱动
- [ ] I3C 驱动
- [ ] FDCAN 驱动
- [ ] Ethernet 驱动

### 阶段 4：高级外设

- [ ] LTDC LCD 驱动
- [ ] DMA2D 2D 加速器
- [ ] DCMIPP 摄像头驱动
- [ ] JPEG 编解码器驱动
- [ ] VENC 视频编码器驱动
- [ ] NPU 驱动
- [ ] USB OTG HS 驱动
- [ ] SDMMC 驱动

### 阶段 5：安全特性

- [ ] TrustZone 安全/非安全分区
- [ ] SAU 配置
- [ ] RIF 完整配置
- [ ] 安全启动

## 参考资料

1. **Zephyr STM32N6 实现**: `soc/st/stm32/stm32n6x/`
2. **STM32N657 参考手册**: RM0486
3. **STM32N657 数据手册**: DS14206
4. **ARM Cortex-M55 技术参考手册**: DDI0553
5. **ARMv8.1-M 架构参考手册**: DDI0553
6. **NuttX STM32 现有驱动**: `arch/arm/src/stm32/`
7. **STM32Cube N6 HAL**: STM32Cube N6 包

## Zephyr 源码参考路径

```
zephyr/
├── soc/st/stm32/stm32n6x/
│   ├── Kconfig                      # SOC 系列配置
│   ├── Kconfig.soc                  # SOC 选择
│   ├── Kconfig.defconfig            # 默认配置
│   ├── soc.c                        # SoC 初始化 (Cache, PWR, RIF)
│   ├── soc.h                        # SoC 头文件
│   ├── mpu_regions.c                # MPU 区域配置
│   ├── mpu_regions.ld               # MPU 链接脚本
│   ├── ram_check.ld                 # RAM 检查
│   ├── CMakeLists.txt               # CMake 构建
│   └── npu/
│       ├── Kconfig                  # NPU 配置
│       ├── npu_cache_stm32n6.c      # NPU Cache 驱动
│       └── npu_stm32n6.c            # NPU 驱动
├── dts/arm/st/n6/
│   ├── stm32n6.dtsi                 # STM32N6 基础设备树
│   ├── stm32n657.dtsi               # STM32N657 设备树
│   ├── stm32n657X0.dtsi             # 封装变体
│   └── stm32n657X0_ns.dtsi          # 非安全模式
├── boards/st/stm32n6570_dk/
│   ├── stm32n6570_dk.dts            # Discovery Kit 设备树
│   ├── stm32n6570_dk_common.dtsi    # 公共定义 (时钟, 外设)
│   ├── stm32n6570_dk_defconfig      # 默认配置
│   ├── Kconfig.defconfig            # 板级 Kconfig
│   ├── board.cmake                  # 构建配置
│   └── arduino_r3_connector.dtsi    # Arduino 连接器
├── drivers/misc/stm32n6_axisram/
│   ├── stm32n6_axisram.c            # AXISRAM 驱动
│   └── Kconfig                      # AXISRAM 配置
├── include/zephyr/dt-bindings/
│   ├── clock/stm32n6_clock.h        # 时钟绑定定义
│   ├── reset/stm32n6_reset.h        # 复位绑定定义
│   └── power/stm32n6_iocell.h       # IO Cell 电源配置
└── tests/drivers/clock_control/stm32_clock_configuration/
    ├── stm32n6_core/                # STM32N6 内核时钟测试
    └── stm32n6_devices/             # STM32N6 外设时钟测试
```

## Changelog

| Date | Description |
|------|-------------|
| 2026-04-24 | Initial document based on Zephyr STM32N6 implementation |
| 2026-04-24 | Added SoC initialization code from Zephyr soc.c |
| 2026-04-24 | Added memory map from Zephyr stm32n6.dtsi |
| 2026-04-24 | Added clock configuration from Zephyr stm32n6_clock.h |
| 2026-04-24 | Added NPU driver from Zephyr npu_stm32n6.c |
| 2026-04-24 | Added AXISRAM driver from Zephyr stm32n6_axisram.c |
| 2026-04-24 | Added MPU regions from Zephyr mpu_regions.c |
| 2026-04-24 | Added STM32N6570 Discovery Kit board configuration |
| 2026-04-24 | Added NuttX equivalent implementations |
| 2026-04-24 | Updated all NuttX code to follow NuttX coding style (Apache 2.0 license header, function comments, naming conventions) |
