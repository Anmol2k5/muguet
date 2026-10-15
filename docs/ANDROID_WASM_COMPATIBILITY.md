# Muguet Android — WASM Compatibility (M4 groundwork)

Status: **source-derived only — NOT tested.** No `game.wasm` available, no device yet.
Commit: `99f3704`.

## Wasmtime version

- `wasmtime 49.0.2` (`Cargo.lock:2787-2788`), declared `version="49"` with
  `default-features=false, features=["runtime","threads","profiling","anyhow"]`
  (`Cargo.toml:11`). `jit` feature adds `wasmtime/cranelift`,
  `wasmtime/parallel-compilation`, `wasmparser`, `wasm-encoder` (`Cargo.toml:31-33`).

## WASM feature requirements (from `src/main.rs:439-466` + `make-ios.sh:50-54`)

| Feature | Source | Notes |
|---|---|---|
| Memory64 | `wasm_memory64(true)` (`main.rs:441`), `--enable-memory64` | game memory is 64-bit |
| Threads + atomics | `wasm_threads(true)` (`main.rs:442`), `--enable-threads` | custom `__pthread_create_js` spawn |
| Shared-everything memory | `shared_memory(true)` (`main.rs:443`) | single `SharedMemory` for all instances |
| Bulk memory | `wasm_bulk_memory(true)` (`main.rs:444`) | |
| Multi-memory | `wasm_multi_memory(true)` (`main.rs:445`) | |
| SIMD, sign-ext, nontrapping-f2i, multivalue, reference-types, mutable-globals | `wasm-opt` flags (`make-ios.sh:50-54`) | optimizer flags; engine-level settings UNCONFIRMED beyond the five explicit `Config` calls |
| Memory/table64 split, GC, tail-call, component model | — | not referenced anywhere; presumed unused (UNCONFIRMED until a real module is inspected) |

Cranelift: `opt_level(Speed)` (jit only, `main.rs:447`); heap/table spectre
mitigations disabled (jit only, `main.rs:462-463`). No pooling allocator, no file
cache (manual `.cwasm` serialize/deserialize instead), no WASI.

## Target CPU architecture

- Game path proven on `aarch64` only (Apple Silicon mac + iOS arm64).
- iOS cross-compile pins `target("aarch64-apple-ios")` + `has_lse` (`main.rs:184-185`).
- Android target: `aarch64-linux-android`. LSEatomics are universal on ARMv8.1+;
  whether to set the equivalent Cranelift flag for the Android precompile is
  UNCONFIRMED — carry over `has_lse`-style setting and verify on device.

## Shared memory constraints

- One shared memory, type taken from the module's `env.memory` import
  (`main.rs:250-260`); patch does not rewrite memory declarations.
- iOS clamps declared max to `IOS_HEAP_MAX` preserving min/sharedness/page size
  (`heap_type`, `main.rs:414-429`). Android needs the same seam with its own cap.
- Growth via `emscripten_resize_heap`, guest-visible max 16 GiB
  (`host.rs:170-171,310-328`); pool grow 256 MiB steps, iOS cap 2560 MiB
  (`host.rs:271,297-308`).

## Virtual address requirements

- macOS: 17 GiB reservation + 2 GiB guard, `may_move(false)`.
- iOS: 5/8 GiB reservation + 64 KiB guard (`MUGUET_IOS_HEAP_GB`, `make-ios.sh:28`).
- Android: **UNCONFIRMED.** Do not reuse either value. Plan: `mmap` probe test on
  device for candidate reservation sizes before locking one in.

## Host function dependencies

Full list in `ANDROID_ARCHITECTURE_AUDIT.md` §"How the runtime works" + audit §5:
9 patched hooks (`muguet_script`, `muguet_explosion`, `muguet_timewarp`,
`muguet_resource_map`, `muguet_pool`, `muguet_cash`, `muguet_txd`,
`muguet_opus_frame`, `muguet_text`), emscripten threading/heap/time/env syscalls,
FD-gated logging, FS syscalls, `wgpu_start_worker`, `wasm_input_publish_js`,
`wasm_audio_publish_js`, `wasm_httpfs_*`. Re-exports: `muguet:malloc` + ~170
`mp:{COMMANDS}` entries (`mp.rs:14-214`). All host-side, OS-agnostic.

