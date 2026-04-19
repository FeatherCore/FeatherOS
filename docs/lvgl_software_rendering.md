# LVGL 软件渲染后端实现分析

## 1. 架构概述

LVGL 9.x 采用 Draw Unit 架构，软件渲染器是其中的核心渲染单元之一。整体架构如下：

```
┌─────────────────────────────────────────────────────────────┐
│                      lv_draw (Draw Manager)                  │
│  - 创建 Draw Task                                            │
│  - 分配任务给 Draw Unit                                       │
│  - 管理渲染图层 (Layer)                                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   Draw Unit (抽象接口)                       │
│  - dispatch_cb: 获取并执行任务                               │
│  - evaluate_cb: 评估任务是否可执行                           │
│  - delete_cb: 清理资源                                       │
└─────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│  SW Draw Unit │    │ GPU Draw Unit │    │  Other Units  │
│  (软件渲染)    │    │ (硬件加速)    │    │ (VG-Lite等)   │
└───────────────┘    └───────────────┘    └───────────────┘
```

### 1.1 核心数据结构

**Draw Task** (`lv_draw_task_t`):
```c
typedef enum {
    LV_DRAW_TASK_TYPE_FILL,         // 填充矩形
    LV_DRAW_TASK_TYPE_BORDER,       // 边框
    LV_DRAW_TASK_TYPE_BOX_SHADOW,   // 阴影
    LV_DRAW_TASK_TYPE_LETTER,       // 单字符
    LV_DRAW_TASK_TYPE_LABEL,        // 文本标签
    LV_DRAW_TASK_TYPE_IMAGE,        // 图像
    LV_DRAW_TASK_TYPE_LAYER,        // 图层混合
    LV_DRAW_TASK_TYPE_LINE,         // 线段
    LV_DRAW_TASK_TYPE_ARC,          // 圆弧
    LV_DRAW_TASK_TYPE_TRIANGLE,     // 三角形
    LV_DRAW_TASK_TYPE_MASK_RECTANGLE, // 矩形遮罩
    LV_DRAW_TASK_TYPE_BLUR,         // 模糊
    LV_DRAW_TASK_TYPE_VECTOR,       // 矢量图形
} lv_draw_task_type_t;
```

**Layer** (`lv_layer_t`):
```c
struct _lv_layer_t {
    lv_draw_buf_t * draw_buf;       // 目标缓冲区
    lv_draw_task_t * draw_task_head; // 任务链表
    lv_area_t buf_area;             // 缓冲区区域
    lv_area_t phy_clip_area;        // 物理裁剪区域
    lv_color_format_t color_format; // 颜色格式
    lv_opa_t opa;                   // 图层透明度
};
```

## 2. 软件渲染器核心实现

### 2.1 文件结构

```
src/draw/sw/
├── lv_draw_sw.c              # 主入口，Draw Unit 注册和任务分发
├── lv_draw_sw.h              # 公共 API
├── lv_draw_sw_private.h      # 内部数据结构
├── lv_draw_sw_fill.c         # 矩形填充
├── lv_draw_sw_border.c       # 边框绘制
├── lv_draw_sw_line.c         # 线段绘制
├── lv_draw_sw_arc.c          # 圆弧绘制
├── lv_draw_sw_img.c          # 图像渲染
├── lv_draw_sw_triangle.c     # 三角形绘制
├── lv_draw_sw_box_shadow.c   # 阴影效果
├── lv_draw_sw_blur.c         # 模糊效果
├── lv_draw_sw_letter.c       # 字符渲染
├── lv_draw_sw_label.c        # 文本渲染
├── lv_draw_sw_mask.c         # 遮罩系统
├── lv_draw_sw_mask_rect.c    # 矩形遮罩
├── lv_draw_sw_grad.c         # 渐变计算
├── lv_draw_sw_transform.c    # 图像变换
├── lv_draw_sw_utils.c        # 工具函数
├── lv_draw_sw_vector.c       # 矢量图形 (ThorVG)
└── blend/                    # Blend 实现
    ├── lv_draw_sw_blend.c    # Blend 入口
    ├── lv_draw_sw_blend_to_rgb565.c
    ├── lv_draw_sw_blend_to_argb8888.c
    ├── lv_draw_sw_blend_to_rgb888.c
    ├── lv_draw_sw_blend_to_l8.c
    ├── lv_draw_sw_blend_to_a8.c
    ├── lv_draw_sw_blend_to_al88.c
    ├── lv_draw_sw_blend_to_i1.c
    ├── neon/                 # ARM NEON 优化
    └── riscv_v/              # RISC-V 向量扩展优化
```

