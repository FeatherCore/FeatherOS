# RA8P1 Zephyr 移植参考

本文档总结了从 Zephyr RTOS 移植到 NuttX RTOS 的 RA8P1 实现细节。

## 参考来源

### Zephyr RTOS 目录结构

```
third/zephyrproject/zephyr/
├── soc/renesas/ra/ra8p1/          # SOC 配置
│   ├── Kconfig.soc                # SOC 选择配置
│   ├── Kconfig                    # SOC 特性配置
│   ├── Kconfig.defconfig          # 默认配置
│   ├── CMakeLists.txt             # 构建配置
│   ├── power.c                    # 电源管理
│   └── sections.ld                # 链接脚本段定义
├── dts/arm/renesas/ra/ra8/        # 设备树定义
│   ├── ra8x1.dtsi                 # RA8X1 系列 (单核 CM85)
│   ├── ra8x2.dtsi                 # RA8X2 系列 (双核 CM85+CM33)
│   ├── r7ka8p1xf.dtsi             # RA8P1 基础定义
│   ├── r7ka8p1kflcac.dtsi         # RA8P1 具体型号
│   ├── r7ka8p1kflcac_cm85.dtsi    # CM85 核心配置
│   └── r7ka8p1kflcac_cm33.dtsi    # CM33 核心配置
├── boards/renesas/ek_ra8p1/       # EK-RA8P1 开发板
│   ├── ek_ra8p1_r7ka8p1kflcac_cm85.dts
│   ├── ek_ra8p1-pinctrl.dtsi      # 引脚配置
│   └── ek_ra8p1.dtsi              # 开发板定义
└── drivers/pinctrl/renesas/ra/    # Pinmux 驱动
    └── pinctrl_ra.c
```

## 关键映射

### 1. 设备树到内存映射

Zephyr 设备树定义转换为 NuttX 内存映射：

| Zephyr DTS 节点 | NuttX 定义 | 基地址 |
|-----------------|-----------|--------|
| `ioport0` | `RA8P_GPIO0_BASE` | 0x40400000 |
| `sci0` | `RA8P_SCI0_BASE` | 0x40358000 |
| `pwm0` | `RA8P_GPT0_BASE` | 0x40322000 |
| `iic0` | `RA8P_IIC0_BASE` | 0x4025E000 |
| `npu0` | `RA8P_NPU_BASE` | 0x40140000 |

### 2. 时钟配置映射

Zephyr 时钟定义 (`r7ka8p1xf.dtsi`)：

```dts
pll: pll {
    clocks = <&xtal>;    /* 24 MHz XTAL */
    div = <3>;
    mul = <250 0>;       /* VCO = 24MHz * 250 / 3 = 2000MHz */
    pllp: pllp { div = <2>; freq = <DT_FREQ_M(1000)>; };  /* 1000 MHz CPU */
    pllq: pllq { div = <6>; freq = <333333333>; };        /* 333 MHz */
    pllr: pllr { div = <5>; freq = <DT_FREQ_M(400)>; };   /* 400 MHz */
};
```

NuttX 实现 (`ra8p_cgc.c`)：

```c
#define RA8P_PLL_MUL                250
#define RA8P_PLL_DIV                3
#define RA8P_PLLP_FREQUENCY         1000000000 /* 1000 MHz */
```

### 3. Pinmux 映射

Zephyr pinctrl 定义 (`ek_ra8p1-pinctrl.dtsi`)：

```dts
sci8_default: sci8_default {
    group1 {
        psels = <RA_PSEL(RA_PSEL_SCI_8, 13, 2)>;  /* TX: P132 */
    };
    group2 {
        psels = <RA_PSEL(RA_PSEL_SCI_8, 13, 3)>;  /* RX: P133 */
    };
};
```

NuttX 实现 (`ra8p_pinctrl.c`)：

```c
void ra8p_pinctrl_configure(uint8_t port, uint8_t pin, uint8_t psel);
```

### 4. 外设中断映射

Zephyr 中断定义 (`ra8x1.dtsi`)：

```dts
sci0: sci0@40358000 {
    interrupts = <4 1>, <5 1>, <6 1>, <7 1>;
    interrupt-names = "rxi", "txi", "tei", "eri";
};
```

NuttX 中断定义 (`ra8p_irq.h`)：

```c
#define RA8P_IRQ_SCI0_TXI0     28
#define RA8P_IRQ_SCI0_RXI0     29
#define RA8P_IRQ_SCI0_TEI0     30
#define RA8P_IRQ_SCI0_ERI0     31
```

## 外设支持状态

### 已完成

