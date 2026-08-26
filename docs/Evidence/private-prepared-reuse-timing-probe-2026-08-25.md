# Private Prepared-Reuse Timing Probe

| Field | Value |
| --- | --- |
| Date | 2026-08-25 |
| Fixture | `corpus/golden/built-in-template-rules` |
| Source size | 55 bytes |
| Toolchain | Rust 1.95.0, x86_64-pc-windows-msvc, LLVM 22.1.2 |
| OS | Microsoft Windows 10.0.26200 |
| CPU identifier | AMD64 Family 25 Model 97 Stepping 2, AuthenticAMD |
| Command | `cargo test --release -p fastxslt measures_parse_per_invocation_against_prepared_reuse -- --ignored --nocapture` |
| Claim | Private local seam measurement; no cache default or host-performance claim |

## Compared paths

Stylesheet compilation occurs once before both paths. A correctness check first
requires identical serialization.

The direct reference iteration performs:

```text
admitted source-byte lookup fixed outside timing
    -> XML parse
    -> owned XDM construction
    -> transform with fresh invocation control
    -> serialization
```

The prepared iteration performs:

```text
prepared-set identity lookup + Arc clone
    -> transform over retained owned XDM with fresh invocation control
    -> serialization
```

Each run takes seven samples of 10,000 iterations and reports the median
nanoseconds per complete iteration. Results are consumed through `black_box`.

## Results

| Run | Direct median | Prepared median | Direct/prepared ratio |
| --- | ---: | ---: | ---: |
| 1 | 2,596.3 ns | 804.2 ns | 3.23× |
| 2 | 2,922.9 ns | 806.9 ns | 3.62× |
| 3 | 2,663.5 ns | 814.9 ns | 3.27× |

The prepared path avoids roughly 1.8–2.1 microseconds per iteration on this tiny
fixture and recorded machine. The narrow prepared range also shows the ignored
probe can serve as a repeatable local regression aid.

## Interpretation

This establishes that parse/XDM reuse has measurable value even when transform
and serialization work are very small. It does not establish the benefit for a
large source, many stylesheets, concurrent requests, allocator pressure, cache
misses, snapshot replacement, or managed-host transfer.

The ratio must not be extrapolated. Larger semantic work will reduce the
relative share of parsing; larger documents may increase the absolute avoided
cost while also increasing retained and peak memory. Compilation is deliberately
excluded because both candidate lifecycles are compile-once.

## Limitations

- This is an ignored micro-workload probe, not a Criterion benchmark or CI
  performance gate.
- It measures wall-clock time without allocator-inclusive retained or peak
  memory instrumentation.
- The source is only 55 bytes and the workload is not yet confirmed by an
  intended consumer.
- It does not include ASP.NET, FFI, marshaling, scheduling, contention, cold
  process start, or file import.
- It does not compare eager, lazy, single-flight, eviction, or reconstruction
  policies and therefore cannot select one.
- All-feature verification now runs 56 tests: 51 pass and five manual measurement
  probes are ignored by default.
