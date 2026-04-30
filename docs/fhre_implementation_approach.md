# FHRE 实现思路

本文定义当前最新 FHRE 实现方向。结论先放前面：

```text
FHRE 不是旧版 FHRE 的复刻，也不是迷你 Bevy。

FHRE = no_std Rust 纯 3D 轻量游戏/图形引擎核心 + 绘制后端抽象 + ECS/scene/input/assets
Wing = 基于 FHRE 的无宏、声明式 ECS Shell/UI runtime
Wing 游戏应用 = 基于 FHRE 的 game loop、输入、scene 和渲染能力开发
```

旧 `/home/uan-gpd/codes/FeatherOS/apps/hand_down/docs/ARCHITECTURE.md`
仍然有参考价值。它的“FHRE 是纯 3D 引擎，2D 只是 z=0 平面的特例”应该成为新实现从第一天开始的核心设计理念，否则后续扩展 3D 会牵动 Wing 和游戏应用的上层 API。需要收敛的是旧实现的复杂度：第一阶段不做完整双世界、不做复杂 scene graph、不做 Bevy 式大调度，但运行时语义必须是纯 3D。

## 目标环境

目标设备：

- 高性能 MCU。
- 低功耗 MPU。
- 智能手表。
- MP4 / 小屏手机。
- 复古分辨率掌机。
- 小型 2D/2.5D 游戏。
- 轻量 3D 或伪 3D demo。

可能的显示能力：

- 纯软件 framebuffer。
- 2D blitter。
- DMA2D / PXP / VG-Lite 这类 2D 或 2.5D 加速。
- 初级 OpenGL ES / OpenCL ES / 3D 能力。

基础约束：

- Rust 优先。
- 同时保留 clang/C 版本目录和 ABI 规划。
- Rust 必须 `no_std`。
- 不依赖外部 Rust crate。
- 不使用宏生成核心 ECS/UI 逻辑。
- 允许 `alloc`，但热路径分配必须可控。
- 默认优先 RGB565。
- `fhre_build.sh` 必须能独立构建 `fhre_demo`。
- `wing_build.sh` 必须也能编入 `fhre_demo`，用于 Wing 调试时确认 FHRE 正常。

## V3.10 运行链路可串接编排

V3.10 在 V3.9 的基础上把绘制候选链升级为 **run 级编排器**：在顺序不变前提下，尽量把可加速操作拼成长片段提交，减少硬件/软件切换次数；无法提交的部分仅以局部回退方式执行，避免重绘整帧。

- `DrawList` 在 full 或 dirty pass 里：
  - 先按 dirty clip 编译 `DrawChain`；
  - 根据 backend 能力、`max_chain_ops`、状态切换边界、候选能力一致性切成 run；
  - 每个 run 独立尝试 `submit_draw_chain()`。
- 回退策略（最小打断）：
  - `submit_draw_chain()` 返回 `Unsupported/Fallback` 时，只回退执行当前 run 对应命令区间。
  - clip/layer/mask 状态变化会作为 run 边界候选，保持命令语义并减少无效回退。
- 并行能力：
  - 编排阶段仅采集 `parallel hints`（非重叠候选 run）作为下一步调度提示，不在本阶段执行并行绘制。
- 统计与观测：
  - `RenderStats` 增加 `draw_chain_runs`、`draw_chain_hw_runs`、`draw_chain_sw_runs`、`draw_chain_splits`、`draw_chain_parallel_hints`；
  - run 级计数与 existing `top chain task/fallback` 一起保留，`fhre_demo` HUD 继续展示 V3.10 行。
- 接口兼容：`DrawChain`、`DrawBackendDispatch`、`BackendCapabilities` 及 `RenderBackend::submit_draw_chain()` 保持接口稳定扩展；默认软件 backend 仍返回 `Unsupported` 并走原软件渲染结果。

本阶段不改变 FHRE 软件像素输出，只增强“软硬兼容调度”能力，为后续 DMA2D/GPU backend 提供更少打断、可直接下发的 run 边界。

## V3.9 硬件可插手绘制/资源管线

V3.9 继续 FHRE-first，但不直接绑定 DMA2D、PXP、VG-Lite、OpenGL ES 或某个具体芯片。目标是先把 draw 与 codec 拆成硬件更容易接管的固定容量 descriptor/stage 边界：软件 `Surface` 渲染结果保持不变，真实硬件后端未来只需要覆写提交入口。

本轮新增和稳定的内容：

- Draw chain descriptor：
  - 新增固定容量 `DrawChain<const OPS>`、`DrawChainOp`、`DrawChainStats`、`DrawChainCapabilities` 和 `DrawChainSubmitResult`。
  - `DrawChainOpKind` 只表达硬件容易接管的最小操作：`SolidFill`、`AlphaFill`、`ImageBlit`、`ImageBlend`、`Clip`、`MaskEnter/MaskExit`、`LayerEnter/LayerExit` 和 `FallbackRange`。
  - `DrawList` 在 full draw 或 dirty pass 中先按当前 dirty clip 编译 chain candidate，再调用 `RenderBackend::submit_draw_chain()`；默认软件 backend 返回 `Unsupported`，随后仍执行原有软件绘制路径，因此不会改变当前画面。
  - `BackendCapabilities` 新增 `draw_chain`、`max_chain_ops`、`chain_draw_features`，用于后续硬件 backend 声明可链化任务边界。
  - `RenderStats` 新增 draw chain candidate/op/submitted/fallback/unsupported/overflow 和 top chain task 统计；`fhre_demo` HUD 新增 V3.9 行，可直接看到当前帧哪些命令已经能被编译为链、哪些仍会回落软件。
- Codec pipeline stage skeleton：
  - 新增 `CodecStageKind`、`CodecStagePlan`、`CodecPipelinePlan<const STAGES>`、`CodecAcceleratorCapabilities`、`CodecPipelineStats`。
  - PNG/JPEG/FRAW/TTF/SVG 都提供轻量 `plan_pipeline()` / `plan_pipeline_with_caps()` 入口，只描述阶段，不改变现有自研 decode 结果。
  - 统一 stage 口径为 read、inspect、header、entropy/parse、transform/raster、color/pack、cache insert、fallback。当前默认 accelerator capability 为 `NONE`，所以硬件候选 stage 会记录 unsupported/fallback，但 decode 仍由软件路径完成。
  - `ImageCache::prewarm_result()` 在资源读取后记录 PNG/FRAW/JPEG pipeline stats；`fhre_demo` 的 TTF/SVG fixture prewarm 也会记录 pipeline stats。draw 热路径仍只查 resolver/cache，不做 decode。
- Demo 与文档：
  - `fhre_demo` HUD 增加 `V39 CHAIN/PIPE` 行，显示 draw chain candidate/submitted/fallback、top chain task 和 codec pipeline candidate/fallback。
  - Wing 不新增页面，不拥有硬件路径，只继续作为 FHRE draw/resource/cache/stats 的消费方。
  - ROMFS manifest/budget 和 stale ROMFS 清理继续保留，防止大资源绕过构建期检查进入默认镜像。

## V3.8 绘制/资源管线硬化

V3.8 继续 FHRE-first：不扩 Wing 页面，不引入 LVGL、FreeType、libjpeg、librsvg 或外部 Rust crate，把 V3.7 的绘制、codec、cache、dirty 和 stats 从“可用基线”推进到可复用、可观测、可被 Wing 稳定消费的运行时边界。

本轮新增和稳定的内容：

- V3.7 修复保留为基线：
  - `parse_svg_document_into()` 保持为 stack-safe SVG document 解析入口，demo 不再把大 `SvgDocument` 临时压到 NuttX task 栈。
  - NuttX sim framebuffer `FBIOPAN_DISPLAY` 失败时保留 copy-present fallback，避免 demo 主循环存活但窗口黑屏。
  - `fhre_demo` / `wing_demo` Kconfig 继续使用 `$APPSDIR`，sim ROMFS 资源继续按 config 安装，避免 stale absolute path 和缺资源构建失败。
- SVG document cache：
  - 新增固定容量 `SvgDocumentCache<const DOCS, const PATHS, const CMDS>` 和 `SvgDocumentCacheStats`，提供 `prewarm_result()`、`view()`、`stats()`；复杂 SVG decode 只发生在 `ResourceLoader -> cache prewarm` 路径，绘制 resolver 只查 cache。
  - `fhre_demo` 已从私有 SVG 静态数组迁移到 `SvgDocumentCache`，SVG cache miss、load failure、decode failure、overflow 都返回 fallback 并写入 codec/cache/render stats，不阻塞帧提交。
- RenderStats / HUD：
  - `RenderStats::mark_svg_document_cache()` 将 SVG document cache hit/miss/load/fallback/slot 汇入帧统计，`fhre_demo` HUD 的 V3.8 行可观察 SVG document cache 与 glyph/text fast path。
  - `RenderStats` 继续以 `DrawTaskKind` / `DrawBackendDispatch` / dirty clip / present 为统一口径；新增字段只服务 HUD 和 runtime 验收，不扩无用计数。
- Draw 热路径：
  - RGB565 alpha blend 内部提成共享 raw helper，image 与 text 不再各自维护一份整数混合逻辑。
  - `GlyphView` 的 A8 glyph 在 RGB565 framebuffer 上新增 mask-aware raw blend 路径，裁剪后直接按 glyph bitmap 写入目标像素，避免每个 glyph 像素都构造 `Color` 并重复进入 `put_pixel()`。
  - `DrawImage` / `DrawImageStyled` 的 RGB565 normal-blend 快路径扩展到 mask/rounded clip 场景；无 mask 时仍保留 row-copy / nearest scale，带 mask 时在同一快路径内合并 image alpha 与 mask alpha 后直接写 RGB565。
- 资源边界：
  - `ImageCache::prewarm_result()` 继续作为 PNG/FRAW/JPEG 统一入口；字体与 glyph run 仍通过 `GlyphCache` / `GlyphRunCache` 预热；SVG document 与 path-level `SvgCache` 并存，不互相替代。
  - cache miss、decode 失败、容量超限、scratch 超预算都只显示 fallback 并进入 `CodecStats` / `RenderStats`。
  - NuttX sim ROMFS 不再整目录复制资源，而是使用 `apps/fhre/resource/romfs_manifest.txt` + `apps/tools/check_resource_budget.sh` 做构建期 allowlist 和 byte budget 检查；清单内资源超预算会让 `fhre_build.sh` 直接失败并打印具体文件。
  - `fhre_build.sh` / `wing_build.sh` 在 sim profile 切换时会清理旧的 `boards/sim/sim/sim/src/etc/{fhre,wing}` 和 `etctmp.*`，避免上一次构建残留的大资源绕过 manifest 进入本次 ROMFS。
  - 过大的参考 SVG 不进入默认 ROMFS。`lvgl_tiger.svg` 保留在源码树中作为参考素材，不再作为默认 demo fixture；SVG overflow 仍通过小型 `svg/overflow.svg` 覆盖，避免为了测试 fallback 而引入不轻量资源。
  - 过大的通用字体不进入默认 ROMFS。`dejavu_sans.ttf` 保留为源码参考，`fhre_demo` 默认只安装小型 TTF/OTF fixture，避免 NuttX `etctmp.c` 因嵌入大资源而占用过高主机内存。

## V3.7 LVGL draw 层与自研 codec 基线继续收敛

V3.7 的主线继续放在 FHRE core：参考 LVGL draw 层的任务口径和热路径组织方式，但不引用 LVGL、FreeType、libjpeg、librsvg 或任何外部 Rust crate。目标是让 `fhre_demo` 的 parity suite 同时覆盖绘制性能、codec/prewarm/cache 和 fallback 统计，而 Wing 只作为消费 FHRE 能力的 Shell 压力场景。

本轮新增和稳定的内容：

- Draw 热路径：
  - `DrawImage` / `DrawImageFit` 的 RGB565 software backend 增加 normal-blend 快路径边界：无 mask、无 tint、`BlendMode::Normal` 时，RGB565/RGB888/RGBA8888/A8 image view 可直接进入 RGB565 row-copy、nearest scale 或 alpha blend 路径，避免每像素走完整 `Color` sample + `put_pixel()` 分派。
  - `RenderStats::mark_command_kind()` 将普通 image 命令和满足 fast-path 条件的 `DrawImageStyled` 纳入 `fast_path_hits` 观察范围，继续通过 `DrawTaskKind::Image` / `DrawBackendDispatch` 记录 software/fallback 任务口径。
