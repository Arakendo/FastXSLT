# ASP.NET Native Large Prepared-Engine Pressure

| Field | Value |
| --- | --- |
| Date | 2026-08-31 |
| Review | AR-0017; adversarial review Finding 6 |
| Scope | Three overlapping 16-engine generations over a 5,000-item prepared input |
| Claim | Shape-specific prepared-engine retention and reclamation evidence; no general estimator or quota selected |

## Method

The existing ASP.NET registry-pressure endpoint executes the exact admitted
XSLT30 `for-004` stylesheet over a generated 5,000-item source. The source is
170,036 bytes and the stylesheet is 377 bytes. Three independently compiled and
prepared generations contain 16 native engines each. The experiment therefore
admits 8,179,824 aggregate source/stylesheet bytes across 48 experiment engines,
in addition to the ordinary singleton.

After all generations are live, the newest generation produces and retains 256
validated semantic results. Old generations, outcomes, and the current
generation are then released separately. ADR-0015 scalar observations establish
logical ownership; working set, private bytes, and managed heap describe the
whole ASP.NET process. No collection or allocator trimming is forced.

This is standards-shaped execution but not a new conformance denominator: the
stylesheet is unchanged `for-004`, while the larger source is generated pressure
data.

## Result

| Checkpoint | Engines | Outcomes | Outcome bytes | Working set | Private bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Baseline | 1 | 0 | 0 | 67,313,664 | 52,928,512 |
| Generation 1 prepared | 17 | 0 | 0 | 157,720,576 | 149,827,584 |
| Generation 2 prepared | 33 | 0 | 0 | 247,726,080 | 245,764,096 |
| Generation 3 prepared | 49 | 0 | 0 | 338,653,184 | 342,155,264 |
| 256 results retained | 49 | 256 | 14,336 | 339,881,984 | 342,339,584 |
| Two old generations retired | 17 | 256 | 14,336 | 163,991,552 | 157,110,272 |
| All experiment handles released | 1 | 0 | 0 | 73,183,232 | 59,559,936 |
| One-second settlement | 1 | 0 | 0 | 73,613,312 | 61,128,704 |

Each generation's incremental process delta was unusually stable:

| Added generation | Working-set delta per engine | Private-byte delta per engine |
| --- | ---: | ---: |
| 1 | 5,650,432 | 6,056,192 |
| 2 | 5,625,344 | 5,996,032 |
| 3 | 5,682,944 | 6,024,448 |

This does not mean a FastXSLT engine costs six megabytes generally. It means one
engine prepared over this 5,000-item shape contributed approximately that much
whole-process pressure in this run. The earlier tiny `for-004` probe attributed
only about 98 KiB per engine. Engine cardinality therefore cannot be translated
to a memory ceiling without workload shape.

At peak, the process held about 289.4 MB more private memory than baseline while
the aggregate admitted resource-byte lower bound was only 8.18 MB. Raw admitted
bytes omit XDM nodes, relationships, owned strings, compiled state, maps,
allocator metadata, and other process effects; they cannot serve as a
conservative prepared-engine quota estimate.

All logical handles returned immediately to baseline. One second later, working
set remained about 6.3 MB and private bytes about 8.2 MB above baseline. The
settlement series did not establish a return-to-baseline half-life; that result
is right-censored beyond the one-second observation window and must not be
called retained FastXSLT ownership.

## Disposition

This completes the first materially larger prepared-engine pressure slice and
reinforces the hybrid-policy direction:

```text
engine count
    -> cheap cardinality/abandonment protection

exact outcome bytes
    -> deterministic bounded-envelope ownership

prepared-engine estimate
    -> unresolved; must be representation/shape aware if admitted
```

A fixed engine count may still be a useful abuse ceiling, but it is not memory
accounting. A future conservative estimate needs engine-owned XDM/compiled
capacity observations or another defensible upper-bound model; multiplying raw
input bytes by a factor observed here would be benchmark folklore, not policy.

AR-0017 remains Incubating. Longer-duration natural reclamation, more than one
large stylesheet/source shape, consumer headroom, and actual atomic quota races
remain open.

## Reproduction

```powershell
./scripts/verify-aspnet-workbench.ps1 `
  -NativeRegistryPressure `
  -RegistrySummaryOnly `
  -MeasurementRuns 1 `
  -MeasurementRequests 1 `
  -RegistryItems 5000 `
  -RegistryConcurrency 16 `
  -RegistryGenerations 3 `
  -RegistryDelayedOutcomes 256
```