### 2.2 任务分发流程

```c
// lv_draw_sw.c: dispatch()
static int32_t dispatch(lv_draw_unit_t * draw_unit, lv_layer_t * layer)
{
    // 1. 获取可用任务
    t = lv_draw_get_available_task(layer, NULL, DRAW_UNIT_ID_SW);
    
    // 2. 分配缓冲区
    void * buf = lv_draw_layer_alloc_buf(layer);
    
    // 3. 标记任务为进行中
    t->state = LV_DRAW_TASK_STATE_IN_PROGRESS;
    
    // 4. 执行绘制
    execute_drawing(t);
    
    // 5. 标记完成
    t->state = LV_DRAW_TASK_STATE_FINISHED;
}
```

### 2.3 任务执行

```c
static void execute_drawing(lv_draw_task_t * t)
{
    switch(t->type) {
        case LV_DRAW_TASK_TYPE_FILL:
            lv_draw_sw_fill(t, t->draw_dsc, &t->area);
            break;
        case LV_DRAW_TASK_TYPE_BORDER:
            lv_draw_sw_border(t, t->draw_dsc, &t->area);
            break;
        case LV_DRAW_TASK_TYPE_IMAGE:
            lv_draw_sw_image(t, t->draw_dsc, &t->area);
            break;
        case LV_DRAW_TASK_TYPE_LINE:
            lv_draw_line_iterate(t, t->draw_dsc, lv_draw_sw_line);
            break;
        case LV_DRAW_TASK_TYPE_ARC:
            lv_draw_sw_arc(t, t->draw_dsc, &t->area);
            break;
        // ... 其他类型
    }
}
```

## 3. Blend 系统

### 3.1 Blend 描述符

```c
struct _lv_draw_sw_blend_dsc_t {
    const lv_area_t * blend_area;    // 目标区域
    const void * src_buf;            // 源图像缓冲区
    uint32_t src_stride;             // 源图像步长
    lv_color_format_t src_color_format; // 源颜色格式
    const lv_area_t * src_area;      // 源图像区域
    lv_opa_t opa;                    // 整体透明度
    lv_color_t color;                // 填充颜色
    const lv_opa_t * mask_buf;       // 遮罩缓冲区
    lv_draw_sw_mask_res_t mask_res;  // 遮罩结果
    const lv_area_t * mask_area;     // 遮罩区域
    int32_t mask_stride;             // 遮罩步长
    lv_blend_mode_t blend_mode;      // 混合模式
};
```

### 3.2 支持的颜色格式

| 格式 | 说明 | 每像素位数 |
|------|------|-----------|
| RGB565 | 16位 RGB | 16 |
| RGB565_SWAPPED | 字节交换的 RGB565 | 16 |
| RGB888 | 24位 RGB | 24 |
| XRGB8888 | 32位 RGB (无 Alpha) | 32 |
| ARGB8888 | 32位 RGBA | 32 |
| ARGB8888_PREMULTIPLIED | 预乘 Alpha | 32 |
| L8 | 8位灰度 | 8 |
| A8 | 8位 Alpha | 8 |
| AL88 | 16位灰度+Alpha | 16 |
| I1 | 1位单色 | 1 |

### 3.3 Blend 流程

