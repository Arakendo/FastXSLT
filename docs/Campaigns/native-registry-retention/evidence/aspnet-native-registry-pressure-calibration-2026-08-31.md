# ASP.NET Native Registry Pressure Calibration

| Field | Value |
| --- | --- |
| Date | 2026-08-31 |
| Review | AR-0017; adversarial review Finding 6 |
| Workload | Unchanged W3C XSLT30 `for-004` stylesheet; generated 500-item source; independently retained native engines and valid outcomes |
| Host | ASP.NET 8 workbench, native in-process boundary, x86-64 Windows 10.0.26200, AMD Family 25 Model 97 |
| Claim | First end-to-end legitimate registry high-water and logical-versus-process-memory settlement trace; no quota selected |

## Instrumentation

The unpublished native workbench now exposes read-only scalar observations for
current engine, control, and outcome handle counts plus exact bytes owned by
byte-valued outcomes. The ASP.NET adapter uses `SafeHandle` for retained engine
and outcome ownership. No private map, capacity, pointer, compiled form, or XDM
representation crosses the boundary.

The experiment prepares two or three simultaneous generations, each containing
one independent native engine per requested concurrency slot. It then executes
the newest generation, reads and checks every exact semantic result, retains the
valid outcome handles deliberately, retires old generations, releases outcomes,
and finally releases the current generation. Registry counts, outcome bytes,
whole-process working set/private bytes, and managed heap are sampled at each
checkpoint and for 0/10/50/100/250/1,000 ms after explicit release.

The ASP.NET process retains its ordinary singleton native engine throughout, so
the engine high-water includes a one-engine baseline. All experiment handles
are legitimate delayed ownership; abandoned-handle count is zero.

## Matrix

Each row ran in a fresh process. Outcome results were 55 UTF-8 bytes each.

| Concurrency | Generations | Delayed outcomes | Engine high-water | Outcome high-water | Exact outcome bytes | Working-set baseline | Maximum observed working set | Working set at 1 s | Private-byte baseline | Maximum observed private bytes | Private bytes at 1 s |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 2 | 16 | 3 | 16 | 880 | 66,183,168 | 68,964,352 | 68,964,352 | 52,084,736 | 54,423,552 | 52,916,224 |
| 4 | 2 | 64 | 9 | 64 | 3,520 | 66,478,080 | 72,491,008 | 69,369,856 | 52,027,392 | 58,781,696 | 53,366,784 |
| 8 | 3 | 64 | 25 | 64 | 3,520 | 66,170,880 | 82,112,512 | 69,386,240 | 51,773,440 | 69,832,704 | 53,121,024 |
| 16 | 3 | 128 | 49 | 128 | 7,040 | 66,125,824 | 95,821,824 | 69,271,552 | 51,691,520 | 86,523,904 | 53,104,640 |
| 32 | 3 | 256 | 97 | 256 | 14,080 | 66,314,240 | 124,628,992 | 70,021,120 | 51,937,280 | 120,434,688 | 53,886,976 |

Every row returned engine, control, outcome, and outcome-payload ownership
exactly to its pre-experiment registry baseline immediately after explicit
release. The exact `for-004` result remained:

```xml
<?xml version="1.0" encoding="UTF-8"?><out>500.00</out>
```

The 32 × three-generation point retained 96 experiment engines and 256 delayed
results. Its working set rose about 58.3 MiB and private bytes about 68.5 MiB at
the largest sampled checkpoint, then settled to about 3.5 MiB working set and
1.9 MiB private bytes above the fresh-process baseline at one second. Smaller
rows also ended above baseline, and working set was not monotonic after logical
release.

## Interpretation

The trace proves the observation seam, generation/result ownership accounting,
exact outcome-byte attribution, semantic sentinel, and immediate logical
reclamation through concurrency 32. It also confirms that Task Manager-style
memory cannot substitute for registry ownership: allocator, runtime, and OS
page retention continue after FastXSLT owns no experiment handles.

The admitted source plus stylesheet bytes are reported by the endpoint only as
a lower bound. They are not prepared-engine retained-byte accounting. Likewise,
these short runs do not establish latency percentiles or a reclamation
half-life; they establish the sampling mechanism and show that a one-second
sample remains noisy.

AR-0017 remains Incubating. Before policy comparison, the host trace still
needs active controls, cancellation and diagnostic bursts, large/near-limit
outcomes, larger corpus-backed prepared shapes, repeated generation replacement
over a bounded soak, and latency percentiles. Candidate replay and exhaustion
delivery comparison have not started.

## Reproduction

One matrix row is executed with:

```powershell
./scripts/verify-aspnet-workbench.ps1 `
  -NativeRegistryPressure `
  -RegistrySummaryOnly `
  -MeasurementRuns 1 `
  -MeasurementRequests 1 `
  -RegistryItems 500 `
  -RegistryConcurrency 32 `
  -RegistryGenerations 3 `
  -RegistryDelayedOutcomes 256
```

The recorded matrix uses concurrency `1`, `4`, `8`, `16`, and `32`; two- and
three-generation overlap; and delayed outcomes scaled from 16 through 256.
