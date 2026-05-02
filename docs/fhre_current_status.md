# FHRE 当前实现状态

日期：2026-05-02

本文是 FHRE 当前代码状态的盘点文档，不替代 `fhre_implementation_approach.md` 的演进日志。它重点回答三个问题：

- 目前已经实现了什么。
- 目前还没有实现什么。
- 下一阶段应该继续实现什么。

当前基线可以概括为：

```text
FHRE V4.7 = no_std + alloc 的轻量图形/游戏引擎底座
          + 软件 Surface renderer
          + DrawChain -> 2D Packet -> Executor -> Descriptor Ring/Fence 的硬件接管契约
          + Hybrid parallel run scheduler 的确定性并行提交模型
          + 默认关闭的真实 EGL/OpenGL worker 并行后端
          + Image/Codec pipeline plan/job/mock executor
          + 默认关闭的 sim OpenGL packet probe / EGL worker 边界
          + NuttX demo runtime 回归入口

但它还不是：
          - 真实 DMA2D/PXP/VG-Lite/GPU 后端
          - 默认 OpenGL/GLX/EGL present 路径
          - 完整 PNG/JPEG/SVG/TTF 引擎
          - 完整 Bevy 级 ECS/调度器
          - 完整 3D GPU renderer
```

## 一、项目定位

FHRE 的定位是 FeatherOS 上的轻量 Rust 图形引擎底座，目标运行环境是 NuttX + Rust，目标设备是高性能 MCU、低功耗 MPU、智能手表、MP4、小屏手机和复古掌机这类资源受限平台。

核心约束：

- `no_std` crate，允许 `alloc`。
- 不依赖外部 Rust crate。
- 不使用宏生成核心 ECS/UI 逻辑。
- 优先 RGB565 / 小 framebuffer / 固定容量数据结构。
- 支持纯软件渲染，并给 DMA2D、PXP、VG-Lite、简单 GPU、OpenGL ES / OpenCL ES 类平台预留接管边界。
- WING 目前只作为 FHRE 的上层消费者和回归目标，不在本阶段拥有私有硬件后端、私有 codec 或私有 framebuffer 路径。

## 二、代码模块地图

当前 FHRE Rust crate 位于 `apps/fhre/rust`，公开 API 主要从 `src/lib.rs` 和 `src/prelude/mod.rs` re-export。

| 模块 | 当前职责 | 当前状态 |
| --- | --- | --- |
| `color` / `geom` / `math` | `Color`、`PixelFormat`、`Rect/Point/Size`、Fixed16 工具 | 已实现基础能力 |
| `ecs` / `schedule` | `EntityWorld`、`ComponentStorage`、`ResourceSlot`、固定容量 `Schedule` | 已实现最小无宏 ECS，不是完整 Bevy |
| `event` / `input` | 固定容量事件队列、pointer/key、gesture recognizer | 已实现 demo 可用输入模型 |
| `runtime` / `dirty` | frame clock、frame stats、dirty region/tracker、present stats | 已实现 demo/runtime 统计基础 |
| `draw` | `DrawCommand`、`DrawList`、样式结构、depth sort、draw-chain planner | 已实现核心绘制命令与 V4 run 调度 |
| `surface*` | 软件 framebuffer renderer、像素读写、图元、图片、文本、SVG、layer/mask | 已实现一批软件路径和 fallback |
| `backend` | `RenderBackend` / `ParallelRenderBackend`、能力位、stats、draw-chain contract、codec pipeline/job/mock codec | 已实现 V4.7 抽象与统计 |
| `accel2d` | 2D packet、framebuffer/source descriptor、executor、submission ring、fence | 已实现平台无关契约层 |
| `mock_accel` | `MockAccelBackend` / `MockParallelAccelBackend`，通过 executor/packet/ring 执行到 `Surface` | 已实现同步与并行测试后端 |
| `opengl_accel` | `sim-opengl` / `sim-opengl-egl` feature 下的 OpenGL-like packet mapping probe 与 EGL worker | 已实现默认关闭的 probe 与真实 EGL/OpenGL worker 后端 |
| `image` / `png` / `jpeg` | FRAW/PNG/JPEG 识别、轻量 decode、ImageCache、placeholder | 已实现基础子集与占位策略 |
| `svg` | 有界 SVG document/path parser、cache、fallback/unsupported 统计 | 已实现基础 path/shape/style 子集 |
| `ttf` / `glyph` / `text` | TTF/OTF 检查、最小 raster、glyph cache/run cache、文本布局 | 已实现基础 Latin/TTF 路径 |
| `scene` | Camera、Transform3D、mesh/textured mesh emit 到 DrawList | 已实现轻量 3D/伪 3D 投影入口 |
| `platform` | framebuffer/input trait | 已实现最小 trait |
| `apps/examples/common/fhre_nuttx_runtime.rs` | NuttX framebuffer/input/resource glue | 已实现 demo 共享 runtime |

