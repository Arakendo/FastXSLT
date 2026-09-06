# Native and Isolated Worker Wave-Occupancy Experiment

| Field       | Value |
| ----------- | ----- |
| Date        | 2026-09-06 |
| Runtime     | .NET 10 preview workbench |
| Source      | Unchanged W3C `for03.xml` five-item source |
| Stylesheet  | Unchanged W3C `for-004.xsl` |
| Workers     | Eight persistent isolated processes and eight dedicated native handles/threads |
| Transport   | Isolated batch-of-one or direct native call per selected worker |
| Samples     | Three fresh host processes |
| Rounds      | Five alternating-order rounds per process |
| Waves       | 2,000 synchronized waves per round |
| Status      | Focused exploratory evidence; not publication-eligible |

## Question

The best-practice benchmark's `items-5` label describes nodes processed inside
each transform, not the number of independent requests presented to the worker
pool. This experiment instead varies the number of independent transforms in
one synchronized wave while holding source work, stylesheet, transport batch
size, prepared state, and the eight-worker pool constant:

```text
wave size 5 -> at most five workers active
wave size 8 -> all eight workers can be active
```

Every wave is pre-armed behind a release gate, released together, and awaited
to completion before the next wave begins. Every result is checked against the
exact `for-004` semantic oracle. The reported throughput includes wave
coordination, which is part of the question being measured.

## Result

Each fresh process contributes the median of its five rounds. The table reports
the median of those three process medians.

| Engine   | Wave size | Occupancy | Transforms/s | Wave p50 | Wave p95 | Wave p99 |
| -------- | --------: | --------: | -----------: | -------: | -------: | -------: |
| Isolated |         5 |         5 |     79,416/s |  52.1 us |  88.6 us | 170.1 us |
| Isolated |         8 |         8 |     80,667/s |  87.2 us | 129.0 us | 220.8 us |
| Native   |         5 |         5 |    132,987/s |  36.6 us |  88.8 us | 130.3 us |
| Native   |         8 |         8 |    166,717/s |  53.3 us |  94.0 us | 128.4 us |

Moving from five to eight independent requests raises native synchronized-wave
throughput by about 25.4%, while whole-wave p50 rises by about 45.6%. Isolated
throughput rises only 1.6% in this paired repeat while its p50 rises 67.4%.
An earlier isolated-only tranche measured an 11.4% uplift; that magnitude did
not repeat after adding and interleaving the native lane. The pool-fill effect
is real, but the isolated throughput magnitude is not stable enough to claim.

The native eight-request wave remains about 8.2 times slower than the earlier
continuously supplied native x8 deployment result of 1.36 million transforms
per second. Mandatory start and completion barriers between every tiny group
turn this into a coordination-latency experiment; they do not reveal a higher
unconstrained engine ceiling. Native fresh-process medians were also materially
variable, reinforcing the exploratory status.

The result confirms the narrow hypothesis: an eight-request wave can fill an
eight-worker pool and produces a measurable ceiling increase over a five-
request wave. It does not imply that an eight-item XML source fills more
workers, that synchronized-wave dispatch is an efficient deployment policy, or
that eight workers improve throughput proportionally. The continuously supplied
deployment family remains the representative throughput experiment.

## Reproduction

```powershell
./scripts/verify-aspnet-workbench.ps1 `
  -TargetFramework net10.0 `
  -TieredOnly `
  -WaveOccupancyBenchmark `
  -WaveOccupancyWaves 2000 `
  -WaveOccupancyRounds 5
```

Run that command in three fresh processes on distinct ports and reduce each
wave size to the median round before comparing process medians.
