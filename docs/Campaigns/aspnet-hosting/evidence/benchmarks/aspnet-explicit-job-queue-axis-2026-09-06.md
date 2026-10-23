# ASP.NET Explicit Job-Queue Axis Comparison

| Field             | Value                                                                                            |
| ----------------- | ------------------------------------------------------------------------------------------------ |
| Date              | 2026-09-06                                                                                       |
| Runtime           | .NET 10 preview workbench                                                                        |
| Workload          | Exact `for-004` generated fixture family at 5, 8, 50, and 500 items per transform                |
| Job queues        | 500 and 5,000 independent transforms per measured lane                                           |
| Concurrency       | Separate 1-, 4-, and 8-worker runs                                                               |
| Samples           | Three fresh processes and three seeded rounds per process                                        |
| FastXSLT isolated | Incremental non-retaining transport; private batches 1/8/32/128                                  |
| Saxon             | Local non-distributed SaxonCS-HE 13.0.0 `TextWriter` lane                                        |
| Microsoft         | Shared linear sibling-walk `XslCompiledTransform` XSLT 1.0 equivalent                            |
| Status            | Exploratory queue-scaling evidence; not publication-eligible or representative consumer evidence |

## Purpose and axes

The earlier best-practice benchmark varied request count by source tier to keep
measurement durations useful. That made `members` visible but did not provide a
clean job-queue axis. This follow-up holds the independent transform count
constant across every source tier and reports four orthogonal dimensions:

```text
items per transform       = semantic work inside one transform
queued jobs               = independent transforms waiting to drain
worker concurrency        = simultaneous native handles or worker processes
transport batch members   = isolated requests sharing one IPC transaction
```

Compilation, source preparation, and worker startup remain outside timing.
Every result is validated. The test therefore measures a prepared, warm engine
queue, not file acquisition, XML preparation, database access, publication, or
the complete field application pipeline.

## Cross-process median throughput

The isolated column reports the highest median among private batch sizes for
that cell and names the batch. This is an observation, not a default selection.

| Items |  Jobs | Workers |    Isolated best |      Native | Microsoft |     Saxon |
| ----: | ----: | ------: | ---------------: | ----------: | --------: | --------: |
|     5 |   500 |       1 |   55,563/s (b32) |   300,716/s | 149,544/s |  27,104/s |
|     5 |   500 |       4 |  188,715/s (b32) |   654,022/s | 330,579/s | 107,643/s |
|     5 |   500 |       8 |  237,891/s (b32) |   810,110/s | 584,590/s | 112,377/s |
|     5 | 5,000 |       1 |  88,273/s (b128) |   339,799/s | 193,681/s | 116,403/s |
|     5 | 5,000 |       4 |  315,286/s (b32) |   978,857/s | 508,668/s | 308,644/s |
|     5 | 5,000 |       8 | 391,567/s (b128) | 1,052,654/s | 415,286/s | 272,081/s |
|    50 |   500 |       1 |  67,322/s (b128) |   138,148/s |  42,295/s |   9,713/s |
|    50 |   500 |       4 | 193,603/s (b128) |   460,066/s |  76,011/s |  24,750/s |
|    50 |   500 |       8 |  204,432/s (b32) |   606,649/s | 139,989/s |  58,053/s |
|    50 | 5,000 |       1 | 101,772/s (b128) |   177,795/s |  59,340/s |  49,394/s |
|    50 | 5,000 |       4 | 324,610/s (b128) |   491,487/s | 157,913/s | 136,643/s |
|    50 | 5,000 |       8 | 358,431/s (b128) |   840,732/s | 326,064/s | 238,484/s |
|   500 |   500 |       1 |  28,591/s (b128) |    19,895/s |   6,205/s |   1,424/s |
|   500 |   500 |       4 |  76,537/s (b128) |    92,644/s |  10,155/s |   3,227/s |
|   500 |   500 |       8 |    92,926/s (b8) |   123,044/s |  26,089/s |   9,428/s |
|   500 | 5,000 |       1 |  29,693/s (b128) |    29,928/s |   9,158/s |   6,515/s |
|   500 | 5,000 |       4 |   98,064/s (b32) |    78,056/s |  25,392/s |  19,681/s |
|   500 | 5,000 |       8 | 111,189/s (b128) |   158,001/s |  45,967/s |  36,224/s |

The 8-item diagnostic is retained in the machine-readable runner output but is
omitted from the main table because it was introduced to test tiny-work
occupancy rather than define another product tier.

## Five-thousand-job drain time at eight workers

Derived drain time is `5,000 / throughput`; it is not a separately timed phase.

| Items per transform | Isolated best |   Native | Microsoft |     Saxon |
| ------------------: | ------------: | -------: | --------: | --------: |
|                   5 |      12.77 ms |  4.75 ms |  12.04 ms |  18.38 ms |
|                  50 |      13.95 ms |  5.95 ms |  15.33 ms |  20.97 ms |
|                 500 |      44.97 ms | 31.65 ms | 108.77 ms | 138.03 ms |

## Interpretation

Increasing the explicit queue from 500 to 5,000 jobs does not produce a
length-driven throughput collapse in this warm generated workload. Most cells
report higher throughput with 5,000 jobs because pool fill, drain, JIT, timer,
and other fixed costs occupy a smaller share of the longer interval. A few
short/noisy in-process cells reverse direction, reinforcing that the 500-job
rows are not steady-state publication evidence.

For the 5,000-job anchor at eight workers, native FastXSLT drains all three main
tiers fastest. The best isolated lane is in the same region as Microsoft on the
five-item tier and leads Microsoft and Saxon on the 50- and 500-item tiers.
Those are exact generated-workload observations, not broad processor rankings.

More importantly for the motivating field report, this probe does **not**
explain an application taking a long time to process 5,000 XML documents. Its
timed path reuses one compiled stylesheet and one prepared input and repeatedly
executes that immutable state. A real queue may pay different-source parsing,
XDM preparation, resource resolution, parameters, larger results, database
reads/writes, result publication, diagnostics, or a materially different
stylesheet. The next useful application investigation is phase attribution on
the representative 5,000-document distribution, not another synthetic queue-
length optimization.

## Reproduction

```powershell
./scripts/measure-job-queue-deployment.ps1 `
  -Samples 3 `
  -WithinProcessRounds 3 `
  -TargetFramework net10.0 `
  -QueuedJobs 500,5000 `
  -Concurrency 1,4,8 `
  -LocalSaxonCs `
  -CompactSummary
```

The Saxon installation remains gitignored and is not distributed with
FastXSLT.
