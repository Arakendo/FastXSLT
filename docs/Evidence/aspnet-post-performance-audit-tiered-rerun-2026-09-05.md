# ASP.NET Post-Performance-Audit Tiered Rerun

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| FastXSLT checkpoint | `10d2f30` |
| Host | AMD Ryzen 7 7800X3D, 16 logical processors, Windows `10.0.26200.0` |
| Toolchain | Rust 1.95.0; .NET SDK/runtime `10.0.100-preview.7.25380.108` |
| Managed target | `net10.0` |
| Stylesheet | Pinned XSLT30 `for-004`; Microsoft lane uses the reviewed XSLT 1.0 equivalent |
| Tiers | 5, 50, and 500 deterministic `order-item` elements |
| Runs | Five fresh ASP.NET host processes; medians of one tiered sample per process |
| Command | `./scripts/verify-aspnet-workbench.ps1 -TargetFramework net10.0 -TieredBenchmark -TieredSummaryOnly -TieredRequests 1000 -TieredConcurrency 4 -MeasurementRuns 1` |
| Comparison | Post-specialization baseline in [`for-004` exact-decimal activated-path evidence](for-004-exact-decimal-activated-path-2026-09-04.md) |
| Claim | Private same-machine drift evidence for one narrow warm workload; not a general engine ranking |

## Method

The rerun repeats the latest comparable post-specialization protocol: 10,000
transforms at five items, 4,000 at 50 items, and 1,000 at 500 items for each
lane. Every process compiled and prepared the workload before measurement,
materialized and checked every result, and measured sequential execution plus
four independent native handles or persistent isolated workers.

The local SaxonCS overlay was not requested. Its dependency graph was rejected
by the repository's vulnerability gate during the prior .NET 10 comparison.
Microsoft's lane remains a semantically equivalent but algorithmically
different XSLT 1.0 formulation and is shown only as a local comparison.

## Current throughput medians

Values are completed transforms per second.

| Engine path | 5 sequential | 5 x4 | 50 sequential | 50 x4 | 500 sequential | 500 x4 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| FastXSLT isolated | 20,364 | 75,899 | 18,530 | 74,263 | 11,490 | 42,861 |
| FastXSLT native | 291,852 | 1,011,030 | 167,920 | 535,010 | 27,552 | 111,602 |
| Microsoft equivalent XSLT 1.0 | 194,854 | 418,074 | 15,014 | 51,248 | 311 | 949 |

## Drift from the latest comparable FastXSLT baseline

Positive values mean higher current throughput. The prior baseline was recorded
immediately after the allocation-free exact-decimal specialization with the
same machine, target, SDK, tiers, request counts, and concurrency.

| FastXSLT path | 5 sequential | 5 x4 | 50 sequential | 50 x4 | 500 sequential | 500 x4 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Isolated | -3.6% | -9.3% | -5.2% | -6.2% | -0.5% | +2.0% |
| Native | -4.9% | -2.3% | +1.1% | -3.8% | +0.6% | +3.7% |

The 500-item lanes, where semantic work dominates fixed boundary costs, remain
effectively stable or slightly faster. Native lanes through 50 items also stay
within about five percent of the prior record. The isolated five- and 50-item
lanes show a modest negative signal, strongest at five items with four workers,
but this run does not localize it to worker framing, scheduling, engine work, or
ordinary machine noise.

Two of the five native 500-item processes were materially slower. Sequential
observations were approximately 29,250, 28,340, 27,552, 17,668, and 18,635
transforms per second; four-way observations were approximately 118,301,
111,602, 118,156, 79,730, and 82,252. The medians remain close to the prior
baseline, but the distribution is bimodal enough that the small positive drift
must not be treated as a precise improvement.

## Current FastXSLT latency medians

Values are microseconds. Each cell is the median of the five process-level
percentiles.

| Path/tier | Concurrency | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: |
| Isolated, 5 items | 1 | 42.4 | 79.6 | 119.9 |
| Isolated, 5 items | 4 | 45.6 | 67.7 | 99.8 |
| Native, 5 items | 1 | 2.6 | 4.0 | 5.1 |
| Native, 5 items | 4 | 3.0 | 4.7 | 7.8 |
| Isolated, 50 items | 1 | 47.6 | 87.8 | 128.7 |
| Isolated, 50 items | 4 | 49.7 | 72.0 | 103.2 |
| Native, 50 items | 1 | 5.5 | 6.9 | 10.1 |
| Native, 50 items | 4 | 6.0 | 9.3 | 13.5 |
| Isolated, 500 items | 1 | 79.6 | 126.1 | 221.3 |
| Isolated, 500 items | 4 | 87.7 | 124.3 | 170.8 |
| Native, 500 items | 1 | 32.6 | 57.5 | 75.1 |
| Native, 500 items | 4 | 31.0 | 49.0 | 69.1 |

Managed allocation remains approximately 455 bytes per five-item native call
and 464 bytes per 50/500-item native call in the ordinary observations. The
isolated managed side remains approximately 3.1 KiB per call. These figures do
not include Rust allocations. Median aggregate worker working set was about
21.8 MiB, 22.2 MiB, and 25.8 MiB across the three tiers.

## Comparison perspective

On this exact workload, current native FastXSLT is about 1.50x and 2.42x the
throughput of the Microsoft equivalent at five items, about 11.18x and 10.44x
at 50 items, and about 88.55x and 117.56x at 500 items for sequential and
four-way execution respectively. Isolated FastXSLT pays the expected fixed
transport cost at five items, then exceeds the Microsoft equivalent by about
1.23x/1.45x at 50 items and 36.93x/45.15x at 500 items.

Those ratios are not an engine ranking. The Microsoft stylesheet uses XSLT 1.0
recursion to express work performed by the exact XSLT 2.0 `for` expression in
FastXSLT, and its retained algorithm scales very differently.

## Disposition

No material post-audit regression is established for this workload. The
allocation-free exact-decimal gain remains intact, and the larger FastXSLT
lanes are stable against the latest comparable record. Preserve the current
implementation. Treat the small fixed-overhead isolated decline and native
500-item bimodality as observations to revisit only if they recur under a
longer randomized or time-based benchmark; they do not reopen the completed
performance audit by themselves.