| 外设 | Zephyr 驱动 | NuttX 驱动 | 状态 |
|------|------------|-----------|------|
| GPIO | `gpio_renesas_ra.c` | `ra8p_gpio.c` | ✅ |
| UART (SCI_B) | `uart_sci_b.c` | `ra8p_sci_b.c` | ✅ |
| Pinmux | `pinctrl_ra.c` | `ra8p_pinctrl.c` | ✅ |
| Clock | `clock_control_ra.c` | `ra8p_cgc.c` | ✅ |
| PWM (GPT) | `pwm_ra.c` | `ra8p_gpt.c` | ✅ |

### 待完善

| 外设 | Zephyr 驱动 | NuttX 状态 |
|------|------------|-----------|
| SPI | `spi_ra.c` | 🔄 头文件已定义 |
| I2C | `i2c_ra.c` | 🔄 头文件已定义 |
| USB FS | `usb_dc_ra.c` | ⏳ 待实现 |
| USB HS | `usb_dc_ra.c` | ⏳ 待实现 |
| CANFD | `can_ra.c` | ⏳ 待实现 |
| SDHC | `sdhc_ra.c` | ⏳ 待实现 |
| Ethernet | `eth_ra.c` | ⏳ 待实现 |
| RTC | `rtc_ra.c` | ⏳ 待实现 |
| ADC | `adc_ra.c` | ⏳ 待实现 |
| DAC | `dac_ra.c` | ⏳ 待实现 |
| I3C | `i3c_ra.c` | ⏳ 待实现 |
| GLCDC | `display_ra.c` | ⏳ 待实现 |
| MIPI DSI | `mipi_dsi_ra.c` | ⏳ 待实现 |
| NPU (Ethos-U55) | `ethos_u.c` | ⏳ 待实现 |

## 双核支持

RA8P1 支持双核配置 (Cortex-M85 + Cortex-M33)：

### Zephyr 配置

```dts
cpus {
    cpu0: cpu@0 {
        compatible = "arm,cortex-m85";
        reg = <0>;
    };
    cpu1: cpu@1 {
        compatible = "arm,cortex-m33";
        reg = <1>;
    };
};
```

### NuttX Kconfig

```kconfig
config RA8P_DUAL_CORE
    bool "Enable dual-core support"
    default y if ARCH_CHIP_R7KA8P1KFLCAC
```

### 内存分配

| 区域 | CM85 | CM33 |
|------|------|------|
| Code MRAM | 768 KB (0x02000000) | 256 KB (0x02C00000) |
| SRAM | 1404 KB (0x22000000) | 共享 |
| SRAM1 | 468 KB (0x2215F000) | 共享 |

## 电源管理

### Zephyr 实现 (`power.c`)

```c
void pm_state_set(enum pm_state state, uint8_t substate_id)
{
    switch (state) {
    case PM_STATE_RUNTIME_IDLE:
        R_LPM_Open(&pm_state_ctrl, &pm_state_runtime_idle_cfg);
        R_LPM_LowPowerModeEnter(&pm_state_ctrl);
        break;
    case PM_STATE_STANDBY:
        R_LPM_Open(&pm_state_ctrl, &pm_state_standby_cfg);
        R_LPM_LowPowerModeEnter(&pm_state_ctrl);
        break;
    }
}
```

### NuttX 实现 (`ra8p_power.c`)

```c
void ra8p_power_enter_sleep(void);
void ra8p_power_enter_standby(void);
void ra8p_power_enter_deep_standby(void);
```

## 开发板配置

### EK-RA8P1 引脚定义

| 功能 | Zephyr DTS | NuttX 定义 |
|------|-----------|-----------|
| LED1 | P600 | GPIO_LED1 |
| LED2 | P303 | GPIO_LED2 |
| LED3 | PA07 | GPIO_LED3 |
| SW1 | P009 | GPIO_SW1 |
| SW2 | P008 | GPIO_SW2 |
| UART8 TX | P713 | SCI8 TX |
| UART8 RX | P712 | SCI8 RX |

## 参考链接

1. [Zephyr RA8P1 SOC 实现](https://github.com/zephyrproject-rtos/zephyr/tree/main/soc/renesas/ra/ra8p1)
2. [Zephyr RA8P1 设备树](https://github.com/zephyrproject-rtos/zephyr/tree/main/dts/arm/renesas/ra/ra8)
3. [Zephyr EK-RA8P1 开发板](https://github.com/zephyrproject-rtos/zephyr/tree/main/boards/renesas/ek_ra8p1)
4. [Renesas FSP 文档](https://renesas.github.io/fsp/)