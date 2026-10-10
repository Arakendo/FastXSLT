# AR-0028: Owner Qualified Runtime Node Values

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-10-06 |
| Last reviewed | 2026-10-07 |
| Scope | Invocation node values, document focus and mixed selections |
| Trigger | Remaining unchanged OASIS cases require node ownership to survive document switches |
| Related ADRs | ADR-0004, ADR-0012, ADR-0017, ADR-0021, ADR-0022 |
| Related evidence | [Global literal-document variables](../Evidence/oasis-xslt10-global-literal-document-variable-2026-09-30.md), [current attribute lookup](../Evidence/oasis-xslt10-current-predicate-lexical-boundaries-2026-10-04.md) |

## Architectural question

Can invocation values retain ordinary nodes from more than one immutable
document through variables, focus, parameters and unions, without confusing
local node IDs, duplicating execution semantics or exposing private owners
through the public facade?

## Trigger and evidence

The 2026-10-06 complete OASIS CD04 replay preserves 2,493 / 3,173 exact XML
comparisons (78.57%). It reports 2,694 initialized cases, 2,645 successful
executions, 476 initialization failures, 49 execution failures, 14 XML
mismatches, four comparator gaps and 423 / 431 expected-error credits.
The pinned archive and comparison rules are unchanged.

Several remaining cases require more than another scalar document recognizer:

- `Lotus/mdocs_mdocs14#1` binds a secondary document and dispatches templates
  while principal-source node globals must remain associated with their source.
  It currently returns located unsupported behavior, not an unbound-variable
  error or a fabricated successful result.
- `Lotus/mdocs_mdocs16#1` navigates and filters a variable bound to a secondary
  document. Scalarizing that variable would discard the relationships needed
  by its later expression.
- `Lotus/select_select59#1` composes key, principal-source and literal-document
  selections in a union. Deduplicating bare local IDs cannot establish whether
  nodes from distinct document owners are the same node.

`idkey49`, `idkey51` and `idkey52` add dynamic document selection or identity
consumers. They are later pressure, not promises that the first representation
experiment will make them execute or compare exactly.

The current `RuntimeGlobals.nodes`, `RuntimeVariables.source_nodes` and
`InvocationParameterValue::SourceNodes` retain local `NodeId` vectors.
`SequenceInputs.source` supplies the active document; `SequenceContext.node`
does not independently qualify an ordinary node with its document.
Literal-document variables separately retain `DynamicDocument`, which owns
an immutable `Arc<Document>` and invocation identity. Existing guards reject
document switching when unqualified source-node values would become unsafe
semantically. Detached string views deliberately do not preserve node identity.

ADR-0021 already requires qualified semantic nodes and leaves cross-document
ranking unselected. Its namespace occurrence representation does not itself
settle how retained ordinary node sequences keep their owners alive.

## Ownership and constraints

XDM owns node identity and relationships. XPath owns selection, ordering,
duplicate removal, focus and node functions. The invocation runtime owns
document retention, effective views, variable frames, parameters and cleanup.
Resource resolution continues to use the supplied sealed snapshot; retaining
a node grants no new acquisition authority.

The experiment must preserve these boundaries:

- Compiled state remains stylesheet-derived. Source owners, selections,
  rankings, views and resolver state stay outside compiled/prepared artifacts.
- Identity is qualified by document origin and local node identity, not content,
  URI spelling, storage address or coincident numeric IDs. Resolution identity,
  admitted bytes and node identity remain distinct concepts.
- ADR-0012 effective relationships remain controlling. A visible node keeps
  its identity when whitespace visibility changes; hidden nodes cannot reappear
  through a variable that bypasses the effective view.
- Namespace occurrences retain ADR-0021 owner/element/prefix qualification;
  they do not become fabricated ordinary arena nodes.
- ADR-0017 lexical-frame sharing stays within one invocation. Mutation must
  detach the affected value-kind map and shadow metadata; the complete-clone
  oracle remains available.
- Selectors, focus, key indexes and document-rooted match membership must use
  the actual owning document. Swapping `source` must not reinterpret existing
  globals or leak memberships between owners.
- Charge traversal, selection retention and owner-table growth before retained
  allocation. Observe cancellation and discard partial values on failure.
- Safe Rust, the shared semantic engine and private adapter boundaries remain
  unchanged. No public provider trait, mutable DOM, ABI or cache is selected.

## Alternatives

### Keep explicit unsupported boundaries

Retain local-ID fast paths and the existing scalar document operations.
This is the safe fallback and reference checkpoint, but leaves multi-document
variables and mixed dispatch unavailable.

### Retained values with explicit immutable owners

Represent a selection with an immutable document owner and local nodes, or
qualify each member when a sequence spans owners. This makes lifetime ownership
explicit, but may add reference-count traffic, retained view state or repeated
owner handles. A small safe reference can test correctness before optimization.

### Invocation document table and qualified handles

Keep immutable owners in an invocation table and store owner tokens plus local
IDs in values. This can reduce repeated owner storage, but introduces lookup,
admission, ranking, cleanup and accounting responsibilities. Tokens cannot
escape the invocation or silently survive replacement of their owner.

### Expand detached scalar views

Continue converting values before document switches where only string,
cardinality or boolean semantics are required. This is useful for eligible
operations but cannot serve navigation, identity, copying or general unions.
It must not become an alternate interpretation of a node set.

## Findings and uncertainties

Owner qualification is necessary for the observed cases. No representation,
cross-document ranking algorithm or general document-expression grammar is
selected. The experiment must distinguish the principal source, stylesheet
documents and repeated resolution of a secondary document without inventing
identity from a hash or making cache membership the identity contract.

Template execution during global constructor initialization (`variable40`)
is a separate lifecycle question. It is not included merely because global
node values are touched here. Secondary-document DTD policy, external entities,
namespace-copy recovery and general namespace-valued variables also remain
outside the first slice.

## Disposition

Under Review for implementation follow-up. ADR-0022 accepts the explicit
retained-owner direction; compiled integration remains pending. The first
private correctness reference retains an explicit
immutable document owner alongside each local node ID. `Arc<Document>` supplies
lifetime retention, not semantic identity: document-origin qualification must
remain explicit, and pointer equality must not define node equality. Switching
the active source must never reinterpret a retained node.

Ownership and cross-document ordering are separate checkpoints. First prove
identity, lifetime, variables and effective-view behavior; exercise mixed
identity/deduplication and existing within-document order without selecting a
general cross-document ranking rule. Keep combinations requiring that rule
unsupported until a deliberate decision and tests cover them.

Only after the retained-owner reference passes should an invocation-table
candidate be compared for retained bytes, reference-count traffic, owner lookup
cost, cleanup/accounting complexity and ordinary single-document overhead.
The [accepted retained-owner ADR](../ADR/ADR-0022-explicit-retained-owners-for-runtime-source-node-values.md)
selects explicit immutable retention as the ownership direction, not a physical
layout or executable coverage. Document-origin/local-ID identity is distinct
from the effective owner's navigation relationships. Runtime integration may
now begin with `mdocs14`; unchanged executable corpus comparisons still
determine credited support. Cross-document ranking and cache-boundary expansion
remain unselected.

## Required follow-up

- [x] Build the explicit retained-owner plus local-ID safe reference, with
  separate semantic document-origin identity and no runtime grammar expansion.
- [x] Pin two distinct documents with colliding local IDs and one repeated
  secondary-document lookup; prove identity, owner lifetime and cleanup.
- [x] Exercise effective-view/reference parity, same-origin mixed normalization, namespace
  occurrence coexistence, COW shadowing and complete-clone frame parity. Do not
  infer general cross-document ranking from identity or deduplication tests.
