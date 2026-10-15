#!/bin/bash
# Muguet Android build: native Rust runtime for aarch64-linux-android.
# Usage: scripts/build-android.sh [check|build|release]  (default: check)
# Requires: Rust 1.98.1 + aarch64-linux-android std, Android SDK + NDK r27.
# Env knobs: ANDROID_HOME / ANDROID_SDK_ROOT, ANDROID_NDK_HOME, NDK_API (default 35).
set -euo pipefail

MODE="${1:-check}"
TARGET=aarch64-linux-android
API="${NDK_API:-35}"

SDK="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Library/Android/sdk}}"
if [ -n "${ANDROID_NDK_HOME:-}" ]; then NDK="$ANDROID_NDK_HOME"; else NDK="$SDK/ndk/27.0.12077973"; fi
case "$(uname -s)" in
  Darwin) PREBUILT=darwin-x86_64 ;;
  Linux)  PREBUILT=linux-x86_64 ;;
  MINGW*|MSYS*|CYGWIN*) PREBUILT=windows-x86_64 ;;
  *) echo "unsupported host: $(uname -s)" >&2; exit 1 ;;
esac
BIN="$NDK/toolchains/llvm/prebuilt/$PREBUILT/bin"
if [ ! -d "$BIN" ]; then echo "NDK not found at $NDK (set ANDROID_NDK_HOME)" >&2; exit 1; fi

export PATH="$BIN:$PATH"
CC_TGT="aarch64-linux-android$API-clang"
export ANDROID_NDK_HOME="$NDK"
export "CC_aarch64_linux_android=$CC_TGT"
export "CXX_aarch64_linux_android=aarch64-linux-android$API-clang++"
export "AR_aarch64_linux_android=llvm-ar"
export "CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=$CC_TGT"
# cmake crate (audiopus_sys -> vendored opus): NDK toolchain + NMake/VS-free generator.
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export "CMAKE_TOOLCHAIN_FILE_aarch64_linux_android=$REPO_ROOT/scripts/android.toolchain.cmake"
if [ "$PREBUILT" != "windows-x86_64" ] && command -v ninja >/dev/null; then
  export CMAKE_GENERATOR=Ninja
fi

command -v cargo >/dev/null || { echo "cargo not found" >&2; exit 1; }
command -v rustc >/dev/null || { echo "rustc not found" >&2; exit 1; }

case "$MODE" in
  check)   exec cargo check --target "$TARGET" ;;
  build)   exec cargo build --target "$TARGET" ;;
  release) exec cargo build --release --target "$TARGET" ;;
  *) echo "usage: $0 [check|build|release]" >&2; exit 1 ;;
esac
