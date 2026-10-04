# AR 0027 Direct Rust Lifecycle Baseline

| Field | Value |
| --- | --- |
| Date | 2026-10-02 |
| Review | [AR-0027](../Architectural%20Reviews/AR-0027-execution-oriented-prepared-document-layout.md) |
| Workload | Unchanged XSLT30 `for-004` stylesheet and `for03.xml` source |
| Suite revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Toolchain | Rust 1.95.0, local Windows native release build |
| Scope | Private direct Rust lifecycle timing control; no layout, adapter or public API change |

The first AR-0027 implementation establishes a directly executable Rust caller
control without .NET, P/Invoke, worker framing or ambient file access. It proves
the current lifecycle and records exploratory timings before any optimization.
It does not yet compare host adapters or nominate a compact representation.

## Implementation and functional controls

The private [lifecycle harness](../../crates/fastxslt/src/runtime/prepared_input_experiment/direct_lifecycle_tests.rs)
uses existing resource admission, compilation, preparation, execution and
serialization owners. Input bytes come from compile-time references to the
pinned upstream files; the fixture bytes and upstream gitlink are unchanged.
No new dependency, public type, unsafe surface, semantic evaluator or native
observation export is introduced.

Two ordinary tests verify:

- the exact serialized `for-004` result `<out>36.02</out>` with the current XML
  declaration, cold execution under two source identities, and repeated prepared
  execution; and
- cancellation at the second real XML-event charge, exact cancellation domain,
  absence of a partially admitted prepared entry, and successful preparation and
  execution on retry with a fresh control.

The manual probe has two explicitly different lifecycle scopes:

1. **One-shot:** copy/admit source and stylesheet bytes, seal the snapshot,
   compile, prepare the source, execute, serialize, validate, and release all
   owned state. Compilation occurs on every iteration deliberately.
2. **Compiled/prepared reuse:** retain one compiled stylesheet and one prepared
   source outside timing, then lookup/execute, serialize, validate and release
   invocation results. This is not a fresh-document or finite-job-queue test.

Phase clocks separately observe admission/sealing, compilation, preparation,
execution including prepared lookup, and serialization. Total timing includes
validation, bookkeeping and cleanup; phase timing excludes validation. Preparation
currently combines XML parsing and XDM construction. Source/stylesheet byte
copies are inside one-shot admission, but host file/database acquisition and
identity-string generation are not measured.

## Reproduction and method

```text
cargo test -p fastxslt --all-features direct_lifecycle -- --nocapture
cargo test -p fastxslt --release --features workbench measure_direct_lifecycle_baseline -- --ignored --nocapture
```

The timing build intentionally omits `allocation-observation`. Each fresh test
process warms both scopes 32 times, then records 1,001 observations per scope,
alternating which scope executes first. Validation occurs on every invocation.
Three fresh-process repetitions below ran after agent-started verification and
WASM checks completed. Earlier runs overlapped verification and are not used in
these timing tables. Unrelated host activity was not controlled.

These are short, instrumented unit-test windows, not a publication-quality
convergence experiment. Clock overhead is not subtracted. Test-only prepared
retention bookkeeping is present; production adapter creation is not assumed
to have identical overhead. Phase medians must not be summed or subtracted from
the total median as though they describe one specific invocation.

## Current observations

All values below are per-process medians in microseconds.

| One-shot run | Admission/seal | Compile | Prepare | Execute/lookup | Serialize | Complete lifecycle |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.4 | 16.7 | 8.1 | 1.6 | 0.4 | 28.7 |
| 2 | 0.4 | 18.6 | 8.4 | 1.8 | 0.5 | 32.5 |
| 3 | 0.4 | 16.4 | 8.0 | 1.6 | 0.4 | 28.4 |

| Reuse run | Execute/lookup | Serialize | Complete invocation |
| --- | ---: | ---: | ---: |
| 1 | 1.5 | 0.3 | 2.0 |
| 2 | 1.5 | 0.3 | 2.1 |
| 3 | 1.5 | 0.3 | 2.0 |

Admission, compilation and preparation are not performed inside the reuse
window; zero-valued output fields for them mean omitted work, not instantaneous
work. The complete one-shot timing is not a recommended deployment strategy
for a queue using one reusable stylesheet.

Each run reports the same preparation observation: 216 admitted source bytes,
23 XDM nodes, 7,116 parser-phase accounted capacity bytes and 8,894 owned-XDM
accounted capacity bytes. Logical identity length influences provenance storage;
do not treat differences from historical identity strings as memory drift.
These capacities are not allocator-inclusive peak, total process memory or a
hard memory ceiling. This tranche does not measure allocation counts or total
copied-byte traffic across host adapters.

## Interpretation and next work

For this tiny source and stylesheet, compilation and preparation are much larger
than the measured semantic execution and serialization phases. That supports
using a compile-once/prepared-reuse control and keeping cold ingestion visible;
it does not establish a general reuse speedup, a new layout win or a managed
staging bottleneck. No .NET-to-Rust operation was moved.

Next instrument the existing native and isolated creation paths without adding
public timing exports. Separate managed encoding/framing, foreign-buffer/pipe
transfer, Rust admission, compilation, XML parsing and XDM construction. Before
using a 5,000-job lane, add compile-once fresh-document ingestion and distinct
request/source identities; do not relabel one prepared source reused 5,000 times
as a 5,000-document queue. Longer windows, tail measurements, allocation/copy
attribution, failure cleanup and representative source shapes remain pending.

## Cohesion and validation

The new 182-line test child owns direct lifecycle control and observation only.
It is under the named `prepared_input_experiment` subject and depends one-way on
existing private owners. The 1,075-line preparation parent adds only four
test-module registration lines; its source preparation/lifetime responsibility
is unchanged. The dense runtime root stays at 54 direct Rust files. Retain the
parent at this checkpoint and use the named child for future lifecycle probes;
no unrelated file moves or new generic provider abstractions are justified.

Workspace verification passes: formatting, strict all-feature Clippy, 1,507 core
tests with 36 manual probes ignored, other workspace tests, documentation,
Markdown links, corpus inventories and the reviewed unsafe-surface gate. The
official `wasm32-unknown-unknown` adapter check passes. The narrower release
feature configuration emits existing feature-conditional unused-code warnings;
none was suppressed, and the strict all-feature workspace gate remains intact.
No corpus selection, expected result or compatibility numerator changed.

## Compile once with fresh documents

The follow-up adds a third lifecycle scope rather than relabeling prepared
reuse. One compiled `for-004` program remains outside timing; every request
copies and admits its own source bytes, seals a source-only snapshot, prepares
a new document, executes, serializes, checks exact output and releases its
request-owned state. This particular stylesheet has no runtime resource
dependencies. The source-only snapshot is not a general contract for separating
arbitrary stylesheet and dynamic-resource graphs.

An additional ordinary test proves that equal-byte sources have distinct
document origins and correct logical-resource provenance. Dropping one prepared
owner does not invalidate the other, and the same compiled program executes a
newly prepared source after that retirement.

```text
cargo test -p fastxslt --release --features workbench measure_direct_lifecycle_fresh_sources -- --ignored --nocapture
```

Each of three fresh processes warms 32 times, then processes 5,000 distinct
source/request identities on one continuously supplied sequential worker.
Fixed-width identities and report-vector capacity are created before timing.
The whole queue clock includes per-job instrumentation, validation, release and
sample insertion; per-job total clocks exclude sample insertion. The reported
one-document high-water is a structural bound of the scoped sequential harness,
not allocator or process-memory instrumentation. Raw identity strings and phase
samples are retained outside that document scope.