## 三、目前已实现

### 1. Crate 与构建约束

- FHRE crate 已保持 `#![no_std]`，并显式使用 `extern crate alloc`。
- `Cargo.toml` 没有外部依赖。
- release profile 已设置 `panic = "abort"`、LTO、单 codegen unit 和 `opt-level = "z"`，符合 MCU/MPU 体积优先方向。
- `fhre_build.sh` 与 `wing_build.sh` 已作为 NuttX 回归入口使用。

### 2. 无宏轻量 ECS / Runtime 基础

已实现：

- `EntityWorld<N>`：固定容量 entity 分配/释放。
- `ComponentStorage<T, N>`：固定容量组件存储。
- `ResourceSlot<T>`：轻量 resource 容器。
- `Schedule<C, N>`：固定容量 system 函数列表。
- `GameRuntime` / `FhreRuntime`：把 world、input queue、dirty tracker、frame clock 组合起来。
- `FrameClock` / `FramePolicy` / `FrameStats`：帧率、late/dropped frame、draw/present/input 时间统计。
- `EventQueue<T, N>`、`InputQueue<N>`、pointer/key/gesture 基础类型。

未做到 Bevy 级别：

- 没有 query borrow system。
- 没有 archetype/table ECS。
- 没有 change detection。
- 没有 schedule graph、stage、parallel executor。
- 没有 macro derive component/resource。

这符合当前“无宏、轻量、声明式 ECS”的目标。

### 3. DrawCommand / DrawList / 软件渲染

已实现的命令类别：

- 清屏、clip、矩形填充、圆角矩形、渐变填充、styled fill。
- circle、line、wide line、styled/dashed line。
- border、shadow、arc。
- text、label、selection/underline/strikethrough 的基础绘制。
- image、fit、tint、styled image、clip radius。
- SVG icon/document。
- mask rect、bitmap mask、push/pop mask。
- layer fallback、begin/end layer、opacity/recolor/blur。
- blur fallback。
- triangle、gradient triangle、textured triangle。

软件 `Surface` 已具备：

- framebuffer 直接写像素。
- clip/mask 后再 alpha blend。
- RGB565 等格式的像素读写。
- opaque fill / alpha fill 的快速路径。
- 图片采样、stretch/contain/cover、tint、圆角裁剪。
- 有界 layer scratch、shadow/mask/blur fallback。
- SVG path/document 的基础 raster。
- glyph resolver / glyph run resolver / kerning resolver 接入。

限制：

- 软件 renderer 仍以简单 scanline/像素循环为主，复杂路径未系统优化。
- mask/layer/vector/text/3D 还没有进入 2D packet 硬件提交。
- blur/shadow/layer 仍是 fallback 性质，适合小屏验证，不是高性能最终形态。

### 4. DrawChain 与硬件接管契约

已实现到 V4.4：

- `DrawChainOpPayload` 已为硬件后端提供固定字段：
  - `FillRect { rect, color }`
  - `Image { rect, image, opacity, fit, tint }`
  - `Clip { clip }`
  - `Mask { spec }`
  - `Layer { rect, spec }`
  - `None`
