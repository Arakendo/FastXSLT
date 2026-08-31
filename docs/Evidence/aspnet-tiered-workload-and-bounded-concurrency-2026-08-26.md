# ASP.NET Tiered Workload and Bounded Concurrency

| Field       | Value                                                                                                                              |
| ----------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| Date        | 2026-08-26                                                                                                                         |
| Host        | ASP.NET Core targeting .NET 8 on 16 logical processors                                                                             |
| Stylesheet  | Pinned XSLT30 `for-004`; Microsoft lane uses the reviewed XSLT 1.0 equivalent                                                      |
| Tiers       | 5, 50, and 500 deterministic `order-item` elements                                                                                 |
| Concurrency | Sequential and four in-flight transformations                                                                                      |
| Runs        | Five independent host processes after measurement stabilization                                                                    |
| Command     | `./scripts/verify-aspnet-workbench.ps1 -LocalSaxonCs -TieredBenchmark -TieredSummaryOnly -TieredRequests 250 -TieredConcurrency 4` |
| Claim       | Private workload-scaling evidence; not a product benchmark or general engine ranking                                               |

## Lifecycle and measurement

Each tier creates a new source containing `price="1.00" qty="1"` items. The
expected result is therefore the item count formatted with two decimal places.
Source and result sizes were:

| Tier      | Source bytes | Result bytes |
| --------- | -----------: | -----------: |
| 5 items   |          206 |           53 |
| 50 items  |        1,736 |           54 |
| 500 items |       17,036 |           55 |

FastXSLT used a bounded pool of four isolated workers. Every worker received the
source and exact XSLT 2.0 stylesheet once, then retained its own compiled
stylesheet and prepared XDM document. Stable logical request identities were
correlated independently of the worker that completed them.

SaxonCS-HE 13.0.0 used one in-process compiled `XsltExecutable` and prepared
`XdmNode`, creating invocation-local transformers and serializers. Microsoft
used one in-process `XslCompiledTransform` and `XPathDocument` with the XSLT 1.0
equivalent. SaxonCS remained in the gitignored local overlay described by the
earlier comparison evidence.

The largest tier used 250 requests per lane. Request counts were multiplied by
four for 50 items and by twenty for 5 items so the shorter lanes had useful
timing duration. A forced full managed collection occurred between engine lanes
and outside timed regions to reduce cross-engine allocation contamination.
Every timed invocation materialized and verified its result.

## Throughput medians

| Engine path                   | 5 items sequential | 5 items ×4 | 50 items sequential | 50 items ×4 | 500 items sequential | 500 items ×4 |
| ----------------------------- | -----------------: | ---------: | ------------------: | ----------: | -------------------: | -----------: |
| FastXSLT isolated             |           25,966/s |   84,939/s |            17,032/s |    59,702/s |              5,550/s |     22,903/s |
| SaxonCS exact, in-process     |           24,485/s |   91,558/s |            13,738/s |    43,470/s |              1,798/s |      5,950/s |
| Microsoft equivalent XSLT 1.0 |          138,528/s |  456,813/s |            17,352/s |    56,870/s |                264/s |        935/s |

FastXSLT's 500-item ranges were 5,277–5,724/s sequential and
16,752–24,270/s at four-way concurrency. SaxonCS was bimodal at that tier:
1,363–6,408/s sequential and 5,023–23,861/s concurrently. The medians retain
that variability rather than discarding the fast Saxon outlier.

## Latency medians

The values below are medians of each run's measured percentile, in
microseconds. Concurrent latency includes scheduling and worker acquisition.

