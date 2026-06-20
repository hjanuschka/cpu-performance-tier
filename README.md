# cpu-performance-tier

`cpu-performance-tier` is a small Rust crate that implements the CPU
performance tier classification algorithm used by Chromium's experimental Web
CPU Performance API.

The goal is to make the core algorithm easy for other browser engines, test
suites, and tooling to consume without depending on Chromium internals.

## API

```rust
use cpu_performance_tier::{tier_from_cpu_info, PerformanceTier};

let tier = tier_from_cpu_info("Intel(R) Core(TM) Ultra 7 155H", 8);
assert_eq!(tier, PerformanceTier::Ultra);
```

The crate exposes:

- `tier_from_cores(cores)` for the simple core-count fallback.
- `tier_from_cpu_info(cpu_model, cores)` for the full classifier.
- `split_cpu_model(cpu_model)` for normalizing CPU model strings.
- `Manufacturer` and `PerformanceTier` enums.

## Tier values

- `Unknown`
- `Low`
- `Mid`
- `High`
- `Ultra`

## Provenance

This is a Rust port of Chromium's `content/browser/cpu_performance` algorithm.
The original implementation is BSD-licensed by The Chromium Authors.

## Status

Initial reference crate. The public API is intentionally small so it can track
spec and Chromium changes closely.
