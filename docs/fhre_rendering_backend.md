# FHRE 渲染后端实现分析

## 1. 架构概述

FHRE (Feather Hybrid Render Engine) 是一个轻量级的混合渲染引擎，设计用于嵌入式系统。作为 UI 系统和游戏引擎的混合体，采用类似 Bevy 的 Extract-Prepare-Queue 架构模式。

```
┌─────────────────────────────────────────────────────────────────┐
│                        Main World                               │
│  - Entity + Transform3D + Sprite                                │
│  - Entity + Mesh                                                │
└─────────────────────────────────────────────────────────────────┘
                              │ Extract
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       Render World                              │
│  - ExtractedTransform, ExtractedMesh, ExtractedView            │
│  - RenderCommand, DrawCall, Texture, Gradient                  │
└─────────────────────────────────────────────────────────────────┘
                              │ Queue
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       Render Phases                             │
│  Background → Opaque2d → Opaque3d → AlphaMask → Transparent → Ui│
└─────────────────────────────────────────────────────────────────┘
                              │ Render
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Rendering Backend                            │
│  SoftwareBackend (CPU) / GpuBackend / HybridScheduler           │
└─────────────────────────────────────────────────────────────────┘
```

## 2. 渲染后端类型

### 2.1 Renderer Trait

```rust
pub trait Renderer {
    fn new(width: u32, height: u32) -> Self where Self: Sized;
    fn execute_commands(&mut self, commands: &[RenderCommand]);
    fn execute_phase(&mut self, phase: RenderPhaseType, commands: &[RenderCommand]);
    fn framebuffer(&self) -> &[u32];
    fn framebuffer_mut(&mut self) -> &mut [u32];
    fn resize(&mut self, width: u32, height: u32);
    fn set_viewport(&mut self, rect: Rect);
    fn reset(&mut self);
}
```

### 2.2 渲染器类型

```rust
pub enum RendererType {
    Software,  // 纯软件 CPU 渲染
    Gpu,       // GPU 加速渲染 (规划中)
    Hybrid,    // 混合渲染 (GPU批处理 + CPU小任务)
}
```

## 3. SoftwareBackend 实现

### 3.1 核心结构

```rust
pub struct SoftwareBackend {
    framebuffer: Vec<u32>,              // 像素缓冲区 (BGRA8888)
    width: u32,
    height: u32,
    viewport: Rect,                     // 裁剪区域
    blend_lut: [[u8; 256]; 256],        // Alpha 混合查找表
}
```

### 3.2 支持的渲染命令

| 命令 | 说明 | 实现算法 |
|------|------|----------|
| `Clear` | 清屏 | 直接填充 |
| `DrawRect` | 绘制矩形 | 逐像素填充 + Alpha 混合 |
| `DrawRectGradient` | 渐变矩形 | 逐像素采样渐变 |
| `DrawRectRounded` | 圆角矩形 | 四角圆形裁剪 |
| `DrawRectRoundedGradient` | 圆角渐变矩形 | 组合实现 |
| `DrawLine` | 绘制线段 | Bresenham 算法 |
| `DrawTriangle` | 绘制三角形 | 包围盒 + 重心坐标 |
| `DrawPolygon` | 绘制多边形 | 扫描线填充算法 |
| `DrawText` | 绘制文本 | 占位符 (矩形) |
| `DrawImage` | 绘制纹理 | 纹理采样 + 混合 |
| `DrawImageTransformed` | 变换纹理 | 旋转变换 + 多边形填充 |
| `SetScissor` | 设置裁剪区 | 修改 viewport |
| `DisableScissor` | 禁用裁剪 | 恢复全屏 |
| `PushMask` | 压入遮罩 | 遮罩栈 (规划中) |
| `PopMask` | 弹出遮罩 | 遮罩栈 (规划中) |

## 4. Alpha 混合系统

### 4.1 BlendMode 枚举

```rust
pub enum BlendMode {
    Normal,    // 标准 Alpha 混合 (Source Over)
    Additive,  // 加法混合
    Multiply,  // 乘法混合
    Screen,    // 屏幕混合
}
```

### 4.2 混合算法

