# Renesas RA8P1 移植记录

> 基于 Zephyr RTOS 实现，为 FeatherOS/NuttX 添加 RA8P1 支持

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
| `dts/arm/renesas/ra/ra8/` | 设备树定义 |
| `boards/renesas/ek_ra8p1/` | EK-RA8P1 开发板 |

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