- Resource / codec 边界：
  - `ImageCache::{get_or_load_result,prewarm_result}` 返回 `Result<ImageView, CodecErrorKind>`，保留旧 `Option` API；demo fixture 不再需要从 stats delta 反推 image decode 失败类型。
  - 新增统一 resource image inspect/decode 入口：`inspect_resource_image(_result)`、`decode_resource_image(_result)`、`ImageResourceKind`、`ImageResourceInfo`。FRAW/PNG/JPEG 仍由各自自研 decoder 负责，统一入口只做格式识别、metadata 汇总和错误归类。
  - `ImageDecodeErrorKind::as_codec_error()` 稳定 image decode failure 到 `CodecErrorKind` 的映射，保持 missing/invalid/truncated/unsupported/overflow 可观测。
- Demo 验收：
  - `fhre_demo` image fixture prewarm 改用 `ImageCache::prewarm_result()`，PNG/FRAW/JPEG 正向和负向 fixture 的错误分类直接来自 cache/codec 边界。
  - Wing 不新增页面，不私有解析资源，不绕过 FHRE draw/resource API。

## V3.6 LVGL draw 层性能与真实热路径收敛

V3.6 继续不扩 Wing 页面，也不增加新的复杂格式名。目标是把 V3.5 已经接通的 glyph-id 文本、SVG mask/filter、codec/cache 和 dispatch stats 变成可长期比较的性能基线：每个 parity 模式都能看到当前压力类型、top dispatch task、top fallback task、draw/present/dirty/copy/cache 成本。

本轮新增和稳定的内容：

- 性能基线：
  - `RenderStats::benchmark_summary()` 新增轻量汇总入口，返回 draw_us、present_us、dirty passes、dirty copy bytes、present bytes、top dispatch task 和 top fallback task。
  - `DrawTaskKind::from_index()` / `short_label()` 和 `RenderStats::{dispatch_hits_for,fallback_hits_for,top_dispatch_task,top_fallback_task}` 形成 backend 可替换统计边界，后续 DMA2D/VG-Lite/DRM/SDL/OpenGL ES backend 可复用同一任务口径。
  - `fhre_demo` HUD 顶部显示当前 parity mode 的 benchmark profile，例如 `IMAGE HEAVY`、`TEXT HEAVY`、`VECTOR HEAVY`、`FULL REDRAW STRESS`；模式切换仍强制 full redraw，稳定后回到 dirty redraw，`STRESS` 模式保持 full redraw 压力。
- Text/Font V3.6：
  - `GlyphIdResolver + GlyphRunResolver + GlyphRunCache` 路径保持 V3.5 语义：demo 启动或模式切换前 prewarm，draw 热路径只查 cache/resolver。
  - HUD 继续显示 glyph-id draw、codepoint fallback、selection fallback、layout overflow、glyph-run cache hit/miss/insert/overflow。
- SVG/Vector V3.6：
  - `VectorMaskScratch` 的 success/overflow/fallback 成本进入固定 VECTOR/STRESS 可验收样例；SVG filter blur/drop-shadow 的 layer budget fallback 继续进入 stats。
  - `DrawSvgDocument` 的 fill/stroke/gradient/clipPath/mask/filter layer pass 继续必须经过 `DrawBackendDispatch` 和 dirty clip。
- Codec/Resource V3.6：
  - fixture manifest 仍为 `fhre_demo` 私有，继续覆盖 PNG/FRAW/JPEG/TTF/OTF/SVG 正向和负向用例。
  - success、missing、invalid、truncated、unsupported、overflow、budget-exceeded 都通过 `CodecStats` / `RenderStats` 汇入 HUD，不进入 draw 热路径解码。

## V3.5 GlyphRun / SVG mask 热路径收敛

V3.5 不扩 Wing 页面，也不继续堆新的 codec 类型。目标是把 V3.4 的 draw/codec 基线修成真实可持续使用的热路径：`DrawLabel` 的 shaped-run 路径必须按 glyph id 绘制，selection/layout 必须基于源文本范围，SVG mask/filter 必须有界、可 fallback、可统计。

本轮新增和稳定的内容：

- Text/Font V3.5：
  - `GlyphIdResolver` 成为 FHRE 公共文本入口之一。`DrawLabel` 的 shaped `GlyphRun` 路径优先按 `GlyphRunItem.glyph_id` 取 glyph bitmap；缺 resolver、缺 glyph 或 debug font 时才 fallback 到 codepoint / 5x7 path。
  - `GlyphRunItem` 增加 `char_start/char_len` 源文本范围。`TextLayout::from_glyph_run()`、selection background/foreground、underline/strikethrough 不再假设 “1 glyph = 1 char”，可以正确表达 Latin ligature。
  - `GlyphRunCache<RUNS>::prewarm()` 文档化为启动或模式切换阶段的预热入口；绘制阶段的 `GlyphRunResolver` 只查 cache，miss 返回 fallback run，不在 draw 热路径解析 TTF/OTF。
  - `RenderStats` 新增 glyph-id draw、codepoint fallback 和 selection fallback 计数，并继续保留 glyph-run cache hit/miss/insert/evict/overflow。
- SVG/Vector V3.5：
  - `VectorMaskScratch` 继续作为有界 A8 mask scratch。简单 `clipPath` / mask path 约束 fill、stroke、cap/join 和 linear/radial gradient；mask 或 layer 超预算只 fallback 并进入 stats。
  - SVG filter blur/drop-shadow 仍映射到 bounded layer，filter layer 内部 draw pass 必须经过 `DrawBackendDispatch` 和 dirty clip。
- Draw dispatch / parity：
  - 顶层 `PushMask` / `PushBitmapMask` 的 dispatch 统计改为按 `DrawTaskKind::MaskRect` / `MaskBitmap` 进入 per-task counters。
  - `fhre_demo` 继续固定 `RECT/BORDER/SHADOW/ARC/TEXT/IMAGE/VECTOR/LAYER/MASK/TRIANGLE/3D/STRESS` parity suite，所有模式都走 `DrawList -> DrawBackendDispatch -> dirty clip -> RenderStats -> present`。
- Codec/Resource V3.5：
  - PNG/JPEG/TTF/OTF/SVG decode 仍只能通过 `ResourceLoader -> Cache::prewarm()` 发生；draw 热路径只查 `ImageCache/GlyphCache/SvgCache/GlyphRunCache` 或 resolver。

## V3.4 LVGL draw 层能力收敛与性能基线

V3.4 不继续增加 Wing 页面，也不引入 LVGL、libjpeg、FreeType、librsvg 或外部 Rust crate。目标是把 V3.3 已经接入的 SVG A8 mask、shaped `GlyphRun` label、codec fixture 和 draw stats 收敛为可长期维护的 draw 层基线：每个能力都要能通过 `fhre_demo` 的 parity suite 观察、定位和 fallback。

本轮新增和稳定的内容：

- Draw parity baseline：
  - `fhre_demo` 固定以 `RECT/BORDER/SHADOW/ARC/TEXT/IMAGE/VECTOR/LAYER/MASK/TRIANGLE/3D/STRESS` 作为 V3.4 parity suite；所有模式继续走 `DrawList -> DrawBackendDispatch -> dirty clip -> RenderStats -> present`。
  - LVGL draw task 口径继续限定为 fill、border、box shadow、line、arc、triangle、label、image、vector、layer、mask、blur、3D；这不是 LVGL widget/object/style cascade。
  - HUD 继续显示 codec/cache、dispatch、text layout、vector mask、SVG filter budget、layer/mask/blur、present 成本，并新增 glyph-run cache 摘要。
- Text/Font V3.4：
  - `GlyphRunResolver` 稳定为 FHRE 公共文本入口。`DrawLabel` 优先消费 shaped run，失败后回到 UTF-8 fast path。
  - 新增固定容量 `GlyphRunCache<RUNS>` / `GlyphRunCacheStats`，用于缓存短文本 shaped run。cache key 使用 font、size、`ShapeOptions` 和文本指纹；cache miss 只返回 fallback run 或触发预先配置的 resolver，不在 draw 热路径解析字体文件。
  - `RenderStats` 新增 `glyph_run_cache_hits/misses/inserts/evictions/overflows/slots`，`fhre_demo` 的 V3.4 HUD 行可直接看到 shaped-run cache 是否命中、是否溢出。
  - `TextLayout<N>` 仍保持固定容量；layout overflow、missing glyph、GSUB/GPOS/kern hit、CFF fallback 继续进入 `RenderStats` 和 demo HUD。
- SVG/Vector V3.4：
  - `VectorMaskScratch` 继续作为有界 A8 mask scratch：简单 `clipPath`、mask path、fill/stroke/gradient 受 mask 约束；mask/layer 超预算只 fallback，不崩溃。
  - SVG filter blur/drop-shadow 仍只允许 bounded layer 映射，filter layer 内部 draw pass 也必须走 dispatch 与 dirty clip。
  - linear/radial gradient 支持 `spread=pad`、stop opacity 和基础 transform；repeat/reflect/pattern/mesh gradient 明确 fallback。
- Codec/Resource V3.4：
  - PNG/JPEG/TTF/OTF/SVG decode 仍只能通过 `ResourceLoader -> Cache::prewarm()` 发生；draw 热路径只查 `ImageCache/GlyphCache/SvgCache/GlyphRunCache` 或 resolver。
  - fixture manifest 覆盖 Adam7/tRNS/16-bit PNG、baseline/progressive/CMYK/EXIF JPEG、小型 GPOS TTF/OTF-CFF 字体、SVG clip/mask/filter/gradient，以及 missing/invalid/truncated/unsupported/overflow 负向用例。
  - 资源失败、cache 超限、scratch 超预算统一显示 fallback，并写入 `CodecStats` / `RenderStats`。

## V3.3 LVGL draw 层真实可用度收敛

V3.3 继续不引入 LVGL、libjpeg、FreeType、librsvg 或外部 Rust crate，而是把 V3.2 中仍偏“统计边界/示例 layout”的能力继续收敛到可持续使用的 draw 热路径。所有复杂资源仍必须先进入 `ResourceLoader -> Cache::prewarm()`，绘制热路径只查 `ImageCache/GlyphCache/SvgCache` 或 resolver。

本轮新增和稳定的内容：

- SVG/Vector V3.3：
  - `VectorMaskScratch` / `MaskRasterOptions` 不再只是预算/统计边界。`DrawSvgDocument` 会为简单 `clipPath` / `mask path` 构建有界 A8 scratch mask，小 mask 复用该 mask 约束 fill、stroke、cap/join 和 gradient；超出预算时回到受控 fallback，并继续记录 overflow。
  - fill 与 stroke 都受 SVG `clipPath` / `mask` 约束。fill 路径使用 A8 mask span，stroke 路径使用 mask-aware line/cap/join 绘制，避免 clip/mask 只约束填充、不约束描边。
  - `RenderStats` 继续记录 `vector_mask_rasters`、`vector_mask_scratch_overflows`、`svg_filter_budget_exceeded`，`fhre_demo` HUD 的 V3.3 mask/text 摘要可同时看到成功路径和 fallback 路径。
  - linear/radial gradient 继续保持 stop offset / opacity / 基础 transform 的轻量映射；`spread=pad` 是当前目标，reflect/repeat/pattern/mesh gradient 仍 fallback。
- Text/Font V3.3：
  - `DrawLabel` 新增 shaped-run 消费路径。若 `Surface` 配置了 `GlyphRunResolver`，label 会优先取得固定容量 `GlyphRun<64>`，再通过 `TextLayout<N>::from_glyph_run()` 完成 wrap、align、selection、underline 和 strikethrough；没有 shaped run 时继续使用 UTF-8 fast path。
  - `GlyphRunResolver` 返回 draw-space 像素 advance，保证 `DrawLabel` 不在绘制热路径解析 TTF/OTF。字体文件解析、GSUB/GPOS/kern 和 glyph raster 仍只发生在 prewarm/cache 路径。
  - 固定容量 `TextLayout<N>` / `TextLayoutOptions` / `TextLayoutLine` 继续作为 label layout 边界；layout overflow、shaping fallback 和 shaped layout success 都进入 demo HUD。
  - `ShapeOptions` 继续保持当前 `kerning/gsub/gpos` 字段名，同时稳定 `enable_kern()` / `enable_gsub()` / `enable_gpos()` 语义别名，避免破坏现有调用点。
  - Latin-only GSUB/GPOS/kern 的真实 shaping 仍由 `FontFace::shape_text()` 输出固定容量 `GlyphRun<N>`；复杂脚本、bidi、mark positioning 继续受控 fallback。
