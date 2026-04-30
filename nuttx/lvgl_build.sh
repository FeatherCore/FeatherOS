#!/bin/bash

# FeatherOS LVGL simulator build script
# Usage:
#   ./lvgl_build.sh
#   LVGL_SRC_REPO=/home/uan-gpd/codes/lvgl LVGL_REF=v9.2.1 ./lvgl_build.sh

set -e

LVGL_SRC_REPO="${LVGL_SRC_REPO:-/home/uan-gpd/codes/lvgl}"
LVGL_REF="${LVGL_REF:-v9.2.1}"
LVGL_APP_DIR="../apps/graphics/lvgl"
LVGL_LINK="${LVGL_APP_DIR}/lvgl"

normalize_timestamp() {
    if [ -e "$1" ]; then
        touch -d '2 seconds ago' "$1" 2>/dev/null || touch -c "$1"
    fi
}

normalize_dependency_timestamps() {
    find . ../apps -type f \( -name 'Make.dep' -o -name '.depend' \) \
        -exec touch -d '2 seconds ago' {} + 2>/dev/null || true
}

prepare_lvgl_source() {
    if git -C "${LVGL_SRC_REPO}" rev-parse --verify "${LVGL_REF}^{commit}" >/dev/null 2>&1; then
        local expected
        expected="$(git -C "${LVGL_SRC_REPO}" rev-parse --verify "${LVGL_REF}^{commit}")"

        if [ -e "${LVGL_LINK}/.git" ] &&
            git -C "${LVGL_LINK}" rev-parse --verify HEAD >/dev/null 2>&1 &&
            [ "$(git -C "${LVGL_LINK}" rev-parse --verify HEAD)" = "${expected}" ]; then
            echo "Using LVGL source: ${LVGL_LINK} (${LVGL_REF})"
            return
        fi

        echo "Preparing LVGL source from local git repo:"
        echo "  repo: ${LVGL_SRC_REPO}"
        echo "  ref:  ${LVGL_REF}"

        if [ -L "${LVGL_LINK}" ]; then
            rm -f "${LVGL_LINK}"
        elif [ -e "${LVGL_LINK}" ]; then
            git -C "${LVGL_SRC_REPO}" worktree remove --force "$(readlink -f "${LVGL_LINK}")" 2>/dev/null ||
                rm -rf "${LVGL_LINK}"
        fi

        git -C "${LVGL_SRC_REPO}" worktree prune >/dev/null 2>&1 || true
        git -C "${LVGL_SRC_REPO}" worktree add --detach "$(readlink -m "${LVGL_LINK}")" "${LVGL_REF}" >/dev/null
        return
    fi

    echo "Local LVGL ref was not found:"
    echo "  repo: ${LVGL_SRC_REPO}"
    echo "  ref:  ${LVGL_REF}"
    echo "The NuttX LVGL package makefile will try to download LVGL."
}

echo "=========================================="
echo "FeatherOS LVGL simulator build"
echo "=========================================="
echo ""

echo "[1/4] Clean project..."
make distclean
echo "Done"
echo ""

echo "[2/4] Prepare LVGL source..."
prepare_lvgl_source
echo "Done"
echo ""

echo "[3/4] Configure project (sim:lvgl_fb)..."
./tools/configure.sh sim:lvgl_fb
mkdir -p ../apps/builtin/registry
normalize_timestamp .config
normalize_timestamp .config.old
normalize_dependency_timestamps
echo "Done"
echo ""

echo "[4/4] Build project..."
make -j
echo "Done"
echo ""

if [ -f "nuttx" ]; then
    echo "=========================================="
    echo "Build succeeded"
    echo "=========================================="
    echo "Executable: ./nuttx"
    echo "Size: $(ls -lh nuttx | awk '{print $5}')"
    echo ""
    echo "Run:"
    echo "  ./nuttx"
    echo ""
    echo "The sim:lvgl_fb config starts lvgl_window10_main directly."
    echo ""
else
    echo "=========================================="
    echo "Build failed"
    echo "=========================================="
    exit 1
fi
