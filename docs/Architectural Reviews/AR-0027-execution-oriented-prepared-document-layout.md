# AR-0027: Execution Oriented Prepared Document Layout

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-10-02 |
| Last reviewed | 2026-10-02 |
| Scope | Private owned XDM storage, preparation placement, managed/native staging, navigation, and provenance |
| Trigger | Explore whether FastXSLT can reorganize ingested XML for cheaper execution rather than expose or inherit a general editable document model |
| Related ADRs | ADR-0002, ADR-0003, ADR-0004, ADR-0007, ADR-0012, ADR-0020, ADR-0021 |
| Related reviews | [AR-0009](AR-0009-prepared-input-retention-and-cache-lifecycle.md), [AR-0012](AR-0012-rust-embedding-facade-and-lifecycle.md), [AR-0013](AR-0013-prepared-representation-and-data-layout-audit.md), [AR-0026](AR-0026-namespace-node-identity-and-navigation.md) |
| Related evidence | [Prepared XDM byte anatomy](../Evidence/prepared-xdm-byte-anatomy-2026-08-31.md), [direct Rust lifecycle baseline](../Evidence/ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md), and the current anatomy reproduction below |

## Architectural question

Can a private execution-oriented prepared document materially reduce retained
memory or navigation cost compared with the current owned tree, while preserving
XDM semantics, identity, effective views, provenance, bounded preparation, and
the shared semantic engine?

The request began as whether FastXSLT should own something analogous to
`XDocument` or `XmlDocument`. It already owns an internal document. This review
asks how to store that document for execution, not whether to ship a public DOM
or permit callers to edit a prepared source.

### Rust native ownership

Rust applications are first-class consumers, not merely an internal benchmark
control. Host-neutral resource admission/sealing, XML/XDM preparation, compiled
state, invocation controls, semantic results and diagnostics belong in the
shared Rust layer. .NET, isolated-worker and WASM adapters project that
lifecycle into their respective host environments; they must not be required
to supply missing engine lifecycle semantics.

That ownership direction is independent of whether moving a particular staging
loop benchmarks faster. Measurements determine useful representations and
handoff costs, not whether a Rust application must depend on .NET. Hosts in any
language still own external acquisition, application validation, policy values,
workflow and publication. Transport framing and foreign-memory lifetime bridges
remain adapter responsibilities rather than prerequisites for direct Rust use.

The supported Rust facade is still governed by AR-0012. This review does not
publish the private workbench engine, document records or an editable DOM.
Candidate work must remain directly exercisable in Rust without requiring a
managed runtime, P/Invoke, or the worker wire protocol.

## Trigger and evidence

The current [owned tree](../../crates/fastxslt/src/xdm/owned_tree_experiment.rs)
stores private node records in shared immutable vector storage. Each record
includes its kind, parent, child and attribute vectors, namespace declarations,
optional name/prefix/value, source location, and document-order metadata.
Separate document state retains origin identity, typed-ID lookup, and optional
effective-child overrides. The
[preparation owner](../../crates/fastxslt/src/runtime/prepared_input_experiment.rs)
retains selected documents within an explicit sealed snapshot lifecycle.

This is already an engine-owned representation independent of parser events.
It is not evidence that the present record shape is optimal. General records
reserve fields even for node kinds that cannot use them, and relationship
storage and strings have separate allocations. Whether changing those choices
improves cache behavior or execution is still a hypothesis.

### Current baseline reproduction

On 2026-10-02, the existing release-mode anatomy probe passed with all features:

```text
cargo test -p fastxslt --release --all-features measure_prepared_xdm_capacity_anatomy -- --ignored --nocapture
```

The generated fixture contains 1,000 repeated namespaced items with one
attribute and text each: 42,039 input bytes and 3,002 XDM nodes.

| Accounted capacity | Bytes | Approximate share |
| --- | ---: | ---: |
| Node-record vector storage | 1,015,808 | 83.0% |
| Per-node resource-identity strings | 93,062 | 7.6% |
| Child and attribute ID storage | 72,224 | 5.9% |
| Remaining header, names, namespaces, prefixes and values | 43,329 | 3.5% |
| Total | 1,224,423 | 100% |