- `DrawChainRunContract` / `DrawChainRunDescriptor` 已表达 run 的最小状态：
  - command span
  - clip/mask/layer 标记
  - bounds
  - op kind 序列
- marker 语义已清理：
  - 合成 clip marker `command_len = 0`。
  - 真实可软件回放命令仍保留 command span。
- run planner 已能按 backend capabilities 拆分：
  - alpha/non-alpha 混合能力。
  - clip/mask/layer 混合能力。
  - `max_chain_ops` 容量。
  - stateful mask/layer run 的整体回放边界。
- fallback 语义已分层：
  - stateless run 失败后可尝试单 op。
  - mask/layer stateful run 失败时整体软件回放。
  - 失败只影响当前 run/op，不污染前后绘制。

已实现的测试后端：

- `ProbeChainBackend`：只读 payload + contract，不反查 `DrawCommand`。
- `MockAccelBackend`：通过 executor/packet/ring 真写像素，并和纯软件 `Surface` 做像素一致性测试。
- `MockParallelAccelBackend`：queue 阶段不写像素，flush 阶段按 ring/fence 顺序执行，用于验证 disjoint run、software-while-hw 与 overlap/stateful barrier。
- `SimOpenGlAccel2dExecutor`：在 `sim-opengl` feature 下验证 packet 可映射到 OpenGL-like scissor/quad/texture 命令；当前不链接真实 GL 库。
- `EglParallelAccelBackend`：在 `sim-opengl-egl` feature 下通过 C/FFI EGL/OpenGL worker 真实异步执行 `SetClip/Fill/Image` packet；运行期不可用或失败时稳定 fallback/软件回放。

### 5. 2D Packet / Executor / Descriptor Ring / Fence / EGL Worker

已实现：

- `Accel2dFramebufferDescriptor`
  - data pointer
  - len
  - width/height
  - stride
  - bpp
  - `PixelFormat`
  - clip
- `Accel2dImageSourceDescriptor`
  - data pointer
  - len
  - width/height
  - stride
  - `ImageFormat`
- `Accel2dCommandList<N>`
  - 固定容量，无堆分配。
  - `SetClip`
  - `Fill`
  - `Image`
- `build_accel_2d_command_list()` / `build_accel_2d_command_list_for_framebuffer()`
  - 只读 `DrawChainOpPayload + DrawChainRunContract + framebuffer/source descriptor`。
  - 不反查 `DrawCommand`。
  - build 阶段处理 missing image、empty framebuffer、overflow、payload mismatch、mask/layer/fallback 等错误。
- `Accel2dExecutor`
  - no_std 友好的 packet 执行抽象。
  - `Accel2dExecutorCapabilities` 表达 fill/image/clip、target/source format、sync/async fence 能力。
  - `Accel2dExecutorError` 将 executor 失败稳定映射为 unsupported/fallback/overflow/submit/fence/ring 错误。
- `SurfaceAccel2dExecutor`
  - 当前 mock 使用的 offscreen/readback 等价执行器。
  - 只消费 `Accel2dCommandList`，不反查 `DrawCommand`。
  - fill、alpha fill、clip、image 的输出与软件 `Surface` 基线一致。
- `Accel2dSubmissionRing<RUNS, CMDS>`
  - 固定容量 submission ring。
  - `queue()`、`flush()`、`complete_fence()`。
  - `Accel2dFence`、`Accel2dFenceState`。
  - stable errors：overflow、empty submission、invalid fence、not submitted、already completed。
- `MockAccelBackend`
  - 内部流程为 `build packet -> executor queue -> flush -> execute submitted packet -> complete fence`。
  - 默认同步完成，保持当前 draw planner 行为稳定。
  - 支持 fill、alpha fill、clip、image blit/blend。
  - 不支持 mask/layer/fallback payload 的链内硬件提交。