```c
void lv_draw_sw_blend(lv_draw_task_t * t, const lv_draw_sw_blend_dsc_t * blend_dsc)
{
    // 1. 检查透明度
    if(blend_dsc->opa <= LV_OPA_MIN) return;
    
    // 2. 计算实际混合区域
    lv_area_intersect(&blend_area, blend_dsc->blend_area, &t->clip_area);
    
    // 3. 检查自定义 Blend Handler
    lv_draw_sw_blend_handler_t handler = lv_draw_sw_get_blend_handler(layer->color_format);
    if(handler) {
        handler(t, blend_dsc);
        return;
    }
    
    // 4. 根据源类型选择路径
    if(blend_dsc->src_buf == NULL) {
        // 纯色填充
        lv_draw_sw_blend_color(layer->color_format, &fill_dsc);
    } else {
        // 图像混合
        lv_draw_sw_blend_image(layer->color_format, &image_dsc);
    }
}
```

### 3.4 颜色混合算法

**ARGB8888 混合核心**:
```c
static inline lv_color32_t lv_color_32_32_mix(lv_color32_t fg, lv_color32_t bg,
                                              lv_color_mix_alpha_cache_t * cache)
{
    // Porter-Duff "Source Over" 混合
    // result = fg * fg.alpha + bg * (1 - fg.alpha)
    
    if(fg.alpha >= LV_OPA_MAX) {
        return fg;  // 完全覆盖
    }
    if(fg.alpha <= LV_OPA_MIN) {
        return bg;  // 完全透明
    }
    
    // 计算混合比例
    uint32_t ratio = (fg.alpha * 255) / 255;
    
    // 混合各通道
    res.red = (fg.red * ratio + bg.red * (255 - ratio)) / 255;
    res.green = (fg.green * ratio + bg.green * (255 - ratio)) / 255;
    res.blue = (fg.blue * ratio + bg.blue * (255 - ratio)) / 255;
    res.alpha = fg.alpha + (bg.alpha * (255 - fg.alpha)) / 255;
    
    return res;
}
```

### 3.5 混合模式

```c
typedef enum {
    LV_BLEND_MODE_NORMAL,      // 正常
    LV_BLEND_MODE_ADDITIVE,    // 加法
    LV_BLEND_MODE_SUBTRACTIVE, // 减法
    LV_BLEND_MODE_MULTIPLY,    // 乘法
} lv_blend_mode_t;

// 非正常混合模式处理
static inline void blend_non_normal_pixel(lv_color32_t * dest, lv_color32_t src,
                                          lv_blend_mode_t mode, ...)
{
    switch(mode) {
        case LV_BLEND_MODE_ADDITIVE:
            dest->red = LV_MIN(dest->red + src.red, 255);
            dest->green = LV_MIN(dest->green + src.green, 255);
            dest->blue = LV_MIN(dest->blue + src.blue, 255);
            break;
        case LV_BLEND_MODE_SUBTRACTIVE:
            dest->red = LV_MAX(dest->red - src.red, 0);
            dest->green = LV_MAX(dest->green - src.green, 0);
            dest->blue = LV_MAX(dest->blue - src.blue, 0);
            break;
        case LV_BLEND_MODE_MULTIPLY:
            dest->red = (dest->red * src.red) >> 8;
            dest->green = (dest->green * src.green) >> 8;
            dest->blue = (dest->blue * src.blue) >> 8;
            break;
    }
}
```

## 4. 遮罩系统

### 4.1 遮罩类型

```c
typedef enum {
    LV_DRAW_SW_MASK_TYPE_LINE,   // 线段遮罩
    LV_DRAW_SW_MASK_TYPE_ANGLE,  // 角度遮罩
    LV_DRAW_SW_MASK_TYPE_RADIUS, // 圆角遮罩
    LV_DRAW_SW_MASK_TYPE_FADE,   // 渐变遮罩
    LV_DRAW_SW_MASK_TYPE_MAP,    // 位图遮罩
} lv_draw_sw_mask_type_t;
```

### 4.2 遮罩结果

```c
typedef enum {
    LV_DRAW_SW_MASK_RES_TRANSP,    // 完全透明
    LV_DRAW_SW_MASK_RES_FULL_COVER, // 完全覆盖
    LV_DRAW_SW_MASK_RES_CHANGED,   // 部分改变
} lv_draw_sw_mask_res_t;
```

