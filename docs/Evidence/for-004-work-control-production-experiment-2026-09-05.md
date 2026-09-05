# `for-004` Work-Control Production Experiment

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Reference checkpoint | `38b65cb` |
| Status | Complete; candidate rejected and production source restored |
| Related review | [Performance optimization review](../Reviews/performance-optimization-review-2026-09-04.md) |

## Question

Can the existing `for-004` work-control path be made materially cheaper without
removing a charge, weakening finite-budget rejection, reducing cancellation
observation, or changing failure ordering?

The representative ASP.NET workbench does not use unbounded XPath limits. Its
default maps both XPath node visits and XPath operations to a finite one-million
unit ceiling. Skipping bookkeeping only for unbounded domains would therefore
not improve this workload and was not tested.

## Candidate

A temporary safe candidate made two implementation-only changes:

- the deterministic `cancelling_on_charge` fault field and branch were compiled
  only for Rust tests, which are their only caller;
- `charge`, its work-domain limit selectors, and the cancellation poll were
  marked inline so release compilation could specialize the small dispatch.

Every production charge, atomic cancellation load, domain limit lookup,
remaining-budget comparison, decrement, and failure result remained present and
in the same order. The candidate did not change the workbench's finite limits.
Focused cancellation and budget tests passed before measurement.

## ASP.NET comparison

The existing .NET 10 workbench ran the 500-item tier through the native
in-process boundary. Three fresh processes were sampled for each build. A short
500-request comparison was followed by a longer 2,000-request comparison after
the short run produced contradictory sequential and concurrent results.

```powershell
./scripts/verify-aspnet-workbench.ps1 -TargetFramework net10.0 `
  -TieredBenchmark -TieredSummaryOnly -TieredRequests 2000 `
  -TieredConcurrency 4 -MeasurementRuns 1
```

| Longer run | Concurrency | Samples (transforms/s) | Median | Median p50 |
| --- | ---: | --- | ---: | ---: |
| Candidate | 1 | 23,861 / 17,051 / 17,918 | 17,918/s | 60.2 us |
| Reference | 1 | 16,684 / 22,988 / 26,033 | 22,988/s | 36.4 us |
| Candidate | 4 | 73,948 / 75,827 / 75,269 | 75,269/s | 58.4 us |
| Reference | 4 | 71,984 / 70,685 / 66,094 | 70,685/s | 64.9 us |

The candidate was about 22% slower by the sequential medians and 6.5% faster by
the four-way medians. The preceding shorter comparison pointed the other way:
the candidate was about 4.7% faster sequentially and 12.5% slower at four-way
concurrency. Several runs were visibly bimodal, including p50 movement between
roughly 34 and 60 microseconds. Managed allocation was unchanged.

The experiment was intentionally run candidate-first and was not randomized;
the machine-state shift makes the exact ratios unsuitable as stable performance
claims. More importantly, neither sample duration produced a consistent gain
across the two lanes, and the direction reversed between durations.

## Disposition

Reject the candidate and retain the complete production implementation. The
source was restored before this evidence was recorded.

The audit's work-control timing question is closed for the current `for-004`
path: its `8N + 1` charge shape is known, but this narrow implementation cleanup
does not show a repeatable consumer-visible benefit. Reopen only when a profiler
attributes material production time to a named control operation or a candidate
can be compared with better isolated, randomized evidence while preserving the
same exact control semantics.

