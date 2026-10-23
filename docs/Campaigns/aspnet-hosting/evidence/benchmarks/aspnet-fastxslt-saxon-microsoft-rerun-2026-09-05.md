# ASP.NET FastXSLT, SaxonCS, and Microsoft Tiered Rerun

| Field               | Value                                                                                                                                                                           |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Date                | 2026-09-05                                                                                                                                                                      |
| FastXSLT checkpoint | `10d2f30` plus uncommitted test-only/documentation work                                                                                                                         |
| Host                | AMD Ryzen 7 7800X3D, 16 logical processors, Windows `10.0.26200.0`                                                                                                              |
| Toolchain           | Rust 1.95.0; .NET SDK/runtime `10.0.100-preview.7.25380.108`                                                                                                                    |
| Managed target      | `net10.0`                                                                                                                                                                       |
| Stylesheet          | Pinned XSLT30 `for-004`; Microsoft lane uses the reviewed XSLT 1.0 equivalent                                                                                                   |
| Tiers               | 5, 50, and 500 deterministic `order-item` elements                                                                                                                              |
| Runs                | Five fresh ASP.NET host processes; median of one tiered sample per process                                                                                                      |
| Command             | `./scripts/verify-aspnet-workbench.ps1 -TargetFramework net10.0 -LocalSaxonCs -TieredBenchmark -TieredSummaryOnly -TieredRequests 1000 -TieredConcurrency 4 -MeasurementRuns 1` |
| Claim               | Private same-machine evidence for one narrow warm workload; not a general engine ranking                                                                                        |

## SaxonCS local-overlay admission

The first attempt correctly failed the repository vulnerability gate. The
gitignored SaxonCS-HE 13.0.0 graph still selected vulnerable AngleSharp 1.2.0
and `System.Security.Cryptography.Xml` 10.0.7 packages.

The local, non-redistributed overlay was amended to pin AngleSharp 1.7.3 and
`System.Security.Cryptography.Xml` 10.0.11. NuGet regenerated its gitignored
lock file, restore completed with zero warnings and zero errors, and the normal
`TreatWarningsAsErrors` audit gate remained enabled. No Saxon package, adapter,
license, lock file, or dependency override entered the distributable project.

This produces evidence for **SaxonCS-HE 13.0.0 with locally overridden patched
transitive packages**, not the original unmodified 13.0.0 dependency graph.

## Throughput medians

Values are completed transforms per second.

| Engine path                   | 5 sequential |    5 x4 | 50 sequential |   50 x4 | 500 sequential |  500 x4 |
| ----------------------------- | -----------: | ------: | ------------: | ------: | -------------: | ------: |
| FastXSLT isolated             |       18,101 |  66,876 |        16,924 |  66,809 |         12,420 |  42,867 |
| FastXSLT native               |      289,103 | 757,381 |       158,857 | 514,297 |         25,818 | 102,785 |
| SaxonCS-HE 13.0.0             |       25,691 |  70,174 |        22,282 |  83,847 |          5,819 |  13,853 |
| Microsoft equivalent XSLT 1.0 |      132,059 | 292,306 |        16,904 |  50,052 |            266 |     740 |

## Relative position in this workload

Ratios use the independent throughput medians above.

| FastXSLT path             | 5 sequential |   5 x4 | 50 sequential |  50 x4 | 500 sequential |  500 x4 |
| ------------------------- | -----------: | -----: | ------------: | -----: | -------------: | ------: |
| Native versus SaxonCS     |       11.25x | 10.79x |         7.13x |  6.13x |          4.44x |   7.42x |
| Isolated versus SaxonCS   |        0.70x |  0.95x |         0.76x |  0.80x |          2.13x |   3.09x |
| Native versus Microsoft   |        2.19x |  2.59x |         9.40x | 10.28x |         97.19x | 138.98x |
| Isolated versus Microsoft |        0.14x |  0.23x |         1.00x |  1.33x |         46.76x |  57.96x |

Native FastXSLT leads every comparison lane. The isolated process boundary is
slower than in-process SaxonCS at 5 and 50 items, nearly equal at five items
with four workers, and ahead by 2.13x sequential and 3.09x at 500 items as its
fixed transport cost becomes a smaller part of the transform.

Microsoft remains much faster than isolated FastXSLT for the tiny tier, is
approximately equal sequentially at 50 items, and falls far behind at 500
items. That result is not an XSLT implementation ranking: Microsoft cannot
execute the exact XPath 2.0 `for` expression and instead runs a recursive XSLT
1.0 equivalent whose allocation and algorithm scale differently.

## Drift from the immediately preceding .NET 10 record

The preceding record did not include SaxonCS because its then-current local
graph failed the security gate. FastXSLT and Microsoft use the same target,
request counts, concurrency, host, and benchmark implementation here.

| Path                 | 5 sequential |   5 x4 | 50 sequential |  50 x4 | 500 sequential | 500 x4 |
| -------------------- | -----------: | -----: | ------------: | -----: | -------------: | -----: |
| FastXSLT isolated    |       -11.1% | -11.9% |         -8.7% | -10.0% |          +8.1% |  +0.0% |
| FastXSLT native      |        -0.9% | -25.1% |         -5.4% |  -3.9% |          -6.3% |  -7.9% |
| Microsoft equivalent |       -32.2% | -30.1% |        +12.6% |  -2.3% |         -14.6% | -22.1% |

The sharp native five-item x4 decline is not accompanied by a comparable
sequential decline. Its five process observations ranged from about 528,198 to
865,089 transforms per second and declined across the collection order. Native
500-item x4 ranged from about 89,334 to 114,448. These distributions are more
consistent with unresolved host/system variability than a localized semantic
regression, but the benchmark does not identify the cause.

SaxonCS also varied substantially. Its 50-item sequential observations ranged
from about 9,560 to 39,814 transforms per second, and its 500-item x4 p99 was
occasionally several milliseconds despite a roughly 13,853/s throughput
median. The medians are useful comparative observations, not precise stable
constants.

## Allocation perspective

Representative process observations retained the earlier allocation shape:

- FastXSLT native managed allocation was roughly 464 bytes per 500-item call;
- FastXSLT isolated managed allocation was roughly 3.1 KiB per call;
- SaxonCS reported roughly 386 KiB per 500-item call; and
- Microsoft's recursive 500-item equivalent reported approximately 8.79 MiB
  per sequential call and substantially more aggregate allocation in the x4
  lane.

These are managed allocations reported by the ASP.NET process. They exclude
Rust allocations and therefore are not total-engine memory comparisons.

## Disposition

The rerun preserves the existing product interpretation: native FastXSLT is
the low-boundary-cost trusted lane, while isolated FastXSLT pays fixed transport
for a stronger containment option. On this exact workload, FastXSLT remains
competitive with or materially ahead of SaxonCS as semantic work grows.

No general conformance, security, or performance superiority follows from this
one stylesheet. The next useful performance-method improvement is randomized
engine order or time-balanced interleaving across fresh processes so sustained
machine drift cannot consistently favor the lane that happens to run first.