**Normal (Source Over)**:
```rust
fn blend_normal(self, bg: Color) -> Color {
    let fg_alpha = self.a as u32;
    let bg_alpha = bg.a as u32;
    let out_alpha = fg_alpha + ((255 - fg_alpha) * bg_alpha + 127) / 255;

    let blend_channel = |fg: u8, bg: u8| -> u8 {
        ((fg as u32 * 255 * fg_alpha + bg as u32 * bg_alpha * (255 - fg_alpha)) 
         / (out_alpha * 255)) as u8
    };

    Color::new(
        blend_channel(self.r, bg.r),
        blend_channel(self.g, bg.g),
        blend_channel(self.b, bg.b),
        out_alpha as u8,
    )
}
```

**Additive**:
```rust
fn blend_additive(self, bg: Color) -> Color {
    Color::new(
        self.r.saturating_add((bg.r as u16 * (255 - self.a as u16) / 255) as u8),
        self.g.saturating_add((bg.g as u16 * (255 - self.a as u16) / 255) as u8),
        self.b.saturating_add((bg.b as u16 * (255 - self.a as u16) / 255) as u8),
        self.a.max(bg.a),
    )
}
```

### 4.3 混合查找表优化

```rust
// 预计算 Alpha 混合结果
fn build_blend_lut(&mut self) {
    for fg_alpha in 0..256 {
        for bg_alpha in 0..256 {
            let result = fg_alpha + ((255 - fg_alpha) * bg_alpha + 127) / 255;
            self.blend_lut[fg_alpha][bg_alpha] = result.min(255) as u8;
        }
    }
}

// 快速混合
fn blend_colors_fast(fg: Color, bg: Color, lut: &[[u8; 256]; 256]) -> Color {
    let out_alpha = lut[fg.a as usize][bg.a as usize];
    // ... 使用查找表加速
}
```

## 5. 纹理系统

### 5.1 纹理格式

```rust
pub enum TextureFormat {
    Rgba32,     // 32位 RGBA
    Rgb24,      // 24位 RGB
    Argb32,     // 32位 ARGB
    A8,         // 8位 Alpha
    L8,         // 8位灰度
    La16,       // 16位灰度+Alpha
    Rgb565,     // 16位 RGB
    Rgba4444,   // 16位 RGBA
}
```

### 5.2 采样器配置

```rust
pub enum SamplerFilter {
    Nearest,  // 最近邻采样
    Linear,   // 双线性插值
}

pub enum SamplerAddress {
    ClampToEdge,  // 边缘拉伸
    Repeat,       // 重复
    Mirror,       // 镜像重复
}

pub struct Sampler {
    pub min_filter: SamplerFilter,
    pub mag_filter: SamplerFilter,
    pub address_u: SamplerAddress,
    pub address_v: SamplerAddress,
}
```

### 5.3 纹理采样

**最近邻采样**:
```rust
fn sample_nearest(&self, u: f32, v: f32) -> Color {
    let x = (u * self.width as f32).min(self.width as f32 - 0.001) as u32;
    let y = (v * self.height as f32).min(self.height as f32 - 0.001) as u32;
    self.get_pixel(x, y)
}
```

**双线性插值**:
```rust
fn sample_linear(&self, u: f32, v: f32) -> Color {
    let fx = u * self.width as f32 - 0.5;
    let fy = v * self.height as f32 - 0.5;

    let x0 = fx.floor() as u32;
    let y0 = fy.floor() as u32;
    let x1 = (x0 + 1).min(self.width - 1);
    let y1 = (y0 + 1).min(self.height - 1);

    let dx = (fx - x0 as f32).clamp(0.0, 1.0);
    let dy = (fy - y0 as f32).clamp(0.0, 1.0);

    // 双线性插值
    let c0 = Color::lerp(c00, c10, dx);
    let c1 = Color::lerp(c01, c11, dx);
    Color::lerp(c0, c1, dy)
}
```

### 5.4 纹理区域 (TextureRegion)

```rust
pub struct TextureRegion {
    pub texture_id: u32,
    pub u0: f32, pub v0: f32,  // 左上角 UV
    pub u1: f32, pub v1: f32,  // 右下角 UV
}

impl TextureRegion {
    // 从图集创建子区域
    pub fn from_rect(texture_id: u32, x: u32, y: u32, 
                     width: u32, height: u32,
                     tex_width: u32, tex_height: u32) -> Self {
        Self {
            texture_id,
            u0: x as f32 / tex_width as f32,
            v0: y as f32 / tex_height as f32,
            u1: (x + width) as f32 / tex_width as f32,
            v1: (y + height) as f32 / tex_height as f32,
        }
    }
}
```

