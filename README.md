# FeatherOS + FHRE

这是针对 NuttX + Rust 的轻量图形引擎 FHRE 与 HUD 演示工程骨架。当前主线优先推进 FHRE 底座，Wing 仅用于保持验证，不新增功能性 UI 页面。

## 快速构建（NuttX sim）

在 `nuttx` 目录执行：

```bash
cd nuttx
./fhre_build.sh
./wing_build.sh
```
- `./fhre_build.sh`
  - 配置 `sim:fhre`
  - 编译 NuttX 镜像
  - 最终产物为 `./nuttx`
- `./wing_build.sh`
  - 配置 `sim:wing`
  - 编译并将 `fhre_demo` 与 `wing_demo` 同时链接
  - 保持兼容性验证：即使 Wing 本体未扩功能，也要能通过 `fhre_demo` 运行链路

## 运行

```bash
./nuttx
# 在 NSH 中执行:
fhre_demo
```

`wing_build.sh` 场景下可继续执行：

```bash
fhre_demo
wing_demo
```

## V4 当前验收要点

- draw chain 有候选 run 时优先提交，失败仅回退对应 run；
- `ResourceDecodeResult` 统一支持“成功 / 占位 / 失败”；
- `codec` 与 `draw chain` 的关键统计可见，HUD 支持回退与候选命中可观测；
- 不新增额外宏化抽象，不依赖 Wing 侧新增实现。
