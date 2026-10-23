# ASP.NET native extended reclamation observation — 2026-08-31

## Scope

This extends AR-0017's one-second settlement window after a large legitimate
prepared-engine and delayed-outcome burst. It observes natural process behavior
only: the experiment does not force managed collection, trim an allocator, or
recycle the ASP.NET process.

The workbench now accepts a bounded 1,000–60,000 millisecond settlement window
and samples fixed milestones through the requested endpoint. The default
remains one second; this run requested 30 seconds.

## Workload

- unchanged XSLT30 `for-004` semantic sentinel;
- 5,000 source items;
- three generations ×16 native engines (49-engine high-water including the
  ordinary singleton);
- 256 deliberately delayed valid outcomes containing exactly 14,336 native
  payload bytes; and
- explicit release of every experiment handle before settlement sampling.

Command:

```powershell
./scripts/verify-aspnet-workbench.ps1 `
  -NativeRegistryPressure `
  -RegistrySummaryOnly `
  -RegistryItems 5000 `
  -RegistryConcurrency 16 `
  -RegistryGenerations 3 `
  -RegistryDelayedOutcomes 256 `
  -RegistrySettlementMilliseconds 30000
```

## Observation

| Milliseconds after release | Working set bytes | Private bytes | Managed heap bytes |
| ---: | ---: | ---: | ---: |
| Baseline | 67,395,584 | 52,858,880 | — |
| Peak | 339,099,648 | 341,225,472 | — |
| 0 | 72,679,424 | 57,831,424 | 4,385,080 |
| 10 | 72,482,816 | 56,193,024 | 4,397,336 |
| 50 | 72,491,008 | 56,193,024 | 4,405,560 |
| 100 | 72,491,008 | 56,176,640 | 4,413,784 |
| 250 | 72,626,176 | 57,151,488 | 4,413,784 |
| 1,000 | 72,658,944 | 56,172,544 | 4,421,752 |
| 2,000 | 72,724,480 | 56,164,352 | 4,429,976 |
| 5,000 | 72,749,056 | 56,254,464 | 4,429,976 |
| 10,000 | 68,915,200 | 52,334,592 | 4,438,200 |
| 30,000 | 69,451,776 | 52,531,200 | 4,462,688 |

Logical registry ownership returned exactly to baseline. Relative to the peak,
98.06% of the working-set delta and 98.28% of the private-byte delta had already
disappeared before the first zero-delay observation. The experiment therefore
cannot resolve a meaningful peak-to-half reclamation time: it occurred during
explicit disposal and synchronous destruction before the first process sample.

At ten seconds private bytes were 524,288 bytes below the pre-experiment
baseline; at 30 seconds they remained 327,680 bytes below it. Working set was
about 1.5 MiB above baseline at ten seconds and 2.0 MiB above it at 30 seconds.
The non-monotonic intermediate values reinforce that neither measure is exact
ownership accounting.

## Interpretation

- This run supplies no evidence of a long-lived approximately 289 MB native
  retention tail after logical release.
- It does not establish a universal reclamation guarantee, because allocator,
  OS, workload, and surrounding host state can change the curve.
- A precise half-life claim would be false precision: the relevant reduction
  occurred below the harness's post-disposal sampling resolution.
- Registry cardinality, exact outcome bytes, and the private compositional
  engine estimate remain the deterministic policy inputs. Process memory is a
  corroborating operational signal.
- Forced collection or process recycling may still be measured separately if a
  consumer requires a recovery action, but those are different policies from
  natural reclamation.