- `MockParallelAccelBackend`
  - 内部流程为 `build packet -> queue ring`，在 barrier/final flush 时执行 packet 并 complete fence。
  - 允许 disjoint hardware candidate run 延迟完成。
  - pending hardware run 与 disjoint software fallback run 可确定性重叠调度。
  - overlap、stateful mask/layer、marker 边界会触发 barrier，最终像素与纯软件一致。
- `SimOpenGlAccel2dExecutor`（默认关闭）
  - Cargo feature：`sim-opengl`。
  - packet mapping：`SetClip -> Scissor`，`Fill -> FillQuad`，`Image -> TextureQuad`。
  - 当前执行仍委托 `SurfaceAccel2dExecutor` 做 offscreen/readback 等价结果，保证不引入 GLX/EGL/OpenGL 链接依赖。
- `EglParallelAccelBackend<RUNS, OPS>`（默认关闭）
  - Cargo feature：`sim-opengl-egl`。
  - C worker 线程独占 EGL context / pbuffer / FBO，Rust 主线程只提交 packet 与等待 barrier。
  - `queue_draw_chain_with_contract()` 在 build packet 成功后把 run 复制给 worker 并立即返回 `Queued`，主线程可继续软件绘制不相交 run。
  - `flush_pending_draw_chain()` 等待 worker readback，并按提交顺序写回 framebuffer；GL 运行期失败时使用 Rust 侧保存的 pending packet 软件回放，保证最终像素与软件基线一致。
  - 真实 worker 默认不编入；构建时显式设置 `FHRE_SIM_OPENGL=egl` 或 `FHRE_EGL_WORKER_ENABLE=1` 时才启用真实 worker，避免普通 cargo test / NuttX build 被 host GL 状态影响。
  - 未来真实 GLX/EGL/OpenGL ES 后端只需要替换 execute 层，不改变 `DrawChain -> Packet -> Executor` 的上游契约。
- `EglOpenGlAccel2dExecutor`（默认关闭）
  - Cargo feature：`sim-opengl-egl`。
  - 通过 `egl_shim.c` 复用同一 C/FFI 边界；默认构建 gate 未打开时编入 unavailable stub。
  - 显式启用后可作为 packet/executor 层的 EGL worker 探针；不复用 NuttX sim 的 X11 `Display`。

统计已实现：

- `draw_chain_candidates`
- `draw_chain_ops`
- `draw_chain_submitted`
- `draw_chain_fallbacks`
- `draw_chain_unsupported`
- `draw_chain_overflows`
- `draw_chain_runs`
- `draw_chain_hw_runs`
- `draw_chain_sw_runs`
- `draw_chain_splits`
- `draw_chain_parallel_hints`
- `draw_chain_parallel_queued`
- `draw_chain_parallel_completed`
- `draw_chain_parallel_barriers`
- `draw_chain_parallel_software_runs`
- `draw_chain_parallel_fallbacks`
- `accel2d_ring_submissions`
- `accel2d_ring_flushes`
- `accel2d_fences_issued`
- `accel2d_fences_completed`
- `accel2d_ring_overflows`

### 6. 图片资源、ImageCache 与 placeholder

已实现：

- `ImageView`：硬件友好的 width/height/stride/format/data/len view。
- `ImageResolver`：绘制热路径只查 resolver。
- builtin image：
  - `IMAGE_SWATCH`
  - `IMAGE_MASK_DOT`
- `ImageCache`
  - max slots / max bytes。
  - pinned slot。
  - LRU-like eviction。
  - `view()`。
  - `get_or_load()` 保持兼容：可返回 rendered 或 placeholder view。
  - `get_or_load_result()` 严格 API：placeholder 会转成错误。
  - `prewarm()` / `prewarm_result()` / `prewarm_with_placeholder()`。
- `ResourceDecodeResult`
  - `Rendered(ImageView)`
  - `Placeholder(ImageView, ImageDecodeErrorKind)`
  - `Failed(CodecErrorKind)`
- missing resource 统计已与 decode failure 分开：
  - missing 计入 load/missing/placeholder。
  - `decode_failures` 只表示已有 bytes 进入 decoder 后失败。

