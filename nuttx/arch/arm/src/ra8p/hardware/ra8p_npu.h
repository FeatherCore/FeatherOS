/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_npu.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_NPU_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_NPU_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* Ethos-U55 NPU Register Offsets - Based on ARM Ethos-U55 Technical Reference Manual */
#define RA8P_NPU_CTRL_OFFSET             0x0000  /* Control Register */
#define RA8P_NPU_STATUS_OFFSET           0x0004  /* Status Register */
#define RA8P_NPU_CMD_OFFSET              0x0008  /* Command Register */
#define RA8P_NPU_INT_STATUS_OFFSET       0x000C  /* Interrupt Status Register */
#define RA8P_NPU_INT_ENABLE_OFFSET       0x0010  /* Interrupt Enable Register */
#define RA8P_NPU_SHRAM_BASE_OFFSET       0x0020  /* SHRAM Base Address Register */
#define RA8P_NPU_SHRAM_SIZE_OFFSET       0x0024  /* SHRAM Size Register */
#define RA8P_NPU_CMD_QUEUE_BASE_OFFSET   0x0028  /* Command Queue Base Register */
#define RA8P_NPU_CMD_QUEUE_SIZE_OFFSET   0x002C  /* Command Queue Size Register */
#define RA8P_NPU_CMD_QUEUE_READ_OFFSET   0x0030  /* Command Queue Read Pointer */
#define RA8P_NPU_CMD_QUEUE_WRITE_OFFSET  0x0034  /* Command Queue Write Pointer */
#define RA8P_NPU_CMD_QUEUE_PENDING_OFFSET 0x0038  /* Command Queue Pending Register */
#define RA8P_NPU_CMD_QUEUE_STATUS_OFFSET 0x003C  /* Command Queue Status Register */
#define RA8P_NPU_PWR_CTRL_OFFSET         0x0040  /* Power Control Register */
#define RA8P_NPU_PWR_STATUS_OFFSET       0x0044  /* Power Status Register */
#define RA8P_NPU_VERSION_OFFSET          0x0048  /* Version Register */
#define RA8P_NPU_ERROR_OFFSET            0x004C  /* Error Register */
#define RA8P_NPU_DEBUG_OFFSET            0x0050  /* Debug Register */
#define RA8P_NPU_PERF_CTRL_OFFSET        0x0054  /* Performance Control Register */
#define RA8P_NPU_PERF_STATUS_OFFSET      0x0058  /* Performance Status Register */
#define RA8P_NPU_SECURE_OFFSET           0x005C  /* Secure Register */

/* NPU Control Register bits */
#define RA8P_NPU_CTRL_NPUEN              (1 << 0)   /* NPU Enable */
#define RA8P_NPU_CTRL_NPURESET           (1 << 1)   /* NPU Reset */
#define RA8P_NPU_CTRL_SHRAMEN            (1 << 2)   /* SHRAM Enable */
#define RA8P_NPU_CTRL_CMDQEN             (1 << 3)   /* Command Queue Enable */
#define RA8P_NPU_CTRL_CLKGATEEN          (1 << 4)   /* Clock Gating Enable */
#define RA8P_NPU_CTRL_SLEEPEN            (1 << 5)   /* Sleep Enable */
#define RA8P_NPU_CTRL_PWRGATEEN          (1 << 6)   /* Power Gating Enable */
#define RA8P_NPU_CTRL_PROFILINGEN        (1 << 7)   /* Profiling Enable */

/* NPU Status Register bits */
#define RA8P_NPU_STATUS_NPUSTS            (1 << 0)   /* NPU Status */
#define RA8P_NPU_STATUS_PWRSTS            (1 << 1)   /* Power Status */
#define RA8P_NPU_STATUS_CLKSTS            (1 << 2)   /* Clock Status */
#define RA8P_NPU_STATUS_SLEEPSTS          (1 << 3)   /* Sleep Status */
#define RA8P_NPU_STATUS_CMDQEMPTY         (1 << 4)   /* Command Queue Empty */
#define RA8P_NPU_STATUS_CMDQFULL          (1 << 5)   /* Command Queue Full */
#define RA8P_NPU_STATUS_CMDQVALID         (1 << 6)   /* Command Queue Valid */
#define RA8P_NPU_STATUS_ERRORSTS          (1 << 7)   /* Error Status */

