/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_iic.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_IIC_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_IIC_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* IIC Register Offsets - Based on RA8P Hardware Manual and Zephyr definitions */
#define RA8P_IIC_ICCR1_OFFSET            0x0000  /* Control Register 1 */
#define RA8P_IIC_ICCR2_OFFSET            0x0001  /* Control Register 2 */
#define RA8P_IIC_ICMR1_OFFSET            0x0002  /* Mode Register 1 */
#define RA8P_IIC_ICMR2_OFFSET            0x0003  /* Mode Register 2 */
#define RA8P_IIC_ICMR3_OFFSET            0x0004  /* Mode Register 3 */
#define RA8P_IIC_ICFER_OFFSET            0x0005  /* Function Enable Register */
#define RA8P_IIC_ICSER_OFFSET            0x0006  /* Status Enable Register */
#define RA8P_IIC_ICIER_OFFSET            0x0007  /* Interrupt Enable Register */
#define RA8P_IIC_ICSR1_OFFSET            0x0008  /* Status Register 1 */
#define RA8P_IIC_ICSR2_OFFSET            0x0009  /* Status Register 2 */
#define RA8P_IIC_ICSARL_OFFSET           0x000A  /* Slave Address Low Register */
#define RA8P_IIC_ICSARU_OFFSET           0x000B  /* Slave Address High Register */
#define RA8P_IIC_ICDRT_OFFSET            0x000C  /* Data Register Transmit */
#define RA8P_IIC_ICDRR_OFFSET            0x000D  /* Data Register Receive */
#define RA8P_IIC_ICBRL_OFFSET            0x000E  /* Bit Rate Low Register */
#define RA8P_IIC_ICBRH_OFFSET            0x000F  /* Bit Rate High Register */
#define RA8P_IIC_SAR0_OFFSET             0x0010  /* Slave Address Register 0 */
#define RA8P_IIC_SAR1_OFFSET             0x0011  /* Slave Address Register 1 */
#define RA8P_IIC_SAR2_OFFSET             0x0012  /* Slave Address Register 2 */
#define RA8P_IIC_SMR_OFFSET              0x0013  /* Slave Mode Register */
#define RA8P_IIC_SNCR_OFFSET             0x0014  /* Stop/Noise Control Register */
#define RA8P_IIC_SIMR_OFFSET             0x0015  /* SDA/Interrupt Control Register */
#define RA8P_IIC_SISR_OFFSET             0x0016  /* SDA/Interrupt Status Register */
#define RA8P_IIC_SCMR_OFFSET             0x0017  /* SCL Control Register */
#define RA8P_IIC_TMOER_OFFSET            0x0018  /* Timeout Error Register */
#define RA8P_IIC_TMORR_OFFSET            0x0019  /* Timeout Control Register */
#define RA8P_IIC_NMCR_OFFSET             0x001A  /* Noise Filter Control Register */
#define RA8P_IIC_NMR_OFFSET              0x001B  /* Noise Filter Control Register */
#define RA8P_IIC_SOER_OFFSET             0x001C  /* SDA Output Control Register */
#define RA8P_IIC_SOAR_OFFSET             0x001D  /* SDA Output Control Register A */
#define RA8P_IIC_SOBR_OFFSET             0x001E  /* SDA Output Control Register B */
#define RA8P_IIC_SOCR_OFFSET             0x001F  /* SDA Output Control Register C */
#define RA8P_IIC_SSDRR_OFFSET            0x0020  /* SDA Delay Register R */
#define RA8P_IIC_SSDWR_OFFSET            0x0021  /* SDA Delay Register W */
#define RA8P_IIC_SCCR_OFFSET             0x0022  /* SCL Clock Control Register */
#define RA8P_IIC_SCFSR_OFFSET            0x0023  /* SCL Function Selection Register */
#define RA8P_IIC_SCMSR_OFFSET            0x0024  /* SCL Mode Selection Register */
#define RA8P_IIC_SPMR_OFFSET             0x0025  /* SDA/Interrupt Mode Register */
#define RA8P_IIC_SPBR_OFFSET             0x0026  /* SDA/Interrupt Bit Rate Register */
#define RA8P_IIC_SMDMR_OFFSET            0x0027  /* SDA/Interrupt Mode Register M */
#define RA8P_IIC_SBMR_OFFSET             0x0028  /* SDA/Interrupt Bit Rate Register M */
#define RA8P_IIC_SIER_OFFSET             0x0029  /* SDA/Interrupt Enable Register */
#define RA8P_IIC_SISR_OFFSET             0x002A  /* SDA/Interrupt Status Register */
#define RA8P_IIC_SIER2_OFFSET            0x002B  /* SDA/Interrupt Enable Register 2 */
#define RA8P_IIC_SISR2_OFFSET            0x002C  /* SDA/Interrupt Status Register 2 */
#define RA8P_IIC_ICBR_OFFSET             0x002D  /* Bit Rate Register */
#define RA8P_IIC_ICBMR_OFFSET            0x002E  /* Bit Rate Mode Register */
#define RA8P_IIC_ICBCR_OFFSET            0x002F  /* Bit Rate Control Register */
#define RA8P_IIC_ICBFCR_OFFSET           0x0030  /* Bit Rate Filter Control Register */
#define RA8P_IIC_ICBCCR_OFFSET           0x0031  /* Bit Rate Clock Control Register */
#define RA8P_IIC_ICBDSR_OFFSET           0x0032  /* Bit Rate Data Setup Register */
#define RA8P_IIC_ICBHCR_OFFSET           0x0033  /* Bit Rate Hold Control Register */
#define RA8P_IIC_ICBSCR_OFFSET           0x0034  /* Bit Rate Start Condition Register */
#define RA8P_IIC_ICBPCR_OFFSET           0x0035  /* Bit Rate Stop Condition Register */
#define RA8P_IIC_ICBER_OFFSET            0x0036  /* Bit Rate Error Register */
#define RA8P_IIC_ICBECR_OFFSET           0x0037  /* Bit Rate Error Control Register */
#define RA8P_IIC_ICBFCR2_OFFSET          0x0038  /* Bit Rate Filter Control Register 2 */
#define RA8P_IIC_ICBCCR2_OFFSET          0x0039  /* Bit Rate Clock Control Register 2 */
#define RA8P_IIC_ICBDSR2_OFFSET          0x003A  /* Bit Rate Data Setup Register 2 */
#define RA8P_IIC_ICBHCR2_OFFSET          0x003B  /* Bit Rate Hold Control Register 2 */
#define RA8P_IIC_ICBSCR2_OFFSET          0x003C  /* Bit Rate Start Condition Register 2 */
#define RA8P_IIC_ICBPCR2_OFFSET          0x003D  /* Bit Rate Stop Condition Register 2 */
#define RA8P_IIC_ICBER2_OFFSET           0x003E  /* Bit Rate Error Register 2 */
#define RA8P_IIC_ICBECR2_OFFSET          0x003F  /* Bit Rate Error Control Register 2 */

