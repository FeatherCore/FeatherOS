# Bevy 精灵系统分析

## 1. 概述

Bevy 的精灵系统是一个基于 ECS (Entity-Component-System) 架构的现代 2D 渲染系统。它利用 Bevy 的渲染管线、资源管理和组件系统，提供了高效、灵活的精灵渲染能力。

## 2. 核心组件

### 2.1 Sprite 组件

`Sprite` 组件是精灵系统的核心，定义了精灵的渲染属性：

```rust
// crates/bevy_sprite/src/sprite.rs
#[derive(Component, Debug, Default, Clone, Reflect, FromTemplate)]
#[require(Transform, Visibility, VisibilityClass, Anchor)]
#[reflect(Component, Default, Debug, Clone)]
#[component(on_add = visibility::add_visibility_class::<Sprite>)]
pub struct Sprite {
    /// 精灵使用的图像
    pub image: Handle<Image>,
    /// 可选的纹理图集
    #[template(OptionTemplate<TextureAtlasTemplate>)]
    pub texture_atlas: Option<TextureAtlas>,
    /// 精灵的颜色色调
    pub color: Color,
    /// 沿 X 轴翻转
    pub flip_x: bool,
    /// 沿 Y 轴翻转
    pub flip_y: bool,
    /// 可选的自定义尺寸
    pub custom_size: Option<Vec2>,
    /// 可选的矩形区域（用于渲染图像的一部分）
    pub rect: Option<Rect>,
    /// 图像缩放模式
    pub image_mode: SpriteImageMode,
}
```

**必需组件** (通过 `#[require]` 属性自动添加)：
- `Transform`: 位置、旋转、缩放
- `Visibility`: 可见性控制
- `VisibilityClass`: 可见性分类
- `Anchor`: 锚点（相对于 Transform 的偏移）

### 2.2 Anchor 组件

`Anchor` 定义了精灵相对于其 `Transform` 的标准化偏移量：

```rust
// crates/bevy_sprite/src/sprite.rs
#[derive(Component, Debug, Clone, Copy, PartialEq, Deref, DerefMut, Reflect)]
#[reflect(Component, Default, Debug, PartialEq, Clone)]
#[doc(alias = "pivot")]
pub struct Anchor(pub Vec2);

impl Anchor {
    pub const BOTTOM_LEFT: Self = Self(Vec2::new(-0.5, -0.5));
    pub const BOTTOM_CENTER: Self = Self(Vec2::new(0.0, -0.5));
    pub const BOTTOM_RIGHT: Self = Self(Vec2::new(0.5, -0.5));
    pub const CENTER_LEFT: Self = Self(Vec2::new(-0.5, 0.0));
    pub const CENTER: Self = Self(Vec2::ZERO);
    pub const CENTER_RIGHT: Self = Self(Vec2::new(0.5, 0.0));
    pub const TOP_LEFT: Self = Self(Vec2::new(-0.5, 0.5));
    pub const TOP_CENTER: Self = Self(Vec2::new(0.0, 0.5));
    pub const TOP_RIGHT: Self = Self(Vec2::new(0.5, 0.5));
}
```

### 2.3 SpriteMesh 组件

`SpriteMesh` 是使用 Mesh 后端而不是 Sprite 后端的替代实现：

```rust
// crates/bevy_sprite/src/sprite_mesh.rs
#[derive(Component, Debug, Default, Clone, Reflect, PartialEq, FromTemplate)]
#[require(Transform, Visibility, VisibilityClass, Anchor)]
pub struct SpriteMesh {
    pub image: Handle<Image>,
    pub texture_atlas: Option<TextureAtlas>,
    pub color: Color,
    pub flip_x: bool,
    pub flip_y: bool,
    pub custom_size: Option<Vec2>,
    pub rect: Option<Rect>,
    pub image_mode: SpriteImageMode,
    pub alpha_mode: SpriteAlphaMode,  // 额外的 alpha 模式控制
}
```

## 3. 精灵创建方式

### 3.1 从图像创建

```rust
// 从图像句柄创建精灵
let sprite = Sprite::from_image(image_handle);

// 创建带自定义尺寸的精灵
let sprite = Sprite::sized(Vec2::new(100.0, 100.0));

// 从纯色创建精灵
let sprite = Sprite::from_color(Color::RED, Vec2::new(50.0, 50.0));
```

### 3.2 从纹理图集创建

```rust
// 从纹理图集创建精灵
let sprite = Sprite::from_atlas_image(image_handle, texture_atlas);
```

### 3.3 实体创建

```rust
// 创建精灵实体
commands.spawn((
    Sprite::from_image(asset_server.load("sprite.png")),
    Transform::from_xyz(100.0, 100.0, 0.0),
));
```

## 4. 纹理图集系统 (Texture Atlas)

### 4.1 TextureAtlasLayout

`TextureAtlasLayout` 定义了纹理图集的布局：