- [ ] Prove exact budget boundaries, cancellation, denial, failed resolution,
  concurrent invocations and overlapping generations without owner leakage.
- [ ] Compare the proved retained-owner reference with an invocation-table
  candidate before selecting the production representation; measure retained
  bytes, clone/drop traffic, lookup cost and cleanup/accounting complexity.
- [x] Review identity-qualified key and match-cache dependencies before
  admitting template dispatch across documents.
- [x] Draft and accept the narrow representation ADR as ADR-0022, separating
  document-origin node identity from effective navigation ownership.
- [x] Select the principal borrowed-document adaptation after measuring its
  complete metadata-retention cost; do not deep-copy node payload to get an owner.
- [x] Integrate only `mdocs14` as the
  first compiled vertical slice: principal-source globals retained during
  secondary-document dispatch.
- [x] Then replay unchanged `mdocs16` and `select59`; classify later
  blockers individually, conserving the complete OASIS denominator.
- [ ] Measure ordinary single-document execution and retained selection memory
  before retaining a migration; local-ID fast paths require proven eligibility.
- [x] Run workspace, official WASM and corpus gates after integration. Follow
  ADR-0004 checkpoints rather than growing the runtime composition root.

## Retained owner reference on 2026-10-07

The test-only `xdm/retained_nodes_reference.rs` retains `Arc<Document>` and
local `NodeId`, delegating identity, parent and same-origin comparison to the
existing qualified source-node carrier. `Document::same_origin` compares the
existing logical origin token shared by effective views, not the address of
the `Document` allocation. No new origin scheme or production carrier is added.

Four focused tests establish distinct identity for colliding IDs and equal-byte
reparses, repeated aliases of one owner, final-owner release, parent navigation,
same-origin ordering and explicit rejection of cross-origin comparison.
Visibility views and the complete derivation preserve visible identity while
retained navigation uses their effective relationships. Borrowed namespace
occurrences remain distinct nodes whose parent is the retained ordinary node.
An exact two-visit retention allowance succeeds; one visit fails and releases
the partial sequence. Pre-cancellation releases without retained clones, and
an unbounded subsequent attempt succeeds.

The reference charge precedes each owner clone and sequence growth; it is not
a selected production accounting formula. Its repeated-owner test aliases
an already prepared owner, not repeated sealed-resource resolution. Production
focus, globals, COW frames, template dispatch, resolver failures, invocation
isolation and the table candidate remain unchecked. Namespace coexistence is
tested through the existing borrowed occurrence, not an owned namespace carrier.
Runtime grammar, authority, public APIs, unsafe surface and corpus credit are
unchanged. The module is private and compiled only for tests.

Reproduce the focused reference with:

```powershell
cargo test -p fastxslt --all-features retained_nodes_reference
```

Workspace verification passes with 1,610 core tests (57 ignored), 22 native
tests (three ignored), four WASM tests and 18 worker tests, including strict
Clippy, formatting, documentation and provenance/inventory checks. The official
WASM build-target check passes. The complete release corpus replay conserves
all totals recorded above: 2,493 exact matches, no new credited cases. Local
Markdown links and `git diff --check` pass. No performance gain is claimed.

The reference remains one cohesive test-only owner below ADR-0004's size
trigger; XDM has eight direct files after this addition. It adds no production
dependency from XDM to runtime variables, resources or template execution.

## Resolver and frame controls on 2026-10-07

Three additional tests in `runtime/dynamic_document/retained_owner_tests.rs`
use the existing sealed resolver, dynamic-document preparation and literal-
document frame bindings. The retained-node oracle is exposed only within the
crate's test build so runtime tests can consume it; XDM acquires no dependency
on runtime or resources.

`a.xml` and `./a.xml` resolve to one retained owner in the same invocation
cache. A distinct admitted resource with colliding local IDs remains a distinct
node. Independently prepared invocation caches and a separately sealed
snapshot do not alias, even when invocation scalar document identities coincide.
Dropping the cache and lookup objects leaves retained values valid; dropping
the last retained aliases releases the owner.

The existing literal-document binding map is exercised through both shared
frames and the complete-clone oracle. An alias preserves its owner after a
child replaces the original binding with another document. A child source-node
binding shadows a global document binding without changing parent fallback.
These tests cover the existing whole-document value kind, not a newly admitted
map of qualified ordinary-node sequences or compiled focus switching.

Denial is checked before returning a cached document (`FXRS0003 / denied`).
Missing resources produce `FXRS0001 / missing-resource`; an unclosed tree
produces the existing construction error `FXXD0002 / invalid`. Those failures
preserve request identity and leave cache cardinality unchanged. Pre-cancelled
preparation returns `FXCT0001 / cancelled` without installing a new owner;
a subsequent healthy lookup preserves its earlier owner.

Reproduce this tranche with:

```powershell
cargo test -p fastxslt --all-features retained_owner_tests
```

Qualified ordinary-node frame storage, mixed normalization, active compiled
focus switching, concurrent invocation stress and candidate-table measurement
remain pending. These controls do not select a representation, widen DTD or
resource authority, change production semantics or earn corpus credit.

Full workspace verification passes with 1,613 core tests (57 ignored), 22 native
tests (three ignored), four WASM tests and 18 worker tests. Strict Clippy,
formatting, documentation, local links, diff checks and the official WASM
target pass. The full release OASIS replay conserves all recorded totals,
including 2,493 exact matches. The 243-line private test module owns resolver
and frame controls only; production dynamic-document preparation gains just
a test-gated child declaration. No performance claim or public surface changes.

## Retained sequence frames on 2026-10-07

The private `runtime_context/retained_node_frames_reference.rs` experiment adds
a test-gated retained ordinary-node sequence kind to actual runtime globals
and lexical frames. Its map, initialization, clearing, alias lookup and
complete-clone hooks are absent from non-test builds. No compiled expression,
parameter or dispatch path produces or consumes this kind.

Three focused tests compare shared frames against the complete-clone oracle.
Mixed-owner sequences preserve member identity and authored sequence order;
replacing a child binding cannot reinterpret its aliases or parent values.
Global aliases retain their owner, and local source-node bindings hide global
retained values without changing parent fallback. Rebinding retained nodes
over source nodes removes the old kind. An unrelated source-node mutation
does not detach the retained map. Owners survive parent/child disposal while
a complete-clone frame still refers to them, and release after its final drop.

Visibility-view and complete-derived owners retain the same visible node
identities in both frame modes. The effective owner's relationships still
exclude stripped whitespace; the original remains unchanged. This does not
admit a general mixed-node normalization rule, cross-document ranking,
namespace-valued frames or compiled focus switching.

```powershell
cargo test -p fastxslt --all-features retained_node_frames_reference
```

The experiment extends existing frame mutation hooks rather than implementing
another interpreter. Production retained-node accounting and allocation costs
are still unselected; ordinary frame performance must be measured in non-test
builds, which omit the candidate field.

ADR-0004 review retains the 2,502-line runtime-context owner at this checkpoint:
the added hooks concern its existing frame initialization/mutation/cloning
responsibility, with reference methods and three regressions isolated in one
187-line private child. The test-only hooks must be removed or deliberately
migrated at representation acceptance; they are not permission to expand
production runtime responsibilities without the planned ADR.

Full workspace verification passes with 1,616 core tests (57 ignored), 22 native
tests (three ignored), four WASM tests and 18 worker tests. Formatting, strict
Clippy, documentation, unsafe-surface checks, local links, corpus provenance
and inventory checks, and the official WASM target pass. The full release OASIS
replay remains at 2,493 / 3,173 exact comparisons (78.57%): 2,694 initialized,
2,645 executions, 476 initialization failures, 49 execution failures, 14 XML
mismatches and four comparator gaps. Expected-error credit remains 423 / 431.
This test-only ownership checkpoint earns no additional corpus passes.