| Fresh-source run | Admission/seal us | Prepare us | Execute/lookup us | Serialize us | Complete per-job median us | 5,000-job elapsed ms | Jobs/s |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.2 | 8.1 | 1.5 | 0.4 | 11.5 | 77.9884 | 64,112 |
| 2 | 0.3 | 11.9 | 2.3 | 0.6 | 16.8 | 93.5143 | 53,468 |
| 3 | 0.2 | 7.5 | 1.5 | 0.4 | 10.6 | 63.9989 | 78,126 |

Compilation is excluded, not instantaneous. These short 64-94 ms windows vary
substantially and do not meet a publication/convergence gate. The table is not
a controlled speedup comparison with the earlier two-lane runs. It establishes
a missing lifecycle control: compiling once still leaves fresh-document
preparation visible, whereas prepared reuse excludes it.

All 15,000 timed outcomes match the unchanged expected serialized result. This
is a synthetic equal-byte queue with separate origins, not a replay of the
consumer's 5,000 different XML files, an eight-worker capacity measurement or
evidence that a compact layout wins. Adapter attribution, varied source shapes,
XML/XDM phase separation and allocation/copy accounting remain pending.

The private child is now 269 lines; the preparation parent and production
behavior remain unchanged. Updated workspace gates pass with 1,508 core tests
and 37 ignored manual probes, and the official WASM check passes. The original
two-scope observations above remain historical evidence of their own runs.

The later [shared preparation phase attribution](ar-0027-shared-preparation-phase-attribution-2026-10-02.md)
separates XML and XDM at the existing owner. It extends, rather than replaces,
these complete-lifecycle observations; adapter and layout comparisons remain open.

## Distinct payload compile once follow up on 2026 10 03

The private direct Rust control now varies XML payloads as well as logical
origins across a sequential 5,000-job mixed queue. The unchanged pinned stylesheet
is compiled once. Jobs cycle through 5/50/500 items, with a unique root job
attribute and rotated quantities 1–5; expected totals are computed independently
during fixture generation. This is synthetic distinct input, not a consumer
publication replay or new conformance credit.

Each job freshly admits one source, seals, prepares, executes, serializes,
validates, and releases result/prepared state inside its total clock.
Acquisition, fixture/identity creation, compilation and sample collection are
outside per-job clocks. One prepared document is live in the measurement loop.
Invocation controls remain unbounded as in the earlier direct Rust baseline,
so this is not a limits-matched comparison with native/isolated initialization.
No source-only snapshot is generalized to stylesheets with runtime dependencies.

An ordinary control compares reused compilation with fresh compilation over
12 jobs, checking 24 exact outputs, then overlaps independent prepared origins,
checks root provenance, retires one and verifies the other still executes.
It adds 12 retirement-survivor outputs and 12 complete-lifecycle outputs.
The private child is grouped under the existing `host_placement` test subject;
no production owner, API, ABI, parser, capacity policy or dependency changes.

Three fresh all-feature release test processes use starting offsets 0/1/2,
32 warmups and 5,000 timed jobs each. All 15,000 timed outputs compare exactly.
Each window handles about 31.7 MB of supplied source bytes, freshly copied on
admission. The phases below are medians in microseconds; independently
computed medians need not sum to total.

| Offset | Items per transform | Admission and preparation | Execution serialization validation | Release | Total |
| --- | --- | --- | --- | --- | --- |
| 0 | 5 | 11.2 | 2.6 | 0.9 | 14.7 |
| 0 | 50 | 64.7 | 5.6 | 5.6 | 76.3 |
| 0 | 500 | 627.5 | 37.1 | 51.9 | 723.6 |
| 1 | 5 | 11.0 | 2.6 | 0.9 | 14.6 |
| 1 | 50 | 65.1 | 5.8 | 5.6 | 76.7 |
| 1 | 500 | 625.5 | 37.7 | 51.7 | 728.9 |
| 2 | 5 | 11.2 | 2.7 | 1.0 | 15.0 |
| 2 | 50 | 64.8 | 5.6 | 5.8 | 76.6 |
| 2 | 500 | 610.1 | 36.6 | 52.1 | 708.3 |

Preparation remains dominant at 500 items (610.1–627.5 us); execution plus
serialization/validation is 36.6–37.7 us and release 51.7–52.1 us. This does not
measure XML versus XDM separately, or select a staging/layout optimization.
The complete per-job total is 708.3–728.9 us. Short windows, test instrumentation,
unbounded controls and differing payloads prevent subtracting adapter medians
as an exact interop or compilation tax. No publication-convergence gate passed.

Reproduce after building the release test binary, with no simultaneous
verification or compilation workload:

```powershell
$env:AR0027_INGESTION_ORDER = '0'
cargo test -p fastxslt --release --all-features measure_distinct_payload_ingestion -- --ignored --nocapture --test-threads=1
```

Repeat in fresh processes with offsets 1 and 2. Full gates pass with 1,540 core
tests (50 ignored), native 18, WASM 3 and worker 18 tests, strict Clippy, docs,
links and corpus inventory. The official WASM target check passes. Filesystem
hard-link-cache fallback warnings remain environmental; no lint is waived.
Compile-once adapter ingestion, cancellation/retention attribution and
result-heavy distinct sources remain open.

## Result heavy prepared reuse follow up on 2026 10 03

The private Rust lifecycle now has a source-copy workload rather than a tiny
decimal result. Each of 5/50/500 items contains 512 ASCII text bytes plus escaped
ampersand and less-than characters. Unique root job attributes distinguish
sources; independently generated expected XML checks full copied content.
Results for job 8 are 2,759/27,099/270,949 bytes; two-digit job attributes add
one byte. This synthetic fixture is not a consumer corpus or conformance claim.

One stylesheet is compiled outside timing and copies the document element.
Every sample admits owned bytes and prepares a fresh source, then executes,
serializes and validates either one or eight independent invocations. Reuse
means multiple transforms of that sample's prepared source, not eight queued
sources. Outputs/results are released after each invocation, prepared state
after the final invocation; all release is inside total timing. Input and
expected-output generation are outside timing. No tree-layout candidate or
adapter changes are included.

The probe explicitly supplies parser limits of 10,000 events and depth 64,
with resource/result ceilings of 1 MiB and unbounded invocation controls.
The first 500-item control correctly failed the builder's default 1,024-event
limit; only this experiment's supplied ceiling changed. Production defaults
remain unchanged. Neither these limits nor the unbounded controls imply parity
with a deployment profile.

The ordinary control checks six distinct sources with 54 one/eight-use outputs
and six additional results serialized after source/prepared owners are dropped.
Full exact output survives source retirement. Three fresh all-feature release
test processes rotate size order, with eight warmups and 32 samples per
size/reuse group. All 2,592 timed outputs match. Reuse order remains one then
eight, and windows are short; no publication-convergence gate passed.
[Raw phase observations](ar-0027-result-heavy-ingestion-2026-10-03.json)
retain the three run outputs.

Phase medians below are microseconds per complete sample. Execution,
serialization/validation and release aggregate all uses; preparation occurs
once. Phase medians do not necessarily sum to independently measured total.