| Engine/tier                     | Concurrency |     p50 |     p95 |      p99 |
| ------------------------------- | ----------: | ------: | ------: | -------: |
| FastXSLT, 5 items               |           1 |    30.7 |    72.0 |    102.8 |
| FastXSLT, 5 items               |           4 |    40.8 |    61.5 |     94.0 |
| SaxonCS, 5 items                |           1 |    37.3 |    59.1 |     81.8 |
| SaxonCS, 5 items                |           4 |    38.0 |    68.2 |     86.1 |
| FastXSLT, 50 items              |           1 |    50.7 |    93.0 |    129.0 |
| FastXSLT, 50 items              |           4 |    62.1 |    88.8 |    131.9 |
| SaxonCS, 50 items               |           1 |    66.7 |   104.4 |    144.3 |
| SaxonCS, 50 items               |           4 |    78.3 |   142.1 |    189.7 |
| FastXSLT, 500 items             |           1 |   164.8 |   273.8 |    461.1 |
| FastXSLT, 500 items             |           4 |   156.4 |   252.4 |    321.1 |
| SaxonCS, 500 items              |           1 |   539.0 |   691.7 |    829.0 |
| SaxonCS, 500 items              |           4 |   630.8 |   889.4 |  1,075.2 |
| Microsoft equivalent, 500 items |           1 | 3,370.6 | 6,239.1 |  6,816.4 |
| Microsoft equivalent, 500 items |           4 | 4,275.8 | 5,317.7 | 12,911.0 |

## Allocation, CPU, and retained-memory observations

Median approximate managed allocation per transformation was:

| Engine path          |   5 items |   50 items |  500 items | Scope caveat                                                |
| -------------------- | --------: | ---------: | ---------: | ----------------------------------------------------------- |
| FastXSLT isolated    |  ~3.1 KiB |   ~3.1 KiB |   ~3.1 KiB | Managed framing/result path only; excludes Rust allocations |
| SaxonCS in-process   | ~25.6 KiB |  ~57.6 KiB | ~377.6 KiB | Whole managed engine invocation                             |
| Microsoft equivalent | ~20.1 KiB | ~110.5 KiB |  ~8.39 MiB | Dominated at scale by recursive XSLT 1.0 node-set rewriting |

Across the five runs, the aggregate working set of four initialized FastXSLT
workers was about 17.7 MiB at 5 items, 18.2 MiB at 50 items, and 21.2 MiB at
500 items. These are whole-worker process observations, not retained-XDM
measurements. They make the memory multiplication of a process pool visible:
each worker owns a separate compiled/prepared generation.

CPU time was collected from the ASP.NET process and, for FastXSLT, all worker
processes. Windows exposed process CPU in roughly 15.625 ms increments, while
several concurrent lanes completed in tens of milliseconds. Normalized CPU
percentages are therefore recorded by the harness but are too quantized for a
fine ranking. Longer-duration CPU and energy measurements remain required.

Host working-set readings for the in-process engines are also whole-process and
not attributable retained-state measurements. They are retained in raw output
for leak/trend work, but not compared as engine footprints here.

## Interpretation

The fixed-boundary hypothesis survived this experiment, with qualifications.
FastXSLT's isolated path remained near SaxonCS at five items and its median
relative position improved at 50 and 500 items. Four isolated workers provided
3.27×, 3.51×, and 4.13× the corresponding FastXSLT sequential throughput across
the three tiers. That is evidence that independent prepared workers can exploit
bounded host concurrency without changing transformation semantics.

It is not evidence that FastXSLT is generally faster than SaxonCS. FastXSLT is
executing a narrow evaluator admitted specifically for this expression, while
SaxonCS is a broad standards engine; only one stylesheet shape was exercised,
and SaxonCS's large-tier observations were unusually variable.

The Microsoft curve does not compare the same algorithm. XSLT 1.0 cannot express
the original XPath 2.0 `for` expression, and the recursive equivalent repeatedly
constructs decreasing node selections. Its excellent tiny-tier result and poor
500-item result jointly demonstrate why equivalent output is insufficient for
an algorithm-neutral performance comparison.

The experiment excludes cancellation, worker crash/restart, snapshot
replacement, mixed stylesheets, multiple result sizes, cold deployment,
representative application data, and sustained load. AR-0002 remains Proposed.