- Codec/Stats V3.3：
  - `RenderStats` 新增 `text_shaped_layouts`，并继续保留 `text_layout_overflows`、`text_shaping_fallbacks`、progressive JPEG、CFF raster、OpenType shaping、SVG mask/filter/gradient 统计。
  - PNG/JPEG/TTF/SVG decode 继续只能发生在 prewarm/cache 路径；draw command 热路径仍只查 resolver/cache，miss 只显示 fallback。
  - `fhre_demo` 的大状态对象在 demo 侧使用受控堆分配，避免 NuttX sim app 栈被 codec fixture、SVG/TTF cache 和 demo ECS 状态共同挤爆；FHRE core 的 ECS/UI/draw command 仍保持固定容量优先，只有资源/cache/layer/mask scratch 走受控 `alloc`。

- JPEG V3.1：
  - `JpegDecodeOptions::DEFAULT.allow_progressive` 已打开，`allow_progressive=false` 仍可强制走 `UnsupportedProgressive` fallback。
  - Progressive SOF2 子集作为正向验收能力：支持 8-bit、Huffman、1/3 分量 grayscale/YCbCr、DC/AC first scan、successive refinement、EOB run 和 restart interval；当前目标 fixture 是 `lvgl_progressive.jpg`。
  - Progressive decode 增加 scan 顺序、successive approximation、restart interval、scan/component mismatch 和系数缓冲预算检查；progressive+CMYK、arithmetic coding、12-bit JPEG 和精确 ICC/gamma 色彩管理仍为受控 fallback，并由 `fixture_progressive_cmyk.jpg` 等负向 fixture 进入 HUD。
  - baseline CMYK/YCCK、APP14、EXIF orientation 1-8 保持可用。
- Font/Text V3.1：
  - `FontFaceKind::OpenTypeCff` 不再只是可见 fallback：新增 Type2 charstring 最小 raster，支持 moveto/lineto/curveto、h/v variants、`endchar`、local/global subr，并用固定 subr 深度上限避免失控递归。
  - TrueType glyf、cmap format 4/12、legacy `kern`、GPOS/GSUB table presence 保持；`ShapeOptions` 继续表达 `kerning/gsub/gpos` 边界。V3.1 的 `TextShaper` 提供 Latin-only 最小闭环：GSUB SingleSubst/LigatureSubst、GPOS PairPos format 1/2 和 legacy kern 可进入固定容量 `GlyphRun<N>`，并在 `GlyphRunItem::shaping` 中标记 GSUB/GPOS/KERN 命中。
  - OTF/CFF 解析失败、subr 深度超限、CFF2/WOFF/WOFF2/复杂脚本 shaping 仍输出占位 glyph 或 codec fallback，不阻塞绘制。
- SVG/Vector V3.1：
  - `SvgPaint` 新增 `LinearGradient` / `RadialGradient`，`DrawSvgDocument` 不再把 gradient URL 只平均成单色，而是在 scanline fill 中做轻量线性/径向渐变。
  - `SvgDocument` 新增 `unsupported_features` 标记；parser 可识别 `clipPath`、`mask`、`filter`、gradient spread 和复杂 CSS selector 的边界。简单 `clipPath` / `mask` 已从 bounds 近似推进为固定容量 path mask，fill scanline 会与 clip/mask path 相交后写入；复杂 clip/mask 仍 fallback。
  - 小区域 `feGaussianBlur` / `feDropShadow` 继续映射到 FHRE blur/shadow 近似路径，受 `SvgRasterOptions.layer_budget_bytes/filter_max_radius` 控制；超预算或复杂 filter 进入 fallback 统计。
  - `defs/use`、shape-to-path、inline/class style、group opacity、nested transform、fill+stroke、cap/join、evenodd/nonzero 保持 V2.8/V2.9 可见能力；pattern/mesh gradient、完整 CSS cascade、复杂 mask/filter 仍作为后续阶段。
- Stats/API：
  - `RenderStats` 的 progressive JPEG、CFF raster、OpenType shaping、SVG clip/mask/filter/gradient 计数字段已经接入真实 fixture prewarm / SVG document feature scan / demo HUD，避免只存在 API 字段但无法观察。
  - V3.1 继续增加 progressive scan fallback、CFF fallback glyph、GSUB/GPOS/KERN 命中、SVG real clip/mask 和 SVG filter/gradient fallback 统计。
  - `SvgRasterOptions` 新增 `clip_mask_path_capacity/mask_scratch_bytes`，为后续把 path mask 与 bitmap mask 做成可协商预算保留边界。

## V2.8 复杂格式扩展基线

V2.8 的目标是让 FHRE 能面对更多真实资源，但仍保持嵌入式边界：复杂格式必须先进入 fixture/prewarm/cache，再由 draw 热路径只查 resolver，不能边画边解码。

本轮新增和稳定的内容：

- 真实 fixture 已导入 `apps/fhre/resource`：
  - LVGL test assets: `lvgl_16bit_rgba.png`、`lvgl_palette.png`、`lvgl_cmyk.jpg`、`lvgl_exif_90/180/270.jpg`、`test_gpos_one.ttf`。`lvgl_tiger.svg` 仅作为源码参考，不进入默认 ROMFS。
  - Progressive JPEG fixture: `lvgl_progressive.jpg`，V2.9 已升级为正向验收 fixture；progressive+CMYK 仍保留为后续 fallback 边界。
  - Bootstrap Icons fixture: `bootstrap_bezier2.svg`、`bootstrap_cloud_rain.svg`、`bootstrap_star_fill.svg`，并保留 `resource/licenses/bootstrap_icons_LICENSE`。
  - `test_kern_one_otf.c` 已用 Python 标准库提取为 `fonts/test_kern_one.otf`；运行时不解析 C 文件。
- PNG V2.8：
  - `PngInfo` 新增 `interlaced/srgb/gamma/icc_present`。
  - critical chunk 增加 CRC 校验，CRC 错误归入 `Invalid`。
  - Adam7 interlace 已可展开到统一 `DecodedImage`，demo 使用 `fixture_adam7_rgba.png` 作为正向 fixture。
  - `gAMA/sRGB/iCCP` 只解析 metadata，不做完整色彩管理；APNG 仍为 `Unsupported`。
- JPEG V2.8 / V2.9：
  - `JpegDecodeOptions` 新增 `apply_exif_orientation/allow_progressive/allow_cmyk`。
  - baseline CMYK/YCCK 已支持：解析 Adobe APP14 transform，输出 RGB565。
  - EXIF orientation 1-8 已在输出阶段旋转/翻转；超预算会 fallback。
  - V2.9 已补 progressive SOF2 第一版，支持 DC/AC scan、successive refinement、EOB run 和 restart interval；progressive+CMYK、arithmetic coding、12-bit 和精确 ICC/gamma 色彩管理仍 fallback。
- TTF/OTF V2.8 / V2.9：
  - 新增 `FontFaceKind::{TrueTypeGlyf, OpenTypeCff}`、`OpenTypeLayout`、`ShapeOptions`、`GlyphRun<N>`。
  - TrueType glyf 路径继续支持 cmap format 4/12、simple/composite glyph 和 legacy `kern`。
  - OTF/CFF 现在可被解析为 `OpenTypeCff`，V2.9 已提供 Type2 charstring 最小 raster；hinting、完整 Type2、CFF2、WOFF/WOFF2 和复杂 shaping 仍未实现。
  - GPOS/GSUB 当前作为 table-presence/layout 边界被识别，复杂 shaping 仍不启用。
- SVG V2.8 / V2.9：
  - `DefaultSvgDocument` 扩为 `12 paths / 256 commands`。
  - parser 新增 `rect/circle/ellipse/line/polyline/polygon/use` 转 path。
  - 支持 inline `style="..."`、简单 `.class { fill/stroke/stroke-width/opacity/fill-rule }`、nested transform、group opacity、`url(#linearGradient/radialGradient)` 的轻量渐变填充。
  - 大型 tiger SVG 不再作为默认 ROMFS fixture；V3.8 后使用小型 synthetic overflow SVG 验证 `Overflow` 统计。V2.9 对简单 clipPath/filter blur/drop-shadow 有可见近似，复杂 CSS/mask/filter/SVG gradient spread/pattern 仍 fallback。

## 当前实现进度

截至 2026-05-01，新的 FHRE Rust 原型已经具备这些最小能力：

