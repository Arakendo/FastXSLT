# Capacity lifecycle reuse sweep and ingestion attribution

- Date: 2026-10-03
- Review: [AR-0027](../Architectural%20Reviews/AR-0027-execution-oriented-prepared-document-layout.md)
- Prior measurement: [One and eight use lifecycle](ar-0027-capacity-single-use-lifecycle-2026-10-02.md)
- Disposition: Broader reuse measured; no universal timing or break-even decision

The direct Rust lifecycle now covers 1/2/4/8/16/32/64 uses of one freshly admitted
source. In three recorded release processes, pre-sizing improves wide-source
single-use total medians by 7.6–11.1%, but the advantage falls to 0.6–3.1% at
64 uses. Other shapes show smaller or reversing differences. The earlier
retained-capacity evidence remains stronger than any general speed claim.

## Scope and reproduction

Only the private lifecycle test and measurement harness is extended. Production
construction, admission, preparation, execution and serialization use the same
owners as the previous lifecycle experiment. Compilation and fixture creation
remain outside timing. Every sample freshly admits and seals its source, builds
a prepared document, executes independent invocations, serializes and validates
each exact count result, then releases sample-owned state before its total
clock ends. One source used 64 times is not 64 queued sources.

The ordinary control now covers 126 source/candidate/reuse combinations and
2,286 exact outputs. The release probe warms each lane 16 times and records
201 samples per shape/reuse group, rotating the candidate lane first in each
iteration. Reuse-group order also rotates by shape and an explicitly supplied
test-only process offset. Offsets 0, 4 and 6 supply the three recorded processes;
shape order remains fixed.

```powershell
$env:AR0027_REUSE_ROTATION = '0' # Repeat in fresh processes with '4' and '6'.
cargo test -p fastxslt --release --features workbench measure_capacity_single_use_and_reuse_lifecycle -- --ignored --nocapture --test-threads=1
Remove-Item Env:AR0027_REUSE_ROTATION
```

The offset must be an integer from zero through six. It changes observation order
only, not engine semantics or policy. Allocation-observation is omitted for
timing. Each recorded process emits 126 phase-median records and validates
459,486 timed outputs, excluding warm-up. These are synthetic regression
outputs, not conformance passes or independent job queues.

The three recorded processes last approximately 28–30 seconds. An additional
rotation-six run that overlapped verification activity was excluded and rerun
after the gates completed. No steady-state convergence, background-load control,
cross-machine validation or host latency distribution is established. The
workbench-only feature build emits existing optional-test unused-import/dead-code
warnings; the all-feature strict gate passes without suppressing them.

## Complete lifecycle medians

Microseconds per fresh admitted-source lifecycle through release. Each cell is
growth / frozen / pre-sized. Values at higher reuse counts include all those
transforms; they are not per-transform latencies. Phase medians are not additive.

