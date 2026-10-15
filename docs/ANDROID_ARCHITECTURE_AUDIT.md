# Muguet Android — Architecture Audit (M0)

Commit audited: `99f3704` (main, 2026-10-15). Upstream: https://github.com/c22dev/muguet
Wasmtime `49.0.2`, wgpu `30.0.1`, winit `0.30.13` (both path-patched under `vendor/`), cpal `0.18.2`.
Rust pinned by upstream: `1.98.1`. No `AGENTS.md` in repo.

## How the runtime works

- `src/config.rs:90-108` `setup()` resolves `support/` dir, `support/game` dir
  (`game_dir`, `src/config.rs:38-42`), picks build via `game/b/*/shaders/index.json`
  (`locate`, `src/config.rs:110-117`), requires `data/manifest.json`
  (+ `b/*/game.wasm` when `jit` and not embedded).
- `src/main.rs:242-272` `make_runtime()`: `Engine::new(&engine_config())`,
  `load_module()`, binds every module import through one dispatcher
  (`linker.func_new` → `host::call`, `src/main.rs:471-479`, `src/host.rs:103-127`).
  Only `Memory` + `Func` imports accepted; `Table`/`Global` bail
  (`src/main.rs:250-260`). Single shared memory shared with all instances
  (`SharedMemory::new`, `linker.define(... "env","memory" ...)`, `src/main.rs:252,471`).
- `src/patch.rs` rewrites `game.wasm` (version `VERSION=11`, `src/patch.rs:43`):
  appends 9 hook imports (`HOOKS`, `src/patch.rs:28-38`), re-exports
  `muguet:malloc` + `mp:{COMMANDS}` (`src/patch.rs:269-276`), injects hook CALLs
  into named game functions (scenario check, movie preload, pool grow, opus frame,
  cash write, txd loaded, text lookup, resource map). Memories/tables/data segments
  pass through untouched.
- Threads: custom `__pthread_create_js → spawn_pthread` (`src/host.rs:148-151`,
  `src/main.rs:481-497`), 16 MiB stacks, fresh `Store` + `instantiate` per thread,
  emscripten TLS/stack init sequence (`src/main.rs:499-533`). No WASI.
- Graphics: WASM writes opcodes into shared-heap ring; native `gpu` thread
  (32 MiB stack) translates to wgpu encoders/passes (`src/gpu.rs:45-66,1114-1297`).
  Single graphics import: `wgpu_start_worker` (`src/host.rs:216`).
- Game shaders are WGSL text from `MUGUET_SHADERS` (`<build>/shaders/{hash}.wgsl` +
  `{hash}.consts.json`, `src/gpu.rs:951-957,1649-1685`). Pipelines compiled async on
  4 threads, blocking ≤8 ms (`PIPELINE_WAIT`, `src/gpu.rs:959-973,2486-2514`).
- Audio: cpal stereo 48 kHz default buffer (`src/audio.rs:32-39`), f32 ring in shared
  memory; Opus decoded natively via `audiopus_sys` static libopus (`src/opus.rs`).
  No Apple audio API in the Rust path.
- Input: winit events + polled GameController/keyboard/mouse → shared-memory atomic
  block (`src/input.rs:9-54`); iOS touch overlay is raw UIKit via objc2
  (`src/touch.rs`, ~796 lines, ~30 buttons + sticks + nav pad).
- Multiplayer: pure `std::net::UdpSocket`, port 47520 (`src/proto.rs:5`),
  dumb relay server (`src/server.rs`), reliable `REL/ACK` wrapper for key messages
  (`src/mp.rs:304-342`). No Apple networking API (except `local_ips()` trying
  `/sbin/ifconfig` first, `src/server.rs:108-121`, and a macOS-only `afplay` hook,
  `src/mp.rs:1177-1188`).
- FS: guest syscall layer over `MUGUET_FSROOT` (`src/fs.rs:149-344`); guest socket
  syscalls stubbed `-ENOSYS`. Import keeps only `data/*`, `b/*/shaders/*`,
  `b/*/game.wasm` from user zip, `enclosed_name()` traversal guard, staged+rename
  atomicity, no hash verification (`src/import.rs:12-106`).

## JIT vs AOT (iOS model to mirror on Android)

