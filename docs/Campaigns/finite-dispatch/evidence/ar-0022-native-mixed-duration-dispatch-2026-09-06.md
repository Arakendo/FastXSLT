# AR-0022 Native Mixed-Duration Dispatch

| Field | Value |
| --- | --- |
| Date | 2026-09-06 |
| Runtime | .NET 10 preview workbench |
| Engine | FastXSLT native in-process |
| Stylesheet | Unchanged W3C `for-004.xsl` |
| Workers | Eight dedicated threads; one native handle per worker and source tier |
| Set | 4,096 independent transforms containing equal 5-, 50-, and 500-item populations |
| Orders | Interleaved `5/50/500`; clustered `5.../50.../500...` |
| Candidates | Static balanced assignment; completion-driven claims 1/2/4/8 |
| Samples | Three fresh processes, three alternating-order rounds per process |
| Result scope | Duration and ordering pressure only; every result remains small |

## Method

Every worker owns an independently compiled/prepared native handle for each
source tier. Each workload contains the same tier counts and expected results;
only submission order changes. The timed region performs no compilation or
source preparation. Every outcome is checked against its tier-specific exact
result.

The dispatcher uses one start gate for the complete set and no wave barrier.
Static assignment gives each worker one contiguous job range. Dynamic workers
claim their next range only after completing the previous range. Queue claim
size remains distinct from isolated transport batching, which is absent here.

## Cross-process throughput

Each fresh process contributes the median of three alternating-order rounds.
The table reports the median of those three process medians.

| Order | Static | Claim 1 | Claim 2 | Claim 4 | Claim 8 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Interleaved | 314,571/s | 378,912/s | **409,825/s** | 349,315/s | 379,716/s |
| Clustered | 219,111/s | **377,362/s** | 374,766/s | 377,115/s | 377,950/s |

Claim two's interleaved median is about 8.2% above claim one, but the direction
does not survive every fresh process: its process medians were about 458k,
410k, and 350k/s versus claim one's 402k, 379k, and 370k/s. Claims one through
eight are effectively tied on clustered throughput at this measurement
quality. No larger claim produces a repeatable cross-order win.

Static contiguous assignment is particularly poor for clustered input because
later workers receive disproportionate 500-item work. Its cross-process median
falls to about 219k/s and sampled busy fraction is roughly 0.41-0.44, while the
dynamic candidates remain near 0.98. Dynamic completion-driven refill, rather
than a larger local claim, supplies the material scheduling gain.

## Tail and acquisition behavior

Across fresh-process median rounds, claim one's final tail is approximately
41-58 microseconds for interleaved work and 34-56 microseconds for clustered
work. Claim two is approximately 61-68 and 92-116 microseconds respectively.
Claims four and eight continue the general increase in final claim radius,
reaching clustered medians as high as roughly 342-441 microseconds for claim
eight.

Larger claims reduce acquisition count deterministically: 4,096, 2,048, 1,024,
and 512 acquisitions for claims one, two, four, and eight. As in the uniform
tranche, fewer acquisitions do not establish a consumer-visible win.

## Interpretation

The mixed-duration result strengthens completion-driven claim one as the
private reference:

- dynamic refill removes severe clustered static imbalance;
- claim one keeps aggregate measured busy time near full utilization;
- claim two's interleaved lead is noisy and does not survive every process;
- larger claims increase tail/ownership radius without a stable throughput
  benefit; and
- job ordering materially changes static balance, but does not establish a
  robust larger-claim rule.

Together with the uniform tranche, this was sufficient to close AR-0022 as No
Change: the proposed larger worker grab did not earn admission. Result sizes
remain uniformly small, and the probe did not exercise cancellation,
generation drain, worker loss, isolated transport coupling, or another machine.
Those experiments are dormant reopening work rather than prerequisites for the
no-change conclusion.

## Reproduction

```powershell
./scripts/verify-aspnet-workbench.ps1 `
  -TargetFramework net10.0 `
  -TieredOnly `
  -FiniteDispatchBenchmark `
  -FiniteDispatchMixed `
  -FiniteDispatchSummaryOnly `
  -FiniteDispatchRounds 3
```

Run the command in three fresh processes on distinct ports. Treat each emitted
row as that process's median and preserve its min/max range as variance context.