Local names have 2,001 occurrences but three unique values; resource identity
has 3,002 occurrences and one unique value. Nevertheless, repeated strings are
not the dominant capacity owner in this deliberately repetitive fixture.

The allocation observation reported 5,030 allocation requests, 2,228,454 total
requested bytes, and 1,302,989 peak live requested bytes within the measured
XDM-construction closure. Parsing occurs before that closure. Its reported
131,049 bytes still live are scope-relative, not the completed document's total
retention; moved parser allocations make that number unsuitable as a complete
memory estimate. The field estimate includes vector capacity, not only live
records, and excludes allocator metadata, fragmentation, Arc control blocks,
and process/runtime memory. None of these quantities is RSS or a hard cap.

No alternative representation was built or timed in this tranche. This one
synthetic source does not establish consumer benefit or a general XML expansion
ratio. Historical evidence remains historical rather than overwritten by this
current reproduction.

### Managed and Rust preparation placement

The owner also requested measurements of whether moving document preparation or
staging from .NET into Rust could improve the complete ingestion path. This is
an additional experiment axis, not evidence that Rust is automatically faster.

Current code inspection establishes the baseline:

- [ASP.NET startup](../../workbenches/FastXSLT.AspNet.Workbench/Program.cs)
  acquires source and stylesheet bytes through host-owned file reads.
- The [native client](../../workbenches/FastXSLT.AspNet.Workbench/NativeFastXsltClient.cs)
  encodes logical identities and hands byte arrays to the native boundary.
  The reviewed boundary copies foreign input into Rust-owned storage.
- The [isolated client](../../workbenches/FastXSLT.AspNet.Workbench/FastXsltWorkerClient.cs)
  frames identities and bytes for initialization over the worker pipe.
- The [shared Rust workbench engine](../../crates/fastxslt/src/runtime/workbench_experiment.rs)
  admits/seals resources, compiles the stylesheet, and prepares source XDM.
  FastXSLT's current lanes do not first build a managed XML DOM for import.

Consequently, parsing and prepared-tree construction are already Rust-owned.
Remaining placement hypotheses concern managed staging, encoding, request
assembly, transfer, copy count, and repeated initialization. Do not invent a
managed DOM round trip and present removing it as an improvement over today's
baseline. A real caller that already owns strings or a DOM may supply a separate
consumer-specific lane, with its input contract and conversion costs disclosed.

Use matched native and isolated comparisons, changing placement separately from
tree layout and holding source bytes, identities, diagnostics, limits,
compilation reuse, thread budget and output validation constant:

| Measurement lane | Purpose |
| --- | --- |
| Current managed byte handoff and Rust preparation | Reference deployment path |
| Candidate Rust-owned staging of already supplied bytes/metadata | Test one identified managed allocation, conversion, or repeated setup cost |
| Direct Rust caller lifecycle with equivalent admitted bytes | First-class Rust embedding measurement; phase attribution control for adapter comparisons, not an end-to-end .NET competitor |
| Real consumer string/DOM handoff, if present | Measure actual caller conversion without handicapping the byte baseline |

Attribute host acquisition, managed staging/encoding, boundary transfer/copies,
Rust admission, XML parsing, XDM construction, compilation, execution and result
transfer separately. If the existing creation call combines phases, instrument
it privately rather than labeling its entire duration as document preparation.
Count copied bytes and allocations as well as elapsed time; managed allocation
does not cover Rust memory. Record pins/leases and GC observations where relevant,
and distinguish deterministic ownership capacity from process memory.

Measure the direct Rust caller's complete admit/prepare/execute/result lifecycle
as well as its phases; do not reduce Rust embedding evidence to a parser-only
microbenchmark. Compare every adapter with this shared lifecycle for ownership,
diagnostic, cancellation and cleanup parity. Language-specific boundary costs
remain visible rather than being charged to the semantic engine indiscriminately.