## Mixed ordering and concurrent retention on 2026-10-07

Three test-only controls extend the retained-owner reference. Same-origin
selections containing elements, derived namespaces, attributes and descendants
sort and deduplicate through the existing qualified-node comparator. Visible
identities and ordering agree through shared whitespace views and complete
derivation. Equal-byte reparses with colliding IDs and namespace prefixes remain
distinct, and cross-origin comparison returns no order. A separate identity-only
deduplication reference retains both documents' nodes; its insertion order is
not a cross-document union-order decision.

Four scoped threads each run 32 local retain/read/drop cycles over one immutable
prepared document, alternating preserving and stripping owners. A barrier makes
all four selections live together in every round. Each value uses
its effective relationships, visible identity remains stable and owner reference
counts return to baseline after all threads join. This exercises local retained
values, not shared invocation frames, concurrent frame mutation, compiled
dispatch or generation replacement.

```powershell
cargo test -p fastxslt --all-features ordering_tests
```

The private child owns only these controls. Test-local sorting and identity
deduplication do not select a production normalization algorithm, charging
formula, namespace-valued frame or cross-document ranking. The invocation-table
comparison and representation ADR remain prerequisites for runtime admission.
Corpus credit remains 2,493 / 3,173 (78.57%).

Workspace verification passes with 1,619 core tests (57 ignored), 22 native
tests (three ignored), four WASM tests and 18 worker tests. Formatting, strict
Clippy, documentation, unsafe-surface checks, local links and pinned corpus
checks pass, as does the official WASM target. This tranche changes only
test-built ownership controls; no additional full corpus replay is credited.

## Invocation table reference on 2026-10-07

The test-only `retained_nodes_reference/table_reference.rs` candidate retains
immutable owners in one local table and stores owner-index/local-ID pairs in
selections. A selection borrows its table; admission keys carry a separate table
identity and foreign-table keys are rejected before selection. Table identity
is not document identity. Repeated admission reuses the same storage owner,
while equal-origin effective views occupy separate slots to preserve their
different relationships. Equal-byte reparses remain distinct nodes.

Three controls prove identity/view parity against explicit owners, capacity and
control failure without partial owner insertion or returned selection, foreign
and missing owner rejection, and teardown. Selection release does not remove
an admitted owner: the table retains it until teardown. This is a retention
tradeoff, not permission for cross-invocation reuse or silent eviction.

On the current 64-bit native test build, selection entries in both candidates
occupy 16 bytes. The measured vector capacities are:

| Selected nodes | Explicit owner buffer bytes | Table selection plus owner buffer bytes |
| --- | --- | --- |
| 1 | 64 | 96 |
| 16 | 256 | 288 |
| 1,024 | 16,384 | 16,416 |

The table clones the document owner once, rather than once per selected node.
Those counts do not establish elapsed-time benefit. The buffer totals exclude
document storage, stack headers, table/key identity control blocks, allocator
overhead and process memory; they are not total retained memory. This candidate
does not save selection-buffer space and adds lookup and lifetime coupling.
Lookup latency, frame clone/drop costs, many-owner pressure, full allocation
accounting and ordinary-path overhead remain to be measured before choosing
either representation. The table comparison checklist therefore remains open.

```powershell
cargo test -p fastxslt --all-features table_reference -- --nocapture
```

No production carrier, runtime consumer, document ranking, grammar, authority
or public surface changes. OASIS credit remains 2,493 / 3,173 (78.57%).

Workspace verification passes with 1,622 core tests (57 ignored), 22 native
tests (three ignored), four WASM tests and 18 worker tests. Formatting, strict
Clippy, documentation, unsafe-surface checks, local links, pinned corpus checks
and the official WASM target pass. No new corpus execution is credited for this
test-only comparison.

## Reference timing and allocation on 2026-10-07

The private `table_measurements.rs` probe compares retained ordinary-node
references, not compiled execution or host throughput. Owners and input ID
vectors are prepared before measurement. Release timing omits allocator
instrumentation, rotates paired lane order over five trials, and runs each phase
for at least 20 ms. Construction includes selection creation and drop with
unbounded controls; table admission is excluded and measured separately.
Lookup extracts every borrowed carrier, without evaluating XPath or navigation.
Clone/drop deep-copies the selection vector; this is not the normal COW frame
clone path. Medians from one process on the current machine, in nanoseconds per
complete selection operation, are:

| Nodes with one admitted owner | Explicit construct/drop | Table construct/drop | Explicit lookup all | Table lookup all | Explicit clone/drop | Table clone/drop |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 59.7 | 54.5 | 0.8 | 1.1 | 24.7 | 24.2 |
| 16 | 170.0 | 140.8 | 12.9 | 15.3 | 57.8 | 25.3 |
| 1,024 | 4,372.4 | 2,051.0 | 823.7 | 976.3 | 3,636.7 | 229.7 |

The table reduces repeated owner-reference operations for large selections but
adds carrier lookup cost. Repeated admission of the last owner, with one-node
selections in the matrix, takes median 27.6 / 46.7 / 101.9 ns with 1 / 16 / 64
owners: this candidate scans the table. These local measurements do not justify
a compiled-performance claim or a production table.

Allocation scopes separately include table construction, keys, controls,
selection collections and teardown. Existing immutable documents and input ID
vectors are outside the counter. Each owner receives one selection; values
coexist before drop. All nine cells return both live requested bytes and live
allocation count to zero, and document-owner reference counts return to baseline.
Total requested and peak live requested bytes are:

| Owners | Nodes per selection | Explicit total / peak bytes | Table total / peak bytes |
| --- | --- | --- | --- |
| 1 | 1 | 112 / 112 | 208 / 184 |
| 1 | 16 | 496 / 432 | 592 / 504 |
| 1 | 1,024 | 32,752 / 24,624 | 32,848 / 24,696 |
| 16 | 1 | 1,792 / 1,432 | 2,800 / 1,960 |
| 16 | 16 | 7,936 / 4,632 | 8,944 / 5,160 |
| 16 | 1,024 | 524,032 / 270,744 | 525,040 / 271,272 |
| 64 | 1 | 7,168 / 5,656 | 11,248 / 7,720 |
| 64 | 16 | 31,744 / 18,072 | 35,824 / 20,136 |
| 64 | 1,024 | 2,096,128 / 1,058,328 | 2,100,208 / 1,060,392 |

The table allocates more total and peak requested bytes in every measured cell.
These scopes are not allocator overhead, RSS or document-memory estimates.
Reproduce independently with:

```powershell
cargo test -p fastxslt --release --features workbench retained_owner_reference_timing -- --ignored --nocapture
cargo test -p fastxslt --all-features retained_owner_reference_allocation_scopes -- --nocapture
```

The borrowed-table prototype also prevents mutable admission while a selection
is live. Dynamic document lookup would need a different stable ownership/mutation
seam before this shape could become a runtime carrier. That additional design,
owner retention until teardown and slower lookup must be weighed against the
large-vector clone benefit. Neither prototype is selected; integrated COW-frame
cost, dynamic admission, mixed-owner consumers and ordinary compiled execution
remain outside these measurements. Production behavior and corpus credit are
unchanged.

Full workspace verification and the official WASM target pass. The allocation
regression runs in the normal all-features gate; the release timing probe remains
manual and ignored there. Formatting, strict Clippy, documentation, local links,
unsafe-surface and pinned-corpus checks pass. This is a reference comparison,
not additional conformance credit.

