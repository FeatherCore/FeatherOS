# FeatherRender 2.5D 混合渲染引擎实现文档

## 1. 项目概述

### 1.1 设计目标

FeatherRender 是一个面向嵌入式和轻量级设备的 2.5D 混合渲染引擎，结合 HoneyGUI 的 3D 渲染能力和 LVGL 的 UI 绘制架构，实现以下目标：

- **2.5D 伪 3D 渲染**：支持斜 45° 视角、卡牌翻转、像素风 3D 效果
- **流畅的 3D UI 转场**：支持 UI 元素的 3D 变换和过渡动画
- **GPU/CPU 混合调度**：最大化 GPU 利用率，减少 Draw Call
- **游戏级渲染能力**：支持骨骼动画、粒子效果、物理模拟
- **嵌入式友好**：低内存占用，高效能渲染

### 1.2 目标应用场景

| 场景类型 | 描述 | 技术需求 |
|---------|------|---------|
| **卡牌游戏** | 卡牌翻转、抽卡动画、卡组展示 | 2.5D 变换、深度排序 |
| **斜 45° 策略** | 等角视角地图、单位移动、建筑放置 | 等角投影、层级管理 |
| **像素风 3D** | 复古风格的 3D 场景和角色 | 低多边形、像素纹理 |
| **3D UI 转场** | 界面切换、弹窗动画、菜单效果 | 3D 变换、混合渲染 |
| **2.5D 物理** | 骨骼动画、布料模拟、粒子效果 | 物理引擎、顶点变形 |

### 1.3 核心设计原则

```
┌─────────────────────────────────────────────────────────────────┐
│                     FeatherRender 核心原则                       │
├─────────────────────────────────────────────────────────────────┤
│ 1. GPU 优先：能批量渲染的，无论多小都给 GPU                       │
│ 2. 2.5D 核心：以 2D 为基础，3D 为增强，避免纯 3D 开销             │
│ 3. 数据驱动：扁平 ECS 架构，SOA 布局，缓存友好                   │
│ 4. 延迟提交：批量收集，统一提交，减少状态切换                     │
│ 5. 平台抽象：统一渲染接口，支持多后端 (OpenGL ES/Software)       │
│ 6. 兼容优先：Adapter Pattern 保持 LVGL API 兼容                  │
└─────────────────────────────────────────────────────────────────┘
```

### 1.4 架构选型决策

#### 1.4.1 为什么选择扁平 ECS + 关系组件？

经过对 Bevy、LVGL、HoneyGUI、TouchGFX 的深度分析，我们选择了**扁平 ECS（Entity-Component-System）+ 关系组件**架构：

| 架构模式 | 缓存效率 | 批量能力 | 并行性 | 适用场景 |
|---------|---------|---------|--------|---------|
| **对象树（LVGL）** | 低 | 弱 | 差 | 简单 UI |
| **MVP（TouchGFX）** | 中 | 中 | 中 | 业务逻辑复杂 |
| **双世界 ECS（Bevy）** | 极高 | 极强 | 极强 | 高性能游戏 |
| **扁平 ECS（推荐）** | 极高 | 极强 | 强 | **嵌入式 2.5D** |

**核心优势：**
- ✅ **SOA 布局**：组件连续存储，缓存命中率 95%+
- ✅ **批量友好**：天然支持 GPU 批量渲染
- ✅ **并行处理**：无树形锁竞争
- ✅ **内存效率**：大规模场景内存占用减少 25%
- ✅ **兼容层**：通过 Adapter Pattern 保持 API 兼容

## 2. 架构设计

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           应用层 (Application)                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
│  │   UI 组件    │  │  游戏场景    │  │  动画系统    │  │   物理引擎       │ │
│  │  (Widgets)   │  │  (Scene)     │  │ (Animation)  │  │   (Physics)     │ │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └────────┬────────┘ │
└─────────┼────────────────┼────────────────┼──────────────────┼──────────┘
          │                │                │                  │
          └────────────────┴────────────────┴──────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      场景图与实体管理层 (Scene Graph)                      │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  FeatherEntity (实体)                                            │   │
│  │  ├── Transform2D/Transform3D (变换组件)                          │   │
│  │  ├── Render2D/Render3D (渲染组件)                                │   │
│  │  ├── Skeleton (骨骼组件)                                         │   │
│  │  └── Physics (物理组件)                                          │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      渲染命令生成层 (Render Command)                       │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────┐  │
│  │  2D 命令生成器   │  │  2.5D 命令生成器 │  │     3D 命令生成器        │  │
│  │  (UI/精灵)      │  │  (等角/卡牌)     │  │  (模型/粒子)             │  │
│  └────────┬────────┘  └────────┬────────┘  └────────────┬────────────┘  │
│           │                    │                        │               │
│           └────────────────────┼────────────────────────┘               │
│                                ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  RenderCommand (统一渲染命令)                                     │   │
│  │  ├── 顶点数据 (positions/uvs/colors)                             │   │
│  │  ├── 纹理句柄                                                    │   │
│  │  ├── 变换矩阵 (2D/3D)                                            │   │
│  │  ├── 混合模式                                                    │   │
│  │  └── 渲染层级 (Layer/Z-Order)                                    │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      GPU/CPU 混合调度层 (Hybrid Scheduler)                 │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  任务依赖图 (Task Dependency Graph)                              │   │
│  │  ├── 自动检测区域依赖                                            │   │
│  │  ├── 拓扑排序生成执行组                                          │   │
│  │  └── 组内并行/组间串行                                           │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  GPU 任务收集器 (GPU Task Collector)                             │   │
│  │  ├── 兼容性检查 (纹理/混合模式/变换)                              │   │
│  │  ├── 批量构建 (Batch Building)                                   │   │
│  │  └── 延迟提交策略                                                │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  执行组调度 (Execution Group Scheduler)                          │   │
│  │  ├── 组 1: 背景层 (GPU 批量)                                     │   │
│  │  ├── 组 2: 3D 场景 (GPU)                                         │   │
│  │  ├── 组 3: 2.5D 元素 (GPU 批量)                                  │   │
│  │  ├── 组 4: UI 层 (GPU 批量)                                      │   │
│  │  └── 组 5: 特效层 (GPU/CPU 混合)                                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    ▼                               ▼
┌─────────────────────────────────┐  ┌──────────────────────────────────┐
│      GPU 渲染后端 (GPU Backend)  │  │      CPU 渲染后端 (CPU Backend)   │
│  ┌─────────────────────────┐   │  │  ┌──────────────────────────────┐ │
│  │  OpenGL ES 2.0/3.0      │   │  │  │  Software Rasterizer         │ │
│  │  - 批量纹理渲染          │   │  │  │  - 2D 矩形/图像绘制           │ │
│  │  - 顶点着色器 (2D/3D)    │   │  │  │  - 简单混合操作               │ │
│  │  - 片段着色器            │   │  │  │  - 备用渲染路径               │ │
│  │  - VBO/IBO 批量提交      │   │  │  └──────────────────────────────┘ │
│  └─────────────────────────┘   │  └──────────────────────────────────┘
└─────────────────────────────────┘
```

### 2.2 核心模块职责

| 模块 | 职责 | 参考实现 |
|------|------|----------|
| **Entity Component System** | 实体管理、组件存储、系统调度 | Bevy ECS + LVGL 对象系统 |
| **Scene Graph** | 场景层级、变换传播、可见性剔除 | Cocos2d-x Node 树 |
| **Render Command Generator** | 生成统一渲染命令 | Spine RenderCommand |
| **Hybrid Scheduler** | GPU/CPU 任务分配、批量管理 | 参考优化文档 |
| **GPU Backend** | OpenGL ES 批量渲染 | HoneyGUI Lite3D + LVGL OpenGLES |
| **CPU Backend** | 软件光栅化 | LVGL Software Renderer |
| **2.5D Renderer** | 等角投影、深度排序、层级管理 | 自定义实现 |
| **Animation System** | 骨骼动画、关键帧插值 | Spine Runtimes |
| **Physics System** | 2.5D 物理模拟 | 轻量级物理引擎 |

## 3. 核心数据结构

### 3.1 实体与组件系统

```c
/**
 * @brief FeatherRender 实体 ID
 */