- `no_std` crate 可独立 `cargo check`。
- `Surface` 可包装 NuttX framebuffer。
- 基础颜色、矩形、点、尺寸类型。
- RGB565/RGB888/RGBA8888 像素写入。
- opaque rectangle fast path，避免所有矩形都走 `put_pixel()`。
- RGB565/RGB888/RGBA8888 软件路径已补第一版 scanline alpha blend fast path，半透明大块控件不再从 `Surface::put_pixel()` 入口逐像素重复做 clip/pixel-format 分派。
- `fill_round_rect()` 已改为中心矩形、左右边条和四角 mask 的分区绘制，圆角控件的大面积部分可以复用矩形 fast path。
- line、wide line、circle、round rect、vertical gradient、5x7 debug text。
- `ImageFormat` / `ImageView` / `builtin_image()` 已具备第一版内存图片视图，`DrawCommand::DrawImage` 已接入真实 blit 路径；当前支持内存中的 RGB565/RGB888/RGBA8888/A8，`ImageView` 已从纯 `'static` 切片扩展为可引用 cache 中 decoded image 的只读视图。
- `ResourceLoader` / `PngInfo` / `PngDecoder` / `PngError` / `DecodedImage` / `ImageCache` 已接入。PNG 解码器来自旧 `wing_old2` 的无外部 crate zlib/PNG 实现并收敛到 FHRE：支持非 interlace、8-bit/16-bit RGB、RGBA、grayscale、grayscale-alpha、indexed palette、`tRNS` gray/RGB/palette alpha 和 PNG filter 0-4；16-bit sample 会轻量降采样到 8-bit，RGB 类资源优先输出 RGB565，带 alpha 的资源输出 RGBA8888。V2.7 的 codec 统计把 image、TTF、SVG 的 missing-resource、invalid、truncated、unsupported、overflow 统一汇入 `CodecStats` / `RenderStats`，同时保留 image cache 自身的 decode 分类；`fhre_demo` 已有 demo-side codec fixture manifest，统一预热 FRAW、RGBA PNG、palette+tRNS PNG、grayscale+tRNS PNG、16-bit gray-alpha PNG、baseline JPEG、小型 TTF/OTF 和 SVG document fixture，并增加 missing/invalid/truncated/unsupported/overflow 负向 fixture 与 expected/actual mismatch HUD。
- `JpegInfo` / `JpegDecodeOptions` / `JpegDecoder` 已接入 JPEG 子集：支持 SOF0/SOF2、DQT、DHT、SOS、DRI restart interval、8-bit、1/3/4 分量、常见 4:4:4/4:2:2/4:2:0/4:1:1 采样、baseline CMYK/YCCK、APP14 transform、EXIF orientation 1-8 和 RGB565 输出；progressive SOF2 子集覆盖 DC/AC scan、successive refinement、EOB run，并在 V3.x 增加 scan 顺序和系数缓冲预算检查。`fhre_demo` 已通过真实 LVGL JPEG fixture 验证 runtime decode/cache/display。progressive+CMYK、ICC/gamma 精确色彩、arithmetic coding 和 12-bit JPEG 会让 `ImageCache` 走 fallback。
- `ImageCache` 使用 `alloc` 存储 decoded bytes，但以有界 slot 和总 decoded bytes 控制资源占用；默认 demo 路径使用 8 个 slot / 2MiB 上限，cache miss 或解码失败时仍可回退到 fallback image/icon。
- `FRAW` raw 图片资源格式已接入：文件签名为 `FHREIMG1`，header 记录 width/height/format/stride/data_len，运行时只需读取 header + 像素数据即可得到 `DecodedImage`。它用于从 LVGL `native-with-alpha` C 数组预提取出来的轻量图标，避免 Wing 运行时解析 LVGL C 文件。
- `ImageCache` 已可按资源签名自动分发 PNG/FRAW decode，demo 默认扩大到 8 个 decoded slot / 2MiB 上限；资源缺失、格式不支持或超限时仍返回 fallback，不让 shell 崩溃。
- `ImageCacheStats` 已区分 `load_failures` 和 `decode_failures`，并进一步记录 `decode_invalid`、`decode_truncated`、`decode_unsupported`、`decode_overflow`，可以判断是 ROMFS/文件路径问题、格式不支持、文件截断、非法数据还是 cache/尺寸超预算。
- `ImageCache::prewarm()` 已接入，可把常用资源加载到 cache 并标记 pinned slot；LRU 逐出会跳过 pinned 资源，全 pinned 或超限时失败并进入统计，不阻塞帧提交。
- Demo runtime 已抽出共享 NuttX glue：`NuttxFramebuffer`、`NuttxInput`、`NuttxResourceLoader`、`now_us()`、`sleep_remaining()` 由 `fhre_demo` / `wing_demo` 共同使用；NuttX FFI 仍停留在 demo/platform 层，不进入 FHRE 上层 API 或 Wing 页面。
- `DrawCommand::DrawImageFit` 和 `DrawCommand::DrawImageTint` 已接入 `ImageFit::{Stretch, Contain, Cover}`，`Surface` 侧支持图片 contain/cover/stretch 缩放和简单 tint 合成，可服务壁纸、live tile、W10M 图标 accent recolor。
- RGB565 软件热路径继续加强：半透明 rect 已有专用 RGB565 alpha blend；circle/triangle 改为按水平 scanline 填充；无缩放 RGB565 image blit 可走逐行 copy。Draw V2/V2.1 样式化命令已接入 `FillStyle`、`GradientStyle`、`BorderStyle`、`ShadowStyle`、`LineStyle`、`ArcStyle`、`MaskSpec`、`LayerSpec`、`ImageDrawStyle`、`TextStyle`、`TriangleStyle` 和 `BlendMode`，用于对齐 LVGL draw 层的 fill/border/shadow/line/arc/mask/layer/text/image/triangle 语义；`BorderStyle` 已有 inside/center/outside 对齐，`LineStyle` 已有 cap/join 字段，`DrawGradientTriangle` 支持顶点色三角形。
- `ImageDrawStyle::clip_radius` 已从“画边框提示”改为真实 rounded mask 裁剪：图片 tile/stretch/contain/cover/tint 仍走统一 `DrawImageStyled`，当圆角裁剪生效时自动进入 mask-aware blend，避免 Shell 或页面层私有裁剪图片。`ImageDrawStyle::blend` 也已经接入真实采样像素合成，`Normal` 保留 RGB565 row-copy/nearest fast path，并在带 mask/rounded clip 时继续走 RGB565 raw alpha blend；`Multiply/Add/Subtract/Difference` 进入可统计的 blend path。
- `GlyphA8` / `GlyphBitmap` / `GlyphView` / `GlyphResolver` / `KerningResolver` / `GlyphCache<GLYPHS, BYTES>` 已具备固定容量接口，当前 5x7 debug font 已改为 A8 glyph path；`GlyphCache::insert_a8_metrics()` 可接收 TTF raster 得到的 A8 glyph，并通过 `Surface::set_glyph_resolver()` 让 `DrawText` 优先走 cached glyph。`GlyphView` 在 RGB565 framebuffer 上已有 A8 raw blend 快路径，支持 clip 和 mask alpha，不再把每个 glyph 像素都退回 `put_pixel()`；`DrawText` 的 newline 已按原始 x 坐标换行推进，`DrawLabel` 的 selection foreground 已按字符区间绘制，不再把相交 selection 的整行文字一起染色；V2.3 新增 `TextStyle::kerning`，打开后由 `Surface::set_kerning_resolver()` 提供 pair adjust，默认关闭。
- `TtfDecoder` / `FontFace` 已接入 TrueType glyf 子集 parser 和 OTF/CFF 识别：支持 sfnt table directory、`cmap` format 4/12、`head/hhea/hmtx/maxp/loca/glyf/kern`、simple glyph 轮廓、基础 composite glyph 平移/缩放、legacy `kern`、GSUB SingleSubst/LigatureSubst、GPOS PairPos format 1/2 和 CFF Type2 最小 raster；`FontFace::kerning()` 可读取 legacy `kern` format 0 pair，并可通过 `KerningResolver` 接入 `DrawLabel`。`fhre_demo` 默认 ROMFS 已改用小型 GPOS TTF 和提取出的 OTF fixture 验收，大型 DejaVuSans 仅保留为源码参考。不做 hinting、完整 CFF/CFF2、WOFF/WOFF2、复杂脚本 shaping 和 bidi 排版。
- `Vec3`、`Transform3D`、`Rotation`、`Projection`、`Camera`。
- `Camera::screen_canvas()` 默认 orthographic 1:1 映射。
- `RenderNode` 可通过 Camera 投影到 Screen Canvas。
- `Vertex3D` / `MeshRef` / `MeshDrawOptions` 已具备第一版固定切片 mesh 引用，`MeshRef::emit_with_options()` 可把 3D 顶点/索引投影成 flat triangle 和 wireframe 命令；`TexturedVertex3D` / `TexturedMeshRef` / `DrawTexturedTriangle` 已接入 affine nearest texture sampling，用于 mesh-heavy 模式展示 textured triangle。Camera 已有 orthographic 和基础 perspective projection，仍不做 z-buffer 和完整材质系统。
- `DrawCommand` / `DrawList` 可表达投影后的平面 primitive、`FillCircle`、`StrokeLine`、triangle placeholder，以及第一批 LVGL draw parity 命令：`FillStyled`、`StrokeStyledLine`、`DrawBorder`、`DrawShadow`、`DrawArc`、`DrawLabel`、`DrawImageStyled`、`DrawMask`、`DrawBlur`、`DrawLayer`。当前新增真实分组绘制命令 `BeginLayer` / `EndLayer`、`PushMask` / `PopMask`、`PushBitmapMask`，用于把 layer、mask stack 和 bitmap A8 mask 纳入统一 `DrawList -> dirty clip -> RenderStats -> present` 管线；旧 `DrawLayer` / `DrawBlur` 保留为兼容 fallback/debug 入口。
- `DrawCommand::bounds()` 已具备基础命令包围盒，用于后续脏区过滤。
- `DrawCommand::clipped_bounds()` 已具备基础命令包围盒与 clip rect 求交能力。
- `DrawCommand::DrawSvgIcon` 已接入 `Surface::draw_svg_icon()`，当前可按 `SvgId` 取内置 SVG path data，经过 `parse_svg_path()` 和 `draw_svg_path()` 光栅化为线段图标；旧的 vector placeholder 只作为未知 icon 的兜底。
- `SvgPathCommand` / `SvgPath<N>` / `SvgCache<ICONS, CMDS>` / `parse_svg_path()` 已具备轻量 SVG path 接口，支持 Bootstrap Icons 常见 `M/L/H/V/C/Q/A/S/T/Z` 的最小解析；`SvgDocument<PATHS, CMDS>` 已扩展为固定容量多 path document，可解析 `viewBox`、多 `path d`、`rect/circle/ellipse/line/polyline/polygon/use`、`fill`、`stroke`、`stroke-width`、`stroke-linecap`、`stroke-linejoin`、inline/class style、linear/radial gradient paint、`opacity`、`fill-rule`、group opacity 和简单 nested `translate/scale/matrix` transform。`SvgResolver` / `SvgDocumentView` / `DrawSvgDocument` 能让真实 SVG 文件解析结果通过 draw command 进入 dirty clip 和 `RenderStats` 管线；closed path 已支持 even-odd 与 nonzero scanline fill，stroke round cap/join 已有可见实现，简单 `clipPath` 映射为路径 bounds clip，小区域 `feGaussianBlur/feDropShadow` 映射为 FHRE blur/shadow 近似；复杂 CSS/mask/filter/pattern/gradient spread 仍 fallback。
- `parse_svg_document_into()` / `SvgDocument::EMPTY` / `SvgDocument::reset()` 已作为 stack-safe SVG document 解析和复用入口稳定下来；`SvgDocumentCache<DOCS, PATHS, CMDS>` 提供固定容量 document-level cache，`prewarm_result()` 只在资源预热阶段读取/解析 SVG，`view()` 供绘制 resolver 查询，`SvgDocumentCacheStats` 暴露 hit/miss/load/fallback/slot 统计。
- `DrawSvgIcon` 已新增 settings gear `SvgId(7)` 的 SVG path，验证 Wing 新页面可以继续通过 `IconNode -> SvgId -> DrawCommand -> FHRE SVG path` 使用统一图标管线，而不是在 Shell 中私有绘制图标。
- `DrawList` 已支持每条绘制命令携带 `Option<Rect>` clip，排序时 clip 与命令一起移动，dirty 过滤时使用 clipped bounds。
- `RenderStats` 已接入 `DrawList::execute_tracked_on()` 和 `execute_dirty_tracked_on()`，可记录 commands seen/drawn、dirty rect 数、clip changes、clipped commands、overflow 和粗略像素成本。
- `RenderStats` 已扩展 frame/present/cache/input/dirty-copy 统计字段，可合并 `FrameStats`、`PresentStats`、`ImageCacheStats` 和 `CodecStats`，并可通过 `merge_draw_stats()` 合并多个 draw pass 的命令、clip、dirty pass 和像素成本。Draw V2.7 统计已经记录 LVGL draw task 对齐口径：fill/border/box-shadow/letter/label/image/layer/line/arc/triangle/mask-rect/mask-bitmap/blur/vector/3D，以及 blend/mask/layer/blur/vector/text/image/3D 汇总、codec missing/invalid/truncated/unsupported/overflow、draw-task fallback、software/accelerated/fallback path hits、per-task dispatch counters、scratch/mask/layer bytes 和 fast path hits，用于 demo HUD 观察 draw/present/cache/late frame 状态。`fhre_demo` 的 HUD 本身也已经通过 `DrawList -> dirty clip -> RenderStats` 绘制，不再绕过 FHRE 绘制管线；HUD 显示上一帧完成后的 aggregate draw stats，并新增 per-task dispatch bars、top fallback task 和 codec fixture pass/mismatch bars，能区分 image/text/vector/layer/mask/blur/3D 等任务的路径命中。
- `LayerScratch` 已具备有界 RGBA8888 离屏 scratch：库默认预算 256KiB，NuttX demo 使用 2MiB。`Surface::draw_layer_commands_impl()` 会把 `BeginLayer..EndLayer` 内命令渲染到局部 scratch，应用 blur/recolor/opacity/layer mask 后再合成回目标 framebuffer；超预算会进入 `layer_alloc_failures/layer_fallbacks` 统计并显示 fallback，不崩溃、不提交半帧。
- `MaskSpec` 已扩展为 rect / rounded rect / bitmap A8 三类，支持 inverted 和 opacity；`Surface` 内部有固定容量 mask stack，`put_pixel`、alpha blend、常见 primitive、image path 在 mask 激活时都会走 mask-aware blend。`PushBitmapMask` 通过 `ImageResolver` 读取 A8/RGBA alpha，资源缺失时按无 mask fallback，避免页面黑屏。
- `DrawShadow` 已改为优先使用 A8 rounded-rect mask + separable box blur + tint composite 生成真实阴影；`DrawBlur` 和 layer blur 使用同一类有界 scratch 策略，blur 半径会 clamp，内存不足或路径失败时只记录 stats 并走 fallback。
- `FramePolicy` / `FrameClock` / `FrameStats` / `PresentStats` 已接入 FHRE runtime 公共层；NuttX demo 不再无条件“绘制后固定 sleep 16.6ms”，而是按 draw + present 实际耗时计算剩余 sleep，超时帧只记录 late/dropped，不补阻塞帧。
- `Surface` 已具备当前 clip rect 状态，`put_pixel()`、矩形快路径和普通 primitive 都会受 clip 限制。
- `DirtyRegion<N>` 已具备固定容量 rect 列表、overflow 标记、full redraw 标记和 touches 判断。
- `DirtyTracker<N>` 已具备 previous/current bounds 合并 helper 和明确帧生命周期；`DrawList::execute_dirty_tracked_on()` 已改成按 dirty rect pass 执行，并把 `dirty_rect ∩ command_clip` 作为实际 clip，避免全屏背景/壁纸命中小 dirty 后仍整屏重画。
- `NuttxFramebuffer::prepare_dirty_frame()` 已修正 page-pan dirty present 的双缓冲一致性：dirty 帧先把当前可见页整页复制到不可见绘制页，再按 dirty rect clip 执行绘制，避免隐藏页非 dirty 区域残留旧内容导致闪烁；overflow/full redraw 不复制，直接整帧重画，copy 成本会进入 stats。
- `DrawList::execute_dirty_on()` 已可按 DirtyRegion 过滤执行命令。
- `Time` / `RenderContext` 已具备最小 frame tick 和 camera/dirty 上下文承载能力。
- `Tween` / `Easing` 已具备无分配、可重复、可 ping-pong 的 fixed-point 采样能力，可服务 UI transition、live tile 和后续游戏对象动画。
- `PointerEvent` / `KeyEvent` / `InputEvent` 已具备平台无关输入事件表示。
- `EventQueue<T, N>` / `InputQueue<N>` 已具备固定容量、无 alloc、overflow 可观测的事件队列。
- `FhreRuntime<INPUT, DIRTY>` 已具备最小 render context + input queue + frame tick 聚合入口。
- `Entity` / `EntityWorld<N>` 已具备固定容量实体分配、generation 校验和 despawn。
- `ComponentStorage<T, N>` 已具备固定容量泛型组件表、insert/remove/get/for_each/for_each_mut 和 retain_alive。
- `ResourceSlot<T>` 已具备最小 typed resource slot。
- `Schedule<C, N>` 已具备固定容量函数指针 system 调度，不依赖宏和动态分配。
- `GameRuntime<ENTITIES, INPUT, DIRTY>` 已把实体世界、render context 和 input queue 组合成游戏运行时雏形。
- `GestureRecognizer` / `GestureEvent` 已具备固定成本 tap、swipe、drag primitive，可供 Wing shell 和后续游戏应用复用。
- `RenderBackend` trait 已存在，当前由 `Surface` 实现。
- `RenderBackend` 已新增 `BackendCapabilities`，当前软件 framebuffer `Surface` 会上报 pixel format、clip、alpha blend、gradient、mask/layer/blur、vector icon、text layout、image transform、triangle/3D 支持，并以 LVGL draw task 口径声明 `draw_fill/draw_border/draw_box_shadow/draw_letter/draw_label/draw_image/draw_layer/draw_line/draw_arc/draw_triangle/draw_mask_rect/draw_mask_bitmap/draw_blur/draw_vector/draw_3d`。V2.7 在 `DrawTaskKind` / `DrawFeatureFlags` / `DrawBackendDispatch` / `DrawPathKind` 之上继续稳定 `RenderBackend::draw_dispatched_command()` 和 `draw_dispatched_layer_commands()` 默认入口，顶层 draw pass 和 layer 内部 draw pass 都通过这个入口提交命令；默认软件 backend 行为不变，后续 NuttX DMA2D/PXP/VG-Lite、Linux framebuffer/DRM/SDL/OpenGL ES backend 可按任务覆写单项能力。
- `DrawChain<const OPS>` / `DrawChainOp` / `DrawChainStats` 已作为 V3.9 compile-only descriptor 边界接入 `DrawList`：dirty clip 后会生成 chain candidate 并调用 `RenderBackend::submit_draw_chain()`。当前 `Surface` 不提交真实硬件链，只返回 unsupported 并保留软件渲染结果；统计可观察 candidate、submitted、fallback、unsupported、overflow 和 top chain task。
- `CodecPipelinePlan<const STAGES>` / `CodecStagePlan` / `CodecPipelineStats` 已作为 V3.9 codec stage skeleton 接入 PNG/JPEG/FRAW/TTF/SVG。当前只描述 read/inspect/header/entropy-or-parse/transform-or-raster/color-or-pack/cache-insert/fallback 阶段，并在默认无 accelerator capability 时记录 unsupported/fallback；真实硬件 decoder 还没有接入。
- `FramebufferBackend` / `InputSource` 平台抽象 trait 已建立，`Surface` 实现 `FramebufferBackend`；NuttX `/dev/fb0`、`/dev/input0`、`/dev/kbd` 的 FFI 仍保留在 demo/platform glue，不进入上层 UI 页面。
- `fhre_demo` 的 NuttX framebuffer glue 已改为优先使用 `yres_virtual >= yres * 2` 的双 framebuffer page pan：绘制发生在不可见页，帧末通过 `FBIOPAN_DISPLAY` 翻页；没有双 framebuffer 时才 fallback 到 `640x480/480x640x32` 静态离屏 backbuffer copy，避免 X11 sim 中直接写可见 framebuffer 导致清屏/重绘中间态可见而闪烁。
- `fhre_demo` 已开始通过 `GameRuntime + Schedule + EntityWorld + ComponentStorage<Transform3D>` 驱动动态对象，并通过 `Transform3D + Camera + DrawList` 绘制主要元素。
- `fhre_demo` 已开始展示 FHRE 能力闭环：dirty `RenderStats`、A8 glyph cache、SVG path cache、image blit、`MeshRef` 投影到 flat triangle、ECS 动态对象和 pointer/input 管线。当前 demo cache 已统一到 8 slots / 2MiB，并支持键盘 `1..0/Q/W` 或方向键切换 `RECT/BORDER/SHADOW/ARC/TEXT/IMAGE/VECTOR/LAYER/MASK/TRIANGLE/3D/STRESS` draw parity 模式。
- `wing_demo` 已开始通过 FHRE 3D Screen Canvas 模型绘制壁纸和主面板，并由 Wing 的 `UiTree/apply_ui_frame` 消费 FHRE dirty/bounds 基础能力。
- Wing 的 `UiTree` 内部已开始复用 FHRE `EntityWorld<N>` 和多个 `ComponentStorage<T, N>`，当前已拆出 `UiNode`、`Parent`、`Children`、`LayoutBox`、`LayoutResult`、`Visual`、`Opacity`、`Clip`、`ResolvedClip`、`TextNode`、`IconNode`、`Animation`、`Button`，验证了 FHRE tiny ECS 可以作为 Shell/UI runtime 的底层节点、父子关系、布局结果、透明度、裁剪、图标、动画和组件存储。
- Wing 已在 `UiTree` 上实现第一版 `LayoutSpace::Parent` 父局部坐标解析，child 节点仍然使用 FHRE `Transform3D + Camera` 投影为 screen-space `LayoutResult`，验证 UI 局部布局可以建立在纯 3D Screen Canvas 模型上。
- Wing 已在 `UiTree` 上实现第一版 `GridLayout` / `LayoutRule::GridCell`，launcher tiles 由固定容量 grid rule 解析到 FHRE screen-space `LayoutResult`，验证 layout 规则可以作为轻量 ECS 组件参与渲染管线。
- Wing 已在 `UiTree` 上实现第一版 `StackLayout` / `LayoutRule::StackItem`，notification quick actions 和 cards 由固定容量 row/column stack rule 解析到 FHRE screen-space `LayoutResult`，验证横向工具栏和纵向列表布局可以继续沿用同一套 3D Screen Canvas 投影路径。
- Wing 已在 `UiTree` 上实现第一版 `ContentInset` / `LayoutSpace::Content`，父节点可声明 content rect，child 节点可相对 content rect 解析到 FHRE screen-space `LayoutResult`，验证 UI 的内容区布局不会破坏 FHRE 纯 3D Screen Canvas 模型。
- Wing 已在 `UiTree` 上实现第一版 layout clip inheritance，`Clip` 保留声明侧 rect，`ResolvedClip` 保存与父级 clip 合成后的最终 screen clip，render/hit-test/dirty 均使用 effective clip。
- `wing_demo` 已修正 NuttX sim 下的启动栈压力：大块 Wing UI 状态和 builder 缓冲改为持久状态，demo task stack 提升到 `32768`，避免 `UiTree` / `UiBuilder` 固定容量数组压垮 8KB 默认栈。
- `wing_demo` 的 NuttX glue 已可非阻塞读取 `/dev/input0` 和 `/dev/kbd`，并转换为 FHRE `InputEvent` 写入 `InputQueue`。
- Wing 的 UI hit-test 已按 `Transform3D -> Camera -> screen rect` 路径实现，验证了 UI action/picking 可以在不破坏纯 3D 世界模型的前提下落在 screen-space 上。
- Wing shell 当前主视觉已不直接调用 framebuffer 绘制 helper，而是通过 `DrawList` 或 `UiBuilder -> UiTree -> DrawCommand` 汇入 FHRE 管线。
- Wing demo 的 Lumia/Windows 10 Mobile Start tiles、底部 nav bar、notification quick action grid、notification cards 已继续复用 FHRE 的 `DrawCommand::FillRoundRect` / `FillRect` / `FillCircle` / `StrokeLine` / `DrawSvgIcon` / `DrawText`，验证 UI 绘制质量改进不需要绕过 FHRE 管线。
- 第一版 shell live tile 动画继续使用 FHRE `Tween` / `Easing`，由 Wing `Animation` component 采样到 opacity，再经 FHRE dirty/render build 路径重绘，后续游戏对象动画可以沿用同一套 fixed-point tween 基础。
- Wing 内置 `SampleApp` 已开始作为 FHRE 游戏/图形能力的全屏应用样例：它不走 Wing 私有直绘，而是用 FHRE `DrawList<96>` 组合 `FillGradient`、`StrokeLine`、`FillCircle`、`DrawTriangle` 和 depth sort，绘制网格地平线、星点和 pseudo-3D wireframe cube，验证 Wing 应用 surface 可以直接承载 FHRE 图形/game loop 风格内容。
- FHRE Rust crate 已完成第二轮目录模块化拆分：`lib.rs` 现在只声明模块和 re-export 公共能力，具体实现已下沉到 `animation/mod.rs`、`backend/mod.rs`、`color/mod.rs`、`dirty/mod.rs`、`draw/mod.rs`、`ecs/mod.rs`、`event/mod.rs`、`geom/mod.rs`、`input/mod.rs`、`math/mod.rs`、`raster/mod.rs`、`runtime/mod.rs`、`schedule/mod.rs`、`scene/mod.rs`、`surface/mod.rs`、`surface_pixels/mod.rs`、`surface_primitives/mod.rs`、`surface_icons/mod.rs`、`surface_text/mod.rs`、`text/mod.rs`。
- `Surface` 已从单文件绘制集合拆成四类内部能力：`surface_pixels` 负责 pixel format read/write 和 opaque fast path，`surface_primitives` 负责 line/circle/round-rect/triangle，`surface_icons` 负责 `SvgId` 内置 path 图标、stroke raster 和未知图标 fallback，`surface_text` 负责内置 debug glyph raster。这个边界对应 LVGL 中 draw unit / blend / text / image 管线的职责拆分，但不引入 LVGL 依赖和对象模型。
- `prelude/mod.rs` 已作为应用侧常用能力入口，demo、Wing 和未来 SDK 可以选择 `use fhre::prelude::*`，而不需要依赖 crate root 内部实现细节。

