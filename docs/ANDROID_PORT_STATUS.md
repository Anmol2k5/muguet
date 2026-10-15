# Muguet Android — Port Status

Commit: `99f3704`. Updated: 2026-10-15.

## Milestones

| ID | Gate | Status | Evidence |
|---|---|---|---|
| M0 | Repo audit | DONE | `ANDROID_ARCHITECTURE_AUDIT.md`, 3 parallel source reviews |
| M1 | REA readiness | DONE (deferred, approved) | `ANDROID_REA_FINDINGS.md`, doctor + dry-run JSON on file |
| M2 | Rust Android compilation | DONE | `cargo check`/`build`/`clippy` exit 0, tests compile (`--no-run`); ELF64 AArch64 PIE, NEEDED limited to on-device libs. Host `cargo test` N/A on Windows (upstream is Unix-only) |
| M3 | APK shell | STRUCTURE DONE, DEVICE-BLOCKED | `assembleDebug` exit 0; APK `dev.muguet.android` 0.1.0 (SHA256 `9AD4EC6B…F062`), `lib/arm64-v8a/libmuguet.so` = staged release build byte-identical (27,353,176 B, `android_main`+`ANativeActivity_onCreate` exported); aapt badging: NativeActivity launchable, Vulkan 1.1 feature, INTERNET only, minSdk 29/targetSdk 35. Install/launch/logcat need a device |
| M3 | APK shell | NOT STARTED | blocked on M2 + NDK verify |
| M4 | Wasmtime compatibility | NOT STARTED | blocked on device + `game.wasm`; probes planned in WASM doc |
| M5 | Vulkan init (V1–V2) | NOT STARTED | blocked on M3/M4 |
| M6 | Game module init | NOT STARTED | needs user-provided game zip (never bundle assets) |
| M7 | First gameplay frame | NOT STARTED | — |
| M8 | Controls + audio | NOT STARTED | — |
| M9 | Memory/perf | NOT STARTED | 8 GB → 6 GB → experimental 4 GB |
| M10 | Multiplayer | NOT STARTED | single-player first; protocol is portable (std UDP) |
| M11 | Release verification | NOT STARTED | — |

## Verified statement (honest deltas)

- Native code compiles for Android: YES (`cargo build --target aarch64-linux-android` exit 0, 2026-10-15; ELF64 AArch64 PIE, needs only on-device `.so`s)
- APK installs: NO (no APK, no device)
- Wasmtime initializes on Android: NOT TESTED
- Game module executes: NOT TESTED (no `game.wasm` in repo; must be user-provided)
- Vulkan initializes: NOT TESTED
- Game renders / input / audio: NOT TESTED
- Sustained gameplay: NOT TESTED

## Active blockers

1. No Android test device attached — install/launch/logcat/run (rest of M3), M4–M9 all blocked. **This is the critical path.**
2. No user-provided game zip — M6+ blocked (do not fabricate; do not redistribute).
3. Host tool execution (`--precompile-android`, `wasm_probe`, `cargo test`) needs a Unix host or the device (K-08).