| Offset | Items | Uses | Admission preparation | Execution | Serialization validation | Release | Total |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 5 | 1 | 16.00 | 4.60 | 9.35 | 5.30 | 36.55 |
| 0 | 5 | 8 | 14.45 | 25.90 | 51.65 | 8.95 | 101.90 |
| 0 | 50 | 1 | 169.90 | 30.50 | 74.80 | 31.30 | 322.70 |
| 0 | 50 | 8 | 172.00 | 258.55 | 594.45 | 97.65 | 1120.65 |
| 0 | 500 | 1 | 1365.10 | 469.80 | 631.65 | 272.25 | 2882.00 |
| 0 | 500 | 8 | 1208.00 | 2069.00 | 4668.80 | 629.20 | 8605.60 |
| 1 | 5 | 1 | 9.70 | 3.00 | 6.30 | 1.50 | 20.60 |
| 1 | 5 | 8 | 9.70 | 23.15 | 49.90 | 4.80 | 88.65 |
| 1 | 50 | 1 | 150.95 | 42.55 | 84.05 | 28.95 | 298.85 |
| 1 | 50 | 8 | 134.55 | 237.55 | 582.60 | 88.30 | 1038.30 |
| 1 | 500 | 1 | 1209.80 | 475.40 | 639.85 | 263.10 | 2627.85 |
| 1 | 500 | 8 | 1246.35 | 2115.45 | 4676.40 | 678.05 | 8861.05 |
| 2 | 5 | 1 | 10.05 | 3.30 | 6.30 | 1.40 | 21.30 |
| 2 | 5 | 8 | 10.05 | 26.10 | 50.30 | 5.00 | 92.50 |
| 2 | 50 | 1 | 72.35 | 22.45 | 56.00 | 10.10 | 161.20 |
| 2 | 50 | 8 | 83.05 | 182.10 | 449.25 | 40.25 | 771.90 |
| 2 | 500 | 1 | 1494.35 | 590.25 | 666.65 | 264.20 | 3019.65 |
| 2 | 500 | 8 | 1368.05 | 3030.10 | 5015.60 | 780.70 | 10489.80 |

At 500 items, single-use preparation medians are 1.21–1.49 ms, against
0.47–0.59 ms execution and 0.63–0.67 ms serialization/validation; total is
2.63–3.02 ms. At eight uses, preparation remains 1.21–1.37 ms but aggregate
serialization/validation rises to 4.67–5.02 ms and execution to 2.07–3.03 ms,
with 8.61–10.49 ms total. Setup cannot stand in for the whole lifecycle:
result processing becomes a major cost when setup is amortized. This supports
separate preparation and result-heavy experiments, not adopting an optimization.
Timing variability remains material, especially at the medium tier.

Reproduce after building the release test binary, then repeat fresh processes
with offsets 1 and 2, without simultaneous build/verification load:

```powershell
$env:AR0027_RESULT_ORDER = '0'
cargo test -p fastxslt --release --all-features measure_result_heavy_ingestion -- --ignored --nocapture --test-threads=1
```

Full gates pass with 1,541 core tests (51 ignored), native 18, WASM 3 and worker
18 tests, formatting, strict Clippy, docs, links and corpus inventories; official
WASM target checks pass. Filesystem hard-link fallback warnings remain.
The new private child owns this workload and its measurement only, under
`host_placement`; no source-unit extraction or public contract is selected.

The timing tranche did not measure allocator/retention accounting, cancellation,
bounded failure controls or host-visible result transfer. The follow-up below
addresses selected allocation and failure scopes. There is no
result-heavy capacity-candidate comparison here and no claim about RSS,
native/isolated throughput, 5,000-job pool capacity or OASIS percentage.

## Result heavy allocation and failure controls on 2026 10 03

Three fresh release processes produce identical requested allocation records
for six separate, non-nested scopes at each source size. The compiled program,
fixture bytes and expected output are constructed outside each scope. Preparation
is fresh in every scope; retained figures include the snapshot/prepared owners,
then the semantic copied result, then serialized output as applicable. These are
complete lifecycle prefixes, not incremental allocations to sum together.
[Raw allocation records](ar-0027-result-heavy-allocation-2026-10-03.json)
preserve all 54 observations.

| Items | Source bytes | Prepared retained bytes | With result retained bytes | With output retained bytes | Complete prefix peak bytes |
| --- | --- | --- | --- | --- | --- |
| 5 | 2,742 | 18,205 | 26,263 | 30,871 | 34,014 |
| 50 | 27,082 | 150,540 | 222,425 | 259,289 | 278,560 |
| 500 | 270,932 | 1,364,044 | 2,061,655 | 2,356,567 | 2,504,862 |

At 500 items, copied-result ownership adds 697,611 retained requested bytes;
the serialized string adds another 294,912. This is a baseline for measuring
result-heavy candidates, not evidence that shrinking prepared capacity alone
will remove result or serialization costs. Allocator requests are not RSS,
physical pages, ABI payload charges or a process-memory guarantee.

All 27 release/failure scopes across the three processes end with zero tracked
live allocations and bytes. Successful serialization accepts exactly the expected
output length. A ceiling one byte below that length returns `FXSR0002 / limit`;
cancellation at the second result-node charge returns `FXCT0001 / cancelled`.
Both preserve request correlation under the workbench diagnostic accessor.
The cancellation probe exercises partial copy teardown, not cancellation during
preparation or every possible execution phase. The compiled program remains
reusable after failures; these measurements do not prove same-prepared-owner
recovery or a full work-budget boundary matrix.

Reproduce three fresh processes after the release build, without concurrent gates:

```powershell
cargo test -p fastxslt --release --all-features measure_result_heavy_allocations -- --ignored --nocapture --test-threads=1
```

Full verification passes with 1,542 core tests and 52 ignored probes, plus native
18, WASM 3 and worker 18 tests. Formatting, strict Clippy, docs, local links,
corpus inventories and the official WASM target check pass. Filesystem hard-link
fallback warnings remain environmental. This private test child adds no production
layout, public API, unsafe surface, adapter or corpus credit. Result-heavy
capacity-candidate comparison and host-visible transfer accounting remain open.

## Result heavy capacity comparison on 2026 10 03

The private growth/freeze/pre-sizing comparison now covers the same 5/50/500-item
copied-result fixture. It admits fresh bytes into a sealed snapshot and constructs
a `Document` directly with the existing test-only candidate constructors. It does
not retain the `PreparedInputSet` maps and `Arc` wrappers measured above, so compare
candidates within this matrix rather than subtracting its totals from that probe.
The candidate path does not include the separately investigated capacity checkpoint
polls. Production preparation and its defaults remain unchanged.

Three fresh timing processes rotate candidate order. Each candidate/size/use group
has eight warmups and 32 samples, at one and eight uses. All 7,776 timed outputs
match the independently generated expected XML. The ordinary control adds 81 exact
outputs, including nine serialized after source retirement, and conserves all ten
work-domain counters across the three constructors. Allocation is measured separately
in complete, non-nested one-use scopes. All 18 allocation records reproduce identically
in three fresh processes; all 27 released scopes end with zero tracked bytes and
allocations. [Raw comparison records](ar-0027-result-heavy-capacity-2026-10-03.json)
retain phase timing and requested-allocation details.

| Items | Candidate | Source result output retained bytes | Complete prefix peak bytes | Cumulative requested bytes |
| --- | --- | --- | --- | --- |
| 5 | Growth | 29,881 | 33,022 | 66,446 |
| 5 | Freeze | 26,409 | 29,550 | 70,910 |
| 5 | Pre-sizing | 26,409 | 29,550 | 56,030 |
| 50 | Growth | 258,297 | 277,566 | 569,204 |
| 50 | Freeze | 232,753 | 252,022 | 607,148 |
| 50 | Pre-sizing | 232,753 | 252,022 | 481,164 |
| 500 | Growth | 2,355,573 | 2,503,866 | 5,656,485 |
| 500 | Freeze | 2,220,413 | 2,368,706 | 6,029,229 |
| 500 | Pre-sizing | 2,220,413 | 2,368,706 | 5,014,413 |

