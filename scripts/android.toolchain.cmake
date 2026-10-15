# Muguet: NDK toolchain wrapper for crate CMake builds (audiopus_sys -> opus).
# The `cmake` crate does not pass ANDROID_ABI/ANDROID_PLATFORM itself, so pin
# them here and delegate to the NDK's own toolchain file.
# ANDROID_NDK_HOME must be set by the calling build script.
set(ANDROID_ABI "arm64-v8a")
set(ANDROID_PLATFORM "android-29")
set(ANDROID_STL "c++_static")
include("$ENV{ANDROID_NDK_HOME}/build/cmake/android.toolchain.cmake")