```rust
// crates/bevy_image/src/texture_atlas.rs
#[derive(Asset, PartialEq, Eq, Debug, Clone)]
pub struct TextureAtlasLayout {
    /// 纹理图集的总尺寸
    pub size: UVec2,
    /// 图集中每个纹理的区域
    pub textures: Vec<URect>,
}

impl TextureAtlasLayout {
    /// 从网格创建布局
    pub fn from_grid(
        tile_size: UVec2,
        columns: u32,
        rows: u32,
        padding: Option<UVec2>,
        offset: Option<UVec2>,
    ) -> Self {
        // 生成网格布局...
    }
}
```

### 4.2 TextureAtlas

`TextureAtlas` 组件引用了一个 `TextureAtlasLayout` 和特定的索引：

```rust
// crates/bevy_image/src/texture_atlas.rs
#[derive(Component, Debug, Default, Clone, Reflect, PartialEq)]
#[reflect(Component, Default, Debug, Clone)]
pub struct TextureAtlas {
    /// 纹理图集布局的句柄
    pub layout: Handle<TextureAtlasLayout>,
    /// 当前显示的纹理索引
    pub index: usize,
}
```

### 4.3 使用示例

```rust
// 加载纹理图集
let texture_handle = asset_server.load("spritesheet.png");
let layout = TextureAtlasLayout::from_grid(UVec2::new(32, 32), 4, 4, None, None);
let layout_handle = texture_atlases.add(layout);

// 创建动画精灵
commands.spawn((
    Sprite::from_atlas_image(texture_handle, TextureAtlas {
        layout: layout_handle,
        index: 0,
    }),
    Transform::default(),
));
```

## 5. 图像缩放模式

### 5.1 SpriteImageMode

```rust
// crates/bevy_sprite/src/sprite.rs
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Debug, Default, Clone)]
pub enum SpriteImageMode {
    /// 使用图像的原始尺寸
    #[default]
    Auto,
    /// 使用自定义尺寸
    Custom(Vec2),
    /// 按比例缩放
    Scale(SpriteScalingMode),
}
```

### 5.2 SpriteScalingMode

```rust
// crates/bevy_sprite/src/sprite.rs
#[derive(Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Debug, Default, Clone)]
pub enum SpriteScalingMode {
    /// 填充并居中
    #[default]
    FillCenter,
    /// 填充并对齐到起始位置
    FillStart,
    /// 填充并对齐到结束位置
    FillEnd,
    /// 适应并居中
    FitCenter,
    /// 适应并对齐到起始位置
    FitStart,
    /// 适应并对齐到结束位置
    FitEnd,
}
```

## 6. 渲染管线

### 6.1 SpritePlugin

`SpritePlugin` 是精灵系统的入口插件：

```rust
// crates/bevy_sprite/src/lib.rs
pub struct SpritePlugin;

impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        // 添加纹理图集插件
        if !app.is_plugin_added::<TextureAtlasPlugin>() {
            app.add_plugins(TextureAtlasPlugin);
        }
        
        // 添加边界计算系统
        app.add_systems(
            PostUpdate,
            (calculate_bounds_2d, calculate_bounds_2d_sprite_mesh)
                .chain()
                .in_set(VisibilitySystems::CalculateBounds),
        );
        
        // 可选：添加文本和拾取支持
        #[cfg(feature = "bevy_text")]
        app.add_systems(PostUpdate, update_text2d_layout...);
        
        #[cfg(feature = "bevy_picking")]
        app.add_plugins(SpritePickingPlugin);
    }
}
```

### 6.2 渲染阶段

Bevy 的渲染管线分为多个阶段：

```
Extract → Prepare → Queue → PhaseSort → PrepareResources → Render
```

**系统集定义** (crates/bevy_render/src/lib.rs):

```rust
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum RenderSystem {
    ExtractCommands,      // 提取命令
    PrepareAssets,        // 准备资源
    Queue,                // 队列渲染项
    PhaseSort,            // 排序渲染阶段
    PrepareResources,     // 准备资源
    PrepareBindGroups,    // 准备绑定组
    Render,               // 执行渲染
}
```

## 7. 批处理系统

### 7.1 自动批处理

Bevy 使用批处理来优化精灵渲染：

```rust
// crates/bevy_render/src/batching/mod.rs
#[derive(Component, Default, Clone, Copy)]
pub struct NoAutomaticBatching;

/// 批处理元数据
#[derive(PartialEq)]
struct BatchSetMeta<T: PartialEq> {
    pipeline_id: CachedRenderPipelineId,
    draw_function_id: DrawFunctionId,
    dynamic_offset: Option<NonMaxU32>,
    user_data: T,
}
```

### 7.2 批处理条件

两个绘制命令可以合并的条件：
- 相同的渲染管线 (pipeline_id)
- 相同的绘制函数 (draw_function_id)
- 相同的动态偏移 (dynamic_offset)
- 相同的用户数据 (user_data)

### 7.3 GPU 预处理

```rust
// crates/bevy_render/src/batching/gpu_preprocessing.rs
pub struct UntypedPhaseIndirectParametersBuffers {
    // 间接参数缓冲区
}
```

## 8. 可见性系统

### 8.1 边界计算

