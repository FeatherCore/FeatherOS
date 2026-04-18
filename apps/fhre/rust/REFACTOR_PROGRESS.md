# FHRE 单相机架构重构进度

## 目标

将当前「3D 透视相机 + UI 正交相机」双相机架构，重构为「唯一 3D 透视相机」单相机架构。

### 架构理念

```
3D 主世界（唯一 3D 透视摄像机）
    │
    │ 摄像机采集（MVP 变换）
    │ 摄像机 target 自动追踪幕布位置
    ▼
2D 屏幕幕布（Screen，3D 世界中的投影平面）
    │  - 含 Transform3D，可操作（拉近拉远、旋转等）
    │  - 3D 控件：经 MVP 投影到幕布坐标
    │  - 2D UI：直接用幕布像素坐标绘制
    │  - 默认与呈现窗口 1:1 重叠
    ▼
呈现窗口（X11 / FB / 用户定义，不属于 FHRE）
```

**核心原则**：
1. 主世界是纯 3D 的，只有一个 3D 透视摄像机
2. 2D 屏幕幕布是摄像机投影平面，所有内容统一绘制到幕布上
3. 幕布是 3D 世界中的对象，含 Transform3D，可操作
4. 摄像机 target 自动追踪幕布位置，幕布移动时摄像机跟随
5. 呈现窗口与 FHRE 主世界独立，由用户定义
6. 移除 UiCamera，不再需要独立的正交相机

---

## 修改清单

### Phase 1: 移除 UiCamera 资源

| # | 文件 | 修改内容 | 状态 |
|---|------|----------|------|
| 1.1 | `resources/camera.rs` | 移除 `UiCamera` struct 及其 impl | ✅ |
| 1.2 | `resources/mod.rs` | 移除 `UiCamera` 的 re-export | ✅ |
| 1.3 | `lib.rs` | 移除 `UiCamera` 的 pub use | ✅ |

### Phase 2: 修改 CameraPlugin

| # | 文件 | 修改内容 | 状态 |
|---|------|----------|------|
| 2.1 | `camera/mod.rs` | 移除 UiCamera 注册，只注册 Camera | ✅ |
| 2.2 | `camera/mod.rs` | 更新文档注释 | ✅ |

### Phase 3: 重构 Extract 系统（核心）

| # | 文件 | 修改内容 | 状态 |
|---|------|----------|------|
| 3.1 | `extract/extract.rs` | 移除 `UiCamera` import | ✅ |
| 3.2 | `extract/extract.rs` | 重构 `extract_renderable_components`：统一使用 Camera 的 MVP，移除双 View | ✅ |
| 3.3 | `extract/extract.rs` | 重构 `extract_buttons`：移除独立正交 View，直接用幕布坐标 | ✅ |
| 3.4 | `extract/extract.rs` | 移除 `ui_camera_to_view_bundle` 函数 | ✅ |
| 3.5 | `extract/extract.rs` | 移除 `create_default_view` 函数 | ✅ |
| 3.6 | `extract/extract.rs` | 更新 `default_extract_schedule` | ✅ |
| 3.7 | `extract/extract.rs` | 重构 `extract_sprites`：移除独立正交 View | ✅ |

### Phase 4: 更新 DefaultPlugins 文档

| # | 文件 | 修改内容 | 状态 |
|---|------|----------|------|
| 4.1 | `plugin/default_plugins.rs` | 更新 CameraPlugin 描述 | ✅ |

### Phase 5: 更新示例代码

| # | 文件 | 修改内容 | 状态 |
|---|------|----------|------|
| 5.1 | `examples/fhre/rust/src/lib.rs` | 移除 `UiCamera` import 和 `insert_resource(UiCamera::orthographic())` | ✅ |

### Phase 6: 验证编译

| # | 内容 | 状态 |
|---|------|------|
| 6.1 | FHRE 框架编译通过 | ✅ |
| 6.2 | 示例编译通过 | ✅ |

### Phase 7: 清除残留正交投影引用

| # | 文件 | 修改内容 | 状态 |
|---|------|----------|------|
| 7.1 | `resources/camera.rs` | 移除 `ProjectionType::Orthographic` 变体及 `is_orthographic()` | ✅ |
| 7.2 | `resources/camera.rs` | 更新注释 `Projection type (always Perspective)` | ✅ |
| 7.3 | `extract/extract.rs` | `camera_to_view_bundle` 移除 `Orthographic` match 分支，`orthographic: false` | ✅ |
| 7.4 | `render_world/view.rs` | 移除 `View::new_2d`，`View::default` 改用 `new_3d` | ✅ |
| 7.5 | `render_world/view.rs` | `ViewBundle::new_screen/new_texture` 改用 `View::new_3d` | ✅ |
| 7.6 | `render_world/world.rs` | `RenderWorld::create_default_view` 改为创建 3D 透视 View | ✅ |
| 7.7 | `node/node3d.rs` | `Camera3D` 组件移除 `orthographic`/`orthographic_size` 字段及方法 | ✅ |