当前还没有实现：

- 完整 FreeType/OpenType 级字体能力；当前有 TrueType glyf 子集、legacy kern、GPOS/GSUB table presence、OTF/CFF 识别和 CFF Type2 最小 raster，但不支持 hinting、完整 Type2/CFF2、WOFF/WOFF2、bidi、Arabic/Indic shaping 或复杂 OpenType shaping。
- 完整 SVG style/css、filter、mask 和渐变；当前已有轻量 multi-path document/path/cache、shape-to-path、inline/class style、linear/radial gradient、nested transform、group opacity、简单 clipPath/filter 映射、even-odd/nonzero closed path fill、stroke cap/join、内置 SVG path 图标和 stroke-line raster，复杂 CSS selector、复杂 mask/filter、pattern/mesh gradient 仍 fallback。
- 完整 libjpeg 级 JPEG；当前支持 baseline sequential grayscale/YCbCr/CMYK/YCCK、常见采样、DRI restart interval、EXIF orientation 和 progressive grayscale/YCbCr 子集，仍不支持 progressive+CMYK、arithmetic coding、12-bit JPEG 以及 ICC/gamma 精确色彩管理。
- FRAW 目前只作为轻量运行时格式使用，生成工具已沉淀为 `apps/wing/resource/windows10_mobile/extract_fraw.py`，运行时已补 signature/format/flags/stride/data_len 校验和 `FrawError` 结果接口；还没有接入自动构建步骤。
- 资源预算已经前移到 ROMFS manifest 构建期检查，但还没有完整的生产 profile、自动 downscale/quantize/atlas 工具和“源码参考资源 -> 目标轻量资源”的转换流水线。
- 可复用的 Linux/SDL backend 到 `InputEvent` 的完整平台桥接模块；当前已有共享 NuttX demo glue 和 `FramebufferBackend` / `InputSource` trait。
- 真实 DMA2D/PXP/VG-Lite/OpenGL ES draw chain backend 和真实 PNG/JPEG/TTF/SVG 硬件 decoder 还没有实现；V3.9 只提供可接管 descriptor/stage 边界和统计。
- 输入状态聚合、完整 gesture state 和多 stage schedule dispatch。
- 更通用的 query API、parent/child scene graph、滚动容器 mask、flex/flow layout 和 archetype/稠密批处理优化。
- 真实 mesh asset loader、triangle z-buffer、材质系统、光照和 perspective-correct textured triangle；当前只有固定切片 mesh、painter sort、flat/wireframe 和 affine nearest textured triangle。
- 更细粒度的绘制管线优化，例如 draw command 分批、mask/alpha map 复用、blur/shadow SIMD/DMA2D 化、跨命令 image/glyph 批处理、triangle depth 和 backend capability dispatch；当前已完成 `surface` 侧的基础拆分、RGB565 rect/image/glyph 热路径、有界 layer/mask/blur compositor 和 Draw V2.1 能力矩阵，下一步应继续做真实 workload 驱动的热路径优化和 backend capability dispatch，而不是为了文件数量继续拆碎。

