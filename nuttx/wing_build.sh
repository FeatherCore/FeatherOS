#!/bin/bash

# FeatherOS Wing 构建脚本
# 使用方法: ./wing_build.sh

set -e

normalize_timestamp() {
    if [ -e "$1" ]; then
        touch -d '2 seconds ago' "$1" 2>/dev/null || touch -c "$1"
    fi
}

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
        STASHED_INCLUDE_ARCH="$(mktemp -d "${TMPDIR:-/tmp}/wing-include-arch.XXXXXX")"
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
echo "FeatherOS Wing 构建脚本"
echo "=========================================="
echo ""

prepare_tracked_include_arch

sanitize_app_kconfig_sources

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

echo "[2/3] 配置项目 (sim:wing)..."
./tools/configure.sh sim:wing
mkdir -p ../apps/builtin/registry
normalize_timestamp .config
normalize_timestamp .config.old
echo "✓ 配置完成"
echo ""

echo "[3/3] 编译项目..."
# sim:wing links two independent Rust no_std staticlibs; both carry the
# minimal Rust panic/runtime symbols, so the intermediate relocatable link must
# tolerate duplicate definitions.
make -j LDLINKFLAGS+=" --allow-multiple-definition"
echo "✓ 编译完成"
echo ""

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
    echo "  wing_demo"
    echo ""
else
    echo "=========================================="
    echo "构建失败!"
    echo "=========================================="
    exit 1
fi