At 500 items, both candidates save 135,160 retained requested bytes, or 5.74% of
the complete retained source/result/output scope; peak falls 5.40%. Freeze adds a
resize and increases cumulative requests by 372,744 bytes. Pre-sizing instead
reduces cumulative requests by 642,072 bytes. Result and output ownership dilute
the prepared-tree saving: this is not a 5.74% process-memory or throughput claim.

Large one-use total medians reverse with run order: growth is 2.62/1.67/2.66 ms,
freeze 1.63/2.35/1.58 ms and pre-sizing 1.69/1.62/2.94 ms. Eight-use totals are
7.48/8.08/7.57 ms, 6.91/7.78/7.52 ms and 7.30/7.29/8.01 ms respectively.
Short windows, fixed size/use order and no convergence gate prevent a speedup,
stable break-even or adoption claim. Memory savings reproduce; timing does not
select a winner. Broader failure/checkpoint parity, adapter placement and retention,
and official-target candidate execution remain separate follow-ups.

Reproduce three fresh processes with `AR0027_RESULT_ORDER` set to 0, 1 and 2:

```powershell
cargo test -p fastxslt --release --all-features measure_result_heavy_capacity_lifecycle -- --ignored --nocapture --test-threads=1
cargo test -p fastxslt --release --all-features measure_result_heavy_capacity_allocations -- --ignored --nocapture --test-threads=1
```

Full verification passes with 1,543 core tests and 54 ignored probes, plus native
18, WASM 3 and worker 18 tests; official WASM compilation also passes. The new
private child owns one candidate-comparison workload and remains below source-size
pressure. Its host-placement directory has five direct Rust test files, below
ADR-0004's density-review threshold. No public contract, layout decision, unsafe
exception or corpus credit follows.

## Result heavy capacity control parity on 2026 10 03

The private candidate-control child adds ordinary regressions for the same
5/50/500-item copied-result sources. Production constructors remain unchanged;
these tests compare existing growth/freeze/pre-sizing experiments with and without
their uncharged checkpoint polls. This is control evidence, not another timing run.

The construction matrix contains 108 differential pairs: three source sizes,
three constructors, zero/one-less/exact XDM-node limits and four cancellation
positions (none, charge zero, charge one and final-node charge). Expected success
is asserted independently; both paths must produce the same failure or matching
capacity/string-value result and consume the same XDM-node work. XML parsing is
outside the constrained construction control, so this does not establish XML-phase
budget or cancellation parity.

Twelve additional controls signal cancellation immediately before or after the
pre-sizing reserve or freeze resize. All return the exact XDM cancellation failure
without publishing a document. Pre-sizing consumes one node charge at that point;
freeze consumes the full node count. Fresh reconstruction from the unchanged input
then succeeds. These deterministic observer controls do not measure elapsed
cancellation latency or interrupts inside an allocator operation.

The invocation matrix contains 162 comparisons across result-node, result-text-byte
and serialized-byte domains, exact/one-less/cancelled controls, three sizes and six
constructor/poll combinations. All outcomes equal the growth reference, including
the complete structured failure and consumed work in the targeted domain. Under
the workbench accessor, exhaustion is `FXCT0002 / limit`, cancellation is
`FXCT0001 / cancelled`, and request correlation remains `capacity-controls`.
All 162 invocation scopes return tracked allocations and bytes to zero; prepared
documents and compiled programs are outside those scopes. This is invocation
cleanup evidence, not complete construction-failure allocation accounting.

After each controlled invocation, an unbounded fresh invocation executes and
serializes exactly against the **same** prepared document. All 162 recovery outputs
match. That closes the earlier same-source recovery gap for this workload without
claiming general cancellation, host containment or full standards coverage.

Reproduce the focused ordinary tests:

```powershell
cargo test -p fastxslt --all-features result_heavy_ -- --test-threads=1
```

The focused set passes six ordinary tests with five manual probes ignored. Full
verification passes with 1,546 core tests and 54 ignored probes, native 18, WASM 3
and worker 18 tests, formatting, strict Clippy, docs, links and corpus inventories.
Official WASM compilation passes; it does not execute these host tests in WASM.
The six-file host-placement directory remains below ADR-0004 density pressure;
the private child owns construction/invocation control parity only. No unsafe,
ABI, public lifecycle or production capacity policy changes. Adapter retention,
representative host measurements and checkpoint publication/adoption review remain
open, and OASIS credit is unchanged.

## Construction failure allocation cleanup on 2026 10 03

Fresh complete XML/XDM scopes now cover construction failures, not just execution
over already prepared storage. Fixtures and the differential reference are created
outside each scope; XML parsing, XDM construction, invocation control and teardown
are inside. No sealed snapshot, prepared map, host framing or output is retained.
XML parsing remains unbounded in this probe; the constrained domain is XDM nodes.

Each run contains 216 construction scopes: three sizes, three capacity candidates,
polls enabled/disabled, zero/one-less/exact limits and four charge-cancellation
positions. Each outcome and consumed XDM work equals the non-polled reference;
198 fail and 18 succeed as independently expected. Twelve further scopes cancel
before/after reserve or freeze checkpoints and verify exact cancellation without
publishing a document. Every scope ends at zero tracked live allocations and bytes.
The ordinary regression enforces this matrix, not only the manual probe.

Three fresh release processes reproduce all 30 summary/resize records identically:
630 failures and 54 successful constructions release completely. The largest peak
among each lane's 12 independent construction scopes is 22,285 / 186,885 / 1,878,287
requested bytes for 5/50/500 items respectively, equal across candidates and polls.
These maxima include the success case and XML parsing; they do not isolate XDM
peak, demonstrate equal construction cost or measure cancellation latency.
[Raw observations](ar-0027-construction-failure-allocation-2026-10-03.json)
preserve the fresh-process outputs.

At 500 items, reserve-checkpoint cancellation has a 1,248,742-byte peak before
reserve and 1,433,615 after. Freeze cancellation has the same 1,878,287-byte peak
before and after resizing, although cumulative requested bytes rise from 3,460,307
to 3,833,051. Peak and cumulative allocation are distinct; unchanged peak does not
mean that a resize costs nothing. All failure ownership is released, without an
RSS/reclamation-time or allocator-interruption guarantee.

Reproduce the ordinary regression or repeat the manual release probe in three
fresh processes:

```powershell
cargo test -p fastxslt --all-features result_heavy_construction_and_resize
cargo test -p fastxslt --release --all-features measure_result_heavy_construction_allocations -- --ignored --nocapture --test-threads=1
```

Full verification passes with 1,547 core tests and 55 ignored probes, native 18,
WASM 3 and worker 18 tests; official WASM compilation also passes. Local links
and diff checks pass. The private allocation child adds a seventh direct Rust test
file to host placement, below ADR-0004 density pressure. No production preparation,
public lifecycle, unsafe, ABI, authority or corpus policy changes. The measured
result-heavy construction-failure accounting gap is closed; broader XML failure
profiles, checkpoint publication/adoption review and host-side retention remain open.

## Candidate publication boundary review on 2026 10 03

Code review distinguishes three boundaries: the constructor's cancellation
observation, return of a complete owned document, and caller publication/admission.
They are not one atomic operation. Freeze polls after its resize and returns
`Ok(document)` if the token is clear at that observation. A signal after that
poll may be unobserved, including a signal before the caller publishes the owner.
The previous checkpoint evidence now calls this a final observation rather than
a publication fence. No new synchronization or production completion rule follows.

Two private ordinary tests cover the result-heavy sizes. Twelve replacement
controls signal before/after reserve or freeze, reject construction and preserve
the old published owner and its lease. A later successful replacement has a
distinct document origin. Both old and new copied results remain exact while
owners overlap; a weak reference confirms old storage expires when its final
lease drops. Each case validates four outputs, for 48 exact outputs. This test
caller publishes only successful construction; it is not an adapter registry,
host transaction or automatic replacement mechanism.

