# Bevy 渲染世界（Render World）分析

## 1. 概述

Bevy 的渲染世界（Render World）是一个独立的 ECS 世界，负责管理所有的 GPU 渲染任务。它采用双世界架构（Main World + Render World），通过 Extract 阶段将主世界的数据同步到渲染世界，然后在渲染世界中执行高效的 GPU 渲染。

## 2. 渲染世界架构

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Bevy 渲染世界架构                                    │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                      Main World (主世界)                             │   │
│  │                                                                     │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐  │   │
│  │  │  Entity     │  │  Component  │  │  System     │  │ Resources │  │   │
│  │  │  (Game)     │  │  (Game)     │  │  (Game)     │  │ (Game)    │  │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └───────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │                                              │
│                              ▼ Extract Schedule                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                     Render World (渲染世界)                          │   │
│  │                                                                     │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐  │   │
│  │  │  Entity     │  │  Component  │  │  System     │  │ Resources │  │   │
│  │  │  (Render)   │  │  (Render)   │  │  (Render)   │  │ (Render)  │  │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └───────────┘  │   │
│  │                                                                     │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │  Render Phases (渲染阶段)                                    │   │   │
│  │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐           │   │   │
│  │  │  │Opaque3d │ │Alpha3d  │ │Shadow   │ │UI       │ ...       │   │   │
│  │  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘           │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  │                                                                     │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │  Views (视图/摄像机)                                         │   │   │
│  │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐                       │   │   │
│  │  │  │Camera 1 │ │Camera 2 │ │Shadow   │ ...                   │   │   │
│  │  │  └─────────┘ └─────────┘ └─────────┘                       │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                              │                                              │
│                              ▼ Render to GPU                                │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                      GPU (显卡)                                      │   │
│  │                                                                     │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                 │   │
│  │  │  Buffers    │  │  Textures   │  │  Pipelines  │                 │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘                 │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 3. 渲染阶段（Render Phases）

渲染阶段是 Bevy 组织绘制任务的核心机制。每个视图（摄像机、光源等）可以有多个渲染阶段。

### 3.1 渲染阶段类型

```rust
// crates/bevy_render/src/render_phase/mod.rs

/// 分箱渲染阶段 - 用于不透明物体（顺序不重要）
pub struct BinnedRenderPhase<BPI>
where
    BPI: BinnedPhaseItem,
{
    /// 可多维绘制的网格批次
    pub multidrawable_meshes: IndexMap<BPI::BatchSetKey, RenderMultidrawableBatchSet<BPI>>,
    
    /// 可批处理但不可多维绘制的网格
    pub batchable_meshes: IndexMap<(BPI::BatchSetKey, BPI::BinKey), RenderBin>,
    
    /// 不可批处理的网格（每个实体单独绘制）
    pub unbatchable_meshes: IndexMap<(BPI::BatchSetKey, BPI::BinKey), UnbatchableBinnedEntities>,
    
    /// 非网格项目（自定义绘制命令）
    pub non_mesh_items: IndexMap<(BPI::BatchSetKey, BPI::BinKey), NonMeshEntities>,
}

/// 排序渲染阶段 - 用于透明物体（需要排序）
pub struct SortedRenderPhase<P: PhaseItem> {
    /// 排序后的绘制项列表
    pub items: Vec<P>,
}
```

### 3.2 渲染阶段分类

| 阶段类型 | 用途 | 排序方式 | 示例 |
|---------|------|---------|------|
| **BinnedRenderPhase** | 不透明物体 | 不排序（按批次） | Opaque3d, Opaque2d |
| **SortedRenderPhase** | 透明物体 | 从后往前排序 | AlphaMask3d, Transparent3d |

### 3.3 标准渲染阶段

```rust
// 3D 渲染阶段
pub struct Opaque3d;      // 不透明 3D 物体
pub struct AlphaMask3d;   // Alpha 测试 3D 物体
pub struct Transparent3d; // 透明 3D 物体

// 2D 渲染阶段
pub struct Opaque2d;      // 不透明 2D 物体
pub struct Transparent2d; // 透明 2D 物体

// 阴影渲染阶段
pub struct Shadow;        // 阴影贴图

// UI 渲染阶段
pub struct Ui;            // 用户界面
```

## 4. 视图（Views）管理

### 4.1 视图类型

```rust
// crates/bevy_render/src/view/mod.rs

/// 提取的视图 - 渲染世界的视图表示
#[derive(Component)]
pub struct ExtractedView {
    /// 投影矩阵
    pub projection: Mat4,
    /// 视图矩阵
    pub view: Mat4,
    /// 视图投影矩阵
    pub view_projection: Mat4,
    /// 摄像机位置
    pub camera_position: Vec3,
    /// 视口
    pub viewport: Viewport,
    /// 渲染目标
    pub target: NormalizedRenderTarget,
}

/// 视图目标 - 渲染输出目标
pub struct ViewTarget {
    /// 颜色附件
    pub color_attachment: ColorAttachment,
    /// 深度附件
    pub depth_attachment: Option<DepthAttachment>,
    /// 尺寸
    pub size: Extent3d,
}
```