typedef uint32_t fre_entity_id_t;
#define FRE_INVALID_ENTITY 0

/**
 * @brief 组件类型枚举
 */
typedef enum {
    FRE_COMP_TRANSFORM_2D = 0,   // 2D 变换
    FRE_COMP_TRANSFORM_3D,       // 3D 变换
    FRE_COMP_RENDER_2D,          // 2D 渲染
    FRE_COMP_RENDER_25D,         // 2.5D 渲染
    FRE_COMP_RENDER_3D,          // 3D 渲染
    FRE_COMP_SKELETON,           // 骨骼动画
    FRE_COMP_PHYSICS,            // 物理组件
    FRE_COMP_PARTICLE,           // 粒子系统
    FRE_COMP_COUNT
} fre_component_type_t;

/**
 * @brief 2D 变换组件
 */
typedef struct {
    float x, y;                  // 位置
    float rotation;              // 旋转 (角度)
    float scale_x, scale_y;      // 缩放
    float anchor_x, anchor_y;    // 锚点 (0-1)
    float skew_x, skew_y;        // 倾斜
    fre_entity_id_t parent;      // 父实体 (层级)
} fre_transform_2d_t;

/**
 * @brief 3D 变换组件 (用于 2.5D/3D)
 */
typedef struct {
    float position[3];           // 位置 (x, y, z)
    float rotation[3];           // 旋转 (欧拉角)
    float scale[3];              // 缩放
    float anchor[3];             // 锚点
    float matrix[16];            // 变换矩阵 (缓存)
    fre_entity_id_t parent;      // 父实体
} fre_transform_3d_t;

/**
 * @brief 渲染类型
 */
typedef enum {
    FRE_RENDER_NONE = 0,
    FRE_RENDER_RECT,             // 矩形
    FRE_RENDER_IMAGE,            // 图像
    FRE_RENDER_TEXT,             // 文本
    FRE_RENDER_25D_SPRITE,       // 2.5D 精灵
    FRE_RENDER_25D_ISOMETRIC,    // 等角块
    FRE_RENDER_25D_CARD,         // 卡牌
    FRE_RENDER_3D_MESH,          // 3D 网格
    FRE_RENDER_SKELETON,         // 骨骼动画
    FRE_RENDER_PARTICLE,         // 粒子
} fre_render_type_t;

/**
 * @brief 2D 渲染组件
 */
typedef struct {
    fre_render_type_t type;
    uint32_t layer;              // 渲染层级
    float z_order;               // Z 序 (用于排序)
    
    // 视觉属性
    fre_color_t color;           // 颜色/色调
    float opacity;               // 不透明度
    fre_blend_mode_t blend_mode; // 混合模式
    
    // 纹理
    fre_texture_handle_t texture;
    fre_rect_t src_rect;         // 源矩形 (纹理坐标)
    
    // 变换
    bool use_3d_transform;       // 是否使用 3D 变换
    float transform_3d[16];      // 可选 3D 变换矩阵
} fre_render_2d_t;

/**
 * @brief 2.5D 渲染组件
 */
typedef struct {
    fre_render_type_t type;      // 25D_SPRITE / 25D_ISOMETRIC / 25D_CARD
    
    // 2.5D 特定属性
    float depth;                 // 深度值 (用于排序)
    float tilt_x, tilt_y;        // 倾斜角度 (卡牌翻转)
    bool is_isometric;           // 是否等角投影
    
    // 等角投影参数
    float iso_tile_width;        // 等角块宽度
    float iso_tile_height;       // 等角块高度
    
    // 纹理数组 (多面)
    uint8_t face_count;
    fre_texture_handle_t faces[6]; // 最多 6 个面
    
    // 光照
    float light_intensity;
    fre_color_t light_color;
} fre_render_25d_t;
```

### 3.2 渲染命令系统

```c
/**
 * @brief 统一渲染命令
 * 参考 Spine RenderCommand 设计
 */
typedef struct fre_render_command {
    // 命令类型
    fre_render_type_t type;
    
    // 顶点数据 (动态分配)
    float *positions;            // 位置 (x, y, z) * num_vertices
    float *uvs;                  // UV (u, v) * num_vertices
    uint32_t *colors;            // 颜色 (RGBA) * num_vertices
    uint32_t *dark_colors;       // 暗部颜色 (双色调)
    int32_t num_vertices;
    
    // 索引数据
    uint16_t *indices;
    int32_t num_indices;
    
    // 渲染状态
    fre_texture_handle_t texture;
    fre_blend_mode_t blend_mode;
    uint32_t layer;
    float z_order;
    
    // 变换
    float transform[16];         // 模型变换矩阵
    bool has_transform;
    
    // 裁剪
    fre_rect_t clip_rect;
    bool has_clip;
    
    // 链表指针
    struct fre_render_command *next;
} fre_render_command_t;

/**
 * @brief 渲染批次
 * 相同状态的命令合并为一个批次
 */
typedef struct {
    fre_texture_handle_t texture;
    fre_blend_mode_t blend_mode;
    uint32_t layer;
    
    // 合并后的顶点数据
    float *positions;
    float *uvs;
    uint32_t *colors;
    uint16_t *indices;
    int32_t num_vertices;
    int32_t num_indices;
    int32_t capacity;
    
    // 变换矩阵数组 (instancing)
    float *transforms;
    int32_t num_instances;
} fre_render_batch_t;
```

### 3.3 GPU/CPU 调度数据结构

```c
/**
 * @brief 任务依赖关系
 */