| Mode | Produce | Consume |
|---|---|---|
| macOS JIT (default `jit` feature) | first-run `Module::new` + `serialize()` → `game-*.cwasm` cache (`src/main.rs:371-386`) | `Module::deserialize_file` |
| macOS embedded (`--no-default-features` + `MUGUET_GAME_CODE`) | `make-app.sh:23-25` `--precompile` → `game.cwasm`, rebuilt with env var | `include_bytes!` + `Module::deserialize` (`src/main.rs:300-304`) |
| iOS AOT (only iOS game path) | `make-ios.sh:33-60`: extract → `--patch` → opt `wasm-opt -O3` → `--precompile-ios` (target `aarch64-apple-ios` + `has_lse`, `src/main.rs:183-189`) → `MUGUET_GAME_CODE=game.cwasm` → `--no-default-features` build | `.section __GAME,__code` + `.incbin` (`src/main.rs:307-314`) + `Module::deserialize_raw` + `SignedCode` custom code memory (`src/main.rs:347-353,457-458`), `build.rs:15` `-segprot,__GAME,rx,rx` |

`MUGUET_GAME_CODE` sites: `build.rs:7-10`, `make-ios.sh:60,63`, `make-app.sh:25`,
`src/main.rs:302,311,318`. `game_code` cfg sites: `build.rs:7,10`,
`src/main.rs:297,300,306,316,332,338,343,346,348,457`.

## Wasmtime config (`src/main.rs:439-466`)

Enabled: `memory64`, `threads`, `shared_memory`, `bulk_memory`, `multi_memory`;
Cranelift `Speed` (jit only); spectre-mitigation flags OFF (jit only).
Reservations: macOS 17 GiB reserve / 2 GiB guard (`src/main.rs:452-455`);
iOS `IOS_HEAP_MAX` (5 or 8 GiB via `MUGUET_IOS_HEAP_GB`, `make-ios.sh:23-28`) /
64 KiB guard, `may_move(false)` both. Guest-visible heap cap 16 GiB
(`emscripten_get_heap_max`, `src/host.rs:170`). iOS clamps declared memory max
(`heap_type`, `src/main.rs:414-429`).
Not configured: pooling allocator, cache config, WASI, epoch interruption —
defaults apply (UNCONFIRMED which defaults matter on Android until device-tested).

## Platform compatibility matrix

### A. Directly reusable (no change expected)

- `src/patch.rs` — pure wasmparser/wasm-encoder rewriting. Host-OS agnostic.
- `src/host.rs` Emscripten/threding/heap/time/env/logging dispatch — POSIX-level only.
- `src/mp.rs`, `src/server.rs`, `src/proto.rs`, `src/bin/muguet-server.rs` — std UDP.
  Exceptions: `server.rs:108-121` ifconfig fallback order, `mp.rs:1177-1188` afplay.
- `src/opus.rs` — vendored static libopus, no Apple API.
- `src/fs.rs`, `src/io.rs`, `src/import.rs` — `std::os::unix` + `libc::statvfs` are
  fine on Android; only default *paths* change (see C).
- `src/gpu.rs` ring protocol, opcode set, WGSL pipeline builder, BC CPU decode
  (`bcdec_rs`), BC1→ASTC compute path, mip-drop, `FLOAT32_FILTERABLE` fallback —
  backend-agnostic Rust. `Backends::METAL` literals at `gpu.rs:877,891` must change.
- `src/config.rs` settings/env-knob layer — paths + heap sizing need Android values.
- `vendor/wgpu-core` local patch (per-pass bind-group dedup) — backend-agnostic, keep.

### B. Reusable with modifications (cfg-gate / small edits)

- `src/main.rs` — gate out: Mach `proc_pid_rusage`/`task_info`/`malloc_zone_statistics`
  (`:56-104`), `MTLCreateSystemDefaultDevice` + `#[link(Metal)]` (`:106-118`),
  `pthread_set_qos_class_self_np` (`:120-138,487-491`), `getrlimit/setrlimit`
  (`:140-158`), `osascript` alert (`:642-646`), Mach-O `global_asm! __GAME` + `.incbin`
  (`:307-314`), `SignedCode` (`:347-353`). Keep: engine config, runtime, pthread
  spawn shape (QoS → Android thread priority), argv builder.
- `src/app.rs` — `Backends::METAL` (`:70-73`) → Vulkan; iOS window attrs (`:60-64`)
  → Android equivalents; `Occluded/Focused` handling largely reusable.
- `src/gpu.rs:875-922,944-950,3397-3416` — adapter/device/surface-format/swapchain
  selection needs Vulkan-tuned choices; `want` feature set (`:899-903`) needs
  per-device validation (see §D).
