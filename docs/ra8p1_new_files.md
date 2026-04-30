# RA8P1 新增文件列表

本文档列出了基于 Zephyr RTOS 实现新增的所有 RA8P1 相关文件。

## 目录结构

```
FeatherOS/nuttx/arch/arm/src/ra8p/
├── hardware/
│   ├── ra8p_memorymap.h        # 内存映射定义
│   ├── ra8p_irq.h               # 中断定义
│   ├── ra8p_cgc.h               # 时钟生成控制
│   ├── ra8p_pinctrl.h           # 端口功能选择
│   ├── ra8p_sci_b.h             # SCI_B UART
│   ├── ra8p_spi_b.h             # SPI_B
│   ├── ra8p_gpt.h               # GPT PWM
│   ├── ra8p_iic.h               # IIC I2C
│   ├── ra8p_icu.h               # ICU 外部中断
│   ├── ra8p_rtc.h               # RTC 实时时钟
│   ├── ra8p_usb.h               # USB FS/HS
│   ├── ra8p_wdt.h               # WDT 看门狗
│   ├── ra8p_canfd.h             # CANFD
│   ├── ra8p_sdhc.h              # SDHC
│   ├── ra8p_power.h             # 电源管理
│   └── ra8p_dmac.h              # DMA
├── ra8p_start.c                 # 启动代码
├── ra8p_clockconfig.c           # 时钟配置
├── ra8p_clockconfig.h
├── ra8p_lowsetup.c              # 底层初始化
├── ra8p_lowsetup.h
├── ra8p_gpio.c                  # GPIO 驱动
├── ra8p_gpio.h
├── ra8p_cgc.c                   # CGC 时钟驱动
├── ra8p_pinctrl.c               # Pinmux 驱动
├── ra8p_sci_b.c                # UART 驱动
├── ra8p_spi_b.c                # SPI 驱动
├── ra8p_gpt.c                   # PWM 驱动
├── ra8p_icu.c                   # ICU 中断驱动
├── ra8p_power.c                 # 电源管理
├── ra8p_dmac.c                 # DMA 驱动
├── ra8p_iic.c                   # IIC I2C 驱动
├── ra8p_peripherals.h           # 外设整合头文件
├── chip.h                       # 芯片定义
├── CMakeLists.txt
├── Make.defs
└── Kconfig

boards/arm/ra8p/ek-ra8p1/
├── include/board.h
├── src/
│   ├── ek-ra8p1_boot.c
│   ├── ek-ra8p1_leds.c
│   ├── ek-ra8p1_buttons.c
│   └── ek-ra8p1_bringup.c
├── scripts/ek-ra8p1.ld
└── configs/nsh/defconfig

include/arch/ra8p/
└── irq.h                        # IRQ 入口头文件

docs/
├── ra8p1_porting.md             # 移植指南
├── ra8p1_zephyr_reference.md    # Zephyr 参考
└── ra8p1_implementation_status.md # 实现状态
```

## 新增文件统计

| 类别 | 数量 |
|------|------|
| HAL 头文件 | 16 |
| 驱动实现 (.c) | 14 |
| 驱动头文件 (.h) | 1 |
| 板级支持 | 9 |
| 文档 | 5 |
| **总计** | **45** |

## 参考的 Zephyr 文件

### SOC 配置
- `soc/renesas/ra/ra8p1/Kconfig.soc`
- `soc/renesas/ra/ra8p1/Kconfig`
- `soc/renesas/ra/ra8p1/Kconfig.defconfig`
- `soc/renesas/ra/ra8p1/CMakeLists.txt`
- `soc/renesas/ra/ra8p1/power.c`

### 设备树
- `dts/arm/renesas/ra/ra8/ra8x1.dtsi`
- `dts/arm/renesas/ra/ra8/ra8x2.dtsi`
- `dts/arm/renesas/ra/ra8/r7ka8p1xf.dtsi`
- `dts/arm/renesas/ra/ra8/r7ka8p1kflcac.dtsi`

### 开发板
- `boards/renesas/ek_ra8p1/ek_ra8p1.dtsi`
- `boards/renesas/ek_ra8p1/ek_ra8p1-pinctrl.dtsi`
- `boards/renesas/ek_ra8p1/ek_ra8p1_r7ka8p1kflcac_cm85.dts`

### 驱动
- `drivers/gpio/gpio_renesas_ra_ioport.c`
- `drivers/serial/uart_renesas_ra8_sci_b.c`
- `drivers/spi/spi_b_renesas_ra8.c`
- `drivers/i2c/i2c_renesas_ra_iic.c`
- `drivers/pwm/pwm_renesas_ra.c`
- `drivers/clock_control/clock_control_renesas_ra_cgc.c`
- `drivers/pinctrl/renesas/ra/pinctrl_ra.c`
- `drivers/misc/renesas_ra_external_interrupt/renesas_ra_external_interrupt.c`
- `drivers/dma/dma_renesas_ra.c`
- `drivers/rtc/rtc_renesas_ra.c`
- `drivers/watchdog/wdt_renesas_ra.c`
- `drivers/can/can_renesas_ra.c`
- `drivers/sdhc/sdhc_renesas_ra.c`

## 构建说明

### 编译 RA8P1

```bash
cd nuttx

# 配置 NuttX
make menuconfig

# 选择:
#   System Type ->
#     ARM Cortex-M processor(s) ->
#       ARM or Thumb-2
#     Platform Selection ->
#       Renesas RA8P
#     RA8P Chip Selection ->
#       R7KA8P1KFLCAC (RA8P1)

# 构建
make
```

### 编译单个驱动

```bash
# 编译 CGC 时钟驱动
arm-none-eabi-gcc -c ra8p_cgc.c

# 编译 GPIO 驱动
arm-none-eabi-gcc -c ra8p_gpio.c
```