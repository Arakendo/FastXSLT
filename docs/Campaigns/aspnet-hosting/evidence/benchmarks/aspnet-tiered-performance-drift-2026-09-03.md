# ASP.NET Tiered Performance Drift

| Field               | Value                                                                                                                                                 |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| Date                | 2026-09-03                                                                                                                                            |
| FastXSLT checkpoint | `8735c97`                                                                                                                                             |
| Host                | AMD Ryzen 7 7800X3D, 16 logical processors, Windows `10.0.26200.0`                                                                                    |
| Toolchain           | Rust 1.95.0; .NET SDK 10.0.100-preview.7 targeting `net8.0`                                                                                           |
| Stylesheet          | Pinned XSLT30 `for-004`; Microsoft lane uses the reviewed XSLT 1.0 equivalent                                                                         |
| Tiers               | 5, 50, and 500 deterministic `order-item` elements                                                                                                    |
| Runs                | Five fresh ASP.NET host processes; medians of one tiered sample per process                                                                           |
| Command             | `./scripts/verify-aspnet-workbench.ps1 -LocalSaxonCs -TieredBenchmark -TieredSummaryOnly -TieredRequests 250 -TieredConcurrency 4 -MeasurementRuns 1` |
| Claim               | Same-machine private drift evidence for one narrow warm workload; not a general engine ranking                                                        |

## Method

The current harness compiled and prepared every lane before measurement,
materialized and checked every result, and preserved the earlier request-count
scaling: 5,000 requests at five items, 1,000 at 50 items, and 250 at 500 items.
Sequential and four-in-flight measurements used the same generated input and
result as the August evidence.

FastXSLT isolated used four persistent worker processes; FastXSLT native used
four independent in-process handles. SaxonCS-HE 13.0.0 remained a local,
gitignored in-process comparison over the exact XSLT 2.0 stylesheet. Microsoft's
`XslCompiledTransform` used the semantically equivalent but algorithmically
different XSLT 1.0 stylesheet.

The current machine used a preview .NET 10 SDK to build a `net8.0` target. The
August records identify the target but not the exact SDK. This and ordinary
machine load limit attribution of small changes to FastXSLT code alone.

## Current throughput medians

Values are completed transforms per second.

| Engine path                   | 5 sequential |    5 x4 | 50 sequential |   50 x4 | 500 sequential | 500 x4 |
| ----------------------------- | -----------: | ------: | ------------: | ------: | -------------: | -----: |
| FastXSLT isolated             |       17,305 |  64,402 |        14,964 |  57,027 |          5,258 | 21,576 |
| FastXSLT native               |      296,990 | 863,767 |        72,861 | 271,128 |          8,836 | 30,585 |
| SaxonCS exact                 |       25,993 | 126,686 |        15,417 |  36,217 |          1,835 | 14,593 |
| Microsoft equivalent XSLT 1.0 |      164,209 | 458,817 |        19,580 |  58,023 |            261 |    989 |

SaxonCS's 500-item concurrent lane remained bimodal: the five observations were
approximately 4,110, 5,784, 14,593, 26,146, and 26,318 transforms per second.
Its median is reported, but is not stable enough to support a precise relative
claim.

## Drift from the August evidence

The isolated, SaxonCS, and Microsoft comparisons use the five-process August
baseline in
[ASP.NET Tiered Workload and Bounded Concurrency](aspnet-tiered-workload-and-bounded-concurrency-2026-08-26.md).
The native comparison uses the three-process baseline in
[ASP.NET Native and Isolated FastXSLT Tier Comparison](aspnet-native-vs-isolated-tiered-comparison-2026-08-26.md),
because that was the first recorded native tier matrix. Positive values mean
higher current throughput.

