#!/bin/bash

# FeatherOS Wing 构建脚本
# 使用方法: ./wing_build.sh

set -e

normalize_timestamp() {
    if [ -e "$1" ]; then
        touch -d '2 seconds ago' "$1" 2>/dev/null || touch -c "$1"
    fi
}

echo "=========================================="
echo "FeatherOS Wing 构建脚本"
echo "=========================================="
echo ""

echo "[1/3] 清理项目..."
make distclean
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