```rust
// crates/bevy_sprite/src/lib.rs
pub fn calculate_bounds_2d(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    images: Res<Assets<Image>>,
    atlases: Res<Assets<TextureAtlasLayout>>,
    new_mesh_aabb: Query<(Entity, &Mesh2d), ...>,
    new_sprite_aabb: Query<(Entity, &Sprite, &Anchor), ...>,
) {
    // 计算 Mesh2d 的 AABB
    // 计算 Sprite 的 AABB
}
```

### 8.2 视锥剔除

使用 `Aabb` 组件进行视锥剔除：

```rust
commands.spawn((
    Sprite::from_image(image_handle),
    Transform::default(),
    // 自动计算 AABB
));
```

禁用自动 AABB 计算：

```rust
commands.spawn((
    Sprite::from_image(image_handle),
    Transform::default(),
    NoAutoAabb,  // 禁用自动计算
    Aabb::default(),  // 手动设置
));
```

## 9. 纹理切片 (Texture Slicing)

### 9.1 TextureSlice

支持 9-patch 切片：

```rust
// crates/bevy_sprite/src/texture_slice.rs
#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component, Default, Debug, Clone)]
pub struct TextureSlice {
    /// 边框矩形
    pub border: BorderRect,
    /// 缩放模式
    pub scale_mode: SliceScaleMode,
}
```

### 9.2 使用示例

```rust
commands.spawn((
    Sprite::from_image(image_handle),
    TextureSlice {
        border: BorderRect::new(10.0, 10.0, 10.0, 10.0),
        scale_mode: SliceScaleMode::Stretch,
    },
));
```

## 10. 2D 文本渲染

### 10.1 Text2d 组件

```rust
// crates/bevy_sprite/src/text2d.rs (feature: bevy_text)
#[derive(Component, Debug, Default, Clone, Reflect)]
#[reflect(Component, Default, Debug, Clone)]
pub struct Text2d {
    pub text: String,
    pub font: Handle<Font>,
    pub font_size: f32,
    pub color: Color,
}
```

### 10.2 使用示例

```rust
commands.spawn((
    Text2d::new("Hello, World!"),
    TextFont {
        font: asset_server.load("font.ttf"),
        font_size: 24.0,
        ..default()
    },
    TextColor(Color::WHITE),
    Transform::from_xyz(0.0, 0.0, 0.0),
));
```

## 11. 拾取系统 (Picking)

### 11.1 SpritePickingPlugin

```rust
// crates/bevy_sprite/src/picking_backend.rs (feature: bevy_picking)
pub struct SpritePickingPlugin;

impl Plugin for SpritePickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, sprite_picking...);
    }
}
```

### 11.2 使用示例

```rust
commands.spawn((
    Sprite::from_image(image_handle),
    Transform::default(),
    Pickable::default(),
    On::<Pointer<Click>>::run(|| {
        println!("Sprite clicked!");
    }),
));
```

## 12. 性能优化建议

### 12.1 使用纹理图集

将多个小图合并为纹理图集，减少纹理切换：

```rust
// 使用 TextureAtlasBuilder 创建图集
let mut builder = TextureAtlasBuilder::default();
builder.add_texture(id1, &image1);
builder.add_texture(id2, &image2);
let (atlas, sources) = builder.finish().unwrap();
```

### 12.2 启用批处理

确保精灵可以自动批处理：
- 使用相同的材质/纹理
- 避免频繁修改 Transform

### 12.3 视锥剔除

利用 AABB 进行视锥剔除：

```rust
// 自动计算 AABB
app.add_plugins(SpritePlugin);

// 手动控制 AABB
commands.spawn((
    Sprite::from_image(image_handle),
    NoFrustumCulling,  // 禁用视锥剔除
));
```

### 12.4 使用 SpriteMesh 进行高级效果

需要自定义 alpha 混合时使用 `SpriteMesh`：

```rust
commands.spawn((
    SpriteMesh {
        image: image_handle,
        alpha_mode: SpriteAlphaMode::Blend,
        ..default()
    },
    Transform::default(),
));
```

## 13. 与 LVGL 的对比

| 特性 | Bevy Sprite | LVGL Widget |
|------|-------------|-------------|
| 架构 | ECS | 对象树 |
| 渲染 | GPU 批处理 | CPU/GPU 混合 |
| 动画 | 纹理图集/程序化 | 属性动画 |
| 文本 | 独立系统 | 内置支持 |
| 拾取 | 射线检测 | 事件冒泡 |
| 缩放 | 多种模式 | 固定比例 |
| 切片 | 9-patch | 类似支持 |

## 14. 总结

Bevy 的精灵系统特点：

1. **ECS 架构**：组件化设计，灵活组合
2. **自动批处理**：GPU 优化渲染
3. **纹理图集**：高效的精灵动画
4. **多种缩放模式**：适应不同需求
5. **集成渲染管线**：与 3D 渲染统一
6. **可选功能**：文本、拾取等按需启用

适用于需要高性能 2D 渲染的游戏和应用。

---

**文档版本**: v1.0  
**分析日期**: 2026-04-14  
**Bevy 版本**: 0.15+  
**作者**: FeatherOS Team
