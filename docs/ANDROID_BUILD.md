# Muguet Android — Build (M2/M3 groundwork)

## Toolchain inventory (verified 2026-10-15, Windows 11 host)

| Component | State | Path / version |
|---|---|---|
| Git | present | 2.50.1 |
| Rust | installed 2026-10-15 (approved) | 1.98.1 (default), `aarch64-linux-android` std added |
| Android SDK | present | `C:\Users\Mayur\AppData\Local\Android\Sdk`, platform android-35, build-tools 35.0.0 |
| Android NDK | installed 2026-10-15 (approved) | r27 LTS `27.0.12077973`, clang 18.0.1, `Sdk\ndk\27.0.12077973` |
| adb | present (not on PATH) | `Sdk\platform-tools\adb.exe`, v37.0.0 |
| Java | present | 22.0.1 (JDK 22); AGP prefers 17 — revisit when Gradle lands |
| Test device | **none attached** (`adb devices` empty) | all device milestones blocked until one exists |
| REA | deferred (see `ANDROID_REA_FINDINGS.md`) | pinned `rea-agents@6.5.0` for later |

`local.properties` / Gradle wrapper do not exist yet — created at M3.

## Commands

Host baseline (upstream, unchanged):
- macOS app: `./make-app.sh [game.zip]`
- iOS: `TEAM=<id> ./make-ios.sh game.zip --install` / `--ipa`, `FREE=1` for free accounts

Android (canonical, `scripts/build-android.sh [check|build|release]`;
Windows mirror `scripts\build-android.ps1`):

- `scripts/build-android.sh check` — `cargo check --target aarch64-linux-android`
  (host check, no link). The script resolves the NDK (`ANDROID_NDK_HOME` else
  `ndk/27.0.12077973` under the SDK) and exports `CC/CXX/AR_<target>` +
  `CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER` (`aarch64-linux-android35-clang`).
  No repo config change needed; plain cargo without the script cannot find the
  NDK compiler (proven: `cc` ToolNotFound for wasmtime/audiopus_sys build scripts).
- `scripts/build-android.sh build|release` — full compile + link (needs NDK).
- `./gradlew assembleDebug` → `app/build/outputs/apk/debug/*.apk` (M3)
- install/launch: `adb install -r …apk`, `adb logcat -c && adb logcat | grep -i muguet`

## NDK link config (to add once NDK verified)

```toml
[target.aarch64-linux-android]
ar = "<ndk>/toolchains/llvm/prebuilt/windows-x86_64/bin/llvm-ar"
linker = "<ndk>/toolchains/llvm/prebuilt/windows-x86_64/bin/aarch64-linux-android35-clang.cmd"
```

API level: minSdk 29 (Android 10, Vulkan 1.1 baseline) / targetSdk 35.
Final call after adapter-feature validation (M5).

## Debugging

- `adb logcat` (Rust `eprintln!`/`log` → stderr → logcat via android-activity).
  NOTE: stderr-to-logcat piping on-device is UNVERIFIED (no device yet); if bare
  `eprintln!` is swallowed by NativeActivity, add an `__android_log_write` shim
  before any device debugging milestone.
- Native crashes: tombstone in logcat + `ndk-stack -sym` against unstripped `.so`.
- Findings go in `docs/ANDROID_CRASH_INVESTIGATIONS.md` (created on first crash).

## Gradle (M3)

- Project: `android/` (settings, root + app build files, manifest, gradle.properties).
- Gradle 8.10.2 distribution under `%USERPROFILE%\gradle-dl` (user-approved fetch);
  AGP 8.5.2 from Google Maven at first build. Wrapper jar deferred — system Gradle
  used directly until the wrapper is vendored.
- Stage the Rust lib first:
  `copy target\aarch64-linux-android\release\libmuguet.so android\app\src\main\jniLibs\arm64-v8a\`
  then `gradle :app:assembleDebug` in `android/`.
- Verified 2026-10-15: `app-debug.apk` (27,391,747 B, SHA256
  `9AD4EC6B825AD028991E6F9036C38026390D4584A165D4F21EC554E7467F062D`)
  contains `lib/arm64-v8a/libmuguet.so` byte-identical to the staged file
  (27,353,176 B; exports `android_main`, `ANativeActivity_onCreate`).
  aapt badging: package `dev.muguet.android` 0.1.0, minSdk 29, targetSdk 35,
  `NativeActivity` launchable, Vulkan 1.1 feature, INTERNET/ACCESS_NETWORK_STATE
  only, `native-code: arm64-v8a`, no game assets bundled.
- NOTE: the APK currently packages the RELEASE lib (small, limited symbols).
  For on-device crash triage, stage a DEBUG `libmuguet.so` instead (unstripped,
  ~360 MB) and keep the matching unstripped file for `ndk-stack -sym`.