/* ICCR1 - Control Register 1 */
#define RA8P_IIC_ICCR1_ICE               (1 << 7)   /* IIC Enable */
#define RA8P_IIC_ICCR1_IICRST            (1 << 6)   /* IIC Reset */
#define RA8P_IIC_ICCR1_SOWP              (1 << 4)   /* Software Output Wait */
#define RA8P_IIC_ICCR1_CLO               (1 << 3)   /* Clock Low Output */
#define RA8P_IIC_ICCR1_SCLI              (1 << 2)   /* SCL Input */
#define RA8P_IIC_ICCR1_SDAI              (1 << 1)   /* SDA Input */
#define RA8P_IIC_ICCR1_SDAO              (1 << 0)   /* SDA Output */

/* ICCR2 - Control Register 2 */
#define RA8P_IIC_ICCR2_BBSY              (1 << 7)   /* Bus Busy */
#define RA8P_IIC_ICCR2_MST               (1 << 6)   /* Master/Slave Select */
#define RA8P_IIC_ICCR2_TRS               (1 << 5)   /* Transmit/Receive Select */
#define RA8P_IIC_ICCR2_SP                (1 << 3)   /* Stop Condition */
#define RA8P_IIC_ICCR2_RS                (1 << 2)   /* Repeated Start Condition */
#define RA8P_IIC_ICCR2_ST                (1 << 1)   /* Start Condition */
#define RA8P_IIC_ICCR2_TDRE              (1 << 0)   /* Transmit Data Empty */

