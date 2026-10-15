# Muguet Android build (Windows mirror of scripts/build-android.sh).
# Usage: scripts\build-android.ps1 [check|build|release|apk]  (default: check)
param([string]$Mode = "check")
# NOTE: keep at Continue — cargo/gradle write warnings to stderr, and "Stop"
# would turn any warning into a terminating error. Native failures are caught
# via $LASTEXITCODE checks below.
$ErrorActionPreference = "Continue"
function Invoke-Native([string]$Cmd, [string[]]$CmdArgs) {
  & $Cmd @CmdArgs
  if ($LASTEXITCODE -ne 0) { throw "$Cmd $($CmdArgs -join ' ') failed with exit $LASTEXITCODE" }
}

$Api = if ($env:NDK_API) { $env:NDK_API } else { "29" }
$Sdk = if ($env:ANDROID_HOME) { $env:ANDROID_HOME } elseif ($env:ANDROID_SDK_ROOT) { $env:ANDROID_SDK_ROOT } else { "$env:LOCALAPPDATA\Android\Sdk" }
$Ndk = if ($env:ANDROID_NDK_HOME) { $env:ANDROID_NDK_HOME } else { "$Sdk\ndk\27.0.12077973" }
$Bin = "$Ndk\toolchains\llvm\prebuilt\windows-x86_64\bin"
if (-not (Test-Path $Bin)) { throw "NDK not found at $Ndk (set ANDROID_NDK_HOME)" }

$env:PATH = "$Bin;$env:PATH"
# Cargo lives outside the NDK bin dir: pick up standard install locations
# first (non-interactive shells often lack them on PATH).
foreach ($d in @("E:\.cargo\bin", "$env:USERPROFILE\.cargo\bin")) {
  if ((Test-Path $d) -and ($env:PATH -notlike "*$d*")) { $env:PATH = "$d;$env:PATH" }
}
$Cc = "aarch64-linux-android$Api-clang.cmd"
$env:ANDROID_NDK_HOME = $Ndk
# --- tool validation (fail fast with actionable messages) ---
function Require-Cmd($name, $hint) {
  if (-not (Get-Command $name -ErrorAction SilentlyContinue)) { throw "$name not found on PATH ($hint)" }
}
Require-Cmd cargo "install Rust 1.98.1 via rustup"
Require-Cmd rustc "install Rust 1.98.1 via rustup"
if (((rustup target list --installed) -join "`n") -notmatch "aarch64-linux-android") { throw "rust target aarch64-linux-android missing: rustup target add aarch64-linux-android" }
if (-not (Test-Path "$Bin\$Cc")) { throw "NDK clang missing: $Bin\$Cc" }
Require-Cmd cmake "install CMake 3.22+"
Require-Cmd java "install JDK 17+"
if (-not (Test-Path $Sdk)) { throw "Android SDK not found at $Sdk (set ANDROID_HOME)" }
$env:CC_aarch64_linux_android = $Cc
$env:CXX_aarch64_linux_android = "aarch64-linux-android$Api-clang++.cmd"
$env:AR_aarch64_linux_android = "llvm-ar.exe"
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = $Cc
# 16 KB page-size compatibility (Android 15+ devices): align ELF segments to
# 16384. Verified with llvm-readelf (LOAD Align must read 0x4000).
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS = "-C link-arg=-Wl,-z,max-page-size=16384 -C link-arg=-Wl,--build-id=sha1"
# cmake crate (audiopus_sys -> vendored opus): NDK toolchain + NMake generator.
$RepoRoot = Split-Path (Split-Path $PSCommandPath)
$env:CMAKE_TOOLCHAIN_FILE_aarch64_linux_android = "$RepoRoot\scripts\android.toolchain.cmake"
$env:CMAKE_GENERATOR = "NMake Makefiles"
$nmake = Get-ChildItem "E:\VS2022BuildTools\VC\Tools\MSVC\*\bin\Hostx64\x64\nmake.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if ($nmake) { $env:PATH = "$($nmake.DirectoryName);$env:PATH" }

switch ($Mode) {
  "check"   { Invoke-Native cargo @("check", "--target", "aarch64-linux-android") }
  "build"   { Invoke-Native cargo @("build", "--target", "aarch64-linux-android") }
  "release" { Invoke-Native cargo @("build", "--release", "--target", "aarch64-linux-android") }
  "clippy"  { Invoke-Native cargo @("clippy", "--target", "aarch64-linux-android") }
  "test-norun" { Invoke-Native cargo @("test", "--target", "aarch64-linux-android", "--no-run") }
  "apk" {
    Invoke-Native cargo @("build", "--release", "--target", "aarch64-linux-android")
    $So = "target\aarch64-linux-android\release\libmuguet.so"
    if (-not (Test-Path $So)) { throw "release build did not produce $So (refusing to package a stale library)" }
    $JniDir = "android\app\src\main\jniLibs\arm64-v8a"
    New-Item -ItemType Directory -Force $JniDir | Out-Null
    Remove-Item "$JniDir\libmuguet.so" -ErrorAction SilentlyContinue
    Copy-Item $So "$JniDir\libmuguet.so" -Force
    echo "[apk] staged $So"
    $readelf = "$Bin\llvm-readelf.exe"
    echo "[apk] build-id: $((& $readelf -n $So | Select-String 'Build ID' | Select-Object -First 1))"
    $Gradle = if ($env:GRADLE_BIN) { $env:GRADLE_BIN } else { "$env:USERPROFILE\gradle-dl\gradle-8.10.2\bin\gradle.bat" }
    Push-Location android; Invoke-Native $Gradle @(":app:assembleDebug"); Pop-Location
    $Apk = "android\app\build\outputs\apk\debug\app-debug.apk"
    if (-not (Test-Path $Apk)) { throw "assemble succeeded but $Apk is missing" }
    $bt = Get-ChildItem "$Sdk\build-tools\*" -Directory | Sort-Object Name -Descending | Select-Object -First 1
    $badging = & "$($bt.FullName)\aapt2.exe" dump badging $Apk
    foreach ($want in @("package: name='dev.muguet.android'", "native-code: 'arm64-v8a'", "launchable-activity")) {
      if (($badging -join "`n") -notmatch [regex]::Escape($want)) { throw "APK verification failed: missing [$want]" }
    }
    echo "[apk] verified: dev.muguet.android, arm64-v8a, NativeActivity launchable"
    echo "[apk] $Apk"
    (Get-FileHash $Apk -Algorithm SHA256).Hash
  }
  default   { throw "usage: build-android.ps1 [check|build|release|clippy|test-norun|apk]" }
}
