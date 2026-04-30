/****************************************************************************
 * arch/arm/src/ra8p/hardware/ra8p_ceu.h
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

#ifndef __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_CEU_H
#define __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_CEU_H

/****************************************************************************
 * Included Files
 ****************************************************************************/

#include <nuttx/config.h>

/****************************************************************************
 * Pre-processor Definitions
 ****************************************************************************/

/* CEU Register Offsets - Based on RA8P Hardware Manual and Zephyr definitions */
#define RA8P_CEU_CEUDLYC_OFFSET          0x0000  /* Data Delay Control Register */
#define RA8P_CEU_CEUCTR_OFFSET           0x0004  /* CEU Control Register */
#define RA8P_CEU_CEUICR_OFFSET           0x0008  /* Input Control Register */
#define RA8P_CEU_CEUIDSR_OFFSET          0x000C  /* Input Data Swap Register */
#define RA8P_CEU_CEUIWCR_OFFSET          0x0010  /* Input Window Clip Register */
#define RA8P_CEU_CEUIHCR_OFFSET          0x0014  /* Input Horizontal Clip Register */
#define RA8P_CEU_CEUIWC_OFFSET           0x0018  /* Input Window Clip Register */
#define RA8P_CEU_CEUIVC_OFFSET           0x001C  /* Input Vertical Clip Register */
#define RA8P_CEU_CEUIWCS_OFFSET          0x0020  /* Input Window Clip Start Register */
#define RA8P_CEU_CEUIVCS_OFFSET          0x0024  /* Input Vertical Clip Start Register */
#define RA8P_CEU_CEUIWCE_OFFSET          0x0028  /* Input Window Clip End Register */
#define RA8P_CEU_CEUIVCE_OFFSET          0x002C  /* Input Vertical Clip End Register */
#define RA8P_CEU_CEUISCR_OFFSET          0x0030  /* Input Size Control Register */
#define RA8P_CEU_CEUICCR_OFFSET          0x0034  /* Input Color Compensation Register */
#define RA8P_CEU_CEUICCR2_OFFSET         0x0038  /* Input Color Compensation Register 2 */
#define RA8P_CEU_CEUFCR_OFFSET          0x0040  /* FIFO Control Register */
#define RA8P_CEU_CEUFSR_OFFSET          0x0044  /* FIFO Status Register */
#define RA8P_CEU_CEUFWCR_OFFSET          0x0048  /* FIFO Write Control Register */
#define RA8P_CEU_CEUFVCR_OFFSET          0x004C  /* FIFO Valid Control Register */
#define RA8P_CEU_CEUFDCA0_OFFSET         0x0050  /* FIFO Data Count Address 0 Register */
#define RA8P_CEU_CEUFDCA1_OFFSET         0x0054  /* FIFO Data Count Address 1 Register */
#define RA8P_CEU_CEUFWC_OFFSET           0x0060  /* FIFO Write Counter Register */
#define RA8P_CEU_CEUVC_OFFSET            0x0064  /* Valid Counter Register */
#define RA8P_CEU_CEUEIWF_OFFSET          0x0070  /* Even Input Field Write Flag Register */
#define RA8P_CEU_CEUOIWF_OFFSET          0x0074  /* Odd Input Field Write Flag Register */
#define RA8P_CEU_CEUFPF_OFFSET           0x0078  /* FIFO Pack Flag Register */
#define RA8P_CEU_CEUDBR_OFFSET           0x007C  /* Debug Register */
#define RA8P_CEU_CEUCRDR_OFFSET          0x0080  /* Capture Read Data Register */
#define RA8P_CEU_CEUCCDR_OFFSET          0x0084  /* Capture Compare Data Register */
#define RA8P_CEU_CEUCAFR_OFFSET          0x0088  /* Capture Alpha Blending Factor Register */
#define RA8P_CEU_CEUCAFDR_OFFSET         0x008C  /* Capture Alpha Blending Factor Data Register */
#define RA8P_CEU_CEUDBLCR_OFFSET         0x0090  /* Debug Luminance Control Register */
#define RA8P_CEU_CEUDBLDR_OFFSET         0x0094  /* Debug Luminance Data Register */
#define RA8P_CEU_CEUDBCCR_OFFSET         0x0098  /* Debug Chrominance Control Register */
#define RA8P_CEU_CEUDBCDR_OFFSET         0x009C  /* Debug Chrominance Data Register */
#define RA8P_CEU_CEUINTR_OFFSET          0x00A0  /* Interrupt Register */
#define RA8P_CEU_CEUINTSR_OFFSET         0x00A4  /* Interrupt Status Register */
#define RA8P_CEU_CEUINTRMSK_OFFSET       0x00A8  /* Interrupt Mask Register */

