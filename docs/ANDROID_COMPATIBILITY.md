# Muguet Android — Compatibility (devices)

No devices tested yet. This file records real results only — no projections.

## Tested devices

| Device | RAM | SoC/GPU | Android | Vulkan | Status |
|---|---|---|---|---|---|
| *none* | — | — | — | — | `adb devices` empty on 2026-10-15 |

## Targets (engineering goals, NOT verified minimums)

- Primary: 8 GB RAM, ARM64, Vulkan-capable
- Secondary: 6 GB RAM
- Experimental: 4 GB RAM

## Support matrix (current truth)

| Feature | VERIFIED | IMPLEMENTED — UNTESTED | NOT STARTED / BLOCKED |
|---|---|---|---|
| macOS JIT run | yes (upstream) | — | — |
| iOS AOT run | yes (upstream) | — | — |
| Android compile (check/build/clippy/test-compile) | yes, exit 0 | — | — |
| Android APK assembly (+lint, +16 KB align) | yes, exit 0 | — | — |
| Android install/launch/logcat | — | — | BLOCKED (no device) |
| Wasmtime on Android | — | static audit + `--selftest=wasm` ready | BLOCKED (device) |
| Vulkan on Android | — | backend select + `--selftest=vulkan` ready | BLOCKED (device) |
| Touch / gamepad / audio-focus | — | minimal touch boundary only | BLOCKED (device); full UI NOT STARTED |
| Game import UI (doc picker) | — | — | NOT STARTED (Rust ZIP layer already safe) |
| Multiplayer interop | — | — | NOT STARTED (needs M6 first) |

## OS compatibility (statically enforced)

- minSdk 29: NDK C toolchain + linker pinned to API 29 (`NDK_API` default);
  linked `.so` carries no post-29 `LIBC_*` version refs (verified by readelf).
- 16 KB pages: ELF `LOAD Align 0x4000` + APK `zipalign -c -P 16` pass.
- `android.app.NativeActivity` entry; Vulkan 1.1 feature required.
