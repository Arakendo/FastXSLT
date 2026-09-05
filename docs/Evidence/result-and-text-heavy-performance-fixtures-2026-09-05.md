# Result-Heavy and Text-Heavy Performance Fixtures

| Field | Value |
| --- | --- |
| Date | 2026-09-05 |
| Source checkpoint | `69b731b0` plus the performance follow-up described here |
| Status | Text serializer optimization retained; result-builder pressure established but not optimized |
| Related review | [Performance optimization review](../Reviews/performance-optimization-review-2026-09-04.md) |
| Governing review | [AR-0013](../Architectural%20Reviews/AR-0013-prepared-representation-and-data-layout-audit.md) |

## Questions

The `for-004` result is too small to justify result-construction or serializer
work. Two ignored release probes therefore execute actual compiled stylesheets
through the ordinary prepared-input, semantic-result, and serializer paths:

- a result-heavy stylesheet constructs 100, 1,000, or 5,000 elements, each
  with an attribute and text child; and
- a text-heavy stylesheet constructs one element containing 4 KiB, 64 KiB, or
  512 KiB of XML-safe UTF-8 text.

The text fixture is also exposed by the .NET 10 ASP.NET workbench for both the
persistent isolated-worker and native in-process boundaries. It is a focused
optimization workload, not a general engine benchmark or a standards claim.

## Result-construction pressure

The allocation-observed release probe reported the following single-run
medians of five timing samples. Allocation counts and bytes are one retained
semantic execution observed by `allocation_counter`.

| Constructed items | Result bytes | Execution | Serialization | Execution allocations | Requested bytes | Peak live bytes |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 100 | 3,311 | 48.259 us | 15.169 us | 816 | 247,803 | 113,880 |
| 1,000 | 33,011 | 485.745 us | 144.593 us | 8,022 | 2,359,083 | 1,092,360 |
| 5,000 | 165,011 | 2,428.378 us | 788.140 us | 40,028 | 13,271,723 | 6,014,600 |

The roughly eight allocation requests per constructed item and linear retained
result growth establish real pressure for the review's append-oriented builder
experiment. They do not establish that a builder will improve host-visible
throughput. No result-construction implementation was changed in this tranche.

## Bounded safe-text runs

The complete serializer writes ordinary text one Unicode scalar at a time. The
safe prototype scans for escaping boundaries and submits XML-safe UTF-8 in
chunks no larger than 4,096 bytes. It is selected only when no character map or
Unicode normalization is active. Entity replacements and HTML5 C1 references
retain the existing path; normalization and character maps retain the complete
character-wise implementation.

The bounded path charges exact serialized bytes before each mutation and
checks cancellation at every chunk. If a byte limit falls within a chunk, it
writes only the complete-character prefix that the reference path would admit,
then attempts the same next character. Focused differential tests conserve the
result, structured byte-limit failure, partial output, charged-byte total, and
cancellation-before-next-mutation behavior, including an escape boundary and a
multibyte scalar.

One same-machine release probe before and after the implementation reported:

| Safe text bytes | Reference serialization | Bounded-run serialization | Change | Reference allocations/bytes | Bounded allocations/bytes |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 4,096 | 27.618 us | 6.248 us | -77.4% | 12 / 16,391 | 4 / 12,326 |
| 65,536 | 476.481 us | 135.049 us | -71.7% | 16 / 262,152 | 7 / 127,155 |
| 524,288 | 3,464.225 us | 955.570 us | -72.4% | 19 / 2,097,161 | 10 / 1,045,780 |

These are diagnostic local values rather than a stable performance contract.

## ASP.NET same-machine A/B

The .NET 10 workbench used SDK `10.0.100-preview.7.25380.108`, four independent
engines/workers, and 50 base requests. The request multipliers produced 1,000,
200, and 50 invocations for the three tiers. Each side of the A/B was measured
in five fresh ASP.NET processes. The reference build temporarily routed the
same workload through the complete character-wise serializer; that dispatch
change was then removed. Values below are medians of the five process results.

| Payload | Boundary | Reference seq | Bounded seq | Ratio | Reference x4 | Bounded x4 | Ratio |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 4 KiB | isolated | 9,276/s | 14,180/s | 1.53x | 30,882/s | 41,017/s | 1.33x |
| 4 KiB | native | 27,825/s | 104,206/s | 3.75x | 67,777/s | 194,284/s | 2.87x |
| 64 KiB | isolated | 1,504/s | 3,166/s | 2.10x | 5,661/s | 12,320/s | 2.18x |
| 64 KiB | native | 1,757/s | 5,943/s | 3.38x | 5,772/s | 20,195/s | 3.50x |
| 512 KiB | isolated | 229/s | 631/s | 2.75x | 772/s | 1,808/s | 2.34x |
| 512 KiB | native | 208/s | 766/s | 3.68x | 634/s | 1,800/s | 2.84x |

The bounded implementation's median latency percentiles were:

| Payload | Boundary/concurrency | p50 | p95 | p99 |
| --- | --- | ---: | ---: | ---: |
| 4 KiB | isolated seq | 60.1 us | 105.2 us | 148.5 us |
| 4 KiB | isolated x4 | 64.7 us | 97.0 us | 189.7 us |
| 4 KiB | native seq | 8.0 us | 10.6 us | 13.8 us |
| 4 KiB | native x4 | 10.3 us | 20.5 us | 42.8 us |
| 64 KiB | isolated seq | 272.7 us | 395.3 us | 533.0 us |
| 64 KiB | isolated x4 | 266.2 us | 388.9 us | 1,747.4 us |
| 64 KiB | native seq | 119.8 us | 155.9 us | 396.2 us |
| 64 KiB | native x4 | 142.5 us | 248.1 us | 1,678.7 us |
| 512 KiB | isolated seq | 1,365.0 us | 1,900.2 us | 5,707.4 us |
| 512 KiB | isolated x4 | 1,585.6 us | 6,849.7 us | 7,766.0 us |
| 512 KiB | native seq | 953.4 us | 1,305.6 us | 7,217.9 us |
| 512 KiB | native x4 | 1,561.6 us | 6,672.5 us | 9,152.6 us |

Managed allocation per invocation did not materially distinguish the A/B
because result transfer and managed string creation dominate that observation;
the Rust allocation probe supplies the engine-local allocation evidence. Tail
latencies under x4 are noisy, but every throughput lane retained a material
median gain.

## Disposition

Retain the private safe bounded-run serializer. It has a complete semantic
oracle, focused limit/cancellation conservation, material engine-local savings,
and repeatable host-visible gains through both supported experimental hosting
boundaries. This does not admit direct serialization from instructions, unsafe
code, a public serializer contract, unbounded cancellation latency, stack-buffer
numeric formatting, or a capacity-estimation policy.

The first narrow append experiment executed a static integer range directly
into one caller-owned vector. It removed 5,001 allocations at 5,000 items but
increased the largest allocation observation and produced mixed host results,
so it was removed. The fixture remains the admission gate for a materially
different result-builder design.

[Negative result-destination experiment](static-range-result-destination-experiment-2026-09-05.md)
