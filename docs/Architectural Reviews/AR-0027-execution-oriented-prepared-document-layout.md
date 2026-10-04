# AR-0027: Execution Oriented Prepared Document Layout

| Field | Value |
| --- | --- |
| Status | Deferred |
| Opened | 2026-10-02 |
| Last reviewed | 2026-10-03 |
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
comparative baselines now include matched fresh creation and persistent-worker
ingestion; no staging-placement change has been nominated or measured.

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

**Deferred after successful bounded capacity feasibility.** The current campaign
is concluded. Retain production growth and the private safe freeze/pre-sizing
candidates, differential controls and measurements. Capacity tightening can
reduce known retained and peak requested allocation without changing node records,
identity or the semantic engine. At 500 items, complete two-engine pre-sizing
saves 9.85% retained requested bytes and 14.13% construction-prefix peak; delayed
results narrow the peak advantage to 6.35%. These are measured workload results,
not universal memory ratios or RSS guarantees.

No general lifecycle speedup or reuse break-even is established. Native/isolated
host comparisons remain production-growth baselines; official WASM builds and
runtime controls likewise do not execute the capacity candidates. Candidate host
admission, target-width allocation behavior and an accepted cancellation/completion
policy remain unproven. Those gaps justify deferred adoption, not additional
public knobs, protocol fields or an indefinitely expanding measurement campaign.

The incidental structural-XML-limit diagnostic defect is repaired through the
shared preparation owner and verified across current host boundaries. This does
not select a capacity policy. Relationship compaction, field changes, interning,
hot/cold layouts, staging relocation and other new hypotheses are separate future
work under AR-0013, not requirements to close this campaign. A named consumer's
prepared-memory pressure may justify reopening the bounded candidate decision.
No representation, public DOM, facade, ABI or production checkpoint rule is accepted.

## Required follow-up

This checklist retains the experiment's evidence and unresolved adoption
prerequisites. Unchecked items are deferred, not an active work queue. Reopening
must name a consumer pressure and select only the proof needed for that decision.

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
    - [x] Establish matched fresh full-engine creation controls in direct Rust,
      native .NET and isolated workers, including compilation, exact first
      results, disposal and ordinary/measured diagnostic parity. Three process
      repetitions recorded; reusable and distinct-source host lanes remain open.
    - [x] Remove process startup from a sequential 5,000-job mixed ingestion
      control, retaining full compile+prepare semantics and proving failed
      replacement recovery, field-limit rejection and worker-loss retirement.
      Compile-once ingestion and representative distinct payloads remain open.
    - [x] Extend unchanged native/persistent-isolated ingestion across six
      synthetic copy shapes and three fresh 5,000-job repetitions. Balance
      first-lane order per shape; disclose the depth-64 adapter ceiling and
      structural-depth misclassification. This is not candidate measurement.
    - [x] Correct structural XML limit projection so well-formed input denied
      by depth/event ceilings is not reported as malformed/invalid XML. Keep
      parser provenance, adapter parity and prior-generation recovery.
    - [x] Extend the direct compile-once control to distinct payloads and mixed
      5/50/500-item jobs: 15,000 timed exact outputs, fresh-compile differential
      checks and independent origin/provenance/retirement controls. Controls
      remain unbounded; limits-matched compile-once adapters remain open.
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
    - [x] Compare two sealed external-subset fixtures through five constructors,
      preserving typed IDs/default precedence, effective whitespace, parent-path
      deduplication, provenance and exact execution charges.
    - [x] Repair the temporary-path `xsl:for-each` dispatch panic using the
      shared charged temporary selector; verify nested focus and empty paths.
    - [x] Resolve plain relative attribute-value paths under temporary focus,
      re-enable the unchanged regression, and compare growth/pre-sized/frozen
      focus, output charges and result retirement on that fixture. General
      temporary axes, predicates and sorting remain outside this proof.
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
  - [x] Review the candidate's final cancellation observation versus return and
    caller publication. Prove failed replacement preserves old leases and
    post-return cancellation does not revoke immutable source storage; do not
    claim an atomic publication fence from a final token read.
  - [x] Establish current native/isolated retention and admission controls with
    delayed results, overlapping generations, failed replacement, quota denial
    and release/readmission; distinguish exact payload ownership from managed
    copies and unmeasured engine/transient memory.
  - [x] Prove current private native known-capacity and aggregate-byte admission
    at exact/one-less limits across result-heavy overlapping generations, retained
    outcomes and concurrent last-capacity insertion; do not add a metrics export.
  - [x] Verify result-heavy concurrent old/new candidate leases with independent
    cancellation and exact/one-less invocation budgets, complete growth-oracle
    diagnostics/charges, same-source recovery and serialized result retirement.
  - [x] Exercise candidates at the real shared prepared-set seam using test-only
    selection; preserve snapshot/map/lease ownership, exact/one-less preparation
    failure projection and recovery without partial publication.
  - [x] Integrate candidate selection into complete shared Rust engine creation
    in tests only; conserve retention components, exact reuse/lease retirement,
    creation failures and per-invocation diagnostic/recovery behavior.
  - [ ] Select any supported completion rule deliberately and measure complete
    adapter lifecycle retention/admission before candidate adoption.
  - [x] Prove measured source/snapshot/XML/XDM allocations release on successful
    construction and exact one-less node-budget failure across six shapes and
    all three constructors; do not equate that with process reclamation.
