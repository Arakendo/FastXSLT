# AR-0021 Result-Pressure Batch Sweep

| Field             | Value                                                                                                   |
| ----------------- | ------------------------------------------------------------------------------------------------------- |
| Date              | 2026-09-05                                                                                              |
| Source checkpoint | Working tree above `10d2f30`                                                                            |
| Host              | AMD Ryzen 7 7800X3D, 16 logical processors, Windows `10.0.26200.0`                                      |
| Runtime           | .NET 10 preview workbench; release Rust worker                                                          |
| Workload          | Existing result-heavy 100/1,000/5,000-element fixtures                                                  |
| Members per cell  | 256                                                                                                     |
| Repetitions       | Three, with candidate batch order rotated                                                               |
| Claim             | Result-transfer and aggregate-framing pressure; not a default batch policy or general performance claim |

## Question and method

The first AR-0021 sweep returned only 85-11,013 bytes per response frame. This
follow-up asks whether transport batching remains useful when each transform
returns 3,311, 33,011, or 165,011 UTF-8 bytes.

Every worker retains one compiled stylesheet and prepared source and executes
one sequential lane. Each correlated result is compared exactly. Candidate
batch sizes shrink as result size grows so the calculated response remains
below the private 1 MiB aggregate-response ceiling:

|    Result | Candidate batches | Largest response frame |
| --------: | ----------------- | ---------------------: |
|   3,311 B | 1 / 8 / 32 / 128  |              428,549 B |
|  33,011 B | 1 / 8 / 16        |              528,773 B |
| 165,011 B | 1 / 2 / 4         |              660,193 B |

Command:

```powershell
./scripts/verify-aspnet-workbench.ps1 `
  -TargetFramework net10.0 `
  -IsolatedBatchResultPressure `
  -BatchResultMembers 256 `
  -TieredSummaryOnly `
  -MeasurementRuns 3 `
  -MeasurementRequests 1
```

## Throughput

Median transforms per second were:

|    Result | Workers | Batch 1 | Batch 2 | Batch 4 | Batch 8 | Batch 16 | Batch 32 | Batch 128 |
| --------: | ------: | ------: | ------: | ------: | ------: | -------: | -------: | --------: |
|   3,311 B |       1 |   5,206 |       - |       - |   6,643 |        - |    7,928 |    10,029 |
|   3,311 B |       4 |  20,025 |       - |       - |  22,196 |        - |   24,097 |    14,140 |
|   3,311 B |       8 |  18,455 |       - |       - |  26,752 |        - |   30,322 |    14,865 |
|  33,011 B |       1 |     757 |       - |       - |     820 |      861 |        - |         - |
|  33,011 B |       4 |   2,789 |       - |       - |   2,789 |    2,803 |        - |         - |
|  33,011 B |       8 |   2,683 |       - |       - |   2,846 |    2,880 |        - |         - |
| 165,011 B |       1 |     174 |     166 |     175 |       - |        - |        - |         - |
| 165,011 B |       4 |     585 |     594 |     575 |       - |        - |        - |         - |
| 165,011 B |       8 |     578 |     590 |     563 |       - |        - |        - |         - |

Batching remains materially useful for the 3.3 KiB outcome, but batch 128 is
counterproductive once four or eight workers compete. Batch 32 is the observed
multi-worker peak. At 33 KiB, batching produces little multi-worker throughput
change. At 165 KiB, no batch size has a repeatable material advantage over one
member per transaction.

Four workers are also effectively sufficient for the two largest results in
this workload. Eight workers add no median throughput and sometimes regress it,
which is consistent with result construction, memory traffic, host receipt,
or copying becoming shared-machine pressure. This observation does not isolate
which component dominates.

## Aggregate-response latency and retention

Because the current protocol emits one aggregate input-order response, time to
first result equals time to final result for every transaction. Median p50 grew
approximately with batch size:

|    Result | Workers |  Batch 1 |   Largest tested batch |
| --------: | ------: | -------: | ---------------------: |
|   3,311 B |       4 | 0.181 ms | 17.440 ms at batch 128 |
|   3,311 B |       8 | 0.231 ms | 15.704 ms at batch 128 |
|  33,011 B |       4 | 1.367 ms |  21.716 ms at batch 16 |
|  33,011 B |       8 | 1.660 ms |  25.928 ms at batch 16 |
| 165,011 B |       4 | 6.558 ms |   27.062 ms at batch 4 |
| 165,011 B |       8 | 7.913 ms |   30.234 ms at batch 4 |

The 8-worker tails were noisy: 33 KiB batch 16 reported a 68.942 ms median
per-run p95/p99, while 165 KiB batch 4 reported 288.595 ms p99. These short
256-member cells do not establish stable tail percentiles, but they demonstrate
that aggregate batching can withhold a large completed set behind one slow
transaction.

Managed allocation per member was dominated by outcome materialization rather
than framing: roughly 11.5-13.1 KiB for a 3.3 KiB result, 100.7-102.2 KiB for a
33 KiB result, and 497-498 KiB for a 165 KiB result. Batching removes only a
small fraction of that cost. Observed worker working set remained broadly
process-count proportional, though allocator high-water and collection timing
make it an observation rather than exact retained ownership.

## Failure-radius consequence

The response-byte ceiling naturally restricts ambiguity radius as outputs grow,
but it does not eliminate the tradeoff. A 660 KiB four-member frame can leave
four attempts ambiguous after one worker loss (32 across eight simultaneous
worker losses) without providing a throughput win in this workload.

## Disposition

Transport batching is a workload-sensitive optimization for fixed boundary
cost, not a default treatment for every transform set. Result size and semantic
cost can consume the opportunity before the response ceiling is reached.
Retain aggregate framing as the correctness oracle. Incremental input-order
delivery now has concrete latency/retention pressure to study, but must earn
admission against its additional frames and protocol states; it is not required
to recover throughput for the largest result fixture.