Eighteen post-return controls span three sizes, three constructors and polls
enabled/disabled. Cancellation is deliberately signalled after constructor return
but before sharing through `Arc`. Reusing that cancelled control for execution
returns `FXCT0001 / cancelled` with the same request identity. A fresh invocation
still executes the immutable source exactly, including after the original owner
drops and its lease remains. All 36 outputs match; final weak references expire.
Returned storage is not revoked by later cancellation. These controlled orderings
are not natural race-frequency measurements and do not prove a linearization
point between final poll and return.

The candidate recommendation is to preserve cooperative observation semantics:
fail and release construction when cancellation is observed before successful
handoff; do not invalidate already returned source storage; let subsequent work
observe its own supplied control. Caller admission/publication remains separate
under ADR-0005 and ADR-0016. An atomic cancel-versus-publication promise would need
an explicit contract and synchronization design, not another boolean check.
This recommendation does not select a supported completion rule or capacity policy.

The remaining adoption gate is complete adapter retention/admission evidence,
including overlapping prepared/compiled generations, exact registry charges and
release, transient construction peak, denied replacement preserving old leases,
and downstream host cancellation/failure behavior. Existing adapter paths still
use production growth construction. A test caller's `Arc`/weak-owner proof does
not substitute for native/isolated/WASM publication or a supported Rust facade.

Reproduce the two focused controls:

```powershell
cargo test -p fastxslt --all-features publication_tests -- --test-threads=1
```

The private child owns candidate handoff and lease observations only. Eight direct
Rust files in host placement remain below ADR-0004 directory-density pressure.
No production constructor, scheduler, registry, public API, authority or unsafe
surface changed. OASIS counts and capacity adoption remain unchanged.

Full verification passes with 1,549 core tests and 55 ignored probes, plus native
18, WASM 3 and worker 18 tests, formatting, strict Clippy, docs, links, corpus
inventories and the official WASM build check. The host-native controls do not
establish execution of these candidate tests on WASM. Filesystem hard-link fallback
warnings remain environmental.

## Adapter retention and admission baseline on 2026 10 03

The private .NET CLI probe now runs the same generated copied-result sources
through existing native and isolated boundaries, without a new export, opcode
or preparation option. Both adapters still use production growth construction.
Three fresh .NET 10.0.12 processes reproduce all ten quiescent native ownership
checkpoints and 39 exact output checks (18 native, 21 isolated).
[Raw adapter reports](ar-0027-adapter-retention-admission-2026-10-03.json)
preserve limits, counts and exact retained payload bytes.

The native probe explicitly supplies limits of two engines, zero registered
controls, three outcomes and 1,048,576 outcome bytes. Known-engine-capacity and
aggregate-accounted-byte limits opt out using maximum values. These are engineered
test limits, not recommended deployment defaults. Creation can transiently use
the third outcome slot while two delayed results remain live. Observation uses
only ADR-0015's existing four scalar exports at quiescent checkpoints, not an
atomic snapshot or new metrics interface.

| Native checkpoint | Engines | Outcomes | Exact payload bytes |
| --- | --- | --- | --- |
| Old generation and delayed result | 1 | 1 | 2,759 |
| Old/new generations and delayed results | 2 | 2 | 273,708 |
| Third engine denied | 2 | 2 | 273,708 |
| Malformed initialization or cancelled invocation drained | 2 | 2 | 273,708 |
| Old engine released, result survives | 1 | 2 | 273,708 |
| Released engine slot readmitted | 2 | 2 | 273,708 |
| Both engines released, results survive | 0 | 2 | 273,708 |
| Old result released | 0 | 1 | 270,949 |
| All ownership released | 0 | 0 | 0 |

The third engine returns exact `FXFFI0102 / resource-exhausted`, consumes no
ordinary outcome slot and does not evict either valid engine. Malformed XML
initialization returns an invalid failure; a pre-cancelled invocation returns
`FXCT0001 / cancelled`. Both failure deliveries release their transient outcomes.
Old/new engines still execute exactly. Releasing the old engine permits admission
of a 50-item engine, proving capacity recovery through the managed boundary.
Delayed byte outcomes stay independently readable after both engine handles drop.
All registered control counts remain zero; this is not active-control concurrency
or cancellation-race evidence.

The isolated lane retains two independent worker generations, preserves the
current generation after failed reinitialization, then disposes old and new workers
while already transferred result strings remain exact. Worker IDs do not change
during the live checks; the code performs no implicit replacement or retry.
These are host-owned managed result copies, not worker-resident outcome handles.
The isolated lane has no equivalent process-wide engine-count quota. Do not equate
its string retention with native registry payload charges or infer identical
generation topology from these two lanes.

Reproduce in a fresh process after the managed/native builds and DLL copy shown
in the workbench README:

```powershell
dotnet workbenches/FastXSLT.AspNet.Workbench/bin/Release/net10.0/FastXSLT.AspNet.Workbench.dll --retention-admission-probe
```

Managed Release build passes with zero warnings/errors; the shared native library
and worker Release build passes. Normal restore required access to the existing
user NuGet configuration and passed without changing audit settings. No Rust code
changed in this tranche; previous full Rust/WASM gates remain applicable, and
local links/diff checks pass. The private C# owner remains below source-size
pressure; creation measurement now has six direct C# files, below directory-density
review pressure. No public API, unsafe or
authority surface changes.

This closes a current-path ownership/admission baseline, not the candidate-adoption
gate. Exact engine capacity charges, transient compilation/preparation peak,
candidate construction through complete adapter lifecycles, active concurrent
leases and WASM runtime retention still need separate evidence. No RSS, throughput,
production quota, supported completion rule or OASIS credit follows.

## Private native byte admission controls on 2026 10 03

The managed baseline above deliberately opts out of known-engine and aggregate
byte ceilings. Four private native registry tests now cover those dimensions
without adding an engine-size export or changing the production estimator.
Generated 5/50/500-item copied-result fixtures use the current shared engine,
normal preparation and independently constructed exact expected XML.

Eighteen exact/one-less scenarios exercise three distinct admission operations:
overlapping engine generations under a known-capacity limit; replacement engine
admission while an old result payload remains retained under the aggregate limit;
and result admission while its engine remains retained under the aggregate limit.
The limits come from each engine's existing private retention estimate plus exact
result lengths, not empirical RSS or a recommended deployment number.

At the exact ceiling admission succeeds. One byte less returns the precise tagged
engine-capacity or total-accounted-byte status. Rejected creation publishes neither
an engine nor a creation outcome; retained engines remain usable with exact output.
Release restores capacity, allows readmission and returns every recorded charge
to zero. A retained copied result remains exact after its engine is released.

Six concurrent last-capacity scenarios race two prepared engines against space for
only one additional generation, separately for known capacity and total accounted
bytes. Count limits are deliberately non-binding. Exactly one wins, exactly one
receives the relevant byte-exhaustion tag, and the winner's creation outcome is
published atomically with its engine. Old and winning generations execute exactly;
final release returns registry counts and both byte charges to zero.

Reproduce the four focused tests:

```powershell
cargo test -p fastxslt-dotnet-workbench retention_admission_tests
```

The cohesive private child module owns only these registry admission/release
controls and depends on existing safe registry helpers and shared engine behavior.
It adds no production field, export, unsafe block, policy dimension or alternative
engine path. Admission still occurs after compilation/preparation, so these
assertions do not bound construction peak, allocator retention or process memory.
They do not exercise compact constructors through the adapter or establish active
lease/cancellation races, WASM runtime retention or a supported public lifecycle.