当前策略：

- 不做负缓存。
- 热路径应该使用预热后的 cache/resolver。
- placeholder 主要用于开发期、资源缺失、格式不支持时保持画面连续。

### 7. Codec pipeline/job/mock executor

已实现：

- `CodecAcceleratorCapabilities`
  - png/jpeg/fraw/ttf/svg capability。
  - max stages。
- `CodecPipelinePlan<STAGES>`
  - 固定容量 stage list。
  - overflow/fallback/unsupported 统计。
- `CodecStageKind`
  - Read / Inspect / Header / Entropy / Parse / Transform / Raster / ColorConvert / Pack / CacheInsert / Fallback。
- `CodecPipelineJob<STAGES>`
  - `Planned -> Prepared -> Submitted -> Completed`
  - `Fallback`
  - `Failed`
- `CodecPipelineBackend` trait。
- `MockCodecBackend`
  - capability 验证。
  - prepare/submit/complete 调用计数。
  - submit/complete failure 模拟。
  - `CodecPipelineJobToken` 模拟提交 token/fence。

已支持 plan 的资源类型：

- FRAW。
- JPEG。
- PNG。
- SVG。
- TTF。

限制：

- Mock codec backend 不读取资源字节。
- 没有真实 JPEG/PNG/SVG/TTF 硬件 decoder job。
- pipeline 目前主要是“可接管状态机和统计”，不是完整异步解码执行层。

### 8. FRAW / PNG / JPEG / SVG / TTF 子集

FRAW：

- 已实现自定义 `FHREIMG1` header。
- 支持 RGB565/RGB888/RGBA8888/A8。
- 支持 inspect、decode、pipeline plan。

PNG：

- 已实现自研轻量 PNG parser/decoder。
- 支持基础 chunk inspect、IDAT inflate、filter、palette/alpha 等子集。
- 对 unsupported bit depth/color type/compression/filter/interlace 等返回稳定错误。
- 不是完整 libpng 替代品。

JPEG：

- 已实现自研轻量 JPEG inspect/decode。
- 具备 baseline/progressive、CMYK、EXIF orientation 等部分路径。
- 对 restart、复杂 progressive、unsupported sampling/table/scan 等返回稳定错误。
- 不是完整 libjpeg 替代品。

SVG：

- 已实现有界 SVG document/path parser。
- 支持 path、rect、circle、ellipse、line、polyline、polygon、use 的基础转 path。
- 支持基础 fill/stroke、opacity、transform、clip/mask/filter 统计与有限 fallback。
- 有固定容量 `DefaultSvgDocument` / `SvgDocumentCache`。
- 不支持完整 CSS、复杂 clip/mask/filter、完整 SVG 规范。

TTF/OTF：

- 已实现 SFNT table 识别。
- 支持 TrueType glyf / OpenType CFF 的基础识别。
- 有 glyph raster options、glyph cache、glyph run cache。
- 支持 Latin-oriented shaping、基础 ligature/GSUB/GPOS/kern 命中统计。
- 复杂脚本 shaping、完整 OpenType layout、hinting、完整 CFF 都未完成。

### 9. Text / Glyph / SVG cache

已实现：

- `GlyphCache`：固定容量 glyph bitmap/cache。
- `GlyphRunCache`：固定容量 glyph run cache，暴露 hit/miss/insert/evict/overflow/slots。
- `GlyphResolver`、`GlyphIdResolver`、`GlyphRunResolver`、`KerningResolver`。
- `TextLayout<N>`：固定容量文本行布局。
- `SvgCache` / `SvgDocumentCache`：固定容量 SVG path/document cache。
- `RenderStats` 已暴露 glyph/svg/text 的 fallback、overflow 和 feature counters。

限制：

- 文本布局不是完整排版引擎。
- fallback 字体/缺字逻辑仍偏 demo/调试用途。
- SVG cache 主要服务小图标/小文档，不适合直接吞复杂桌面级 SVG。

