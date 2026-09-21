# ASP.NET Stable .NET 10 Performance-Drift Rerun

| Field               | Value                                                                                                                                     |
| ------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| Date                | 2026-09-20                                                                                                                                |
| FastXSLT checkpoint | `6010136`                                                                                                                                 |
| Host                | AMD Ryzen 7 7800X3D, Windows `10.0.26200.0`                                                                                               |
| Toolchain           | Rust 1.95.0; .NET SDK 10.0.401; .NET runtime 10.0.12                                                                                      |
| Managed target      | `net10.0`                                                                                                                                 |
| Workload            | Pinned XSLT30 `for-004`; Microsoft uses the reviewed linear XSLT 1.0 equivalent                                                           |
| Saxon               | Local, non-distributed SaxonCS-HE 13.0.0 `TextWriter` lane                                                                                |
| Exact-call baseline | 2026-09-05 publication-gate rerun in [competitive fairness remediation](competitive-benchmark-fairness-remediation-tranche-2026-09-05.md) |
| Deployment baseline | [ASP.NET best-practice deployment comparison](aspnet-best-practice-deployment-comparison-2026-09-06.md)                                   |
| Claim               | Same-machine drift observation across a runtime, repository, and machine-state change; not an engine ranking or pure code A/B             |

## Environment correction

The retained baselines used .NET 10 preview SDK/runtime
`10.0.100-preview.7.25380.108`. This rerun used the installed stable .NET 10 SDK
10.0.401 and runtime 10.0.12. The newer SDK rejected the gitignored local Saxon
overlay's explicit `System.Security.Cryptography.Xml` package reference because
the target framework already supplies it. Removing that redundant local-only
reference restored a warning-free build without changing the distributed
workbench project or bypassing its warning/security gates.

The machine also contained unrelated managed processes during the run. The
results therefore combine repository drift, stable-runtime drift, and ordinary
shared-host effects. They nominate controlled follow-up; they do not attribute
the change to the Rust evaluator.

## Exact-call family

The sampler used seven fresh ASP.NET processes, three seeded rounds per process,
a 250-request floor, time-balanced measurement intervals, and requested
concurrency four:

```powershell
./scripts/measure-competitive-fairness.ps1 `
  -Samples 7 `
  -WithinProcessRounds 3 `
  -TargetFramework net10.0 `
  -TieredRequests 250 `
  -TieredConcurrency 4 `
  -LocalSaxonCs `
  -SelectedSaxonDestination TextWriter
