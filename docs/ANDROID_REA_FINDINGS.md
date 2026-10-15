# Muguet Android — REA Findings (M1)

## Availability (2026-10-15, `rea-agents@6.5.0` via npx)

- `doctor --client opencode --json`: **not connected.** `healthy:false`,
  `registration:opencode` → `config_drift`
  (`C:\Users\Mayur\.config\opencode\opencode.json`), skill missing, no backends:
  Hopper/Ghidra/IDA/JADX all unconfigured. Node 24.19.0 healthy, win32 x64.
- `setup --client opencode --dry-run --json`: `status:planned`, 2 actions —
  `configure_client:opencode` (create MCP registration, preserve unrelated config)
  and `install_skill` (skill `reverse-engineer-anything`). No provider env vars.
- Decision (user-approved 2026-10-15): **defer setup** until binary/APK/WASM
  artifacts exist. Revisit at M4 (Wasmtime probes) / M8 (APK inspection).
- Pinned version for reproducibility once adopted: `rea-agents@6.5.0`
  (tools digest `805433f1…`, prompts `7152916a…` per doctor output).

## Backends

None installed. Policy: install only on demonstrated need —
WABT/wasm-tools for large-module metadata (local tools preferred over REA's
8 MiB-limited WASM path), JADX only when an APK exists to inspect, ADB already
available via platform-tools for device work.

## Investigations

- 2026-10-15 hardening pass: all findings came from conventional tools
  (`cargo`/`rustc`/`clippy` diagnostics, `llvm-readelf`/`llvm-nm`, `aapt2`,
  `zipalign`, registry-source grep) per the investigation policy — no REA
  backend was needed. Static inspection only; nothing here proves runtime.
- Artifacts now in scope for future REA work (hashes in `ANDROID_BUILD.md`):
  `libmuguet.so` (ELF AArch64, build ID present) and `app-debug.apk`
  (arm64-v8a). APK inspection so far is `aapt2`-level (badging, zip listing);
  JADX-backed decompilation still unconfigured (deferred with setup).
- Status values used across docs: NOT STARTED | IN PROGRESS |
  IMPLEMENTED — UNTESTED | VERIFIED | BLOCKED | FAILED (see PORT_STATUS).
Per the investigation policy, source-level questions stay on conventional tools;
every future entry here must record artifact hash, tool version, command, and
evidence scope (`static inspection` / `runtime observation` / `inferred` /
`not tested` / `blocked`).