### 10. 3D / Scene

已实现：

- `Vec3`
- `Transform3D`
- `Camera`
- `Projection`
- `RenderNode`
- `MeshRef`
- `TexturedMeshRef`
- `Vertex3D`
- `TexturedVertex3D`
- mesh emit 到 `DrawList`。
- textured triangle 通过软件 `Surface` 采样绘制。

当前更准确地说是轻量 3D/伪 3D：

- 没有 Z-buffer。
- 没有 GPU pipeline。
- 没有 material/shader。
- 没有 3D 资源管理。
- painter sort 只是简单 depth sort。

### 11. NuttX demo runtime

已实现：

- `/dev/fb0` framebuffer 打开。
- video/plane info ioctl。
- 双 framebuffer page pan 优先。
- 没有 pan buffer 时使用静态 backbuffer copy。
- dirty present 前同步可见页到绘制页，避免 dirty 区域外旧内容闪烁。
- `/dev/input0` touch 和 `/dev/kbd` keyboard 非阻塞读取。
- move coalescing，避免输入队列被 move flood 塞满。
- `NuttxResourceLoader` 从 ROMFS/resource path 读取资源。
- `now_us()` / `sleep_remaining()` 接入 frame clock。
- `poll_sim_events()` 已改为 no-op，因为 NuttX sim 自己的 `sim_loop_task` 已负责 X11 event/update pumping，避免两个线程同时调用 Xlib 导致 demo 运行一段时间后卡住。

## 四、目前未完成

### 1. 真实硬件 2D/GPU 后端未完成

还没有实现：

- STM32 DMA2D 后端。
- NXP PXP 后端。
- VG-Lite 后端。
- GLX / OpenGL ES / OpenCL ES / 3D GPU 后端。
- 真实 DMA/PXP/VG-Lite 异步 descriptor queue（V4.7 已有 EGL worker 与 deterministic mock 并行调度，但还不是 MCU 外设）。
- 真实 fence interrupt/poll completion。
- 真实硬件失败后的恢复策略。

当前的 `MockAccelBackend`、`MockParallelAccelBackend` 与 `SimOpenGlAccel2dExecutor` 仍是验证契约的 mock/probe；`EglParallelAccelBackend` 是 host/sim 真实 EGL/OpenGL worker 后端，但不是 MCU/MPU 外设 API。`sim-opengl-egl` 不替换 NuttX sim 的 X11 framebuffer present 路径，只把 offscreen run readback 写回 FHRE framebuffer descriptor。

### 2. 2D packet 字段还不够贴近真实外设

已具备基础 descriptor，但还缺：

- source window / destination window。
- pitch alignment 要求。
- memory alignment 要求。
- color key。
- 更细的 blend mode。
- pre-multiplied alpha 标记。
- rotation/scale/format conversion 的明确字段。
- cache clean/invalidate 边界。
- framebuffer ownership / lifetime token。
- 多 plane 或 tiled framebuffer descriptor。

### 3. mask/layer/vector/text/3D 尚未硬件 packet 化

当前 packet 只覆盖：

- SetClip。
- Fill。
- Image。

未进入硬件 packet：

- mask enter/exit。
- layer begin/end。
- blur/shadow。
- vector path raster。
- glyph/text raster。
- triangle/3D。

这些路径目前走软件或 fallback。

### 4. codec 还没有真实硬件 decode 执行层

当前已经有 plan/job/mock token，但没有：

- 硬件 JPEG decoder submit。
- 硬件 PNG/zlib/IDAT submit。
- 硬件 SVG/vector raster submit。
- 硬件 glyph raster submit。
- decoded buffer allocator。
- cache insert 的异步完成回调。
- decoder job cancellation/timeout。
- resource bytes 的 zero-copy DMA 描述符。

### 5. 软件 codec 子集仍然有限

明确不是目标：

- 不追完整 PNG/JPEG/SVG/TTF 规范覆盖。
- 不引入重型外部库。

当前缺口：

