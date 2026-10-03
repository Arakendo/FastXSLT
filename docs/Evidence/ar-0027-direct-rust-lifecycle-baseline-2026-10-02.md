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
