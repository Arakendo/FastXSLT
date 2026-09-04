# ASP.NET .NET 10 Tiered Target Comparison

| Field                      | Value                                                                                                                                   |
| -------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| Date                       | 2026-09-03                                                                                                                              |
| FastXSLT source checkpoint | `3cf41dc`; `net10.0` retarget is part of this evidence change                                                                           |
| Host                       | AMD Ryzen 7 7800X3D, 16 logical processors, Windows `10.0.26200.0`                                                                      |
| Toolchain                  | Rust 1.95.0; .NET SDK/runtime 10.0.100-preview.7                                                                                        |
| Managed target             | `net10.0`                                                                                                                               |
| Stylesheet                 | Pinned XSLT30 `for-004`; Microsoft lane uses the reviewed XSLT 1.0 equivalent                                                           |
| Tiers                      | 5, 50, and 500 deterministic `order-item` elements                                                                                      |
| Runs                       | Five fresh ASP.NET host processes; medians of one tiered sample per process                                                             |
| Command                    | `./scripts/verify-aspnet-workbench.ps1 -TieredBenchmark -TieredSummaryOnly -TieredRequests 250 -TieredConcurrency 4 -MeasurementRuns 1` |
| Claim                      | Private target-framework and boundary-cost evidence; not a production target selection or general engine ranking                        |

## Change and method

The unsupported ASP.NET workbench moved from `net8.0` to `net10.0`. Its verifier
now copies the native library beside the `net10.0` managed assembly. Historical
.NET 8 evidence retains its original target label.

The tiered workload and request scaling are unchanged from the immediately
preceding
[performance-drift run](aspnet-tiered-performance-drift-2026-09-03.md):
5,000 requests at five items, 1,000 at 50 items, and 250 at 500 items. Every
lane compiles and prepares before timing, warms once, materializes every result,
and checks semantic equivalence.

FastXSLT isolated uses four persistent worker processes. FastXSLT native uses
four independent in-process handles. Microsoft's `XslCompiledTransform` uses
the semantically equivalent but algorithmically different XSLT 1.0 stylesheet.

## Current throughput medians

Values are completed transforms per second.

| Engine path                   | 5 sequential |    5 x4 | 50 sequential |   50 x4 | 500 sequential | 500 x4 |
| ----------------------------- | -----------: | ------: | ------------: | ------: | -------------: | -----: |
| FastXSLT isolated             |       19,896 |  71,885 |        15,980 |  60,652 |          5,347 | 22,110 |
| FastXSLT native               |      261,828 | 769,373 |        62,429 | 222,841 |          7,372 | 27,021 |
| Microsoft equivalent XSLT 1.0 |      140,976 | 431,001 |        17,357 |  55,322 |            294 |    807 |

## Change from the immediately preceding `net8.0` run

Positive values mean higher `net10.0` throughput. The source checkpoint,
machine, tier sizes, request counts, concurrency, and .NET SDK are the same.
The earlier run included the local SaxonCS adapter and package; this run could
not, so the comparison is close but not a perfectly isolated target-framework
A/B.

| Engine path                   | 5 sequential |   5 x4 | 50 sequential |  50 x4 | 500 sequential | 500 x4 |
| ----------------------------- | -----------: | -----: | ------------: | -----: | -------------: | -----: |
| FastXSLT isolated             |       +15.0% | +11.6% |         +6.8% |  +6.4% |          +1.7% |  +2.5% |
| FastXSLT native               |       -11.8% | -10.9% |        -14.3% | -17.8% |         -16.6% | -11.7% |
| Microsoft equivalent XSLT 1.0 |       -14.1% |  -6.1% |        -11.4% |  -4.7% |         +13.0% | -18.4% |

The direction differs by boundary. Isolated FastXSLT improved at every tier,
especially for tiny work. Native FastXSLT declined consistently even though it
executes the same Rust library. This points toward managed runtime, P/Invoke,
host scheduling, or short-lane measurement effects rather than an XSLT
evaluator change. The experiment does not isolate which mechanism is
responsible.

