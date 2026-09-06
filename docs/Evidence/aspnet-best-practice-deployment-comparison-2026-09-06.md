# ASP.NET Best-Practice Deployment Comparison

| Field             | Value                                                                                     |
| ----------------- | ----------------------------------------------------------------------------------------- |
| Date              | 2026-09-06                                                                                |
| Runtime           | .NET 10 preview workbench                                                                 |
| Workload          | Exact `for-004` fixture family at 5, 50, and 500 items                                    |
| Samples           | Three fresh processes, three seeded rounds per process                                    |
| Concurrency       | Separate four-worker and eight-worker runs                                                |
| Members           | 100,000 / 20,000 / 4,000 at 5 / 50 / 500 items                                            |
| Saxon             | Local, non-distributed SaxonCS-HE 13.0.0; selected UTF-8 `TextWriter` lane                |
| Microsoft         | Shared linear sibling-walk `XslCompiledTransform`; fixture-equivalent XSLT 1.0            |
| FastXSLT isolated | Incremental input-order transport, non-retaining consumer, private batch sizes 1/8/32/128 |
| Status            | Exploratory deployment-family evidence; not publication-eligible                          |

## Purpose

The competitive fairness audit requires best-practice deployment comparisons
to remain separate from exact-call results. This experiment allows isolated
FastXSLT to amortize IPC through the accepted ADR-0019 transport while retaining
the evidenced in-process deployment patterns for native FastXSLT, SaxonCS, and
Microsoft.

Every lane processes the same logical member count within a tier. Compilation,
source preparation, and worker startup occur before timing. Isolated results
are delivered incrementally to an allocation-bounded callback and are validated
without retaining a completed outcome collection. Lane order is seeded and
rotated.

## Throughput observations

### Four-worker envelope

| Engine/mode                 |       5 items |      50 items |    500 items |
| --------------------------- | ------------: | ------------: | -----------: |
| FastXSLT isolated batch 1   |      86,498/s |      83,038/s |     45,223/s |
| FastXSLT isolated batch 8   |     244,224/s |     212,100/s |     86,002/s |
| FastXSLT isolated batch 32  | **304,530/s** |     286,777/s |     86,737/s |
| FastXSLT isolated batch 128 |     297,307/s | **313,029/s** | **87,166/s** |
| FastXSLT native             |   1,125,603/s |     575,450/s |    105,212/s |
| Microsoft linear XSLT 1.0   |     373,887/s |     135,082/s |     21,871/s |
| SaxonCS `TextWriter`        |     211,784/s |      82,747/s |     13,203/s |

### Eight-worker envelope

| Engine/mode                 |       5 items |      50 items |     500 items |
| --------------------------- | ------------: | ------------: | ------------: |
| FastXSLT isolated batch 1   |      95,356/s |      83,795/s |      49,208/s |
| FastXSLT isolated batch 8   |     284,093/s |     256,313/s |      86,343/s |
| FastXSLT isolated batch 32  | **391,469/s** |     353,838/s | **123,287/s** |
| FastXSLT isolated batch 128 |     365,255/s | **375,868/s** |     103,513/s |
| FastXSLT native             |   1,358,965/s |     904,883/s |     165,948/s |
| Microsoft linear XSLT 1.0   |     646,873/s |     215,976/s |      29,672/s |
| SaxonCS `TextWriter`        |     246,675/s |     138,817/s |      18,261/s |

Bold values identify only the highest measured isolated throughput within that
tier and worker envelope. They are not selected defaults.

## Interpretation

Batching materially improves isolated throughput. Against batch one, the best
observed isolated lane improves by approximately 3.52x / 3.77x / 1.93x at four
workers and 4.11x / 4.49x / 2.51x at eight workers across 5 / 50 / 500 items.

Native FastXSLT remains the fastest lane in every cell. The best isolated lane
trails Microsoft on the tiny five-item workload, then leads the fixture-
equivalent Microsoft lane by about 2.32x and 3.99x at four workers and 1.74x and
4.15x at eight workers for 50 and 500 items. It leads SaxonCS in every tested
tier, ranging from about 1.44x to 6.75x. These are workload-specific deployment
observations, not general engine rankings.

The batch-size result is deliberately non-universal:

- batch 32 leads tiny work at both worker counts;
- batch 128 leads medium work at both worker counts;
- 500-item work is nearly tied between 32 and 128 at four workers, while batch
  32 leads at eight workers; and
- increasing from four to eight workers does not double isolated throughput.

The throughput leader also carries a containment price. A batch-32 lane can
leave at most 32 admitted members ambiguous for one worker loss, or 128/256 if
all four/eight workers are simultaneously lost. Batch 128 raises those bounds
to 128 per worker and 512/1,024 across the tested pools. The host does not select
these private batch sizes; future FastXSLT policy must balance throughput,
latency, bytes, and ambiguity radius.

## Publication limitations

All requested four-way and eight-way lanes reached their real active-operation
high-water, and semantic validation passed. The three-process distributions
still contained CV failures, and some fixed-member lanes completed below the
250 ms exact-family duration gate. More importantly, this first deployment
family performs correctness warm-up but does not yet implement the sustained
per-lane convergence protocol used by the exact-call publication sampler.

`PublicationEligible` is therefore hard-coded false. The results justify
continuing the benchmark family, not publishing a trophy chart. The next tranche
must add sustained per-lane warm-up, time-balanced or sufficiently long equal-
work intervals, seven fresh processes, and an explicit selection objective that
does not turn one batch size into public policy.

## Reproduction

```powershell
./scripts/measure-best-practice-deployment.ps1 `
  -Samples 3 `
  -WithinProcessRounds 3 `
  -TargetFramework net10.0 `
  -BaseMembers 4000 `
  -Concurrency 4 `
  -LocalSaxonCs

./scripts/measure-best-practice-deployment.ps1 `
  -Samples 3 `
  -WithinProcessRounds 3 `
  -TargetFramework net10.0 `
  -BaseMembers 4000 `
  -Concurrency 8 `
  -LocalSaxonCs
```

The Saxon installation remains under `.workbench/` and is not distributed or
admitted into source control.