typedef struct {
    fre_entity_id_t task_id;
    fre_entity_id_t depends_on;
    fre_dep_type_t type;         // 区域依赖/数据依赖
} fre_task_dependency_t;

/**
 * @brief 执行组
 * 组内任务无依赖，可并行/批量执行
 */
typedef struct {
    uint32_t group_id;
    fre_entity_id_t *tasks;
    uint32_t task_count;
    uint32_t capacity;
    
    // 执行统计
    bool is_gpu_batch;           // 是否 GPU 批量
    uint32_t estimated_cost;     // 预估开销
} fre_execution_group_t;

/**
 * @brief GPU 任务收集器
 */
typedef struct {
    fre_render_command_t *commands;
    uint32_t num_commands;
    uint32_t capacity;
    
    // 当前批次
    fre_render_batch_t current_batch;
    
    // 提交阈值
    uint32_t batch_size_threshold;
    uint32_t wait_time_ms;
    uint32_t last_submit_time;
    
    // 统计
    uint32_t total_batches;
    uint32_t total_vertices;
} fre_gpu_collector_t;

/**
 * @brief 混合调度器
 */
typedef struct {
    // 依赖图
    fre_task_dependency_t *dependencies;
    uint32_t num_dependencies;
    
    // 执行组
    fre_execution_group_t *groups;
    uint32_t num_groups;
    
    // GPU 收集器
    fre_gpu_collector_t gpu_collector;
    
    // CPU 任务队列
    fre_render_command_t *cpu_queue;
    uint32_t cpu_queue_size;
    
    // 状态
    uint32_t current_group;
    bool is_initialized;
} fre_hybrid_scheduler_t;
```

## 4. 核心算法实现

### 4.1 2.5D 等角投影

```c
/**
 * @brief 将 3D 世界坐标转换为 2D 屏幕坐标 (等角投影)
 * 
 * 等角投影公式:
 * screen_x = (world_x - world_y) * tile_width / 2
 * screen_y = (world_x + world_y) * tile_height / 2 - world_z * tile_height
 */
static inline void fre_iso_world_to_screen(
    float world_x, float world_y, float world_z,
    float tile_width, float tile_height,
    float *screen_x, float *screen_y
) {
    *screen_x = (world_x - world_y) * tile_width * 0.5f;
    *screen_y = (world_x + world_y) * tile_height * 0.5f - world_z * tile_height;
}

/**
 * @brief 2.5D 深度排序
 * 用于确定绘制顺序 (画家算法)
 */
int fre_25d_depth_compare(const void *a, const void *b) {
    const fre_render_command_t *cmd_a = a;
    const fre_render_command_t *cmd_b = b;
    
    // 首先按层排序
    if (cmd_a->layer != cmd_b->layer) {
        return (int)cmd_a->layer - (int)cmd_b->layer;
    }
    
    // 同层按 Z-Order 排序 (从远到近)
    if (cmd_a->z_order < cmd_b->z_order) return -1;
    if (cmd_a->z_order > cmd_b->z_order) return 1;
    
    return 0;
}

/**
 * @brief 生成 2.5D 等角块顶点
 */
void fre_25d_generate_isometric_block(
    fre_render_25d_t *comp,
    fre_transform_3d_t *transform,
    fre_render_command_t *out_cmd
) {
    float w = comp->iso_tile_width * 0.5f;
    float h = comp->iso_tile_height * 0.5f;
    
    // 等角块的 4 个顶点 (菱形)
    // 顺序: 上、右、下、左
    float vertices[4][3] = {
        {0, -h, 0},      // 上
        {w, 0, 0},       // 右
        {0, h, 0},       // 下
        {-w, 0, 0},      // 左
    };
    
    // 应用变换并填充命令
    for (int i = 0; i < 4; i++) {
        fre_vec3_transform(&vertices[i][0], transform->matrix, &out_cmd->positions[i * 3]);
    }
    
    // UV 坐标
    float uvs[4][2] = {
        {0.5f, 0.0f},
        {1.0f, 0.5f},
        {0.5f, 1.0f},
        {0.0f, 0.5f},
    };
    memcpy(out_cmd->uvs, uvs, sizeof(uvs));
    
    // 索引 (2 个三角形)
    uint16_t indices[6] = {0, 1, 2, 0, 2, 3};
    memcpy(out_cmd->indices, indices, sizeof(indices));
    
    out_cmd->num_vertices = 4;
    out_cmd->num_indices = 6;
}
```

### 4.2 卡牌翻转效果

```c
/**
 * @brief 生成卡牌翻转顶点
 * 实现 3D 翻转效果，但使用 2D 投影
 */
void fre_25d_generate_card(
    fre_render_25d_t *comp,
    fre_transform_3d_t *transform,
    fre_render_command_t *out_cmd
) {
    float w = comp->width * 0.5f;
    float h = comp->height * 0.5f;
    
    // 基础矩形顶点
    float vertices[4][3] = {
        {-w, -h, 0},     // 左上
        {w, -h, 0},      // 右上
        {w, h, 0},       // 右下
        {-w, h, 0},      // 左下
    };
    
    // 应用倾斜变换 (模拟 3D 翻转)
    float tilt_x_rad = comp->tilt_x * FRE_PI / 180.0f;
    float tilt_y_rad = comp->tilt_y * FRE_PI / 180.0f;
    
    float cos_tx = cosf(tilt_x_rad);
    float sin_tx = sinf(tilt_x_rad);
    float cos_ty = cosf(tilt_y_rad);
    float sin_ty = sinf(tilt_y_rad);
    
    for (int i = 0; i < 4; i++) {
        float x = vertices[i][0];
        float y = vertices[i][1];
        float z = vertices[i][2];
        
        // 绕 X 轴旋转 (上下翻转)
        float y1 = y * cos_tx - z * sin_tx;
        float z1 = y * sin_tx + z * cos_tx;
        
        // 绕 Y 轴旋转 (左右翻转)
        float x2 = x * cos_ty + z1 * sin_ty;
        float z2 = -x * sin_ty + z1 * cos_ty;
        
        vertices[i][0] = x2;
        vertices[i][1] = y1;
        vertices[i][2] = z2;
    }
    
    // 应用世界变换
    for (int i = 0; i < 4; i++) {
        fre_vec3_transform(&vertices[i][0], transform->matrix, &out_cmd->positions[i * 3]);
    }
    
    // 根据翻转角度选择纹理 (正面/背面)
    if (fabsf(comp->tilt_y) > 90.0f) {
        out_cmd->texture = comp->back_texture;
        // 翻转 UV X 坐标
    } else {
        out_cmd->texture = comp->front_texture;
    }
    
    // 索引
    uint16_t indices[6] = {0, 1, 2, 0, 2, 3};
    memcpy(out_cmd->indices, indices, sizeof(indices));
    
    out_cmd->num_vertices = 4;
    out_cmd->num_indices = 6;
}
```

### 4.3 GPU 批量构建算法

```c
/**
 * @brief 检查两个命令是否可批量
 */
