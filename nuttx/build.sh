#!/bin/bash

# FeatherOS FHRE 构建脚本
# 使用方法: ./build.sh

set -e  # 遇到错误立即退出

echo "=========================================="
echo "FeatherOS FHRE 构建脚本"
echo "=========================================="
echo ""

# 步骤 1: 清理项目
echo "[1/3] 清理项目..."
make distclean
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
    echo ""
else
    echo "=========================================="
    echo "构建失败!"
    echo "=========================================="
    exit 1
fi
