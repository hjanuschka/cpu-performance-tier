# Engine integration guide

This crate contains only the deterministic classification algorithm. Browser
engines remain responsible for collecting the local CPU information and deciding
when to expose the resulting tier.

## Required inputs

Call:

```rust
use cpu_performance_tier::{tier_from_cpu_info, tier_from_cores};

let tier = tier_from_cpu_info(cpu_model, logical_cores);
```

Inputs:

- `cpu_model`: the OS-provided CPU model or brand string.
- `logical_cores`: the number of logical processors available to the browser
  process.

If the model string is not available, use `tier_from_cores(logical_cores)` as a
fallback. If the core count is not available or is `<= 0`, the full algorithm
returns `PerformanceTier::Unknown`.

## Where engines typically get the inputs

Examples of platform sources:

- Linux/ChromeOS: parse the CPU model from `/proc/cpuinfo`; get logical CPU
  count from the scheduler or standard library.
- macOS: read `machdep.cpu.brand_string` with `sysctl`; get logical CPU count
  from `hw.logicalcpu` or equivalent process APIs.
- Windows: read the processor brand string from OS processor information APIs,
  registry, WMI, or existing engine platform abstraction; get logical CPU count
  from system information APIs.
- Android: use the engine's existing device/CPU abstraction where available;
  otherwise fall back to logical core count.

Engines should prefer their existing platform abstraction if it already matches
what they use for telemetry or scheduling decisions.

## Suggested engine flow

1. During browser startup, compute a non-blocking fallback with
   `tier_from_cores`.
2. On a background thread, collect the CPU model string if that may block.
3. Recompute with `tier_from_cpu_info(cpu_model, logical_cores)`.
4. Cache the tier for the browser session and expose that cached value to the
   renderer or web-facing API.

This mirrors Chromium's split between a fast core-count fallback and a more
accurate asynchronous CPU-model classifier.

## Interoperability notes

To get matching behavior across engines:

- Pass the raw OS CPU brand/model string before engine-specific cleanup.
- Pass logical processor count, not physical core count.
- Keep the crate version pinned in engine builds and update deliberately.
- Add web-platform or engine-level tests that cover the model strings in this
  crate's Chromium-derived test set.

The crate intentionally does not probe the host system. That keeps OS access,
privacy review, threading, caching, and feature gating under the embedding
engine's control.

## Publishing releases

Release tags of the form `vX.Y.Z` publish the crate to crates.io through GitHub
Actions. The repository needs a `CARGO_REGISTRY_TOKEN` secret with permission to
publish this crate.