/* CEUCTR - CEU Control Register */
#define RA8P_CEU_CEUCTR_CEUEN            (1 << 0)   /* CEU Enable */
#define RA8P_CEU_CEUCTR_IEN              (1 << 1)   /* Input Enable */
#define RA8P_CEU_CEUCTR_OEN              (1 << 2)   /* Output Enable */
#define RA8P_CEU_CEUCTR_CKEYEN           (1 << 3)   /* Color Key Enable */
#define RA8P_CEU_CEUCTR_CKICEN           (1 << 4)   /* Color Key Input Change Enable */
#define RA8P_CEU_CEUCTR_CKOCEN           (1 << 5)   /* Color Key Output Change Enable */
#define RA8P_CEU_CEUCTR_RGBSFT           (1 << 8)   /* RGB Shift Enable */
#define RA8P_CEU_CEUCTR_YCSWAP           (1 << 9)   /* YC Swap Enable */
#define RA8P_CEU_CEUCTR_FIELDINV         (1 << 10)  /* Field Inversion */
#define RA8P_CEU_CEUCTR_FLDINV           (1 << 11)  /* Field Inversion */
#define RA8P_CEU_CEUCTR_HFLIP            (1 << 12)  /* Horizontal Flip */
#define RA8P_CEU_CEUCTR_VFLIP            (1 << 13)  /* Vertical Flip */
#define RA8P_CEU_CEUCTR_CIPRST           (1 << 16)  /* Capture Input Path Reset */
#define RA8P_CEU_CEUCTR_COEFEN           (1 << 17)  /* Coefficient Enable */
#define RA8P_CEU_CEUCTR_CIPRST2          (1 << 18)  /* Capture Input Path Reset 2 */
#define RA8P_CEU_CEUCTR_FIFORST          (1 << 24)  /* FIFO Reset */

/* CEUICR - Input Control Register */
#define RA8P_CEU_CEUICR_HDPOS            (1 << 0)   /* HD Position */
#define RA8P_CEU_CEUICR_VDPOS            (1 << 1)   /* VD Position */
#define RA8P_CEU_CEUICR_FDPOS            (1 << 2)   /* FD Position */
#define RA8P_CEU_CEUICR_CPTON            (1 << 4)   /* Capture On */
#define RA8P_CEU_CEUICR_HDSEL            (1 << 8)   /* HD Select */
#define RA8P_CEU_CEUICR_VDSEL            (1 << 9)   /* VD Select */
#define RA8P_CEU_CEUICR_FDSEL            (1 << 10)  /* FD Select */
#define RA8P_CEU_CEUICR_HDPOL            (1 << 12)  /* HD Polarity */
#define RA8P_CEU_CEUICR_VDPOL            (1 << 13)  /* VD Polarity */
#define RA8P_CEU_CEUICR_FDPOL            (1 << 14)  /* FD Polarity */

