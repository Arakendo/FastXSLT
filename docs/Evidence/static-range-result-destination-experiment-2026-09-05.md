# Static-Range Result-Destination Experiment

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Source checkpoint | `1992a88` plus the temporary prototype described here |
| Status | Complete negative admission experiment; candidate not retained |
| Workload | Result-heavy 100-, 1,000-, and 5,000-element construction fixture |
| Related review | [Performance optimization review](../Reviews/performance-optimization-review-2026-09-04.md) |

## Question

Does executing each iteration of a statically bounded integer range directly
into its caller's `Vec<ResultNode>` remove enough transient result vectors to
improve the result-heavy workload without an unfavorable peak-memory tradeoff?

This was deliberately narrower than a general `ResultBuilder`. Only the static
integer-range instruction used the destination form. Every other instruction
and nesting boundary retained the complete result-returning contract.

## Prototype and conservation

The safe prototype factored sequence execution into a destination-taking
helper. The static range reused its caller's result vector rather than creating
one result vector per iteration and moving those nodes into a range-owned
vector. A test-only switch retained the complete implementation as the oracle.

Focused tests produced identical semantic trees and exact XPath-operation,
XSLT-instruction, result-node, and result-text-byte totals. An injected
result-node cancellation produced the same structured failure at the same
charge count. No serializer, public API, ABI, prepared state, unsafe code, or
cross-invocation state changed.

## Focused allocation result

At 5,000 constructed elements, one release allocation observation reported:

| Path | Allocation requests | Requested bytes | Retained bytes | Largest live/requested observation | Median execution |
| --- | ---: | ---: | ---: | ---: | ---: |
| Direct destination | 35,028 | 10,271,747 | 4,923,523 | 6,397,664 | 5,249.550 us |
| Complete buffers | 40,029 | 13,271,747 | 4,923,523 | 6,014,624 | 5,025.185 us |

The destination removed 5,001 allocation requests and 3,000,000 requested
bytes, but it was about 4.5% slower in this focused timing and increased the
largest allocation observation by about 6.4%. The caller-owned vector's growth
geometry erased the simple “fewer vectors means lower peak” hypothesis.

## ASP.NET A/B

The .NET 10 workbench added the result-heavy fixture to both native and
isolated pools. The initial A/B used 50 base requests in five fresh processes;
the multipliers produced 500, 100, and 50 transforms. Values are medians.

| Items | Boundary | Complete seq | Destination seq | Complete x4 | Destination x4 |
| ---: | --- | ---: | ---: | ---: | ---: |
| 100 | isolated | 4,270/s | 4,343/s | 15,710/s | 16,510/s |
| 100 | native | 12,921/s | 14,140/s | 42,011/s | 40,545/s |
| 1,000 | isolated | 633/s | 689/s | 2,336/s | 2,464/s |
| 1,000 | native | 1,014/s | 1,377/s | 3,624/s | 3,721/s |
| 5,000 | isolated | 146/s | 152/s | 544/s | 555/s |
| 5,000 | native | 159/s | 165/s | 549/s | 578/s |

The 1,000-item native sequential samples were bimodal, and the small gains at
5,000 items overlapped the process-to-process distributions. A longer
three-process comparison increased the largest tier to 200 transforms:

| Boundary | Complete | Destination | Change |
| --- | ---: | ---: | ---: |
| isolated sequential | 150.6/s | 153.7/s | +2.1% |
| isolated x4 | 519.0/s | 551.5/s | +6.3% |
| native sequential | 152.1/s | 147.6/s | -3.0% |
| native x4 | 476.7/s | 485.8/s | +1.9% |

The longer comparison remained mixed and included a native sequential
regression. Together with the larger peak allocation, it does not establish a
clear product tradeoff.

## Disposition

Do not retain direct static-range execution into one caller-owned vector. The
production implementation and test-only switch were removed. Keep the
result-heavy Rust and ASP.NET fixtures because they now provide a reusable
admission gate for a materially different builder design.

This result does not reject append-oriented construction in general. A future
candidate must address destination capacity/peak behavior and should target a
more meaningful ownership boundary than merely deleting the per-iteration
vectors. It still requires complete-path differential tests and host-visible
evidence.
