# Muguet Android — Known Issues

| ID | Area | Symptom / blocker | Repro | Status |
|---|---|---|---|---|
| K-01 | Toolchain | No test device attached (`adb devices` empty) | `adb devices -l` | OPEN — blocks M3+ |
| K-02 | Toolchain | NDK r27 installed + verified (clang 18.0.1, android35) | sdkmanager 2026-10-15 | CLOSED |
| K-03 | Compile | Apple-only deps/APIs gated; `cargo check` + `cargo build` for Android exit 0 (2026-10-15) | check-android8/build-android1 logs | CLOSED |
| K-04 | Design | 17 GiB macOS WASM reservation unsuitable for Android; replacement size unknown | needs device `mmap` probe | OPEN |
| K-05 | Design | Mach-O `__GAME` AOT path has no ELF equivalent yet | `main.rs:307-314` | OPEN — needs `--precompile-android` |
| K-06 | Assets | No user-provided game zip; nothing to validate against | — | OPEN — M6+ blocked |
| K-08 | Host | This dev host is Windows: no muguet code (lib/bins/tools, incl. `--precompile-android` and `wasm_probe`) can EXECUTE here — pre-existing Unix-only code (`std::os::unix`, `statvfs`, …; upstream gates on Darwin+arm64 in `build.sh:89-90`). Android-target COMPILE is verified; host tool runs need mac/Linux CI or the device. | `cargo test` + `cargo build --bin wasm_probe` on win-msvc, 2026-10-15 | OPEN |
| K-07 | Lint | Upstream is not rustfmt-clean (hundreds of pre-existing diffs, e.g. `mp.rs`, `gpu.rs`, `touch.rs`); `cargo fmt --check` is not a usable gate. New/edited code must match surrounding style instead. | `cargo fmt --check` (rustfmt 1.98.1) 2026-10-15 | NOTED |

Upstream (macOS/iOS) functionality is untouched so far; no regressions possible yet.