### 4.2 多视图渲染

Bevy 支持同时渲染多个视图：

```
┌─────────────────────────────────────────────────────────────┐
│                    多视图渲染示例                            │
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │  主摄像机   │  │  小地图     │  │  阴影贴图   │         │
│  │  (View 1)   │  │  (View 2)   │  │  (View 3)   │         │
│  │             │  │             │  │             │         │
│  │ Opaque3d    │  │ Opaque3d    │  │ Shadow      │         │
│  │ AlphaMask3d │  │ AlphaMask3d │  │             │         │
│  │ Transparent3d│  │ Transparent3d│  │             │         │
│  │ UI          │  │             │  │             │         │
│  └─────────────┘  └─────────────┘  └─────────────┘         │
│                                                             │
│  每个视图有自己的渲染阶段列表和绘制命令                      │
└─────────────────────────────────────────────────────────────┘
```

## 5. 渲染系统调度（Render Systems）

### 5.1 渲染系统集

```rust
// crates/bevy_render/src/lib.rs

pub enum RenderSystems {
    ExtractCommands,          // 应用 ExtractSchedule 的命令
    PrepareAssets,            // 准备资源
    PrepareMeshes,            // 准备网格
    CreateViews,              // 创建视图（阴影等）
    Specialize,               // 材质网格特化
    PrepareViews,             // 准备视图
    Queue,                    // 将实体加入渲染阶段
    QueueMeshes,              // 网格队列子集
    QueueSweep,               // 清理不可见网格
    PhaseSort,                // 排序渲染阶段
    Prepare,                  // 准备渲染资源
    PrepareResources,         // 准备缓冲区、纹理、uniform
    PrepareResourcesBatchPhases, // 批处理渲染阶段
    PrepareResourcesWritePhaseBuffers, // 写入 GPU
    PrepareResourcesCollectPhaseBuffers, // 收集阶段缓冲区
    PrepareResourcesFlush,    // 刷新缓冲区
    PrepareBindGroups,        // 创建绑定组
    Render,                   // 实际渲染
    Cleanup,                  // 清理资源
    PostCleanup,              // 最终清理
}
```

### 5.2 渲染流程

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         渲染流程（每帧）                                     │
│                                                                             │
│  1. ExtractCommands                                                         │
│     └── 将 Main World 的数据提取到 Render World                             │
│                                                                             │
│  2. PrepareAssets                                                           │
│     └── 准备 GPU 资源（纹理、网格、材质）                                    │
│                                                                             │
│  3. CreateViews                                                             │
│     └── 创建额外视图（阴影贴图、反射探针）                                   │
│                                                                             │
│  4. Specialize                                                              │
│     └── 特化渲染管线（根据材质和网格组合）                                   │
│                                                                             │
│  5. PrepareViews                                                            │
│     └── 准备视图目标（颜色/深度附件）                                        │
│                                                                             │
│  6. Queue                                                                   │
│     └── 将可见实体加入渲染阶段                                               │
│                                                                             │
│  7. PhaseSort                                                               │
│     └── 排序渲染阶段中的绘制项                                               │
│                                                                             │
│  8. Prepare                                                                 │
│     └── 准备渲染资源（Uniforms、绑定组）                                     │
│                                                                             │
│  9. Render                                                                  │
│     └── 执行 GPU 绘制命令                                                    │
│                                                                             │
│  10. Cleanup                                                                │
│     └── 清理临时资源                                                         │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 6. 批处理系统（Batching）

### 6.1 批处理类型

```rust
// crates/bevy_render/src/batching/mod.rs

/// 批处理元数据
#[derive(PartialEq)]
struct BatchSetMeta<T: PartialEq> {
    pipeline_id: CachedRenderPipelineId,
    draw_function_id: DrawFunctionId,
    dynamic_offset: Option<NonMaxU32>,
    user_data: T,
}

/// GPU 预处理批处理
pub struct GpuPreprocessingMode;

/// 非 GPU 预处理批处理
pub struct NoGpuPreprocessing;
```

### 6.2 批处理流程

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         批处理流程                                           │
│                                                                             │
│  1. 收集阶段                                                                  │
│     └── 收集所有可见实体                                                      │
│                                                                             │
│  2. 分箱阶段                                                                  │
│     └── 按 pipeline_id + draw_function_id 分箱                               │
│                                                                             │
│  3. 批处理阶段                                                                │
│     └── 合并相同批次的绘制命令                                                │
│                                                                             │
│  4. GPU 预处理（可选）                                                        │
│     └── 使用计算着色器预处理实例数据                                          │
│                                                                             │
│  5. 写入阶段                                                                  │
│     └── 将批处理数据写入 GPU 缓冲区                                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 6.3 批处理条件

