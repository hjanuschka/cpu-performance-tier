# cpu-performance-tier

`cpu-performance-tier` is a small Rust crate that implements the CPU
performance tier classification algorithm used by Chromium's experimental Web
CPU Performance API.

The goal is to make the core algorithm easy for other browser engines, test
suites, and tooling to consume without depending on Chromium internals. The
repository provides both a Rust crate and a small C++ implementation generated
from the same rule/test data for engines that cannot consume Rust.

## API

```rust
use cpu_performance_tier::{tier_from_cpu_info, PerformanceTier};

let tier = tier_from_cpu_info("Intel(R) Core(TM) Ultra 7 155H", 8);
assert_eq!(tier, PerformanceTier::Ultra);
```

Browser engines provide the OS CPU model string and logical processor count.

The crate exposes:

- `tier_from_cores(cores)` for the simple core-count fallback.
- `tier_from_cpu_info(cpu_model, cores)` for the full classifier.
- `split_cpu_model(cpu_model)` for normalizing CPU model strings.
- `Manufacturer` and `PerformanceTier` enums.

## Optional host information helpers

By default the crate only provides the pure classifier. If an embedder wants a
convenience API for local host probing, enable the optional `host-info` feature:

```toml
[dependencies]
cpu-performance-tier = { version = "0.1", features = ["host-info"] }
```

Then call:

```rust
use cpu_performance_tier::{host_cpu_info, tier_from_host};

let info = host_cpu_info();
let tier = tier_from_host();
```

The `host-info` feature adds no crate dependencies. It uses:

- Rust `std::thread::available_parallelism()` for logical processor count.
- `/proc/cpuinfo` on Linux and Android for the CPU model string.
- `sysctl -n machdep.cpu.brand_string` on macOS-like platforms.
- `PROCESSOR_IDENTIFIER` on Windows.

Engines can still bypass these helpers and pass their own platform abstraction
values to `tier_from_cpu_info`.

## Tier values

- `Unknown`
- `Low`
- `Mid`
- `High`
- `Ultra`

## Engine integration guide

This crate contains only the deterministic classification algorithm. Browser
engines remain responsible for collecting the local CPU information and deciding
when to expose the resulting tier.

### Required inputs

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

### Where engines typically get the inputs

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

### Suggested engine flow

1. During browser startup, compute a non-blocking fallback with
   `tier_from_cores`.
2. On a background thread, collect the CPU model string if that may block.
3. Recompute with `tier_from_cpu_info(cpu_model, logical_cores)`.
4. Cache the tier for the browser session and expose that cached value to the
   renderer or web-facing API.

This mirrors Chromium's split between a fast core-count fallback and a more
accurate asynchronous CPU-model classifier.

### Interoperability notes

To get matching behavior across engines:

- Pass the raw OS CPU brand/model string before engine-specific cleanup.
- Pass logical processor count, not physical core count.
- Keep the crate version pinned in engine builds and update deliberately.
- Add web-platform or engine-level tests that cover the model strings in this
  crate's Chromium-derived test set.

The crate intentionally does not probe the host system. That keeps OS access,
privacy review, threading, caching, and feature gating under the embedding
engine's control.

## Shared Rust/C++ implementation

The tier pattern rules and Chromium-derived test vectors live in
`algorithm/cpu_performance.json`. `tools/generate.py` produces:

- `src/generated.rs` for the Rust crate.
- `cpp/generated_rules.inc` for the C++ implementation.
- `cpp/generated_tests.cc` for the C++ test binary.

Generated files are checked in so embedders can vendor either implementation
without running the generator. CI runs `python3 tools/generate.py --check`, Rust
tests, and C++ CTest coverage to prevent drift.

C++ consumers can vendor `cpp/cpu_performance_tier.h`,
`cpp/cpu_performance_tier.cc`, and `cpp/generated_rules.inc`.

## Provenance

This is a Rust and C++ port of Chromium's
`content/browser/cpu_performance` algorithm. The original implementation is
BSD-licensed by The Chromium Authors.

## Tests

The unit tests include the Chromium `cpu_performance_unittest.cc` cases for the
pure classifier: core-count fallback, CPU model normalization, CPU-info tiering,
and integer tier conversion. Chromium's browser initialization test is not
ported because this crate intentionally does not perform OS probing or global
browser-process initialization.

## Status

Initial reference crate. The public API is intentionally small so it can track
spec and Chromium changes closely.