bool fre_batch_can_merge(
    const fre_render_batch_t *batch,
    const fre_render_command_t *cmd
) {
    // 检查渲染状态
    if (batch->texture.handle != cmd->texture.handle) return false;
    if (batch->blend_mode != cmd->blend_mode) return false;
    if (batch->layer != cmd->layer) return false;
    
    // 检查顶点容量
    if (batch->num_vertices + cmd->num_vertices > batch->capacity) return false;
    
    return true;
}

/**
 * @brief 将命令合并到批次
 */
void fre_batch_merge_command(
    fre_render_batch_t *batch,
    const fre_render_command_t *cmd
) {
    // 复制顶点数据
    int base_vertex = batch->num_vertices;
    
    memcpy(
        batch->positions + base_vertex * 3,
        cmd->positions,
        cmd->num_vertices * 3 * sizeof(float)
    );
    memcpy(
        batch->uvs + base_vertex * 2,
        cmd->uvs,
        cmd->num_vertices * 2 * sizeof(float)
    );
    memcpy(
        batch->colors + base_vertex,
        cmd->colors,
        cmd->num_vertices * sizeof(uint32_t)
    );
    
    // 复制索引 (需要偏移)
    int base_index = batch->num_indices;
    for (int i = 0; i < cmd->num_indices; i++) {
        batch->indices[base_index + i] = cmd->indices[i] + base_vertex;
    }
    
    batch->num_vertices += cmd->num_vertices;
    batch->num_indices += cmd->num_indices;
}

/**
 * @brief GPU 任务收集器 - 添加命令
 */
void fre_gpu_collector_add_command(
    fre_gpu_collector_t *collector,
    fre_render_command_t *cmd
) {
    // 检查是否可合并到当前批次
    if (fre_batch_can_merge(&collector->current_batch, cmd)) {
        fre_batch_merge_command(&collector->current_batch, cmd);
    } else {
        // 提交当前批次
        fre_gpu_submit_batch(collector);
        
        // 开始新批次
        fre_batch_init(&collector->current_batch, cmd);
    }
    
    // 检查是否达到提交阈值
    uint32_t current_time = fre_get_time_ms();
    if (collector->current_batch.num_vertices >= collector->batch_size_threshold ||
        current_time - collector->last_submit_time >= collector->wait_time_ms) {
        fre_gpu_submit_batch(collector);
    }
}
```

### 4.4 混合调度器核心逻辑

```c
/**
 * @brief 构建执行组
 * 使用 Kahn 拓扑排序算法
 */
void fre_scheduler_build_groups(fre_hybrid_scheduler_t *scheduler) {
    // 计算入度
    uint32_t *in_degree = calloc(scheduler->num_tasks, sizeof(uint32_t));
    for (uint32_t i = 0; i < scheduler->num_dependencies; i++) {
        fre_task_dependency_t *dep = &scheduler->dependencies[i];
        in_degree[dep->task_id]++;
    }
    
    // 初始化队列 (入度为 0 的任务)
    fre_queue_t queue;
    for (uint32_t i = 0; i < scheduler->num_tasks; i++) {
        if (in_degree[i] == 0) {
            fre_queue_push(&queue, i);
        }
    }
    
    // 拓扑排序并分组
    uint32_t current_group = 0;
    scheduler->groups[current_group].group_id = current_group;
    
    while (!fre_queue_empty(&queue)) {
        fre_entity_id_t task_id = fre_queue_pop(&queue);
        
        // 添加到当前组
        fre_group_add_task(&scheduler->groups[current_group], task_id);
        
        // 减少依赖任务的入度
        for (uint32_t i = 0; i < scheduler->num_dependencies; i++) {
            fre_task_dependency_t *dep = &scheduler->dependencies[i];
            if (dep->depends_on == task_id) {
                in_degree[dep->task_id]--;
                if (in_degree[dep->task_id] == 0) {
                    fre_queue_push(&queue, dep->task_id);
                }
            }
        }
        
        // 检查组是否可提交 (简单策略：每 N 个任务一组)
        if (scheduler->groups[current_group].task_count >= FRE_GROUP_SIZE) {
            current_group++;
            scheduler->groups[current_group].group_id = current_group;
        }
    }
    
    scheduler->num_groups = current_group + 1;
    free(in_degree);
}

/**
 * @brief 执行调度
 */
void fre_scheduler_execute(fre_hybrid_scheduler_t *scheduler) {
    for (uint32_t g = 0; g < scheduler->num_groups; g++) {
        fre_execution_group_t *group = &scheduler->groups[g];
        
        // 分析组内任务
        uint32_t gpu_tasks = 0;
        uint32_t cpu_tasks = 0;
        
        for (uint32_t i = 0; i < group->task_count; i++) {
            fre_entity_id_t task_id = group->tasks[i];
            if (fre_should_use_gpu(task_id)) {
                gpu_tasks++;
            } else {
                cpu_tasks++;
            }
        }
        
        // 决策：是否使用 GPU 批量
        if (gpu_tasks > FRE_GPU_BATCH_THRESHOLD) {
            // 收集 GPU 任务
            for (uint32_t i = 0; i < group->task_count; i++) {
                fre_entity_id_t task_id = group->tasks[i];
                if (fre_should_use_gpu(task_id)) {
                    fre_render_command_t *cmd = fre_generate_command(task_id);
                    fre_gpu_collector_add_command(&scheduler->gpu_collector, cmd);
                }
            }
            // 提交 GPU 批次
            fre_gpu_submit_batch(&scheduler->gpu_collector);
        } else {
            // 单独处理每个任务
            for (uint32_t i = 0; i < group->task_count; i++) {
                fre_entity_id_t task_id = group->tasks[i];
                fre_render_command_t *cmd = fre_generate_command(task_id);
                
                if (fre_should_use_gpu(task_id)) {
                    fre_gpu_render_immediate(cmd);
                } else {
                    fre_cpu_render(cmd);
                }
            }
        }
        
        // 等待组完成
        fre_wait_group_complete(group);
    }
}
```

## 5. OpenGL ES 渲染后端

### 5.1 着色器设计

```glsl
// 顶点着色器 (支持 2D/2.5D/3D)
attribute vec3 a_position;
attribute vec2 a_uv;
attribute vec4 a_color;
attribute vec4 a_dark_color;  // 双色调

uniform mat4 u_projection;
uniform mat4 u_view;
uniform mat4 u_model;
uniform bool u_use_3d;

varying vec2 v_uv;
varying vec4 v_color;
varying vec4 v_dark_color;

void main() {
    vec4 pos;
    if (u_use_3d) {
        pos = u_projection * u_view * u_model * vec4(a_position, 1.0);
    } else {
        // 2D/2.5D 正交投影
        vec4 world_pos = u_model * vec4(a_position, 1.0);
        pos = u_projection * world_pos;
    }
    
    gl_Position = pos;
    v_uv = a_uv;
    v_color = a_color;
    v_dark_color = a_dark_color;
}