Include one-shot documents, prepared reuse and a continuously refilled 5,000-job
set under equal concurrency and retained-byte ceilings. Source size, job count,
reuse count and worker count are independent axes. Preserve the already-warm
transform lane as a negative control: reducing ingestion cost should not be
credited as a warm evaluator speedup. Repeat in rotated fresh processes, report
tail latency and peak co-resident managed/Rust buffers, and retain a no-change
result if staging does not contribute materially.

Moving work is not moving authority. Hosts still acquire database/file/network
data and select policy; Rust consumes explicitly admitted memory. No hidden
file reopening, retained foreign pointers, zero-copy lifetime promise, new
unsafe export, or general staging scheduler is admitted. AR-0020's rejected
preparation-pool topology is not reopened merely by measuring placement. These
new comparative measurements have not yet been implemented or run.

## Ownership and constraints

- XDM owns document meaning, identity, order, names, values, typed IDs, namespace
  relationships and provenance. Physical compaction must not change them.
- Build and validate a candidate before sharing it; do not mutate already
  published prepared storage in place. Node identity must not depend on a
  physical slot if relocation changes that slot.
- Prepared state remains immutable and source-derived. Source-only topology
  metadata is distinct from indexes depending on stylesheet keys, collations,
  parameters or invocation context. Such indexes retain their existing owners.
- Equal bytes, names or URIs do not merge documents or authorize global,
  cross-snapshot or cross-generation sharing. Document-local interning is a
  candidate, not an accepted cache policy.
- ADR-0012 effective whitespace relationships must still govern every semantic
  consumer. A physical subtree interval must not expose hidden whitespace or
  change visible focus position, size, string values or copying.
- ADR-0021 namespace occurrences stay owner-qualified and derived. A compact
  tree must not eagerly materialize inherited namespace nodes or alias them by
  URI merely to simplify its layout.
- Preserve exact source spans and logical resource identities, including DTD
  metadata where admitted. Compressing provenance is not permission to lose it.
- Charge construction, index retention and work before admitting partial state;
  cancellation or exhaustion must cleanly abandon construction. Preserve
  existing charges unless a deliberate, tested accounting change is justified.
- Memory stays attributable to the owning document/generation and invocation.
  No hidden disk spill, ambient acquisition, allocator-global cache or unsafe
  exception follows from this experiment.
- Keep one semantic engine and the complete safe tree as a differential oracle.
  Private storage/access changes do not create a public provider trait, DOM,
  ABI layout, persistent format or alternate execution backend.

## Alternatives

1. **Retain the current tree.** Lowest migration risk; the reference remains the
   default if compact alternatives do not deliver useful gains.
2. **Compact the existing representation.** Compare tighter record fields,
   capacity sizing, document-local provenance/name storage, and contiguous
   child/attribute IDs behind the current owner. Change one major variable per
   experiment so gains and costs can be attributed.
3. **Separate hot topology from cold payload.** Test compact navigation records
   with kind-specific names, text, namespace and diagnostic storage. Additional
   indirection might offset the smaller working set; measure it rather than
   assuming structure-of-arrays wins.
4. **Add bounded source-only navigation metadata.** Subtree boundaries or name
   indexes may reduce repeated traversal but add preparation and retention.
   These are separate experiments, not mandatory additions to a compact tree.

A public mutable document model is not a performance alternative here. Editing
would require a separate consumer-driven review of node identity, index
invalidation, snapshot authority, concurrency, and resealing/repreparation.

## Findings and uncertainties

Node-record capacity is the first measured candidate, not a chosen design.
Distinguish live record size from vector capacity slack before redesigning
fields. The current accessors return borrowed `ExpandedName` and
`SourceLocation` records; table-based names or provenance may require private
access changes, not just swapping a container. Inventory those consumers before
assuming interning is a local edit.

Unknowns include preparation cost, peak coexistence during conversion, axis
costs, string-value cost, workload-specific reuse, and native versus WASM layout
behavior. A 5,000-document job queue does not mean 5,000 reuses of one prepared
document. Single-use ingestion must remain visible alongside warm reuse.

