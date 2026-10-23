# AR-0021 Batch, Worker, and Resource Sweep

| Field             | Value                                                                                                        |
| ----------------- | ------------------------------------------------------------------------------------------------------------ |
| Date              | 2026-09-05                                                                                                   |
| Source checkpoint | Working tree above `10d2f30`                                                                                 |
| Host              | AMD Ryzen 7 7800X3D, 16 logical processors, Windows `10.0.26200.0`                                           |
| Runtime           | .NET 10 preview workbench; release Rust worker                                                               |
| Workload          | Pinned XSLT30 `for-004`; generated 5/50/500-item sources                                                     |
| Members per cell  | 8,192                                                                                                        |
| Matrix            | 1/2/4/8 persistent workers crossed with batch 1/8/32/128                                                     |
| Claim             | Exploratory operational-economics evidence; not a default batch size, worker count, or performance guarantee |

## Method

Each worker retained its own compiled stylesheet and prepared source. Worker
creation and warm-up occurred outside the timed interval. Transactions were
distributed round-robin, while every worker retained exactly one sequential
execution lane. Every correlated result was compared exactly.

The harness records throughput, process CPU delta, aggregate observed working
set before/after, managed allocation, exact request/response wire bytes,
maximum outstanding members, ambiguity radius for one worker loss, and the
hypothetical aggregate radius if every worker were lost simultaneously.

Command:

```powershell
./scripts/verify-aspnet-workbench.ps1 `
  -TargetFramework net10.0 `
  -IsolatedBatchWorkerSweep `
  -BatchSweepMembers 8192 `
  -TieredSummaryOnly `
  -MeasurementRuns 1 `
  -MeasurementRequests 1
```

## Throughput observation

Transforms per second from the first full matrix:

| Tier | Batch | 1 worker | 2 workers | 4 workers | 8 workers |
| ---- | ----: | -------: | --------: | --------: | --------: |
| 5    |     1 |   14,972 |    37,789 |    74,199 |    81,733 |
| 5    |     8 |   58,392 |   105,125 |   184,674 |   328,640 |
| 5    |    32 |   81,907 |   130,320 |   232,764 |   619,452 |
| 5    |   128 |   92,505 |   180,895 |   281,038 |   626,103 |
| 50   |     1 |   22,034 |    48,910 |    70,119 |   147,128 |
| 50   |     8 |   62,578 |   126,566 |   203,169 |   201,900 |
| 50   |    32 |   79,581 |   162,079 |   241,011 |   295,494 |
| 50   |   128 |   90,950 |   171,825 |   251,649 |   316,268 |
| 500  |     1 |   12,434 |    27,073 |    45,283 |    49,139 |
| 500  |     8 |   21,975 |    43,324 |    80,208 |   132,063 |
| 500  |    32 |   24,140 |    49,537 |   100,674 |   100,442 |
| 500  |   128 |   26,813 |    53,218 |    86,827 |   137,254 |

Batching and process count compound materially, but the surface is not
monotonic. Four workers at 500 items favored batch 32 over 128 in this run;
eight workers made batch 8 and 128 close while batch 32 fell behind. The lanes
were measured in fixed size order and only once, so scheduling and thermal
variance remain plausible. No batch-size rule follows from this table.

## Failure radius and exact wire pressure

The maximum ambiguity radius caused by losing one worker is the active batch
size, independent of total worker count. The hypothetical simultaneous-loss
radius is batch size multiplied by worker count:

| Batch | One worker loss | 1/2/4/8-worker aggregate radius |
| ----: | --------------: | ------------------------------: |
|     1 |               1 |                   1 / 2 / 4 / 8 |
|     8 |               8 |                8 / 16 / 32 / 64 |
|    32 |              32 |             32 / 64 / 128 / 256 |
|   128 |             128 |         128 / 256 / 512 / 1,024 |

For these tiny results, maximum response frames remained small:

| Batch |  5 items | 50 items | 500 items |
| ----: | -------: | -------: | --------: |
|     1 |     85 B |     87 B |      89 B |
|     8 |    645 B |    661 B |     677 B |
|    32 |  2,597 B |  2,661 B |   2,725 B |
|   128 | 10,501 B | 10,757 B |  11,013 B |

Those byte counts do not represent a result-heavy transform. The existing 1
MiB response ceiling, not member count alone, remains the real safety bound.

Managed allocation generally fell from roughly 3.0-3.2 KiB per batch-of-one
member toward 1.67-1.70 KiB at batch 128. One 8-worker/50-item pair reported
large process-wide managed-allocation outliers; as in earlier evidence,
`GC.GetTotalAllocatedBytes` includes unrelated ASP.NET activity.