/* ICMR1 - Mode Register 1 */
#define RA8P_IIC_ICMR1_MTM               (1 << 7)   /* Master Transfer Wait */
#define RA8P_IIC_ICMR1_CKS_MASK          (0x07 << 4) /* Clock Select Mask */
#define RA8P_IIC_ICMR1_CKS_SHIFT         4
#define RA8P_IIC_ICMR1_BC_MASK           (0x0F << 0) /* Bit Counter Mask */
#define RA8P_IIC_ICMR1_BC_SHIFT          0
#define RA8P_IIC_ICMR1_BC_9BIT           (0 << 0)   /* 9 bits (8 data + 1 ACK/NACK) */
#define RA8P_IIC_ICMR1_BC_8BIT           (7 << 0)   /* 8 bits */
#define RA8P_IIC_ICMR1_BC_1BIT           (8 << 0)   /* 1 bit */
#define RA8P_IIC_ICMR1_BC_2BIT           (9 << 0)   /* 2 bits */

/* ICMR2 - Mode Register 2 */
#define RA8P_IIC_ICMR2_DLCS              (1 << 6)   /* Digital Filter Clock Select */
#define RA8P_IIC_ICMR2_TMOS              (1 << 5)   /* Timeout Monitor Select */
#define RA8P_IIC_ICMR2_MSS               (1 << 4)   /* Master/Slave Select */
#define RA8P_IIC_ICMR2_NF_MASK           (0x03 << 2) /* Noise Filter Mask */
#define RA8P_IIC_ICMR2_NF_SHIFT          2
#define RA8P_IIC_ICMR2_NF_DISABLE        (0 << 2)   /* Disable */
#define RA8P_IIC_ICMR2_NF_1CYCLE         (1 << 2)   /* 1 cycle */
#define RA8P_IIC_ICMR2_NF_2CYCLE         (2 << 2)   /* 2 cycles */
#define RA8P_IIC_ICMR2_NF_3CYCLE         (3 << 2)   /* 3 cycles */
#define RA8P_IIC_ICMR2_TMOL              (1 << 1)   /* Timeout L */
#define RA8P_IIC_ICMR2_TMOH              (1 << 0)   /* Timeout H */

/* ICMR3 - Mode Register 3 */
#define RA8P_IIC_ICMR3_ACKWP             (1 << 4)   /* ACK Wait */
#define RA8P_IIC_ICMR3_RDRFS             (1 << 3)   /* RDRF Select */
#define RA8P_IIC_ICMR3_ACKBT             (1 << 2)   /* ACK Bit */
#define RA8P_IIC_ICMR3_ACKBR             (1 << 1)   /* ACK Bit Receive */
#define RA8P_IIC_ICMR3_WAIT              (1 << 0)   /* Wait */

/* ICFER - Function Enable Register */
#define RA8P_IIC_ICFER_TMOE              (1 << 7)   /* Timeout Enable */
#define RA8P_IIC_ICFER_MALE              (1 << 6)   /* Master Arbitration Lost Enable */
#define RA8P_IIC_ICFER_NACKE             (1 << 5)   /* NACK Enable */
#define RA8P_IIC_ICFER_SALE              (1 << 4)   /* Slave Arbitration Lost Enable */
#define RA8P_IIC_ICFER_NFE               (1 << 3)   /* Noise Filter Enable */
#define RA8P_IIC_ICFER_SCLE              (1 << 2)   /* SCL Synchronous Enable */
#define RA8P_IIC_ICFER_FMPE              (1 << 1)   /* Fast Mode Plus Enable */
#define RA8P_IIC_ICFER_DTE               (1 << 0)   /* Digital Filter Enable */

