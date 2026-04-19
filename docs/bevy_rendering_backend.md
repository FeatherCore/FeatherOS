# Bevy 渲染后端架构分析

## 1. 架构概述

Bevy 是一个现代的 Rust 游戏引擎，采用 ECS (Entity Component System) 架构，其渲染系统基于 wgpu 实现跨平台 GPU 访问。

```
┌─────────────────────────────────────────────────────────────────┐
│                        Main World (App)                         │
│  - ECS 组件: Sprite, Mesh2d, Material, Camera                  │
│  - Assets: Image, Shader, Mesh                                   │
└─────────────────────────────────────────────────────────────────┘
                              │ Extract
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       Render World                               │
│  - Extracted Components: ExtractedSprite, ExtractedView         │
│  - GPU Resources: GpuImage, RenderPipeline, BindGroup           │
└─────────────────────────────────────────────────────────────────┘
                              │ Render Graph
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         wgpu                                     │
│  - RenderDevice, RenderQueue, RenderAdapter                     │
│  - Cross-platform: Vulkan, Metal, DX12, OpenGL, WebGPU          │
└─────────────────────────────────────────────────────────────────┘
```

### 1.1 核心设计模式

**Extract-Prepare-Queue 模式**:
```
Main World                Render World
    │                          │
    │   ExtractSchedule        │  提取数据到渲染世界
    │ ────────────────────────>│
    │                          │
    │   PrepareAssets          │  准备 GPU 资源
    │                          │
    │   Prepare                │  创建缓冲区、绑定组
    │                          │
    │   Queue                  │  将实体加入渲染阶段
    │                          │
    │   PhaseSort              │  排序渲染项目
    │                          │
    │   Render                 │  执行 GPU 绘制
    │                          │
```

## 2. 渲染后端

### 2.1 wgpu 抽象层

Bevy 使用 wgpu 作为底层 GPU 抽象，支持多种图形 API：

| 平台 | 后端 |
|------|------|
| Windows | DirectX 12, DirectX 11, Vulkan |
| macOS/iOS | Metal |
| Linux | Vulkan |
| Web | WebGL2, WebGPU |
| Android | Vulkan |

### 2.2 核心资源

```rust
// 渲染设备 - GPU 设备抽象
#[derive(Resource)]
pub struct RenderDevice(Arc<WgpuWrapper<wgpu::Device>>);

// 渲染队列 - GPU 命令提交
#[derive(Resource)]
pub struct RenderQueue(Arc<WgpuWrapper<wgpu::Queue>>);

// 渲染适配器 - 物理设备
#[derive(Resource)]
pub struct RenderAdapter(Arc<WgpuWrapper<wgpu::Adapter>>);

// 渲染实例 - wgpu 入口
#[derive(Resource)]
pub struct RenderInstance(Arc<WgpuWrapper<wgpu::Instance>>);
```

### 2.3 后端选择环境变量

```bash
WGPU_DEBUG=1              # 启用调试标签
WGPU_VALIDATION=0         # 禁用验证层
WGPU_FORCE_FALLBACK_ADAPTER=1  # 强制软件渲染
WGPU_ADAPTER_NAME         # 按名称选择适配器
WGPU_SETTINGS_PRIO=webgl2 # 使用 WebGL2 限制
```

## 3. 渲染阶段 (Render Phase)

### 3.1 2D 渲染阶段

Bevy 2D 渲染分为三个阶段：

```rust
// 不透明 2D 对象 - 使用 BinnedPhaseItem (批处理优化)
pub struct Opaque2d {
    pub batch_set_key: BatchSetKey2d,
    pub bin_key: Opaque2dBinKey,
    pub representative_entity: (Entity, MainEntity),
    pub batch_range: Range<u32>,
    pub extra_index: PhaseItemExtraIndex,
}

// Alpha 遮罩 2D 对象
pub struct AlphaMask2d { ... }

// 透明 2D 对象 - 使用 SortedPhaseItem (从后到前排序)
pub struct Transparent2d {
    pub sort_key: FloatOrd,  // 按深度排序
    pub entity: (Entity, MainEntity),
    pub pipeline: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
}
```

### 3.2 阶段类型