- PNG interlace 等复杂路径仍会 unsupported。
- JPEG restart/复杂 progressive/复杂 sampling 等仍会 unsupported。
- SVG CSS/复杂 filter/mask/clip/path feature 不完整。
- TTF/OTF 完整 hinting、复杂脚本 shaping、完整 GPOS/GSUB 未完成。

### 6. ECS/调度器还只是最小底座

未完成：

- 并行 system executor。
- query/filter。
- command buffer。
- event/resource 依赖分析。
- change tick。
- scene hierarchy 与 ECS 的完整统一。

当前 ECS 足够支撑 Wing 声明式 UI 的固定容量节点状态，但还不是完整游戏 ECS。

### 7. 性能工程还需要继续收口

未完成：

- 系统性 benchmark suite。
- 目标 MCU/MPU 上的 cycle/带宽测量。
- renderer 热路径的 cache-aware 优化。
- dirty region 与 draw-chain 的更强合并策略。
- 软件 renderer 对不同 bpp/format 的专门优化。
- 长时间运行 soak test 自动化。

### 8. 文档/API 稳定性仍在演进

当前 V4.7 已经形成契约，但以下 API 还可能继续变化：

- `Accel2dCommand` 的 image/blend 字段。
- `Accel2dFramebufferDescriptor` / source descriptor。
- executor/ring/fence/parallel backend 的真实平台 trait。
- GLX/EGL/OpenGL ES shim 的窗口、context、FBO/readback 生命周期。
- codec job/backend trait 的 decoded buffer 生命周期。
- `RenderStats` 字段可能继续增加。

## 五、下一阶段待实现建议

### V4.8 建议目标：第一块真实 MCU/MPU 2D 外设接入

V4.7 已把默认关闭的 EGL host worker 接入到 `ParallelRenderBackend`。下一步应选择一个真实 MCU/MPU 2D 外设，把同一套 `Accel2dCommandList + ring/fence + fallback` 契约落到板级后端：

- STM32 DMA2D / NXP PXP / VG-Lite：把 `Accel2dCommandList` 映射到外设 descriptor。
- `prepare_submission_ring()` / `flush_submissions()`。
- `poll_fence()` / `complete_fence()`。
- backend 私有错误到 `Fallback/Unsupported/Failed` 的映射。
- framebuffer/source descriptor lifetime 规则。

优先从最小集合开始：

- RGB565 framebuffer。
- solid fill。
- alpha fill。
- memory-to-memory image blit。
- global alpha blend。
- clip/scissor。
- 同步 flush。
- 完整 fallback。

建议先做 STM32 DMA2D 或 NXP PXP 其中之一，字段映射稳定后再抽公共差异。EGL worker 继续作为 host/sim 的并行调度验证后端。

### V4.9 建议目标：image descriptor 扩展

继续完善：

- source rect。
- destination rect。
- stride/pitch alignment。
- format conversion 标记。
- color key。
- premultiplied alpha。
- nearest/bilinear 策略。
- rotate/flip 能力位。

### V4.10 建议目标：codec job 真执行

在不扩完整 codec 的前提下，把 job 接到真实执行层：

- resource bytes descriptor。
- decoded buffer descriptor。
- hardware candidate stage -> prepare/submit/complete。
- complete 后 cache insert。
- timeout/failure 后 placeholder。
- stats 保持 missing/decode failure/placeholder 分离。

### 持续目标：软件路径保持可用

每轮硬件化都必须保持：

- 软件 `Surface` baseline 像素稳定。
- missing/unsupported 能稳定 placeholder。
- fallback 不污染 framebuffer。
- `fhre_build.sh` 和 `wing_build.sh` 成功。
- WING 不解析 FHRE 私有硬件 packet/ring/fence。

## 六、当前测试覆盖

本次盘点时 `cargo test --manifest-path apps/fhre/rust/Cargo.toml` 显示 FHRE 默认 feature 有 41 个单测；`sim-opengl` 为 42 个，`sim-opengl-egl` 为 45 个。覆盖重点如下：