## 当前新增 API 边界

本轮需要保持稳定的公开入口：

- LVGL draw task parity matrix：FHRE 对齐的是 LVGL draw 层，不是 LVGL object/widget/style cascade。当前软件 backend 的能力矩阵如下：
  - `FILL`：solid、rounded、vertical/horizontal/linear/radial/conical gradient，受 clip/mask/blend 约束。
  - `BORDER`：full 和 sides，inside/center/outside 对齐，rounded outline 第一版可用。
  - `BOX_SHADOW`：A8 rounded rect mask + separable box blur + tint composite，超预算 fallback。
  - `LETTER/LABEL`：UTF-8 fast path、按 glyph id 绘制的 shaped `GlyphRun`、`GlyphRunItem::char_start/char_len` 源文本范围、固定容量 `GlyphRunCache`、RGB565 A8 glyph raw blend、wrap、align、letter/line spacing、legacy kern/Latin GSUB/GPOS 最小闭环、underline/strikethrough、selection background、selection foreground、missing glyph 占位符；复杂脚本 shaping fallback。
  - `IMAGE`：PNG/FRAW/JPEG cache view、stretch/contain/cover、tile、tint、opacity、rounded clip radius、style blend mode、RGB565 row copy/nearest scale fast path 和 mask-aware normal blend。
  - `LAYER`：bounded RGBA8888 scratch、opacity/recolor/blur/mask composite，超预算进入 stats fallback。
  - `LINE/ARC`：宽线、dash、round/square/butt cap 字段、arc rounded cap。
  - `TRIANGLE`：flat color、vertex color gradient、textured affine nearest triangle。
  - `MASK_RECT/MASK_BITMAP`：rect、rounded rect、A8/RGBA alpha bitmap mask，支持 invert/opacity。
  - `BLUR`：separable box blur，半径 clamp，scratch budget 控制。
  - `VECTOR`：固定容量 SVG document/path，fill/stroke、evenodd/nonzero、shape-to-path、inline/class style、use、linear/radial gradient、简单 clipPath/filter 映射和 transform；复杂 CSS/mask/pattern/gradient spread fallback。
  - `3D`：纯 3D 语义下的 orthographic/perspective projection、painter sort、flat/wire/textured triangle；暂不做 z-buffer/material/light。
- `RenderStats`：由 `DrawList::execute_tracked_on()` / `execute_dirty_tracked_on()` 填充，应用和 demo 可观测渲染成本；当前已包含 blend/mask/layer/blur/vector/text/image/3D 命令数、fallback 次数、codec missing/invalid/truncated/unsupported/overflow、progressive JPEG decode/scan fallback、CFF raster/fallback、OpenType shaping/GSUB/GPOS/KERN、GlyphRunCache hit/miss/insert/evict/overflow、SvgDocumentCache hit/miss/load/fallback/slot、glyph-id draw/codepoint fallback/selection fallback、SVG clip/mask/filter/gradient success/fallback、scratch/mask/layer bytes、scratch peak、layer alloc failure、mask stack overflow、blur/shadow pixels、per-task dispatch counters、V3.9 draw chain counters 和 codec pipeline counters。V3.6 新增 `benchmark_summary()` / top dispatch / top fallback helper，V3.8 新增 `mark_svg_document_cache()`，V3.9 新增 top chain task helper，用于 `fhre_demo` 直接定位每个 draw parity 模式、资源预热和硬件可接管边界的主要成本来源。
- `DrawTaskKind` / `DrawTaskCounters` / `DrawFeatureFlags` / `DrawBackendDispatch` / `DrawPathKind`：以 LVGL draw task 口径声明软件/硬件绘制能力，`BackendCapabilities::supports_draw_task()` 可查询 fill/image/label/vector/layer/mask/blur/3D 等能力是否由当前 backend 支持，dispatch path counters 可区分软件路径、未来加速路径和 fallback 路径；V2.7 后 `DrawList` 和 layer 内部绘制都通过 `RenderBackend::draw_dispatched_command()` / `draw_dispatched_layer_commands()` 提交命令，后端可覆写该入口接管某类任务。
- `DrawChain` / `DrawChainOp` / `DrawChainCapabilities` / `DrawChainStats`：V3.9 硬件可插手 draw descriptor API。当前由 `DrawList::compile_draw_chain()` 在 dirty clip 后生成候选链，并通过 `RenderBackend::submit_draw_chain()` 提交；默认软件 backend 返回 unsupported，未来 DMA2D/GPU backend 只需覆写这个入口。
- Draw V2.7 样式 API：`FillStyle`、`GradientStyle`、`BorderStyle`、`BorderAlign`、`ShadowStyle`、`LineStyle`、`LineCap`、`LineJoin`、`ArcStyle`、`TextStyle`、`ImageDrawStyle`、`TriangleStyle`、`MaskSpec`、`LayerSpec`、`BlendMode`，用于对齐 LVGL draw task 层，不包含 LVGL widget/object/style cascade。`DrawLabel` 已切到 glyph advance line breaker，优先空格换行；`DrawImageStyled::clip_radius` 只做图片裁剪，不再私自附加边框。
- Layer/Mask API：`BeginLayer` / `EndLayer`、`PushMask` / `PopMask`、`PushBitmapMask`、`LayerScratch`、`LayerBudget`、`MaskKind`、`MaskStack<N>`。这些 API 是 FHRE draw 层能力，不引入 LVGL object/widget/style cascade。
- `ImageFormat` / `ImageView`：内存图片视图，服务 RGB565/RGB888/RGBA8888/A8 blit；runtime PNG decode 和 `ImageCache` 输出同一视图，不要求 Wing 页面知道图片来源。
- `ImageFit` / `DrawImageFit` / `DrawImageTint`：图片绘制的缩放和 tint 入口，Shell 页面只声明 fit/tint，具体采样、裁剪、alpha blend 由 FHRE backend 实现。
- `ResourceLoader`：平台无关资源加载入口，NuttX 文件读取、ROMFS、Linux host backend 后续都只实现这个小接口。
- `PngInfo` / `PngDecoder` / `PngError` / `DecodedImage`：无外部 crate 的 PNG inspect/decode API，当前覆盖 8/16-bit 常见颜色类型、`tRNS`、Adam7 interlace、critical chunk CRC、filter 0-4 和 `gAMA/sRGB/iCCP` metadata，并提供 result-style error 边界。
- `JpegInfo` / `JpegDecodeOptions` / `JpegDecoder`：无外部 crate 的 JPEG inspect/decode API，当前覆盖 baseline grayscale/YCbCr/CMYK/YCCK、progressive grayscale/YCbCr 第一版、常见采样、restart interval、APP14 和 EXIF orientation，输出统一 `DecodedImage`，由 `ImageCache` 自动识别；progressive+CMYK、arithmetic coding 和 12-bit 仍明确 fallback。
- `FrawInfo` / `FrawDecoder` / `decode_fraw()`：轻量 raw 图片 decode API，当前覆盖 RGB565/RGB888/RGBA8888/A8，主要承载从 W10M LVGL C 数组提取出的图标资源。
- `ImageCache` / `ImageCacheStats` / `ImageDecodeErrorKind` / `CodecErrorKind` / `CodecStats`：有界 decoded image cache 和统一 codec 统计入口，允许 `alloc` 只存在于资源读取、PNG/FRAW/JPEG decode、TTF/SVG prewarm 和 cache 路径，ECS/UI/draw command 主体仍固定容量优先；统计中区分 hit/miss/load/evict/load failure/decode failure，并把 decode failure 细分为 invalid/truncated/unsupported/overflow，同时把 TTF/SVG missing/invalid/unsupported/overflow 汇总到 codec stats。
- `CodecStageKind` / `CodecStagePlan` / `CodecPipelinePlan` / `CodecAcceleratorCapabilities` / `CodecPipelineStats`：V3.9 硬件可插手 codec stage API。PNG/JPEG/FRAW/TTF/SVG 都可先输出 stage plan，当前不改变软件 decode，只把 hardware candidate、unsupported、fallback 和 overflow 汇入 stats。
- `CodecFixture` / demo-side fixture manifest：`fhre_demo` 私有的验收清单，记录资源路径、目标 `ImageId` / `SvgId`、是否 pinned、预期成功/失败类型以及 fallback 策略；它不是 FHRE core API，但固定了 V2.7 的资源验收入口。`CodecFixtureStats` 只存在于 demo，用于 HUD 展示 fixture pass/mismatch。
- `FramePolicy` / `FrameClock` / `FrameStats` / `PresentStats`：帧节奏和 present 统计公共入口，demo/platform glue 负责提供真实时间和 sleep，FHRE 负责目标帧时长、late/dropped frame 统计和 sleep budget。
- `DirtyTracker<N>`：previous/current bounds dirty helper，适合动画对象、UI 滚动和小块资源更新；overflow 或首帧回退 full redraw。
- `NuttxFramebuffer` / `NuttxInput` / `NuttxResourceLoader`：仅供 demo/platform glue 使用的 NuttX 运行时边界；上层库仍只依赖 FHRE traits 和 draw/resource API。
- `GlyphCache<GLYPHS, BYTES>` / `GlyphView` / `GlyphResolver` / `KerningResolver`：固定容量 glyph bitmap cache，当前接 5x7 A8 debug font，也可接收 TTF 子集 raster 出来的 A8 glyph，并通过 resolver 接入 `DrawText` / `DrawLabel`。
- `GlyphIdResolver` / `GlyphRunResolver` / `GlyphRunCache<RUNS>` / `GlyphRunCacheStats`：V3.5 稳定的 shaped text run 入口和固定容量短文本 cache。`DrawLabel` 先按 glyph id 绘制 shaped `GlyphRun`，再生成 `TextLayout<N>`；cache hit/miss/insert/evict/overflow、glyph-id draw、codepoint fallback 会汇入 `RenderStats`，字体文件解析仍不进入 draw 热路径。
- `TtfDecoder` / `FontFace` / `FontFaceKind` / `OpenTypeLayout` / `ShapeOptions` / `GlyphRun<N>` / `GlyphRasterOptions` / `RasterGlyph`：TrueType glyf 与 OTF/CFF 识别入口，支持 cmap format 4/12、legacy kern format 0 查询、GSUB SingleSubst/LigatureSubst、GPOS PairPos format 1/2 和 CFF Type2 最小 raster，输出 A8 glyph 后可进入 `GlyphCache::insert_a8_metrics()`。
- `SvgDocument<PATHS, CMDS>` / `SvgDocumentView` / `SvgDocumentCache<DOCS, PATHS, CMDS>` / `SvgDocumentCacheStats` / `SvgResolver` / `SvgRasterOptions` / `SvgPath<N>` / `SvgCache<ICONS, CMDS>` / `SvgId` / `SvgFillRule` / `SvgStrokeCap` / `SvgStrokeJoin`：固定容量 path/icon/document cache，当前 parser 能吃 Bootstrap Icons path 子集、多 path document、shape elements、`use href=#id`、inline/class style、fill-rule、group opacity、stroke cap/join、linear/radial gradient、简单 clipPath/mask path、filter 映射和 nested transform，`DrawSvgIcon` / `DrawSvgDocument` 已可从真实 SVG 文件走到 FHRE dirty/stats/raster 管线。document-level cache 只负责预热后的 SVG document 复用，不替代 path-level `SvgCache`。
- `Vertex3D` / `MeshRef` / `MeshDrawOptions` / `TexturedVertex3D` / `TexturedMeshRef` / `TexCoord`：FHRE 3D mesh primitive 的第一版公开模型，当前通过 orthographic/perspective 投影发出 flat triangle、wireframe 和 textured triangle。
- `FramebufferBackend` / `InputSource`：平台抽象边界，NuttX/Linux/SDL 后续只实现 trait 和 glue，不污染 Wing page。