两个绘制命令可以合并的条件：
- 相同的渲染管线 (`pipeline_id`)
- 相同的绘制函数 (`draw_function_id`)
- 相同的动态偏移 (`dynamic_offset`)
- 相同的用户数据 (`user_data`)

## 7. 绘制命令管理

### 7.1 PhaseItem  trait

```rust
// crates/bevy_render/src/render_phase/mod.rs

/// 渲染阶段项 - 代表一个可绘制的实体
pub trait PhaseItem: Send + Sync + 'static {
    /// 排序键类型
    type SortKey: Ord;
    
    /// 获取排序键
    fn sort_key(&self) -> Self::SortKey;
    
    /// 获取绘制函数 ID
    fn draw_function(&self) -> DrawFunctionId;
    
    /// 获取实体（可选）
    fn entity(&self) -> Option<Entity>;
    
    /// 设置绘制函数
    fn set_draw_function(&mut self, draw_function: DrawFunctionId);
}

/// 分箱阶段项 - 用于 BinnedRenderPhase
pub trait BinnedPhaseItem: PhaseItem {
    /// 批次集键类型
    type BatchSetKey: PartialEq + Eq + PartialOrd + Ord + Hash + Clone;
    
    /// 分箱键类型
    type BinKey: PartialEq + Eq + PartialOrd + Ord + Clone;
    
    /// 获取批次集键
    fn batch_set_key(&self) -> Option<Self::BatchSetKey>;
    
    /// 获取分箱键
    fn bin_key(&self) -> Self::BinKey;
}
```

### 7.2 绘制函数

```rust
// crates/bevy_render/src/render_phase/draw.rs

/// 绘制函数 trait
pub trait Draw<P: PhaseItem>: Send + Sync + 'static {
    /// 准备绘制状态
    fn prepare(
        &self,
        item: &P,
        pass: &mut TrackedRenderPass<'static>,
    ) -> Result<(), DrawError>;
    
    /// 执行绘制
    fn draw(
        &self,
        item: &P,
        pass: &mut TrackedRenderPass<'static>,
    ) -> Result<(), DrawError>;
}
```

## 8. 资源管理

### 8.1 渲染资源类型

```rust
// crates/bevy_render/src/render_resource/

pub struct RenderPipeline;      // 渲染管线
pub struct BindGroup;           // 绑定组
pub struct BindGroupLayout;     // 绑定组布局
pub struct Buffer;              // GPU 缓冲区
pub struct Texture;             // GPU 纹理
pub struct Sampler;             // 采样器
pub struct Shader;              // 着色器
```

### 8.2 管线缓存

```rust
// crates/bevy_render/src/render_resource/pipeline_cache.rs

pub struct PipelineCache {
    /// 已编译的渲染管线
    render_pipelines: HashMap<CachedRenderPipelineId, RenderPipeline>,
    /// 已编译的计算管线
    compute_pipelines: HashMap<CachedComputePipelineId, ComputePipeline>,
    /// 等待编译的管线
    waiting_pipelines: Vec<WaitingPipeline>,
}
```

## 9. 与 FHRE 的对比

| 特性 | Bevy | FHRE (设计目标) |
|------|------|-----------------|
| **渲染架构** | 双世界 (Main + Render) | **双世界 (Main + Render)** |
| **渲染阶段** | Binned + Sorted | **Binned + Sorted** |
| **视图管理** | 多视图（摄像机、阴影） | **单视图（嵌入式）** |
| **批处理** | GPU 预处理 + 自动批处理 | **CPU 批处理（嵌入式限制）** |
| **管线缓存** | 完整管线缓存 | **简化管线管理** |
| **资源管理** | AssetServer + RenderAssets | **简化资源管理** |
| **目标平台** | 桌面/移动/主机 | **嵌入式（NuttX）** |

## 10. 总结

Bevy 的渲染世界设计特点：

1. **双世界架构**: Main World 处理游戏逻辑，Render World 处理 GPU 渲染
2. **渲染阶段**: 灵活的阶段系统，支持不透明和透明物体的不同处理
3. **多视图**: 支持同时渲染多个摄像机/阴影贴图
4. **自动批处理**: GPU 预处理和自动批处理减少绘制调用
5. **系统调度**: 精细的系统集控制渲染流程
6. **资源管理**: 完善的管线缓存和资源管理系统

适用于高性能、多平台的游戏渲染需求。

---

**文档版本**: v1.0  
**分析日期**: 2026-04-14  
**Bevy 版本**: 0.15+  
**作者**: FeatherOS Team