Aggregate observed working set scaled approximately with process count. At
batch 128 it was about 5.9/11.7/23.2/46.8 MiB for the 5-item source,
5.9/11.8/23.6/47.3 MiB for 50 items, and 6.6/13.2/26.3/52.7 MiB for 500 items.
These are boundary observations, not exact ownership or peak-memory accounts.

## CPU measurement limitation

The initial 1,024-member run exposed zero process-CPU deltas in many cells.
Increasing to 8,192 members made the 500-item cells mostly observable but still
left several tiny batched cells below Windows' roughly 15.625 ms process-time
quantum. Effective-core values for those cells are therefore unusable. A later
CPU tranche must lengthen each measured cell independently or obtain a finer
worker-local CPU counter. Zero in this report means unobservable at this timer
resolution, not zero CPU cost.

## Rotated long-window follow-up

A second run increased every cell to 65,536 members and repeated the complete
matrix three times with batch order rotated between repetitions:

```powershell
./scripts/verify-aspnet-workbench.ps1 `
  -TargetFramework net10.0 `
  -IsolatedBatchWorkerSweep `
  -BatchSweepMembers 65536 `
  -TieredSummaryOnly `
  -MeasurementRuns 3 `
  -MeasurementRequests 1
```

Median transforms per second were:

| Tier | Workers | Batch 1 | Batch 8 | Batch 32 | Batch 128 |
| ---- | ------: | ------: | ------: | -------: | --------: |
| 5    |       1 |  22,394 |  73,267 |   97,696 |   114,377 |
| 5    |       2 |  49,951 | 141,623 |  192,587 |   217,732 |
| 5    |       4 |  95,256 | 265,894 |  368,735 |   413,846 |
| 5    |       8 | 174,050 | 450,648 |  583,267 |   557,302 |
| 50   |       1 |  20,480 |  56,231 |   73,075 |    81,034 |
| 50   |       2 |  45,674 | 114,243 |  140,563 |   157,313 |
| 50   |       4 |  89,044 | 218,562 |  294,058 |   295,756 |
| 50   |       8 |  98,855 | 373,617 |  408,694 |   478,242 |
| 500  |       1 |  11,449 |  20,933 |   23,498 |    24,270 |
| 500  |       2 |  22,417 |  39,006 |   48,929 |    40,220 |
| 500  |       4 |  50,481 |  87,600 |   93,019 |    92,182 |
| 500  |       8 |  51,062 |  86,719 |   94,483 |    93,898 |

The longer run confirms there is no monotonic worker-count/batch-size rule.
Batch 128 remains strongest in most tiny and medium cells, but batch 32 has the
higher 8-worker tiny median and is the strongest 2/4/8-worker choice at 500
items. At 500 items, moving from batch 32 to 128 therefore increases the
single-worker-loss ambiguity radius from 32 to 128 without improving median
throughput at those worker counts.

The repeat ranges also reject a premature automatic selector. For example,
8-worker/5-item throughput ranged from 486,213 to 809,502/s at batch 32 and
533,803 to 722,518/s at batch 128. The corresponding 8-worker/50-item ranges
were 355,103-420,702/s and 341,596-550,087/s. A short observation window could
choose differently from the median on the same machine.

The longer windows made aggregate worker CPU observable in every cell. Median
effective busy-core estimates ranged from about 0.28 to 3.90. At 500 items,
four-worker batch 32 used about 3.19 effective worker cores and delivered
93,019/s; eight-worker batch 32 used about 3.78 and delivered 94,483/s. This
plateau nominates host/transport coordination or machine contention for later
attribution; it does not establish an evaluator bottleneck, because the metric
excludes ASP.NET host CPU and is derived from process-level samples.

Median observed working set and managed allocation retained the first sweep's
shape: memory stayed approximately process-count proportional, while batching
reduced managed allocation per member from roughly 3.0-3.2 KiB toward 1.68-1.70
KiB. These remain boundary observations rather than total ownership accounts.

## Disposition

The sweeps confirm that batching remains attractive across multiple workers
and make its failure-radius cost explicit. They do not nominate one batch size:
smaller batches retain much of the gain, cap ambiguity more tightly, and win
under several worker/workload combinations. The rotated long-window run closes
the immediate CPU-timer gap but exposes material throughput variance and a
four-to-eight-worker plateau on large work. A result-heavy workload is required
before the response-byte ceiling or incremental delivery can be judged.