## Live frame admission and cache review on 2026-10-07

Two test-only controls combine actual runtime frames with the sealed dynamic
document resolver. In both COW and complete-clone modes, admitting a second
document while first-owner values remain live preserves parent bindings and
child aliases, despite colliding local IDs. Failed third-resource preparation
does not grow the cache. Repeated normalized resolution reuses the first
storage owner. Values survive cache teardown, and final frame disposal releases
both owners.

Four concurrent scopes overlap two immutable snapshots with the same logical
resource identity but different bytes. Each scope prepares its own document,
retains its own frame values and uses either COW or complete cloning. A barrier
keeps all four frames live together. Local cross-kind rebinding leaves parent
values and aliases unchanged; final frame disposal releases each document.
This proves snapshot/frame isolation, not compiled generation replacement or
admitted focus switching.

```powershell
cargo test -p fastxslt --all-features live_admission
```

The cache migration review preserves current source-context confinement:

| Private owner | Existing key and payload | Required integration constraint |
| --- | --- | --- |
| `key_lookup::KeyIndexCache` | Expanded key name to value/local-node-ID lists | Never use an index built against another effective source; evaluate use/match expressions against the actual source owner |
| `template_selector::DocumentRootedMatchCache` | Compiled template index to local-node-ID membership bits | Preserve ADR-0013 source/program confinement and bounded charged construction/fallback |
| `SequenceInputs` | One source plus both invocation caches | Preserve fresh caches at document switches until any owner-qualified cache extension is deliberately reviewed |

`external_document_inputs`, literal-document `for-each` and literal-document
apply-template contexts currently create fresh key and match caches. Their
invocation-owned dynamic-document cache is shared only for admitted immutable
resource preparation. That does not make semantic indexes reusable across
document switches. A compiled migration must keep this distinction; ordinary
node qualification alone cannot validate an old membership or key index.

Logical origin is also insufficient as a future index key: equal-origin
preserving and stripping owners can expose different effective relationships.
Reuse would require both semantic owner/view context and compiled context, not
just a URI, scalar dynamic-document ID or local node ID. No such cache extension
is selected here. The first `mdocs14` integration should retain fresh
source-context caches rather than expand their ownership boundary.

The explicit retained-owner reference supports live admission without borrowing
a mutable owner table through its values. Together with the measured table
tradeoffs, this makes it the simpler candidate for the narrow representation
ADR. That is a nomination, not acceptance or corpus support. Cross-document
ranking, namespace-valued frames and compiled consumer migration remain open.

The new private 206-line child owns live-admission controls only; it adds no
production hook. The core suite passes with 1,625 tests and 58 ignored probes,
alongside 22 native tests, four WASM tests and 18 worker tests. This checkpoint
does not rerun or increase the OASIS exact numerator. Full workspace gates and
the official WASM target pass, including formatting, strict Clippy, documentation,
links, unsafe-surface and pinned-corpus checks.

## Principal borrowed-owner adaptation on 2026-10-07

The execution entry point currently borrows the effective principal document.
The private test-only `xdm/retained_owner_adaptation.rs` probes conversion to a
retained `Arc<Document>` without reparsing or copying immutable node records.
It shares the existing origin and node storage, copies only effective child
overrides and typed-ID lookup metadata, and charges metadata before retained
growth. This is a reference candidate, not a production migration or a selected
charge formula.

Two correctness controls establish preserved origin/local-ID identity despite
different effective relationships, identical provenance and ID lookup metadata,
survival after the original borrowed owner is dropped, equal-byte reparse
separation, exact N/N-1 budget boundaries, and cancellation at every actual
charge checkpoint. Failed adaptation publishes no owner and leaves node-storage
reference counts unchanged. A third allocation-observation control covers six
cells and returns all experiment-owned allocations to baseline after drop.

| Items | Shape | Adaptation requested total bytes | Adaptation requested peak bytes | Median construct/drop ns |
| --- | --- | ---: | ---: | ---: |
| 50 | Preserve | 160 | 160 | 74.2 |
| 50 | Strip | 1,668 | 936 | 460.2 |
| 50 | Typed IDs | 8,174 | 5,815–5,816 | 6,037.9 |
| 500 | Preserve | 160 | 160 | 66.2 |
| 500 | Strip | 12,436 | 8,120 | 1,350.6 |
| 500 | Typed IDs | 119,218 | 92,133–92,136 | 68,307.3 |

Timings are five release windows of at least 20 ms per cell, without the
allocation-observation feature. They include candidate construction and drop,
not XML preparation, compiled execution or a host boundary. Allocation probes
exclude the already-prepared source; totals count cumulative requested growth,
not retained or OS memory. Small peak variation reflects hash-map layout.
The typed-ID fixture injects parser-event metadata solely to isolate retention
cost; it is not a DTD or corpus claim. Preserving/stripping fixtures contain
50 or 500 repeated elements, with whitespace relationships kept or stripped.

The preserving-source wrapper is small, but copied typed-ID metadata becomes
material at 500 items. Do not adopt unconditional metadata copying as the
production owner adaptation from this probe. Compare immutable metadata
sharing or another bounded adaptation before compiled admission, then measure
the complete integration and ordinary single-document path. ADR-0022 already
permits private physical representation choices; no new host knob, owner table,
cache scope or public contract follows from this observation.

Reproduction:

```text
cargo test -p fastxslt --all-features retained_owner_adaptation -- --nocapture
cargo test -p fastxslt --release --features workbench borrowed_owner_adaptation_timing -- --ignored --nocapture
```

The 257-line child owns only the adaptation reference and its controls. XDM
has nine direct source files after this addition, below ADR-0004's directory
density trigger. Runtime composition and upstream corpus bytes are unchanged.
No corpus replay or new credit is claimed: 2,493 / 3,173 (78.57%) remains the
last measured checkpoint, with 46 further exact comparisons needed for 80%.

Full workspace verification passes with 1,628 core tests and 59 ignored probes,
22 native tests, four WASM tests and 18 worker tests, alongside formatting,
strict Clippy, documentation, Markdown links, unsafe-surface and pinned-corpus
checks. The official `wasm32-unknown-unknown` build check also passes. The
release-only probe emits existing feature-specific unused test-hook warnings;
the all-feature strict Clippy gate passes without a source-code exemption.

## Shared immutable metadata comparison on 2026-10-07

The private `Document` representation now holds its source-derived ID lookup
map and effective child-override map behind immutable `Arc` storage. ID-map
mutation uses safe exclusive/COW access only while the source is constructed.
A whitespace view shares immutable source ID metadata, while its relationship
overrides remain owned by that invocation's effective document. Nothing is
interned across document origins, snapshots or invocations. The complete
derivation and adaptation references still copy their metadata independently.

The test-only shared adaptation charges one header-retention unit before
cloning immutable handles. It does not inspect, copy or reparse source payload.
Both copied and shared owners use the same accessors and origin/local-ID
identity. Controls compare effective child navigation and provenance, verify
metadata sharing versus the independent-copy oracle, and prove node storage,
ID metadata and view relationships are reclaimed after the last owner drops.
Exact 1/0 adaptation budgets and pre-retention cancellation are pinned separately
from the copying candidate's metadata-dependent reference charges.

