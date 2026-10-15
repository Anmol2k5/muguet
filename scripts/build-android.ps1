# Muguet Android build (Windows mirror of scripts/build-android.sh).
# Usage: scripts\build-android.ps1 [check|build|release]  (default: check)
param([string]$Mode = "check")
$ErrorActionPreference = "Stop"

$Api = if ($env:NDK_API) { $env:NDK_API } else { "35" }
$Sdk = if ($env:ANDROID_HOME) { $env:ANDROID_HOME } elseif ($env:ANDROID_SDK_ROOT) { $env:ANDROID_SDK_ROOT } else { "$env:LOCALAPPDATA\Android\Sdk" }
$Ndk = if ($env:ANDROID_NDK_HOME) { $env:ANDROID_NDK_HOME } else { "$Sdk\ndk\27.0.12077973" }
$Bin = "$Ndk\toolchains\llvm\prebuilt\windows-x86_64\bin"
if (-not (Test-Path $Bin)) { throw "NDK not found at $Ndk (set ANDROID_NDK_HOME)" }

$env:PATH = "$Bin;$env:PATH"
$Cc = "aarch64-linux-android$Api-clang.cmd"
$env:ANDROID_NDK_HOME = $Ndk
$env:CC_aarch64_linux_android = $Cc
$env:CXX_aarch64_linux_android = "aarch64-linux-android$Api-clang++.cmd"
$env:AR_aarch64_linux_android = "llvm-ar.exe"
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = $Cc
# cmake crate (audiopus_sys -> vendored opus): NDK toolchain + NMake generator.
$RepoRoot = Split-Path (Split-Path $PSCommandPath)
$env:CMAKE_TOOLCHAIN_FILE_aarch64_linux_android = "$RepoRoot\scripts\android.toolchain.cmake"
$env:CMAKE_GENERATOR = "NMake Makefiles"
$nmake = Get-ChildItem "E:\VS2022BuildTools\VC\Tools\MSVC\*\bin\Hostx64\x64\nmake.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if ($nmake) { $env:PATH = "$($nmake.DirectoryName);$env:PATH" }
$env:PATH = "E:\.cargo\bin;$env:USERPROFILE\.cargo\bin;$env:PATH"

switch ($Mode) {
  "check"   { cargo check --target aarch64-linux-android }
  "build"   { cargo build --target aarch64-linux-android }
  "release" { cargo build --release --target aarch64-linux-android }
  default   { throw "usage: build-android.ps1 [check|build|release]" }
}