```

Values are median completed transforms per second. Drift is relative to the
2026-09-05 publication-eligible exact-call record.

| Engine                    |      5 items 1x |        5 items 4x |     50 items 1x |      50 items 4x |   500 items 1x |    500 items 4x |
| ------------------------- | --------------: | ----------------: | --------------: | ---------------: | -------------: | --------------: |
| FastXSLT isolated         | 19,939 (-19.9%) |    90,795 (-3.1%) | 18,716 (-12.8%) |   83,012 (-0.4%) | 11,474 (-8.9%) |  42,216 (-9.7%) |
| FastXSLT native           | 353,830 (-8.9%) | 1,103,316 (-4.8%) | 165,356 (-3.3%) | 510,251 (-15.9%) | 26,193 (-7.1%) | 104,102 (-3.3%) |
| Microsoft linear XSLT 1.0 | 214,118 (+6.3%) |  556,242 (+21.4%) |  64,693 (+5.7%) | 181,518 (+24.3%) |  8,430 (-7.3%) | 24,694 (+24.6%) |
| SaxonCS `TextWriter`      | 116,611 (-2.9%) |  236,291 (+15.8%) |  43,655 (-1.7%) | 118,274 (+31.0%) |  6,017 (+3.8%) | 15,673 (+14.6%) |

Every FastXSLT warm-up stabilized. Every requested concurrency and minimum
measurement duration was reached, and every selected distribution stayed below
the 0.20 coefficient-of-variation ceiling. The overall publication gate still
failed closed: Microsoft linear five-item stabilized in six of seven processes,
the Saxon byte-stream five-item oracle in six of seven, and the selected Saxon
`TextWriter` five-item lane in five of seven. The selected post-warm-up
distributions were nevertheless stable.

The exact-call signal is mixed but negative for FastXSLT. Most native cells are
within nine percent of the earlier record, but native 50-item 4x is 15.9% lower.
Isolated 4x at five and 50 items is essentially stable, while its sequential
small/medium cells and both 500-item cells are lower. The simultaneous gains in
many Microsoft and Saxon concurrent cells make stable-runtime and shared-host
effects material confounders rather than evidence of one uniform engine-wide
slowdown.

## Exploratory best-practice deployment family

The deployment family used three fresh processes, three rounds per process,
4,000 members at the 500-item tier, and separate four/eight-worker runs. It
allows isolated FastXSLT to use private bounded incremental batches while the
in-process engines use their retained deployment shapes.

The table reports the highest measured isolated batch in each cell and the
ordinary in-process lane. Drift compares the corresponding best cell in the
2026-09-06 exploratory record; changing batch winners are not selected defaults.

| Envelope / lane              |            5 items |         50 items |        500 items |
| ---------------------------- | -----------------: | ---------------: | ---------------: |
| x4 isolated best (batch 128) |    277,368 (-8.9%) | 273,872 (-12.5%) |   89,542 (+2.7%) |
| x4 native                    |   968,502 (-14.0%) | 463,052 (-19.5%) |  77,271 (-26.6%) |
| x4 Microsoft                 |    407,542 (+9.0%) | 154,811 (+14.6%) |   20,453 (-6.5%) |
| x4 SaxonCS                   |    228,963 (+8.1%) | 104,101 (+25.8%) |   14,051 (+6.4%) |
| x8 isolated best (batch 128) |   327,539 (-16.3%) |  346,014 (-7.9%) |  119,434 (-3.1%) |
| x8 native                    | 1,174,020 (-13.6%) | 711,521 (-21.4%) | 123,727 (-25.4%) |
| x8 Microsoft                 |   736,820 (+13.9%) |  212,381 (-1.7%) |  36,152 (+21.8%) |
| x8 SaxonCS                   |   333,842 (+35.3%) | 153,599 (+10.6%) |  29,269 (+60.3%) |

The x4 run reached requested concurrency and kept every distribution at or
below 0.20 CV, but some cells were shorter than the 250 ms duration threshold.
The x8 run additionally missed the strict achieved-concurrency gate and had two
unstable isolated distributions (batch-one 500 items and batch-32 five items).
This family remains explicitly non-publication-eligible. Its larger apparent
native declines are a reason to repeat under a controlled idle host, not a
regression verdict.

## Current competitive shape

On the exact-call family, native FastXSLT remains ahead of the selected
Microsoft and Saxon lanes in every tier/concurrency cell. Isolated exact-call
execution still pays its fixed per-command boundary and exceeds both comparison
lanes only at the 500-item tier. In the deployment family, bounded batching
lets isolated FastXSLT exceed both competitors at 50 and 500 items for x4/x8;
at five items it trails Microsoft and is approximately tied with Saxon at x8.

These statements are fixture-specific. Microsoft executes a disclosed XSLT 1.0
equivalent rather than the exact XPath 2.0 expression, and Saxon implements a
substantially broader language surface.

## Disposition

Do not replace the publication-eligible 2026-09-05 exact-call record with this
run, because the selected Saxon warm-up gate did not pass. Record the current
run as a material drift signal, especially for native 50-item 4x and the short
best-practice native cells. Before changing engine code, rerun on an otherwise
idle host or compare the old and current repository checkpoints under the same
stable .NET runtime. The evidence does not localize the drift to compilation,
P/Invoke, dispatch, semantic execution, or result transfer.