| Items | Shape | Copy requested total bytes | Share requested total/peak bytes | Copy median ns | Share median ns |
| --- | --- | ---: | ---: | ---: | ---: |
| 50 | Preserve | 144 | 80 / 80 | 85.6 | 56.2 |
| 50 | Strip | 1,716 | 80 / 80 | 452.9 | 57.6 |
| 50 | Typed IDs | 8,158 | 80 / 80 | 5,546.8 | 54.3 |
| 500 | Preserve | 144 | 80 / 80 | 86.9 | 55.9 |
| 500 | Strip | 12,484 | 80 / 80 | 1,463.8 | 60.2 |
| 500 | Typed IDs | 119,202 | 80 / 80 | 72,015.3 | 64.1 |

These are twelve allocation cells and five paired, alternating-order release
windows of at least 20 ms per cell. Both lanes include owner construction and
drop; XML preparation, execution and host transfer are outside their scope.
Every allocation cell ends at zero experiment-owned bytes and allocations.
The shared wrapper makes two allocations and its measured cost is independent
of metadata cardinality in these fixtures. Copy numbers differ from the earlier
table because the document's private inline header has changed; the earlier
measurements remain historical observations, not overwritten results.

Known-capacity accounting now includes the out-of-line immutable ID-map header
and its reference counters, and effective-view accounting includes its map
header and counters. Shared capacity remains conservatively attributable to
each document owner; summing such estimates is not unique allocation ownership
or RSS. No admission estimate is made smaller by silently omitting the newly
out-of-line headers.

This clears the metadata-copy cost concern for the borrowed-owner candidate.
Metadata sharing alone does not admit a cross-document consumer. The retention
adapter is still test-only; full `mdocs14` dispatch and ordinary host-visible
integration costs remain pending. The ordinary source-value guard is unchanged
rather than allowing unmigrated bare-ID consumers to misinterpret another
document's IDs. The separate bound-document dispatch slice is described below.
The 330-line adaptation child remains cohesive and XDM remains at nine direct
source files. No unsafe code, adapter representation or upstream fixture changed.

## Guarded bound-document dispatch on 2026-10-07

The fresh full replay first reports `mdocs14` at `FXRT0007`: applying templates
to `$html-template` is rejected before principal source-value ownership is
tested. The production slice now lets a bound literal secondary-document value
enter the ordinary template executor, with its frame-retained immutable owner
live throughout the borrowed focus. Whitespace composition precedes dispatch;
key and document-rooted match caches are fresh for the effective source. Root
selection supplies focus position/size 1/1 and preserves mode and atomic
parameters. Sorted document-variable selection remains explicitly unsupported.

Two compiled regressions verify a local alias of a global sealed document,
colliding principal/secondary shapes with distinct values, selected mode,
atomic parameter transfer, COW/complete-clone frame parity, complete-reference/
visibility-view source parity, four concurrent invocations, denial, cancellation,
budget failure and subsequent fresh recovery. Ordinary principal node globals
and nonempty node parameters remain rejected by `FXRT1017`; the new dispatch
does not reinterpret those bare IDs against the secondary document.

The private 43-line `document_variable_dispatch` owner contains only dispatch
composition and delegates execution to the existing template path. Its 171-line
test owner groups compiled document-value controls. The runtime test directory
has eight direct files; the execution child directory has 15, triggering a
cohesion inspection but not mandatory regrouping. Keep this named dispatch seam
separate from resource acquisition and preparation; regroup a growing document
dispatch family by that semantic responsibility rather than by file count.
The large parent gains only wiring and one dispatch branch; existing
composition-root decomposition debt remains, not declared resolved.

No principal-source carrier migration or general cross-document ranking is
inferred. Full `mdocs14` support still requires `$xml-source` to carry its own
effective owner on the return from secondary focus.

The unchanged complete OASIS CD04 replay conserves every recorded total:
2,493 / 3,173 exact XML comparisons (78.57%), 2,694 initialized, 2,645 executed,
476 initialization failures, 49 execution failures, 14 XML mismatches, four
comparator gaps and 423 / 431 expected-error credits. `mdocs14` now reaches
`FXRT1017` rather than `FXRT0007`; it remains explicitly unsupported and
uncredited. The existing negative regression now tests sorted bound-document
dispatch and `for-each`, retaining their diagnostic provenance checks instead
of expecting the newly admitted unsorted operation to fail.

Reproduction uses the existing focused commands above plus:

```text
cargo test -p fastxslt --all-features document_variable_dispatch
./scripts/measure-oasis-xslt10.ps1 -TraceCase mdocs14
```

Final workspace verification passes with 1,630 core tests (59 ignored), 22
native tests, four WASM tests and 18 worker tests. Formatting, strict Clippy,
documentation, local links, unsafe-surface and pinned-corpus checks pass;
the official `wasm32-unknown-unknown` build also passes. Work remains uncommitted.

## Reopening triggers

Reconsider scope if qualification forces a second executor, retains invocation
state beyond its owner, bypasses whitespace views or resource controls, requires
public representation changes, or materially regresses ordinary node paths.

## Review history

- 2026-10-06 -- Opened as Incubating after the conserved corpus replay and
  literal-document decomposition checkpoint. No engine semantics changed.
- 2026-10-07 -- Monday supports the ownership seam and recommends the explicit
  retained-owner correctness oracle before the table candidate. Separate
  ownership from ranking; keep `mdocs14` as the first compiled integration
  target. Review guidance does not select a production representation or
  credit additional corpus cases.
- 2026-10-07 -- The four-test retained ordinary-node reference passes. Resolver
  reuse and production frame/focus integration remain separate follow-ups.
- 2026-10-07 -- Three resolver/frame controls pass, proving actual sealed
  owner reuse and existing literal-document COW/complete-clone parity. General
  qualified-node frame and compiled focus integration remain open.
- 2026-10-07 -- Three additional retained-sequence frame tests pass through
  real test-gated runtime frame hooks. Compiled focus, mixed normalization,
  concurrency stress and table-candidate comparison remain open.
- 2026-10-07 -- Three mixed-order/concurrent-retention controls pass. Existing
  same-origin order and identity-only cross-origin deduplication are conserved;
  general cross-document ranking and compiled admission remain unselected.
- 2026-10-07 -- Three invocation-table reference controls pass. The candidate
  reduces document-owner clones but adds retention and lookup responsibilities,
  with no selection-buffer saving. Timing and full accounting remain open.
- 2026-10-07 -- Rotated release reference timings and nine complete allocation
  scopes show cheaper large-vector clone/drop but slower lookup and higher
  allocation pressure for this table. Dynamic admission while values are live
  remains a representation constraint; no production choice is inferred.
- 2026-10-07 -- Two real-frame/live-admission controls pass in both clone modes
  and concurrent snapshot scopes. Reviewed key/match-cache confinement; keep
  fresh source-context caches during the first compiled migration. Explicit
  retained owners are nominated for the representation ADR, not admitted.
- 2026-10-07 -- Drafted the unnumbered proposed retained-owner ADR. It separates
  ownership direction from physical layout and compiled admission, retains
  fresh source-context caches and leaves ranking and runtime parity unselected.
- 2026-10-07 -- Maintainer accepted ADR-0022 with the normative identity versus
  navigation-owner distinction. AR remains Under Review for compiled integration
  and validation; no anticipated corpus passes are credited.
- 2026-10-07 -- Added bounded borrowed-principal owner adaptation controls and
  six allocation/timing cells. Node payload remains shared, but copied typed-ID
  metadata is material; compare immutable metadata sharing before production
  migration. All workspace and official WASM gates pass. No new corpus credit.
- 2026-10-07 -- Shared immutable metadata removes cardinality-dependent copying
  from the adaptation reference. Added guarded bound-document root dispatch
  through the ordinary executor; principal ordinary-node values remain guarded.
  The complete OASIS replay conserves 2,493 exact cases, with `mdocs14` now
  exposing the remaining `FXRT1017` ownership boundary rather than the prior
  secondary-variable rejection. No public carrier, ranking or cache extension.