// 片段着色器
precision mediump float;

varying vec2 v_uv;
varying vec4 v_color;
varying vec4 v_dark_color;

uniform sampler2D u_texture;
uniform int u_blend_mode;  // 0: normal, 1: additive, 2: multiply
uniform bool u_two_color;

void main() {
    vec4 tex_color = texture2D(u_texture, v_uv);
    
    // 双色调混合 (Spine 风格)
    vec4 final_color;
    if (u_two_color) {
        float alpha = tex_color.a * v_color.a;
        vec3 light = tex_color.rgb * v_color.rgb;
        vec3 dark = v_dark_color.rgb * (1.0 - tex_color.rgb);
        final_color = vec4(light + dark, alpha);
    } else {
        final_color = tex_color * v_color;
    }
    
    // 混合模式
    if (u_blend_mode == 1) {
        // Additive
        gl_FragColor = vec4(final_color.rgb * final_color.a, final_color.a);
    } else if (u_blend_mode == 2) {
        // Multiply
        gl_FragColor = vec4(final_color.rgb * final_color.a, final_color.a);
    } else {
        // Normal
        gl_FragColor = final_color;
    }
}
```

### 5.2 批量渲染实现

```c
/**
 * @brief OpenGL ES 批量渲染
 */
typedef struct {
    GLuint vbo;                  // 顶点缓冲区
    GLuint ibo;                  // 索引缓冲区
    GLuint vao;                  // 顶点数组对象 (ES 3.0+)
    
    // 着色器
    GLuint shader_program;
    GLint loc_projection;
    GLint loc_view;
    GLint loc_model;
    GLint loc_use_3d;
    GLint loc_texture;
    GLint loc_blend_mode;
    GLint loc_two_color;
    
    // 当前状态
    fre_texture_handle_t current_texture;
    fre_blend_mode_t current_blend;
    
    // 缓冲区容量
    uint32_t max_vertices;
    uint32_t max_indices;
} fre_gl_backend_t;

/**
 * @brief 提交批次到 GPU
 */
void fre_gl_submit_batch(fre_render_batch_t *batch) {
    fre_gl_backend_t *gl = &g_gl_backend;
    
    // 设置混合模式
    if (batch->blend_mode != gl->current_blend) {
        fre_gl_set_blend_mode(batch->blend_mode);
        gl->current_blend = batch->blend_mode;
    }
    
    // 绑定纹理
    if (batch->texture.handle != gl->current_texture.handle) {
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, batch->texture.handle);
        glUniform1i(gl->loc_texture, 0);
        gl->current_texture = batch->texture;
    }
    
    // 更新 VBO
    glBindBuffer(GL_ARRAY_BUFFER, gl->vbo);
    
    // 交错顶点数据 (位置 + UV + 颜色)
    size_t vertex_size = (3 + 2 + 4) * sizeof(float);
    size_t total_size = batch->num_vertices * vertex_size;
    
    glBufferData(GL_ARRAY_BUFFER, total_size, NULL, GL_STREAM_DRAW);
    
    // 填充数据
    float *mapped = glMapBufferRange(GL_ARRAY_BUFFER, 0, total_size, GL_MAP_WRITE_BIT);
    for (int i = 0; i < batch->num_vertices; i++) {
        // 位置
        memcpy(mapped, &batch->positions[i * 3], 3 * sizeof(float));
        mapped += 3;
        // UV
        memcpy(mapped, &batch->uvs[i * 2], 2 * sizeof(float));
        mapped += 2;
        // 颜色 (转换为 float)
        uint32_t c = batch->colors[i];
        *mapped++ = ((c >> 16) & 0xFF) / 255.0f;  // R
        *mapped++ = ((c >> 8) & 0xFF) / 255.0f;   // G
        *mapped++ = (c & 0xFF) / 255.0f;          // B
        *mapped++ = ((c >> 24) & 0xFF) / 255.0f;  // A
    }
    glUnmapBuffer(GL_ARRAY_BUFFER);
    
    // 更新 IBO
    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, gl->ibo);
    glBufferData(GL_ELEMENT_ARRAY_BUFFER, 
                 batch->num_indices * sizeof(uint16_t),
                 batch->indices, GL_STREAM_DRAW);
    
    // 设置顶点属性
    glEnableVertexAttribArray(0);  // position
    glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, vertex_size, (void*)0);
    
    glEnableVertexAttribArray(1);  // uv
    glVertexAttribPointer(1, 2, GL_FLOAT, GL_FALSE, vertex_size, (void*)(3 * sizeof(float)));
    
    glEnableVertexAttribArray(2);  // color
    glVertexAttribPointer(2, 4, GL_FLOAT, GL_FALSE, vertex_size, (void*)(5 * sizeof(float)));
    
    // 绘制
    glDrawElements(GL_TRIANGLES, batch->num_indices, GL_UNSIGNED_SHORT, 0);
    
    // 清理
    glDisableVertexAttribArray(0);
    glDisableVertexAttribArray(1);
    glDisableVertexAttribArray(2);
}
```

## 6. 骨骼动画系统

### 6.1 数据结构 (参考 Spine)

```c
/**
 * @brief 骨骼
 */
typedef struct {
    char name[64];
    float x, y, rotation, scale_x, scale_y;
    float shear_x, shear_y;
    
    // 变换矩阵 (缓存)
    float local_matrix[16];
    float world_matrix[16];
    
    // 层级
    int parent_index;
} fre_bone_t;

/**
 * @brief 插槽
 */
typedef struct {
    char name[64];
    int bone_index;
    fre_color_t color;
    
    // 当前附件
    int attachment_index;
} fre_slot_t;

/**
 * @brief 附件类型
 */
typedef enum {
    FRE_ATTACHMENT_REGION,       // 区域 (矩形)
    FRE_ATTACHMENT_MESH,         // 网格
    FRE_ATTACHMENT_BOUNDING_BOX, // 边界框
} fre_attachment_type_t;

/**
 * @brief 区域附件
 */
typedef struct {
    fre_attachment_type_t type;
    char name[64];
    
    float x, y, rotation, scale_x, scale_y;
    float width, height;
    
    // 顶点 (本地坐标)
    float vertices[4][2];        // 4 个角点
    float uvs[4][2];             // UV 坐标
    
    fre_texture_handle_t texture;
} fre_region_attachment_t;

/**
 * @brief 网格附件
 */
typedef struct {
    fre_attachment_type_t type;
    char name[64];
    
    // 顶点数据
    float *vertices;             // 本地顶点 (x, y)
    float *uvs;                  // UV 坐标
    uint16_t *indices;           // 三角形索引
    int num_vertices;
    int num_indices;
    
    // 骨骼权重 (用于蒙皮)
    int *bones;                  // 每个顶点影响的骨骼
    float *weights;              // 权重值
    
    fre_texture_handle_t texture;
} fre_mesh_attachment_t;

