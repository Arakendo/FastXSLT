# ASP.NET Native Boundary Breakdown

| Field                            | Value                                                                                     |
| -------------------------------- | ----------------------------------------------------------------------------------------- |
| Date                             | 2026-09-03                                                                                |
| FastXSLT source checkpoint       | Working tree above `703f75b`                                                              |
| Host                             | AMD Ryzen 7 7800X3D, 16 logical processors, Windows `10.0.26200.0`                        |
| Toolchain                        | Rust 1.95.0; .NET SDK 10.0.100-preview.7; .NET 8 and .NET 10 runtimes                     |
| Stylesheet                       | Pinned XSLT30 `for-004`                                                                   |
| Tiers                            | 5, 50, and 500 deterministic `order-item` elements                                        |
| Repetitions                      | Five reports per target in one fresh host process per target                              |
| Requests per report              | 10,000 / 4,000 / 1,000 for the 5 / 50 / 500-item tiers                                    |
| Optional comparison dependencies | None in either target                                                                     |
| Claim                            | Private boundary-localization evidence; not an ABI guarantee or general performance claim |

## Question and method

The first `net10.0` tier comparison observed an 11-18% native decline against a
same-day `net8.0` run, while isolated execution improved. That comparison kept
the source checkpoint and SDK fixed but did not keep optional dependency shape
identical, used short lanes, and sampled fresh processes. It nominated the
managed/native boundary for inspection; it did not identify a cause.

The workbench now has a private diagnostic endpoint over the existing native
ABI. It makes no Rust export, unsafe-surface, registry-policy, or transformation
semantic change. For each tier it:

1. compares direct `NativeFastXsltClient.Transform` with the same operation
   behind a one-slot `NativeFastXsltPool`;
2. uses `Stopwatch` around request encoding, the transform export, outcome kind
   and length calls, managed buffer allocation, outcome copying, UTF-8 decoding,
   and outcome release; and
3. verifies every materialized result against the expected XML.

The transform-export phase includes request marshalling, Rust input copying and
identity decoding, engine-registry lookup and `Arc` acquisition, complete
FastXSLT execution/serialization, and outcome accounting/publication. The probe
cannot subdivide those operations. Per-phase timings also include repeated
`Stopwatch` overhead and therefore are localization evidence rather than values
that may be added to an uninstrumented latency budget.

Commands:

```powershell
./scripts/verify-aspnet-workbench.ps1 -TargetFramework net10.0 `
  -NativeBoundaryBreakdown -TieredRequests 1000 -MeasurementRuns 5
./scripts/verify-aspnet-workbench.ps1 -TargetFramework net8.0 `
  -NativeBoundaryBreakdown -TieredRequests 1000 -MeasurementRuns 5
```

The checked-in project remains `net10.0`; `-TargetFramework net8.0` is a local
diagnostic override. Both runs used the same current Rust release library and
excluded the local SaxonCS overlay.

## Controlled target comparison

Values are medians of five reports. Higher throughput and lower export time are
better.

| Tier      | .NET 8 direct/s | .NET 10 direct/s | Direct change | .NET 8 one-slot pool/s | .NET 10 one-slot pool/s | Pool change |
| --------- | --------------: | ---------------: | ------------: | ---------------------: | ----------------------: | ----------: |
| 5 items   |         248,808 |          253,180 |         +1.8% |                264,604 |                 279,325 |       +5.6% |
| 50 items  |          65,014 |           66,721 |         +2.6% |                 62,174 |                  65,133 |       +4.8% |
| 500 items |           7,549 |            7,626 |         +1.0% |                  7,490 |                   7,929 |       +5.9% |

The ordinary single-source `/measure/inprocess` median was 275,459/s under
.NET 8 and 280,426/s under .NET 10, a 1.8% increase. The earlier native decline
therefore did not reproduce when optional dependency shape and current code
were held constant. These small positive movements remain within the range that
deserves longer randomized, power-controlled measurement; they are sufficient
to reject the stronger claim that this .NET 10 target intrinsically caused the
reported 11-18% loss.

Direct-versus-pool differences changed sign across repetitions. At this scale,
separate prepared engines, fixed measurement order, runtime warm-up, and host
noise dominate the small expected lease cost. The experiment does not establish
that pooling is free or faster; it establishes that the current pool machinery
is not a demonstrated explanation for the earlier drift.

## Phase localization

Values are median mean microseconds per instrumented direct call.

| Target / tier | Transform export | Instrumented total | Export share |
| ------------- | ---------------: | -----------------: | -----------: |
| .NET 8 / 5    |            3.102 |              3.677 |        84.4% |
| .NET 10 / 5   |            3.060 |              3.676 |        83.2% |
| .NET 8 / 50   |           15.192 |             15.775 |        96.3% |
| .NET 10 / 50  |           14.585 |             15.157 |        96.2% |
| .NET 8 / 500  |          130.402 |            130.925 |        99.6% |
| .NET 10 / 500 |          125.887 |            126.414 |        99.6% |

The .NET 10 transform-export median was 1.4%, 4.0%, and 3.5% lower than .NET 8
across the three tiers. Copy, decode, and release were individually generally
well below 0.2 microseconds. All probed work outside the combined export was
about 0.5-0.6 microseconds by difference of medians, including probe overhead.

This rules out managed result copying, UTF-8 decoding, outcome release, and the
one-slot lease as material explanations for the larger historical movement. It
does **not** prove that core XSLT evaluation alone owns the time: the transform
export still combines the native request copy/decode, global engine-registry
lookup, `Arc` clone, panic guard, semantic execution, serialization, accounting
lock, outcome-registry lock, handle allocation, and outcome insertion.

## Safe Rust component probe

An ignored release-mode test then measured those combined-export components
without changing or crossing the ABI:

```powershell
cargo test --release -p fastxslt-dotnet-workbench `
  measure_native_transform_export_components -- --ignored --nocapture
```

| Tier      | Request copy + decode | Engine lookup + `Arc` clone | Transform + serialization | Outcome insert | Outcome release | Transform share of listed components |
| --------- | --------------------: | --------------------------: | ------------------------: | -------------: | --------------: | -----------------------------------: |
| 5 items   |              0.048 us |                    0.035 us |                  2.720 us |       0.065 us |        0.052 us |                                93.1% |
| 50 items  |              0.067 us |                    0.035 us |                 12.370 us |       0.058 us |        0.052 us |                                98.3% |
| 500 items |              0.046 us |                    0.035 us |                102.337 us |       0.062 us |        0.049 us |                                99.8% |

The absolute Rust-test and ASP.NET timings are not directly subtractable: they
run in different processes and binaries and the manual probe is one averaged
sample. The within-probe proportions are nevertheless decisive for this
sequential workload. Registry lookup/publication and request identity handling
are not demonstrated hot spots. They may still matter under registry contention
and must retain their current atomic admission behavior.

## Disposition

The previous .NET 10 evidence remains a valid record of the observed runs, but
its target-framework hypothesis is not reproduced by this more controlled A/B.
Do not optimize the managed pool or result-transfer calls from the earlier
drift alone.

The safe Rust split now directs the next probe into `engine.transform` for the
50- and 500-item tiers: profile plan execution, XPath evaluation, result-tree
construction, and serialization before changing representation. A distinct
concurrent registry-contention probe remains appropriate if scaling degrades,
but sequential evidence does not justify registry surgery. Use the existing
AR-0013 prepared-layout and activated-path process for any candidate
optimization. No result here admits a new cache, ABI, unsafe path, or public
contract.