## First compiled retained-owner slice on 2026-10-07

The unchanged OASIS `Lotus/mdocs_mdocs14#1` now compares exactly. Principal
source-node globals share one immutable retained effective document per
invocation; lexical aliases retain that owner together with their local IDs.
Owner-aware `apply-templates` switches navigation to that owner through the
existing template executor. A proved same-effective-storage selection can use
the current context; other switches receive fresh source-context key and
document-rooted match caches. Storage eligibility is deliberately stricter
than node identity, and is not an identity component.

The migration is bounded. Every remaining production local-ID variable
navigation lookup now checks effective-owner eligibility. A nonempty foreign
selection returns `FXRT1017` rather than navigating coincident IDs against the
current document. Identity-free presence/boolean checks do not navigate IDs.
Bare node parameters, sorted document-variable dispatch, broader foreign-owner
scalar/path consumers and general cross-document ranking remain unsupported.
There is no owner table, alternate executor, ambient acquisition, public
carrier, wire change or new unsafe surface.

Compiled controls now establish:

- Principal return through a lexical alias under both whitespace representations
  and both COW/complete-clone sequence frame paths.
- Concurrent invocations using separately compiled old/new generations.
- Exact consumed XDM budget versus one-less failure, cancellation during
  template-candidate charging, and successful fresh invocation after failure.
- Different key values and document-rooted match membership for colliding
  local IDs, with cached/uncached parity after secondary-to-principal switching.
- Explicit denial, unmigrated foreign-navigation rejection, cross-kind
  shadowing and final retained-owner release. Allocation scopes return all
  requested bytes and allocation counts to zero after execution/result disposal.
- Shared/copy owner-adaptation differential tests distinguish visible identity
  from effective storage, preserving source locations and typed-ID metadata.

The owner-selection frame map is lazy: ordinary frames without qualified
aliases allocate no new empty map. Complete-clone frames preserve this absence
and independently copy present selection maps while retaining immutable owners.

### Integrated cost observations

One ordinary single-document control compares direct root dispatch with a
principal-global root selection producing identical XML. Compilation and source
preparation are excluded; global initialization, controlled execution,
serialization and disposal are included. This comparison includes the additional
global/selection work, not just `Arc` retention, and is not a pre/post engine
regression measurement or a host throughput claim.

With the final lazy map, requested allocation totals are 3,934 bytes / 37
allocations direct versus 4,543 bytes / 40 allocations retained-global dispatch.
Peak requested bytes are 1,766 versus 2,337. Five rotated 30 ms release windows
give median complete calls of 1,857.5 ns direct versus 2,005.9 ns retained
(about 148 ns additional work on this tiny fixture). The same output is checked
before timing. Broader workload/host measurements remain an independent gate.

The secondary-document fixture with an unused principal global records 14,299
versus 14,916 total requested bytes, and 113 versus 117 allocations; it includes
secondary resolution/preparation as well as dispatch. These requested-byte
scopes exclude fixture construction and do not measure RSS, pipe buffers or
allocator/OS reclamation.

### Conserved corpus and next blockers

The complete pinned OASIS CD04 replay reports 2,494 / 3,173 exact XML
comparisons (78.60%): 2,694 initialized, 2,646 executed, 476 initialization
failures, 48 execution failures, 14 XML mismatches, four comparator gaps and
423 / 431 expected-error credits. Only `mdocs14` earns new exact credit.

Independent unchanged traces classify subsequent targets:

- `mdocs16`: initialization remains explicitly unsupported (`FXXP1008`) for
  `($terms/table/expand[@language=$lang]/gentext[@key=$key])[1]`.
- `select59`: initialization remains explicitly unsupported (`FXXP1001`) for
  the `document('select59.xml')/child` branch of a union. General cross-document
  order is not inferred from the retained-owner dispatch decision.

The next work must admit these operations independently rather than treating
owner retention as universal multi-document support. AR-0028 remains Under
Review; the representation decision is accepted, the first compiled slice is
executable, and broader consumer/host cost gates remain open.

Validation passes formatting, strict workspace Clippy, 1,636 core tests
(60 manual probes ignored), 22 native tests (three probes ignored), four WASM
adapter tests, 18 worker tests, documentation, 1,981 Markdown-file link checks,
pinned conformance inventory, `git diff --check` and the official WASM target
check. The final failure-path allocation control also returns requested live
bytes/counts to zero after denied, exhausted and actively cancelled invocations.
The new production responsibilities stay in private 133-line retained-selection
and 88-line document-dispatch owners; they do not introduce a cross-layer API.

Reproduction:

```text
cargo test -p fastxslt --all-features document_variable_dispatch -- --nocapture
cargo test -p fastxslt --all-features retained_source_frames
cargo test --release -p fastxslt --features workbench ordinary_retained_dispatch_timing -- --ignored --nocapture
./scripts/measure-oasis-xslt10.ps1 -TraceCase mdocs14
./scripts/measure-oasis-xslt10.ps1 -TraceCase mdocs16
./scripts/measure-oasis-xslt10.ps1 -TraceCase select59
./scripts/verify.ps1
cargo check -p fastxslt-wasm-workbench --all-features --target wasm32-unknown-unknown
```

## Bounded variable-path integration follow-up — 2026-10-07

The next production slice admits grouped first-node selection over a retained
document variable's named child paths, with optional attribute-equals-atomic-
string filters. It uses the ordinary charged XPath evaluator and retains the
actual effective owner through selected elements, attributes and lexical aliases.
XSLT 1.0 string conversion reads that owner, never the current focus's coincident
local ID. Grouped `[1]` applies after all parents are evaluated. Missing attributes
remain distinct from empty attribute values; numeric filter operands remain
explicitly unsupported, and missing variables remain invalid input.

Unchanged `mdocs16` now compares exactly. Its first conserved replay reports
2,495 / 3,173 exact XML comparisons (78.63%), 2,695 initialized, 2,647 executed,
475 initialization failures, 48 execution failures, 14 XML mismatches, four
comparator gaps and 423 / 431 expected-error credits. This is executable credit,
not credit inferred from ADR acceptance.

Focused controls cover grouped selection across multiple parents, retained
attribute aliases, first-node focus, missing-versus-empty attributes, both
whitespace representations, COW/complete-clone frames, concurrent separately
compiled fixtures, exact XDM retention limits, active cancellation, denied
authority and complete allocation release after success and partial failure.
The compiler rejects unadmitted syntax rather than rewriting arbitrary XPath.

The next `select59` experiment admits a literal document union branch only when
it evaluates empty. It still resolves and parses the resource normally, preserving
failure and control outcomes. Nonempty document branches remain unsupported;
there is no cross-document ranking decision. An explicit first-party import
mapping seals the unchanged `select59.xml` bytes at the stylesheet-relative
identity, distinct from the principal input's `source/` identity, following the
existing `select68` mapping. Engine execution gains no acquisition authority.
The final conserved replay also credits unchanged `select59`: 2,496 / 3,173
exact comparisons (78.66%), 2,697 initialized, 2,648 executed, 473 initialization
failures, 49 execution failures, 14 XML mismatches, four comparator gaps and
423 / 431 expected-error credits. Two previously unsupported initializations
now reach execution; only independently exact comparisons increase the numerator.