### Phase 8: PrimaryScreen 扩展为 3D 幕布对象

| # | 文件 | 修改内容 | 状态 |
|---|------|----------|------|
| 8.1 | `resources/screen.rs` | `PrimaryScreen` 添加 `transform: Transform3D` 字段 | ✅ |
| 8.2 | `resources/screen.rs` | 默认幕布位置 `(width/2, height/2, 0)` 对齐 Camera target | ✅ |
| 8.3 | `resources/screen.rs` | 添加 `with_transform()`、`move_closer()`、`move_further()`、`position()` 方法 | ✅ |
| 8.4 | `resources/screen.rs` | 更新模块文档，描述幕布架构 | ✅ |

### Phase 9: 摄像机与幕布位置联动

| # | 文件 | 修改内容 | 状态 |
|---|------|----------|------|
| 9.1 | `camera/mod.rs` | CameraPlugin 从 PrimaryScreen.position() 构建 Camera target | ✅ |
| 9.2 | `extract/extract.rs` | `extract_renderable_components` 读取幕布位置，传入 View 构建 | ✅ |
| 9.3 | `extract/extract.rs` | `camera_to_view_bundle` 使用幕布位置作为 look-at target（覆盖 Camera.target） | ✅ |
| 9.4 | `extract/extract.rs` | `create_perspective_view_with_canvas` 使用幕布位置 | ✅ |

### Phase 10: 最终验证

| # | 内容 | 状态 |
|---|------|------|
| 10.1 | FHRE 框架编译通过 | ✅ |
| 10.2 | 示例编译通过 | ✅ |
| 10.3 | 全局无 `UiCamera`/`Orthographic`/`new_2d`/`is_orthographic` 残留 | ✅ |

### Phase 11: 渲染流程审查修复

| # | 文件 | 修改内容 | 状态 |
|---|------|----------|------|
| 11.1 | `resources/camera.rs` | `Camera::default_3d` target 从 `height/3` 改为 `height/2`（对齐幕布中心） | ✅ |
| 11.2 | `resources/camera.rs` | 更新 Camera struct 文档：说明 target 在 extract 阶段被幕布位置覆盖 | ✅ |
| 11.3 | `render_world/world.rs` | `RenderWorld::create_default_view` target 从 `height/3` 改为 `height/2` | ✅ |
| 11.4 | 全部 | NuttX build.sh 构建验证通过 | ✅ |

---

## 已完成的修改摘要

### 修改的文件

1. **`resources/camera.rs`**
   - 移除 `UiCamera` struct 及其所有 impl
   - 移除 `ProjectionType::Orthographic` 变体及 `is_orthographic()`
   - 更新模块文档，描述单相机架构

2. **`resources/screen.rs`**
   - `PrimaryScreen` 添加 `transform: Transform3D` 字段
   - 默认幕布位置 `(width/2, height/2, 0)` 对齐 Camera target
   - 添加 `with_transform()`、`move_closer()`、`move_further()`、`position()` 方法
   - 更新模块文档，描述幕布架构

3. **`resources/mod.rs`**
   - 从 re-export 中移除 `UiCamera`

4. **`lib.rs`**
   - 从 pub use 中移除 `UiCamera`

5. **`camera/mod.rs`**
   - 移除 `UiCamera` import 和注册逻辑
   - CameraPlugin 从 PrimaryScreen.position() 构建 Camera target
   - Camera 位置 = (canvas.x, canvas.y, canvas.z + 600)
   - Camera target = canvas.position
   - 更新文档注释

6. **`extract/extract.rs`**（核心变更）
   - 移除 `UiCamera` import
   - `extract_renderable_components`: 读取幕布位置，传入 View 构建
   - `camera_to_view_bundle`: 使用幕布位置作为 look-at target，Camera target 自动追踪幕布
   - `create_perspective_view_with_canvas`: 使用幕布位置
   - `extract_buttons`: 移除独立正交 View，直接用幕布像素坐标
   - `extract_sprites`: 移除独立正交 View，直接用幕布坐标
   - 移除 `create_default_view`、`ui_camera_to_view_bundle`、`create_perspective_view` 函数

7. **`render_world/view.rs`**
   - 移除 `View::new_2d`
   - `View::default` 改用 `new_3d(800, 600, 45)`
   - `ViewBundle::new_screen/new_texture` 改用 `View::new_3d`

8. **`render_world/world.rs`**
   - `RenderWorld::create_default_view` 改为创建 3D 透视 View

9. **`node/node3d.rs`**
   - `Camera3D` 组件移除 `orthographic`/`orthographic_size` 字段及方法