/* ICSER - Status Enable Register */
#define RA8P_IIC_ICSER_HOE               (1 << 7)   /* Host Enable */
#define RA8P_IIC_ICSER_DIE               (1 << 6)   /* Device ID Enable */
#define RA8P_IIC_ICSER_GCAE              (1 << 5)   /* General Call Address Enable */
#define RA8P_IIC_ICSER_SARE              (1 << 0)   /* Slave Address Enable */

/* ICIER - Interrupt Enable Register */
#define RA8P_IIC_ICIER_TMOIE             (1 << 7)   /* Timeout Interrupt Enable */
#define RA8P_IIC_ICIER_ALIE              (1 << 6)   /* Arbitration Lost Interrupt Enable */
#define RA8P_IIC_ICIER_NAKIE             (1 << 5)   /* NACK Interrupt Enable */
#define RA8P_IIC_ICIER_STIE              (1 << 4)   /* Start Interrupt Enable */
#define RA8P_IIC_ICIER_SPIE              (1 << 3)   /* Stop Interrupt Enable */
#define RA8P_IIC_ICIER_RIE               (1 << 2)   /* Receive Interrupt Enable */
#define RA8P_IIC_ICIER_TEIE              (1 << 1)   /* Transmit End Interrupt Enable */
#define RA8P_IIC_ICIER_TIE               (1 << 0)   /* Transmit Interrupt Enable */

/* ICSR1 - Status Register 1 */
#define RA8P_IIC_ICSR1_HOA               (1 << 7)   /* Host Address */
#define RA8P_IIC_ICSR1_DID               (1 << 6)   /* Device ID */
#define RA8P_IIC_ICSR1_GCA               (1 << 5)   /* General Call Address */
#define RA8P_IIC_ICSR1_AAS0              (1 << 4)   /* Address As Slave 0 */
#define RA8P_IIC_ICSR1_AAS1              (1 << 3)   /* Address As Slave 1 */
#define RA8P_IIC_ICSR1_AAS2              (1 << 2)   /* Address As Slave 2 */
#define RA8P_IIC_ICSR1_REC               (1 << 1)   /* Receive */
#define RA8P_IIC_ICSR1_TRS               (1 << 0)   /* Transmit */

/* ICSR2 - Status Register 2 */
#define RA8P_IIC_ICSR2_TMOF              (1 << 7)   /* Timeout Flag */
#define RA8P_IIC_ICSR2_AL                (1 << 6)   /* Arbitration Lost */
#define RA8P_IIC_ICSR2_NACKF             (1 << 5)   /* NACK Flag */
#define RA8P_IIC_ICSR2_START             (1 << 4)   /* Start */
#define RA8P_IIC_ICSR2_STOP              (1 << 3)   /* Stop */
#define RA8P_IIC_ICSR2_RDRF              (1 << 2)   /* Receive Data Full */
#define RA8P_IIC_ICSR2_TEND              (1 << 1)   /* Transmit End */
#define RA8P_IIC_ICSR2_TDRE              (1 << 0)   /* Transmit Data Empty */

/* I2C Base Addresses */
#define RA8P_IIC0_BASE                   0x4025E000
#define RA8P_IIC1_BASE                   0x4025E100
#define RA8P_IIC2_BASE                   0x4025E200

/* Maximum number of I2C channels */
#define RA8P_IIC_MAX_CHANNELS            3

/* I2C interrupt numbers */
#define RA8P_IRQ_IIC0                    68
#define RA8P_IRQ_IIC1                    69
#define RA8P_IRQ_IIC2                    70

/* Default I2C clock frequencies */
#define RA8P_IIC_STANDARD_FREQ           100000  /* 100 kHz */
#define RA8P_IIC_FAST_FREQ               400000  /* 400 kHz */
#define RA8P_IIC_FAST_PLUS_FREQ          1000000 /* 1 MHz */
#define RA8P_IIC_ULTRA_FAST_FREQ         3400000 /* 3.4 MHz */

/* Clock divider settings for different speeds */
#define RA8P_IIC_CLK_DIVIDER_100KHZ      62  /* Assuming PCLKB=62.5MHz / (62*16) ≈ 100kHz */
#define RA8P_IIC_CLK_DIVIDER_400KHZ      15  /* Assuming PCLKB=62.5MHz / (15*16) ≈ 400kHz */
#define RA8P_IIC_CLK_DIVIDER_1MHZ        3   /* For faster speeds */