| Shape | Uses per source | Offset 0 | Offset 4 | Offset 6 |
| --- | ---: | ---: | ---: | ---: |
| wide | 1 | 1785.7 / 1633.5 / 1650.4 | 1391.4 / 1392.8 / 1237.3 | 1315.7 / 1340.3 / 1209.7 |
| wide | 2 | 1290.0 / 1245.8 / 1196.7 | 1387.4 / 1398.0 / 1248.3 | 1423.8 / 1405.9 / 1284.4 |
| wide | 4 | 1384.8 / 1335.6 / 1310.2 | 1475.2 / 1461.1 / 1357.0 | 1723.8 / 1755.4 / 1563.9 |
| wide | 8 | 1660.0 / 1631.0 / 1598.7 | 2273.8 / 2320.6 / 2096.4 | 1904.8 / 1876.7 / 1770.7 |
| wide | 16 | 2196.6 / 2184.7 / 2124.0 | 2484.1 / 2668.2 / 2396.7 | 2549.8 / 2603.0 / 2505.2 |
| wide | 32 | 3301.1 / 3267.4 / 3194.7 | 3592.0 / 3651.9 / 3555.9 | 3855.5 / 3938.7 / 3822.6 |
| wide | 64 | 5693.4 / 5640.8 / 5578.8 | 6398.2 / 6371.3 / 6360.7 | 6746.7 / 6700.0 / 6540.1 |
| deep | 1 | 125.5 / 122.7 / 122.0 | 187.1 / 189.2 / 183.2 | 132.4 / 132.7 / 129.7 |
| deep | 2 | 150.3 / 148.6 / 145.0 | 145.3 / 145.4 / 141.7 | 164.5 / 164.8 / 160.9 |
| deep | 4 | 191.8 / 192.0 / 188.7 | 192.8 / 194.3 / 191.9 | 225.6 / 226.0 / 223.2 |
| deep | 8 | 290.9 / 291.2 / 288.1 | 287.9 / 288.2 / 285.7 | 351.4 / 347.1 / 349.5 |
| deep | 16 | 491.7 / 492.3 / 487.0 | 481.4 / 481.4 / 479.5 | 579.7 / 580.4 / 578.0 |
| deep | 32 | 888.3 / 890.1 / 888.9 | 894.9 / 899.2 / 895.3 | 1073.4 / 1071.8 / 1066.8 |
| deep | 64 | 1704.9 / 1692.0 / 1702.3 | 2105.2 / 2152.6 / 2171.2 | 2050.0 / 2041.7 / 2050.7 |
| attribute-heavy | 1 | 542.9 / 539.3 / 526.9 | 514.7 / 518.4 / 502.5 | 527.0 / 520.6 / 510.2 |
| attribute-heavy | 2 | 518.4 / 525.4 / 519.4 | 508.8 / 510.1 / 498.3 | 518.0 / 518.1 / 506.4 |
| attribute-heavy | 4 | 519.9 / 516.8 / 507.4 | 539.9 / 553.9 / 523.5 | 531.4 / 533.7 / 523.1 |
| attribute-heavy | 8 | 531.2 / 529.3 / 524.0 | 533.0 / 536.8 / 518.9 | 542.4 / 541.8 / 529.4 |
| attribute-heavy | 16 | 559.7 / 550.4 / 544.7 | 561.3 / 565.1 / 551.4 | 571.4 / 570.7 / 558.6 |
| attribute-heavy | 32 | 630.3 / 638.2 / 624.7 | 625.5 / 622.3 / 613.5 | 619.2 / 620.5 / 603.2 |
| attribute-heavy | 64 | 707.4 / 716.7 / 705.0 | 700.9 / 704.3 / 686.2 | 712.6 / 709.6 / 700.8 |
| text-heavy | 1 | 167.7 / 167.5 / 166.2 | 169.9 / 169.7 / 168.4 | 165.0 / 165.6 / 164.0 |
| text-heavy | 2 | 177.6 / 177.7 / 176.5 | 180.2 / 181.4 / 178.4 | 173.0 / 173.0 / 171.5 |
| text-heavy | 4 | 212.6 / 220.8 / 214.4 | 192.3 / 192.0 / 187.7 | 187.8 / 188.6 / 186.0 |
| text-heavy | 8 | 211.1 / 210.6 / 208.8 | 210.6 / 211.5 / 208.8 | 212.5 / 213.3 / 211.3 |
| text-heavy | 16 | 254.3 / 253.9 / 252.7 | 350.0 / 332.5 / 360.4 | 264.2 / 264.4 / 261.6 |
| text-heavy | 32 | 362.6 / 362.0 / 363.7 | 368.1 / 365.9 / 358.7 | 371.7 / 371.9 / 369.8 |
| text-heavy | 64 | 527.8 / 526.3 / 522.4 | 535.9 / 538.3 / 538.6 | 586.9 / 590.2 / 585.5 |
| namespace-heavy | 1 | 718.6 / 704.3 / 732.8 | 405.1 / 404.7 / 404.1 | 438.3 / 439.3 / 434.6 |
| namespace-heavy | 2 | 539.5 / 557.2 / 541.9 | 412.2 / 412.7 / 411.3 | 434.6 / 433.8 / 432.6 |
| namespace-heavy | 4 | 489.9 / 507.4 / 517.0 | 477.5 / 480.9 / 476.5 | 442.1 / 444.1 / 440.0 |
| namespace-heavy | 8 | 680.0 / 738.8 / 690.0 | 440.3 / 442.1 / 439.3 | 466.2 / 466.4 / 464.4 |
| namespace-heavy | 16 | 543.1 / 546.8 / 552.0 | 492.1 / 494.5 / 493.7 | 504.9 / 504.6 / 503.6 |
| namespace-heavy | 32 | 556.0 / 561.3 / 559.3 | 595.5 / 592.7 / 589.1 | 581.7 / 585.5 / 578.8 |
| namespace-heavy | 64 | 787.7 / 776.8 / 785.6 | 692.9 / 694.3 / 690.9 | 731.9 / 733.6 / 734.1 |
| low-repetition | 1 | 650.4 / 679.4 / 657.1 | 613.6 / 623.9 / 603.2 | 604.6 / 610.2 / 597.7 |
| low-repetition | 2 | 650.6 / 643.0 / 643.1 | 634.3 / 634.2 / 624.6 | 644.1 / 645.6 / 629.1 |
| low-repetition | 4 | 734.3 / 714.5 / 733.4 | 958.7 / 938.4 / 912.5 | 718.5 / 722.5 / 709.7 |
| low-repetition | 8 | 840.0 / 829.0 / 829.6 | 963.0 / 990.3 / 972.7 | 898.1 / 894.0 / 884.9 |
| low-repetition | 16 | 1166.1 / 1150.0 / 1143.1 | 1151.9 / 1140.1 / 1126.9 | 1210.6 / 1211.1 / 1208.2 |
| low-repetition | 32 | 1880.5 / 1846.3 / 1897.9 | 1735.5 / 1698.0 / 1774.2 | 1866.2 / 1861.3 / 1844.9 |
| low-repetition | 64 | 2983.9 / 3032.1 / 2977.1 | 2845.9 / 2825.6 / 2886.2 | 3139.6 / 3160.0 / 3129.1 |