/* NPU Command Register bits */
#define RA8P_NPU_CMD_CMDTYPE_MASK         (0x0F << 0)  /* Command Type */
#define RA8P_NPU_CMD_CMDTYPE_SHIFT        0
#define RA8P_NPU_CMD_CMDTYPE_EXECUTE      0           /* Execute Command */
#define RA8P_NPU_CMD_CMDTYPE_LOAD_MODEL   1           /* Load Model Command */
#define RA8P_NPU_CMD_CMDTYPE_SAVE_MODEL   2           /* Save Model Command */
#define RA8P_NPU_CMD_CMDTYPE_FLUSH_CACHE  3           /* Flush Cache Command */
#define RA8P_NPU_CMD_CMDTYPE_INVALIDATE_CACHE 4       /* Invalidate Cache Command */
#define RA8P_NPU_CMD_CMDQPTR_MASK         (0x03 << 4)  /* Command Queue Pointer */
#define RA8P_NPU_CMD_CMDQPTR_SHIFT        4
#define RA8P_NPU_CMD_CMDQPTR_READ         0           /* Read Pointer */
#define RA8P_NPU_CMD_CMDQPTR_WRITE        1           /* Write Pointer */

/* NPU Interrupt Status Register bits */
#define RA8P_NPU_INT_STATUS_CMD_DONE      (1 << 0)   /* Command Done Interrupt */
#define RA8P_NPU_INT_STATUS_CMD_ERR       (1 << 1)   /* Command Error Interrupt */
#define RA8P_NPU_INT_STATUS_CMD_QUEUE_EMPTY (1 << 2) /* Command Queue Empty */
#define RA8P_NPU_INT_STATUS_CMD_QUEUE_FULL (1 << 3)  /* Command Queue Full */
#define RA8P_NPU_INT_STATUS_PWR_CHANGED   (1 << 4)   /* Power Changed Interrupt */
#define RA8P_NPU_INT_STATUS_CACHE_MISS    (1 << 5)   /* Cache Miss Interrupt */
#define RA8P_NPU_INT_STATUS_TIMEOUT       (1 << 6)   /* Timeout Interrupt */
#define RA8P_NPU_INT_STATUS_FATAL_ERR     (1 << 7)   /* Fatal Error Interrupt */

/* NPU Power Control Register bits */
#define RA8P_NPU_PWR_CTRL_PWR_ON          (1 << 0)   /* Power On */
#define RA8P_NPU_PWR_CTRL_PWR_OFF         (1 << 1)   /* Power Off */
#define RA8P_NPU_PWR_CTRL_PWR_RETENTION   (1 << 2)   /* Power Retention */
#define RA8P_NPU_PWR_CTRL_PWR_SLEEP       (1 << 3)   /* Power Sleep */
#define RA8P_NPU_PWR_CTRL_PWR_WAKEUP      (1 << 4)   /* Power Wake-up */

/* NPU Error Register bits */
#define RA8P_NPU_ERROR_CMD_ERR            (1 << 0)   /* Command Error */
#define RA8P_NPU_ERROR_DATA_ERR           (1 << 1)   /* Data Error */
#define RA8P_NPU_ERROR_ADDR_ERR           (1 << 2)   /* Address Error */
#define RA8P_NPU_ERROR_TIMEOUT_ERR        (1 << 3)   /* Timeout Error */
#define RA8P_NPU_ERROR_ALIGN_ERR          (1 << 4)   /* Alignment Error */
#define RA8P_NPU_ERROR_RANGE_ERR          (1 << 5)   /* Range Error */
#define RA8P_NPU_ERROR_INVALID_OP_ERR     (1 << 6)   /* Invalid Operation Error */
#define RA8P_NPU_ERROR_FATAL_ERR          (1 << 7)   /* Fatal Error */

/* NPU Performance Control Register bits */
#define RA8P_NPU_PERF_CTRL_CNT_RST        (1 << 0)   /* Counter Reset */
#define RA8P_NPU_PERF_CTRL_CNT_EN         (1 << 1)   /* Counter Enable */
#define RA8P_NPU_PERF_CTRL_CNT_SEL_MASK   (0x0F << 2) /* Counter Select */
#define RA8P_NPU_PERF_CTRL_CNT_SEL_SHIFT  2

/* NPU Performance Status Register */
#define RA8P_NPU_PERF_STATUS_CYCLES_MASK  (0xFFFFFFFF << 0) /* Cycle Counter */
#define RA8P_NPU_PERF_STATUS_CYCLES_SHIFT 0

/* NPU Base Address */
#define RA8P_NPU_BASE                    0x40140000
#define RA8P_NPU_SIZE                    0x8000

/* NPU Interrupt Number */
#define RA8P_IRQ_NPU                     101

