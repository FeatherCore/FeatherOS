#!/bin/bash

# FeatherOS FHRE 构建脚本
# 使用方法: ./fhre_build.sh

set -e  # 遇到错误立即退出

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

STASHED_INCLUDE_ARCH=""

restore_tracked_include_arch() {
    if [ -n "$STASHED_INCLUDE_ARCH" ] && [ -e "$STASHED_INCLUDE_ARCH" ]; then
        if [ -L "include/arch" ]; then
            rm -f include/arch
        elif [ -e "include/arch" ]; then
            echo "警告: include/arch 已存在，保留临时目录 $STASHED_INCLUDE_ARCH" >&2
            return
        fi

        mkdir -p include
        mv "$STASHED_INCLUDE_ARCH" include/arch
        echo "已恢复仓库 include/arch 源码目录"
    fi
}

restore_dummy_kconfig_placeholders() {
    if [ -f "arch/dummy/dummy_kconfig" ]; then
        cp -f arch/dummy/dummy_kconfig arch/dummy/Kconfig
    fi

    if [ -f "boards/dummy/dummy_kconfig" ]; then
        cp -f boards/dummy/dummy_kconfig boards/dummy/Kconfig
    fi
}

sanitize_app_kconfig_sources() {
    local apps_dir="../apps"

    if [ ! -d "$apps_dir" ]; then
        return
    fi

    find "$apps_dir" -name Kconfig -type f -print0 \
        | xargs -0 sed -i -E 's#source "[^"]*/FeatherOS/apps/#source "$APPSDIR/#g'
}

cleanup_feather_build() {
    local status=$?
    set +e
    restore_tracked_include_arch
    restore_dummy_kconfig_placeholders
    exit "$status"
}

prepare_tracked_include_arch() {
    if [ -d "include/arch" ] && [ ! -L "include/arch" ]; then
        STASHED_INCLUDE_ARCH="$(mktemp -d "${TMPDIR:-/tmp}/fhre-include-arch.XXXXXX")"
        rmdir "$STASHED_INCLUDE_ARCH"
        mv include/arch "$STASHED_INCLUDE_ARCH"
        echo "临时移开仓库 include/arch 源码目录，避免 NuttX dirlink 冲突"
    fi
}

clean_sim_romfs_state() {
    local sim_src="boards/sim/sim/sim/src"

    if [ ! -d "$sim_src" ]; then
        return
    fi

    rm -rf "$sim_src/etc/fhre" "$sim_src/etc/wing"
    rm -f "$sim_src/etctmp.c" "$sim_src/etctmp.o" "$sim_src/romfs.img"
}

trap cleanup_feather_build EXIT

echo "=========================================="
echo "FeatherOS FHRE 构建脚本"
echo "=========================================="
case "${FHRE_SIM_OPENGL:-0}" in
    1)
        echo "FHRE_SIM_OPENGL=1: 启用 FHRE sim OpenGL packet probe feature（默认仍走 /dev/fb0 present）"
        ;;
    egl)
        echo "FHRE_SIM_OPENGL=egl: 启用 FHRE EGL/OpenGL offscreen shim feature（默认仍走 /dev/fb0 present）"
        ;;
esac
echo ""

prepare_tracked_include_arch

sanitize_app_kconfig_sources

# 步骤 1: 清理项目
echo "[1/3] 清理项目..."
if [ -f "Make.defs" ]; then
    make distclean
else
    if [ -f ".config" ] || [ -f ".config.old" ]; then
        rm -f .config .config.old
        echo "检测到不完整 NuttX 配置，已清理 .config"
    fi
    echo "未检测到既有 NuttX 配置，跳过 distclean"
fi
clean_sim_romfs_state
echo "✓ 清理完成"
echo ""

# 步骤 2: 配置项目
echo "[2/3] 配置项目 (sim:fhre)..."
./tools/configure.sh sim:fhre
echo "✓ 配置完成"
echo ""

# 步骤 3: 编译项目
echo "[3/3] 编译项目..."
make -j
echo "✓ 编译完成"
echo ""

# 检查构建结果
if [ -f "nuttx" ]; then
    echo "=========================================="
    echo "构建成功!"
    echo "=========================================="
    echo "可执行文件: ./nuttx"
    echo "文件大小: $(ls -lh nuttx | awk '{print $5}')"
    echo ""
echo "运行命令:"
echo "  ./nuttx"
echo "  fhre_demo"
echo ""
else
    echo "=========================================="
    echo "构建失败!"
    echo "=========================================="
    exit 1
fi