### 4.3 圆角遮罩实现

```c
// 用于绘制圆角矩形
void lv_draw_sw_mask_radius_init(lv_draw_sw_mask_radius_param_t * param,
                                  const lv_area_t * rect, int32_t radius, bool inv)
{
    // 初始化圆角遮罩参数
    // 使用预计算的圆周查找表
}

// 应用遮罩
lv_draw_sw_mask_res_t lv_draw_sw_mask_apply(void * masks[], lv_opa_t * mask_buf,
                                            int32_t abs_x, int32_t abs_y, int32_t len)
{
    // 遍历所有遮罩，计算每个像素的最终透明度
    for each mask in masks:
        mask->cb(mask_buf, abs_x, abs_y, len, mask->param);
}
```

### 4.4 遮罩在填充中的应用

```c
void lv_draw_sw_fill(lv_draw_task_t * t, lv_draw_fill_dsc_t * dsc, const lv_area_t * coords)
{
    // 简单情况：无圆角
    if(dsc->radius == 0 && grad_dir == LV_GRAD_DIR_NONE) {
        lv_draw_sw_blend(t, &blend_dsc);
        return;
    }
    
    // 复杂情况：有圆角
    if(rout > 0) {
        // 创建圆角遮罩
        lv_draw_sw_mask_radius_init(&mask_rout_param, &bg_coords, rout, false);
        mask_list[0] = &mask_rout_param;
        
        // 逐行绘制
        for(h = 0; h < rout; h++) {
            // 应用遮罩到 mask_buf
            blend_dsc.mask_res = lv_draw_sw_mask_apply(mask_list, mask_buf, ...);
            lv_draw_sw_blend(t, &blend_dsc);
        }
    }
}
```

## 5. 渐变系统

### 5.1 渐变类型

```c
typedef enum {
    LV_GRAD_DIR_NONE,    // 无渐变
    LV_GRAD_DIR_HOR,     // 水平渐变
    LV_GRAD_DIR_VER,     // 垂直渐变
    LV_GRAD_DIR_LINEAR,  // 线性渐变 (任意方向)
    LV_GRAD_DIR_RADIAL,  // 径向渐变
    LV_GRAD_DIR_CONICAL, // 锥形渐变
} lv_grad_dir_t;
```

### 5.2 渐变计算

```c
typedef struct {
    lv_color_t * color_map;  // 颜色查找表
    lv_opa_t * opa_map;      // 透明度查找表
    uint32_t size;           // 表大小
} lv_draw_sw_grad_calc_t;

// 预计算渐变颜色
lv_draw_sw_grad_calc_t * lv_draw_sw_grad_get(const lv_grad_dsc_t * gradient,
                                              int32_t w, int32_t h)
{
    // 根据渐变参数预计算颜色值
    // 存储到 color_map 和 opa_map
}

// 计算渐变颜色
void lv_draw_sw_grad_color_calculate(const lv_grad_dsc_t * dsc, int32_t range,
                                     int32_t frac, lv_color_t * color_out, lv_opa_t * opa_out)
{
    // 在渐变停止点之间插值
}
```

### 5.3 渐变在填充中的应用

```c
// 水平渐变：每行使用相同的颜色映射
if(grad_dir == LV_GRAD_DIR_HOR) {
    blend_dsc.src_buf = grad->color_map + clipped_coords.x1 - bg_coords.x1;
    blend_dsc.src_color_format = LV_COLOR_FORMAT_RGB888;
}

// 垂直渐变：每行使用不同的颜色
if(grad_dir == LV_GRAD_DIR_VER) {
    for(h = h_start; h <= h_end; h++) {
        blend_dsc.color = grad->color_map[h - bg_coords.y1];
        blend_dsc.opa = grad->opa_map[h - bg_coords.y1];
        lv_draw_sw_blend(t, &blend_dsc);
    }
}
```

## 6. 图元绘制实现

### 6.1 线段绘制

