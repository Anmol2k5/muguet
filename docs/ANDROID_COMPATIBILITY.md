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

| Feature | Supported | Untested | Unsupported | Blocked |
|---|---|---|---|---|
| macOS JIT run | yes (upstream) | — | — | — |
| iOS AOT run | yes (upstream) | — | — | — |
| Android compile | — | in progress (M2) | — | — |
| Android run/render/input/audio | — | — | — | no device (M3+) |
| Multiplayer interop | — | — | — | single-player first |
