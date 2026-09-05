# `for-004` Monotonic Child-Path Experiment

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Source checkpoint | `ee659758a867fa6698e6468043f554223f73d15c` plus the uncommitted performance review |
| Status | Complete negative admission experiment; candidate not retained |
| Workload | Pinned XSLT30 `for-004`, with 5, 50, and 500 deterministic `order-item` elements |
| Related review | [Performance optimization review](../Reviews/performance-optimization-review-2026-09-04.md) |

## Question

Does a compile-selected scan for one relative, unpredicated, statically named
child step improve the private exact-decimal plan enough to survive the ASP.NET
host boundary? The candidate avoided the complete evaluator's candidate and
match vectors, order repair, duplicate removal, and final tuple vector. The
complete controlled location-path evaluator remained the differential oracle.

## Prototype and conservation

The safe prototype admitted only one relative `ChildNamed` step without a
position, existence, final, or context predicate. It scanned the prepared
document's retained child slice once, charged every child visit exactly as the
complete evaluator did, selected only the same unnamespaced expanded name, and
fed matching node identities directly to the existing decimal consumer.

A focused mixed-child test covered text, comments, rejected elements, selected
elements, and reversed attribute order. Optimized and complete paths produced
the same value and exact totals in the XPath-operation, XPath-node-visit, and
XDM-string-value domains. No unsafe code, public representation, cache,
cross-invocation state, or host contract was introduced.

## Focused observation

One release-mode allocation observation at 500 selected items reported:

| Path | Allocation requests | Requested bytes | Mean time in the local probe |
| --- | ---: | ---: | ---: |
| Candidate fused child scan | 2 | 32 | 32.266 us |
| Complete path with borrowed attribute strings | 20 | 20,360 | 39.022 us |

The candidate therefore removed 18 transient allocation requests and 20,328
requested bytes, and measured about 17% faster inside that test-only evaluator
loop. This is useful mechanism evidence, but AR-0013 and the performance review
require a consumer-visible gain before retaining an activated path.

## Same-machine ASP.NET A/B

The .NET 10 workbench used the same release build, generated sources, prepared
inputs, compiled stylesheets, and 1,000 base requests per tier. Candidate values
are medians of five fresh processes. The temporary complete-path build was
measured in three fresh processes and then removed. The installed SDK identified
itself as `10.0.100-preview.7.25380.108`; these are local diagnostic values, not
a performance guarantee.

| Tier | Candidate native seq | Complete native seq | Candidate native x4 | Complete native x4 |
| --- | ---: | ---: | ---: | ---: |
| 5 items | 344,485/s | 319,588/s | 1,024,244/s | 1,005,652/s |
| 50 items | 180,263/s | 159,310/s | 538,097/s | 538,126/s |
| 500 items | 16,199/s | 25,490/s | 70,760/s | 69,745/s |

The native sequential 500-item reference was bimodal: two processes reported
25,490-26,872/s and one reported 15,847/s. Candidate processes reported
15,798-19,217/s. Median p50 was 60.6 us for the candidate and 35.8 us for the
three-process reference. The x4 lane differed by about 1.5% at 500 items and
was effectively unchanged at 50 items.

The isolated lanes were likewise inconsistent with a material gain. Candidate
versus complete medians were 20,390 versus 20,911/s, 19,396 versus 19,377/s, and
11,002 versus 10,702/s sequentially; x4 medians were 84,362 versus 84,214/s,
73,940 versus 66,912/s, and 43,534 versus 42,468/s. Process scheduling and the
fixed transport boundary make the isolated variance unsuitable as evidence for
this small engine-local change.

## Disposition

Do not retain the monotonic child-path specialization. It clearly reduces
transient work in the microprobe, but it did not produce a repeatable
consumer-visible benefit and the most relevant native sequential tier became
worse in this A/B. The prototype and its production changes were removed; the
complete controlled location-path evaluator remains the only production path.

This negative result does not reject compile-selected path specialization in
general. It rejects this implementation and workload justification. Continue
with the separately scoped paired-attribute experiment, then measure work-control
traffic only if a retained optimization leaves it material.