| 类型 | 特点 | 适用场景 |
|------|------|----------|
| BinnedPhaseItem | 批处理、无严格排序 | 不透明对象 |
| SortedPhaseItem | 严格排序 | 透明对象 |

### 3.3 渲染系统顺序

```rust
pub enum RenderSystems {
    ExtractCommands,  // 应用提取命令
    PrepareAssets,    // 准备修改的资源
    PrepareMeshes,    // 准备网格
    CreateViews,      // 创建视图
    Specialize,       // 特化材质管线
    PrepareViews,     // 准备视图资源
    Queue,            // 将实体加入渲染阶段
    PhaseSort,        // 排序渲染阶段
    Prepare,          // 准备 GPU 资源
    Render,           // 执行渲染
    Cleanup,          // 清理资源
}
```

## 4. Sprite 渲染实现

### 4.1 Sprite 组件

```rust
pub struct Sprite {
    pub image: Handle<Image>,           // 图像句柄
    pub texture_atlas: Option<TextureAtlas>, // 纹理图集
    pub color: Color,                   // 颜色调制
    pub flip_x: bool,                   // 水平翻转
    pub flip_y: bool,                   // 垂直翻转
    pub custom_size: Option<Vec2>,      // 自定义尺寸
    pub rect: Option<Rect>,             // 纹理区域
    pub image_mode: SpriteImageMode,    // 图像模式
}
```

### 4.2 Sprite 渲染管线

```rust
#[derive(Resource)]
pub struct SpritePipeline {
    view_layout: BindGroupLayoutDescriptor,    // 视图绑定组布局
    material_layout: BindGroupLayoutDescriptor, // 材质绑定组布局
    shader: Handle<Shader>,                     // 着色器
}
```

### 4.3 Sprite 着色器 (WGSL)

**顶点着色器**:
```wgsl
struct VertexInput {
    @builtin(vertex_index) index: u32,
    // 实例率顶点属性
    @location(0) i_model_transpose_col0: vec4<f32>,  // 变换矩阵列0
    @location(1) i_model_transpose_col1: vec4<f32>,  // 变换矩阵列1
    @location(2) i_model_transpose_col2: vec4<f32>,  // 变换矩阵列2
    @location(3) i_color: vec4<f32>,                  // 颜色
    @location(4) i_uv_offset_scale: vec4<f32>,        // UV 偏移和缩放
}

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // 从顶点索引生成四边形顶点
    let vertex_position = vec3<f32>(
        f32(in.index & 0x1u),        // x: 0, 1, 0, 1
        f32((in.index & 0x2u) >> 1u), // y: 0, 0, 1, 1
        0.0
    );
    
    // 变换到裁剪空间
    out.clip_position = view.clip_from_world * affine3_to_square(mat3x4<f32>(
        in.i_model_transpose_col0,
        in.i_model_transpose_col1,
        in.i_model_transpose_col2,
    )) * vec4<f32>(vertex_position, 1.0);
    
    // 计算 UV
    out.uv = vec2<f32>(vertex_position.xy) * in.i_uv_offset_scale.zw 
           + in.i_uv_offset_scale.xy;
    out.color = in.i_color;
    
    return out;
}
```

**片段着色器**:
```wgsl
@group(1) @binding(0) var sprite_texture: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // 纹理采样并调制颜色
    var color = in.color * textureSample(sprite_texture, sprite_sampler, in.uv);
    
    // 色调映射 (可选)
    #ifdef TONEMAP_IN_SHADER
    color = tonemapping::tone_mapping(color, view.color_grading);
    #endif
    
    // 颜色空间转换 (可选)
    #ifdef SRGB_OUTPUT
    color = vec4(linear_to_srgb(color.rgb), color.a);
    #endif
    
    return color;
}
```

### 4.4 实例数据布局

```rust
// 每个 Sprite 实例的数据
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct SpriteInstance {
    // 3x4 仿射变换矩阵 (转置存储)
    i_model_transpose_col0: [f32; 4],
    i_model_transpose_col1: [f32; 4],
    i_model_transpose_col2: [f32; 4],
    // 颜色 (RGBA)
    i_color: [f32; 4],
    // UV 偏移 (xy) 和缩放 (zw)
    i_uv_offset_scale: [f32; 4],
}
```

