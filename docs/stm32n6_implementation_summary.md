STM32N6 Support for FeatherOS/NuttX
===================================

This document summarizes the current STM32N6 and STM32N6570-DK support in
FeatherOS/NuttX.

## Current Status

The port is integrated into the NuttX ARM architecture and board framework:

- `CONFIG_ARCH_CHIP_STM32N6`
- `CONFIG_ARCH_CHIP_STM32N657XX`
- `CONFIG_ARCH_BOARD_STM32N6570_DK`
- `stm32n6570-dk:nsh`
- `stm32n6570-dk:full`

The `nsh` configuration is the conservative bring-up target. It builds a
Cortex-M55 image with USART1 console, USART2 serial, GPIO, SysTick, IRQ
handling, heap setup, and NSH.  It does not depend on external XSPI FLASH or
board-level MTD registration.

The `full` configuration opens the board-level compile and bring-up surface for
the official STM32N6570-DK peripherals described by Zephyr's
`stm32n6570_dk_common.dtsi`: I2C1/I2C2/I2C4, GT911 touch reset/interrupt,
SPI5, FDCAN1, SDMMC2, ADC1, TIM1/TIM15 PWM, Ethernet RGMII/MDIO, LTDC RGB,
display power/backlight, XSPI1 PSRAM, and XSPI2 NOR FLASH. Complex
lower-halves that are not hardware-validated are intentionally left as
deferred NuttX entry points returning `NULL` or `-ENOSYS`.

XSPI2 NOR FLASH is now wired through the board bring-up path in `full`.  The
driver probes the Macronix MX66UW1G45G by JEDEC ID, uses conservative
single-SPI indirect commands, and registers the MTD instance as
`/dev/xspi2flash0` when initialization succeeds.

## Implemented

- ARM Kconfig integration for STM32N6 and STM32N657xx.
- Board Kconfig integration for `stm32n6570-dk`.
- Standard board layout:
  `nuttx/boards/arm/stm32n6/stm32n6570-dk/`.
- Standard config layout:
  `configs/nsh/defconfig` and `configs/full/defconfig`.
- Board `scripts/Make.defs` using ARMv8-M toolchain rules.
- Board and SoC CMake/Make build entries.
- Cortex-M55 startup path, IRQ enable/disable, SysTick timer, heap setup, and
  `stm32_boardinitialize()`.
- `arch/arm/include/stm32n6/irq.h` and chip include support.
- GPIO pinset encoding for GPIOA-H and GPION-Q.
- STM32N6570-DK board pins for USART1 console, USART2, LEDs, USER button,
  I2C1/I2C2/I2C4, GT911, SPI5, FDCAN1, SDMMC2, ADC1, TIM1/TIM15 PWM, Ethernet,
  LTDC/display, XSPI1 PSRAM, and XSPI2 NOR FLASH.
- Serial driver updated for the current NuttX `uart_ops_s` interface.
- Board application initialization split into standard
  `stm32_boardinitialize()`, `board_app_initialize()`, and `stm32_bringup()`
  phases.
- Basic compile coverage for SPI, I2C, PWM/TIM, EXTI, FDCAN, NPU, RIF, USB,
  ADC, LTDC, ETH, SDMMC, and XSPI configuration paths.
- NuttX-shaped lower-half entry points for I2C, SPI, PWM, ADC, FDCAN, SDMMC,
  ETH, LTDC, and XSPI so `full` covers the expected driver integration points.
- STM32N6 XSPI `qspi_dev_s` controller entry with locking, frequency setup,
  indirect command, and indirect memory read/write support.
- Minimal MX66UW1G45G MTD driver with JEDEC probe, 4-byte read, page program,
  4KB erase, geometry, and `/dev/xspi2flash0` registration from board app
  initialization.
- XSPI1 PSRAM pin, clock, reset, and controller-selection compile path. PSRAM
  is not used as heap, `.data`, or `.bss` storage.

## Stubbed or Incomplete

These areas are not claimed as runtime-complete:

- Ethernet MAC lower-half: `stm32n6_eth_initialize()` returns `-ENOSYS`.
- I2C/SPI/PWM/ADC/FDCAN/SDMMC lower-halves expose compile-time NuttX entry
  points but return `NULL`, `-ENOSYS`, or no-op until real implementations are
  completed.
- LTDC lower-half: LTDC public entry points return `-ENOSYS`.
- XSPI1 PSRAM, Octal DTR, memory-mapped mode, calibration/training, and
  automatic filesystem mounting are not implemented.
- Most non-console alternate-function numbers are provisional. The local
  Zephyr checkout contains the STM32N6570-DK board wiring but not the included
  ST `stm32n657x0hxq-pinctrl.dtsi` AF source, so these pins need validation
  against ST pinctrl/reference material before claiming hardware operation.
- Hardware validation on STM32N6570-DK has not been performed in this pass.

## Build

```bash
cd /home/uan-gpd/third/FeatherOS/nuttx
./tools/configure.sh -L stm32n6570

make distclean
./tools/configure.sh stm32n6570-dk:nsh
make olddefconfig
make -j$(nproc)

make distclean
./tools/configure.sh stm32n6570-dk:full
make olddefconfig
make -j$(nproc)
```

## References

- STMicroelectronics STM32N6 reference material.
- Zephyr STM32N6570-DK board description.
- Existing NuttX ARMv8-M and STM32 family ports.