/* NPU Constants */
#define RA8P_NPU_MAX_SHRAM_SIZE          (64 * 1024) /* 64KB SHRAM */
#define RA8P_NPU_MAX_CMD_QUEUE_SIZE      (4 * 1024)  /* 4KB Command Queue */
#define RA8P_NPU_VERSION_MAJOR           0x05        /* Major version: 5 */
#define RA8P_NPU_VERSION_MINOR           0x05        /* Minor version: 5 */

/* Command Queue entry structure */
#define RA8P_NPU_CMD_QUEUE_ENTRY_SIZE    64          /* 64 bytes per entry */
#define RA8P_NPU_MAX_CMD_QUEUE_ENTRIES   (RA8P_NPU_MAX_CMD_QUEUE_SIZE / RA8P_NPU_CMD_QUEUE_ENTRY_SIZE)

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* Ethos-U55 NPU tensor data type */
enum ra8p_npu_datatype_e
{
  RA8P_NPU_DATATYPE_INT8 = 0,
  RA8P_NPU_DATATYPE_UINT8,
  RA8P_NPU_DATATYPE_INT16,
  RA8P_NPU_DATATYPE_UINT16,
  RA8P_NPU_DATATYPE_INT32,
  RA8P_NPU_DATATYPE_UINT32,
  RA8P_NPU_DATATYPE_FLOAT16,
  RA8P_NPU_DATATYPE_FLOAT32,
};

/* Ethos-U55 NPU tensor dimension */
struct ra8p_npu_dimension_s
{
  uint32_t height;
  uint32_t width;
  uint32_t channels;
  uint32_t batches;
};

/* Ethos-U55 NPU tensor structure */
struct ra8p_npu_tensor_s
{
  enum ra8p_npu_datatype_e datatype;
  struct ra8p_npu_dimension_s dim;
  uint32_t *buffer;                  /* Physical address of tensor data */
  uint32_t buffer_size;              /* Size of tensor buffer in bytes */
  uint32_t quant_scale;              /* Quantization scale */
  uint32_t quant_zero_point;         /* Quantization zero point */
};

/* Ethos-U55 NPU operation */
struct ra8p_npu_operation_s
{
  uint32_t op_type;                  /* Operation type (conv, pool, etc.) */
  struct ra8p_npu_tensor_s *inputs;  /* Input tensors */
  uint8_t input_count;              /* Number of input tensors */
  struct ra8p_npu_tensor_s *outputs; /* Output tensors */
  uint8_t output_count;             /* Number of output tensors */
  uint32_t *weights;                /* Physical address of weights */
  uint32_t weights_size;            /* Size of weights in bytes */
  uint32_t *bias;                   /* Physical address of bias */
  uint32_t bias_size;               /* Size of bias in bytes */
  uint32_t flags;                   /* Operation flags */
};

/* Ethos-U55 NPU command queue entry */
struct ra8p_npu_cmd_entry_s
{
  uint32_t cmd_type;                 /* Command type */
  uint32_t cmd_id;                   /* Command ID */
  uint32_t src_addr;                 /* Source address */
  uint32_t dst_addr;                 /* Destination address */
  uint32_t length;                   /* Length in bytes */
  uint32_t flags;                    /* Flags */
  uint32_t reserved[56/4 - 6];       /* Padding to 64 bytes */
};

/* Ethos-U55 NPU configuration */
struct ra8p_npu_config_s
{
  bool enable;                       /* Enable NPU */
  bool shram_enable;                 /* Enable SHRAM */
  bool cmd_queue_enable;             /* Enable command queue */
  uint32_t shram_base;               /* SHRAM base address */
  uint32_t shram_size;               /* SHRAM size */
  uint32_t cmd_queue_base;           /* Command queue base address */
  uint32_t cmd_queue_size;           /* Command queue size */
  uint8_t clock_divider;             /* Clock divider */
  bool profiling_enabled;            /* Enable profiling */
  bool secure_mode;                  /* Secure mode enabled */
};