All `scripts/verify.ps1` gates pass: formatting, strict Clippy, 1,549 core tests,
22 native tests, three WASM adapter tests, 18 worker tests, documentation, local
Markdown links and pinned corpus inventories. The official
`wasm32-unknown-unknown` check also passes; it does not execute native registry
controls on WASM. The existing Windows incremental-cache hard-link fallback
warnings remain environmental. Production construction and corpus credit are
unchanged.

## Concurrent result heavy generation leases on 2026 10 03

A private direct-Rust control shares two independently prepared document
generations across four threads. The matrix includes 5/50/500-item sources,
growth/frozen/pre-sized construction, and both checkpoint-polling settings:
18 generation-pair scenarios. Both caller-owned document handles are dropped
before workers leave the start barrier; each document then has exactly four
worker leases. This establishes overlap eligibility, not measured simultaneous
CPU occupancy or a concurrency throughput claim.

Each worker executes both generations with its own invocation control. The four
roles exercise an exact result-node budget, a one-less serialized-byte budget,
charge-indexed cancellation during copying, and a one-less result-text budget.
The growth oracle and candidate agree on all 144 controlled outcomes and all
ten work-domain charges. Failure controls preserve exact `FXCT0001 / cancelled`
or `FXCT0002 / limit` diagnostics, request identity and complete failure equality.
No invocation token or mutable runtime frame is shared with siblings.

All 144 candidate invocations then recover with fresh controls on the same
immutable documents and produce independently expected exact XML. Their serialized
strings remain owned by the test caller after worker completion. Both document
weak owners expire after the last lease drops, and every retained output still
compares exactly. This proves serialized result ownership, not retention of a
semantic result tree, native registry handle or isolated-worker response.

Reproduce:

```powershell
cargo test -p fastxslt --all-features result_heavy_shared_candidates
```

The new private child owns concurrent invocation/lease controls only. It calls
the existing constructor, evaluator, serializer and invocation-control helpers;
it adds no scheduler, adapter option, production constructor or public type.
The host-placement directory now has nine direct Rust files, below ADR-0004's
directory-density review threshold. This test does not measure allocation peaks,
process memory, natural cancellation race frequencies, WASM runtime concurrency
or compact construction through native/isolated adapter lifecycles. Those scopes
remain separate from the candidate's direct-Rust concurrency proof.

Final validation passes all `scripts/verify.ps1` gates with 1,550 core tests,
22 native tests, three WASM adapter tests and 18 worker tests, plus formatting,
strict Clippy, documentation, local links and pinned corpus inventories.
The official WASM build check passes but does not run this threaded host-native
test on WASM. Windows incremental-cache hard-link fallback warnings remain
environmental. No corpus numerator or production behavior changes.

## Shared prepared set capacity seam on 2026 10 03

The capacity candidates now enter the real `PreparedInputBuilder` through a
private `cfg(test)` selector. Snapshot resolution, XML parsing, failure projection,
prepared-map insertion and sealing remain the existing shared path. The selector
changes only which existing safe document constructor runs after parsing.
Ordinary tests default to growth; production builds contain neither the selector
nor its builder field and continue calling `Document::from_parsed_controlled`.
Phase observation remains optional and selects growth, preserving the existing
measurement control. No adapter opcode, exported type or runtime configuration
is added.

Two ordinary tests cover all three constructors and 5/50/500-item sources.
Nine old/new prepared-set pairs preserve raw admitted bytes, parsed-phase
capacity, node counts and separate document origins. Candidate XDM capacity does
not exceed the growth reference in these fixtures. Eighteen exact copy outputs
remain executable after both prepared maps retire while explicit document leases
and snapshot clones keep their own storage alive. Document weak owners expire
when those leases drop; snapshot ownership does not pin the document map.

Twenty-seven preparation cases compare exact XDM-node limits, one-less limits
and charge-indexed cancellation against growth, preserving complete preparation
failure equality and XML-event/XDM-node charges. The eighteen failures retain
neither a document entry nor parsed-phase metadata. Retrying each failed case on
the same builder with a fresh control succeeds, after which sealing publishes
the expected node count. These assertions do not imply allocator reclamation or
bounded process-memory peaks.

Reproduce the two new controls:

```powershell
cargo test -p fastxslt --all-features prepared_set_capacity_tests
```

ADR-0004 cohesion inspection retains the 1,188-line preparation owner at this
checkpoint: its responsibility remains snapshot-to-prepared-set construction,
while the 146-line child owns candidate publication/recovery tests. Dependency
direction is child-to-existing preparation/runtime owners, with no parent
pass-through for unrelated state. The host-placement directory has ten direct
Rust files; another file would trigger a directory-subject review. Revisit this
placement before expanding candidate selection into full engine construction.

This is the shared preparation seam needed for later engine-lifecycle experiments,
not execution of a candidate through P/Invoke, isolated framing or WASM runtime.
Full-engine compilation/coexistence peaks, adapter retention/admission and any
production capacity decision remain open. No timing or corpus credit follows.

All verification gates pass with 1,552 core tests, 22 native tests, three WASM
adapter tests and 18 worker tests, plus formatting, strict Clippy, documentation,
local links and pinned corpus inventories. The official WASM target check passes;
it does not run test-only capacity selection on WASM. Existing Windows incremental
hard-link fallback warnings remain environmental.

## Complete Rust engine capacity controls on 2026 10 03

Capacity candidates now run through the actual engine construction composition:
owned resource admission, sealed snapshot, stylesheet compilation, prepared-set
construction and retained invocation lifecycle. The choice is confined to a
private `cfg(test)` construction option. Existing entry points and DTD experiments
select growth by default. Production contains no capacity option, new export,
worker opcode or host configuration setting.

Three ordinary tests cover 5/50/500-item copied-result sources and all three
constructors. Nine old/new engine pairs perform eight exact byte transforms per
generation through engine leases after the caller releases both engine owners.
All 144 results remain exact after the last engine leases expire. Known snapshot,
prepared-map and non-XDM engine charges match growth; only the prepared XDM
capacity may decrease. These are estimator assertions, not allocator measurements.

Twenty-seven failed creation attempts compare malformed source, malformed
stylesheet and one-less source XDM-node limits against growth using complete
projected failure equality. Existing engines remain exact and fresh replacement
creation recovers. Eighteen invocation controls compare pre-cancellation and zero
instruction budgets against growth, including exact `FXCT0001 / cancelled` and
`FXCT0002 / limit` projection. Fresh invocations on the same retained candidate
engine recover exactly. This is not mid-execution cancellation-race evidence.

Reproduce:

```powershell
cargo test -p fastxslt --all-features construction::capacity_tests
```

### Construction ownership and decomposition checkpoint

The 2,119-line workbench facade triggered ADR-0004 review before this extension.
Construction now belongs to a private child under the facade's named directory.
The parent falls to 1,869 lines, retaining invocation execution, diagnostic
projection, retention observation, public experimental types and existing tests.
The construction child owns byte admission, dependency sealing, compilation and
preparation composition; its test child owns complete-engine candidate controls.
No runtime scheduling, registry mutation, foreign memory or acquisition authority
enters construction. It calls existing resource/compiler/preparation owners and
parent projection helpers; it does not add pass-through execution methods or a
shared mutable runtime context. The runtime directory's direct file count does
not increase; the new source children are grouped under their existing owner.

The mechanical extraction checkpoint reproduced all 37 ordinary pre-existing
engine tests before candidate controls were introduced, with the existing manual
allocator probe still ignored. Subsequent test-only option selection is a separate
experimental extension, not hidden inside the mechanical move. Public constructor
signatures, diagnostic identities, resource bounds, DTD default denial, unsafe
surface and memory-resident ownership remain unchanged. No clean-build speedup
or representation adoption follows from smaller source files.