/* CEUICSR - Input Data Swap Register */
#define RA8P_CEU_CEUIDSR_INRGBSFT_MASK   (0x07 << 0)  /* Input RGB Shift Mask */
#define RA8P_CEU_CEUIDSR_INRGBSFT_SHIFT  0
#define RA8P_CEU_CEUIDSR_INYCSWAP        (1 << 4)    /* Input YC Swap */
#define RA8P_CEU_CEUIDSR_INCDSWAP        (1 << 5)    /* Input CD Swap */
#define RA8P_CEU_CEUIDSR_INCBYRVS        (1 << 6)    /* Input CB/CR Reverse */
#define RA8P_CEU_CEUIDSR_INYCFE          (1 << 7)    /* Input YC Format Expansion */
#define RA8P_CEU_CEUIDSR_OUTFORM_MASK    (0x0F << 8)  /* Output Format Mask */
#define RA8P_CEU_CEUIDSR_OUTFORM_SHIFT   8
#define RA8P_CEU_CEUIDSR_INDATM_MASK     (0x07 << 12) /* Input Data Mode Mask */
#define RA8P_CEU_CEUIDSR_INDATM_SHIFT    12

/* CEUFCR - FIFO Control Register */
#define RA8P_CEU_CEUFCR_CU0              (1 << 0)   /* Capture Unit 0 Enable */
#define RA8P_CEU_CEUFCR_CU1              (1 << 1)   /* Capture Unit 1 Enable */
#define RA8P_CEU_CEUFCR_CU2              (1 << 2)   /* Capture Unit 2 Enable */
#define RA8P_CEU_CEUFCR_CU3              (1 << 3)   /* Capture Unit 3 Enable */
#define RA8P_CEU_CEUFCR_CU4              (1 << 4)   /* Capture Unit 4 Enable */
#define RA8P_CEU_CEUFCR_CU5              (1 << 5)   /* Capture Unit 5 Enable */
#define RA8P_CEU_CEUFCR_CU6              (1 << 6)   /* Capture Unit 6 Enable */
#define RA8P_CEU_CEUFCR_CU7              (1 << 7)   /* Capture Unit 7 Enable */
#define RA8P_CEU_CEUFCR_FIFORST0         (1 << 8)   /* FIFO Reset 0 */
#define RA8P_CEU_CEUFCR_FIFORST1         (1 << 9)   /* FIFO Reset 1 */
#define RA8P_CEU_CEUFCR_FIFORST2         (1 << 10)  /* FIFO Reset 2 */
#define RA8P_CEU_CEUFCR_FIFORST3         (1 << 11)  /* FIFO Reset 3 */
#define RA8P_CEU_CEUFCR_FIFORST4         (1 << 12)  /* FIFO Reset 4 */
#define RA8P_CEU_CEUFCR_FIFORST5         (1 << 13)  /* FIFO Reset 5 */
#define RA8P_CEU_CEUFCR_FIFORST6         (1 << 14)  /* FIFO Reset 6 */
#define RA8P_CEU_CEUFCR_FIFORST7         (1 << 15)  /* FIFO Reset 7 */

/* CEUINTSR - Interrupt Status Register */
#define RA8P_CEU_CEUINTSR_CU0I           (1 << 0)   /* Capture Unit 0 Interrupt */
#define RA8P_CEU_CEUINTSR_CU1I           (1 << 1)   /* Capture Unit 1 Interrupt */
#define RA8P_CEU_CEUINTSR_CU2I           (1 << 2)   /* Capture Unit 2 Interrupt */
#define RA8P_CEU_CEUINTSR_CU3I           (1 << 3)   /* Capture Unit 3 Interrupt */
#define RA8P_CEU_CEUINTSR_CU4I           (1 << 4)   /* Capture Unit 4 Interrupt */
#define RA8P_CEU_CEUINTSR_CU5I           (1 << 5)   /* Capture Unit 5 Interrupt */
#define RA8P_CEU_CEUINTSR_CU6I           (1 << 6)   /* Capture Unit 6 Interrupt */
#define RA8P_CEU_CEUINTSR_CU7I           (1 << 7)   /* Capture Unit 7 Interrupt */
#define RA8P_CEU_CEUINTSR_FEOI           (1 << 8)   /* Field End Interrupt */
#define RA8P_CEU_CEUINTSR_FEI            (1 << 9)   /* Field Interrupt */

/* CEU Base Address */
#define RA8P_CEU_BASE                    0x40348000
#define RA8P_CEU_SIZE                    0x8000

/* CEU Interrupt Numbers */
#define RA8P_IRQ_CEU                     100

