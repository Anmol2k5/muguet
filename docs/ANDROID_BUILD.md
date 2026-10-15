# Muguet Android — Build

## Toolchain inventory (verified 2026-10-15, Windows 11 host)

| Component | State | Path / version |
|---|---|---|
| Git | present | 2.50.1 |
| Rust | installed (approved) | 1.98.1 (default), `aarch64-linux-android` std, rustfmt + clippy |
| Android SDK | present | `%LOCALAPPDATA%\Android\Sdk`, platform android-35, build-tools 35.0.0 |
| Android NDK | installed (approved) | r27 LTS `27.0.12077973`, clang 18.0.1 |
| adb | present (not on PATH; use full path) | `Sdk\platform-tools\adb.exe`, v37.0.0 |
| Java | present | 22.0.1 (JDK 22); Gradle 8.10.2 runs on it; AGP 8.5.2 OK |
| Gradle | fetched (approved) | 8.10.2 (`%USERPROFILE%\gradle-dl`); official wrapper vendored in `android/` |
| CMake / NMake | present | 4.4.0 / VS2022 BuildTools (Windows generator); Ninja on other hosts |
| Test device | **none attached** (`adb devices` empty) | all runtime milestones blocked |
| REA | deferred (see `ANDROID_REA_FINDINGS.md`) | pinned `rea-agents@6.5.0` for later |

## Commands

Host baseline (upstream, unchanged):
- macOS app: `./make-app.sh [game.zip]`
- iOS: `TEAM=<id> ./make-ios.sh game.zip --install` / `--ipa`, `FREE=1` for free accounts

Android — one command (canonical `scripts/build-android.sh [check|build|release|apk]`;
Windows mirror `scripts\build-android.ps1`; the `.sh` is exercised in CI):

- `check` — `cargo check --target aarch64-linux-android` (no link).
- `build` / `release` — full compile + link.
- `apk` — release lib → stage into `jniLibs` (old copy removed first) →
  `assembleDebug` → verify APK exists + `aapt` badging (package, arm64-v8a,
  launchable activity) → print path + SHA-256. A failed Rust build throws
  BEFORE staging, so a stale library can never be packaged as success.
- Both scripts validate tools first (cargo, rustc + android target, NDK clang,
  cmake, java, SDK) and set the full NDK env themselves — no undocumented
  local setup beyond installs above.

NDK policy (enforced by the scripts, not by repo config):
- C toolchain + linker: `aarch64-linux-android29-clang` (`NDK_API` default 29).
  The API level stays ≤ minSdk so the linker refuses post-29 symbols instead of
  crashing on old devices (verified: no `LIBC_[Q-Z]` refs in the linked `.so`).
- `RUSTFLAGS`: `-C link-arg=-Wl,-z,max-page-size=16384` (16 KB pages, Android 15+;
  verified: LOAD `Align 0x4000`) and `-C link-arg=-Wl,--build-id=sha1`
  (tombstone symbolication).
- CMake (audiopus static opus): NDK toolchain file
  `scripts/android.toolchain.cmake` (ABI arm64-v8a, platform android-29),
  NMake generator on Windows, Ninja elsewhere.
- Plain `cargo` without the scripts cannot find the NDK compiler (proven:
  `cc` ToolNotFound for wasmtime/audiopus_sys build scripts).

## Gradle

- Project `android/` with official wrapper (`gradlew`, `gradle-wrapper.jar`,
  pinned 8.10.2) — reproducible from a fresh checkout; AGP 8.5.2 from Google
  Maven at first build. minSdk 29 / targetSdk + compileSdk 35.
- `lintDebug` passes; `assembleRelease` also assembles (unsigned — needs a
  signing config before it can be installed).
- 16 KB check: `zipalign -c -P 16 -v 4 app-debug.apk` → `libmuguet.so (OK)`.

## Verified artifact (2026-10-15, hardening pass)

- `android/app/build/outputs/apk/debug/app-debug.apk`
  SHA256 `C896051A9D10145CAE744F2A2E3D0616EC314E876CC63E1ADFA6A09EF843400B`
  (includes launcher icon; `lintDebug` 0 errors, 2 accepted warnings:
  landscape lock is intentional, `mipmap-anydpi-v26` is required for the
  link — the unversioned qualifier does not resolve in this toolchain).
- Contains `lib/arm64-v8a/libmuguet.so` byte-identical to the staged release
  build (exports `android_main`, `ANativeActivity_onCreate`; AArch64;
  NEEDED limited to on-device libs; GNU build ID present; LOAD `0x4000`).
- `aapt dump badging`: package `dev.muguet.android` 0.1.0, minSdk 29,
  targetSdk 35, `NativeActivity` launchable, Vulkan 1.1 feature,
  INTERNET/ACCESS_NETWORK_STATE only, `native-code: arm64-v8a`, no game assets.
- NOTE: the APK packages the RELEASE lib. For on-device crash triage, stage a
  DEBUG `libmuguet.so` instead (unstripped, ~360 MB) and keep the exact file
  for `ndk-stack -sym`.

## Verified artifact (2026-10-15, hardening pass)

- `android/app/build/outputs/apk/debug/app-debug.apk`
  SHA256 `7D9A1B61D88FF86EF401C0C252534F61ECD70F1129D747806A15E8CDBD333784`
- Contains `lib/arm64-v8a/libmuguet.so` byte-identical to the staged release
  build (exports `android_main`, `ANativeActivity_onCreate`; AArch64;
  NEEDED limited to on-device libs; GNU build ID present).
- `aapt dump badging`: package `dev.muguet.android` 0.1.0, minSdk 29,
  targetSdk 35, `NativeActivity` launchable, Vulkan 1.1 feature,
  INTERNET/ACCESS_NETWORK_STATE only, `native-code: arm64-v8a`, no game assets.
- NOTE: the APK packages the RELEASE lib. For on-device crash triage, stage a
  DEBUG `libmuguet.so` instead (unstripped, ~360 MB) and keep the exact file
  for `ndk-stack -sym`.

## Install / run (device needed — UNTESTED)

- `adb devices -l` (USB debugging on, RSA prompt accepted)
- `adb install -r android/app/build/outputs/apk/debug/app-debug.apk`
- `adb logcat -c && adb logcat -v threadtime -s muguet`
- launch `Muguet` → expect `[lifecycle] android_main`, storage path,
  `[wasm] no game runtime` notice, window/resume logs — NOT a crash.
- WASM smoke: `adb push SELFTEST-wasm <app-files>/SELFTEST`? No — marker file
  goes in the app-private dir, which needs run-as or writable placement;
  easiest: `adb shell run-as dev.muguet.android` (debuggable builds allow it):
  `echo wasm > files/muguet/SELFTEST`, force-stop, relaunch, read logcat
  for `[selftest] ... PASS/FAIL`. Same with `vulkan`.

## CMake cache poisoning (known sharp edge)

The `cmake` crate reuses `target/<triple>/{debug,release}/build/audiopus_sys-*/out`
across runs. If a configure once ran with a different generator (e.g. an
incomplete env that fell back to Visual Studio), later correct runs fail with
`Does not match the generator used previously`. `cargo clean -p audiopus_sys`
does NOT clear the target-triple dir. Fix:

`Remove-Item -Recurse -Force target\aarch64-linux-android\release\build\audiopus_sys-*`
(or `debug`), then rebuild with the full script env. Always build through
`scripts/build-android.*` so the generator stays pinned (NMake on Windows,
Ninja elsewhere).