### 4.5 Sprite 批处理

```
┌─────────────────────────────────────────────────────────────┐
│                    Sprite 批处理流程                         │
├─────────────────────────────────────────────────────────────┤
│  1. 按纹理分组                                               │
│  2. 相同纹理的 Sprite 写入同一个实例缓冲区                   │
│  3. 单次 Draw Call 绘制多个 Sprite                          │
│                                                             │
│  Sprite A ──┐                                               │
│  Sprite B ──┼──> Instance Buffer ──> Draw Instanced         │
│  Sprite C ──┘      (6 floats each)                         │
└─────────────────────────────────────────────────────────────┘
```

## 5. Mesh2D 渲染实现

### 5.1 Mesh2D 组件

```rust
// 2D 网格标记
#[derive(Component, Clone, Debug, Default, Deref, DerefMut)]
pub struct Mesh2d(pub Handle<Mesh>);

// 2D 材质
pub trait Material2d: Asset + AsBindGroup + Sized {
    fn fragment_shader() -> ShaderRef;
    fn vertex_shader() -> ShaderRef;
    fn specialize(descriptor: &mut RenderPipelineDescriptor, ...);
}
```

### 5.2 Mesh2D 管线键

```rust
bitflags::bitflags! {
    pub struct Mesh2dPipelineKey: u32 {
        const HDR;                    // HDR 渲染
        const TONEMAP_IN_SHADER;      // 着色器内色调映射
        const DEBAND_DITHER;          // 去色带抖动
        const BLEND_ALPHA;            // Alpha 混合
        const MAY_DISCARD;            // 可能丢弃像素
        const SRGB_COMPOSITING;       // sRGB 合成
        const OKLAB_COMPOSITING;      // OKLab 合成
        // MSAA 位 (高3位)
        // 色调映射方法位
        // 图元拓扑位
    }
}
```

### 5.3 Mesh2D 着色器

```wgsl
struct Vertex {
    @builtin(instance_index) instance_index: u32,
    #ifdef VERTEX_POSITIONS
    @location(0) position: vec3<f32>,
    #endif
    #ifdef VERTEX_NORMALS
    @location(1) normal: vec3<f32>,
    #endif
    #ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
    #endif
    #ifdef VERTEX_TANGENTS
    @location(3) tangent: vec4<f32>,
    #endif
    #ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
    #endif
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    
    #ifdef VERTEX_UVS
    out.uv = vertex.uv;
    #endif
    
    #ifdef VERTEX_POSITIONS
    var world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    out.world_position = mesh_functions::mesh2d_position_local_to_world(
        world_from_local,
        vec4<f32>(vertex.position, 1.0)
    );
    out.position = mesh_functions::mesh2d_position_world_to_clip(out.world_position);
    #endif
    
    #ifdef VERTEX_NORMALS
    out.world_normal = mesh_functions::mesh2d_normal_local_to_world(
        vertex.normal, vertex.instance_index
    );
    #endif
    
    return out;
}
```

## 6. 纹理映射实现

### 6.1 GPU 图像

```rust
#[derive(Debug, Clone)]
pub struct GpuImage {
    pub texture: Texture,              // wgpu 纹理
    pub texture_view: TextureView,     // 纹理视图
    pub sampler: Sampler,              // 采样器
    pub texture_descriptor: TextureDescriptor,
    pub texture_view_descriptor: Option<TextureViewDescriptor>,
}
```

### 6.2 纹理创建流程

```
Image Asset (主世界)
      │
      │ ExtractSchedule
      ▼
ExtractedImage (渲染世界)
      │
      │ PrepareAssets
      ▼
GpuImage (GPU 资源)
      │
      │ RenderQueue::write_texture
      ▼
GPU 显存
```

### 6.3 纹理绑定组

```rust
// 材质绑定组布局
let material_layout = BindGroupLayoutDescriptor::new(
    "sprite_material_layout",
    &BindGroupLayoutEntries::sequential(
        ShaderStages::FRAGMENT,
        (
            texture_2d(TextureSampleType::Float { filterable: true }),
            sampler(SamplerBindingType::Filtering),
        ),
    ),
);
```

