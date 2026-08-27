# ASP.NET Native and Isolated FastXSLT Tier Comparison

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Host | ASP.NET Core targeting .NET 8 on 16 logical processors |
| Stylesheet | Pinned XSLT30 `for-004` |
| Tiers | 5, 50, and 500 deterministic `order-item` elements |
| Concurrency | Sequential and four in-flight transformations |
| Runs | Three independent ASP.NET host processes |
| Command | `./scripts/verify-aspnet-workbench.ps1 -TieredBenchmark -TieredSummaryOnly -TieredRequests 250 -TieredConcurrency 4` |
| Claim | Private boundary-cost evidence; not a product benchmark or general engine ranking |

## Compared lifecycle

Both lanes received the same generated source bytes and exact XSLT 2.0
stylesheet, compiled and prepared those resources before measurement, and
materialized and verified the same result for every timed invocation.

The isolated lane used four persistent worker processes with one compiled
stylesheet and prepared XDM document per process. Every invocation paid bounded
frame transport and result transfer across the process boundary. The native
lane used four independent ADR-0008 engine handles in the ASP.NET process. A
managed lease selected one handle per invocation, then the version-zero ABI
copied the request identity and copied the completed result from its outcome
registry. Neither lane concurrently used one engine/worker handle.

The largest tier used 250 requests per lane. Request counts were multiplied by
four for 50 items and by twenty for 5 items. A forced managed collection
occurred between lanes and outside timed regions. Initialization and warm-up
were outside each timed lane.

## Throughput medians

| Path | 5 items sequential | 5 items x4 | 50 items sequential | 50 items x4 | 500 items sequential | 500 items x4 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| FastXSLT isolated | 15,911/s | 70,568/s | 14,714/s | 58,727/s | 5,166/s | 19,668/s |
| FastXSLT native in-process | 347,102/s | 1,037,969/s | 76,349/s | 242,142/s | 7,703/s | 29,520/s |
| Native/isolated ratio of medians | 21.82x | 14.71x | 5.19x | 4.12x | 1.49x | 1.50x |

The native lane's three-run ranges were 334,750-361,167/s, 74,428-77,073/s,
and 7,406-8,231/s sequentially from smallest to largest tier. At four-way
concurrency its ranges were 877,162-1,103,753/s, 185,052-254,434/s, and
28,829-34,439/s.

## Latency medians

Values are medians of each run's measured percentile, in microseconds. Pool
acquisition and boundary work are inside the invocation latency.

| Path/tier | Concurrency | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: |
| Isolated, 5 items | 1 | 55.1 | 103.3 | 145.2 |
| Isolated, 5 items | 4 | 47.6 | 77.6 | 133.0 |
| Native, 5 items | 1 | 2.4 | 3.7 | 4.9 |
| Native, 5 items | 4 | 3.1 | 4.6 | 6.4 |
| Isolated, 50 items | 1 | 60.4 | 111.8 | 169.3 |
| Isolated, 50 items | 4 | 62.8 | 91.1 | 132.5 |
| Native, 50 items | 1 | 11.6 | 18.8 | 27.5 |
| Native, 50 items | 4 | 14.2 | 25.3 | 37.8 |
| Isolated, 500 items | 1 | 186.2 | 257.8 | 356.9 |
| Isolated, 500 items | 4 | 182.2 | 328.9 | 450.4 |
| Native, 500 items | 1 | 107.5 | 191.8 | 230.3 |
| Native, 500 items | 4 | 110.4 | 204.4 | 277.2 |

## Allocation and memory scope

Median managed allocation per completed invocation was:

| Path | 5 items | 50 items | 500 items | Scope |
| --- | ---: | ---: | ---: | --- |
| Isolated sequential | ~3.03 KiB | ~3.03 KiB | ~3.07 KiB | Managed framing/result path; excludes worker Rust allocations |
| Isolated x4 | ~3.17 KiB | ~3.15 KiB | ~3.15 KiB | Managed framing/result path; excludes worker Rust allocations |
| Native sequential | ~454 B | ~463 B | ~464 B | Managed lease/PInvoke/result path; excludes in-process Rust allocations |
| Native x4 | ~458 B | ~455 B | ~456 B | Managed lease/PInvoke/result path; excludes in-process Rust allocations |

The isolated four-worker aggregate working-set medians after measurement were
about 17.3 MiB, 17.8 MiB, and 21.1 MiB across the three tiers. Each process owns
its compiled/prepared generation, so this makes process-pool multiplication
visible.

Native initialization records the whole ASP.NET process after four independent
engine handles are created. That process already contains the managed host and
other benchmark state, and later tier readings are cumulative. It cannot
attribute retained bytes to native engines and is intentionally not presented
as a native-engine footprint. Rust-side allocation instrumentation and an
isolated native-only host remain required for a comparable retained-memory
claim.

Windows process CPU time was quantized at roughly 15.625 ms while several
native lanes completed faster than that. CPU percentages are retained by the
harness but are not precise enough for a ranking.

## Interpretation

The process boundary is a large part of tiny warm-transform cost in this
workbench: the native median was about 21.82 times the isolated median at five
items. As semantic work increased, the ratio narrowed to about 1.49 at 500
items. That is direct evidence that fixed isolated transport is amortized by
larger transforms, while the in-process candidate preserves a substantial
small-call advantage.

Four independent native handles also produced useful bounded concurrency
without sharing one handle: median throughput was 2.99x, 3.17x, and 3.83x the
corresponding native sequential lanes. This is an execution observation, not a
same-handle thread-safety contract.

The experiment does not choose a production default. The isolated mode has
demonstrated hard worker termination and replacement; the native mode cannot
offer that guarantee in-process and still lacks cancellation, budget, snapshot
replacement, and full diagnostic lifecycle parity. The test exercises one
narrow admitted evaluator, three synthetic source sizes, short runs, copied
string results, and no representative consumer workload. AR-0002 therefore
remains Proposed.