```c
void lv_draw_sw_line(lv_draw_task_t * t, const lv_draw_line_dsc_t * dsc)
{
    // 1. 水平线
    if(dsc->p1.y == dsc->p2.y) {
        draw_line_hor(t, dsc);
    }
    // 2. 垂直线
    else if(dsc->p1.x == dsc->p2.x) {
        draw_line_ver(t, dsc);
    }
    // 3. 斜线
    else {
        draw_line_skew(t, dsc);
    }
    
    // 4. 绘制端点圆角
    if(dsc->round_end || dsc->round_start) {
        // 使用圆形填充
    }
}
```

**斜线绘制**:
```c
static void draw_line_skew(lv_draw_task_t * t, const lv_draw_line_dsc_t * dsc)
{
    // 使用线段遮罩定义线段边界
    lv_draw_sw_mask_line_points_init(&mask_left_param, ...);
    lv_draw_sw_mask_line_points_init(&mask_right_param, ...);
    
    // 宽度校正 (根据斜率)
    int32_t wcorr_i = (LV_ABS(ydiff) << 5) / LV_ABS(xdiff);
    w = (w * wcorr[wcorr_i] + 63) >> 7;
    
    // 逐行应用遮罩并混合
    for(h = blend_area.y1; h <= y2; h++) {
        blend_dsc.mask_res = lv_draw_sw_mask_apply(masks, mask_buf, ...);
        lv_draw_sw_blend(t, &blend_dsc);
    }
}
```

### 6.2 圆弧绘制

```c
void lv_draw_sw_arc(lv_draw_task_t * t, const lv_draw_arc_dsc_t * dsc, const lv_area_t * coords)
{
    // 1. 完整圆环：使用边框绘制
    if(dsc->start_angle + 360 == dsc->end_angle) {
        lv_draw_sw_border(t, &cir_dsc, &area_out);
        return;
    }
    
    // 2. 创建遮罩
    // 角度遮罩：限制弧度范围
    lv_draw_sw_mask_angle_init(&mask_angle_param, center.x, center.y, start_angle, end_angle);
    // 外圆遮罩
    lv_draw_sw_mask_radius_init(&mask_out_param, &area_out, LV_RADIUS_CIRCLE, false);
    // 内圆遮罩
    lv_draw_sw_mask_radius_init(&mask_in_param, &area_in, LV_RADIUS_CIRCLE, true);
    
    // 3. 逐行绘制
    for(h = 0; h < blend_h; h++) {
        blend_dsc.mask_res = lv_draw_sw_mask_apply(mask_list, mask_buf, ...);
        lv_draw_sw_blend(t, &blend_dsc);
    }
}
```

### 6.3 图像渲染

```c
void lv_draw_sw_image(lv_draw_task_t * t, const lv_draw_image_dsc_t * draw_dsc,
                      const lv_area_t * coords)
{
    // 1. 解码图像
    lv_image_decoder_open(&decoder_dsc, draw_dsc->src, NULL);
    
    // 2. 检查是否需要变换
    if(needs_transform(draw_dsc)) {
        transform_and_recolor(t, draw_dsc, &decoder_dsc, ...);
    }
    // 3. 检查是否需要圆角
    else if(draw_dsc->radius > 0) {
        radius_only(t, draw_dsc, &decoder_dsc, ...);
    }
    // 4. 检查是否需要重新着色
    else if(draw_dsc->recolor_opa > LV_OPA_TRANSP) {
        recolor_only(t, draw_dsc, &decoder_dsc, ...);
    }
    // 5. 简单混合
    else {
        img_draw_core(t, draw_dsc, &decoder_dsc, ...);
    }
}
```

### 6.4 图像变换

```c
void lv_draw_sw_transform(const lv_area_t * dest_area, const void * src_buf,
                          int32_t src_w, int32_t src_h, int32_t src_stride,
                          const lv_draw_image_dsc_t * draw_dsc, ...)
{
    // 支持的变换：
    // - 旋转 (rotation)
    // - 缩放 (scale_x, scale_y)
    // - 倾斜 (skew_x, skew_y) - 软件渲染不支持
    
    // 变换矩阵计算
    // 逐像素反向映射
    for(y = 0; y < dest_h; y++) {
        for(x = 0; x < dest_w; x++) {
            // 计算源图像坐标
            src_x = transform_x(x, y, matrix);
            src_y = transform_y(x, y, matrix);
            
            // 双线性插值采样
            dest_buf[y * dest_stride + x] = sample_bilinear(src_buf, src_x, src_y);
        }
    }
}
```