## 当前验收方式

- `cargo check --manifest-path apps/fhre/rust/Cargo.toml` 必须通过。
- `cargo check --manifest-path apps/examples/fhre_demo/rust/Cargo.toml` 必须通过。
- `nuttx/wing_build.sh` 必须能把 `fhre_demo` 和 `wing_demo` 同时编入 NuttX sim。
- `fhre_build.sh` / `wing_build.sh` 必须在 ROMFS 安装阶段执行 resource manifest budget 检查；未列入 manifest 的大资源不会进入镜像，列入 manifest 但超过 `max-bytes` 会直接构建失败。
- NSH 中运行 `fhre_demo` 应能持续运行且不黑屏，看到 gradient、round rect、border、真实 blurred shadow、arc、push/pop mask、bitmap mask、真实 bounded layer opacity/recolor/blur composite、A8 debug text、PNG/FRAW/JPEG/PNG-fixture image path、SVG path icon、真实 SVG document command、flat/vertex-color/textured triangle、mesh wireframe、动态 ECS bubbles 和 dirty/cache/frame/present/Draw V3.9 benchmark bars；FRAW、RGBA PNG、palette+tRNS PNG、gray+tRNS PNG、16-bit gray-alpha PNG、Adam7 PNG、baseline/CMYK/EXIF/progressive JPEG、progressive+CMYK fallback、小型 GPOS TTF、OTF/CFF Type2 glyph、GlyphIdResolver glyph-id draw、GlyphRunCache hit/miss/overflow、SvgDocumentCache hit/miss/load/fallback、draw chain candidate/fallback、codec pipeline candidate/fallback、Bootstrap/grouped/style/use/gradient/clip/mask/filter SVG fixture 以及 missing/invalid/truncated/unsupported/overflow 负向 fixture 启动前 prewarm，绘制热路径只查 cache/resolver，可用 `1..0/Q/W` 或方向键切换 draw parity 压力模式。
- `wing_demo` 的 NuttX glue 会通过 `ResourceLoader + ImageCache` 从 manifest 允许的 `/etc/wing/resource/windows10_mobile/assets/*.png` 和 `/etc/wing/resource/windows10_mobile/raw/*.fraw` 读取 Windows 10 Mobile 资源并转换为 FHRE `DrawImage*` 命令；未安装的大图只显示 fallback 并进入 cache stats。

## alloc 边界

FHRE 仍以 `no_std` 为主线，但从本轮开始允许 `alloc` 出现在资源路径：

```text
ResourceLoader -> Vec<u8> source bytes
PngDecoder     -> DecodedImage Vec<u8>
FrawDecoder    -> DecodedImage Vec<u8>
JpegDecoder    -> DecodedImage Vec<u8>
ImageCache     -> bounded decoded slots
TtfDecoder     -> RasterGlyph Vec<u8>
SvgDocument    -> fixed-capacity parsed path document
```

这些分配不进入 ECS、layout、draw command、gesture、schedule 的热路径。NuttX demo 必须显式提供 `malloc/free/realloc` 全局 allocator；未来 MCU 目标可以替换为固定 arena allocator，而不改变 Wing 页面和 FHRE draw API。

## 参考关系

### 主要参考

- `/home/uan-gpd/codes/lvgl`
  - 参考绘制质量、脏区刷新、mask、RGB565 快路径、文本/图片管线。
  - 不采用 LVGL object tree、style cascade、callback 模型。

- `/home/uan-gpd/codes/bevy`
  - 参考 ECS 思维、Schedule、Resource、Event、App stage。
  - 不复制 Bevy 的宏、反射、插件体量、复杂 SystemParam。

- `/home/uan-gpd/codes/hecs`
  - 参考轻量 ECS 存储和 API 边界。
  - 不直接依赖，不照搬完整动态借用模型。

- `/home/uan-gpd/codes/planck_ecs`
- `/home/uan-gpd/codes/freecs`
  - 参考小型 ECS 的组件存储、查询和调度取舍。

### 历史参考

- `/home/uan-gpd/codes/FeatherOS/apps/hand_down/docs/ARCHITECTURE.md`
- `/home/uan-gpd/codes/FeatherOS/apps/hand_down/apps/fhre`

保留并作为主线的思想：

- Entity + Component + Resource + Event。
- Schedule stage。
- Time / frame loop。
- 输入桥接。
- picking / pointer event。
- 统一 3D 坐标。
- Transform3D。
- Camera / Screen Canvas / scene node。
- Asset handle。
- draw command / render backend。
- 2D UI 是 z=0 Screen Canvas 上的 3D 对象。

不保留的实现重量：

- 所有对象都在 3D 世界中，但不把所有对象强制建模为 mesh/material；UI、sprite、text、icon 可以是带 `Transform3D` 的轻量 3D primitive。
- 不先做 MainWorld + RenderWorld 双世界。
- 不做 Bevy 风格完整 App/Plugin 系统。
- 不做复杂 tuple query 和宏生成 query。
- 不恢复大量模块和 100+ 文件架构。
- 不把第一版目标变成复杂 3D scene graph。

## 平台后端策略

FHRE 第一目标是 NuttX，但接口不能绑定死 NuttX。

第一阶段：

- NuttX framebuffer backend。
- NuttX touch/key input bridge。
- `Surface::from_raw()` 包装 `/dev/fb0` plane。

后续应保留：

- Linux framebuffer backend。
- Linux DRM/KMS backend。
- SDL host backend，用于桌面调试。
- OpenGL ES backend。
- 2D/2.5D accelerator backend，例如 DMA2D、PXP、VG-Lite。

上层永远面向：

```text
Scene3D / DrawList / RenderBackend / BackendCapabilities
```

而不是直接面向 `/dev/fb0` 或某个具体 OS。

## 最新定位

FHRE 分四层实现。

```text
Layer 0: 3D-aware Render Core
  - Surface
  - Pixel format
  - DirtyRegion
  - Clip
  - DrawCommand
  - Camera projection input
  - SoftwareRenderer
  - Backend capabilities

Layer 1: Media / Primitive
  - Font glyph raster/cache
  - SVG icon parser/cache
  - PNG/JPEG decode hook
  - Image blit
  - Rounded rect / mask / gradient

Layer 2: Game Runtime Core
  - Entity
  - Component storage
  - Resource
  - Event
  - Schedule
  - Time / frame tick
  - Game loop
  - Transform3D
  - Orthographic Camera default
  - Sprite / tile / render node
  - Animation
  - Picking / gesture primitive

Layer 3: Mesh / Perspective / 3D Raster Capability
  - affine transform fast path
  - card perspective / pseudo-3D
  - mesh / vertex buffer model
  - Perspective Camera
  - flat triangle rasterization
  - optional textured triangle
  - optional z-sort or tile z-buffer
```

第一阶段重点是 Layer 0、Layer 1 和 Layer 2 的最小闭环。Layer 2 不是只为 Wing 服务，它要能支撑独立 `fhre_demo` 和后续 Wing 游戏应用。Layer 2 必须从一开始使用 `Transform3D + Camera + Screen Canvas`，只是默认相机使用 orthographic 1:1 映射。Layer 3 不是“把 FHRE 变成 3D”的阶段，因为 FHRE 从第一天就是 3D；Layer 3 只是补上 mesh、Perspective camera 和 triangle raster。

## FHRE 不应该做什么

FHRE 不应该变成：

- LVGL 的 Rust 重写版。
- Bevy 的嵌入式缩小版。
- 只为 Wing 写死的 shell renderer。
- 复杂 3D scene graph。
- 复杂窗口系统。
- 复杂资源管理器。

FHRE 应该是轻量游戏/图形引擎，但边界要清楚。它要提供 game loop、输入、scene、资源和渲染能力；不提供编辑器、复杂物理、脚本系统、网络同步、完整材质系统这类大引擎能力。

## 纯 3D 世界模型

FHRE 从第一天就应该采用旧 FHRE 的核心理念：

**FHRE 是纯 3D 引擎，所有可渲染对象都在 3D 世界中。2D 只是对象恰好位于默认 Screen Canvas 平面 `z=0`。**

这里的“所有对象都是 3D”不是说所有对象都必须是 mesh。更准确的工程表达是：

```text
UI rect     = Entity + Transform3D + UIRect primitive, usually z=0
Sprite      = Entity + Transform3D + textured quad primitive, often z=0
Text/Icon   = Entity + Transform3D + text/svg primitive, usually z=0
Tile        = Entity + Transform3D + tile primitive, usually z=0
Mesh        = Entity + Transform3D + vertex/index primitive, arbitrary z
```

这样上层永远只面对 3D 世界坐标、Transform 和 Camera。底层对 `z=0`、无旋转、无透视的 Screen Canvas 对象可以走专门的平面绘制优化，但这只是后端优化，不改变“对象是 3D 对象”的语义。

默认模型：

```text
world space:    x, y, z
screen canvas:  z = 0
default camera: Orthographic, 1:1 pixel mapping
depth/layer:    z + explicit layer
plane optimize: z=0, no rotation, no perspective -> scanline plane command
mesh path:      mesh / perspective / triangle raster when enabled
```

这更适合小屏 UI：

- 大多数 UI 和 sprite 仍然以平面优化成本绘制。
- z/layer 从第一天存在，后续不需要重写排序模型。
- Camera 从第一天存在，后续不需要重写坐标和投影模型。
- 卡片切换、通知面板、磁贴动效可以自然使用 2.5D。
- mesh/triangle 能沿同一套 Transform/Camera 接入。

推荐结构：

```rust
pub struct Vec3 {
    pub x: Fixed16,
    pub y: Fixed16,
    pub z: Fixed16,
}

pub struct Transform {
    pub position: Vec3,
    pub scale: Vec3,
    pub rotation: Rotation,
}

pub enum Rotation {
    None,
    Rotate2D(i16),
    Euler { x: i16, y: i16, z: i16 },
}

pub enum Projection {
    Orthographic { scale: Fixed16 },
    Perspective { fov_y: i16, near: Fixed16, far: Fixed16 },
}

pub struct Camera {
    pub transform: Transform,
    pub projection: Projection,
    pub viewport: Rect,
}

pub struct RenderNode {
    pub bounds: Rect,
    pub transform: Transform,
    pub clip: Option<Rect>,
    pub opacity: u8,
    pub layer: i16,
}
```

第一版可以只真正执行 `position.x/y/z`、orthographic camera、Screen Canvas plane projection 和简单 `scale`，并让 `Rotation::Euler`、`Projection::Perspective` 先作为 API 和数据结构存在。这样不会过早增加软件渲染复杂度，也不会留下“从 2D 改 3D”的 API 债。

## 游戏引擎能力边界

FHRE 应提供这些具体游戏能力：

- 固定或可变 timestep 的 frame loop。
- `Time` resource：delta、frame count、uptime。
- 输入状态：按键、触摸、指针、简单手柄按键。
- Event：已具备固定容量 `EventQueue<T, N>`，可承载按键、pointer、collision/picking 等事件。
- Schedule：Input、Update、Animation、RenderBuild、Render。
- Entity + Component storage。
- Transform3D、Velocity、Sprite、Camera、RenderNode。
- Sprite atlas / tile map 的基础数据结构。
- Animation：frame animation、tween、simple easing。
- Picking / hit test：rect/circle/polygon 的轻量命中。
- Asset handle：font/image/svg/sprite atlas。
- DrawCommand 输出和 backend dispatch。

第一阶段可以不做：

- 完整 2D 物理引擎。
- 骨骼动画。
- 粒子系统。
- 大型 ECS query。
- 多线程渲染。
- 热重载资源。
- 编辑器和场景文件格式。

这意味着 `fhre_demo` 应该像一个小型游戏引擎 demo，而不是只画 UI primitive。

## 3D 图形能力路线

FHRE 可以有 3D 能力，但应分阶段进入。

### 3D-A: 2.5D

- 2D transform + z/layer。
- 卡片缩放、旋转、透视近似。
- app switcher / tile flip / pseudo depth。
- 不需要 z-buffer。

### 3D-B: 轻量 mesh

- `Vec3`、`Mat4`、`Camera3D`。
- Mesh：静态 vertex/index buffer。
- Material：flat color / vertex color。
- 投影到 screen space。
- z-sort triangle 或 object。

