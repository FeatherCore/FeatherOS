STM32N6 Support for FeatherOS
==============================

This document describes the STM32N6 microcontroller support implementation for FeatherOS/NuttX.

Overview
--------

The STM32N6 series is a high-performance microcontroller family from STMicroelectronics featuring:
- ARM Cortex-M55 core with DSP and MVE (Matrix Vector Extensions)
- Neural Processing Unit (NPU) for AI inference
- Advanced security features (TrustZone, RIF)
- Rich connectivity options (Ethernet, USB, CAN-FD)
- Graphics capabilities (LTDC, DCMIPP, JPEG, VENC)
- Large memory options with multiple AXI SRAM banks

Hardware Features
-----------------

### Core Features
- ARM Cortex-M55 with ARMv8.1-M architecture
- Floating Point Unit (FPU)
- Digital Signal Processing (DSP) extensions
- Matrix Vector Extensions (MVE) for AI workloads
- Memory Protection Unit (MPU)
- Instruction and Data caches

### Memory Configuration
- Internal Flash: Up to 16MB (STM32N657xx)
- Internal SRAM: Up to 1MB
- AXI SRAM banks (AXISRAM1-6): Up to 2.75MB total
- External memory support via XSPI

### Connectivity
- Dual USB OTG High Speed interfaces
- Ethernet MAC with IEEE 1588
- Multiple CAN-FD interfaces (FDCAN1-3)
- Octo-SPI for external memory
- SDMMC interfaces

### Graphics and Imaging
- LTDC (LCD-TFT Display Controller)
- DCMIPP (Digital Camera Interface)
- JPEG hardware codec
- Video Encoder (VENC)
- 2D Graphics Accelerator (DMA2D)

### AI and Security
- Neural-ART NPU with dedicated cache
- Resource Isolation Framework (RIF)
- TrustZone security extension

Configuration Options
---------------------

### Core Configuration
CONFIG_ARCH_CHIP_STM32N657XX
    Select to enable STM32N657xx support with Cortex-M55 core

CONFIG_STM32N6_SYSCLK_FREQUENCY
    System clock frequency (default: 64000000 for HSI)

CONFIG_STM32N6_HSI_FREQUENCY
    HSI oscillator frequency (default: 64000000)

CONFIG_STM32N6_HSE_FREQUENCY
    HSE oscillator frequency (default: 48000000)

### Peripherals Configuration
CONFIG_STM32N6_NPU
    Enable Neural Processing Unit support

CONFIG_STM32N6_RIF
    Enable Resource Isolation Framework

CONFIG_STM32N6_LTDC
    Enable LCD-TFT Display Controller

CONFIG_STM32N6_DCMIPP
    Enable Digital Camera Interface

CONFIG_STM32N6_ETH
    Enable Ethernet controller

CONFIG_STM32N6_USB
    Enable USB OTG HS controllers

CONFIG_STM32N6_XSPI
    Enable Octo-SPI interface

CONFIG_STM32N6_JPEG
    Enable JPEG codec

CONFIG_STM32N6_VENC
    Enable Video Encoder

CONFIG_STM32N6_SDMMC
    Enable SDMMC controllers

CONFIG_STM32N6_DMA2D
    Enable 2D Graphics Accelerator

### Memory Configuration
CONFIG_STM32N6_AXISRAM1/2/3/4/5/6
    Enable individual AXI SRAM banks

### PLL Configuration
Multiple PLL configuration options (PLL1-4) with source selection and frequency settings.

Build and Usage
---------------

To build FeatherOS with STM32N6 support:

1. Configure for STM32N6 target:
   ```
   cd /path/to/featheros
   ./tools/configure.sh stm32n6-[board-name]:[configuration]
   ```

2. Build the system:
   ```
   make
   ```

The STM32N6 support follows the standard NuttX/FeatherOS architecture with:
- Hardware abstraction layer in /arch/arm/src/stm32n6/
- Driver implementations following STM32 family patterns
- Device-specific initialization sequences
- Memory map and register definitions

Reference
---------

The implementation is based on:
- STMicroelectronics STM32N6 reference manuals
- Zephyr RTOS STM32N6 implementation (for comparison)
- Existing NuttX STM32H7 and other STM32 family implementations