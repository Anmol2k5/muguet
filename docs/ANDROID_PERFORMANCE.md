# Muguet Android — Performance

No measurements exist. This file will hold real benchmarks only.

## To collect (per device, per preset)

- Startup: WASM compile time, peak compile RSS, `.cwasm` size, cold vs warm start
- Steady state: peak RSS, PSS, WASM committed, GPU estimates, texture-cache sizes
- Rendering: FPS per preset (Ultra Low / Low / Balanced / High / Custom),
  render scale actually applied, frame-time p50/p95
- Audio: underrun frames/s (existing `audio.rs` counters), callback rate
- Loading: import throughput, shader compile stalls (`pending_skips`)

## Method

- `adb shell dumpsys meminfo`, `logcat` mixer/GPU stats already in code
  (`[audio]`, `MUGUET_MEM_STATS`, pipeline `pending_skips`).
- Each entry must cite: device, build commit, preset, game build id, measurement tool.
- Never invent FPS/memory figures. `UNMEASURED` where tooling cannot capture.