Validation passes formatting, strict workspace Clippy, 1,642 core tests
(60 probes ignored), 22 native tests (three probes ignored), four WASM adapter
tests, 18 worker tests, documentation and 1,981 Markdown-file link checks.
The official `wasm32-unknown-unknown` adapter build and `git diff --check` pass.
The repository verification script additionally checks pinned conformance and
unsafe surfaces. Empty-union controls preserve same-owner order/deduplication,
explicit nonempty rejection, missing-resource `FXRS0001`, exact XPath work
limits and active cancellation. Broader consumer/host cost gates remain open.

### Cohesion review

The new static recognizer is a private 119-line compiler module. Owner-preserving
binding and eligible-path evaluation are extracted into a private 231-line
runtime module; it owns selection/binding, not acquisition policy, template
dispatch, caches or host state. Compilation contains no runtime owners.

ADR-0004 review remains required: the compiler parent is 3,382 lines and runtime
composition root 6,672 lines; their child directories contain 19 and 16 direct
files respectively. The bounded slice finishes existing owner migration without
unrelated structural churn, but does not resolve this size/directory debt.
Before further owner-path or nonempty document-union expansion, review extraction
of mixed-source selection/template orchestration into a named private owner and
group related selection modules where dependency direction remains clear.
Numbered ADR/review directories are not mechanically split to satisfy a count.

### Source-selection cohesion checkpoint — 2026-10-07

Before further semantic expansion, key and mixed-source union selection now
reside in a private 91-line `source_selection/unions.rs` owner. The existing
231-line bound-variable path owner is grouped beside it. The folder names a
related subject, not an arbitrary file-count bucket. Module visibility remains
private and both units retain the same composition-root dependency direction.

The union owner reads the active effective source, compiled alternatives and
lexical values; it delegates charged paths, key/ID lookup and guarded empty
document branches to their existing owners, then restores single-document order
and deduplicates. It neither acquires resources nor owns template dispatch,
cache policy, invocation admission or a new mixed-document carrier. No broad
context or parent forwarding API was introduced; the existing selection inputs
are unchanged. The move deliberately changes no charge formula, failure ordering,
sorting, allocation, public API, ABI or unsafe surface.

The runtime composition root falls from 6,672 to 6,591 lines; its direct-file
child directory falls from 16 to 15, with two related selection files in the
new subdirectory. The remaining root still has decomposition debt. Retain it
for this conserved extraction checkpoint; the next owner-aware dispatch or
selection expansion must review its specific orchestration seam before adding
new semantic branches. The compiler's 19-file directory remains separately
pressured; a runtime grouping does not retire compiler debt. This is not a
performance optimization or a claim that all source-size pressure is resolved.

The post-extraction full OASIS CD04 replay conserves every recorded counter:
2,496 / 3,173 exact XML comparisons (78.66%), 2,697 initialized, 2,648 executed,
473 initialization failures, 49 execution failures, 14 mismatches, four
comparator gaps and 423 / 431 expected-error credits. Full repository verification
passes again (1,642 core tests, 60 ignored; native 22/3 ignored; WASM four;
worker 18), including strict Clippy, formatting, documentation, local links and
pinned conformance/unsafe checks. The official WASM build also passes. Build
commands waited for existing editor/build locks; this is not a clean-build or
incremental-build speed comparison, and no compile-time improvement is claimed.

The refreshed runtime `FXRT1017` trace is
`Microsoft/XSLTFunctions_DocumentInUnion#1`: its document branch is nonempty,
so the empty-branch eligibility rule deliberately does not admit it. General
cross-document ranking remains unselected. Missing stylesheet dependency traces
include historical HTTP/server identities and genuinely absent files; this
checkpoint adds no import aliases and makes no network acquisition attempts.
The selected unsupported-operation and authority boundaries are conserved,
not weakened to manufacture a numerator gain.

## Retained single-owner iteration slice — 2026-10-07

`xsl:for-each select="$selection"` can now iterate an ordinary retained source
selection against its effective owner. The existing sort evaluator and sequence
executor handle both same-source and foreign-owner paths; this is not another
interpreter. Foreign focus creates fresh source-context key/match caches,
preserves the lexical frame, charges the selected-ID copy before allocation and
sets position/size from the selected sequence. Nested principal/secondary owner
switches restore the enclosing focus naturally.

This slice does not admit the separate bound document-root variable kind,
arbitrary document-expression iteration, direct variable-path iteration,
bare lexical source selections crossing owners, namespace-valued frames or
mixed-document ranking. A retained variable does not make unrelated bare IDs
safe. Existing explicit unsupported guards remain.

The first-party MIT golden fixture
`corpus/golden/retained-source-for-each` is hand-authored, not copied from OASIS.
It covers a retained secondary selection, lexical alias, numeric descending
sort, position/size, stripped effective relationships, nested principal return,
key-cache separation and outer-focus restoration with colliding local IDs.
Focused tests compare principal whitespace view/complete derivation and
COW/complete-clone frames, concurrent invocations, exact XDM retention versus
one-less limits, cancellation, denied authority, unqualified transfer rejection,
and zero live requested allocations after success/cancellation/exhaustion.

The implementation stays in the existing private 401-line `for_each` owner,
which owns iteration/focus construction and delegates body execution, sorting,
resource preparation and source-context construction. Its directory has two
direct files. The 6,591-line composition root is not extended by this slice;
no compiler parent, public facade, ABI, wire representation or unsafe surface
changes. This follows the preceding source-selection cohesion checkpoint.

The complete unchanged OASIS CD04 replay conserves 2,496 / 3,173 exact XML
comparisons (78.66%), 2,697 initialized, 2,648 executed, 473 initialization
failures, 49 execution failures and 423 / 431 expected-error credits. This
integration earns golden evidence, not new OASIS credit. The remaining
`DocumentInUnion` guard still needs a separate general ordering decision;
`idkey49` additionally requires mixed-document generate-ID expressions, while
`idkey52` needs source-selected document-root iteration plus owner preservation
for its lexical key argument. These are independently classified next slices,
not implicitly admitted by this one.

The final integration gates pass 1,645 core tests (60 ignored), 22 native tests
(three ignored), four WASM adapter tests and 18 worker tests. Formatting, strict
workspace Clippy, documentation, 1,982 Markdown-file link checks, conformance
inventories and unsafe-surface verification pass; the official WASM target also
builds. The late-cancellation cleanup probe derives its interruption point from
the measured complete XPath visit count rather than a brittle fixed visit count.
Requested live bytes and allocation counts return to zero after every measured
outcome is dropped. Broader host-cost measurements remain open; no performance
benefit or complete multi-document support is claimed.

## Zero/single source-selected document iteration — 2026-10-07

The next bounded runtime slice admits XSLT 1.0 `for-each` over a source-selected
`document(path)` root, or an existing typed source-document path selection, only
when sealed resolution produces zero or one effective document. Empty selections
perform no body execution or unselected-resource acquisition. Multiple resolved
documents report `FXRT1017`; repeated references to one resolved document retain
the existing invocation cache/deduplication behavior. No general cross-document
ranking, new resource authority or alternate executor is selected.

Before a nonempty focus switch, ordinary lexical node selections are qualified
with their current immutable effective owner in an isolated COW frame. Owner and
ID retention are charged before copying; the original frame remains unchanged.
The ordinary sequence/sort executor handles the body with fresh effective-source
key/match caches. Key lookup converts a retained node argument through its
lexical owner, but the searched key index remains attached to the current focus
document. Identity qualification does not authorize cache reuse across views.

Three focused tests independently exercise nonempty root iteration, retained
aliases, nested return to principal focus, key lookup, repeated references,
position/size, whitespace view/complete derivation, COW/complete frames and four
concurrent invocations. Empty, denied, missing and multiple-document selections
are distinguished. Exact XDM budget versus one-less and late cancellation are
checked; success, rejection, missing-resource, cancellation and exhaustion leave
zero live requested allocations after outcomes are dropped.