## 7. 性能优化

### 7.1 多线程渲染

```c
// 配置项
#define LV_DRAW_SW_DRAW_UNIT_CNT  2  // 渲染线程数量

// 线程结构
typedef struct {
    lv_draw_task_t * task_act;  // 当前任务
    lv_thread_t thread;         // 线程句柄
    lv_thread_sync_t sync;      // 同步对象
    uint32_t idx;               // 线程索引
    volatile bool inited;       // 是否已初始化
    volatile bool exit_status;  // 退出标志
} lv_draw_sw_thread_dsc_t;

// 渲染线程入口
static void render_thread_cb(void * ptr)
{
    while(1) {
        // 等待任务
        lv_thread_sync_wait(&thread_dsc->sync);
        
        // 执行绘制
        execute_drawing(thread_dsc->task_act);
        
        // 标记完成，请求新任务
        thread_dsc->task_act->state = LV_DRAW_TASK_STATE_FINISHED;
        lv_draw_dispatch_request();
    }
}
```

### 7.2 SIMD 加速

**支持的 SIMD 指令集**:
- ARM NEON (`LV_DRAW_SW_ASM_NEON`)
- ARM Helium (`LV_DRAW_SW_ASM_HELIUM`)
- RISC-V Vector (`LV_DRAW_SW_ASM_RISCV_V`)
- 自定义 (`LV_DRAW_SW_ASM_CUSTOM`)

**NEON 加速示例**:
```c
// 在 lv_draw_sw_blend_to_argb8888.c 中
#ifndef LV_DRAW_SW_COLOR_BLEND_TO_ARGB8888
    #define LV_DRAW_SW_COLOR_BLEND_TO_ARGB8888(...) LV_RESULT_INVALID
#endif

// NEON 实现在 blend/neon/lv_draw_sw_blend_neon_to_argb8888.c
// 如果 NEON 可用，宏会被替换为 NEON 实现
lv_result_t LV_DRAW_SW_COLOR_BLEND_TO_ARGB8888(dsc)
{
    // 使用 NEON 向量指令一次处理多个像素
    uint32x4_t color_vec = vdupq_n_u32(color32);
    for(y = 0; y < h; y++) {
        vst1q_u32(dest_buf + x, color_vec);
        // ...
    }
    return LV_RESULT_OK;
}
```

### 7.3 循环展开

```c
// 简单填充时的循环展开
for(x = 0; x < w - 15; x += 16) {
    dest_buf[x + 0] = color32;
    dest_buf[x + 1] = color32;
    // ... 16 次展开
    dest_buf[x + 15] = color32;
}
// 处理剩余像素
for(; x < w; x++) {
    dest_buf[x] = color32;
}
```

### 7.4 快速路径

```c
// 填充时的快速路径判断
void lv_draw_sw_fill(...)
{
    // 最简单情况：无圆角、无渐变
    if(dsc->radius == 0 && grad_dir == LV_GRAD_DIR_NONE) {
        blend_dsc.blend_area = &bg_coords;
        blend_dsc.opa = dsc->opa;
        lv_draw_sw_blend(t, &blend_dsc);
        return;  // 快速返回
    }
    
    // 复杂情况：需要遮罩和渐变处理
    // ...
}
```

## 8. 自定义扩展

### 8.1 注册自定义 Blend Handler

```c
typedef struct {
    lv_color_format_t dest_cf;      // 目标颜色格式
    lv_draw_sw_blend_handler_t handler; // 处理函数
} lv_draw_sw_custom_blend_handler_t;

// 注册
lv_draw_sw_custom_blend_handler_t handler = {
    .dest_cf = LV_COLOR_FORMAT_CUSTOM,
    .handler = my_custom_blend_func,
};
lv_draw_sw_register_blend_handler(&handler);

// 自定义处理函数
void my_custom_blend_func(lv_draw_task_t * t, const lv_draw_sw_blend_dsc_t * dsc)
{
    // 自定义混合逻辑
}
```

