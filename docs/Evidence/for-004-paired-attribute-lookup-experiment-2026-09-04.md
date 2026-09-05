# `for-004` Paired-Attribute Lookup Experiment

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Source checkpoint | `ee659758a867fa6698e6468043f554223f73d15c` plus the uncommitted performance review |
| Status | Complete negative admission experiment; candidate not retained |
| Workload | Pinned XSLT30 `for-004`, with 5, 50, and 500 deterministic `order-item` elements |
| Related review | [Performance optimization review](../Reviews/performance-optimization-review-2026-09-04.md) |

## Question

Does resolving the exact-decimal plan's required `price` and `qty` attributes
in one physical scan produce a material consumer-visible gain? The candidate
was private to this already activated expression. It did not add an attribute
index or alter prepared XDM.

## Prototype and accounting decision

The safe candidate scanned each selected element's retained attribute slice
once, recorded the first matching unnamespaced expanded name for each operand,
and stopped when both were present. It handled reversed order, a missing left
attribute, a missing right attribute, and identical requested names. XML parsing
already rejects duplicate expanded-name attributes.

The experiment deliberately charged actual physical attribute visits. It did
not manufacture charges for the eliminated second scan. On the standard
five-item fixture this changed XPath-node-visit consumption from 20 to 15 while
preserving XPath-operation and XDM-string-value totals. A bounded regression
proved the candidate completes inside the reduced physical-visit budget and the
separate two-scan reference consumes the larger total. This was an experiment
accounting choice, not a retained production semantic change.

## Focused observation

One release-mode 500-item observation reported 29.655 us for paired lookup and
31.273 us for the separate borrowed-string reference, about a 5% local
improvement. Both paths performed 20 allocation requests totaling 20,360
requested bytes; the candidate targeted scans and control traffic, not
allocation. It removed 500 of 2,000 total XPath-node visits for this fixture.

## Same-machine ASP.NET A/B

The .NET 10 workbench used the same release build and 1,000 base requests per
tier. Candidate values are medians of five fresh processes. A temporary build
using the separate lookup was measured in three fresh processes and then
removed. The installed SDK was `10.0.100-preview.7.25380.108`.

| Tier | Paired native seq | Separate native seq | Paired native x4 | Separate native x4 |
| --- | ---: | ---: | ---: | ---: |
| 5 items | 322,514/s | 307,514/s | 1,009,958/s | 797,639/s |
| 50 items | 162,965/s | 158,989/s | 503,157/s | 502,689/s |
| 500 items | 26,140/s | 26,157/s | 88,963/s | 104,892/s |

At the most relevant 500-item sequential tier, median throughput differed by
less than 0.1% and median p50 was 35.4 us paired versus 35.3 us separate. The
paired x4 median was about 15% lower. The apparent small-tier x4 win was not
consistent across tiers and therefore is not admission evidence.

Isolated candidate versus reference medians were 19,764 versus 19,674/s,
18,444 versus 17,757/s, and 10,869 versus 11,393/s sequentially. The x4 medians
were 75,628 versus 70,108/s, 68,845 versus 69,838/s, and 40,391 versus
40,890/s. These small and contradictory movements are consistent with process
and transport variance, not a durable host-visible improvement.

## Disposition

Do not retain the paired-attribute lookup. It removes real redundant work and
slightly improves its focused loop, but the consumer boundary showed no gain at
the dominant 500-item sequential tier and a regression under four native
handles. The complete separate lookup remains the production implementation.

Both measured P1 candidates from the review are now closed without retention.
The next justified action is production-shaped attribution of useful XPath work
versus control traffic. It should measure before changing budget or cancellation
mechanics; the test-only nanosecond charge probe is not enough.