## 6. 渐变系统

### 6.1 渐变类型

```rust
pub enum GradientDirection {
    Horizontal,  // 水平渐变
    Vertical,    // 垂直渐变
    Linear,      // 任意角度线性渐变
    Radial,      // 径向渐变
    Sweep,       // 锥形渐变
}
```

### 6.2 渐变停止点

```rust
pub struct GradientStop {
    pub position: f32,  // 0.0 - 1.0
    pub color: Color,
}

pub struct Gradient {
    pub stops: Vec<GradientStop>,
    pub direction: GradientDirection,
    pub start: Vec2,
    pub end: Vec2,
    pub angle: f32,
}
```

### 6.3 渐变创建

```rust
impl Gradient {
    // 水平渐变
    pub fn horizontal(colors: &[Color]) -> Self;

    // 垂直渐变
    pub fn vertical(colors: &[Color]) -> Self;

    // 任意角度线性渐变
    pub fn linear(colors: &[Color], angle: f32) -> Self;

    // 径向渐变
    pub fn radial(colors: &[Color], center: Vec2, radius: f32) -> Self;

    // 锥形渐变
    pub fn sweep(colors: &[Color], center: Vec2, 
                 start_angle: f32, end_angle: f32) -> Self;
}
```

### 6.4 渐变采样

```rust
impl Gradient {
    pub fn sample(&self, x: f32, y: f32) -> Color {
        let t = self.calculate_t(x, y);
        self.sample_at(t)
    }

    fn calculate_t(&self, x: f32, y: f32) -> f32 {
        match self.direction {
            GradientDirection::Horizontal => {
                ((x - self.start.x) / (self.end.x - self.start.x)).clamp(0.0, 1.0)
            }
            GradientDirection::Vertical => {
                ((y - self.start.y) / (self.end.y - self.start.y)).clamp(0.0, 1.0)
            }
            GradientDirection::Radial => {
                let dx = x - self.start.x;
                let dy = y - self.start.y;
                let dist = (dx * dx + dy * dy).sqrt();
                let radius = (self.end - self.start).length();
                (dist / radius).clamp(0.0, 1.0)
            }
            // ...
        }
    }
}
```

### 6.5 预计算渐变

```rust
pub struct PrecomputedGradient {
    pub colors: Vec<Color>,
}

impl Gradient {
    // 预计算渐变颜色表，用于高效渲染
    pub fn precompute(&self, width: u32) -> PrecomputedGradient {
        let mut colors = Vec::with_capacity(width as usize);
        for x in 0..width {
            let t = x as f32 / (width - 1) as f32;
            colors.push(self.sample_at(t));
        }
        PrecomputedGradient { colors }
    }
}
```

## 7. 圆角矩形

### 7.1 算法实现

```rust
fn fill_rect_rounded(&mut self, rect: Rect, color: Color, radius: f32) {
    let r = radius.min(rect.width.min(rect.height) / 2.0);

    // 四个圆角的圆心
    let cx1 = (rect.x + r) as i32;           // 左上
    let cy1 = (rect.y + r) as i32;
    let cx2 = (rect.x + rect.width - r) as i32;  // 右上
    let cy2 = (rect.y + rect.height - r) as i32; // 右下

    for y in y0..y1 {
        for x in x0..x1 {
            let mut in_rect = true;

            // 检查四个角
            if px < cx1 && py < cy1 {
                // 左上角: 检查是否在圆内
                let dx = cx1 - px;
                let dy = cy1 - py;
                in_rect = (dx * dx + dy * dy) as f32 <= r * r;
            } else if px > cx2 && py < cy1 {
                // 右上角
                // ...
            } else if px > cx2 && py > cy2 {
                // 右下角
                // ...
            } else if px < cx1 && py > cy2 {
                // 左下角
                // ...
            }

            if in_rect {
                self.blend_pixel(x, y, color);
            }
        }
    }
}
```

## 8. 渲染阶段系统

### 8.1 渲染阶段类型

```rust
pub enum RenderPhaseType {
    Background = 0,   // 背景清屏
    Opaque2d = 1,     // 不透明 2D
    Opaque3d = 2,     // 不透明 3D
    AlphaMask = 3,    // Alpha 遮罩
    Transparent = 4,  // 透明对象 (需排序)
    Ui = 5,           // UI 覆盖层
}
```

