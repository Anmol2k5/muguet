#!/bin/bash
# Muguet Android build: native Rust runtime for aarch64-linux-android.
# Usage: scripts/build-android.sh [check|build|release|clippy|test-norun|apk]  (default: check)
# Requires: Rust 1.98.1 + aarch64-linux-android std, Android SDK + NDK r27.
# Env knobs: ANDROID_HOME / ANDROID_SDK_ROOT, ANDROID_NDK_HOME, NDK_API (default 29).
set -euo pipefail

MODE="${1:-check}"
TARGET=aarch64-linux-android
# NDK API level for the C toolchain/linker. Must stay <= minSdk (29) so the
# linker refuses symbols newer than Android 10 instead of crashing on old devices.
API="${NDK_API:-29}"

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
# Pick up the standard cargo location for non-interactive shells.
if [ -d "$HOME/.cargo/bin" ]; then export PATH="$HOME/.cargo/bin:$PATH"; fi
CC_TGT="aarch64-linux-android$API-clang"
export ANDROID_NDK_HOME="$NDK"
export "CC_aarch64_linux_android=$CC_TGT"
export "CXX_aarch64_linux_android=aarch64-linux-android$API-clang++"
export "AR_aarch64_linux_android=llvm-ar"
export "CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=$CC_TGT"
# 16 KB page-size compatibility (Android 15+ devices): align ELF segments to
# 16384. Verified with llvm-readelf (LOAD Align must read 0x4000).
export "CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS=-C link-arg=-Wl,-z,max-page-size=16384 -C link-arg=-Wl,--build-id=sha1"
# cmake crate (audiopus_sys -> vendored opus): NDK toolchain + NMake/VS-free generator.
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export "CMAKE_TOOLCHAIN_FILE_aarch64_linux_android=$REPO_ROOT/scripts/android.toolchain.cmake"
if [ "$PREBUILT" != "windows-x86_64" ] && command -v ninja >/dev/null; then
  export CMAKE_GENERATOR=Ninja
fi

command -v cargo >/dev/null || { echo "cargo not found (install Rust 1.98.1 via rustup)" >&2; exit 1; }
command -v rustc >/dev/null || { echo "rustc not found (install Rust 1.98.1 via rustup)" >&2; exit 1; }
rustup target list --installed 2>/dev/null | grep -q aarch64-linux-android || { echo "rust target aarch64-linux-android missing: rustup target add aarch64-linux-android" >&2; exit 1; }
[ -x "$BIN/$CC_TGT" ] || { echo "NDK clang missing: $BIN/$CC_TGT" >&2; exit 1; }
command -v cmake >/dev/null || { echo "cmake not found (3.22+ required)" >&2; exit 1; }
command -v java >/dev/null || { echo "java not found (JDK 17+ required)" >&2; exit 1; }
[ -d "$SDK" ] || { echo "Android SDK not found at $SDK (set ANDROID_HOME)" >&2; exit 1; }

case "$MODE" in
  check)   exec cargo check --target "$TARGET" ;;
  build)   exec cargo build --target "$TARGET" ;;
  release) exec cargo build --release --target "$TARGET" ;;
  clippy)  exec cargo clippy --target "$TARGET" ;;
  test-norun) exec cargo test --target "$TARGET" --no-run ;;
  apk)
    cargo build --release --target "$TARGET"
    SO="target/$TARGET/release/libmuguet.so"
    [ -f "$SO" ] || { echo "release build did not produce $SO (refusing to package a stale library)" >&2; exit 1; }
    JNIDIR="android/app/src/main/jniLibs/arm64-v8a"
    mkdir -p "$JNIDIR"
    rm -f "$JNIDIR/libmuguet.so"
    cp "$SO" "$JNIDIR/libmuguet.so"
    echo "[apk] staged $SO"
    echo "[apk] build-id: $(llvm-readelf -n "$SO" | grep -m1 'Build ID' || true)"
    GRADLE_BIN="${GRADLE_BIN:-gradle}"
    (cd android && "$GRADLE_BIN" :app:assembleDebug)
    APK="android/app/build/outputs/apk/debug/app-debug.apk"
    [ -f "$APK" ] || { echo "assemble succeeded but $APK is missing" >&2; exit 1; }
    BT="$(ls -d "$SDK"/build-tools/* 2>/dev/null | sort -V | tail -1)"
    BADGING="$("$BT/aapt2" dump badging "$APK")"
    for want in "package: name='dev.muguet.android'" "native-code: 'arm64-v8a'" "launchable-activity"; do
      echo "$BADGING" | grep -qF "$want" || { echo "APK verification failed: missing [$want]" >&2; exit 1; }
    done
    echo "[apk] verified: dev.muguet.android, arm64-v8a, NativeActivity launchable"
    echo "[apk] $APK"
    sha256sum "$APK" 2>/dev/null || shasum -a 256 "$APK"
    ;;
  *) echo "usage: $0 [check|build|release|clippy|test-norun|apk]" >&2; exit 1 ;;
esac
