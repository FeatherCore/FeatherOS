# ARM Cortex-M55 and Cortex-M85 Support

> Based on Zephyr RTOS implementation, updated for FeatherOS/NuttX

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
| I-Cache | No | 32-byte lines | 32-byte lines |
| D-Cache | No | 32-byte lines | 32-byte lines |
| TrustZone | Yes | Yes | Yes |
| PMU | No | Yes (8 counters) | Yes (8 counters) |

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

### MPS Board Selection (`arch/arm/src/mps/Kconfig`)

MPS (Corstone) board configurations for Cortex-M55 and M85:

```kconfig
# Cortex-M55 variants
config ARCH_CHIP_MPS3_AN547
    bool "MPS3 AN547 Processor Cortexm55"
    select ARCH_CORTEXM55
    select ARCH_HAVE_FPU
    select ARM_HAVE_MVE
    select ARMV8M_HAVE_ICACHE
    select ARMV8M_HAVE_DCACHE

# Cortex-M85 variants
config ARCH_CHIP_MPS3_AN547_M85
    bool "MPS3 AN547 Processor Cortexm85"
    select ARCH_CORTEXM85
    select ARCH_HAVE_MPU
    select ARM_HAVE_MPU_UNIFIED
    select ARCH_HAVE_FPU
    select ARM_HAVE_MVE
    select ARMV8M_HAVE_ICACHE
    select ARMV8M_HAVE_DCACHE
```

## Renesas RA8P1 Support

RA8P1 is based on Cortex-M85 and is now supported in NuttX.

### RA8P1 Features

| Feature | Value |
|---------|-------|
| CPU | Cortex-M85 |
| Max Frequency | 1 GHz |
| Architecture | ARMv8.1-M Mainline |
| DSP | Yes |
| MVE-F | Yes |
| FPU | FPv5-D16 |
| I/D Cache | 32-byte lines |
| MRAM | 768 KB |
| SRAM | 1 MB |

### RA8P1 Directory Structure

```
arch/arm/src/ra8p/
├── Kconfig                  # RA8P chip selection
├── CMakeLists.txt          # CMake build config
├── Make.defs               # Makefile definitions
├── chip.h                  # Chip definitions
├── ra8p_start.c            # Startup code
├── ra8p_clockconfig.c      # Clock configuration
├── ra8p_clockconfig.h      # Clock API
├── ra8p_lowsetup.c         # Low-level setup
├── ra8p_lowsetup.h         # Low-level API
└── hardware/
    ├── ra8p_memorymap.h    # Memory map
    └── ra8p_irq.h          # Interrupt definitions

include/arch/ra8p/
└── irq.h                   # IRQ header
```

### RA8P1 Kconfig

```kconfig
config ARCH_CHIP_RA8P
    bool "Renesas RA8P"
    select ARCH_HAVE_MPU
    select ARM_HAVE_MPU_UNIFIED
    select ARCH_HAVE_FPU
    select ARMV8M_HAVE_ICACHE
    select ARMV8M_HAVE_DCACHE

config ARCH_CHIP_R7KA8P1KFLCAC
    bool "R7KA8P1KFLCAC (RA8P1)"
    select ARCH_CORTEXM85
    select ARCH_HAVE_FPU
    select ARCH_HAVE_MPU
    select ARM_HAVE_MPU_UNIFIED
    select ARM_HAVE_MVE
    select ARMV8M_HAVE_ICACHE
    select ARMV8M_HAVE_DCACHE
    select ARMV8M_SYSTICK
```

## Toolchain Configuration

### GCC Flags

| CPU | Architecture | FPU | Extensions |
|-----|--------------|-----|-----------|
| M55 | `armv8.1-m.main` | `fpv5-d16` | `+dsp`, `+mve.fp+fp.dp` |
| M85 | `armv8.1-m.main` | `fpv5-d16` | `+dsp`, `+mve.fp+fp.dp` |

Example GCC flags for Cortex-M85:
```bash
-mtune=cortex-m85
-march=armv8.1-m.main+mve.fp+fp.dp+dsp
-mfpu=fpv5-d16
-mfloat-abi=hard
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

M55 and M85 include integrated I-cache and D-cache with 32-byte cache lines.

**Kconfig:**
- `CONFIG_ARMV8M_HAVE_ICACHE` - Hardware has I-cache
- `CONFIG_ARMV8M_HAVE_DCACHE` - Hardware has D-cache
- `CONFIG_ARMV8M_ICACHE` - Enable I-cache at runtime
- `CONFIG_ARMV8M_DCACHE` - Enable D-cache at runtime

### 4. Performance Monitoring Unit (PMU)

ARMv8.1-M includes an enhanced PMU with up to 8 event counters plus a cycle counter.

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

## Usage Example

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

## Changelog

| Date | Description |
|------|-------------|
| 2026-04-23 | Initial M55/M85 implementation based on Zephyr Corstone-300/310/315/320 support |
| 2026-04-23 | Added RA8P1 support (R7KA8P1KFLCAC) |