### 8.2 PhaseItem

```rust
pub struct PhaseItem {
    pub sort_key: i32,        // 排序键
    pub z_depth: f32,         // Z 深度
    pub entity_id: Option<u32>,
    pub draw_command_index: usize,
    pub batchable: bool,      // 是否可批处理
    pub batch_key: u64,       // 批处理键
}
```

## 9. 混合调度器

### 9.1 任务分类

```rust
fn classify_task(command: &RenderCommand) -> TaskType {
    match command {
        // CPU 任务: 状态改变、复杂算法
        RenderCommand::Clear { .. } => TaskType::Cpu,
        RenderCommand::SetScissor { .. } => TaskType::Cpu,
        RenderCommand::DrawText { .. } => TaskType::Cpu,
        RenderCommand::DrawPolygon { .. } => TaskType::Cpu,

        // GPU 任务: 可批处理的几何图元
        RenderCommand::DrawRect { .. } => TaskType::Gpu,
        RenderCommand::DrawLine { .. } => TaskType::Gpu,
        RenderCommand::DrawTriangle { .. } => TaskType::Gpu,
    }
}
```

## 10. 与 LVGL/Bevy 对比

| 特性 | FHRE | LVGL | Bevy |
|------|------|------|------|
| 渲染方式 | 软件 (可扩展 GPU) | 纯软件 | GPU (wgpu) |
| Alpha 混合 | ✅ 多种模式 | ✅ Blend 系统 | ✅ BlendMode |
| 纹理映射 | ✅ 多格式+采样 | ✅ Image 渲染 | ✅ GpuImage |
| 渐变填充 | ✅ 5种类型 | ✅ 多种渐变 | ✅ Material |
| 圆角矩形 | ✅ | ✅ Radius 遮罩 | ❌ |
| 遮罩/裁剪 | ✅ Scissor | ✅ Mask 系统 | ✅ Scissor |
| 渲染阶段 | ✅ 6 阶段 | ❌ | ✅ 多阶段 |
| 批处理 | ✅ HybridScheduler | ❌ | ✅ BinnedPhaseItem |
| 颜色格式 | ✅ 8种 | ✅ 多格式 | ✅ 多格式 |
| 抗锯齿 | ❌ | ✅ 可选 | ✅ MSAA |
| 目标平台 | 嵌入式 | 嵌入式 | 桌面/移动端 |

## 11. 关键文件路径

| 功能 | 文件路径 |
|------|----------|
| 渲染后端入口 | `src/pipeline/mod.rs` |
| 软件渲染后端 | `src/pipeline/backend.rs` |
| 渲染器 Trait | `src/pipeline/renderer.rs` |
| 批处理系统 | `src/pipeline/batch.rs` |
| 渲染命令 | `src/render_world/command.rs` |
| 渲染阶段 | `src/render_world/phase.rs` |
| 纹理系统 | `src/render_world/texture.rs` |
| 渐变系统 | `src/render_world/gradient.rs` |
| 视图系统 | `src/render_world/view.rs` |
| 颜色系统 | `src/math/color.rs` |

## 12. 待实现功能

### 12.1 高优先级
- 纹理渲染集成 (TextureManager)
- 字体渲染系统
- 图集支持 (TextureAtlas)

### 12.2 中优先级
- 抗锯齿 (AA)
- 阴影效果
- 模糊效果
- 裁剪路径 (ClipPath)

### 12.3 低优先级
- GPU 后端 (OpenGL ES)
- 着色器系统
- 后处理效果

## 13. 总结

FHRE 软件渲染后端已实现 LVGL 和 Bevy 共有的核心渲染能力：

1. **Alpha 混合**: 支持 Normal、Additive、Multiply、Screen 四种混合模式
2. **纹理系统**: 支持 8 种颜色格式，最近邻/双线性采样，边缘/重复/镜像寻址
3. **渐变系统**: 支持水平、垂直、线性、径向、锥形五种渐变
4. **圆角矩形**: 四角圆形裁剪算法
5. **裁剪系统**: Scissor 矩形裁剪

相比 LVGL，FHRE 采用了更现代的架构设计（渲染阶段、批处理、混合调度），同时保持了嵌入式友好的纯 CPU 实现。相比 Bevy，FHRE 目前缺少 GPU 加速和着色器系统，但架构上预留了扩展接口。