10. **`plugin/default_plugins.rs`**
    - 更新 CameraPlugin 描述

11. **`examples/fhre/rust/src/lib.rs`**
    - 移除 `UiCamera` import 和 `insert_resource(UiCamera::orthographic())`

---

## 渲染管线变更详情

### 变更前（双相机）

```
Extract 阶段:
  1. 检测 3D 对象 → 创建透视 View (Camera) → 渲染 Cube/SoccerBall
  2. 总是 → 创建正交 View (UiCamera) → 渲染 Button/Sprite
```

### 变更后（单相机 + 幕布联动）

```
Extract 阶段:
  1. 读取 PrimaryScreen.transform.position → canvas_pos
  2. 创建唯一 3D 透视 View (Camera target = canvas_pos)
  3. 3D 对象 (Cube/SoccerBall) → Camera MVP → 幕布坐标 → 绘制
  4. 2D UI (Button/Sprite) → 直接幕布像素坐标 → 绘制
  （两者在同一个幕布坐标系上统一）

幕布操作效果:
  - canvas.move_closer(delta) → 幕布靠近相机 → 呈现窗口放大
  - canvas.move_further(delta) → 幕布远离相机 → 呈现窗口缩小
  - Camera target 自动追踪幕布位置
```

### 幕布坐标系

```
(0,0) ─────────────── (width, 0)
  │                       │
  │    幕布平面           │
  │    原点左上角          │
  │    Y 轴向下            │
  │    单位：像素          │
(0,height) ───────── (width, height)
```

3D 对象经 Camera MVP 投影后输出到此坐标系。
2D UI 的 Transform2D 直接使用此坐标系。

### 默认位置关系

```
Camera:  position = (width/2, height/2, 600)   ← 在幕布正后方 600 单位
         target   = (width/2, height/2, 0)      ← 看向幕布中心
Canvas:  position = (width/2, height/2, 0)      ← 幕布在 z=0 平面

当 canvas.move_closer(100) 后:
Canvas:  position = (width/2, height/2, 100)
Camera:  target 自动追踪到 (width/2, height/2, 100)
→ 幕布更靠近相机，呈现窗口显示放大效果
```

---

## 保留的基础设施

以下底层设施保留，不删除：

- `Mat4::orthographic_rh()` — 底层数学库函数，正交投影矩阵计算
- `View.orthographic: bool` — View 结构体的属性标记字段
- `ProjectionType` enum — 仍保留为 enum 结构（仅含 Perspective 变体），便于未来扩展

---

## 渲染流程验证结果（Phase 11 审查）

### 完整流程追踪（以 640×480 为例）

```
1. 初始化
   App::new(640, 480)
     → PrimaryScreen::new(640, 480)
       → transform = Transform3D::from_position(320, 240, 0)  // 幕布中心
     → CameraPlugin::build()
       → canvas_pos = (320, 240, 0)
       → Camera position = (320, 240, 600)  // 幕布正后方 600
       → Camera target   = (320, 240, 0)    // 看向幕布中心

2. 用户 Setup
   Cube:  Transform3D::from_position(320, 160, 0)  // 3D 世界坐标
   Button: Transform2D::from_position(320, 400)     // 幕布像素坐标

3. Extract 阶段
   a. 读取 canvas_pos = PrimaryScreen.position() = (320, 240, 0)
   b. 构建 View:
      view       = look_at_rh((320,240,600), (320,240,0), (0,1,0))
      projection = perspective_rh(45°, 640/480, 0.1, 1000)
      VP         = projection × view
   c. Cube 投影:
      本地顶点 → 旋转 → + Transform3D.position → 世界坐标
      世界坐标 × VP → NDC → world_to_screen() → 幕布像素坐标
      生成 DrawPolygon 命令（幕布像素坐标）
   d. Button 绘制:
      Transform2D.position 直接作为 Rect 坐标
      生成 DrawRect 命令（幕布像素坐标）

4. 坐标系统一性
   ✅ Cube 经 MVP 后输出 (0,0)~(640,480) 像素坐标 — 和 Button 相同
   ✅ world_to_screen: NDC → (ndc_x+1)/2 × width → 像素坐标
   ✅ 两者都在同一个幕布坐标系上

5. 幕布联动
   ✅ Camera target 被 canvas_pos 覆盖（camera_to_view_bundle 中）
   ✅ 幕布移动时 Camera 自动跟随
```

### 发现并修复的问题

| 问题 | 位置 | 修复 |
|------|------|------|
| `Camera::default_3d` target 为 `height/3` | camera.rs | 改为 `height/2`（对齐幕布中心） |
| `RenderWorld::create_default_view` target 为 `height/3` | world.rs | 改为 `height/2`（对齐幕布中心） |
| Camera struct 文档未说明 target 被覆盖 | camera.rs | 添加说明 |
