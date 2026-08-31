# ASP.NET Native Registry Burst Pressure

| Field | Value |
| --- | --- |
| Date | 2026-08-31 |
| Review | AR-0017; adversarial review Finding 6 |
| Scope | Real active controls, delayed structured failures, and delayed near-limit results through the ASP.NET/native boundary |
| Claim | Legitimate burst-shape and exact outcome-byte evidence; no quota or exhaustion delivery selected |

## Method

The ASP.NET workbench creates eight independent native engines over unchanged
XSLT30 `for-004`. One controlled transform per engine pauses at its first real
work charge, allowing the host to observe all eight active control handles
before sending correlated cancellation. Every transform must return
`FXCT0001 / cancelled` with its request identity before the controls are
released.

The same pool then retains 128 ordinary invalid-request outcomes after decoding
and validating `FXWB0003 / invalid`. A second eight-engine pool prepares a
900,032-byte source and a 184-byte stylesheet that copies a 900,000-byte text
value into one result element. Eight results are decoded and checked for exact
prefix, suffix, length, and repeated payload content while their native outcome
handles remain retained.

Registry counts and exact outcome bytes come from ADR-0015's read-only scalar
observations. Working set, private bytes, and managed heap cover the entire
ASP.NET process. No forced managed collection occurs during settlement.

## Result

| Phase | Engines | Controls | Outcomes | Exact outcome bytes | Working set | Private bytes | Managed heap |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Baseline | 1 | 0 | 0 | 0 | 66,674,688 | 52,240,384 | 3,249,864 |
| Ordinary pool prepared | 9 | 0 | 0 | 0 | 72,478,720 | 58,925,056 | 3,359,648 |
| Active first-charge barriers | 9 | 8 | 0 | 0 | 72,839,168 | 59,236,352 | 3,416,800 |
| Controls completed/released | 9 | 0 | 0 | 0 | 73,117,696 | 59,518,976 | 3,441,216 |
| Delayed structured failures | 9 | 0 | 128 | 9,856 | 73,236,480 | 59,588,608 | 3,531,424 |
| Delayed near-limit results | 17 | 0 | 136 | 7,210,248 | 115,544,064 | 102,273,024 | 29,641,416 |
| Failures released | 17 | 0 | 8 | 7,200,392 | 115,544,064 | 102,273,024 | 29,649,640 |
| Results released | 17 | 0 | 0 | 0 | 106,070,016 | 92,315,648 | 29,657,864 |
| All experiment engines released | 1 | 0 | 0 | 0 | 94,474,240 | 80,740,352 | 29,657,864 |
| One-second settlement | 1 | 0 | 0 | 0 | 96,821,248 | 80,232,448 | 29,698,728 |

Each failure envelope retained exactly 77 bytes. Each large result retained
900,049 bytes, below the 1,048,576-byte per-object ceiling. The legitimate
component high-water was 17 engines, eight controls, 136 outcomes, and 7,210,248
outcome bytes. The active-control phase took 7.95 ms, failure production and
validation 0.84 ms, and large pool creation plus eight transformations and
validation 44.83 ms in this single run. These are phase observations, not
latency distributions or performance claims.

All registry cardinalities and exact outcome bytes returned immediately to the
pre-experiment baseline. Working set and private bytes did not. The managed heap
alone remained roughly 26.4 MiB above baseline because decoded 900 KB strings
had become collection-eligible but no GC was forced. This demonstrates a third
memory class beyond live native ownership and allocator/OS retention: released
native results can leave independently managed allocations awaiting collection.

## Disposition

The first active-control, structured-failure, and near-limit result pressure
requirements are satisfied. The evidence strongly supports retaining an exact
aggregate outcome-byte dimension in any hybrid candidate: outcome count alone
cannot distinguish 136 outcomes occupying 7.2 MiB from 256 outcomes occupying
14 KiB in the earlier generation trace.

AR-0017 remains Incubating. Sustained replacement, latency distributions,
larger admitted corpus/prepared shapes, forced-versus-natural managed
reclamation, policy admission races, and quota-failure delivery remain open.

## Reproduction

```powershell
./scripts/verify-aspnet-workbench.ps1 `
  -NativeRegistryBursts `
  -RegistrySummaryOnly `
  -MeasurementRuns 1 `
  -MeasurementRequests 1 `
  -BurstConcurrency 8 `
  -BurstDelayedFailures 128 `
  -BurstLargeOutcomes 8 `
  -BurstLargePayloadBytes 900000
```
