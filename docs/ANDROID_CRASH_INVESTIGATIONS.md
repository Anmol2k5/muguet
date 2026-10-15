# Muguet Android — Crash Investigations

Template per crash (copy, fill, never delete resolved entries — mark FIXED with
the retest commit):

## <date> — <one-line symptom>

- App/commit: `android-port @ <sha>`
- Device: <model> / Android <ver> / RAM / GPU
- APK: <debug|release> <sha256>
- Native lib build ID: <`llvm-readelf -n libmuguet.so`>
- Repro steps: …
- Logcat (filtered `muguet` tag): …
- Tombstone / stack: …
- Symbolicated (ndk-stack + unstripped .so): …
- Layer: Java / JNI / Rust / Wasmtime / Vulkan / OOM
- Root cause: …
- Fix + retest commit: …
- Status: OPEN / FIXED (retested <sha>) / NEEDS-DEVICE-INFO

## Tooling

- Capture: `adb logcat -c && adb logcat -v threadtime | tee run.log`
  (filter tag `muguet`: `adb logcat -s muguet` after alog wiring).
- Symbolicate: `<ndk>/toolchains/llvm/prebuilt/<host>/bin/llvm-addr2symbol`
  or `ndk-stack -sym <dir-with-unstipped-libmuguet.so> -dump tombstone.txt`.
  Keep the EXACT unstripped `.so` per APK (record its SHA-256 + build ID here).
- Debug builds stage an unstripped `libmuguet.so` (~360 MB); release strips.
  For triage, prefer a debug APK and keep its `.so`.

No crashes recorded yet — no device test has run.