/**
 * @brief 骨骼动画组件
 */
typedef struct {
    fre_bone_t *bones;
    int num_bones;
    
    fre_slot_t *slots;
    int num_slots;
    
    void **attachments;          // 附件数组
    int num_attachments;
    
    // 当前皮肤
    int skin_index;
    
    // 动画状态
    float time;
    bool needs_update;
} fre_skeleton_component_t;
```

### 6.2 渲染命令生成

```c
/**
 * @brief 从骨骼生成渲染命令
 */
void fre_skeleton_generate_commands(
    fre_skeleton_component_t *skeleton,
    fre_render_command_t **out_commands,
    int *out_count
) {
    *out_count = 0;
    
    // 遍历所有插槽
    for (int i = 0; i < skeleton->num_slots; i++) {
        fre_slot_t *slot = &skeleton->slots[i];
        if (slot->attachment_index < 0) continue;
        
        void *attachment = skeleton->attachments[slot->attachment_index];
        fre_attachment_type_t type = *(fre_attachment_type_t*)attachment;
        
        fre_render_command_t *cmd = &(*out_commands)[*out_count];
        memset(cmd, 0, sizeof(fre_render_command_t));
        
        // 获取骨骼世界变换
        fre_bone_t *bone = &skeleton->bones[slot->bone_index];
        
        if (type == FRE_ATTACHMENT_REGION) {
            fre_region_attachment_t *region = attachment;
            
            // 计算世界顶点
            cmd->positions = malloc(4 * 3 * sizeof(float));
            cmd->uvs = malloc(4 * 2 * sizeof(float));
            cmd->colors = malloc(4 * sizeof(uint32_t));
            cmd->indices = malloc(6 * sizeof(uint16_t));
            
            for (int v = 0; v < 4; v++) {
                // 本地坐标 -> 世界坐标
                float local_x = region->vertices[v][0];
                float local_y = region->vertices[v][1];
                
                // 应用骨骼变换
                fre_vec2_transform(local_x, local_y, bone->world_matrix, 
                                   &cmd->positions[v * 3], &cmd->positions[v * 3 + 1]);
                cmd->positions[v * 3 + 2] = 0;  // Z = 0 (2D)
                
                // UV
                cmd->uvs[v * 2] = region->uvs[v][0];
                cmd->uvs[v * 2 + 1] = region->uvs[v][1];
                
                // 颜色
                cmd->colors[v] = slot->color.value;
            }
            
            // 索引
            uint16_t indices[6] = {0, 1, 2, 0, 2, 3};
            memcpy(cmd->indices, indices, sizeof(indices));
            
            cmd->num_vertices = 4;
            cmd->num_indices = 6;
            cmd->texture = region->texture;
            
        } else if (type == FRE_ATTACHMENT_MESH) {
            fre_mesh_attachment_t *mesh = attachment;
            
            // 类似处理，但顶点数量可变
            cmd->positions = malloc(mesh->num_vertices * 3 * sizeof(float));
            cmd->uvs = malloc(mesh->num_vertices * 2 * sizeof(float));
            cmd->colors = malloc(mesh->num_vertices * sizeof(uint32_t));
            cmd->indices = malloc(mesh->num_indices * sizeof(uint16_t));
            
            // 顶点变形计算 (CPU 端)
            for (int v = 0; v < mesh->num_vertices; v++) {
                // ... 骨骼权重计算
            }
            
            cmd->num_vertices = mesh->num_vertices;
            cmd->num_indices = mesh->num_indices;
            cmd->texture = mesh->texture;
        }
        
        (*out_count)++;
    }
}
```

## 7. 物理系统 (2.5D)

### 7.1 轻量级物理引擎

```c
/**
 * @brief 物理体类型
 */
typedef enum {
    FRE_PHYSICS_STATIC,          // 静态
    FRE_PHYSICS_DYNAMIC,         // 动态
    FRE_PHYSICS_KINEMATIC,       // 运动学
} fre_physics_body_type_t;

/**
 * @brief 物理体
 */
typedef struct {
    fre_physics_body_type_t type;
    
    // 变换
    float position[3];
    float velocity[3];
    float acceleration[3];
    float rotation;
    float angular_velocity;
    
    // 属性
    float mass;
    float friction;
    float restitution;           // 弹性
    
    // 碰撞形状
    fre_physics_shape_t shape;
    
    // 睡眠状态
    bool is_sleeping;
    float sleep_threshold;
} fre_physics_body_t;

/**
 * @brief 物理约束 (参考 Spine)
 */
typedef struct {
    fre_physics_constraint_type_t type;
    
    // 连接的物体
    int body_a;
    int body_b;
    
    // 约束参数
    float stiffness;
    float damping;
    
    // 距离约束
    float rest_length;
    
    // 角度约束
    float rest_angle;
    float min_angle;
    float max_angle;
} fre_physics_constraint_t;

/**
 * @brief 物理世界
 */
typedef struct {
    fre_physics_body_t *bodies;
    int num_bodies;
    int capacity;
    
    fre_physics_constraint_t *constraints;
    int num_constraints;
    
    // 全局设置
    float gravity[3];
    float time_step;
    int velocity_iterations;
    int position_iterations;
} fre_physics_world_t;

/**
 * @brief 更新物理世界
 */
void fre_physics_update(fre_physics_world_t *world, float delta_time) {
    // 积分
    for (int i = 0; i < world->num_bodies; i++) {
        fre_physics_body_t *body = &world->bodies[i];
        if (body->type == FRE_PHYSICS_STATIC) continue;
        
        // 应用重力
        body->acceleration[0] += world->gravity[0];
        body->acceleration[1] += world->gravity[1];
        body->acceleration[2] += world->gravity[2];
        
        // 显式欧拉积分
        body->velocity[0] += body->acceleration[0] * delta_time;
        body->velocity[1] += body->acceleration[1] * delta_time;
        body->velocity[2] += body->acceleration[2] * delta_time;
        
        body->position[0] += body->velocity[0] * delta_time;
        body->position[1] += body->velocity[1] * delta_time;
        body->position[2] += body->velocity[2] * delta_time;
        
        // 角速度
        body->rotation += body->angular_velocity * delta_time;
        
        // 重置加速度
        body->acceleration[0] = 0;
        body->acceleration[1] = 0;
        body->acceleration[2] = 0;
    }
    
    // 求解约束
    for (int iter = 0; iter < world->velocity_iterations; iter++) {
        for (int i = 0; i < world->num_constraints; i++) {
            fre_physics_solve_constraint(&world->constraints[i], world->bodies);
        }
    }
    
    // 碰撞检测与响应
    fre_physics_detect_collisions(world);
}
```

## 8. API 设计

### 8.1 初始化与配置

```c
/**
 * @brief 初始化 FeatherRender
 */
bool fre_init(const fre_config_t *config);

