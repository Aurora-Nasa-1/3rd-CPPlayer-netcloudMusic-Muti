#!/usr/bin/env bash
#
# 构建 CPPlayer 的 JNI 音源模块（ncm-api-rs）。
#
# 用法：
#   ./build_module.sh                 # 按当前 OS 构建桌面端（linux / macos / windows）
#   ./build_module.sh android         # Android 三架构：arm64-v8a / armeabi-v7a / x86_64
#   ./build_module.sh linux|windows|macos
#   ./build_module.sh --legacy ...    # 额外导出历史宿主包名的 JNI 符号
#
# 产物：target/ncm-api-rs-<platform>.zip，可直接导入 CPPlayer。
# 打包逻辑统一在 scripts/package_module.py（三端共用，避免 shell 与 PowerShell
# 两份脚本各自漂移）。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

PLATFORM=""
LEGACY=0
for arg in "$@"; do
    case "$arg" in
        --legacy) LEGACY=1 ;;
        -h|--help)
            sed -n '2,16p' "$0"
            exit 0
            ;;
        *) PLATFORM="$arg" ;;
    esac
done

# 未显式指定平台时按当前 OS 推导（MSYS/Git Bash 下 uname 会是 MINGW*/MSYS*）
if [ -z "$PLATFORM" ]; then
    case "$(uname -s)" in
        Linux*)  PLATFORM="linux" ;;
        Darwin*) PLATFORM="macos" ;;
        MINGW*|MSYS*|CYGWIN*) PLATFORM="windows" ;;
        *) echo "无法识别当前系统: $(uname -s)，请显式指定平台" >&2; exit 1 ;;
    esac
fi

case "$PLATFORM" in
    android|linux|windows|macos) ;;
    *) echo "未知平台: $PLATFORM（可选 android / linux / windows / macos）" >&2; exit 1 ;;
esac

# Python：打包脚本在三个 CI runner 上都有，只是名字可能是 python3
PYTHON="${PYTHON:-}"
if [ -z "$PYTHON" ]; then
    if command -v python3 >/dev/null 2>&1; then PYTHON=python3; else PYTHON=python; fi
fi

FEATURES="jni"
if [ "$LEGACY" = "1" ]; then
    FEATURES="jni,legacy-jni-symbols"
fi

echo "==> 平台: $PLATFORM    features: $FEATURES"

case "$PLATFORM" in
    android)
        ABIS=("arm64-v8a" "armeabi-v7a" "x86_64")
        if ! command -v cargo-ndk >/dev/null 2>&1; then
            echo "Error: 未安装 cargo-ndk，请先执行: cargo install cargo-ndk" >&2
            exit 1
        fi
        if [ -z "${ANDROID_NDK_HOME:-}" ] && [ -d "$HOME/Android/Sdk/ndk" ]; then
            LATEST_NDK="$(ls -d "$HOME"/Android/Sdk/ndk/* 2>/dev/null | sort -V | tail -n 1)"
            if [ -n "$LATEST_NDK" ]; then
                export ANDROID_NDK_HOME="$LATEST_NDK"
                echo "Set ANDROID_NDK_HOME=$ANDROID_NDK_HOME"
            fi
        fi
        # 16KB page size 兼容（Android 15+ 要求）
        export RUSTFLAGS="-Clink-arg=-Wl,-z,max-page-size=16384"

        echo "==> cargo ndk: ${ABIS[*]}"
        cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 \
            -o ./target/jniLibs build --release --features "$FEATURES,jni-android"

        LIB_ARGS=()
        for abi in "${ABIS[@]}"; do
            SO_FILE="$(find "./target/jniLibs/$abi" -maxdepth 1 -name 'lib*.so' | head -n 1)"
            if [ -z "$SO_FILE" ] || [ ! -f "$SO_FILE" ]; then
                echo "Error: ./target/jniLibs/$abi 下没找到 .so" >&2
                exit 1
            fi
            LIB_ARGS+=("--lib" "$abi=$SO_FILE")
        done
        ;;

    linux)
        TRIPLE="${TARGET_TRIPLE:-x86_64-unknown-linux-gnu}"
        LIB_NAME="libncm_api_rs.so"
        ABI="x86_64"
        echo "==> cargo build --target $TRIPLE"
        cargo build --release --features "$FEATURES" --target "$TRIPLE"
        LIB_ARGS=("--lib" "$ABI=target/$TRIPLE/release/$LIB_NAME")
        ;;

    windows)
        TRIPLE="${TARGET_TRIPLE:-x86_64-pc-windows-msvc}"
        LIB_NAME="ncm_api_rs.dll"
        ABI="x86_64"
        echo "==> cargo build --target $TRIPLE"
        cargo build --release --features "$FEATURES" --target "$TRIPLE"
        LIB_ARGS=("--lib" "$ABI=target/$TRIPLE/release/$LIB_NAME")
        ;;

    macos)
        if [ "$(uname -m)" = "arm64" ]; then
            TRIPLE="${TARGET_TRIPLE:-aarch64-apple-darwin}"
            ABI="arm64"
        else
            TRIPLE="${TARGET_TRIPLE:-x86_64-apple-darwin}"
            ABI="x86_64"
        fi
        LIB_NAME="libncm_api_rs.dylib"
        echo "==> cargo build --target $TRIPLE"
        cargo build --release --features "$FEATURES" --target "$TRIPLE"
        LIB_ARGS=("--lib" "$ABI=target/$TRIPLE/release/$LIB_NAME")
        ;;
esac

echo "==> 打包"
"$PYTHON" scripts/package_module.py --platform "$PLATFORM" "${LIB_ARGS[@]}"