/* CEU Capture Units */
#define RA8P_CEU_UNITS                   8         /* 8 capture units */

/* Video formats supported by CEU */
#define RA8P_CEU_FORMAT_RGB565           0x00      /* RGB 565 */
#define RA8P_CEU_FORMAT_RGB888           0x01      /* RGB 888 */
#define RA8P_CEU_FORMAT_YUV422           0x02      /* YUV 4:2:2 */
#define RA8P_CEU_FORMAT_YUV444           0x03      /* YUV 4:4:4 */
#define RA8P_CEU_FORMAT_Y_ONLY           0x04      /* Y-only */

/* Input data modes */
#define RA8P_CEU_INDATA_8BIT             0x00      /* 8-bit data */
#define RA8P_CEU_INDATA_16BIT            0x01      /* 16-bit data */
#define RA8P_CEU_INDATA_24BIT            0x02      /* 24-bit data */
#define RA8P_CEU_INDATA_32BIT            0x03      /* 32-bit data */

/****************************************************************************
 * Public Types
 ****************************************************************************/

/* CEU video format structure */
struct ra8p_ceu_format_s
{
  uint8_t format;                     /* Video format */
  uint8_t bits_per_pixel;             /* Bits per pixel */
  bool packed;                        /* True for packed format */
  bool planar;                        /* True for planar format */
  uint8_t yuv_planes;                 /* Number of YUV planes (0 for RGB) */
};

/* CEU video frame structure */
struct ra8p_ceu_frame_s
{
  uint32_t width;                     /* Image width in pixels */
  uint32_t height;                    /* Image height in pixels */
  struct ra8p_ceu_format_s format;    /* Pixel format */
  uint32_t pitch;                     /* Bytes per line */
  uint32_t size;                      /* Total frame size */
  uint32_t addr;                      /* Physical address of buffer */
  bool interlaced;                    /* True for interlaced video */
  bool top_field_first;              /* Top field first for interlaced */
};

/* CEU capture unit configuration */
struct ra8p_ceu_unit_config_s
{
  uint8_t unit_num;                   /* Capture unit number (0-7) */
  struct ra8p_ceu_frame_s input_frame; /* Input frame format */
  struct ra8p_ceu_frame_s output_frame; /* Output frame format */
  uint32_t crop_left;                 /* Left crop offset */
  uint32_t crop_top;                  /* Top crop offset */
  uint32_t crop_right;                /* Right crop offset */
  uint32_t crop_bottom;               /* Bottom crop offset */
  bool color_key_enabled;             /* Color key enable */
  uint32_t color_key_value;           /* Color key value */
  uint8_t alpha;                      /* Alpha blending value (0-255) */
};

/* CEU controller configuration */
struct ra8p_ceu_config_s
{
  bool enabled;                       /* Enable CEU */
  bool input_enabled;                 /* Enable input path */
  bool output_enabled;                /* Enable output path */
  bool field_inverse;                 /* Field inverse enable */
  bool horizontal_flip;              /* Horizontal flip enable */
  bool vertical_flip;                /* Vertical flip enable */
  uint8_t data_swap;                  /* Data swap mode */
  uint8_t input_format;              /* Input format */
  uint8_t output_format;             /* Output format */
};

/****************************************************************************
 * Public Function Prototypes
 ****************************************************************************/