/* I2C address types */
#define RA8P_IIC_ADDR_7BIT               7
#define RA8P_IIC_ADDR_10BIT              10

/* I2C command types */
#define RA8P_IIC_CMD_START               0x01
#define RA8P_IIC_CMD_RESTART             0x02
#define RA8P_IIC_CMD_STOP                0x04
#define RA8P_IIC_CMD_WRITE               0x08
#define RA8P_IIC_CMD_READ                0x10

/* I2C timeout in milliseconds */
#define RA8P_IIC_TIMEOUT_MS              1000

/* Maximum I2C payload */
#define RA8P_IIC_MAX_PAYLOAD             255

/* Maximum number of retries */
#define RA8P_IIC_MAX_RETRIES             3

/* I2C error interrupt flags */
#define RA8P_IIC_ERROR_TIMEOUT           (1 << 0)
#define RA8P_IIC_ERROR_ARBITRATION_LOST (1 << 1)
#define RA8P_IIC_ERROR_NACK_RECEIVED    (1 << 2)
#define RA8P_IIC_ERROR_BUS_ERROR        (1 << 3)

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* I2C timing configuration structure */
struct ra8p_iic_timing_s
{
  uint8_t sda_hold_time;              /* SDA hold time in clock cycles */
  uint8_t sda_setup_time;             /* SDA setup time in clock cycles */
  uint8_t scl_low_time;               /* SCL low period in clock cycles */
  uint8_t scl_high_time;              /* SCL high period in clock cycles */
  uint8_t start_condition_hold_time;   /* Start condition hold time in clock cycles */
  uint8_t stop_condition_setup_time;   /* Stop condition setup time in clock cycles */
  uint8_t bus_free_time;              /* Bus free time in clock cycles */
  uint8_t bus_idle_time;              /* Bus idle time in clock cycles */
};

/* I2C transfer structure */
struct ra8p_iic_transfer_s
{
  uint8_t addr;                       /* 7-bit or 10-bit slave address */
  uint8_t *buffer;                    /* Data buffer */
  uint32_t length;                    /* Number of bytes to transfer */
  bool read;                          /* Read operation */
  bool ten_bit_addr;                  /* Use 10-bit addressing */
  bool restart;                       /* Send repeated start */
  uint32_t flags;                     /* Transfer flags */
};

/* I2C configuration structure */
struct ra8p_iic_config_s
{
  uint8_t channel;                    /* I2C channel (0-2) */
  uint32_t frequency;                 /* I2C clock frequency in Hz */
  uint8_t addr_mode;                  /* Addressing mode (7 or 10 bit) */
  bool master_mode;                   /* Master mode if true, slave if false */
  bool enable_dma;                    /* Enable DMA for transfers */
  bool enable_interrupts;             /* Enable interrupts */
  uint8_t scl_pullup;                 /* SCL pull-up (0=none, 1=internal, 2=external) */
  uint8_t sda_pullup;                 /* SDA pull-up (0=none, 1=internal, 2=external) */
  bool noise_filter;                  /* Enable noise filter */
  uint8_t noise_filter_stages;        /* Number of noise filter stages (1-4) */
  bool timeout_enable;                /* Enable timeout detection */
  struct ra8p_iic_timing_s timing;    /* Timing configuration */
  bool fast_mode_plus;                /* Enable fast mode plus */
  bool digital_filter;                /* Enable digital filter */
};

/* I2C slave callback function type */
typedef void (*ra8p_iic_handler_t)(uint8_t event, void *arg);

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_iic_initialize
 *
 * Description:
 *   Initialize the IIC controller based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   config - Pointer to I2C configuration structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_initialize(const struct ra8p_iic_config_s *config);