### 6.4 纹理图集支持

```rust
// 纹理图集布局
#[derive(Asset, TypePath)]
pub struct TextureAtlasLayout {
    pub size: UVec2,           // 图集总尺寸
    pub textures: Vec<URect>,  // 每个子纹理的区域
}

// 纹理图集引用
pub struct TextureAtlas {
    pub layout: Handle<TextureAtlasLayout>,
    pub index: usize,
}
```

**UV 坐标计算**:
```rust
// 从图集计算 UV
fn compute_uv_from_atlas(layout: &TextureAtlasLayout, index: usize) -> Vec4 {
    let rect = layout.textures[index];
    let uv_offset = Vec2::new(
        rect.min.x as f32 / layout.size.x as f32,
        rect.min.y as f32 / layout.size.y as f32,
    );
    let uv_scale = Vec2::new(
        rect.width() as f32 / layout.size.x as f32,
        rect.height() as f32 / layout.size.y as f32,
    );
    Vec4::new(uv_offset.x, uv_offset.y, uv_scale.x, uv_scale.y)
}
```

## 7. 着色器系统

### 7.1 支持的着色器语言

| 语言 | 扩展 | 说明 |
|------|------|------|
| WGSL | .wgsl | 主要语言 (WebGPU) |
| GLSL | .vert, .frag, .comp | OpenGL |
| SPIR-V | .spv | 编译后二进制 |
| WESL | .wesl | 实验性扩展 |

### 7.2 着色器导入

```wgsl
// 导入内置模块
#import bevy_render::view::View
#import bevy_sprite::sprite_view_bindings::view

// 导入函数库
#import bevy_render::maths::affine3_to_square
#import bevy_render::color_operations::linear_to_srgb
```

### 7.3 着色器定义

```rust
// 运行时着色器定义
shader_defs.push("TONEMAP_IN_SHADER".into());
shader_defs.push(ShaderDefVal::UInt(
    "TONEMAPPING_LUT_TEXTURE_BINDING_INDEX".into(),
    1,
));
```

### 7.4 管线特化

```rust
impl SpecializedRenderPipeline for SpritePipeline {
    type Key = SpritePipelineKey;
    
    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut shader_defs = Vec::new();
        
        // 根据键值添加着色器定义
        if key.contains(SpritePipelineKey::TONEMAP_IN_SHADER) {
            shader_defs.push("TONEMAP_IN_SHADER".into());
        }
        
        // 选择渲染格式
        let format = match key.contains(SpritePipelineKey::HDR) {
            true => ViewTarget::TEXTURE_FORMAT_HDR,
            false => TextureFormat::bevy_default(),
        };
        
        RenderPipelineDescriptor { ... }
    }
}
```

## 8. 视图和相机系统

### 8.1 提取的相机

```rust
pub struct ExtractedCamera {
    pub target: Option<NormalizedRenderTarget>,  // 渲染目标
    pub physical_viewport_size: Option<UVec2>,   // 视口尺寸
    pub physical_target_size: Option<UVec2>,     // 目标尺寸
    pub schedule: InternedScheduleLabel,         // 渲染调度
    pub order: isize,                            // 渲染顺序
    pub output_mode: CameraOutputMode,           // 输出模式
    pub clear_color: ClearColorConfig,           // 清屏颜色
    pub exposure: f32,                           // 曝光
    pub hdr: bool,                               // HDR 模式
}
```

### 8.2 提取的视图

```rust
pub struct ExtractedView {
    pub retained_view_entity: RetainedViewEntity,
    pub clip_from_view: Mat4,       // 投影矩阵
    pub world_from_view: GlobalTransform,  // 世界变换
    pub clip_from_world: Option<Mat4>,     // 组合矩阵
    pub hdr: bool,
    pub compositing_space: Option<CompositingSpace>,
    pub viewport: UVec4,
    pub color_grading: ColorGrading,
    pub invert_culling: bool,
}
```

### 8.3 视图 Uniform