## Disposition

**Incubating.** Initial source/lifecycle inspection and the current baseline
reproduction are complete. A direct Rust cold/reused lifecycle harness now
passes unchanged `for-004` output and cancellation/retry controls. Compile-once
fresh sources and shared-seam XML/XDM observations extend that baseline, with
preliminary timings rather than an adapter comparison or representation
decision. Private freeze/pre-sizing comparisons now measure capacity savings;
pre-sizing also lowers peak in the tested shapes. Typed-ID/view/namespace and
concurrent-owner controls pass, but broader parity and full host measurements
are still prerequisites to adoption. Relationship storage remains a separate
candidate. Select no representation or public API. This is
a focused prepared-document experiment under AR-0013, not a replacement for
its broader profiling program or AR-0009's retention policy review.

## Required follow-up

- [x] Inspect current storage/access/lifecycle owners and reproduce the existing
  release anatomy baseline without changing engine code.
- [x] Inventory current .NET/Rust placement: host byte acquisition and adapter
  framing surround Rust-owned admission, compilation and source preparation.
- [ ] Exercise a first-class direct Rust caller lifecycle without managed or
  wire-protocol dependencies, and compare native/isolated adapter parity without
  selecting public types ahead of AR-0012.
  - [x] Establish one-shot and compiled/prepared-reuse controls over unchanged
    `for-004`, exact output, cancelled preparation cleanup and retry; record
    exploratory release phase timings.
  - [x] Add compile-once fresh-document ingestion with distinct logical origins,
    exact results and independent owner retirement; record a private 5,000-job,
    single-worker equal-byte queue control.
  - [x] Separate controlled XML/XDM timing at the shared preparation seam, with
    ordinary/observed parity over the pinned source and synthetic 5/50/500 items.
  - [ ] Compare existing native/isolated adapters before selecting a staging
    change; replay representative distinct sources.
- [ ] Measure that boundary phase by phase, nominate one real managed staging
  cost, then compare its current and Rust-owned implementations independently
  of any tree-layout change.
- [ ] Include one-shot, reusable and 5,000-job native/isolated ingestion lanes,
  exact diagnostic/result parity, copied-byte/allocation counts, cancellation,
  backpressure, failure cleanup and peak co-resident storage. Do not claim a
  warm transform gain from moving setup outside the timed region.
- [x] Separate live record footprint, capacity slack, allocation counts, and
  repeated payload across wide, deep, attribute-heavy, text-heavy,
  namespace-heavy and low-repetition sources.
- [ ] Inventory borrowed name/provenance consumers and physical-ID assumptions.
- [ ] Nominate one safe layout change, define eligibility and expected tradeoffs,
  and prototype privately after any required ownership decision is reviewed.
  - [x] Prototype node-vector freeze shrinking only, with unchanged construction,
    relationship capacities and accessors; measure retention, peak and paired
    construction/prepare-release latency against the safe reference.
  - [x] Compare test-only tighter construction/pre-sizing independently,
    preserving text coalescing and budget/failure behavior; measure seven shapes
    including near-full occupancy, with allocation and paired latency controls.
  - [ ] Examine relationship capacity separately; resolve broader parity and
    allocation/cancellation gaps before selecting either node-capacity policy.
- [ ] Differentially verify navigation, order/deduplication, identity, typed IDs,
  namespace occurrences, diagnostics, whitespace views, copying and results.
  - [x] Compare both capacity candidates against growth and complete whitespace
    derivation on a bounded typed-ID/default/namespace fixture; verify actual
    transform output, charges, controlled failures and result-owner retirement.
  - [ ] Extend differential controls to wider path/temporary-tree operations
    and sealed external-DTD input families before admission.
