# Muguet Android — Known Issues

| ID | Area | Symptom / blocker | Repro | Status |
|---|---|---|---|---|
| K-01 | Toolchain | No test device attached (`adb devices` empty) | `adb devices -l` | OPEN — blocks all runtime verification |
| K-02 | Toolchain | NDK r27 install | sdkmanager 2026-10-15 | CLOSED (verified: clang 18.0.1, API-29-clean link) |
| K-03 | Compile | Apple-only deps/APIs for Android | `cargo check/build/clippy/test --no-run` exit 0, 2026-10-15 | CLOSED |
| K-04 | Design | WASM reservation sizing | Unsized: Android policy is 8 GiB VA default + `MUGUET_WASM_RESERVE_GB` tunable + declared-max clamp; real working set needs device measurement (M6/M9) | OPEN (policy implemented, size unverified) |
| K-05 | Design | ELF AOT path | `--precompile-android` CLI implemented (mirrors `--precompile-ios` with `aarch64-linux-android` + LSE); execution needs a Unix host (K-08); deserialization path for embedded Android code lands with M6 | OPEN (producer done, consumer pending) |
| K-06 | Assets | No user-provided game zip; nothing to validate against | — | OPEN — M6+ blocked. Never fabricate, never bundle/redistribute |
| K-07 | Lint | Upstream is not rustfmt-clean (hundreds of pre-existing diffs); `cargo fmt --check` is not a usable gate. New/edited code matches surrounding style. Android lint (`lintDebug`): 0 errors, landscape advisory acknowledged | rustfmt 1.98.1 + `lint-results-debug.txt`, 2026-10-15 | NOTED |
| K-08 | Host | Windows dev host: no muguet code can EXECUTE here (pre-existing Unix-only code; upstream gates on Darwin+arm64). Android-target COMPILE fully verified; host tool runs need mac/Linux CI or the device | `cargo test` on win-msvc, 2026-10-15 | OPEN |
| K-09 | Scripts | `build-android.ps1` `$args` param silently dropped arguments (Invoke-Native ran bare `cargo`/`gradle`; one "BUILD SUCCESSFUL" was a Gradle no-op). Fixed by renaming to `[string[]]$CmdArgs`, proven with a splat test. Lesson: verify tool EFFECT (artifact hash/mtime), not just exit codes | 2026-10-15 | CLOSED (regression guard: dropped args now fail loudly since bare cargo exits 1) |
| K-10 | Design | In-process activity recreation calls `android_main` again while old threads may linger. `ANDROID_APP` is now clone-out (no panic); engine/gpu thread duplication across recreation is unhandled and UNTESTED | code inspection | OPEN — needs device lifecycle testing |
| K-11 | Perf | Static audit only: wasmtime JIT path has Android gates (mmap/RWX, signals, icache, fibers); BTI ignored, memfd/CoW off, 16 KiB pages + SELinux/W^X need device proof; Wasm stack-switching fails triple-check (game scope unaffected) | registry source audit, 2026-10-15 | NOTED — device proof required |

Upstream macOS/iOS behavior is preserved by construction (all changes are
`cfg(target_os/vendor)`-gated or additive); it cannot be re-verified on this
Windows/mac-less host beyond `cfg` inspection — needs upstream CI or a Mac.