/****************************************************************************
 * Name: ra8p_iic_start
 *
 * Description:
 *   Start IIC controller operation based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_start(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_stop
 *
 * Description:
 *   Stop IIC controller operation based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_stop(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_transfer
 *
 * Description:
 *   Perform an I2C transfer based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   msgs - Array of I2C messages
 *   count - Number of messages
 *
 * Returned Value:
 *   Number of messages processed on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_transfer(uint8_t channel, struct i2c_msg_s *msgs, int count);

/****************************************************************************
 * Name: ra8p_iic_set_frequency
 *
 * Description:
 *   Set IIC clock frequency based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   frequency - Clock frequency in Hz
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_set_frequency(uint8_t channel, uint32_t frequency);

/****************************************************************************
 * Name: ra8p_iic_set_addr_mode
 *
 * Description:
 *   Set IIC addressing mode (7-bit or 10-bit) based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   mode - Addressing mode (7 or 10)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_set_addr_mode(uint8_t channel, uint8_t mode);

/****************************************************************************
 * Name: ra8p_iic_is_initialized
 *
 * Description:
 *   Check if IIC controller is initialized based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *
 * Returned Value:
 *   true if initialized, false otherwise
 *
 ****************************************************************************/

bool ra8p_iic_is_initialized(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_is_enabled
 *
 * Description:
 *   Check if IIC controller is enabled based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *
 * Returned Value:
 *   true if enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_iic_is_enabled(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_get_status
 *
 * Description:
 *   Get IIC controller status flags based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *
 * Returned Value:
 *   Status flags
 *
 ****************************************************************************/

uint8_t ra8p_iic_get_status(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_clear_status
 *
 * Description:
 *   Clear IIC controller status flags based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   flags - Flags to clear
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_clear_status(uint8_t channel, uint8_t flags);

/****************************************************************************
 * Name: ra8p_iic_set_timing
 *
 * Description:
 *   Set IIC timing parameters based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   timing - Pointer to timing configuration structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_set_timing(uint8_t channel, const struct ra8p_iic_timing_s *timing);

/****************************************************************************
 * Name: ra8p_iic_enable_interrupts
 *
 * Description:
 *   Enable IIC interrupts based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_enable_interrupts(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_iic_set_dma
 *
 * Description:
 *   Enable/disable DMA for IIC transfers based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   enable - true to enable DMA, false to use PIO
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_set_dma(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_iic_reset
 *
 * Description:
 *   Reset IIC controller based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_reset(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_get_error_flags
 *
 * Description:
 *   Get IIC controller error flags based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *
 * Returned Value:
 *   Error flags
 *
 ****************************************************************************/

uint8_t ra8p_iic_get_error_flags(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_clear_errors
 *
 * Description:
 *   Clear IIC controller error flags based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_clear_errors(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_enable_timeout
 *
 * Description:
 *   Enable/disable timeout detection for IIC based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   enable - true to enable timeout, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_enable_timeout(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_iic_is_busy
 *
 * Description:
 *   Check if I2C bus is busy based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *
 * Returned Value:
 *   true if busy, false otherwise
 *
 ****************************************************************************/

bool ra8p_iic_is_busy(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_send_start
 *
 * Description:
 *   Send START condition on I2C bus based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_send_start(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_send_stop
 *
 * Description:
 *   Send STOP condition on I2C bus based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_send_stop(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_write_byte
 *
 * Description:
 *   Write a byte to I2C bus based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   data - Byte to write
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_write_byte(uint8_t channel, uint8_t data);

/****************************************************************************
 * Name: ra8p_iic_read_byte
 *
 * Description:
 *   Read a byte from I2C bus based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   data - Pointer to store read byte
 *   nack_last - true to send NACK for last byte, false to send ACK
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_read_byte(uint8_t channel, uint8_t *data, bool nack_last);

/****************************************************************************
 * Name: ra8p_iic_set_pullup
 *
 * Description:
 *   Set SCL/SDA pull-up resistors based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   scl_pullup - true to enable SCL pull-up, false to disable
 *   sda_pullup - true to enable SDA pull-up, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_set_pullup(uint8_t channel, bool scl_pullup, bool sda_pullup);

/****************************************************************************
 * Name: ra8p_iic_enable_noise_filter
 *
 * Description:
 *   Enable/disable noise filter for IIC bus based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   enable - true to enable noise filter, false to disable
 *   stages - Number of filter stages (0-3)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_enable_noise_filter(uint8_t channel, bool enable, uint8_t stages);

/****************************************************************************
 * Name: ra8p_iic_is_slave_mode
 *
 * Description:
 *   Check if IIC is in slave mode based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *
 * Returned Value:
 *   true if in slave mode, false if in master mode
 *
 ****************************************************************************/

bool ra8p_iic_is_slave_mode(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_set_slave_mode
 *
 * Description:
 *   Set IIC to slave mode based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   enable - true to enable slave mode, false for master mode
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_set_slave_mode(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_iic_set_slave_address
 *
 * Description:
 *   Set IIC slave address based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   addr - 7-bit or 10-bit slave address
 *   index - Slave address index (0-2)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_set_slave_address(uint8_t channel, uint16_t addr, uint8_t index);

/****************************************************************************
 * Name: ra8p_iic_set_fast_mode_plus
 *
 * Description:
 *   Enable/disable fast mode plus (1 MHz) for IIC based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   enable - true to enable FM+, false for standard/fast mode
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_set_fast_mode_plus(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_iic_is_fast_mode_plus
 *
 * Description:
 *   Check if IIC fast mode plus is enabled based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *
 * Returned Value:
 *   true if FM+ enabled, false otherwise
 *
 ****************************************************************************/

bool ra8p_iic_is_fast_mode_plus(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_enable_digital_filter
 *
 * Description:
 *   Enable/disable digital filter for IIC based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   enable - true to enable digital filter, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_enable_digital_filter(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_iic_set_clock_divider
 *
 * Description:
 *   Set IIC clock divider based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   divider - Clock divider value (0-15)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_set_clock_divider(uint8_t channel, uint8_t divider);

/****************************************************************************
 * Name: ra8p_iic_get_clock_divider
 *
 * Description:
 *   Get current IIC clock divider based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *
 * Returned Value:
 *   Current clock divider value
 *
 ****************************************************************************/

uint8_t ra8p_iic_get_clock_divider(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_get_frequency
 *
 * Description:
 *   Get current I2C bus frequency based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *
 * Returned Value:
 *   Current frequency in Hz
 *
 ****************************************************************************/

uint32_t ra8p_iic_get_frequency(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_attach_slave_handler
 *
 * Description:
 *   Attach slave handler for IIC based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   handler - Callback function
 *   arg - Handler argument
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_attach_slave_handler(uint8_t channel, ra8p_iic_handler_t handler, void *arg);

/****************************************************************************
 * Name: ra8p_iic_detach_slave_handler
 *
 * Description:
 *   Detach slave handler for IIC based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_detach_slave_handler(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_get_bus_state
 *
 * Description:
 *   Get I2C bus state based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *
 * Returned Value:
 *   Bus state (0=idle, 1=busy, 2=error)
 *
 ****************************************************************************/

uint8_t ra8p_iic_get_bus_state(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_enable_arbitration_lost_detection
 *
 * Description:
 *   Enable/disable arbitration lost detection based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   enable - true to enable, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_enable_arbitration_lost_detection(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_iic_is_arbitration_lost
 *
 * Description:
 *   Check if arbitration was lost based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *
 * Returned Value:
 *   true if arbitration lost, false otherwise
 *
 ****************************************************************************/

bool ra8p_iic_is_arbitration_lost(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_clear_arbitration_lost
 *
 * Description:
 *   Clear arbitration lost flag based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_clear_arbitration_lost(uint8_t channel);

/****************************************************************************
 * Name: ra8p_iic_enable_general_call
 *
 * Description:
 *   Enable/disable general call address detection based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   enable - true to enable general call, false to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_enable_general_call(uint8_t channel, bool enable);

/****************************************************************************
 * Name: ra8p_iic_set_timeout_period
 *
 * Description:
 *   Set I2C timeout period based on Nuttx I2C driver implementation.
 *
 * Input Parameters:
 *   channel - I2C channel (0-2)
 *   period - Timeout period value
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_iic_set_timeout_period(uint8_t channel, uint8_t period);

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_IIC_H */