- [ ] Measure complete preparation, peak old/new coexistence, retained bytes,
  single-use latency and break-even reuse; then warm throughput, tails and
  concurrency through native and isolated ASP.NET boundaries.
  - [x] Measure direct Rust admission/preparation/execution/serialization/release
    for all six shapes at one/eight reuses, with rotated candidate order and
    exact output; retain timing reversals and decline a general speedup claim.
  - [x] Measure full-transform requested-allocation peaks on six count and six
    copy shapes, with prepared/result/serialized retention and scope release.
  - [x] Measure complete-engine old/new coexistence and delayed outputs across
    three fresh allocation repetitions; reconcile known charges separately and
    prove complete release, failed replacement and cancellation cleanup.
  - [x] Measure complete Rust engine creation/transform/release at one/eight uses
    and with an existing generation retained, without allocator hooks. Rotate
    fresh-process order, preserve exact output and record timing reversals.
  - [x] Resolve the default-debug-stack failure on the 256-deep copy reference
    through safe ancestor-cursor serialization. Retain the shallow recursive
    oracle, promote the deep reference to an ordinary regression and replay
    all six copy shapes. No arbitrary-depth guarantee is inferred.
  - [x] Extend the direct count lifecycle to 1/2/4/8/16/32/64 uses across six
    shapes, with fresh-process reuse-group rotation and exact output. Record
    timing reversals rather than selecting a universal break-even.
  - [ ] Measure result-heavy reuse and representative host creation/ingestion
    before deriving a break-even or adoption decision. Match compilation scope
    and separate isolated process startup from preparation.
- [ ] Validate the official WASM target, distinguishing host-width layout and
  memory limits; retain ordinary-path controls and unchanged corpus cases.
- [x] Record a retained candidate or negative result. Require an ADR before
  changing an established ownership/representation contract; do not promote
  microbenchmark gains into general performance or conformance claims.
  - [x] Conclude the present campaign: capacity savings demonstrated, general
    speedup unproven, candidates retained privately and production adoption
    deferred. No new layout/staging exploration is required for this closeout.

## Reopening triggers

Reassess when a named consumer exposes material prepared-memory pressure and
supplies workload/reuse/concurrency/headroom evidence, or when a safe candidate
earns repeatable end-to-end benefit through the relevant host boundary. Reopening
must address candidate admission, target-width behavior and completion semantics
before adoption. A new access/layout change affecting identity, public lifecycle,
generation ownership or authority requires its own deliberate review, not silent
extension of this concluded campaign.