/**
 * @brief 关闭 FeatherRender
 */
void fre_deinit(void);

/**
 * @brief 配置结构体
 */
typedef struct {
    uint32_t screen_width;
    uint32_t screen_height;
    fre_color_format_t color_format;
    
    // GPU 设置
    bool use_gpu;
    uint32_t gpu_batch_size;
    uint32_t gpu_max_vertices;
    
    // CPU 设置
    bool use_cpu_fallback;
    uint32_t cpu_thread_count;
    
    // 内存池
    uint32_t max_entities;
    uint32_t max_components;
    uint32_t max_commands;
} fre_config_t;
```

### 8.2 实体与组件 API

```c
/**
 * @brief 创建实体
 */
fre_entity_id_t fre_entity_create(void);

/**
 * @brief 销毁实体
 */
void fre_entity_destroy(fre_entity_id_t entity);

/**
 * @brief 添加组件
 */
void *fre_component_add(fre_entity_id_t entity, fre_component_type_t type);

/**
 * @brief 获取组件
 */
void *fre_component_get(fre_entity_id_t entity, fre_component_type_t type);

/**
 * @brief 移除组件
 */
void fre_component_remove(fre_entity_id_t entity, fre_component_type_t type);

/**
 * @brief 设置父子关系
 */
void fre_entity_set_parent(fre_entity_id_t child, fre_entity_id_t parent);
```

### 8.3 渲染 API

```c
/**
 * @brief 开始一帧渲染
 */
void fre_render_begin(void);

/**
 * @brief 提交渲染命令
 */
void fre_render_submit(fre_render_command_t *command);

/**
 * @brief 结束一帧渲染并执行
 */
void fre_render_end(void);

/**
 * @brief 创建 2D 精灵
 */
fre_entity_id_t fre_sprite_create(
    fre_texture_handle_t texture,
    float x, float y,
    float width, float height
);

/**
 * @brief 创建 2.5D 等角块
 */
fre_entity_id_t fre_isometric_block_create(
    fre_texture_handle_t texture,
    float world_x, float world_y, float world_z,
    float tile_width, float tile_height
);

/**
 * @brief 创建 3D 卡牌
 */
fre_entity_id_t fre_card_create(
    fre_texture_handle_t front_texture,
    fre_texture_handle_t back_texture,
    float width, float height
);

/**
 * @brief 设置卡牌翻转
 */
void fre_card_set_tilt(fre_entity_id_t card, float tilt_x, float tilt_y);
```

### 8.4 动画 API

```c
/**
 * @brief 加载骨骼动画
 */
fre_skeleton_handle_t fre_skeleton_load(const char *json_path, const char *atlas_path);

/**
 * @brief 创建骨骼动画实体
 */
fre_entity_id_t fre_skeleton_create(fre_skeleton_handle_t skeleton);

/**
 * @brief 播放动画
 */
void fre_skeleton_play(fre_entity_id_t entity, const char *animation_name, bool loop);

/**
 * @brief 设置动画时间缩放
 */
void fre_skeleton_set_time_scale(fre_entity_id_t entity, float scale);

/**
 * @brief 更新动画
 */
void fre_skeleton_update(fre_entity_id_t entity, float delta_time);
```

## 9. 性能优化策略

### 9.1 批处理优化

| 优化策略 | 实现方式 | 预期收益 |
|---------|----------|----------|
| **纹理图集** | 将多个小图合并为大图 | 减少纹理切换 80% |
| **动态批次** | 根据状态变化自动分批次 | 最大化批次大小 |
| **实例渲染** | 相同网格使用不同变换 | 减少 Draw Call 90% |
| **延迟提交** | 收集足够命令后统一提交 | 减少 GPU 启动开销 |

### 9.2 内存优化

| 优化策略 | 实现方式 | 预期收益 |
|---------|----------|----------|
| **对象池** | 复用渲染命令和顶点缓冲区 | 减少内存分配 90% |
| **SOA 布局** | 组件数据连续存储 | 提高缓存命中率 |
| **顶点压缩** | 使用 half-float 存储位置和 UV | 减少内存 50% |
| **纹理压缩** | 使用 ETC2/ASTC 格式 | 减少显存 75% |

### 9.3 渲染优化

| 优化策略 | 实现方式 | 预期收益 |
|---------|----------|----------|
| **视锥剔除** | 只渲染可见对象 | 减少渲染对象 60% |
| **层级剔除** | 背面/遮挡剔除 | 减少片段着色 40% |
| **LOD 系统** | 根据距离使用不同精度 | 减少顶点数 50% |
| **Early-Z** | 先渲染不透明物体 | 减少 Overdraw 30% |

## 10. 开发路线图

### 阶段 1：基础架构 (4 周)

- [ ] 核心数据结构实现 (Entity, Component, Command)
- [ ] 2D 渲染系统 (矩形、图像、文本)
- [ ] OpenGL ES 后端基础
- [ ] 简单场景图实现

### 阶段 2：2.5D 渲染 (3 周)

- [ ] 等角投影系统
- [ ] 卡牌翻转效果
- [ ] 深度排序算法
- [ ] 2.5D 变换系统

### 阶段 3：混合调度器 (3 周)

- [ ] 依赖图构建
- [ ] GPU 任务收集器
- [ ] 批量构建算法
- [ ] CPU 回退机制

### 阶段 4：动画系统 (3 周)

- [ ] 骨骼动画加载
- [ ] 关键帧插值
- [ ] 附件系统
- [ ] 动画混合

### 阶段 5：物理系统 (2 周)

- [ ] 2.5D 物理世界
- [ ] 碰撞检测
- [ ] 约束求解
- [ ] 物理-渲染同步

### 阶段 6：优化与集成 (3 周)

- [ ] 性能分析与优化
- [ ] LVGL 集成
- [ ] HoneyGUI 集成
- [ ] 文档和示例

## 11. 参考资源

### 11.1 参考项目

| 项目 | 参考内容 |
|------|----------|
| **HoneyGUI** | 3D 渲染管线、Lite3D 集成 |
| **LVGL** | 绘制任务系统、多后端支持 |
| **Spine Runtimes** | 骨骼动画、渲染命令系统 |
| **Bevy** | ECS 架构、渲染图设计 |
| **Cocos2d-x** | 场景图、批处理机制 |

### 11.2 技术文档

- GPU/CPU 混合渲染优化方案 (`GPU_CPU_Rendering_Optimization.md`)
- OpenGL ES 3.0 规范
- 2.5D 游戏开发技术
- 骨骼动画算法

---

## 12. FHRE Rust v2.0 实现 (NuttX SIM 平台)

### 12.1 架构概述

FHRE (Feather Hybrid Render Engine) Rust v2.0 是基于声明式双世界架构的轻量级渲染引擎，专为 NuttX 嵌入式系统设计。

**核心特性：**
- 声明式双世界架构 (Main World + Render World)
- ECS (Entity-Component-System) 设计
- 模块化的系统调度
- 嵌入式友好的无标准库实现 (`#![no_std]`)
- NuttX SIM 平台支持 (类似 LVGL 的 framebuffer 对接)
- 文件夹/mod.rs 模块化结构