This closes complete Rust engine functional integration of the candidates.
Allocator peak/retention across overlapping complete engines, native/isolated
candidate transport and registry admission, host timing, WASM runtime behavior
and any production capacity/completion decision remain open.

Final validation passes all `scripts/verify.ps1` gates, including formatting,
strict Clippy, workspace tests, documentation, local links and pinned corpus
inventories. The official WASM build check passes with unchanged production
selection; it does not execute the candidate engine tests on WASM. Existing
Windows incremental hard-link fallback warnings remain environmental.

## Complete engine overlap allocation on 2026 10 03

Seven fresh, non-nested allocation prefixes now cover complete engine creation
and ownership: one old engine; old/new overlap; both engines plus two delayed
serialized outputs; outputs after engine retirement; complete release; failed
replacement with old-engine recovery; and pre-cancelled invocation with recovery.
Each scope includes engine-owned source/stylesheet copies, resource admission,
compilation and preparation. Caller-generated fixture strings are outside the
scope. There is no managed adapter, native registry, pipe or OS memory observation.

Three fresh release processes reproduce all 63 records identically, for 189
recorded scopes. All 81 complete-release/failure/cancellation scopes return to
zero current allocation count and requested bytes. The recorded runs validate
216 exact outputs. An ordinary allocation-feature regression repeats the same
ownership assertions without printing measurements.
[Raw records and complete process output](ar-0027-complete-engine-overlap-allocation-2026-10-03.json)
preserve estimator charges separately from requested allocation counts/bytes.

### Observed 500 item ownership scopes

| Ownership prefix | Constructor | Retained requested bytes | Peak requested bytes |
| --- | --- | --- | --- |
| Two engines | Growth | 2,743,122 | 3,526,739 |
| Two engines | Frozen | 2,472,802 | 3,391,579 |
| Two engines | Pre-sized | 2,472,802 | 3,028,379 |
| Two engines and delayed outputs | Growth | 3,285,040 | 4,258,131 |
| Two engines and delayed outputs | Frozen | 3,014,720 | 3,987,811 |
| Two engines and delayed outputs | Pre-sized | 3,014,720 | 3,987,811 |
| Only delayed outputs | All three | 541,918 | Full-prefix peak above |

Both candidates save 270,320 retained requested bytes in the two-engine scope,
or 9.85%. Pre-sizing lowers its peak 14.13%, versus 3.83% for freezing. Once
delayed outputs are included, both reduce retention 8.23% and peak 6.35%.
Pre-sizing still requests fewer cumulative bytes: 10,542,722 versus growth's
11,826,866 and freezing's 12,572,354 in that full prefix. A lower prepared-build
peak does not imply a correspondingly lower result-construction peak.

Two output payloads own 541,822 exact bytes; their retaining vector accounts for
the additional 96 requested bytes in the outputs-only prefix. Engine known-capacity
charge is then zero, and all three candidates retain the same output allocations.
Requested bytes are not RSS, and prefix differences do not identify the exact
instant or internal operation at which a peak occurs.

At 5/50 items, two-engine retained savings are 15.25%/16.44%, while pre-sized
construction-prefix peak reductions are 19.88%/21.56%. These generated copied-result
shapes are not a universal workload distribution or proof of a throughput gain.
No timing is recorded, no convergence gate is selected and candidate order is fixed.
The independent scope/release checks retain the complete growth engine as oracle.

Reproduce after building the release test binary:

```powershell
cargo test -p fastxslt --release --all-features measure_complete_engine_overlap_allocations -- --ignored --nocapture --test-threads=1
```

The cohesive 171-line private allocation child depends only on existing complete
engine fixture/constructor helpers. It adds no production selector, ABI, unsafe
surface, allocator policy or engine backend. Full gates pass with 1,556 core tests
and 56 ignored manual probes, 22 native tests, three WASM adapter tests and 18
worker tests, including formatting, strict Clippy, docs, links and corpus inventories.
The official WASM build check passes but does not execute this host-native
allocation experiment on WASM. Windows incremental hard-link warnings remain
environmental. Complete host candidate timing/admission, other source shapes and
production adoption remain open; corpus credit and production behavior are unchanged.

## Complete engine capacity timing on 2026 10 03

Three fresh release processes compare growth, freezing and pre-sizing through
the complete Rust engine lifecycle. Candidate and source-size order rotate;
one/eight-use order also changes. Each process records 36 cells with eight
warmups and 32 timed samples per cell, producing 144 phase records. All 15,552
timed new-engine outputs compare exactly. The ordinary regression covers the
same 36 combinations, including survival of the existing generation.
[All 432 emitted timing records](ar-0027-complete-engine-capacity-timing-2026-10-03.json)
preserve per-process medians and p95 values. The initial build/smoke run is excluded.

Creation includes source/stylesheet owned copies, admission, sealing, compilation
and XML/XDM preparation. Transform time includes execution, serialization and
the byte result, but excludes output validation. Release sums output drops and
new-engine retirement. Total includes the complete new lifecycle, validation
and clock bookkeeping; separately computed phase medians need not sum to its
median. Fixture generation is outside timing. In overlap cells, an old growth
engine is constructed before the clock and verified/retired afterward; only
its retained state overlaps the measured new lifecycle, not concurrent execution.

Allocation observation is disabled for these timing processes. This is the
`workbench` feature profile, not the allocation-instrumented all-feature binary.
It measures no managed adapter, registry, pipe, process startup or host acquisition.
The existing feature-specific test-helper warnings in that build do not fail
compilation; the independent strict all-feature verification passes.

### Complete lifecycle candidate to growth ratios

Ratios below divide candidate median total time by growth median total time
within each fresh process, in process order 0/1/2. Less than one means a lower
observed candidate median, not an established speedup.

| Items per source | Uses | Old engine retained | Frozen ratios | Pre-sized ratios |
| --- | --- | --- | --- | --- |
| 5 | 1 | No | 0.941 / 1.006 / 1.003 | 0.957 / 0.957 / 1.301 |
| 50 | 1 | No | 0.921 / 0.971 / 0.858 | 0.758 / 1.012 / 1.014 |
| 500 | 1 | No | 0.738 / 1.039 / 0.957 | 0.731 / 0.987 / 1.037 |
| 500 | 1 | Yes | 1.126 / 0.872 / 0.800 | 1.016 / 0.968 / 1.177 |
| 500 | 8 | No | 0.977 / 1.002 / 0.970 | 1.041 / 0.964 / 0.965 |
| 500 | 8 | Yes | 1.036 / 0.881 / 1.058 | 1.154 / 0.907 / 1.008 |

The 500-item single-use growth creation medians without overlap range from
767.7 to 1,460.3 us. Its complete lifecycle medians range from 1,646.8 to
3,066.3 us. This variability and repeated candidate reversals prevent a general
timing benefit, precise penalty or reuse break-even claim. The 50-item frozen
single-use cell improves in all three repeats, but that does not generalize
to overlap, other sizes or pre-sizing. Short adjacent candidate windows have no
steady-state convergence or cross-machine gate; every record remains
`publication_eligible=false`.

The preceding allocation savings remain separately reproducible. These timings
do not invalidate those ownership measurements, nor do the memory savings prove
better latency. Candidate selection remains test-only and production remains
growth. Full verification and the official WASM build check pass; this host-native
timing test does not measure WASM runtime behavior. Representative host candidate
creation/admission, broader shapes and any adoption decision remain open.

## Host shape ingestion baseline on 2026 10 03