### 8.2 自定义 SIMD 加速

```c
// 配置
#define LV_USE_DRAW_SW_ASM LV_DRAW_SW_ASM_CUSTOM
#define LV_DRAW_SW_ASM_CUSTOM_INCLUDE "my_simd.h"

// 在 my_simd.h 中定义加速宏
#define LV_DRAW_SW_COLOR_BLEND_TO_ARGB8888(dsc) my_color_blend_argb8888(dsc)
```

## 9. 配置选项

```c
// 基本配置
#define LV_USE_DRAW_SW              1   // 启用软件渲染
#define LV_DRAW_SW_DRAW_UNIT_CNT    1   // 渲染单元数量
#define LV_DRAW_SW_COMPLEX          1   // 启用复杂绘制 (遮罩等)

// 颜色格式支持
#define LV_DRAW_SW_SUPPORT_RGB565           1
#define LV_DRAW_SW_SUPPORT_RGB565_SWAPPED   1
#define LV_DRAW_SW_SUPPORT_RGB888           1
#define LV_DRAW_SW_SUPPORT_XRGB8888         1
#define LV_DRAW_SW_SUPPORT_ARGB8888         1
#define LV_DRAW_SW_SUPPORT_ARGB8888_PREMULTIPLIED 1
#define LV_DRAW_SW_SUPPORT_L8               1
#define LV_DRAW_SW_SUPPORT_A8               1
#define LV_DRAW_SW_SUPPORT_AL88             1
#define LV_DRAW_SW_SUPPORT_I1               1

// SIMD 加速
#define LV_USE_DRAW_SW_ASM          LV_DRAW_SW_ASM_NONE  // 或 NEON/HELIUM/CUSTOM

// 渐变
#define LV_USE_DRAW_SW_COMPLEX_GRADIENTS 1  // 复杂渐变

// 阴影缓存
#define LV_DRAW_SW_SHADOW_CACHE_SIZE 0  // 阴影缓存大小
```

## 10. 关键文件路径

| 功能 | 文件路径 |
|------|----------|
| Draw Unit 入口 | `src/draw/sw/lv_draw_sw.c` |
| Blend 核心 | `src/draw/sw/blend/lv_draw_sw_blend.c` |
| ARGB8888 Blend | `src/draw/sw/blend/lv_draw_sw_blend_to_argb8888.c` |
| RGB565 Blend | `src/draw/sw/blend/lv_draw_sw_blend_to_rgb565.c` |
| 遮罩系统 | `src/draw/sw/lv_draw_sw_mask.c` |
| 渐变系统 | `src/draw/sw/lv_draw_sw_grad.c` |
| 填充 | `src/draw/sw/lv_draw_sw_fill.c` |
| 线段 | `src/draw/sw/lv_draw_sw_line.c` |
| 圆弧 | `src/draw/sw/lv_draw_sw_arc.c` |
| 图像 | `src/draw/sw/lv_draw_sw_img.c` |
| NEON 优化 | `src/draw/sw/blend/neon/` |

## 11. 总结

LVGL 软件渲染后端是一个高度优化的纯 CPU 渲染实现：

1. **模块化设计**：每种图元有独立的实现文件，便于维护和扩展
2. **灵活的 Blend 系统**：支持多种颜色格式，可注册自定义处理器
3. **强大的遮罩系统**：支持圆角、角度、渐变等多种遮罩类型
4. **丰富的渐变支持**：水平、垂直、线性、径向、锥形渐变
5. **多线程支持**：可配置多个渲染线程并行处理
6. **SIMD 加速**：支持 ARM NEON、Helium、RISC-V 向量扩展
7. **可扩展性**：支持自定义颜色格式和 SIMD 加速

这种设计使得 LVGL 能够在没有 GPU 的嵌入式设备上高效运行，同时保持良好的代码结构和可维护性。