- draw-chain run 拆分。
- run contract 最小状态。
- clip marker command span。
- stateful mask/layer run fallback。
- payload-only probe backend。
- packet builder descriptor 字段。
- packet builder 的 missing/overflow/stateful 错误。
- submission ring queue/flush/complete 顺序。
- ring stable errors。
- mock accel fill/alpha/clip/image 像素一致性。
- missing image fallback 不污染 framebuffer。
- 多 packet 顺序。
- parallel mock backend 的 disjoint queue、software-while-hw、overlap barrier、stateful barrier。
- `sim-opengl` packet mapping 与像素一致性。
- `sim-opengl-egl` unavailable/stub 路径、EGL parallel backend 像素一致性、ring overflow fallback。
- ImageCache placeholder/result 语义。
- codec pipeline hardware candidate 统计。
- codec pipeline job 状态机。
- mock codec backend token/failure。

建议每轮 FHRE 底座改动后至少跑：

```sh
cargo test --manifest-path apps/fhre/rust/Cargo.toml
cargo check --manifest-path apps/examples/fhre_demo/rust/Cargo.toml
cargo check --manifest-path apps/wing/rust/Cargo.toml
cargo check --manifest-path apps/examples/wing_demo/rust/Cargo.toml
./nuttx/fhre_build.sh
./nuttx/wing_build.sh
cargo test --manifest-path apps/fhre/rust/Cargo.toml --features sim-opengl
cargo test --manifest-path apps/fhre/rust/Cargo.toml --features sim-opengl-egl
FHRE_EGL_WORKER_ENABLE=1 cargo test --manifest-path apps/fhre/rust/Cargo.toml --features sim-opengl-egl opengl_accel::tests::
```

## 七、当前风险清单

- `MockAccelBackend` 是同步 mock，`MockParallelAccelBackend` 是确定性延迟提交 mock，二者都不能代表真实 DMA/GPU 的 cache coherency、interrupt、bus contention。
- `EglParallelAccelBackend` 是真实 host/sim EGL worker，但构建 gate 默认关闭；启用时依赖宿主 EGL/OpenGL driver，不代表 MCU 外设行为。
- 软件 renderer 可用但仍需要目标板实测，尤其是 480x480、640x480、16bpp/32bpp 场景。
- codec 使用 `alloc::Vec`，资源预热阶段可接受，但不能让复杂 decode 混入每帧热路径。
- SVG/TTF/JPEG/PNG 子集必须继续保持“unsupported 就 placeholder/fallback”的策略，不能为了兼容性牺牲小系统稳定性。
- `RenderStats` 字段很多，后续需要分组或压缩 HUD，否则调试信息会越来越难读。
- NuttX sim 和真实板子的 framebuffer/input 行为可能不同，page-pan、poll、present fallback 都需要真实硬件回归。

## 八、简短结论

FHRE 当前已经完成了“硬件可接管的底座契约”，不是停留在纯软件 demo：

- 上层用 `DrawCommand/DrawList` 声明绘制。
- 中层把命令拆成 `DrawChainRunContract`。
- 硬件边界使用 `DrawChainOpPayload`。
- 2D 加速边界下沉到 `Accel2dCommandList`。
- 多 run 提交边界下沉到 `Accel2dSubmissionRing + Accel2dFence`。
- mock backend 已经能真执行 packet 并通过像素测试，parallel mock backend 已经能验证 queue/barrier/software-while-hw 收敛。
- 默认关闭的 `sim-opengl` / `sim-opengl-egl` 已经形成 GL-like packet probe 与真实 EGL worker 边界。
- codec 侧已经有 plan/job/mock token 状态机。

但 FHRE 还没有完成 MCU/MPU 真实外设接入。下一步最重要的是把 V4.7 的 packet/ring/fence/parallel 契约落到一个真实 2D 外设上，同时继续坚持：软件 fallback 稳定、资源缺失 placeholder、WING 不拥有私有硬件路径。