### 12.2 双世界架构

```
┌─────────────────────────────┐        ┌─────────────────────────────┐
│       Main World            │        │       Render World          │
│      (游戏逻辑世界)          │        │       (渲染世界)             │
│                             │        │                             │
│  ┌─────────────────────┐    │        │  ┌─────────────────────┐    │
│  │ Entities            │    │        │  │ RenderObjects       │    │
│  │ - Transform         │    │        │  │ - ExtractedTransform│    │
│  │ - Sprite            │    │        │  │ - ExtractedSprite   │    │
│  │ - Velocity          │    │        │  │ - z_order           │    │
│  └─────────────────────┘    │        │  └─────────────────────┘    │
│                             │        │                             │
│  ┌─────────────────────┐    │        │  ┌─────────────────────┐    │
│  │ Systems             │    │        │  │ Draw Commands       │    │
│  │ - PreUpdate         │    │        │  │ - Clear             │    │
│  │ - Update            │    │        │  │ - DrawRect          │    │
│  │ - PostUpdate        │    │        │  │ - DrawCircle        │    │
│  └─────────────────────┘    │        │  └─────────────────────┘    │
└─────────────┬───────────────┘        └─────────────┬───────────────┘
              │                                      │
              │          ExtractSchedule             │
              │         (数据提取阶段)                │
              └──────────────────────────────────────▶
```

### 12.3 NuttX SIM 平台实现细节

#### 12.3.1 IOCTL 命令码

**重要**: NuttX 使用 `_FBIOCBASE (0x2800)` 作为 framebuffer ioctl 的基础命令码，与 Linux 不同。

| 命令 | 值 | 功能 |
|------|-----|------|
| `FBIOGET_VIDEOINFO` | **0x2801** | 获取视频信息（分辨率、格式等） |
| `FBIOGET_PLANEINFO` | **0x2802** | 获取平面信息（内存地址、stride等） |
| `FBIO_UPDATE` | **0x2803** | 更新显示区域（触发 X11 刷新） |

**注意**: 早期版本错误地使用了 Linux 的 ioctl 命令码 (0x4600+)，这在 NuttX 中会导致 `ENOTTY` 错误。

#### 12.3.2 结构体定义

Rust 结构体必须与 NuttX C 结构体完全匹配：

```rust
/// Framebuffer video info structure (matches NuttX struct fb_videoinfo_s)
#[repr(C)]
struct FbVideoInfo {
    fmt: u8,        /* see FB_FMT_* */
    xres: u16,      /* Horizontal resolution */
    yres: u16,      /* Vertical resolution */
    nplanes: u8,    /* Number of color planes */
}

/// Framebuffer plane info structure (matches NuttX struct fb_planeinfo_s)
#[repr(C)]
struct FbPlaneInfo {
    fbmem: *mut u8,     /* Start of frame buffer memory */
    fblen: usize,       /* Length of frame buffer memory */
    stride: u16,        /* Length of a line in bytes */
    display: u8,        /* Display number */
    bpp: u8,            /* Bits per pixel */
    xres_virtual: u32,  /* Virtual Horizontal resolution */
    yres_virtual: u32,  /* Virtual Vertical resolution */
    xoffset: u32,       /* Offset from virtual to visible */
    yoffset: u32,       /* Offset from virtual to visible */
}
```

#### 12.3.3 关键实现要点

1. **不要使用 mmap()**: 在 NuttX SIM 平台中，`mmap()` 系统调用会直接传递给主机的 Linux 内核，导致返回 `MAP_FAILED (0xffffffffffffffff)`。应该直接使用 `FBIOGET_PLANEINFO` 返回的 `fbmem` 地址。

2. **使用 usleep() 让出 CPU**: NuttX SIM 平台的 X11 刷新依赖于 `sim_x11loop()` 在 idle 线程中运行。主循环中必须使用 `usleep()` 让出 CPU 时间，否则 X11 窗口不会更新。

3. **无限循环模式**: Demo 应该使用无限循环 (`loop {}`)，不会主动退出，除非用户手动关闭。

### 12.4 项目结构

```
FeatherOS/apps/fhre/rust/src/
├── lib.rs              # 库入口
├── app/                # App 模块
│   ├── mod.rs          # 衔接：导出 app, config
│   ├── app.rs          # App 实现
│   └── config.rs       # App 配置
├── main_world/         # Main World 模块
│   ├── mod.rs          # 衔接：导出 world, entity, component, system
│   ├── world.rs        # MainWorld 实现
│   ├── entity.rs       # Entity 实现
│   ├── component.rs    # Component 实现
│   └── system.rs       # System 实现
├── render_world/       # Render World 模块
│   ├── mod.rs          # 衔接：导出 world, command, object
│   ├── world.rs        # RenderWorld 实现
│   ├── command.rs      # RenderCommand 实现
│   └── object.rs       # RenderObject 实现
├── extract/            # Extract 模块
│   ├── mod.rs          # 衔接：导出 extract
│   └── extract.rs      # Extract 实现
├── schedule/           # Schedule 模块
│   ├── mod.rs          # 衔接：导出 schedule, label, set
│   ├── schedule.rs     # Schedule 实现
│   ├── label.rs        # Label 实现
│   └── set.rs          # SystemSet 实现
├── resources/          # Resources 模块
│   ├── mod.rs          # 衔接：导出 resources, time, config
│   ├── resources.rs    # Resources 实现
│   ├── time.rs         # Time 实现
│   └── config.rs       # Config 实现
├── renderer/           # Renderer 模块
│   ├── mod.rs          # 衔接：导出 renderer, framebuffer, target
│   ├── renderer.rs     # Renderer 实现
│   ├── framebuffer.rs  # Framebuffer 实现
│   └── target.rs       # RenderTarget 实现
├── math/               # Math 模块
│   ├── mod.rs
│   ├── vec2.rs
│   ├── vec3.rs
│   ├── color.rs
│   └── rect.rs
└── platform/           # Platform 模块
    ├── mod.rs          # 衔接：导出 sim, nuttx, default
    ├── sim.rs          # NuttX SIM 平台支持
    ├── nuttx.rs        # NuttX 平台支持
    └── default.rs      # 默认平台支持
```

### 12.5 编译和运行

```bash
# 清理并重新配置
cd /home/uan/develop/FeatherOS-code/FeatherOS/nuttx
make distclean
./tools/configure.sh -l sim:fhre

# 编译
make -j

# 运行
./nuttx
nsh> fhre_rust
```

---

**文档版本**: v1.1  
**更新日期**: 2026-04-14  
**作者**: FeatherRender Team  
**适用范围**: FeatherOS 图形引擎开发 (C + Rust)