/****************************************************************************
 * Name: ra8p_ceu_initialize
 *
 * Description:
 *   Initialize the CEU (Camera Engine Unit) based on Zephyr video_renesas_ra_ceu.c implementation.
 *
 * Input Parameters:
 *   config - Pointer to CEU configuration structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ceu_initialize(const struct ra8p_ceu_config_s *config);

/****************************************************************************
 * Name: ra8p_ceu_set_input_format
 *
 * Description:
 *   Set input format for CEU based on Zephyr video_renesas_ra_ceu.c implementation.
 *
 * Input Parameters:
 *   format - Pointer to input format structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ceu_set_input_format(const struct ra8p_ceu_format_s *format);

/****************************************************************************
 * Name: ra8p_ceu_set_output_format
 *
 * Description:
 *   Set output format for CEU based on Zephyr video_renesas_ra_ceu.c implementation.
 *
 * Input Parameters:
 *   format - Pointer to output format structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ceu_set_output_format(const struct ra8p_ceu_format_s *format);

/****************************************************************************
 * Name: ra8p_ceu_capture_start
 *
 * Description:
 *   Start video capture based on Zephyr video_renesas_ra_ceu.c implementation.
 *
 * Input Parameters:
 *   unit - Capture unit (0-7)
 *   frame - Pointer to frame structure
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ceu_capture_start(uint8_t unit, const struct ra8p_ceu_frame_s *frame);

/****************************************************************************
 * Name: ra8p_ceu_capture_stop
 *
 * Description:
 *   Stop video capture based on Zephyr video_renesas_ra_ceu.c implementation.
 *
 * Input Parameters:
 *   unit - Capture unit (0-7)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ceu_capture_stop(uint8_t unit);

/****************************************************************************
 * Name: ra8p_ceu_set_crop
 *
 * Description:
 *   Set crop window for CEU based on Zephyr video_renesas_ra_ceu.c implementation.
 *
 * Input Parameters:
 *   left - Left crop coordinate
 *   top - Top crop coordinate
 *   right - Right crop coordinate
 *   bottom - Bottom crop coordinate
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ceu_set_crop(uint16_t left, uint16_t top, uint16_t right, uint16_t bottom);

/****************************************************************************
 * Name: ra8p_ceu_set_color_key
 *
 * Description:
 *   Set color key for CEU based on Zephyr video_renesas_ra_ceu.c implementation.
 *
 * Input Parameters:
 *   enabled - True to enable color key
 *   color - Color key value
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ceu_set_color_key(bool enabled, uint32_t color);

/****************************************************************************
 * Name: ra8p_ceu_enable_alpha_blending
 *
 * Description:
 *   Enable alpha blending for CEU based on Zephyr video_renesas_ra_ceu.c implementation.
 *
 * Input Parameters:
 *   alpha - Alpha value (0-255)
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ceu_enable_alpha_blending(uint8_t alpha);

/****************************************************************************
 * Name: ra8p_ceu_reset
 *
 * Description:
 *   Reset the CEU based on Zephyr video_renesas_ra_ceu.c implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ceu_reset(void);

/****************************************************************************
 * Name: ra8p_ceu_is_running
 *
 * Description:
 *   Check if CEU is running based on Zephyr video_renesas_ra_ceu.c implementation.
 *
 * Input Parameters:
 *   None
 *
 * Returned Value:
 *   true if running, false otherwise
 *
 ****************************************************************************/

bool ra8p_ceu_is_running(void);

/****************************************************************************
 * Name: ra8p_ceu_get_frame
 *
 * Description:
 *   Get captured frame from CEU based on Zephyr video_renesas_ra_ceu.c implementation.
 *
 * Input Parameters:
 *   unit - Capture unit (0-7)
 *   frame - Pointer to frame structure to fill
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ceu_get_frame(uint8_t unit, struct ra8p_ceu_frame_s *frame);

/****************************************************************************
 * Name: ra8p_ceu_set_flip
 *
 * Description:
 *   Set image flip for CEU based on Zephyr video_renesas_ra_ceu.c implementation.
 *
 * Input Parameters:
 *   horizontal - True for horizontal flip
 *   vertical - True for vertical flip
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ceu_set_flip(bool horizontal, bool vertical);

/****************************************************************************
 * Name: ra8p_ceu_set_field_inverse
 *
 * Description:
 *   Set field inversion for interlaced video based on Zephyr video_renesas_ra_ceu.c implementation.
 *
 * Input Parameters:
 *   enable - True to enable field inversion
 *
 * Returned Value:
 *   OK on success, negated errno on failure
 *
 ****************************************************************************/

int ra8p_ceu_set_field_inverse(bool enable);

#endif /* __ARCH_ARM_SRC_RA8P_HARDWARE_RA8P_CEU_H */