## Review history

- 2026-10-03 -- Concluded the capacity feasibility campaign at the owner's
  direction; defer production adoption and keep new layout/staging hypotheses
  outside its closeout. Repair structural XML depth/event limit classification
  through shared preparation, preserving source spans and cancellation precedence.
  Direct candidate controls, native/isolated recovery and executable official
  WASM controls pass. No host candidate selector or completion contract is added.
  [Closeout evidence](../Evidence/ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md#bounded-capacity-campaign-closeout-on-2026-10-03)

- 2026-10-03 -- Broadened the production-growth host baseline to six synthetic
  copy shapes: three fresh 5,000-job queues per lane validate 30,000 timed exact
  results, balanced first-lane order and zero residual native registry ownership.
  Depth 256 is rejected consistently but misclassified as FXXM0002/invalid;
  diagnostic repair remains open. .NET Release and conservation gates pass.
  No candidate selector, staging change, production policy or adoption follows.
  [Evidence](../Evidence/ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md#host-shape-ingestion-baseline-on-2026-10-03)

- 2026-10-03 -- Added complete Rust engine lifecycle timing without allocation
  hooks. Three fresh rotated processes record 432 phase medians/p95 values and
  validate 15,552 timed outputs. Candidate reversals and large reference drift
  prevent a general speedup, precise overhead or reuse break-even claim; memory
  savings remain separately reproducible. Full gates and the official WASM
  build check pass. Host candidate timing/admission and adoption remain open.
  [Evidence](../Evidence/ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md#complete-engine-capacity-timing-on-2026-10-03)

- 2026-10-03 -- Measured seven complete-engine ownership prefixes across three
  fresh release processes: all 189 records agree, and 81 released/failure scopes
  return requested ownership to zero. At 500 items, pre-sizing saves 9.85% of
  two-engine retention and 14.13% of construction-prefix peak; delayed outcomes
  reduce the peak advantage to 6.35%, equal to freezing. Known charges remain
  separate from allocator observations. Full gates pass with 1,556 core tests
  and official WASM build check; host candidate timing/admission and adoption
  remain open.
  [Evidence](../Evidence/ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md#complete-engine-overlap-allocation-on-2026-10-03)

- 2026-10-03 -- Extracted shared engine construction privately after the
  2,119-line facade review; the parent is now 1,869 lines. The mechanical
  checkpoint reproduces 37 existing engine tests. Test-only capacity selection
  then passes nine engine generation pairs/144 exact byte results, 27 creation
  failure comparisons and 18 invocation controls with recovery. Full gates and
  official WASM target check pass. Complete-engine allocator peaks and host
  candidate measurements remain open; no production selection changes.
  [Evidence](../Evidence/ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md#complete-rust-engine-capacity-controls-on-2026-10-03)

- 2026-10-03 -- Added test-only capacity selection at shared prepared-set
  construction, preserving production growth and existing phase observation.
  Nine old/new set pairs retain eighteen exact outputs after map retirement;
  twenty-seven preparation controls preserve failure equality and node/event
  charges, with eighteen same-builder recoveries and no partial publication.
  Full gates pass with 1,552 core tests and official WASM build check. Complete
  engine/adapter candidate measurements and adoption remain open.
  [Evidence](../Evidence/ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md#shared-prepared-set-capacity-seam-on-2026-10-03)

- 2026-10-03 -- Added result-heavy concurrent candidate lease controls across
  18 generation pairs and four independent worker controls. All 144 controlled
  outcomes conserve complete growth-reference failures and ten-domain charges;
  144 fresh-control recovery outputs remain exact after both document owners
  expire. This proves direct-Rust isolation and retirement, not occupancy or
  throughput. Workspace tests pass with 1,550 core tests; candidate adapter
  integration, transient peaks and a supported completion decision remain open.
  [Evidence](../Evidence/ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md#concurrent-result-heavy-generation-leases-on-2026-10-03)

- 2026-10-03 -- Added four private native registry controls: eighteen exact/
  one-less result-heavy scenarios and six concurrent last-capacity scenarios.
  Known engine capacity and total accounted bytes reject atomically, preserve
  valid generations and restore charges after release. No observation export,
  production constructor or quota policy changed. Full workspace tests pass
  with 1,549 core and 22 native tests; official WASM build check passes.
  Candidate adapter integration, transient peak and active concurrent leases
  remain open.
  [Evidence](../Evidence/ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md#private-native-byte-admission-controls-on-2026-10-03)

- 2026-10-03 -- Added real native/isolated retention/admission baseline through
  existing exports/framing and production growth construction. Three fresh
  .NET 10 processes reproduce ten native ownership checkpoints and 39 exact
  outputs. Two-generation engine denial consumes no outcome slot or evicts valid
  state; release restores admission and delayed outcomes outlive engine disposal.
  Isolated initialization failure preserves its worker and transferred strings
  outlive disposal. Managed/native Release builds pass. Engine charges/peak,
  candidate adapter integration and active concurrent leases remain open.
  [Evidence](../Evidence/ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md#adapter-retention-and-admission-baseline-on-2026-10-03)

- 2026-10-03 -- Reviewed candidate publication boundaries and corrected the
  earlier fence wording: final cancellation observation is not atomic with
  return or caller publication. Twelve failed replacement/lease controls and
  eighteen post-return cancellation controls pass with 84 exact outputs.
  Returned immutable sources remain usable with fresh invocation controls;
  cancelled controls still fail normally. Distinct old/new origins and final
  weak-owner expiry pass. No supported completion rule, capacity adoption or
  adapter publication change follows. Complete adapter retention remains open.
  [Evidence](../Evidence/ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md#candidate-publication-boundary-review-on-2026-10-03)

- 2026-10-03 -- Closed the measured result-heavy construction-failure allocation
  gap. Each run checks 216 XML/XDM construction scopes and 12 resize cancellations;
  three fresh release processes reproduce records, with all 630 failed and 54
  successful scopes ending at zero tracked ownership. Maxima include parsing
  and are not RSS or isolated XDM peaks. Ordinary regression enforces cleanup;
  production remains unchanged. Host retention, broader XML failure profiles and
  checkpoint publication/adoption remain open. Full gates pass with 1,547 core
  tests and official WASM compilation.
  [Evidence](../Evidence/ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md#construction-failure-allocation-cleanup-on-2026-10-03)

- 2026-10-03 -- Extended result-heavy control parity: 108 construction pairs,
  12 before/after resize cancellation controls and 162 invocation cases across
  result-node/text-byte/serialized-byte limits and cancellation. Structured
  failures and targeted charges equal the growth reference; every invocation
  allocation scope releases to zero and all 162 same-source recovery outputs
  match. Production is unchanged. Construction-failure allocation attribution,
  host retention and checkpoint adoption remain open. Full gates pass with
  1,546 core tests and official WASM compilation.
  [Evidence](../Evidence/ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md#result-heavy-capacity-control-parity-on-2026-10-03)

- 2026-10-03 -- Compared growth/freeze/pre-sizing on result-heavy copied output.
  All 7,776 timed outputs match; all ten work counters and source retirement
  preserve parity. Three processes reproduce allocation records and zero released
  ownership. At 500 items both candidates save 135,160 retained requested bytes
  (5.74% of the complete source/result/output scope); pre-sizing lowers cumulative
  requests while freeze raises them. Timing reversals prevent a speedup or
  adoption claim. Direct Document scopes omit PreparedInputSet wrappers and
  capacity checkpoint polls. Full gates pass with 1,543 core tests and WASM.
  [Evidence](../Evidence/ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md#result-heavy-capacity-comparison-on-2026-10-03)

- 2026-10-03 -- Added result-heavy requested-allocation scopes. Three fresh
  processes reproduce all 18 records identically; successful release, partial
  copy cancellation and one-less output limits leave zero tracked ownership.
  Exact cancellation/limit diagnostics retain request correlation. At 500 items,
  prepared/result/output retained bytes reach 2,356,567 with a 2,504,862 prefix
  peak. These are allocator requests, not RSS or host accounting. Full gates
  pass with 1,542 core tests and WASM. Capacity-candidate comparisons, broader
  failure parity and adapter transfer remain open; no layout selected.
  [Evidence](../Evidence/ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md#result-heavy-allocation-and-failure-controls-on-2026-10-03)

- 2026-10-03 -- Added result-heavy direct Rust one/eight-use controls with
  2,592 timed exact outputs and copied-result survival after source retirement.
  At roughly 271 KB output, one-shot preparation remains largest; eight-use
  serialization/validation becomes the largest aggregate phase. Explicit
  test-only parser ceilings avoid weakening production defaults. Full gates
  pass with 1,541 core tests and WASM. Capacity-candidate result-heavy parity,
  retention and host transfer remain open.
  [Evidence](../Evidence/ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md#result-heavy-prepared-reuse-follow-up-on-2026-10-03)

- 2026-10-03 -- Extended direct compile-once ingestion to distinct payloads.
  Three 5,000-job runs compare every timed result exactly; full-job medians are
  14.6–15.0 us at 5 items, 76.3–76.7 us at 50, and 708.3–728.9 us at 500.
  Preparation dominates the largest tier. Fresh-compilation parity, distinct
  provenance and surviving-owner retirement pass. Full gates pass with 1,540
  core tests and WASM; no adapter speedup or representation choice follows.
  [Evidence](../Evidence/ar-0027-direct-rust-lifecycle-baseline-2026-10-02.md#distinct-payload-compile-once-follow-up-on-2026-10-03)

- 2026-10-03 -- Added persistent-worker initialization over the existing private
  protocol. Three 5,000-job queues per lane validate 30,000 timed exact outputs;
  failed replacement preserves old state, lost-worker retirement never retries,
  and native registry ownership returns to baseline. Isolated 500-item setup
  medians are 0.82–0.90 ms without startup. Retirement costs depend on the prior
  source; repeated initialization still recompiles. No staging relocation or
  production representation selected.
  [Evidence](../Evidence/ar-0027-persistent-worker-ingestion-2026-10-03.md)

- 2026-10-03 -- Added matched full-engine creation measurements through existing
  adapters and direct Rust. Three repetitions conserve 1,152 timed exact results,
  ordinary/measured failure parity and native registry release. At 500 items,
  creation medians are 0.99–1.04 ms Rust, 1.05–1.17 ms native and 15.1–16.3 ms
  fresh isolated worker. Pipe writes absorb startup waiting; no internal phase
  or removable managed cost is inferred. Full gates pass with 1,539 core tests
  and WASM. No new export, production capacity choice or corpus credit.
  [Evidence](../Evidence/ar-0027-matched-engine-creation-2026-10-03.md)

- 2026-10-03 -- Extended reuse observations to seven points. Three recorded
  fresh processes validate 459,486 timed outputs each. Pre-sized wide-source
  single-use totals improve 7.6–11.1%, falling to 0.6–3.1% at 64 uses; 18 of
  42 shape/reuse groups reverse between runs. No universal timing or break-even
  is selected. Adapter inspection confirms existing creation combines admission,
  compilation and preparation, with process startup additionally present in the
  isolated path. Matched host attribution remains pending. Full gates and WASM
  pass; production policy and corpus counts are unchanged.
  [Evidence](../Evidence/ar-0027-capacity-reuse-sweep-and-ingestion-attribution-2026-10-03.md)

- 2026-10-03 -- Resolved the localized serialization blocker with safe borrowed
  ancestor cursors; the unchanged 256-deep copy passes on the default debug
  stack. Differential output, charge, failure-prefix, cancellation and namespace
  unwind controls pass. Three release processes reproduce all 144 allocation
  records identically with 36 zero-retention full-release scopes per run.
  Pre-sized deep-copy peak falls 20.4%; serializer scratch has a small measured
  transient allocation cost. Full gates pass with 1,538 core tests and 48 ignored
  probes; the official WASM target builds. Capacity adoption remains open.
  [Evidence](../Evidence/ar-0027-stack-safe-serialization-and-allocation-replay-2026-10-03.md)

- 2026-10-03 -- Localized the 256-deep default-debug overflow to serialization,
  after successful execution and source retirement. Extracted the recursive
  node writer unchanged into a private 309-line child before attempting the
  repair. Full gates pass with 1,535 core tests and 49 ignored probes; the
  official WASM target builds. The sacrificial crash remains reproducible and
  the deep-copy allocation cell remains excluded. No repair or gain claimed.
  [Evidence](../Evidence/ar-0027-serialization-stack-localization-2026-10-03.md)

- 2026-10-03 -- Extended allocation observation through preparation, semantic
  result, serialized result and release. Three release processes reproduce all
  132 prefix scopes identically, with 33 full-release scopes per run returning
  tracked ownership to zero. Pre-sized full-copy peak falls 19.0% on wide and
  47.7% on attribute-heavy, but just 1.9% on text-heavy. A 256-deep growth-copy
  reference overflows the default debug test-thread stack; the isolated release
  reproducer passes. That cell remains excluded and explicitly unresolved.
  Full gates pass with 1,535 core tests and 49 ignored probes; WASM builds.
  No timing, capacity adoption or corpus gain is claimed.
  [Evidence](../Evidence/ar-0027-full-transform-allocation-peaks-2026-10-03.md)

- 2026-10-03 -- Closed the temporary attribute-value blocker with a narrow
  private projection of eligible relative attribute paths. The unchanged
  AR-0027 regression now passes through growth, pre-sized and frozen inputs,
  including all-domain output charges and serialization after source retirement.
  Source/temporary comparisons cover unqualified and expanded names, wildcard
  attributes and missing values; control tests cover exact/one-less budgets and
  cancellation during selection. Predicate paths remain explicitly unsupported.
  Full gates pass with 1,534 core tests and 47 ignored probes; WASM builds.
  No capacity policy, parser authority or corpus numerator changes.
  [Evidence](../Evidence/ar-0027-temporary-attribute-value-parity-2026-10-03.md)

- 2026-10-03 -- Committed the parity checkpoint at `1e04cc7f`, then extracted
  for-each dispatch into a cohesive private owner and verified it unchanged.
  Temporary-path iteration now uses the existing charged selector and shared
  body evaluator. Three focused golden/control tests pass. The unchanged
  broader probe now reaches an `XPDY0002` failure at ordinary `@key` value
  selection rather than the source-only dispatch panic; it remains explicitly
  ignored and uncredited. Sorted temporary paths are explicitly unsupported
  with sort provenance. No capacity policy or corpus numerator is changed.
  [Evidence](../Evidence/ar-0027-temporary-for-each-dispatch-repair-2026-10-03.md)

- 2026-10-03 -- Committed prior work at `74daf64b`, then extended sealed-subset
  parity across five constructors and twenty exact transforms. A broader
  temporary-tree probe exposes an ordinary-reference dispatch panic before
  candidate comparison. Preserve its explicitly ignored, still-failing
  reproducer as an open correctness blocker rather than crediting parity.
  Full gates pass with 1,526 core tests and 48 ignored probes; WASM builds.
  Production remains unchanged.
  [Evidence](../Evidence/ar-0027-sealed-subset-parity-and-temporary-path-blocker-2026-10-03.md)

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
