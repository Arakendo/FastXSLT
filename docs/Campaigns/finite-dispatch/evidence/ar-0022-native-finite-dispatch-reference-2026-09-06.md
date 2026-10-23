# AR-0022 Native Finite-Dispatch Reference

| Field | Value |
| --- | --- |
| Date | 2026-09-06 |
| Runtime | .NET 10 preview workbench |
| Engine | FastXSLT native in-process |
| Source/style | Unchanged W3C `for03.xml` and `for-004.xsl` |
| Workers | Eight dedicated threads, each owning one native handle |
| Finite sets | 8, 32, 128, 500, 4,096, and 50,000 independent transforms |
| Candidates | Static balanced assignment; completion-driven terminal-refill claims 1/2/4/8 |
| Samples | Three fresh processes, three alternating-order rounds per process |
| Status | Exploratory native reference; not a dispatcher or claim-size selection |

## Method

Every candidate receives the same prebuilt request identities and exact semantic
oracle. One start gate releases the entire finite set; there is no per-wave or
per-claim completion barrier. A completion-driven worker atomically claims its
next bounded range as soon as its previous range finishes. A static worker owns
one balanced contiguous range for the whole set.

The probe records active and participating worker high-water, completed jobs per
worker, queue acquisitions, fill time, time between the first and last worker's
final completion, aggregate transform-call busy fraction, and total drain
throughput. Instrumentation is identical within a candidate family, but this is
short-duration work without publication-grade convergence.

## Representative throughput

Each fresh process contributes its median round. Values below are the median of
three process medians.

| Jobs | Static balanced | Claim 1 | Claim 2 | Claim 4 | Claim 8 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 500 | 1,102,293/s | **1,285,017/s** | 1,035,840/s | 1,193,887/s | 1,202,212/s |
| 4,096 | 1,209,080/s | 1,314,675/s | **1,319,970/s** | 1,163,702/s | 1,226,090/s |
| 50,000 | **1,274,473/s** | 1,218,291/s | 1,249,866/s | 1,127,251/s | 1,169,446/s |

Claim one reaches about 1.285 million transforms per second for 500 actual
independent jobs, approximately 95% of the earlier 1.36-million/s continuously
supplied deployment observation. At 4,096 jobs, claims one and two are nearly
tied. At 50,000 jobs the static candidate leads this sample, while process
medians remain materially variable. No claim size wins across these finite-set
sizes.

## Utilization and tail drain

| Jobs | Candidate | Busy fraction | Tail drain | Queue acquisitions |
| ---: | --- | ---: | ---: | ---: |
| 500 | Static | 81.2% | 98.2 us | 0 |
| 500 | Claim 1 | 89.2% | 4.1 us | 500 |
| 500 | Claim 4 | 89.9% | 15.5 us | 125 |
| 500 | Claim 8 | 88.7% | 28.3 us | 63 |
| 4,096 | Static | 93.4% | 309.2 us | 0 |
| 4,096 | Claim 1 | 96.1% | 4.5 us | 4,096 |
| 4,096 | Claim 4 | 97.6% | 13.8 us | 1,024 |
| 4,096 | Claim 8 | 97.2% | 37.7 us | 512 |
| 50,000 | Static | 95.0% | 2,853.5 us | 0 |
| 50,000 | Claim 1 | 97.9% | 6.3 us | 50,000 |
| 50,000 | Claim 4 | 98.6% | 19.2 us | 12,500 |
| 50,000 | Claim 8 | 98.7% | 32.3 us | 6,250 |

Larger claims reduce acquisition count exactly as expected, but that reduction
does not produce a repeatable throughput benefit. Claim one already keeps the
longer finite sets near full measured busy time and consistently minimizes
dynamic tail drain. The atomic claim operation is not a demonstrated material
bottleneck in this native uniform-work tranche.

Small sets expose claim hoarding directly. With eight jobs, claim eight admits
only one participating worker and active high-water one; claim four can use at
most two workers, and claim two at most four. Claim one can distribute all eight
jobs but individual transforms are short enough that OS scheduling sometimes
finishes work before every thread becomes active. Throughput rankings for these
sub-millisecond cells are not stable selection evidence.

## Interpretation

The first native reference rejects the motivating assumption that a 500-job set
must lose the continuously supplied rate because of an unavoidable dispatcher
barrier. Completion-driven claim one gets close to that rate and removes most
static tail imbalance without local hoarding.

It also supplies negative evidence against a larger default claim. A claim of
four or eight reduces queue operations but increases tail radius and sometimes
reduces occupancy, with no consistent throughput reward. FastXSLT should retain
claim one as the experimental dynamic reference until mixed-duration and
isolated-transport evidence demonstrates a different bottleneck.

This result does not select production scheduling. The probe uses uniform tiny
native work, dedicated threads, one machine, no cancellation/failure injection,
and no isolated transport. Mixed work may favor dynamic balancing even more,
while queue contention on another machine could change the acquisition cost.

## Reproduction

```powershell
./scripts/verify-aspnet-workbench.ps1 `
  -TargetFramework net10.0 `
  -TieredOnly `
  -FiniteDispatchBenchmark `
  -FiniteDispatchRounds 3
```

Run the command in three fresh processes on distinct ports. Reduce each
candidate to its median round, then compare the three process medians.