Pre-sized totals are lower in all three runs for 24 of the 42 shape/reuse groups;
the other 18 reverse direction between processes. None is slower in all three.
These sign counts do not establish statistical significance: several differences
are sub-percent, and samples share a process and group context.

Wide sources show a repeatable preparation-related advantage in this workload;
deep-source advantages become small and eventually reverse with higher reuse.
All seven namespace-heavy groups reverse across runs. The curves do not nominate
one reuse threshold for all source shapes, nor prove a warm evaluator speedup.

As a phase example, offset-zero wide single-use growth / pre-sized preparation
medians are 1,558.3 / 1,428.9 us, with complete totals 1,785.7 / 1,650.4 us.
At 64 uses, preparation is 1,074.1 / 1,012.1 us while summed execution is
4,330.4 / 4,321.4 us. This describes where the observed service time sits;
subtracting phase medians cannot identify allocator cleanup or causal speedup.
Comparing the two reuse groups alone also cannot separate reuse from temporal
machine drift.

## Existing adapter ingestion boundaries

Inspection confirms that XML parsing and XDM construction are already Rust-owned.
No managed DOM is present in these byte-based reference lanes.

| Boundary | Existing work | Required measurement distinction |
| --- | --- | --- |
| Native managed creation | ABI check, UTF-8 encoding of two identities, existing create export, outcome-kind/take-engine calls and SafeHandle wrapper | Separate managed staging and outcome ownership from the combined native call |
| Native create export | Bounded copies of identities/source/style into owned Rust buffers, identity validation, engine creation and atomic registry admission | Do not call the whole export XML preparation |
| Shared engine creation | Admit/seal source and stylesheet, compile stylesheet, prepare selected source and seal prepared set | Compilation, admission and source preparation need separate attribution |
| Isolated managed initialization | Start a process, write framed identities/source/style, flush and wait for readiness | Process startup is not a preparation cost or a warm-worker comparison |
| Isolated worker initialization | Bounded frame decoding into Rust-owned buffers, then the same shared engine constructor | Keep transport/decode separate from semantic construction |

The current native and isolated initialization calls both compile and prepare.
The direct capacity probe compiles outside each measured source lifecycle.
Subtracting its total from either adapter initialization would therefore mix
different lifecycle scopes and cannot establish boundary overhead.

Managed resource byte arrays are supplied to the current native adapter rather
than converted from a DOM. The native export copies the supplied buffers before
semantic work. The isolated adapter encodes identities and writes length-framed
byte arrays; its decoder allocates bounded Rust buffers before shared engine
creation. OS pipe buffering, pinning costs, copy counts beyond these visible
operations and co-resident managed/native peaks remain unmeasured.

Next, instrument the existing paths without expanding the native export surface
or inventing a supported facade: managed encoding/staging, full creation call,
outcome ownership, process startup, request framing and readiness. A separately
matched direct Rust full-engine creation lane should include compilation when
used as that comparison control. Granular Rust admission/compile/XML/XDM
observation must stay explicitly supplied and private; no ambient global
subscriber or production timing contract follows.

These are inspected boundaries and a measurement plan, not host measurements.
No staging relocation, zero-copy borrowing, authority change, new ABI export,
resolver callback or preparation scheduler has been selected.

## Validation and remaining work

Full verification passes with 1,538 core tests and 48 ignored probes, all adapter
tests, formatting, strict Clippy, documentation, local links, corpus integrity
and unsafe-surface checks. The official WASM target builds; timing observations
are host-width Rust measurements, not WASM benchmarks.

AR-0027 remains Incubating. Reuse-point measurement is complete for this count
fixture family; result-heavy reuse, representative adapter creation/ingestion,
retained relationship capacities and checkpoint/publication review remain open.
No break-even or capacity adoption is selected. OASIS stays 2,485 / 3,173
(78.32%).