```rust
pub struct ViewUniform {
    pub clip_from_world: Mat4,      // 裁剪到世界
    pub world_from_clip: Mat4,      // 世界到裁剪
    pub world_from_view: Mat4,      // 世界到视图
    pub view_from_world: Mat4,      // 视图到世界
    pub clip_from_view: Mat4,       // 裁剪到视图
    pub view_from_clip: Mat4,       // 视图到裁剪
    pub world_position: Vec3,       // 世界位置
    pub exposure: f32,              // 曝光
    pub viewport: Vec4,             // 视口
    pub frustum: [Vec4; 6],         // 视锥体
    pub color_grading: ColorGradingUniform,
    pub mip_bias: f32,
    pub frame_count: u32,
}
```

### 8.4 视图目标 (双缓冲)

```rust
pub struct ViewTarget {
    main_textures: MainTargetTextures,  // A/B 缓冲区
    main_texture_format: TextureFormat,
    out_texture: OutputColorAttachment,
}
```

用于后处理的双缓冲：
```
┌─────────────┐     ┌─────────────┐
│  Buffer A   │ ──> │  Buffer B   │ ──> 后处理链
└─────────────┘     └─────────────┘
      ▲                   │
      └───────────────────┘
```

## 9. 渲染图 (Render Graph)

### 9.1 Core2D 调度

```rust
Core2d schedule:
├── main_opaque_pass_2d      // 不透明通道
├── main_transparent_pass_2d // 透明通道
├── tonemapping              // 色调映射
└── upscaling                // 升采样
```

### 9.2 渲染图系统集

```rust
pub enum RenderGraphSystems {
    Begin,   // 帧前设置
    Render,  // 主渲染
    Submit,  // 提交命令缓冲区
    Finish,  // 帧后清理
}
```

### 9.3 主渲染系统

```rust
pub fn render_system(world: &mut World, ...) {
    // 1. 运行渲染图调度
    world.run_schedule(RenderGraph);
    
    // 2. 创建命令编码器
    let mut encoder = render_device.create_command_encoder(...);
    
    // 3. 提交截图和回读命令
    submit_screenshot_commands(world, &mut encoder);
    submit_readback_commands(world, &mut encoder);
    
    // 4. 提交到 GPU
    render_queue.submit([encoder.finish()]);
    
    // 5. 呈现
    for window in windows.values_mut() {
        window.present();
    }
}
```

## 10. 批处理和优化

### 10.1 批处理架构

```
┌─────────────────────────────────────────────────────────────┐
│                     BinnedRenderPhase                        │
├─────────────────────────────────────────────────────────────┤
│  multidrawable_meshes:                                      │
│    BatchSetKey -> RenderMultidrawableBatchSet               │
│    (可多绘制的网格)                                          │
│                                                             │
│  batchable_meshes:                                          │
│    (BatchSetKey, BinKey) -> RenderBin                       │
│    (可批处理但不可多绘制)                                    │
│                                                             │
│  unbatchable_meshes:                                        │
│    (BatchSetKey, BinKey) -> UnbatchableBinnedEntities       │
│    (不可批处理，单独绘制)                                    │
└─────────────────────────────────────────────────────────────┘
```

### 10.2 批处理键

```rust
// 批处理集键 - 决定哪些对象可以放在一起
pub struct BatchSetKey2d {
    pub pipeline: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
    pub material_bind_group_id: Option<BindGroupId>,
}

// 箱键 - 决定具体批处理
pub struct Opaque2dBinKey {
    pub pipeline: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
    pub asset_id: UntypedAssetId,
    pub material_bind_group_id: Option<BindGroupId>,
}
```

### 10.3 实例缓冲区

```rust
// 批处理实例缓冲区
#[derive(Resource)]
pub struct BatchedInstanceBuffer<T: Pod> {
    buffer: RawBufferVec<T>,
    scratch: Vec<T>,
}

// 写入实例数据
pub fn write_batched_instance_buffer<T: Pod>(
    device: &RenderDevice,
    queue: &RenderQueue,
    buffer: &mut BatchedInstanceBuffer<T>,
) {
    buffer.write_buffer(device, queue);
}
```

### 10.4 多绘制间接