| Engine path                   | 5 sequential |   5 x4 | 50 sequential |  50 x4 | 500 sequential |  500 x4 |
| ----------------------------- | -----------: | -----: | ------------: | -----: | -------------: | ------: |
| FastXSLT isolated             |       -33.4% | -24.2% |        -12.1% |  -4.5% |          -5.3% |   -5.8% |
| FastXSLT native               |       -14.4% | -16.8% |         -4.6% | +12.0% |         +14.7% |   +3.6% |
| SaxonCS exact                 |        +6.2% | +38.4% |        +12.2% | -16.7% |          +2.1% | +145.3% |
| Microsoft equivalent XSLT 1.0 |       +18.5% |  +0.4% |        +12.8% |  +2.0% |          -1.3% |   +5.7% |

The mixed movement across engines demonstrates meaningful run/environment
noise. FastXSLT's tiny isolated and native lanes have nevertheless moved down
enough to warrant later profiling if the result repeats under longer-duration
measurement. The larger native lanes did not regress: 500-item sequential
throughput rose about 14.7%, while the isolated 500-item lanes remained within
about 6% of the August baseline.

## Current FastXSLT latency medians

Values are microseconds. Each value is the median of the five runs' reported
percentile.

| Path/tier           | Concurrency |   p50 |   p95 |   p99 |
| ------------------- | ----------: | ----: | ----: | ----: |
| Isolated, 5 items   |           1 |  51.1 |  88.5 | 121.2 |
| Isolated, 5 items   |           4 |  54.0 |  78.4 | 106.2 |
| Native, 5 items     |           1 |   2.9 |   3.9 |   5.3 |
| Native, 5 items     |           4 |   3.5 |   5.9 |   8.4 |
| Isolated, 50 items  |           1 |  60.9 |  96.7 | 134.9 |
| Isolated, 50 items  |           4 |  65.4 |  91.1 | 124.3 |
| Native, 50 items    |           1 |  12.7 |  19.9 |  23.3 |
| Native, 50 items    |           4 |  13.0 |  17.4 |  26.7 |
| Isolated, 500 items |           1 | 182.9 | 238.7 | 357.6 |
| Isolated, 500 items |           4 | 169.7 | 249.4 | 366.7 |
| Native, 500 items   |           1 | 109.6 | 123.8 | 201.4 |
| Native, 500 items   |           4 | 111.0 | 196.9 | 213.0 |

## Boundary and allocation observations

The native/isolated throughput ratio remains strongly workload-shaped. It is
17.16x, 4.87x, and 1.68x sequentially and 13.41x, 4.75x, and 1.42x with four
in-flight calls from the smallest to largest tier. The earlier conclusion still
holds: native has a very large tiny-call advantage, while fixed process
transport is increasingly amortized by real transform work.

FastXSLT isolated managed allocation remains approximately 3.0-3.2 KiB per
call. Native remains about 454-515 bytes per call. The larger native lanes are
roughly 10-13% above the earlier managed-side observations; Rust allocations
remain excluded, so this is a host-boundary clue rather than a total-engine
allocation regression.

Median aggregate working set for the four isolated workers was approximately
19.5 MiB, 19.9 MiB, and 23.4 MiB across the three tiers, about 9-11% above the
August five-run observations. Whole-process working set cannot attribute that
change to prepared XDM, compiled state, binary growth, allocator behavior, or
the operating environment.

## Interpretation

The architectural performance story remains intact:

- warm native execution is exceptionally cheap for tiny calls;
- isolated transport becomes a much smaller fraction of larger transforms;
- four independent native handles retain useful scaling;
- on this exact workload, FastXSLT remains ahead of SaxonCS at 500 items in
  both host modes, subject to Saxon's concurrent variability; and
- no broad engine ranking follows from one specialized stylesheet.

The tiny-work decline is the actionable signal. Before treating it as an engine
regression, repeat with longer timed lanes, pin the exact .NET SDK and power
state, and separate worker framing, native ABI accounting, runtime evaluation,
and serialization. The current evidence does not justify reverting semantic,
budget, registry, or diagnostic work merely to recover a short-run number.