## Current FastXSLT latency medians

Values are microseconds. Each value is the median of the five runs' reported
percentile.

| Path/tier           | Concurrency |   p50 |   p95 |   p99 |
| ------------------- | ----------: | ----: | ----: | ----: |
| Isolated, 5 items   |           1 |  45.0 |  78.3 | 105.0 |
| Isolated, 5 items   |           4 |  48.2 |  64.0 |  86.9 |
| Native, 5 items     |           1 |   3.5 |   3.8 |   5.5 |
| Native, 5 items     |           4 |   3.9 |   6.2 |   9.5 |
| Isolated, 50 items  |           1 |  56.7 |  95.2 | 131.3 |
| Isolated, 50 items  |           4 |  61.2 |  91.1 | 138.0 |
| Native, 50 items    |           1 |  15.2 |  19.0 |  22.0 |
| Native, 50 items    |           4 |  16.0 |  21.9 |  30.6 |
| Isolated, 500 items |           1 | 178.2 | 245.2 | 359.1 |
| Isolated, 500 items |           4 | 165.0 | 251.7 | 299.1 |
| Native, 500 items   |           1 | 131.7 | 143.5 | 205.9 |
| Native, 500 items   |           4 | 133.5 | 216.9 | 235.0 |

The p50 values reinforce the throughput movement. Compared with the preceding
`net8.0` run, isolated p50 improved from 51.1 to 45.0 microseconds at five items
and from 182.9 to 178.2 at 500. Native p50 moved from 2.9 to 3.5 microseconds at
five items and from 109.6 to 131.7 at 500.

Managed allocation did not show a corresponding native increase. Sequential
native allocation remained approximately 454 bytes per five-item call and 463-
464 bytes per 50/500-item call. Isolated managed allocation remained roughly
3.0-3.2 KiB per call. Rust-side allocation remains outside both measurements.

## August baseline perspective

Compared with the original August target-specific records rather than the noisy
same-day run, current isolated throughput is down 23.4%/15.4% at five items,
down 6.2% sequential and up 1.6% concurrently at 50, and down 3.7%/3.5% at 500.
Current native throughput is down 24.6%/25.9%, 18.2%/8.0%, and 4.3%/8.5% across
the corresponding tiers.

The .NET 10 target therefore improves today's isolated result but does not erase
the broader tiny-native drift. Larger isolated work remains essentially stable;
larger native work is closer to the August baseline than tiny native work.

## SaxonCS safety gate

The local SaxonCS-HE 13.0.0 overlay was requested first, but restore failed
because `TreatWarningsAsErrors` promoted current NuGet audit findings to build
errors. The graph reports AngleSharp 1.2.0 under moderate advisory
`GHSA-pgww-w46g-26qg` and System.Security.Cryptography.Xml 10.0.7 under five
high-severity advisories: `GHSA-23rf-6693-g89p`, `GHSA-8q5v-6pqq-x66h`,
`GHSA-cvvh-rhrc-wg4q`, `GHSA-g8r8-53c2-pm3f`, and
`GHSA-mmjf-rqrv-855v`.

The audit was not disabled, the ignored local overlay was not modified, and no
SaxonCS .NET 10 number is reported. A later local comparison requires a clean
dependency graph or an explicit, separately documented quarantined override.

## Disposition

The .NET 10 workbench target is viable for FastXSLT's native, isolated, and
Microsoft-equivalent lanes. The target is still preview tooling and remains an
AR-0002 experiment rather than a supported product contract.

Before attributing the native decline or choosing a host target, run a longer
time-based A/B with both target frameworks, identical optional dependencies,
fixed power state, randomized lane order, and separate timing for managed lease,
P/Invoke, registry outcome publication, result copying, and release.