```rust
// GPU 预处理模式
pub enum GpuPreprocessingMode {
    None,                    // CPU 批处理
    Culling,                 // GPU 剔除
    CullingPreprocessing,    // GPU 剔除 + 预处理
}

// 间接参数
pub struct IndirectParametersCpuMetadata {
    pub batch_set_index: u32,
    pub batch_count: u32,
}
```

## 11. 材质系统

### 11.1 Material2d Trait

```rust
pub trait Material2d: Asset + AsBindGroup + Sized {
    /// 片段着色器
    fn fragment_shader() -> ShaderRef;
    
    /// 顶点着色器
    fn vertex_shader() -> ShaderRef;
    
    /// 管线特化
    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError>;
    
    /// 绑定组数据
    fn as_bind_group_data(&self) -> Self::Data;
}
```

### 11.2 ColorMaterial 示例

```rust
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct ColorMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
}

impl Material2d for ColorMaterial {
    fn fragment_shader() -> ShaderRef {
        "color_material.wgsl".into()
    }
}
```

### 11.3 材质绑定组

```wgsl
// color_material.wgsl
@group(2) @binding(0) var<uniform> color: vec4<f32>;
@group(2) @binding(1) var texture: texture_2d<f32>;
@group(2) @binding(2) var sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var out = color;
    #ifdef VERTEX_UVS
    out *= textureSample(texture, sampler, in.uv);
    #endif
    return out;
}
```

## 12. MSAA 支持

```rust
pub enum Msaa {
    Off = 1,       // 无抗锯齿
    Sample2 = 2,   // 2x MSAA
    Sample4 = 4,   // 4x MSAA (默认)
    Sample8 = 8,   // 8x MSAA
}

// 管线键中的 MSAA
impl SpritePipelineKey {
    pub const fn from_msaa_samples(msaa_samples: u32) -> Self {
        let msaa_bits = (msaa_samples.trailing_zeros() & Self::MSAA_MASK_BITS) 
                      << Self::MSAA_SHIFT_BITS;
        Self::from_bits_retain(msaa_bits)
    }
}
```

## 13. 关键文件路径

| 功能 | 文件路径 |
|------|----------|
| 渲染插件入口 | `crates/bevy_render/src/lib.rs` |
| 渲染设备 | `crates/bevy_render/src/renderer/render_device.rs` |
| 渲染阶段 | `crates/bevy_render/src/render_phase/mod.rs` |
| 纹理系统 | `crates/bevy_render/src/texture/gpu_image.rs` |
| 着色器系统 | `crates/bevy_shader/src/shader.rs` |
| 相机系统 | `crates/bevy_render/src/camera.rs` |
| 视图系统 | `crates/bevy_render/src/view/mod.rs` |
| Sprite 渲染 | `crates/bevy_sprite_render/src/render/mod.rs` |
| Sprite 着色器 | `crates/bevy_sprite_render/src/render/sprite.wgsl` |
| Mesh2D 渲染 | `crates/bevy_sprite_render/src/mesh2d/mesh.rs` |
| Mesh2D 着色器 | `crates/bevy_sprite_render/src/mesh2d/mesh2d.wgsl` |
| Core2D 管线 | `crates/bevy_core_pipeline/src/core_2d/mod.rs` |
| 管线缓存 | `crates/bevy_render/src/render_resource/pipeline_cache.rs` |

## 14. 总结

Bevy 渲染架构的核心特点：

1. **ECS 驱动**: 渲染数据作为组件存储，通过系统处理
2. **Extract-Prepare-Queue**: 清晰的渲染管线分离
3. **wgpu 抽象**: 跨平台 GPU 访问，支持 Vulkan/Metal/DX12/WebGPU
4. **Pipeline Specialization**: 运行时管线特化，支持多种渲染配置
5. **批处理优化**: BinnedPhaseItem 支持高效批处理和多绘制
6. **Shader Import**: WGSL 模块化着色器系统
7. **材质抽象**: Material2d trait 支持自定义材质
8. **视图双缓冲**: 支持后处理链

与 LVGL 软件渲染相比，Bevy 是完全的 GPU 渲染引擎，利用现代图形 API 实现高性能渲染，适合游戏和实时应用场景。