The unchanged OASIS CD04 replay gains `Lotus/idkey_idkey52#1`: 2,497 / 3,173
exact XML comparisons (78.70%), 2,698 initialized, 2,649 executed and 472
initialization failures. Its actual `bibref` focus selects no `refdoc` children,
so that corpus pass proves the empty branch, not nonempty cross-document key
behavior. The independent positive fixture supplies that evidence. Execution
failures remain 49, mismatches 14, comparator gaps four, and expected-error
credits 423 / 431. Imported bytes and expected results are unchanged.

ADR-0004 review retains the existing iteration owner (458 lines, two direct
files), lexical retained-value owner (166 lines) and key-value consumer (685
lines). The compiler's existing document-selection owner (1,673 lines) shares
typed parsing rather than adding a root-only parser; the 3,395-line compiler
composition root adds only the compatibility-gated instruction choice. The
6,591-line runtime composition root is unchanged. Existing compiler/root and
19-file compiler-directory decomposition pressure remains explicit; this small
selection extension does not retire that debt or add a cross-layer dependency.

Repository verification passes 1,648 core tests (60 ignored), 22 native tests
(three ignored), four WASM adapter tests and 18 worker tests. The key argument
fixture also agrees with the complete charged evaluator with key-index caching
disabled. Strict Clippy, formatting, documentation, local Markdown links,
conformance inventories and unsafe-surface checks pass. The official
`wasm32-unknown-unknown` adapter build passes. No host-performance gain or broader
document-expression support is claimed by these semantic tests.

## Current template rule conservation — 2026-10-07

The ordinary frontier exposed an independent dynamic-context defect in
unchanged `Lotus/impincl_impincl18#1`. A single `/` template entered the private
direct-root fast path without matched-rule identity, causing `apply-imports`
to report absent current rule instead of reaching the built-in rule. Delegating
root rules now use ordinary matched-template dispatch. The eligibility check
follows reachable local named calls with cycle suppression; a call unresolved
within the module is conservatively ineligible pending normal module linking.
Unrelated delegation in other matched rules does not disable the root fast path.
This is stylesheet-static eligibility, not invocation state or a new executor.

The associated context audit repairs ordinary, retained, temporary, literal-
document and qualified `for-each` focus construction: its body has no current
template rule, while named calls preserve the caller's rule and outer iteration
return restores it. Unsupported namespace-call and static-range body combinations
are not admitted just to exercise this rule. Continuation failures acquire the
invoking instruction location only when they lack a more specific location.
Request identity, category and nested provenance remain intact.

Before this repair, top-level template declaration routing was extracted
unchanged into the private `template_declaration_compiler.rs` owner. Full
verification and the OASIS replay conserved 2,497 exact comparisons, 2,698
initialized, 2,649 executed and 49 execution failures at that mechanical
checkpoint. The owner consumes parsed declaration/static context and constructs
named/matched template collections and direct-root eligibility; it delegates
pattern/body compilation and does not own runtime caches, authority, focus,
public APIs or host policy. The compiler composition root is now 3,065 lines;
its remaining compilation responsibilities and directory density remain explicit
ADR-0004 debt. No generic mutable context or parent forwarding layer was added.

Four focused regressions cover 1.0/3.0 root fallback, modern `next-match`, named
calls and included delegates, unchanged ordinary-root eligibility, admitted
source/variable/temporary iteration clearing, outer restoration, absent initial
named rule, COW/complete frames, whitespace view/complete derivation, located
errors, exact instruction budgets and late cancellation. Two modern include
tests deliberately update private matched-rule inventory from empty to `[0]`
because an external named call now retains the principal root rule. Their
upstream fixtures, expected results and exact result assertions are unchanged.

The final unchanged OASIS CD04 replay gains `impincl18`: 2,498 / 3,173 exact
XML comparisons (78.73%), 2,698 initialized, 2,650 executed, 472 initialization
failures and 48 execution failures. Mismatches remain 14, comparator gaps four
and expected-error credits 423 / 431. Eighty percent still requires 41 exact
cases; invalid archived inputs, unavailable resources and deferred compatibility
or entity semantics are not reclassified as success to reach that target.

Normative references: [XSLT 1.0 template overriding](https://www.w3.org/TR/xslt.html#section-Overriding-Template-Rules)
and [XSLT 3.0 overriding template rules](https://www.w3.org/TR/xslt-30/#overriding).

The current-rule checkpoint passes the complete repository gates: 1,652 core
tests (60 ignored), 22 native tests (three ignored), four WASM adapter tests and
18 worker tests, plus formatting, strict Clippy, documentation, Markdown links,
conformance inventories and unsafe-surface verification. The official WASM
adapter build also passes.

## Guarded copy-union conservation — 2026-10-07

Unchanged `Lotus/select_select71#1` now compares exactly. The compatibility-gated
`xsl:copy-of` compiler reuses the existing typed mixed-union parser and the
runtime's shared single-effective-source union selector. Document branches
must finish normal sealed resolution, parsing and effective navigation with an
empty selection. Local nodes are normalized in document order and deduplicated
before the existing source copier constructs result-owned payload. Nonempty
foreign branches still report `FXRT1017`; there is no cross-document ranking,
foreign local-ID reinterpretation, additional resource authority or second
semantic executor. Existing path-only copying retains its ordinary route.

The first replay advanced this case from initialization failure to
`FXRS0001`: its literal `document('select71.xml')` reference resolves beside
the stylesheet, not under the harness's principal `source/` identity. The
first-party import overlay now explicitly admits the unchanged archived source
bytes at that second logical identity. The fixture SHA-256 is
`9905ba07451b0dcad80d6176d40fa987e3b47f744f180076159a8ea792d63fc6`.
No expected result or upstream fixture was edited, and equal bytes do not
collapse independently prepared document identity. A missing resource is not
silently treated as an empty selection.

Five focused regressions cover local ordering/deduplication, both whitespace
representations, COW/complete-clone frames, concurrent use, result survival after
source/snapshot/program release, denied/missing resources, nonempty foreign
rejection, exact XPath/result-node budgets, late cancellation, located failures
and zero retained requested allocations after success, rejection or cancellation.
Selection and all foreign guards complete before copying; no partial result
escapes a failed guard. Semantic inspection and compiled-retention accounting
classify the new private instruction alongside existing copy instructions.

The complete OASIS CD04 denominator now reports 2,499 / 3,173 exact comparisons
(78.76%), 2,699 initialized, 2,651 executed, 471 initialization failures and 48
execution failures. XML mismatches remain 14, comparator gaps four and
expected-error credits 423 / 431. Eighty percent requires another 40 exact
cases; this small gain does not resolve the broader owner-value frontier.

ADR-0004 review keeps the 50-line copied-union executor under the existing
three-file source-selection owner. It consumes a typed instruction and delegates
selection and copying; it owns neither resource acquisition nor foreign focus.
The source-copy compiler is 104 lines, while the compiler/runtime composition
roots remain explicit decomposition debt. This routing extension does not
retire that debt. A small private local-variable observation helper preserves
semantic inspection traversal without suppressing the function-size gate.
No public API, wire format, unsafe exception or host-performance claim is added.

Final repository verification passes 1,657 core tests (60 ignored), 22 native
tests (three ignored), four WASM adapter tests and 18 worker tests. Formatting,
strict Clippy, documentation, local links, conformance inventories and the
reviewed unsafe-surface gate pass. The official `wasm32-unknown-unknown` adapter
check passes. The final corpus replay above is separate unchanged-suite evidence,
not an inference from the focused regressions.