- [ ] Exercise cancellation, exact/one-less budgets, construction failure,
  concurrent reuse, and independent overlapping generation retirement.
  - [x] Exercise candidate construction/view controls and four-thread shared
    pre-sized strip/preserve reuse with overlapping old/new owners; prove old
    leases drain and replacement ownership remains independent.
  - [x] Measure shared span-scan, reservation and shrinking cancellation gaps
    with inert hooks and boundary faults; retain observed versus unobserved
    distinctions and do not infer hard latency limits.
  - [x] Prototype uncharged chunk/allocation checkpoints, preserving ordinary
    node charges and charge-indexed failure controls; measure rotated overhead
    and retain timing reversals rather than claiming the polls are free.
  - [ ] Review the candidate publication fence and competing asynchronous
    failures; measure complete adapter lifecycle retention before adoption.
  - [x] Prove measured source/snapshot/XML/XDM allocations release on successful
    construction and exact one-less node-budget failure across six shapes and
    all three constructors; do not equate that with process reclamation.
- [ ] Measure complete preparation, peak old/new coexistence, retained bytes,
  single-use latency and break-even reuse; then warm throughput, tails and
  concurrency through native and isolated ASP.NET boundaries.
  - [x] Measure direct Rust admission/preparation/execution/serialization/release
    for all six shapes at one/eight reuses, with rotated candidate order and
    exact output; retain timing reversals and decline a general speedup claim.
  - [ ] Measure full transform peaks, more reuse points and representative host
    creation/ingestion before deriving a break-even or adoption decision.
- [ ] Validate the official WASM target, distinguishing host-width layout and
  memory limits; retain ordinary-path controls and unchanged corpus cases.
- [ ] Record a retained candidate or negative result. Require an ADR before
  changing an established ownership/representation contract; do not promote
  microbenchmark gains into general performance or conformance claims.

## Reopening triggers

Reassess when a named consumer exposes a prepared-memory or navigation
bottleneck, a safe candidate earns repeatable end-to-end benefit, or an access
change would alter identity, public lifecycle, generation ownership or authority.

## Review history

- 2026-10-02 -- Added private uncharged cancellation checkpoints: at most 256
  scan visits between polls, and checks before/after capacity changes. Ten scan
  controls, 24 resize controls and 216 differential pairs pass. The post-shrink
  publication fence is explicitly different from the reference and remains
  unaccepted. Three rotated construction runs are noisy and establish no
  precise overhead or host benefit. Full gates pass with 1,525 core tests and
  WASM; production remains unchanged.
  [Evidence](../Evidence/ar-0027-bounded-capacity-checkpoints-2026-10-02.md)

- 2026-10-02 -- Measured test-only cancellation boundaries across six shapes
  and three release stress runs. The 96,002-node shared span scan precedes the
  first XDM check and takes 581.3-968.7 us at the process medians; allocation
  intervals are shorter. Reservation can finish after signalling cancellation;
  freezing can return after its final semantic check. Checkpoint design remains
  open, production is unchanged, and full gates pass with 1,522 core tests and
  WASM. [Evidence](../Evidence/ar-0027-capacity-cancellation-gaps-2026-10-02.md)

- 2026-10-02 -- Added direct Rust complete-source lifecycle timing and 36 scoped
  successful/failed-construction release controls. All measured scope ownership
  returns to zero; three timing processes validate 97,686 exact synthetic
  outcomes. Wide single-use pre-sizing improves 2.2-11.6%, but several other
  shape/reuse cells reverse or are nearly flat. No general speedup, break-even,
  production choice or corpus pass follows. Full gates pass with 1,520 core
  tests and WASM. [Evidence](../Evidence/ar-0027-capacity-single-use-lifecycle-2026-10-02.md)

