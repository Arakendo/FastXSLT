# Native Registry Live-Use High-Water Measurement

| Field | Value |
| --- | --- |
| Date | 2026-08-31 |
| Scope | Test-only AR-0017 host-shaped native registry accounting |
| Workload | Two overlapping four-engine generations, eight controls, 64 results, and 64 diagnostics |
| Mode | Optimized sacrificial Rust test process on Windows |
| Claim | One legitimate live-use calibration point; not a production threshold |

## Shape

The probe mirrors the largest currently exercised native benchmark pool
(concurrency four) while adding generation overlap. It retains four old and
four replacement compiled/prepared `for-004` engines, then eight active control
handles and a deliberately delayed burst of 64 successful results plus 64
bounded diagnostics. Creation outcomes are consumed immediately, matching the
managed wrapper. Measurements are phased so engine retention is visible before
scalar controls and outcomes are added.

## Observations

| Checkpoint | Engines | Controls | Outcomes | Outcome payload | Working set |
| --- | ---: | ---: | ---: | ---: | ---: |
| Baseline | 0 | 0 | 0 | 0 | 4,993,024 B |
| Two generations | 8 | 0 | 0 | 0 | 5,795,840 B |
| Active controls | 8 | 8 | 0 | 0 | not separately sampled |
| Delayed result/diagnostic burst | 8 | 8 | 128 | 8,640 B | 5,840,896 B |
| Old generation retired; burst disposed | 4 | 0 | 0 | 0 | 5,799,936 B |
| Current generation released | 0 | 0 | 0 | 0 | 5,824,512 B |

Eight tiny prepared engines added 802,816 bytes over baseline in this process,
about 100 KiB per engine as a process-wide delta. Eight controls and 128 delayed
outcomes increased the measured peak by another 45,056 bytes; exact owned
outcome payload was 8,640 bytes. The maximum legitimate registry cardinality in
this shape was 144 handles, compared with 100,000 handles per class in the
separate abandonment probe.

All handles were released and all logical cardinalities returned to zero.
Working-set samples near the end vary by tens of kilobytes and do not show
monotonic reclamation, reinforcing that process working set is not exact
per-object accounting.

## Limits

- `for-004` is a tiny prepared workload. Larger documents/stylesheets may make
  engine retention dominate handle-map overhead.
- Four engines per generation reflects the existing ×4 benchmark, not a
  supported maximum or consumer requirement.
- Result/diagnostic bursts are synthetic delayed disposal, not measured ASP.NET
  request concurrency.
- One run cannot establish percentiles, allocator stability, or a threshold.

This evidence satisfies the first live-use-versus-abandonment comparison. Any
policy proposal must still cover larger prepared generations and actual
consumer burst requirements. The observed gap makes a 100,000-handle abuse
ceiling an especially poor substitute for host calibration.

## Reproduction

```text
cargo test --release -p fastxslt-dotnet-workbench measure_host_shaped_registry_high_water --all-features -- --ignored --nocapture
```