### 3D-C: 软件三角形光栅

- flat triangle fill。
- optional wireframe。
- optional backface culling。
- tiny tile z-buffer 或 painter sort。
- 只面向低多边形 demo。

### 3D-D: 加速后端

- OpenGL ES backend。
- 2.5D blitter backend。
- VG-Lite/PXP/DMA2D 能力映射。
- CPU software backend 仍作为 fallback。

这样 FHRE 对游戏应用是有成长空间的，但第一版不会被完整 3D 拖住。

## 渲染管线

FHRE 的核心管线应该简单直接：

```text
Scene3D
  -> Transform3D
  -> Camera projection
  -> Screen Canvas primitive or Mesh primitive
  -> DrawCommand / RenderCommand buffer
  -> dirty/clip filter
  -> backend dispatch
  -> framebuffer flush
```

而不是第一阶段就做：

```text
MainWorld -> RenderWorld -> Extract -> Queue -> Phase -> RenderGraph
```

保留 `extract/queue/render` 的概念，但实现上先压成轻量函数边界。

第一阶段 `DrawCommand` 可以先承接投影后的平面 primitive；后续 `MeshCommand` / `TriangleCommand` 接入同一条 backend dispatch。

推荐命令：

```rust
pub enum DrawCommand {
    Clear(Color),
    SetClip(Rect),
    FillRect { rect: Rect, depth: Fixed16, color: Color },
    FillRoundRect { rect: Rect, radius: u16, color: Color },
    FillGradient { rect: Rect, radius: u16, top: Color, bottom: Color },
    FillCircle { center: Point, depth: Fixed16, radius: u16, color: Color },
    StrokeLine { from: Point, to: Point, width: u16, color: Color },
    DrawText { pos: Point, depth: Fixed16, text: TextSpan, font: FontId, color: Color },
    DrawImage { rect: Rect, depth: Fixed16, image: ImageId, opacity: u8 },
    DrawSvgIcon { rect: Rect, depth: Fixed16, icon: SvgId, color: Color, opacity: u8 },
    DrawTriangle { p0: Point, p1: Point, p2: Point, depth: DepthSpan, color: Color },
}
```

命令缓冲：

```rust
pub struct DrawList<const N: usize> {
    cmds: [DrawCommand; N],
    clips: [Option<Rect>; N],
    len: usize,
}
```

容量溢出策略：

- demo 阶段返回错误并显示 debug。
- shell 阶段可退化为简化绘制。
- 不在热路径动态扩容。

## 软件渲染重点

FHRE 第一版必须把软件渲染做好，因为这决定 MCU/低功耗 MPU 上限。

优先实现：

- RGB565 opaque fill。
- RGB565 alpha blend。
- RGB565 A8 glyph blend。
- RGB565 image blit。
- RGB565 vertical gradient。
- clip rect：已具备 Surface 当前 clip、DrawList per-command clip 和 clipped bounds dirty 过滤的第一版。
- Lumia/Windows 10 Mobile 风格 shell 当前已验证这些 primitive 的组合路径：大面积 tile/panel 用矩形快路径，nav/search glyph 用 line/circle，live tile icon 用 `DrawSvgIcon` path 图标，文字暂时走 debug text。
- dirty region union + simple join。
- scanline 绘制，避免每像素虚调用。

暂缓：

- blur。
- shadow。
- 任意 affine image transform。
- anti-aliased vector path 全量实现。
- 多层离屏 composition。

圆角质量：

- 不要每个像素重复算复杂判断。
- 使用每行 A8 mask 或查表。
- 中间区域走矩形快路径。
- 只有上下圆角区域走 mask。

## 脏区刷新

基础结构：

```rust
pub struct DirtyRegion<const N: usize> {
    rects: [Rect; N],
    len: usize,
    overflowed: bool,
}
```

第一阶段：

- rect 裁剪到屏幕。
- 重叠 rect 合并。
- 数量溢出退化为 union。
- 动画较大时直接 full redraw。
- draw command bounds 过滤。

第二阶段：

- tile 对齐。
- greedy compact。
- frame cost 统计。

## 字体、图片、SVG

字体：

- FHRE 提供 font face、glyph raster/cache。
- 默认支持一个系统字体。
- glyph 输出 A8 bitmap。
- 缺失字符使用占位符。
- 字号缩放在 glyph raster 阶段处理。

SVG：

- 先支持 Bootstrap Icons 这类简单 SVG。
- 支持 `viewBox`、`path d`、`fill`、基础 transform。
- 解析后缓存为轻量 path command。
- 频繁使用的图标可以缓存为 A8 bitmap 或单色 mask。

图片：

- PNG/JPEG 解码可以先用 hook/模块接口占位。
- 运行时读取资源，解码成统一 pixel view。
- 背景图可以保持文件系统资源，由 shell 按窗口缩放。
- MCU 目标应优先使用缩放后的资源或 tiled decode。

## ECS 和游戏运行时取舍

FHRE 需要 ECS 基础能力，因为它要支撑游戏应用和 Wing UI。它不应该变成通用 ECS 框架本体，但必须能让小型游戏舒服地组织状态。

第一阶段推荐：

```rust
pub struct Entity {
    index: u16,
    generation: u16,
}

pub struct Storage<T, const N: usize> {
    // dense storage, fixed capacity
}

pub struct Resources {
    // explicit fields first
}

pub struct Time {
    pub delta_ms: u32,
    pub uptime_ms: u64,
    pub frame: u32,
}
```

查询策略：

- 手写 iterator。
- 显式系统函数，例如 `fn(&mut GameRuntime)`。
- 不做宏。
- 不做复杂 tuple query。
- 不做动态反射。

示例：

```rust
fn animate_tiles(world: &mut World, time: &Time) {
    for item in world.iter_tile_transforms_mut() {
        // update transform
    }
}
```

后续如果需要更通用 query，再从 hecs/freecs/planck_ecs 中吸收轻量做法，但仍然保持无宏和 no_std。

游戏应用第一阶段推荐组件：

```rust
pub struct Transform3D;
pub struct Velocity2D;
pub struct Sprite;
pub struct TileMap;
pub struct Camera;
pub struct Collider2D;
pub struct AnimationPlayer;
```

这些组件足以支撑简单 UI 动效、小游戏、掌机风格 demo，以及后续 Wing 内运行的游戏应用。

## 与 Wing 的边界

FHRE 只知道：

- surface。
- color。
- rect。
- draw command。
- font/image/svg resource。
- input primitive。
- gesture primitive。
- render node。
- entity/storage/resource/event。
- time/frame loop。
- sprite/tile/camera/animation。
- optional mesh/camera3D。

FHRE 不知道：

- Desktop。
- SettingsPage。
- NotificationCard。
- AppSwitcher。
- Theme name。
- Navigation stack。

Wing 负责声明式 UI、状态、导航和 shell 语义。Wing 把这些语义转换成 FHRE 的 render node 和 draw command。

Wing 游戏应用可以直接使用 FHRE 的游戏运行时能力。Shell 只负责启动、退出、手势 overlay、资源目录和生命周期管理。

## 推荐目录

```text
apps/fhre/
├── base/
├── clang/
└── rust/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── animation/mod.rs # Easing, Tween
        ├── backend/mod.rs   # RenderBackend trait
        ├── color/mod.rs     # Color, PixelFormat
        ├── dirty/mod.rs     # DirtyRegion
        ├── draw/mod.rs      # DrawCommand, DrawList, resource ids
        ├── ecs/mod.rs       # EntityWorld, ComponentStorage, ResourceSlot
        ├── event/mod.rs     # fixed-capacity EventQueue
        ├── geom/mod.rs      # Point, Rect, Size
        ├── input/mod.rs     # pointer/key/gesture primitives
        ├── math/mod.rs      # Fixed16 and fixed-point helpers
        ├── prelude/mod.rs   # app-facing common exports
        ├── raster/mod.rs    # low-level raster math helpers
        ├── runtime/mod.rs   # Time, RenderContext, GameRuntime
        ├── schedule/mod.rs  # fixed-capacity system schedule
        ├── scene/mod.rs     # Vec3, Transform3D, Camera, RenderNode
        ├── surface/mod.rs   # framebuffer Surface facade and clip state
        ├── surface_pixels/mod.rs     # pixel format read/write and opaque fast path
        ├── surface_primitives/mod.rs # line, circle, round rect, triangle
        ├── surface_icons/mod.rs      # svg path icons and vector fallback
        ├── surface_text/mod.rs       # built-in debug text raster
        └── text/mod.rs      # built-in debug glyph table
```

不要为了文件数量拆碎。当前拆分只保留对热路径优化有意义的边界：backend、raster math、surface pixel fast path、primitive、text/SVG icon、ECS/runtime/scene。

## fhre_demo 目标

`fhre_demo` 是纯 FHRE 能力演示，和 `wing_demo` 独立。

第一版应该展示：

- clear/fill。
- 圆角矩形。
- 渐变。
- 文本。
- SVG 图标。
- 简单图片 blit。
- sprite 移动。
- tile 或网格背景。
- orthographic camera 跟随或偏移。
- Time/frame counter。
- 简单 animation/tween。
- dirty region debug。
- pointer/tap/drag 反馈。
- 一两个 2.5D 卡片或缩放动画。

后续增强版 `fhre_demo` 应展示一个低多边形 cube 或 wireframe/flat triangle demo，用来证明 FHRE 的 3D 图形路线。不要在 `fhre_demo` 里实现 shell 页面，那是 Wing 的职责。

## 实现里程碑

### M0: 构建与目录稳定

- `apps/fhre/base`
- `apps/fhre/clang`
- `apps/fhre/rust`
- `apps/examples/fhre_demo/base`
- `apps/examples/fhre_demo/clang`
- `apps/examples/fhre_demo/rust`
- `fhre_build.sh` 可执行 `fhre_demo`
- `wing_build.sh` 可执行 `fhre_demo` 和 `wing_demo`

### M1: Render Core

- math/color/rect。
- surface。
- RGB565 fill。
- draw command buffer。
- dirty region。

### M2: 软件绘制质量

- scanline rect。
- alpha blend。
- round rect mask。
- gradient。
- line。

### M3: 文本和图标

- font glyph A8。
- glyph cache。
- SVG icon parser/cache。
- image view/blit。

### M4: 输入与游戏运行时

- pointer/key primitive。
- tap/swipe/drag：已具备最小 `GestureRecognizer` primitive。
- Time resource：已具备最小 `Time`。
- frame loop：已在 `FhreRuntime` / `GameRuntime` 中具备最小 tick。
- system schedule：已具备固定容量 `Schedule<C, N>`。
- Entity + Component storage：已具备 `EntityWorld<N>` 和 `ComponentStorage<T, N>`。
- Transform3D：已具备。
- sprite/tile/orthographic camera。
- animation/tween：已具备第一版 `Tween` / `Easing` fixed-point 采样，后续接入 Transform/Sprite 动画 system。

### M5: 轻量 scene 和 picking

- render node。
- layer/z order。
- basic picking。

### M6: Wing 接入

- Wing 声明式 UI 输出 render node。
- layout 输出 draw command，第一版父局部坐标和父 content 坐标 layout 已接入 Wing `UiTree`。
- shell 不直接操作 framebuffer。

### M7: 3D 图形能力

- Vec3/Mat4。
- Perspective camera。
- static mesh。
- flat triangle rasterization：已具备第一版 `DrawCommand::DrawTriangle -> Surface::fill_triangle()`，当前用于 `fhre_demo` 和 Wing `SampleApp` 的低多边形/pseudo-3D 验证。
- wireframe debug：已可用 `StrokeLine` 组合表达，当前用于 Wing `SampleApp` 的 cube edge。
- z-sort 或 tiny tile z-buffer。

### M8: 加速后端预留

- backend capabilities：已具备第一版 `BackendCapabilities`，可表达软件/2D 加速/3D 加速和基础 primitive 支持。
- command dispatch。
- tile buffer。
- DMA2D/PXP/VG-Lite/OpenGL ES 接口草案。

## 验收标准

- `./fhre_build.sh` 无误。
- `./wing_build.sh` 无误。
- `nsh> fhre_demo` 独立运行。
- `nsh> wing_demo` 独立运行。
- FHRE Rust 保持 `no_std`。
- 不引入外部 Rust crate。
- 不引入宏驱动 ECS。
- `fhre_demo` 能证明绘制核心质量和基础游戏运行时能力。
- `wing_demo` 能证明声明式 ECS UI 方向。