The unchanged production-growth engine now has native .NET and persistent
isolated ingestion controls across six synthetic copy shapes. Three fresh
.NET 10.0.12 processes each execute a mixed 5,000-job queue per lane, validating
30,000 timed exact results overall. Logical source identities are distinct;
the six payloads repeat. This is not a real publication distribution, distinct
content per job, compile-once ingestion or a capacity-candidate comparison.
[Raw reports](ar-0027-host-shape-ingestion-2026-10-03.json) retain medians, p95,
minimum/maximum, source/result sizes, preceding shape and first-lane counts.

Each job creates a replacement engine, retaining its predecessor until successful
initialization, then returns one copied result. Both lanes recompile and prepare;
the isolated worker stays alive across the queue. Host fixture generation and
identity assembly are outside timing. Native creation combines encoding, copies,
admission, compilation and preparation. Isolated readiness combines decode,
engine creation and old-engine retirement; it is not a parse-only measurement.
Through-result clocks include validation and the existing result transfer.

First-lane scheduling now alternates within each shape across repeated fixture
cycles. Merely alternating by job had pinned each shape in an even-length cycle
to one first lane. Every recorded group now has native-first/isolated-first counts
differing by at most one, across 833/834 samples per shape/process. Pre-balance
runs and 128-job smokes are excluded from the recorded timing table. The original
decimal ingestion control also passes a 128-job smoke after shared orchestration
was factored, with unchanged exact output and diagnostic comparisons.

### Observed median ranges across three fresh processes

Values are microseconds, with minimum/maximum process medians, not confidence
intervals or publication-quality performance claims.

| Shape | Native creation | Native through result | Isolated initialization | Isolated through result |
| --- | --- | --- | --- | --- |
| Wide | 1,413.0–1,435.8 | 2,260.0–2,311.9 | 1,583.5–1,643.9 | 2,520.1–2,590.9 |
| Deep 64 | 74.7–79.9 | 262.8–266.9 | 273.1–286.0 | 403.9–420.2 |
| Attribute heavy | 658.7–678.9 | 919.2–945.6 | 776.8–791.4 | 1,129.5–1,159.3 |
| Text heavy | 275.5–281.9 | 1,033.4–1,060.8 | 479.7–487.4 | 1,383.9–1,401.0 |
| Namespace heavy | 512.2–528.8 | 811.4–839.7 | 661.4–670.1 | 1,068.9–1,101.2 |
| Low repetition | 704.2–731.1 | 1,152.7–1,173.3 | 877.6–900.8 | 1,419.5–1,463.3 |

Text-heavy source/result payloads are 262,983 bytes. Attribute and namespace
sources retain their original empty-element syntax; copied results use explicitly
closed elements, with expected output pinned independently. No validation or
normalization cost differs between lanes, and no upstream fixture is changed.

Retirement belongs to the preceding shape, not the new one. For example, deep-64
follows wide: retiring wide takes native medians of 124.7–126.4 us, more than
creating deep-64. The report names that predecessor rather than crediting the
old-generation cost to new source preparation. There are no retained-capacity,
managed-allocation, allocator-peak or RSS observations in this probe.

### Depth classification finding and conservation

The baseline adapters enforce depth 64; the timed deep source therefore differs
from the earlier direct-Rust 256-deep anatomy fixture. A separate well-formed
256-deep rejection conserves exact native/isolated code, category, request,
location and detail, and leaves the old engine usable. It exposes a classification
gap: `FXXM0002 / invalid`, at source span 192..195, wrapped `DepthLimit` instead of
reporting a resource-limit outcome. Before the repair below, shared preparation
projected all non-control XML parser failures into `PreparationFailure::InvalidXml`.
Adapter agreement was not evidence that this classification was correct. The
closeout repair below retains malformed-XML distinction without changing capacity
selection; no engine diagnostic change was hidden in the original timing probe.

Each fresh process also verifies malformed-source and default-DTD-denial parity,
seven same-generation recovery outputs, field-limit rejection before framing,
lost-worker retirement without retry, and one explicit fresh-worker recovery.
The main worker PID remains unchanged and native registry ownership returns to
baseline. No candidate selector, public API, ABI export, protocol field, unsafe
operation or production policy is added. All records are non-publication-eligible.

Reproduce after the existing native/worker release build and native DLL copy:

```powershell
dotnet workbenches/FastXSLT.AspNet.Workbench/bin/Release/net10.0/FastXSLT.AspNet.Workbench.dll --shape-ingestion-probe --ingestion-jobs=5000 --creation-order=0
```

Repeat in fresh processes with orders 1 and 2. The private fixture child owns
only synthetic bytes and expected serialization; shared ingestion orchestration
owns clocks, replacement and cleanup. The units are 40 and 178 lines respectively,
and `CreationMeasurement` has seven direct files. This is no new semantic owner
or public source representation. .NET Release build and Rust conservation gates
pass; production preparation and corpus credit remain unchanged. Candidate host
selection/admission still requires a deliberate private measurement seam, and
structural-limit diagnostic repair was the immediate follow-up at that checkpoint.

## Bounded capacity campaign closeout on 2026 10 03

At the owner's direction, conclude the current AR-0027 study with production
adoption deferred. Node-vector capacity tightening has reproducible memory benefit
and substantial differential/lifecycle evidence, but no general speedup or reuse
break-even. Host measurements and executed WASM use production growth; they do
not establish candidate host admission or target-width capacity behavior. Retain
the safe private candidates and growth oracle. Do not add selectors, protocol
fields, staging layers, interning or new layout experiments merely to extend the
study. Reopen only for named consumer pressure and the proof needed to decide it.

The structural-limit diagnostic defect is repaired independently. Shared
preparation now distinguishes recognized parser depth/event/DTD limits from
invalid XML, after preserving cooperative-control precedence. It retains the
original logical source identity/span and projects `FXRS0006 / limit`, matching
the existing stylesheet structural-limit convention. Malformed source XML
remains `FXXM0002 / invalid`; no display-string matching determines category.
No work charge, parser ceiling, DTD authority or capacity default changes.

One prepared-set regression covers growth/freezing/pre-sizing at six one-less
depth/event boundaries, six pre-cancelled controls, six same-builder recoveries
without partial map entries and three exact-boundary successes. Complete-engine
controls additionally check each candidate's depth-five success/depth-four
rejection, exact original span, malformed-source distinction and continued use
of the old generation. The WASM adapter has an ordinary depth-limit regression.

The final native/isolated 128-job host smoke validates 256 timed exact results,
seven same-generation recoveries and explicit fresh-worker recovery. Both lanes
agree on the repaired depth diagnostic and retain span 192..195. The main worker
PID stays fixed and native registry ownership returns to baseline. The executable
official WASM probe checks that same span/category in Node v22.23.2, reuses the
existing sealed-include engine after failed creation and preserves instruction-
budget and malformed-source controls. Unchanged `for-004` warm checks pass across
5/50/500 items. [Raw closeout reports](ar-0027-capacity-closeout-controls-2026-10-03.json)
are conservation evidence, not timing or memory-policy claims.

No atomic cancellation/publication rule is accepted. Earlier tests establish
observed versus unobserved cancellation, cleanup and post-return immutable ownership;
a final token read does not atomically combine cancellation with caller publication.
That completion-contract question, candidate-specific host/WASM admission and wider
parity remain explicit prerequisites to reopening production adoption. Relationship
capacity, record layout and managed staging relocation are separate future questions.
Unchecked AR items remain visible rather than being called completed.

Full Rust verification, .NET Release build and the official executable WASM
release build pass. The preparation owner retains one private XML-failure
projection helper; no new semantic layer, dependency, ABI or unsafe operation
is introduced. The current campaign is deferred after successful feasibility,
not an accepted production representation. Corpus credit is unchanged.