- 2026-10-02 -- Extended capacity conservation through typed IDs/defaults,
  complete whitespace derivation, qualified namespace identity, actual strip/
  preserve transforms, copied result ownership and structured failures. Four
  threads share one pre-sized source across strip/preserve programs while old
  leases drain and replacement owners remain independent. Three focused tests
  and full gates pass with 1,518 core tests and WASM. Production remains growth;
  broader parity, allocation/cancellation gaps and host measurements are open.
  [Evidence](../Evidence/ar-0027-node-vector-presizing-comparison-2026-10-02.md#semantic-and-lifecycle-follow-up)

- 2026-10-02 -- Measured test-only pre-sizing through the existing span scan,
  with unchanged per-node charges and budget-ineligible fallback to growth.
  Three allocation repetitions agree: original-six retained reductions match
  freezing, while peak falls 15.8-48.4%. Near-full occupancy falsifies unchanged
  freeze peak: +15.0% peak for 0.63% retention savings. Three paired timing runs
  show 12.8-14.3% lower wide preparation medians but negligible gains on some
  shapes. Production growth remains unchanged; broader parity, complete
  lifecycle and host measurements remain open. Full gates pass with 1,515 core
  tests and WASM. [Evidence](../Evidence/ar-0027-node-vector-presizing-comparison-2026-10-02.md)

- 2026-10-02 -- Measured the test-only node-vector freeze candidate. Retention
  falls 9.4-44.4% in the six-shape matrix; allocator peak is unchanged there, but
  one extra resize request increases total requested bytes. Three latency runs
  remain variable and do not justify a speedup or production choice. Node,
  provenance, charge, budget, cancellation and pinned transform checks pass.
  Pre-sizing, broader parity and host-visible comparison remain open. Full
  gates pass with 1,514 core tests and WASM.
  [Evidence](../Evidence/ar-0027-node-vector-freeze-comparison-2026-10-02.md)

- 2026-10-02 -- Completed six-shape capacity/allocation anatomy. Occupied records
  and unused vector slots are now separate; unused records account for 9.4-44.4%
  of known document capacity. Text payload dominates the text-heavy counterexample.
  Complete XML/XDM allocation scopes reproduce identically across three runs.
  Nominate capacity tightening before changing record fields, but accept no
  optimization. Full gates pass with 1,511 core tests and WASM.
  [Evidence](../Evidence/ar-0027-document-shape-capacity-anatomy-2026-10-02.md)

- 2026-10-02 -- Added shared-seam XML/XDM phase attribution and paired ordinary
  controls. XML is the larger measured phase on four sources; on synthetic 500
  items it measures 483.1-497.3 us versus 126.7-147.2 us XDM construction. All
  24,024 timed outputs match; charges, exact/one-less limits, cancellation and
  failure diagnostics retain parity. Observation remains test-only, with no
  layout or parser decision. Full gates pass with 1,510 core tests and WASM.
  [Evidence](../Evidence/ar-0027-shared-preparation-phase-attribution-2026-10-02.md)

- 2026-10-02 -- Added the compile-once fresh-source control. Three fresh release
  processes check all 15,000 timed outcomes exactly; per-job medians are
  10.6-16.8 us, with 5,000-job windows of 64.0-93.5 ms on one worker. Equal bytes
  retain separate origins and provenance. These variable short windows do not
  select a layout, staging change or deployment capacity. Full gates pass with
  1,508 core tests and the official WASM check. [Evidence](../Evidence/ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md)

- 2026-10-02 -- Started implementation with a private direct Rust lifecycle
  test/measurement owner. Three fresh release runs observed 28.4-32.5 us complete
  one-shot medians and 2.0-2.1 us compiled/prepared invocation medians on the tiny
  pinned standards case. Full gates pass with 1,507 core tests and the official
  WASM check. Short instrumented timings remain non-decisional; layout and
  adapter staging are unchanged. [Evidence](../Evidence/ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md)

- 2026-10-02 -- Clarified first-class Rust application consumption. Shared
  host-neutral lifecycle ownership does not depend on a .NET performance win;
  direct Rust measurements cover complete embedding as well as attribution.
  Public facade decisions remain in AR-0012, and host acquisition authority is
  unchanged.

- 2026-10-02 -- Added managed/Rust preparation-placement comparisons at the
  owner's request. Code inspection confirms XML/XDM preparation is already in
  Rust; comparative staging/transfer measurements remain pending. No authority,
  ABI, foreign-memory lifetime or scheduler decision follows from the plan.

- 2026-10-02 -- Opened as Incubating after the engine-owned document question;
  inspected the current source representation and reproduced its release anatomy
  baseline. No engine change, representation choice or public DOM was admitted.