## Native compilation strategy (Android)

Mirror the iOS AOT model, ELF-native:

1. Host: extract `b/*/game.wasm` → `--patch` → (optional `wasm-opt -O3`) →
   new `--precompile-android`: `Config::for_target("aarch64-linux-android")` +
   `engine.precompile_module(&patched)` → `game.android.cwasm`.
2. Embed via plain `include_bytes!` (no Mach-O `__GAME` section, no `segprot`,
   no `SignedCode`) behind a new `android_code` cfg; consume with
   `Module::deserialize` (NOT `deserialize_raw` — no pre-signed mapping).
3. Runtime-JIT (Option A) stays on the table pending device tests of executable
   `mmap`/`mprotect`; decide on evidence, not assumption.

`Module::serialize`/`deserialize` bytes are version- AND target-bound: a macOS
`.cwasm` must never be loaded on Android.

## Android-specific blockers

1. `global_asm!(".section __GAME,__code" + .incbin)` (`main.rs:307-314`) — Mach-O only.
2. `build.rs:15` `-Wl,-segprot,__GAME,rx,rx` — Apple ld only.
3. `SignedCode` custom code memory (`main.rs:347-353,457-458`) — iOS no-JIT model.
4. `Backends::METAL` (`app.rs:70-73`, `gpu.rs:877,891`) — needs Vulkan.
5. objc2/Metal/Mach/QoS/osascript sites listed in the architecture audit §B.
6. `memory_reservation` sizing — needs device measurement.
7. REA WASM-inspection 8 MiB limit: real `game.wasm` (hundreds of MB) cannot go
   through it — use `wasmparser`/`wasm-tools` locally for large artifacts.

## Test plan (M4, blocked on device + NDK link + game assets)

- [ ] host-built probe modules: basic exec, memory64, shared-memory, atomics,
      threads, growth, host imports/exports, ≥8 GiB VA reservation, trap paths
- [ ] same probes as an Android APK/native binary on target device
- [ ] `--patch` + precompile of the real `game.wasm` for `aarch64-linux-android`
- [ ] real module instantiation (no entry call) → init → first frame hooks

## Evidence log

| Date | Check | Result |
|---|---|---|
| 2026-10-15 | Source audit (3 parallel reviews, commit `99f3704`) | feature/config table above; runtime UNTESTED |
| 2026-10-15 | `cargo check --target aarch64-linux-android` | exit 0 (M2) |
| 2026-10-15 | Wasmtime 49.0.2 static Android audit (registry sources) | EXEC path SUPPORTED: unix `mmap`/`mprotect` RW→RX publish (`sys/unix/mmap.rs:140-192`), aarch64-Android trap registers (`signals.rs:242-358`), std TLS + unix fibers (`fiber/src/unix.rs`), aarch64 icache flush + membarrier (`jit-icache-coherence/src/libc.rs`), shared-memory growth without mremap/madvise (`vm/memory/mmap.rs:156-232). CONDITIONAL: BTI ignored on Android (`mmap.rs:170`), memfd/CoW off (`vm.rs:110-125`), NORESERVE hint off (`mmap.rs:22`), 16 KiB pages + SELinux/W^X UNCONFIRMED (device-only). BROKEN (out of game scope): Wasm stack-switching fails triple-check for `OperatingSystem::Android` (`engine.rs:488-499`) — game.wasm must not use it (no indication it does). Cross-precompile from any host SUPPORTED (`precompile_module` skips host check, `engine.rs:726-730`); deserialize requires triple match (`serialization.rs:213-250`). Zero `target_os="linux"` gates break the Android build. Device execution still REQUIRED for proof. |
| 2026-10-15 | `--precompile-android` CLI + `wasm_probe` fixtures | code compiles for Android target; HOST execution blocked (Windows dev host, K-08) |