- `src/audio.rs:13-64` — cpal on Android uses AAudio (CONFIRMED at link level:
  the Android binary carries `NEEDED libaaudio.so`); still needs explicit config
  negotiation + audio-focus handling instead of hardcoded 48 kHz.
- `src/input.rs`, `src/pad.rs` — shared-block + key/mouse logic reusable; `pad.rs`
  GameController backend must move to Android input APIs; `keyboard.rs`/`mouse.rs`
  (objc2) are iOS-only modules (`main.rs:14-17`) — Android needs its own adapters.
- `build.rs` — iOS Swift/`segprot` section is already `target_os="ios"`-gated
  (`:12-14` early return otherwise); Android needs no equivalent (normal `.so`).
- `Cargo.toml:21-25` objc2 family → move behind `[target.'cfg(target_vendor="apple")'.dependencies]`.
- `vendor/winit` — already ships Android backend (`android-activity 0.6.1`, `ndk 0.9`
  in `Cargo.lock:87-90,1420-1422`); vendored wholesale, no local edits after vendoring
  (UNCONFIRMED byte-identical to crates.io 0.30.13 — no hash comparison run).

### C. Requires Android implementation

- Launcher Activity (Kotlin/Compose): asset zip pick, import progress, presets,
  startup, error display, log export, lifecycle. Replaces `launcher/Launcher.swift`
  (mac) and `ios/Launcher.swift`.
- Storage paths: `support_dir()` (`src/config.rs:30-36`) → app-internal/external files
  dir via JNI; `MUGUET_FSROOT`/`MUGUET_DATA`/`MUGUET_SHADERS`/cache under it.
- Touch input: `src/touch.rs` UIKit overlay → Android `MotionEvent` adapter driving
  the same `input::*`/`pad::INJECT` shared block (button table `touch.rs:127-162`
  and vehicle IDs `touch.rs:82-110` are reusable data).
- Gamepad: Android InputDevice APIs replacing `pad.rs` GCController calls.
- AOT pipeline for `aarch64-linux-android`: `--precompile-android`
  (`target("aarch64-linux-android")`) + embed/deserialize path replacing Mach-O
  `__GAME` (plain `include_bytes!` + `Module::deserialize`, or runtime JIT if
  executable-memory mapping proves viable — UNCONFIRMED until device-tested).
- `scripts/build-android.sh`, Gradle project, `AndroidManifest.xml`
  (Vulkan feature, permissions: INTERNET + gamepad/vibrate as needed).
- Android address-space tuning: do NOT reuse 17 GiB macOS reservation blindly;
  needs device measurement (see WASM doc).

### D. Requires investigation before feasibility is known

1. Wasmtime Cranelift JIT on Android (executable mmap, mprotect transitions,
   selinux `neverallow` on some OEM builds) — runtime-compile gate M4.
2. 17 GiB `memory_reservation` virtual range on Android ARM64 (tagged-pointer top-byte,
   39/48-bit VA per device) — UNCONFIRMED; measure `mmap` probe on device.
3. `want` GPU features on mobile Vulkan: `DEPTH32FLOAT_STENCIL8`,
   `RG11B10UFLOAT_RENDERABLE`, `BGRA8UNORM_STORAGE`, `CLIP_DISTANCES`,
   `DUAL_SOURCE_BLENDING`, adapter-specific format reinterpretation, BC formats
   (absent on most mobile → BC_SOFT/ASTC paths already exist, need perf validation).
4. Swapchain choices (`alpha_mode[0]`, `Mailbox-else-Fifo`, non-sRGB preference,
   `desired_maximum_frame_latency: 2`) on Android compositors.
5. MSAA counts on mobile GPUs (`max_msaa` probes 16/8/4/2 for desktop formats).
6. `wasm-opt` availability/need for the Android AOT path (optional on iOS).
7. Audio latency/underrun behavior of cpal AAudio backend under game load.
8. 4 GB experimental target: WASM working set is game-determined; cannot be capped
   below it — needs measurement, may be infeasible.

## Evidence scope

All claims above are **inferred from source** at the listed file:line refs
(commit `99f3704`). Nothing here is confirmed by runtime observation.
`cargo check --target aarch64-linux-android` baseline was started during this audit;
results go in `ANDROID_PORT_STATUS.md`. No `game.wasm`, no device, no APK exists yet.