/* Ethos-U55 NPU statistics */
struct ra8p_npu_stats_s
{
  uint32_t cycles_executed;          /* Total cycles executed */
  uint32_t commands_processed;        /* Total commands processed */
  uint32_t cache_hits;               /* Cache hits */
  uint32_t cache_misses;             /* Cache misses */
  uint32_t error_count;              /* Total errors */
  uint32_t power_states;             /* Power state transitions */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_npu_initialize
 *
 * Description:
 *   Initialize the Ethos-U55 NPU based on Zephyr ethos_u.c implementation.
 *
 * Input Parameters:
 *   config - Pointer to NPU configuration structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_npu_initialize(const struct ra8p_npu_config_s *config);

/****************************************************************************
 * Name: ra8p_npu_enable
 *
 * Description:
 *   Enable the Ethos-U55 NPU.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_npu_enable(void);

/****************************************************************************
 * Name: ra8p_npu_disable
 *
 * Description:
 *   Disable the Ethos-U55 NPU.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_npu_disable(void);

/****************************************************************************
 * Name: ra8p_npu_reset
 *
 * Description:
 *   Reset the Ethos-U55 NPU.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_npu_reset(void);

/****************************************************************************
 * Name: ra8p_npu_execute_operation
 *
 * Description:
 *   Execute an NPU operation based on Zephyr ethos_u.c implementation.
 *
 * Input Parameters:
 *   op - Pointer to operation structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_npu_execute_operation(const struct ra8p_npu_operation_s *op);

/****************************************************************************
 * Name: ra8p_npu_load_model
 *
 * Description:
 *   Load a neural network model to NPU based on Zephyr ethos_u.c implementation.
 *
 * Input Parameters:
 *   model_data - Pointer to model data
 *   model_size - Size of model data
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_npu_load_model(const uint8_t *model_data, uint32_t model_size);

/****************************************************************************
 * Name: ra8p_npu_set_tensor
 *
 * Description:
 *   Set tensor data for NPU based on Zephyr ethos_u.c implementation.
 *
 * Input Parameters:
 *   tensor - Pointer to tensor structure
 *   data - Pointer to tensor data
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_npu_set_tensor(const struct ra8p_npu_tensor_s *tensor, const void *data);

/****************************************************************************
 * Name: ra8p_npu_get_tensor
 *
 * Description:
 *   Get tensor data from NPU based on Zephyr ethos_u.c implementation.
 *
 * Input Parameters:
 *   tensor - Pointer to tensor structure
 *   data - Pointer to buffer for tensor data
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_npu_get_tensor(const struct ra8p_npu_tensor_s *tensor, void *data);

/****************************************************************************
 * Name: ra8p_npu_flush_cache
 *
 * Description:
 *   Flush NPU cache based on Zephyr ethos_u.c implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_npu_flush_cache(void);

/****************************************************************************
 * Name: ra8p_npu_invalidate_cache
 *
 * Description:
 *   Invalidate NPU cache based on Zephyr ethos_u.c implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_npu_invalidate_cache(void);

/****************************************************************************
 * Name: ra8p_npu_get_status
 *
 * Description:
 *   Get NPU status based on Zephyr ethos_u.c implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   NPU status bits
 *
 ****************************************************************************/

uint32_t ra8p_npu_get_status(void);

/****************************************************************************
 * Name: ra8p_npu_get_stats
 *
 * Description:
 *   Get NPU statistics based on Zephyr ethos_u.c implementation.
 *
 * Input Parameters:
 *   stats - Pointer to stats structure to fill
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_npu_get_stats(struct ra8p_npu_stats_s *stats);

/****************************************************************************
 * Name: ra8p_npu_set_power_mode
 *
 * Description:
 *   Set NPU power mode based on Zephyr ethos_u.c implementation.
 *
 * Input Parameters:
 *   mode - Power mode (0=normal, 1=low power, 2=sleep)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_npu_set_power_mode(uint8_t mode);

/****************************************************************************
 * Name: ra8p_npu_wait_ready
 *
 * Description:
 *   Wait for NPU to be ready based on Zephyr ethos_u.c implementation.
 *
 * Input Parameters:
 *   timeout_ms - Timeout in milliseconds
 *
 * Returned Value:
 *   OK on success, negated errno on timeout
 *
 ****************************************************************************/

int ra8p_npu_wait_ready(uint32_t timeout_ms);

/****************************************************************************
 * Name: ra8p_npu_is_error
 *
 * Description:
 *   Check if NPU has error based on Zephyr ethos_u.c implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if error occurred, false otherwise
 *
 ****************************************************************************/

bool ra8p_npu_is_error(void);

/****************************************************************************
 * Name: ra8p_npu_clear_error
 *
 * Description:
 *   Clear NPU error flags based on Zephyr ethos_u.c implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_npu_clear_error(void);

/****************************************************************************
 * Name: ra8p_npu_enable_interrupt
 *
 * Description:
 *   Enable NPU interrupts based on Zephyr ethos_u.c implementation.
 *
 * Input Parameters:
 *   interrupt_mask - Interrupt mask to enable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_npu_enable_interrupt(uint32_t interrupt_mask);

/****************************************************************************
 * Name: ra8p_npu_disable_interrupt
 *
 * Description:
 *   Disable NPU interrupts based on Zephyr ethos_u.c implementation.
 *
 * Input Parameters:
 *   interrupt_mask - Interrupt mask to disable
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_npu_disable_interrupt(uint32_t interrupt_mask);

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_NPU_H */