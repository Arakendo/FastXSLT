# ASP.NET Native Sustained Generation Replacement

| Field | Value |
| --- | --- |
| Date | 2026-08-31 |
| Review | AR-0017; adversarial review Finding 6 |
| Scope | Repeated native generation preparation/promotion with bounded old leases and request latency |
| Claim | Sustained legitimate-overlap evidence; no quota, memory estimate, or latency guarantee selected |

## Method

The ASP.NET workbench starts one eight-engine native generation over unchanged
XSLT30 `for-004`, then performs 32 complete replacements. It alternates
500- and 501-item memory-resident sources so every promoted generation has a
distinct expected result. Before each promotion it acquires the current
generation and retains at most two old-generation leases. The replacement is
fully prepared before atomic host promotion.

After each promotion, 16 requests execute through the new current generation at
maximum concurrency eight. The probe conserves the reported generation
identity and exact semantic result. It also executes another transform through
the just-retired lease and checks the old generation's previous result before
eventual release. This exercises 32 replacement samples, 512 promoted-generation
request samples, and 32 retired-generation semantic sentinels.

Registry observations use ADR-0015's read-only scalar exports. Per-request
outcomes are decoded and released normally, so outcome high-water is not sampled
by this probe. Whole-process memory is observational and no managed collection
is forced.

## Result

| Measure | Observation |
| --- | ---: |
| Replacement p50 | 4,634.0 us |
| Replacement p95 | 6,537.4 us |
| Replacement p99 | 7,811.6 us |
| Promoted-generation request p50 | 151.7 us |
| Promoted-generation request p95 | 237.4 us |
| Promoted-generation request p99 | 935.1 us |
| Engine-handle high-water | 25 |
| Control/outcome high-water at checkpoints | 0 / 0 |

The 25-engine high-water is exact for this shape: one ordinary singleton plus
three simultaneous eight-engine generations (current plus two retained old
leases). It first appeared at replacement two and remained bounded through
replacement 32. Draining the two old leases reduced the registry to nine
engines; releasing the current generation restored the baseline singleton.

| Process observation | Baseline | Peak checkpoint | After all releases |
| --- | ---: | ---: | ---: |
| Working set | 65,814,528 | 89,677,824 | 75,137,024 |
| Private bytes | 51,367,936 | 77,979,648 | 60,358,656 |
| Managed heap | 3,303,080 | 7,347,824 | 7,347,824 |

Logical engine ownership remained flat once the overlap window filled and
returned exactly to baseline. Process memory grew and fluctuated despite that
flat ownership, then remained above baseline after release. As in the prior
burst trace, this is not evidence of live native handles: decoded managed
results, native allocators, and the operating system have distinct reclamation
timelines.

## Disposition

This satisfies AR-0017's first sustained replacement and request-latency slice.
It demonstrates that a count candidate must admit at least the configured
current-plus-old-generation overlap for this consumer shape, but it does not
establish that eight engines, two old generations, or 32 replacements are
supported maxima. Replacement latency includes preparation of all eight engines
and is an observation, not a service-level objective.

Larger admitted prepared shapes, a longer-duration soak, a defensible
prepared-engine retention estimate, forced-versus-natural reclamation, and
quota-exhaustion delivery remain open.

## Reproduction

```powershell
./scripts/verify-aspnet-workbench.ps1 `
  -NativeRegistryReplacementSoak `
  -RegistrySummaryOnly `
  -MeasurementRuns 1 `
  -MeasurementRequests 1 `
  -SoakConcurrency 8 `
  -SoakReplacements 32 `
  -SoakRetainedOldGenerations 2 `
  -SoakRequestsPerGeneration 16
```
