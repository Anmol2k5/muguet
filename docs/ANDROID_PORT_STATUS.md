# Muguet Android — Port Status

Branch: `android-port` @ `97a96d0` (parent: upstream `99f3704`, `main` untouched).
Updated: 2026-10-15 (hardening pass).

Statuses: NOT STARTED | IN PROGRESS | IMPLEMENTED — UNTESTED | VERIFIED |
BLOCKED | FAILED.

- **Build verification** = the artifact compiles/assembles on the build host.
- **Runtime verification** = it installs, starts and executes on a device.
- **Gameplay verification** = real game loads, renders and is controllable.

## Milestones

| ID | Gate | Status | Evidence |
|---|---|---|---|
| M0 | Repo audit | VERIFIED | `ANDROID_ARCHITECTURE_AUDIT.md` (3 parallel source reviews, commit `99f3704`) |
| M1 | REA readiness | VERIFIED (deferred) | `ANDROID_REA_FINDINGS.md`; `rea-agents@6.5.0` doctor + dry-run; setup deferred by approval, no backends installed |
| M2 | Rust Android compilation | VERIFIED | `cargo check` / `build` / `clippy` / `test --no-run` for `aarch64-linux-android` all exit 0; ELF64 AArch64 PIE; API-29-clean symbols; LOAD `0x4000` (16 KB); `android_main` + `ANativeActivity_onCreate` exported. Host `cargo test` execution N/A on Windows (upstream Unix-only, K-08) |
| M3 | APK shell (build) | VERIFIED | `assembleDebug` exit 0 via `scripts/build-android.ps1 apk`; APK verified by `aapt` (package, arm64-v8a, launchable NativeActivity) + SHA-256 |
| M3 | APK shell (runtime) | BLOCKED | no device: install/launch/logcat untested |
| M4 | Wasmtime execution | IMPLEMENTED — UNTESTED | static audit says all runtime paths have Android gates; `--precompile-android` + in-app `--selftest=wasm` + `wasm_probe` fixtures compile; host execution blocked (K-08); device execution blocked (no device) |
| M5 | Vulkan init | IMPLEMENTED — UNTESTED | Vulkan backend selection compiles; headless `--selftest=vulkan` compiles; surface/swapchain/device-loss paths need a device |
| M6 | Game module init | NOT STARTED | BLOCKED on user-provided game zip + device |
| M7 | First gameplay frame | NOT STARTED | BLOCKED (M6) |
| M8 | Controls + audio | IMPLEMENTED — UNTESTED (partial) | cpal/AAudio link-verified + config negotiation; minimal Android touch boundary (`atouch.rs`); gamepad + full touch UI NOT STARTED; all BLOCKED on device for verification |
| M9 | Memory/perf | NOT STARTED | BLOCKED (no measurements possible); Android reservation policy implemented (8 GiB VA default, tunable) but unsized |
| M10 | Multiplayer | NOT STARTED | protocol is portable std UDP (audit); needs M6 single-player first |
| M11 | Release verification | NOT STARTED | BLOCKED |

## Capability deltas (explicit)

- Native code compiles for Android: YES (exit 0 across check/build/clippy/test-compile)
- APK assembles reproducibly: YES (`build-android.ps1 apk`, verified contents + hash)
- APK installs / launches: UNTESTED (no device attached, `adb devices` empty)
- Wasmtime initializes on Android: UNTESTED
- Game module executes: UNTESTED (no `game.wasm`; must be user-provided, never bundled)
- Vulkan initializes / game renders: UNTESTED
- Input / audio functional: UNTESTED
- Sustained gameplay: UNTESTED

## Active blockers

1. No Android test device — install/launch/logcat/run (rest of M3), M4–M9 runtime. **Critical path.**
2. No user-provided game zip — M6+. Never fabricate, never bundle/redistribute.
3. Host tool execution needs a Unix host (K-08): `--precompile-android`, `wasm_probe`, `cargo test` cannot run on this Windows box.
