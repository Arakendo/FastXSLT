# AR-0030: Invocation Owned Global Constructor Evaluation

| Field | Value |
| --- | --- |
| Status | Under Review for implementation follow-up; ownership accepted in ADR-0024 |
| Opened | 2026-10-10 |
| Last reviewed | 2026-10-15 |
| Scope | Global initialization, shared sequence execution and dependency resolution |
| Trigger | OASIS `Lotus/variable_variable40#1` requires template dispatch while constructing a global value |
| Related ADRs | ADR-0004, ADR-0007, ADR-0012, ADR-0014, ADR-0017, ADR-0022, ADR-0023 |
| Related evidence | [Current OASIS frontier](../Campaigns/oasis-xslt10/evidence/accounting-and-oracles/oasis-xslt10-xml-indentation-frontier-2026-10-10.md) |

## Architectural question

How should invocation-owned global values be resolved when their constructors
execute the shared instruction/template machinery and discover dependencies
through dispatched templates, without a second interpreter or source-dependent
compiled state?

## Trigger and evidence

**Current implementation checkpoint (2026-10-15):** unchanged `variable40` now
passes through production shared-constructor execution, not only the reference.
Both full OASIS profiles gain one exact comparison: modern **2,501 / 3,173
(78.82%)**, strict **2,498 / 3,173 (78.73%)**. Broader integration remains open;
the historical trigger below describes the original unsupported boundary.

The unchanged `Lotus/variable_variable40#1` constructs `$TreeFrag` with
`xsl:apply-templates select="//b"`. It originally failed compilation with
`FXST1015`. Its matched templates include recursive application and text
construction; the principal transformation later reads the completed value.
No new resource authority is required by this case.

At opening, initialization ran `materialize_global_defaults` before constructing
the transformation's `SequenceInputs`. Direct dependencies recognized by
`global_dependencies` are topologically ordered, including after module merge.
`SequenceInputs` borrows a completed `RuntimeGlobals`; ordinary execution does
not request evaluation of a not-yet-materialized global. General template
execution in a global constructor is explicitly outside the current SDD.

The shared executor already owns template dispatch, lexical scopes, invocation
control and result construction. Reusing that executor requires defining how it
sees globals during initialization. Merely adding an instruction-vector variant
does not solve transitive dependencies, cycles or global focus.

[XSLT 1.0 section 11.4](https://www.w3.org/TR/xslt.html#top-level-variables)
defines global evaluation relative to the principal root with singleton focus,
requires referenced globals to be computed first and rejects circularity. A
modern candidate must also be checked against the selected modern global context
and circularity rules; the legacy case cannot establish those rules by itself.

The original modern exact numerator remains 2,500 / 3,173 (78.79%); strict
preprocessing experiment remains 2,497 / 3,173 (78.70%). This review earns no pass.

## Ownership and constraints

- Compilation owns the graph's resolved definitions and stylesheet-derived
  executable plans. It must not evaluate constructors against a source.
- One invocation owns global evaluation state, supplied parameters, values,
  dynamic documents, temporary trees, controls and any dependency stack.
- The shared executor owns instruction, expression, template, mode and focus
  semantics. Global initialization must not implement alternatives to them.
- A constructor's entry focus must not inherit the template caller's focus,
  local variables, parameters or current matched rule accidentally.
- Dependency identity must use the resolved global binding, not a local-name
  string that can collide after namespace/import processing.
- Effective source relationships, qualified node identity and retained owners
  remain governed by accepted whitespace/node ownership decisions.
- Global values stay invocation-local. No cross-invocation memoization, shared
  mutable prepared state, global registry, public resolver or new authority is
  selected.
- Failure and cancellation must not publish partial values. Parameter overrides,
  related dependency locations and repeated reads need explicit tests.
- Follow ADR-0004 before extending the pressured compiler/runtime composition
  roots. Extraction must be checkpointed separately from semantic changes.

## Alternatives

### Retain bounded specializations

Keep existing direct-dependency plans and reject general constructors. This
preserves the current boundary but leaves legitimate shared-language execution
unsupported. More fixture-shaped specializations are not a general solution.

### Static transitive dependency plans with shared execution

Compile constructors into shared instructions and derive enough dependencies
through named/matched template reachability to initialize them eagerly.
This may retain the current immutable completed-globals interface for a bounded
profile. Dynamic dispatch and conditional reads require conservative admission.
Treating every global read in every template as an unconditional dependency can
invent cycles through templates the constructor never invokes.

### Invocation demand resolution with shared execution

Give resolved bindings invocation-local states conceptually equivalent to
unevaluated, evaluating and complete. A variable read requests the value;
constructor evaluation uses the ordinary executor with deliberate global context.
An active dependency stack detects re-entry and supplies diagnostic provenance.

This accommodates dependencies discovered by dispatch but changes the completed-
globals borrowing model. A safe prototype must not keep a mutable cell/map borrow
across recursive execution. It must define failed-state cleanup, budget charging,
parameter precedence and single-evaluation behavior before promotion. These state
names are experimental, not selected Rust types or a public contract.

### Hybrid resolution

Retain direct static ordering as a private optimization while using one demand
resolver for executable constructors. It must preserve one variable-read meaning
and one cycle/failure model; two independently implemented global evaluators would
defeat the purpose.

## Findings and uncertainties

The blocker is global evaluation ownership, not XML parsing, resource acquisition
or stylesheet preprocessing edition. Selecting strict preprocessing does not
choose eager versus demand evaluation.

ADR-0024 selects one invocation resolver and initial eager scheduling. The safe
demand reference demonstrates recursive
execution with existing lexical frames without cloning the entire invocation or
holding recursive borrows. Static admission's useful coverage and production
declaration integration remain unresolved; bounded original-declaration linking
now works in the reference. Evaluation strategy also affects unused failures and
message observations; compare permitted behavior by edition rather than assume
eager and demand execution are interchangeable in every observable circumstance.

## Disposition

Ownership is accepted in
[ADR-0024](../ADR/ADR-0024-invocation-owned-global-resolution-with-shared-execution.md).
This review remains open for implementation follow-up. The first production
slice is authorized, but retain unsupported guards until production context,
dependency, diagnostic, control and performance gates pass. Acceptance earns no
corpus credit and does not select the reference's physical representation.

### Decision-readiness checkpoint (2026-10-14)

The reference now has 125 focused tests plus a manually executed unchanged
`variable40` replay. This is enough to establish safe
recursive ownership and useful legacy shared-executor feasibility, not enough
to claim complete production integration. Further migration of individual
read-site locations is not the sole remaining architectural gate.

| Question | Evidence now | Remaining gate |
| --- | --- | --- |
| Can recursive reads avoid a live mutable cell borrow? | Demand reference, unwind, reuse and isolation tests; ADR-0024 accepts the owner | Implement the private production interface and preserve those invariants |
| Can actual global bodies reuse sequence lowering? | Original declarations, one supplied include or import, shared validation/linking and unchanged `variable40` replay now work privately | Transitive/mixed graphs, additional value kinds and production admission |
| Does the body preserve compilation-profile meaning? | Strict/modern text difference through the same compiler and executor | Whole graph integration; no profile selection from local version |
| Are schedules interchangeable? | Eager/demand probes expose unused failures, messages and work differences; bounded static admission now handles straight-line named calls but refuses dynamic dispatch | Select edition-specific initialization policy; broader static analysis is not disproved or selected |
| Are controls and diagnostics conserved? | Exact/one-less limits, cancellation, failed-cell reuse and original instruction locations | Remaining typed reads, standard cycle identity and bounded production attachment policy |
| Is the implementation economical? | Same-owner constructor scheduling probe and same-program ordinary-global production control | Production integration cost and host-visible regression measurement; do not ship test-hook plumbing as the representation |

Maintainer acceptance with modifications is recorded in ADR-0024. Production
integration may now proceed within its scope. Further approval is still needed
for materially different ownership, lazy-error policy or broader public/context
contracts. Next checkpoint decomposition, then integrate the first shared-owner
constructor slice and replay unchanged `variable40` through production.

## First demand-reference checkpoint

The private test-only `runtime_context/global_demand_reference.rs` owns binding
states and an active evaluation stack. It has no expression or instruction
interpreter. Resolved binding ordinals identify cells; short interior borrows
are released before invoking an evaluation callback recursively.

Seven focused tests pass. They exercise forward recursive reads, completed-value
reuse, supplied seed precedence, unused failure avoidance, direct/indirect
re-entry detection, cached failure identity, depth and exact/one-less state-work
limits, cancellation before publication and active-marker cleanup. A callback
also invokes the existing sequence executor with fresh lexical variables, root
singleton focus and matched-template dispatch, then uses the existing charged
result-to-temporary-tree conversion. Its owned value is evaluated once and reused.

This is a control/borrowing proof, **not production global-read integration**.
The shared-executor probe evaluates an ordinary compiled template body as the
reference constructor; it does not admit general global constructors or discover
their dependencies through production variable reads. Seed precedence is not yet
an end-to-end supplied-parameter test. No standards circularity diagnostic or
dependency-location contract has been selected by the ordinal cycle trace.

Failure memoization is a candidate reference behavior. Panic-unwind marker reset
is test evidence, not a native recovery guarantee. State cells, active stack
entries and cycle-trace entries are charged before retention using provisional
work units; they are not allocator-byte or process-memory accounting. Result
retention remains charged by the shared converter.

ADR-0004 cohesion is preserved by one private test-only child owner; the pressured
runtime-context parent gains only its module declaration. No mutable executor
context, public abstraction, unsafe code or production evaluation branch is added.
The next tranche must connect real variable reads to this owner, pin constructor
context and diagnostic provenance, and compare static versus demand behavior.
The broad follow-up gates below remain unchecked until that integration earns them.

Checkpoint validation: full `scripts/verify.ps1` passed, including formatting,
strict Clippy, 1,743 core tests (60 ignored), adapter/worker tests, documentation,
local links and conserved conformance inventory. The official
`wasm32-unknown-unknown` adapter check passed. Fresh OASIS replays conserve modern
2,500 exact / 14 mismatches and strict 2,497 exact / 17 mismatches, each over 3,173
cases with 423 expected-error credits. No archived oracle or production global
behavior changed.

### Shared-read reference checkpoint

The reference now connects ordinary compiled XSLT 1.0 string-value reads to the
demand owner through a **test-only** `RuntimeGlobals` hook at the existing
compatibility conversion boundary. Completed values and lexical shadowing are
checked first. Every constructor starts a fresh lexical frame with deliberate
principal-root singleton focus and no current matched rule. No cell, stack or
evaluation-counter borrow crosses recursive executor entry.

Fourteen additional tests pass (21 focused AR-0030 tests in total). Thirteen
exercise shared execution; one conserves an independent modern compile boundary.
The executable proofs cover:

- forward reads discovered through matched templates and modes, built-in and
  named-template dispatch, single evaluation and repeated reads;
- conditional reads, an untaken potential cycle, a taken dispatched cycle and
  repeated failure without partial publication;
- caller-local isolation, cross-kind local shadowing and missing-current-rule
  diagnostics with original stylesheet provenance;
- supplied string-parameter precedence without overriding an ordinary variable;
- resolved namespace-qualified binding names, equal local names in different
  namespaces and aliases of one expanded name;
- already-linked include/import definitions and import-precedence selection;
- exact/one-less result-retention limits and cancellation inside construction;
- independent invocations, concurrent calls over one immutable compiled program
  and source, complete-clone/COW controls, owner release and values outliving
  their invocation and compiled owner.

Named-template bodies still stand in for global constructor plans, with ordinary
placeholder global declarations used only for static name resolution. The
placeholder defaults are not installed. **Production global constructor admission
and variable reads remain unchanged**, and `variable40` earns no credit. This
bridge covers string conversion, not every variable value kind or read operation.
The seed adapter is string-only and does not settle required-parameter/type rules.
The driver uses fresh source-confined execution caches; general dynamic-document,
key and message behavior across recursive global entry remains to be integrated.

The modern probe preserves `FXXP1001` for `string($first)` in a 3.0 stylesheet.
Do not infer modern parity from the legacy read proof. Independently, the
[modern global-context rules](https://www.w3.org/TR/xslt-30/#global-context-item)
distinguish global context from initial match selection; the principal-root
reference is not a general modern global-context implementation. Cycle failures
in this driver use explicitly experimental `FXAR0030`, while existing executor
and control failures retain their exact codes and locations. Standards cycle
diagnostics and related dependency locations remain pending.

ADR-0004 review: `runtime_context.rs` is 2,565 lines, with four direct child files;
it gains only the guarded reference field beside the existing module hook.
`xslt10_compatibility.rs` remains a cohesive conversion owner (1,048 lines,
six direct siblings); its seven-line guarded fallback adds no production branch.
The demand-state owner is 416 lines and its shared-execution test driver is 723
lines in one named child directory. Retain these private reference seams through
the next checkpoint; review extraction of production global initialization before
any production integration. The hook, weak driver link and fixture adapter do not
select a public resolver, frame representation or general mutable executor owner.

Next compare admission strategies and complete value-kind/context/diagnostic
integration before recommending an ADR. The broad checklist remains deliberately
open; the proofs above are bounded reference evidence, not feature completion.

The borrowing seam identified at this checkpoint is concrete: existing `temporary_tree` and source-value
lookups expose references into completed global maps. A string-returning bridge
does not solve their recursive lifetime contract. Compare an owned global-value
read shape against those consumers before admitting general constructors; never
hold a mutable global-map borrow while invoking the executor or materialize a
second completed map merely to hide recursive ownership. The following checkpoint
tests that shape without selecting a value enum, `Arc` layout or public accessor
contract.

Validation for this checkpoint passed the full repository gate with 1,757 core
tests (60 ignored), 22 native tests (3 ignored), four WASM adapter tests and 18
worker tests, plus formatting, strict Clippy, docs, links, unsafe-surface and
corpus inventory checks. The official WASM target check also passed. Fresh modern
and strict OASIS replays retain respectively 2,500 / 14 and 2,497 / 17 exact /
mismatch counts over 3,173 cases; both retain 423 expected-error credits.

### Owned-value reference checkpoint

The demand driver now returns immutable owned handles containing either the
existing temporary tree or an existing retained source selection. Constructors
continue through shared sequence execution; source-path global defaults use the
existing controlled path evaluator. The string-read bridge consumes these values
through their effective semantic owners. The experiment does not introduce a
second completed-global map or keep a resolver borrow alive while executing a
dependent constructor.

Six new tests bring the focused reference set to 27. An outstanding borrowed tree
or source selection remains usable while another binding is evaluated, including
after dependent evaluation fails. Repeated reads retain the same completed value;
temporary identity and source origin/local-ID identity remain stable. Source
values retain their effective owner across a foreign focus with colliding IDs,
and existing lexical frames preserve string conversion through both COW and
complete-clone controls. The frame adapter explicitly copies and charges its ID
selection; zero-copy frame admission is not established.

The visibility-view and complete-derived controls produce equal stripped
relationships and preserve visible identity/provenance, while the original
prepared source keeps its whitespace. Exact/one-less source traversal limits and
mid-selection cancellation retain no partial source value or extra owner.
Releasing the resolver leaves outstanding values valid; releasing the final
value releases its retained source owner.

The copy probe found a concrete remaining consumer boundary: `copy-of` over a
retained selection in a foreign effective source still reports `FXRT1017`.
That guard is pinned. Copying in the supported owner context produces independent
result payload, including copied namespace/attribute data, which survives source
owner release. This reference does not claim foreign-owner copying or integrate
every typed global read into the executor.

The owned-value enum and `Arc` shape are experimental. Atomic/sequence/document
variants, all typed-read adapters, modern global context, standard cycle
diagnostics and complete driver accounting remain open. Production constructor
admission is unchanged, and this tranche earns no corpus pass. The shared driver
remains under 800 lines; the value/lifetime experiment occupies a cohesive private
child module under 500 lines. Existing production frame and copy boundaries gain
no new branches.

Validation passed the full repository gate with 1,763 core tests (60 ignored),
22 native tests (3 ignored), four WASM adapter tests and 18 worker tests, plus
formatting, strict Clippy, docs, Markdown links, unsafe-surface and corpus
inventory checks. The official WASM target check also passed. No full OASIS replay
was performed for this test-only tranche; the preceding checkpoint's measured
counts remain the latest corpus evidence, not a new measurement.

### Evaluation-schedule comparison checkpoint

Five additional tests bring the focused reference set to 32. A private schedule
probe compares declaration-order eager initialization with on-read evaluation.
Both use the same demand cells and the same compiled sequence/template executor;
recursive reads never acquire a second evaluation meaning. This is an executable
schedule comparison, **not a static template-reachability implementation**.

- Forward reads through matched-template dispatch produce the same result and
  evaluate each needed binding once under both schedules. Only eager scheduling
  evaluates the unused binding.
- An unused terminating constructor is never invoked by the demand schedule;
  eager initialization reports its existing `XTMM9000` failure. This demonstrates
  an observation difference, not a ruling on which unused failures each standards
  edition permits processors to report.
- A self-read in an unselected mode does not create an executed dependency under
  either schedule. Treating every matched-template read as an unconditional edge
  would invent a cycle; this does not falsify a mode-aware conservative analysis.
- A supplied parameter bypasses its cyclic default under both schedules. Without
  the supplied value, both encounter the actual cycle and unwind all active
  markers. Cycle identity remains the experimental `FXAR0030`, not an accepted
  standards diagnostic.
- A measured XDM-work budget sufficient for the demand schedule is insufficient
  for eager initialization of unused constructors and yields `FXCT0002`. Schedule
  choice therefore cannot be described as work-accounting neutral. This is not
  an allocator-memory or execution-speed measurement.

Typed consumers were inspected but not changed: existing temporary-tree and
source-node accessors still borrow completed value-kind maps. The owned-value
reference solves the recursive lifetime prerequisite, not automatic admission
into every such consumer. In particular, do not eagerly populate a second global
map merely to let those borrowed accessors keep their old contract; that would
hide the schedule/failure differences established above.

The remaining comparison must distinguish static **admission** from eager
**evaluation**. Static analysis may prove a bounded constructor eligible without
forcing untaken dependencies to execute. A future hybrid must preserve one
authoritative read/cycle meaning and specify any eager optimization's observable
failure, message and budget effects. No alternative is selected by these tests.

The schedule module is a private test child of the existing reference driver;
it adds no compiler scanner, runtime interpreter, production hook or public
surface. Production constructor admission and measured corpus totals remain
unchanged.

Validation passed all 32 focused reference tests and `scripts/verify.ps1`,
including 1,768 core tests (60 ignored), 22 native tests (3 ignored), four WASM
adapter tests, 18 worker tests, formatting, strict Clippy, documentation, links,
unsafe-surface and pinned corpus checks. The official WASM target check also
passed. No OASIS replay or performance claim is attached to this test-only change.

### Typed temporary-copy integration checkpoint

Six further tests bring the focused reference set to 38. A test-only owned
temporary-tree read bridge now connects the compiled `copy-of` variable consumer
to the same demand cells used by legacy string reads. The bridge returns an
immutable retained tree; the consumer delegates to the existing charged temporary
copier. It introduces neither a second global map nor another copying evaluator.
The reference's extra `Arc<TemporaryTree>` is an experimental ownership shape,
not a selected production representation or a performance result.

Recursive constructor copies evaluate dependencies once and preserve actual
element, namespace and attribute structure, not merely string output. Copied
results remain usable after the resolver and its completed trees are released.
Local temporary bindings still shadow globals without evaluating the shadowed
constructor. Mixed copy/string dependency cycles share the existing reference
cycle/failure state. Exact/one-less XDM-work limits and active cancellation leave
no evaluating markers; COW and complete-clone frame controls have equal measured
work-domain totals for the compiled copy fixture.

A separate consumer-failure probe first completes the globals, then exhausts
result-node budget or observes cancellation during copying. The consumer returns
the exact existing `FXCT0002` or `FXCT0001` failure while the completed global
retains its identity and value. Inspecting that state with a fresh test control
does not promise resumability of a cancelled production invocation. Constructor
failure memoization and downstream consumer failure must remain distinct.

This bridge admits temporary-tree reads only. Source values, atomics, sequences,
literal documents and other typed consumers still require their own integration;
source copying is not silently admitted through the temporary bridge. Existing
foreign-source frame copying retains its `FXRT1017` guard. Named template bodies
still stand in for constructor plans, and production global admission is unchanged.

ADR-0004 checkpoint review: `golden_runtime_experiment.rs` remains an exceptional
6,619-line composition unit; this tranche adds only a seven-line `cfg(test)`
fallback to its existing copy consumer. `runtime_context.rs` is 2,567 lines and
adds only the guarded hook field. The callback wrapper (35 lines) and typed-copy
proofs (251 lines) reside under the private demand-reference subject; its shared
driver is 774 lines. Retain these bounded experimental seams for attribution now,
and extract global initialization/copy-consumer ownership separately before
production integration. No production branch, public API, authority, unsafe
surface or corpus classification changes in this checkpoint.

Validation passed all 38 focused reference tests and the full repository gate:
1,774 core tests (60 ignored), 22 native tests (3 ignored), four WASM adapter
tests, 18 worker tests, formatting, strict Clippy, docs, local links, unsafe-surface
and pinned corpus checks. The official WASM target check passed separately.
The test-only bridge earns no corpus credit; no full OASIS replay was performed.

### Failure-observation checkpoint

Four further tests bring the reference set to 42. A terminating message reached
through nested typed copies preserves its exact compiled instruction byte span
and stylesheet resource, `XTMM9000`, category, request identity and absent work
domain. The outer global read does not replace the inner failure location.
Repeated reads of either failed binding return the same structured failure
without replaying messages or constructor execution.

Successful mixed copy/string reads likewise record constructor messages once,
in the first-demand execution order. This is evidence of the candidate demand
schedule, not a claim that eager and demand initialization have identical
observable behavior. A failed constructor's emitted messages remain recorded;
"no partial value publication" does not mean rollback of observations already
made through the shared executor.

Result-work exhaustion and active cancellation inside the dependency preserve
their existing code, category, request identity and `ResultNode` work domain
through both enclosing failed cells. Neither publishes a completed value or
leaves an evaluating marker. Their current missing expression location remains
missing; the reference does not fabricate a caller location. The mixed-read
cycle probe pins the actual ordinal dependency order while retaining the
experimental code and absent source location.

Inspection identifies a concrete provenance gap before production integration:
`GlobalBinding` retains resolved name/kind/default/precedence but no declaration
location, and `ExecutionFailure` has one primary location but no structured
related-location chain. Constructor instruction provenance survives today;
declaration/read-edge cycle provenance does not follow from it. The named-template
stand-ins cannot honestly supply missing global-declaration spans. A future
diagnostic design must preserve those spans from compilation and define bounded
dependency context without replacing an existing originating failure or forcing
hosts to parse display strings. This checkpoint does not change either type or
select a public diagnostic contract.

All additions remain in one private failure-observation test module and the
existing reference driver's module declaration. Production semantics, authority,
corpus accounting and the prior decomposition checkpoint are unchanged.

Validation passed all 42 focused reference tests and the full repository gate:
1,778 core tests (60 ignored), 22 native tests (3 ignored), four WASM adapter
tests and 18 worker tests, plus formatting, strict Clippy, documentation, links,
unsafe-surface and pinned corpus checks. The official WASM target check passed.
No full OASIS replay was performed; no new corpus or performance claim is made.

### Bounded secondary-context reference

Five additional tests bring the focused reference set to 47. A private diagnostic
sidecar experiment retains the original failure by immutable handle and records
an ordered, capped sequence of binding identities with separate declaration and
read-site locations. Re-entering one binding can therefore reuse its declaration
metadata while retaining distinct read sites. This experiment receives explicit
fixture metadata; it does **not** extract missing provenance from compiled globals
or claim module-graph integration.

Secondary context is not allowed to replace the established originating failure.
An entry ceiling, exhausted trace-work budget or cancellation stops capture and
marks the context incomplete, retaining only the admitted prefix. The tests pin
zero/exact/overprovisioned entry caps, exact/one-less work, active cancellation,
cross-resource location ordering, original failure identity and metadata release.
An empty dependency path allocates no entries and performs no secondary work.

Only handles to immutable supplied locations are copied. One provisional XDM-work
unit is charged before each retained entry; this is neither byte accounting nor a
new production diagnostic resource domain. The caller-owned input path and static
metadata require their own bounded ownership in a real integration. The trace's
entry ceiling does not bound those inputs or total process memory.

This nominates a failure-preserving optional-context shape, not an accepted API
or cancellation-precedence policy. The production compiler and `ExecutionFailure`
are unchanged. Next prove where compilation retains declaration/read-edge metadata
and how the authoritative resolver attaches bounded context once, without growing
it again on every cached read. Standard cycle codes and public related-location
representation remain undecided.

Validation passed all 47 focused reference tests and the full repository gate:
1,783 core tests (60 ignored), 22 native tests (3 ignored), four WASM adapter tests
and 18 worker tests, plus formatting, strict Clippy, docs, links, unsafe-surface
and pinned corpus checks. The official WASM target check also passed. No new
OASIS replay, performance result or production-support claim is attached.

### Compiled provenance and capture-once checkpoint

Four new tests bring the focused reference set to 51. The compiler now retains a
**test-only** declaration location alongside each global binding. Actual compiled
metadata, rather than manually assigned fixture spans, feeds the bounded-context
reference. Qualified names with equal local parts retain distinct declaration
spans; multibyte text before the declarations pins byte offsets. Import precedence
keeps the winning declaration's resource, inclusion preserves the included
resource, and dependency ordering does not detach provenance from its binding.
Locations remain valid after the parsed stylesheet owner is released.

The existing compiled `CopyOfVariable` location supplies a distinct read site.
The resulting reference context retains both declaration and read locations
without assigning either as a fabricated primary failure location. These tests
do not yet propagate read-edge locations through recursive read callbacks; the
production global representation still has no added declaration field.

A separate per-failure capture-once slot retains the first bounded context by
immutable handle. Later requests, even with a different path, larger cap or
exhausted secondary budget, return the same context without new trace work or
growth. An incomplete first trace stays explicitly incomplete. Context handles
can outlive the slot and release the primary failure when the final handle is
dropped. The slot is a candidate to attach to the authoritative failed value,
not a new variable-name map, global registry or retry mechanism. It is not yet
wired into resolver failed cells.

ADR-0004: the 3,082-line compiler composition unit adds only guarded provenance
capture at its existing global-binding construction seam; the 2,277-line semantic
plan owner adds only the guarded metadata field. The dedicated compiled-provenance
tests remain under the private demand-reference subject. Preserve this checkpoint
and perform production global-initialization extraction separately; test metadata
does not authorize an unrelated compiler refactor or change production retention.

Validation passed all 51 focused reference tests and the full repository gate:
1,787 core tests (60 ignored), 22 native tests (3 ignored), four WASM adapter tests
and 18 worker tests, plus formatting, strict Clippy, docs, links, unsafe-surface
and pinned corpus checks. The official WASM target check passed. No new OASIS
replay, production-support credit or performance claim is attached.

### Failed-cell context attachment checkpoint

The demand reference's failed cell now owns a capture-once context slot alongside
its original immutable failure handle. No second value map or variable-name trace
registry is introduced. Ordinary reads return the same memoized primary failure;
bounded context capture performs no executor callback while the cell is borrowed.

Four tests cover recursive failure propagation, repeated context/read reuse,
unresolved/successful/unknown binding rejection, secondary exhaustion, metadata
lifetime and distinct failures with identical text. A failed cell's first partial
trace is retained rather than expanded under a later larger budget. Context can
outlive the resolver and releases the primary failure with its last handle.
Failures are not merged by display text or code. Two failed cells propagating one
primary may still have distinct cell-local contexts; no cross-cell trace interning
or global single-capture policy is selected.

This completes failed-cell attachment in the **generic test reference**, not the
shared executor's automatic provenance pipeline. The tests still pass explicit
dependency paths. Actual recursive typed/string read callbacks must supply the
correct read-site path at the failure point before unwinding removes it. Successful
reads must not retain diagnostic traces, and caller-local provenance must not be
substituted for a constructor dependency. Production errors and resolver policy
remain unchanged.

Validation passed all 55 focused reference tests and the full repository gate:
1,791 core tests (60 ignored), 22 native tests (3 ignored), four WASM adapter tests
and 18 worker tests, formatting, strict Clippy, docs, links, unsafe-surface and
pinned corpus checks. The official WASM target check passed. No OASIS replay or
new production-support credit is claimed for this test-only tranche.

### Shared-executor read-site propagation (2026-10-12)

The opt-in test reference now passes the existing compiled `CopyOfVariable`
location into the temporary-value demand callback. A private guarded path owner
records the binding declaration and actual read location, with no mutable path
borrow across recursive execution. String-value consumers do not yet supply a
location and explicitly retain `None`; no caller location is invented.

The reference caps the active path at 32 entries and charges provisional entry
work before retaining metadata. Guards release entries on success, failure and
unwind. Completed/failed-cell rereads bypass path construction. When evaluation
fails, each failed cell captures its own active path before unwinding through the
existing bounded capture-once slot. The originating failure and constructor
messages are preserved, and repeated reads neither reevaluate nor relabel context.

Three additional tests prove real recursive copy locations, unknown string sites,
entry/depth bounds and unwind cleanup. The focused reference now has 58 passing
tests. These are **active read paths**, not complete causal chains: an inner cell
can retain a longer path than its propagating parent, and cycle re-entry can unwind
before the surrounding cell becomes failed. Complete cycle provenance, string
read-site threading and production diagnostic/accounting policy remain open.
The opt-in path uses reference work units, not exact retained-byte accounting.
No production constructor admission, public type or corpus credit changes.

Decomposition review: the reference folder now contains ten files. They remain
one private test-only experiment, with resolver state, read bridges, provenance
and shared-executor probes separately owned. No new subfolder is justified by
this count alone; revisit grouping if additional consumer families expand it.

Validation: all 58 focused reference tests and the full repository gates passed,
including 1,794 core tests (60 ignored), 22 native tests (3 ignored), four WASM
adapter tests and 18 worker tests, formatting, strict Clippy, documentation,
Markdown links, unsafe-surface and pinned conformance checks. The official
`wasm32-unknown-unknown` adapter check also passed. OASIS was not replayed for
this test-only change; the last measured coverage remains unchanged.

### Cycle-detection observation before unwind (2026-10-12)

The opt-in reference now retains at most one bounded first-cycle observation.
Capture occurs on the original ordinal `Circularity` failure while the closing
read guard remains active, before translation through the shared executor and
before the target evaluating cell can become failed. This proves that the full
active cycle path is available without holding a mutable cell borrow through
recursive execution. It does not change the cell-local attachment model.

Four tests cover indirect typed cycles with distinct closing read locations,
mixed typed/string paths with unknown string locations, direct cycles versus
successful cached reads, and secondary capture exhaustion. The original cycle
failure retains its identity when capture is incomplete. Re-observation cannot
grow or replace the first record. Retained metadata outlives the driver and is
released with its final observation handle. The reference now contains 62 tests.

This is deliberately an **inspection experiment**, not a second failure map or
a complete invocation error history. It selects neither production observation
delivery nor how complete cycle context should be attached to propagated errors.
Capture still uses provisional work units and a 32-entry cap. String read-site
threading and the eventual standards diagnostic remain unresolved. The folder
now contains eleven cohesive reference files; the additional file groups cycle
observation tests under the existing provenance owner, without a new runtime layer.
Production admission and the last measured OASIS numerator remain unchanged.

Full verification passed: 1,798 core tests (60 ignored), 22 native tests (3
ignored), four WASM adapter tests and 18 worker tests, strict Clippy, formatting,
documentation, links, unsafe-surface and pinned conformance checks. The official
WASM target check also passed. String-location inspection confirms the conversion
helper serves multiple expression consumers without a location parameter; the
next experiment must preserve the actual compiled expression origin rather than
infer one from ambient caller focus or the constructor entry instruction.

### Explicit direct-string instruction provenance (2026-10-12)

The existing `ValueOf` instruction location now travels explicitly through a
test-only evaluator argument into the direct `Xslt10VariableString` conversion
and demand callback. It identifies the compiled reading instruction, **not an
exact variable-token span within the expression**. There is no ambient current
instruction slot, caller-focus lookup or inherited constructor-entry location.
The ordinary conversion implementation remains shared; callers without proven
provenance explicitly pass no location.

The 63-test reference pins a nested direct string read independently from the
outer copy site and originating terminating-message location, with both ordinary
and disable-output-escaping execution. Existing unknown-site tests now exercise
composed `string-length($variable)` reads, which deliberately remain unwired.
Temporary-text and namespace-scalar evaluator callers likewise supply no inferred
location. Production evaluator signatures and constructor admission are unchanged;
the conversion forwarding refactor adds no alternate value semantics.

This closes direct string-read instruction provenance only. Composed expressions,
boolean reads and other consumers still need deliberate location propagation;
exact subexpression spans and a production related-location contract are not
selected. No OASIS replay, new pass or performance claim is made.

Validation passed all 63 reference tests and full repository gates: 1,799 core
tests (60 ignored), 22 native tests (3 ignored), four WASM adapter tests and 18
worker tests, formatting, strict Clippy, docs, links, unsafe-surface and pinned
conformance checks. The official WASM target check and `git diff --check` passed.

### Selected unary conversion provenance (2026-10-12)

The explicit test-only instruction-location path now covers variable string
length, the already-admitted scaled `string-length(string($variable))` form,
`normalize-space($variable)` and legacy `number($variable)`. Conversion and
character/work charging remain in the shared implementation. Existing callers
without a location continue through the no-location forwarding entry point.

Three matrix tests bring the reference to 66 tests: nested constructor failure
preserves the reading instruction separately from the primary message origin;
success and cached rereads conserve values and XPath-operation counts with
provenance enabled/disabled; local shadowing never evaluates a failing global.
The reference's extra metadata work remains independently charged as XDM entry
work and is not claimed to be free or allocator-exact. Unknown-location tests
now use multi-part `concat(...)`, including a mixed cycle, rather than pretending
all string consumers have become supported. A first scaled-length fixture used
a compiler-unsupported expression shape; it was corrected to the existing
admitted form, without widening compilation for this experiment.

The folder now has twelve files. Unary-conversion evidence is grouped beneath
the existing provenance owner; no new semantic layer or second converter was
introduced. Remaining provenance pressure includes multi-part expressions and
boolean consumers. Production constructor admission, public diagnostics, and
the last measured OASIS numerator are unchanged.

Validation passed all 66 focused reference tests and full repository gates:
1,802 core tests (60 ignored), 22 native tests (3 ignored), four WASM adapter
tests and 18 worker tests, formatting, strict Clippy, docs, links, unsafe-surface
and pinned conformance checks. The official WASM target and diff checks passed.

### Direct concatenation operand provenance (2026-10-12)

The shared concatenation evaluator now accepts explicit test-only instruction
provenance for direct variable operands. Other callers retain the no-location
entry point. No ambient location stack or separate concatenation evaluator is
introduced; nested composed operands are not falsely marked as migrated.

Two additional tests bring the reference to 68 passing tests. They exercise
failure in the first versus second global operand, conserve the original message
failure location, pin which constructors actually ran, preserve a completed
earlier global after a later failure, and verify repeated operands/rereads do not
replay messages. Successful evaluation with provenance enabled/disabled preserves
the same result and XPath-operation count. This pins the existing reference
executor's behavior, not a new public expression-evaluation-order guarantee.

An entire `value-of` instruction remains the location granularity; distinct
operand-token spans are not claimed. Unknown-site and mixed-cycle tests now use
boolean string-length reads, whose provenance remains unwired. Production global
constructor admission, static-versus-demand selection, public diagnostics and
corpus credit remain unchanged.

Validation passed all 68 reference tests and full repository gates: 1,804 core
tests (60 ignored), 22 native tests (3 ignored), four WASM adapter tests and 18
worker tests, strict Clippy, formatting, documentation, links, unsafe-surface
and pinned conformance checks. The official WASM target and diff checks passed.
Follow-up inspection found `ChooseBranch` currently retains its test/body but
not a branch location; boolean provenance must preserve the relevant compiled
condition site rather than substitute the enclosing `choose` or caller location.

### Direct boolean-condition provenance (2026-10-12)

Test-only compiled `ChooseBranch` metadata now retains the actual `when` location.
`if` uses its existing instruction location. Both pass those sites explicitly
to direct variable-string-length boolean evaluation, reusing the same conversion
and charging implementation. Production branch layout remains unchanged, and
callers without an explicit site continue through a no-location entry point.

The reference now passes 70 tests. New checks distinguish the failed `when` from
its enclosing `choose`, preserve the primary constructor-message location, skip
unreached globals, and conserve XPath work with provenance on/off. Unknown-site
and mixed-cycle tests now use nested `not(string-length(...))`, whose recursive
boolean path is deliberately not yet wired. Instruction locations remain broader
than individual expression-token spans; no stronger precision is claimed.

No standards support, public diagnostic shape or global-resolution architecture
is selected by these test-only fields. Production constructor admission and
corpus counts remain unchanged. Nested boolean composition and the remaining
typed-read consumers remain separate follow-up work.

Validation passed all 70 focused reference tests and full repository gates:
1,806 core tests (60 ignored), 22 native tests (3 ignored), four WASM adapter
tests and 18 worker tests, formatting, strict Clippy, docs, links, unsafe-surface
and pinned conformance checks. The official WASM target and diff checks passed.

### Nested boolean provenance and short-circuit conservation (2026-10-12)

Explicit condition locations now pass through the ordinary boolean dispatch and
recursive `not`/`and`/`or` evaluator in test builds. No ambient location slot or
new evaluator is introduced. The selected string-length leaves retain their
own enclosing compiled `if`/`when` site; individual operand-token spans are still
not claimed. Unmigrated variable/literal comparison leaves remain unknown.

Three additional tests bring the reference to 73 passing tests. They cover
evaluated failure operands under both `if` and `when`, runtime short-circuiting
of a failing second global using nonconstant first-global values, and a recursive
cycle whose closing read is inside a nested `when`. Enabled/disabled provenance
conserves XPath work and results in the short-circuit matrix. Skipped constructors
produce neither messages nor cycle observations; path and evaluation stacks drain
after failure. Unknown-site tests now use the unmigrated comparison reader.

The reference directory has thirteen files; composed boolean evidence is grouped
under the existing provenance module, separate from unary/concatenation tests.
This is still one private experiment, not a new runtime responsibility layer.
Production support, public diagnostics, resolver-policy selection and OASIS
credit remain unchanged.

The added plumbing triggered strict Clippy size checks in boolean dispatch.
Rather than add allowances, logical `not`/`and`/`or` routing was consolidated
under one private helper and string-length truth conversion moved to the shared
value owner. Nonlogical dispatch remains separate. This internal refactor adds
no second evaluator or public contract; the full suite verifies the shared paths.

Validation passed all 73 reference tests and full repository gates: 1,809 core
tests (60 ignored), 22 native tests (3 ignored), four WASM adapter tests and 18
worker tests, formatting, strict Clippy, docs, links, unsafe-surface and pinned
conformance checks. The official WASM target and diff checks passed.

## Source comparison bridge boundary (2026-10-13)

A differential probe found a real limitation in the test-only scalar demand
bridge: a source selection containing `A` and `B` made `$source = 'B'` false
after first-node string conversion, whereas the existing retained-node binding
through the same executor returned true. Scalar conversion is legitimate for
`string($source)`, but is not a substitute for general sequence comparison.

The scalar comparison fallback now identifies its purpose to the private demand
hook. A demanded source sequence is explicitly unsupported (`FXAR0030`) there,
rather than silently reduced to a string. Temporary-value comparisons and ordinary
completed bindings retain their existing behavior. The completed source value is
still reusable after this consumer rejection; no constructor is rerun.

One matrix test covers string equality against a later node, inequality against
the first node, and an empty sequence versus an empty string. It compares each
guarded demand read with the existing retained-source binding. This raises the
reference count to 74; it does not establish typed demand comparison support.

An additional exploratory `$source = 2` probe returned false for nodes `1`, `2`.
Inspection shows that this expression takes the separate scalar numeric-literal
boolean route, not the guarded general-comparison fallback. Numeric comparison
admission/typing therefore remains an explicit follow-up, not covered by this
matrix or silently counted as parity. Before production integration, typed reads
must preserve value kind and sequence cardinality across every admitted consumer.
No production support, architectural policy or corpus credit changes here.

Validation: full repository gates passed with 1,810 core tests (60 ignored),
22 native tests (3 ignored), four WASM adapter tests and 18 worker tests.
Formatting, strict Clippy, documentation, 1,988 Markdown link checks, unsafe
surface and pinned conformance checks passed. The official WASM target check
and `git diff --check` also passed.

## Typed source comparison handoff (2026-10-13)

The next test-only candidate hands an owned document plus the complete selection
to a fresh lexical frame, then re-enters the existing string comparison consumer.
It does not implement comparison semantics in the resolver. Eligibility is
deliberately limited to the reference's location-path global definitions;
temporary constructors continue through their existing reader without an extra
demand lookup. Local bindings suppress fallback, including an empty selection.

The previous three-case rejection matrix now passes with exact parity against
completed retained-node bindings: later-node equality, existential inequality
and empty selection versus empty string. The scalar guard remains for unmigrated
consumers. The separate numeric-literal route is not changed or credited.

Two additional tests exercise COW/complete-clone local shadowing, equal-byte
foreign-source rejection (`FXRT1017`), recovery against the correct source,
single evaluation, and budget/cancellation failure with a previously completed
value. Selection-buffer copying is charged before retention. Consumer failure
does not invalidate or recompute the completed resolver cell, leak an owner, or
leave an active evaluation marker. This reference charge is not an allocator-byte
claim or a selected production formula.

The reference has 76 tests. Its directory now contains fourteen files; the new
small source handoff is grouped with the existing temporary handoff, while
comparison semantics stay in the existing value evaluator. No new runtime layer
or public type is introduced. A broader typed-read contract, numeric admission,
read-site provenance and production integration remain open.

Full repository validation passed: 1,812 core tests (60 ignored), 22 native
tests (3 ignored), four WASM adapter tests, 18 worker tests, formatting, strict
Clippy, documentation, Markdown links, unsafe-surface and pinned conformance
checks. The official WASM target and diff checks passed. Production behavior
and the last measured OASIS numerators remain unchanged; no replay or performance
claim is made for this test-only handoff.

## Numeric equality routing correction (2026-10-13)

Closer inspection corrected the earlier route attribution: `$source = 2` uses
`VariableEqualsInteger`, not the separate relational numeric-literal plan.
Its legacy branch converted the whole variable to one number even for completed
source-node bindings. This was a shared-runtime defect, not solely a demand-hook
limitation. [XPath 1.0 section 3.4](https://www.w3.org/TR/1999/REC-xpath-19991116/#booleans)
requires node-set/number comparison to test the numeric string-value of each
node existentially.

Legacy integer equality now delegates to the existing node-set numeric equality
consumer. The expected number is converted to XPath's floating representation
without narrowing an i64 literal to i32. The modern equality branch is unchanged.
The same test-only retained-source handoff used for string comparison now feeds
numeric comparison; the resolver still owns no comparison semantics.

The differential matrix adds a later-node match, an empty selection, a NaN-like
first value followed by a match, and a literal beyond i32 range. A first-party
[golden fixture](../../corpus/golden/source-numeric-equality/README.md) runs through
ordinary production global initialization with no demand hooks. The reference
suite now contains 77 tests. General relational comparisons and variable-to-
variable numeric comparison are separate routes and are not claimed fixed here.
This semantic correction does not select AR-0030's evaluation policy or admit
general global constructors.

Validation passed: 1,813 core tests (60 ignored), 22 native tests (3 ignored),
four WASM adapter tests and 18 worker tests; formatting, strict Clippy, docs,
1,989 Markdown link checks, unsafe-surface and pinned-conformance gates; official
WASM target and diff checks. Fresh unchanged OASIS CD04 replays conserve modern
2,500 exact / 14 mismatches and strict 2,497 exact / 17 mismatches, over 3,173
cases, with 423 expected-error credits in each. No fixture or expectation edits
were made to the upstream suite.

## Relational literal comparison correction (2026-10-13)

A failing probe confirmed that `$source > 2` with nodes `1`, `3` also used
first-node conversion. Relational comparisons against a numeric literal now
share the existing numeric comparison's source-selection traversal and scalar
fallback. Only the scalar predicate differs; the existing floating comparator
still owns operator/NaN behavior. No new expression parser or resolver evaluator
is added. The test-only typed source handoff feeds this same consumer.

The matrix now covers later-node `<`, `>`, `<=`, reversed literal operands,
empty selections and all-NaN content. The production golden exercises greater-
than and reversed inclusive comparison as well as equality. One new control test
pins exact/one-less string-value-node budgets and cancellation after a cached selection
has been prepared: failures release handoff owners, preserve the completed global
and leave no active demand marker. The reference/golden suite has 78 tests.

This closes the literal relational route, not variable-to-variable comparisons,
general typed global admission or AR-0030's evaluation-policy decision. Scalar
conversion functions retain their ordinary first-node semantics. No public API,
resource authority or cross-document ordering decision follows from this fix.

Repository verification passed with 1,814 core tests (60 ignored), 22 native
tests (3 ignored), four WASM adapter tests and 18 worker tests. Formatting,
strict Clippy, docs, Markdown links, unsafe-surface, pinned conformance and the
official WASM target checks passed. Modern OASIS replay retained 2,500 exact
matches, 14 mismatches and 423 expected-error credits over 3,173 cases.
Strict replay retained 2,497 exact matches, 17 mismatches and 423 expected-error
credits. All 78 focused reference/golden tests and the final diff check passed.

## Variable-pair relational comparison (2026-10-13)

A failing demand probe confirmed that `$source > $right` compared only the
first nodes: left `1,4` and right `3` incorrectly returned false. The production
route used the same scalar conversion. The relational variable-pair route now
preserves each node-set's numeric operands and tests pairs existentially using
the existing floating comparator. Both operands are resolved left-to-right
before pairing; an empty operand has no matching pair. This is restricted to
the admitted `<`, `<=`, `>` and `>=` plan, not equality/inequality type dispatch.

Numeric operand retention is charged before vector construction. Scalar operands
stay inline. Each attempted pair charges XPath work before comparison, so a
large Cartesian search remains bounded and cancellable. Partial operands are
ordinary local values released on failure; no resolver cell receives them.
The safe initial implementation prioritizes explicit accounting over a claimed
performance improvement. No performance measurement or optimization is claimed.

The added matrix covers later matches on either side, empty operands and NaN.
The first-party production golden adds node-set pairs and a numeric scalar
variable. A second new test pins exact/one-less pair-work limits, retention
failure, cancellation, owner release and completed-cell reuse. The reference
suite now has 80 tests. General global admission and equality/inequality
conversion coverage remain separate obligations.

ADR-0004 review: pair-operand preparation and bounded pairing live in a small
private `xslt10_compatibility/numeric_pairs.rs` child, rather than expanding the
pressured runtime dispatcher. It consumes existing frames, source guards,
controls and the shared comparator; no authority, host state, parser or demand
resolver semantics move into it. The compatibility parent retains scalar and
literal conversion ownership. This adds a named responsibility child, not a
new generalized value layer or public representation.

Validation passed: 1,816 core tests (60 ignored), 22 native tests (3 ignored),
four WASM adapter tests, 18 worker tests, formatting, strict Clippy, documentation,
Markdown links, unsafe-surface and pinned-conformance gates. Official WASM target
and diff checks passed. Fresh OASIS CD04 replays retain modern 2,500 exact /
14 mismatches and strict 2,497 exact / 17 mismatches, each over 3,173 cases with
423 expected-error credits. No upstream fixtures or expected results changed.

## Scalar equality demand bridge (2026-10-13)

Equality inspection found a different boundary from numeric relations. The
existing value-of variable-pair equality consumer explicitly rejects source
node-sets; condition equality uses a scalar-string plan also shared with modern
behavior. This tranche does not silently turn that plan into general XPath 1.0
equality or treat node-set inequality as negated equality.

A test-only fallback now routes missing scalar-string reads through the existing
demand resolver after ordinary atomic and temporary bindings have priority. It
uses the existing comparison-purpose tag: temporary values may be converted,
but source selections remain explicitly unsupported rather than reduced to their
first node. Read-site location remains unknown on this bridge, not invented.
No production branch, compiled plan or public contract changes.

Three new tests cover equal/different constructed values and repeated-read reuse;
source-sequence rejection and recursive equality-cycle cleanup before evaluating
the right operand; and value-of inequality with a local atomic binding shadowing
a terminating global constructor. The reference suite now contains 83 tests.

The next equality integration must distinguish legacy type-dependent comparison
from modern scalar/collation behavior explicitly. Source node-set, numeric,
boolean, string and temporary operands must not be flattened into one string
comparison. The current guarded slice is evidence for resolver reuse, not proof
of general equality/inequality support or a selected demand-evaluation policy.

Validation passed full repository gates: 1,819 core tests (60 ignored), 22 native
tests (3 ignored), four WASM adapter tests and 18 worker tests, formatting,
strict Clippy, docs, 1,989 Markdown link checks, unsafe-surface and pinned
conformance checks. Official WASM target and diff checks passed. No OASIS replay
was required for this test-only bridge; last measured profile totals are unchanged.

## Explicit string-conversion equality (2026-10-13)

The condition compiler previously erased the distinction between bare variables
and explicit `string($variable)` operands. Its private equality plan now retains
both operand conversion flags and local expression compatibility separately from
collation and compilation-graph preprocessing. A compiler matrix pins bare,
left-only, right-only and two-sided conversions under legacy and modern contexts.

Only two-sided explicit legacy string equality activates the existing controlled
string-conversion consumer. A source node-set therefore contributes its first
node's string value, or the empty string for an empty set, deliberately unlike
existential bare node-set comparison. Bare, mixed-conversion and modern behavior
remain unchanged; general type-aware equality and inequality are still open.
No new resolver, global admission policy, public profile or authority is selected.

Two demand-reference tests prove first-node versus later-node values, empty sets,
constructed temporary operands, repeated-read single evaluation, exact/one-less
string-value work limits, cancellation, cached owner retention and stack cleanup.
The reference/golden suite now contains 85 tests, plus the compiler matrix.
The first-party production golden also exercises explicit conversion without
demand hooks. Its additional assertions are regression evidence, not corpus credit.

ADR-0004 review: the 1,616-line boolean compiler retains one compilation owner;
the 824-line owned-value test unit remains within that subject. The 6,764-line
runtime composition unit retains decomposition debt. This bounded change adds
intent dispatch and reuses the existing conversion owner rather than installing
a type-conversion subsystem there. Broader equality implementation should belong
to the private value-comparison family, not grow the composition root.

Validation: all 85 focused reference/golden tests and 1,822 core tests pass
(60 ignored), as do 22 native tests (3 ignored), four WASM adapter tests and
18 worker tests. Full repository gates, official WASM target and diff checks
pass. Fresh unchanged OASIS replays conserve modern 2,500 exact / 14 mismatches
and strict 2,497 exact / 17 mismatches out of 3,173, with 423 expected-error
credits in each profile. No global-constructor corpus case is newly admitted.

## Typed legacy scalar equality (2026-10-13)

Bare legacy variable-pair equality now preserves scalar type instead of comparing
every value lexically. Conditions and value-of equality/inequality share one
private comparator. It applies boolean precedence, then numeric precedence,
then string comparison as specified by
[XPath 1.0 section 3.4](https://www.w3.org/TR/1999/REC-xpath-19991116/#booleans).
Temporary result fragments retain their distinct boolean meaning even when their
string value is empty. Explicit two-sided string conversion and modern comparisons
keep their previous dispatch; mixed explicit/bare operands are not expanded here.

Source node-set pairs and extended atomic types are explicitly unsupported, not
flattened into scalar strings. The test-only demand reader still rejects source
scalarization and reuses the same cells for temporary reads. No additional global
constructor admission, public type, resolver or evaluation policy is selected.

Three tests cover typed runtime-frame boolean/numeric/string combinations,
reversed numeric/string operands, signed zero, NaN, infinity, demand fragments,
and exact/one-less/cancelled comparison work with completed-binding reuse. Boolean
frames are supplied directly because local `select="true()"` remains outside the
current compiler slice; these tests do not claim to admit that syntax. The
production golden uses an actual numeric global and a local string `02`, pinning
both condition equality and value-of inequality without demand hooks.

ADR-0004: the 120-line `scalar_equality.rs` owns private scalar conversion and
comparison only, under the existing compatibility subject beside numeric-pair
relations. The composition root gains dispatch, not another conversion owner.
The owned-value reference tests are 963 lines; split the comparison test family
before substantial further growth rather than burying ownership tests in it.
The reference/golden suite now has 88 tests.

Validation: 1,825 core tests pass (60 ignored), with 22 native tests (3 ignored),
four WASM adapter tests and 18 worker tests. Full repository verification and
official WASM checks pass. Fresh OASIS replays retain modern 2,500 exact /
14 mismatches and strict 2,497 exact / 17 mismatches out of 3,173, with 423
expected-error credits each. No archival expected output or denominator changes.

## Source-to-source equality and comparison-test extraction (2026-10-13)

First, the comparison tests were extracted from the 963-line owned-value unit;
all 88 reference tests passed at that structural checkpoint. The final split is
409 lines for value ownership/lifetime tests and 658 lines for comparison tests
under `owned_values/`. Shared setup remains private to that subject. This names
the comparison responsibility without adding another runtime owner or widening
the public surface; normal module declarations preserve formatter discovery.

The private equality owner now also admits two source node-set operands.
Equality and inequality each use existential string-value pair comparisons;
inequality is not negated whole-set equality. Empty operands make both false.
Numeric-looking node strings remain strings. The module is now named
`variable_equality.rs` (189 lines), reflecting scalar and source-pair coverage.
Mixed node-set/scalar operands, foreign-effective-owner navigation, and mixed
explicit/bare conversion remain guarded. No general node-set equality claim,
new global-constructor admission or demand-policy selection follows.

Comparison buffers charge XDM item slots before vector retention, controlled
string-value visits during conversion, and `ResultTextByte` capacity charges
before retaining each string payload (before cloning detached payloads). Each
attempted pair charges XPath work. These are deterministic work/retention
charges, not process-memory accounting. Failure drops transient buffers while
already completed globals remain reusable; the source owners are unchanged.

Two new tests cover later matches, reversed operands, simultaneous true equality
and inequality, empty sets and lexical numeric-looking strings; and exact,
one-less and cancelled controls across all four charged domains with owner/stack
cleanup. The reference suite now contains 90 tests. The production golden adds
independent equality/inequality and empty-set checks without demand hooks. The
former general-pair rejection regression now pins the still-deferred mixed
node-set/scalar boundary rather than forbidding the newly admitted source pair.

Validation: all 90 focused tests and 1,827 core tests pass (60 ignored), plus
22 native tests (3 ignored), four WASM adapter tests and 18 worker tests.
Full repository verification, official WASM, Markdown links and diff checks pass.
Fresh OASIS replays conserve modern 2,500 exact / 14 mismatches and strict
2,497 exact / 17 mismatches out of 3,173, with 423 expected-error credits each.

## Mixed node-set/scalar equality (2026-10-13)

The shared private comparator now admits source node-sets against boolean,
numeric, string and temporary-fragment operands in either order. Boolean
comparison converts the whole set by nonemptiness; numeric comparison converts
each node string and compares existentially; string/fragment comparison uses
existential string equality. Inequality remains independent, including NaN and
empty-set behavior. The same typed operand resolution owns left-to-right reads;
there is no fallback that reduces a source set to its first node.

A compiled matrix supplies typed runtime frames and tests both operand orders,
numeric-looking strings, later matches, NaN, empty sets and a nonempty set whose
node string is empty. Boolean frame tests still do not admit currently unsupported
local boolean-declaration syntax. A demand fragment/source comparison now succeeds
while the recursive-constructor case still reports its original circularity and
cleans the stack. The former mixed-pair rejection test is now a production result
test; the golden also covers numeric globals against source sets in both orders.

The existing four-domain exact/one-less/cancellation test now executes mixed
numeric comparisons as well as source pairs. Buffer charges, owner retention,
completed-binding reuse and cleanup remain explicit. There are 91 focused tests.
Condition coverage is equality; variable-pair inequality is exercised through
value-of. A control-fixture attempt confirmed that bare variable `!=` in `xsl:if`
is still compiler-unsupported, and that syntax was not silently admitted here.
Modern comparison, mixed explicit `string()`/bare operands, foreign effective
owners, extended atomic types and global-constructor admission are not expanded.
This closes a bounded consumer gap, not AR-0030's global evaluation decision.

Validation: 1,828 core tests pass (60 ignored), plus 22 native tests (3 ignored),
four WASM adapter tests and 18 worker tests. Full verification, official WASM,
Markdown links and diff checks pass. Fresh OASIS replays retain modern 2,500
exact / 14 mismatches and strict 2,497 exact / 17 mismatches out of 3,173;
each profile retains 423 expected-error credits. No corpus credit is inferred
from the new first-party comparison matrix.

## Mixed explicit-string and bare equality operands (2026-10-13)

Legacy conditional variable equality now preserves conversion intent independently
for each operand. An explicit `string($value)` uses the existing controlled
first-node/empty-string conversion; a bare counterpart retains its node-set,
boolean, numeric or fragment comparison meaning. Both-explicit string comparisons
retain their existing path, and modern expression routing is unchanged. This
reuses the private typed equality owner rather than adding another evaluator.

Two additional reference tests cover both operand orders, first-versus-later
matches, numeric spelling, boolean conversion and empty selections, including
two independently demanded source bindings. Cached values resolve once and leave
no active stack. The existing exact/one-less/cancellation matrix now includes a
mixed-conversion comparison. The production golden independently verifies that
`string($threshold) = $nodes` can match a later bare source node without demand
hooks. These are first-party regression checks, not added conformance credit.

No new conditional inequality syntax, value-of explicit-pair syntax, modern
comparison semantics or foreign-owner navigation is admitted. General global
constructors and production demand resolution remain outside this slice.

Validation: all 93 focused reference/golden tests and 1,830 core tests pass
(60 ignored), alongside 22 native tests (3 ignored), four WASM adapter tests and
18 worker tests. Full repository verification, official WASM target checking and
diff checks pass. Fresh OASIS replays conserve modern 2,500 / 3,173 exact matches
with 14 mismatches and strict 2,497 / 3,173 with 17 mismatches; both retain 423
expected-error credits. No corpus gain or performance claim is made.

## Typed boolean and numeric variable relations (2026-10-13)

The next shared-consumer probe exposed a concrete relational conversion defect:
typed boolean operands were converted to lexical `true`/`false`, then parsed as
numbers, producing NaN. A failing pre-fix demand test recorded `no|no|no|no`
instead of `no|yes|no|yes` for a nonempty source set compared with true.

[XPath 1.0 section 3.4](https://www.w3.org/TR/1999/REC-xpath-19991116/#booleans)
requires node-set/boolean comparisons to use whole-set boolean conversion for
relational as well as equality operators. The existing private numeric-pair
owner now retains boolean type through operand preparation, compares whole-set
truth against it, and uses numeric zero/one for scalar relations. Already-typed
numeric values also retain infinity/NaN meaning instead of passing through the
string-to-number grammar; untyped text such as `INF` is still not a number.
The existing charged source traversal and numeric buffers remain intact; this
is a semantic correction, not a traversal optimization or budget reduction.

Three new tests cover all four relational operators across operand orders,
empty and nonempty node-sets, typed scalar booleans, signed zero, NaN and
infinities. The new first-party
[boolean-variable-relations golden](../../corpus/golden/boolean-variable-relations/README.md)
executes with an ordinary supplied boolean parameter and no demand hooks, plus
exact/one-less work limits and cancellation. No local boolean-expression syntax,
modern comparison route or general global constructor is newly admitted.
The reference suite now has 96 tests.

The numeric comparison owner remains private and cohesive (145 lines), and its
test family remains below the ADR-0004 size trigger (937 lines). No ownership,
public facade, parser, ABI, authority or dependency boundary changes here.

Validation: 1,833 core tests pass (60 ignored), with 22 native tests (3 ignored),
four WASM adapter tests and 18 worker tests. Full verification and the official
WASM target check pass. Fresh OASIS runs remain at modern 2,500 / 3,173 exact
(14 mismatches) and strict 2,497 / 3,173 exact (17 mismatches), with 423
expected-error credits in each. There is no new corpus credit or speed claim.

## Direct effective-boolean demand reads (2026-10-13)

A test-only effective-boolean hook now connects ordinary bare-variable conditions
to the same invocation demand cells. Completed/local values retain precedence;
only an unresolved, unshadowed global reaches the hook. The reference examines
the existing owned value kind rather than converting through text: a temporary
tree has a root and is true even when empty, while a source selection is true
exactly when nonempty. There is no second value map or constructor evaluator.

Six additional tests cover empty-tree truth and repeated reuse, empty/nonempty
source selections with empty text, local numeric shadowing, short-circuiting,
recursive cycle cleanup, failure memoization without repeated messages, and
exact/one-less/cancelled cached reads. The failure test preserves the originating
constructor span while explicitly leaving the new read edge unknown; it does
not manufacture a condition location from the constructor's location.

This bridge and its stored hook are entirely `cfg(test)`. Production reads,
global admission, error identities and public contracts do not change. Atomic
demand values and modern global-context policy remain unproved by this slice.
The test subject has its own private `effective_boolean.rs`; provenance checks
remain under the existing provenance owner without widening private field access.
The reference directory now has 15 direct files, still organized around one
demand-resolution experiment with named ownership, control and read families.
Further expansion should review grouping these read-consumer families rather
than appending indefinitely to either the comparison suite or the driver.

Validation: 102 focused tests and 1,839 core tests pass (60 ignored), alongside
22 native tests (3 ignored), four WASM adapter tests and 18 worker tests. Full
repository verification and the official WASM target check pass. Fresh OASIS
replays remain modern 2,500 / 3,173 exact with 14 mismatches, strict 2,497 / 3,173
with 17 mismatches, and 423 expected-error credits in each. No corpus credit or
production integration is inferred from these reference tests.

## Direct boolean condition provenance (2026-10-14)

The direct effective-boolean reference now receives the compiled condition site
through the existing location-aware boolean dispatcher and passes it to the
same bounded read-path owner. Nested `not`, `and` and `or` keep the enclosing
`if` or `when` site; this is a whole-instruction span, not an operand-token span.
The ordinary locationless entry point remains for unmigrated consumers and
explicitly supplies no location. No caller-focus or failure-location inference
is introduced.

The expanded nested-condition matrix exercises bare variable reads alongside
the existing string-length reads. Failure reuse preserves the original
terminating-message location and memoized observation while separately retaining
the condition read site. A new direct-boolean cycle test preserves all three
edges, including the closing `when`, and verifies both active stacks unwind.
All 103 focused tests pass. The hook remains test-only; production demand
resolution, global admission and diagnostic attachment policy remain undecided.

Checkpoint validation: full repository verification passes with 1,840 core tests
(60 ignored), 22 native tests (3 ignored), four WASM adapter tests and 18 worker
tests. The official WASM target check and diff checks pass. Fresh OASIS replays
conserve modern 2,500 / 3,173 exact with 14 mismatches and strict 2,497 / 3,173
with 17 mismatches; both retain 423 expected-error credits. No corpus gain is
claimed for diagnostic-reference integration.

## Boolean value-expression read provenance (2026-10-14)

The existing value-expression dispatcher now forwards its known instruction
location for `boolean($value)`, admitted boolean-literal comparisons and
two-variable `and`. These consumers use the same location-aware effective-
boolean reader as conditions, with no new boolean evaluator or global state.
The location is still test-only and denotes the containing instruction rather
than an invented operand span.

Two additional tests cover the four expression shapes, primary failure versus
read-site separation, memoized failure/message reuse, and a shadowed false left
operand that skips the failing right operand with observation enabled or
disabled. No skipped operand acquires a dependency edge. Syntax admission,
production global initialization and diagnostic attachment policy are unchanged.
The reference now contains 105 focused tests.

Validation: all focused tests and 1,842 core tests pass (60 ignored), alongside
22 native tests (3 ignored), four WASM adapter tests and 18 worker tests. Full
repository verification, the official WASM check and diff checks pass. Fresh
OASIS replays conserve modern 2,500 / 3,173 exact (14 mismatches) and strict
2,497 / 3,173 exact (17 mismatches), with 423 expected-error credits in both.

## String-literal comparison read provenance (2026-10-14)

Admitted legacy variable/string-literal comparisons now forward known instruction
locations through the existing comparison consumer. This covers conditions and
value-of equality, inequality and the already-admitted negated equality form.
The source-value hook accepts an optional site as well, so typed source handoff
does not silently discard it. Its existing locationless entry point supplies
`None` for unmigrated callers. Comparison meaning, selection ownership and
production global admission are unchanged.

Two previous unknown-location assertions now require the actual `if` site,
including the middle read of a copy/comparison/copy cycle. One new test exercises
value-of comparison forms, originating-message versus read-site distinction,
memoized failure reuse and active-path cleanup. All 106 focused tests pass.
No general comparison-operand token spans or production diagnostic-attachment
contract are selected.

Validation: 1,843 core tests pass (60 ignored), alongside 22 native tests
(3 ignored), four WASM adapter tests and 18 worker tests. Full repository
verification, official WASM checking and diff checks pass. Fresh OASIS runs
remain modern 2,500 / 3,173 exact (14 mismatches) and strict 2,497 / 3,173 exact
(17 mismatches), each with 423 expected-error credits. No corpus gain is claimed.

## Actual declaration-body lowering reference (2026-10-14)

A test-only compiler entry now lowers children of actual `xsl:variable`
declarations through the existing sequence compiler. The demand driver accepts
these immutable instruction plans directly; older named-template surrogates
remain the differential scaffolding. No XML rewriting, new instruction evaluator,
production global variant, runtime policy or public API is introduced.

Four tests establish forward template-discovered dependencies, single evaluation,
strict-versus-modern comment-separated text, COW/complete-clone parity, original
declaration-child failure locations after stylesheet-owner release, memoized
messages, circularity cleanup, exact/one-less limits in four work domains, and
cancellation in each domain. The forward-dispatch document is also compiled by
the normal entry point and still fails with `FXST1015`, explicitly conserving
production's unsupported boundary.

This is **body lowering, not whole-stylesheet integration**. The surrounding
program still comes from an explicit placeholder-declaration scaffold; its
binding metadata and linkage are not evidence that the original global
declarations were admitted. The reference body map is test-only and is not a
selected production plan representation. `variable40` is not credited.

ADR-0004 review: the compiler composition unit is 3,093 lines and gains only an
11-line test-only forwarding seam into its existing sequence compiler. It owns
no new lowering algorithm or runtime state; retain this bounded seam rather than
create an artificial module for a forwarding function. The reference driver
remains below 1,000 lines. The 15-file reference directory gains no file; the
existing evaluation-schedule family now owns declaration-body integration tests
as well as schedule comparisons. Future production admission must review the
compiler composition pressure before adding a declaration-plan owner. Existing
dependency direction remains compiler -> static instructions -> runtime; no
compiler dependency on runtime is added.

Validation: all 110 focused tests pass. Full repository verification passes with
1,847 core tests (60 ignored), 22 native tests (3 ignored), four WASM adapter
tests and 18 worker tests, plus strict Clippy, formatting, documentation, links,
unsafe-surface and pinned-corpus checks. The official WASM target check and diff
checks pass. Fresh unchanged OASIS replays conserve modern 2,500 / 3,173 exact
(14 mismatches) and strict 2,497 / 3,173 (17 mismatches), with 423 expected-error
credits in each. No performance measurement or production-support credit is
attached to this tranche.

## Single-module declaration/link reference (2026-10-14)

The next test-only compiler owner, `global_constructor_reference.rs`, reads the
original declarations and bodies from one unchanged stylesheet. It reuses the
ordinary global attribute/name/required checks, sequence compiler, top-level
compiler and named-template validator. This removes the separately authored
placeholder stylesheet from this lane. Actual declaration locations and expanded
names survive; duplicate names using different prefixes for the same URI fail.

The returned sidecar still pairs resolved binding metadata with instruction
bodies, using a private empty default marker that the reference driver never
materializes. That marker is not a selected production representation. No
source-dependent state enters compiled plans, and no second evaluator is added.
The ordinary global compiler calls the same validation helper with its unchanged
default compiler; production admission remains unchanged.

Three regular tests cover forward matched/named dispatch, qualified variable
reads, supplied-parameter precedence, original declaration provenance, duplicate
and malformed names, invalid required values, missing nested named calls and
explicit profile guards. This lane rejects select defaults, typed/required and
empty defaults, imports/includes, namespace aliases and attribute sets. Empty
defaults must not accidentally acquire result-tree-fragment truth semantics.
Broader composition and rewriting must be proved before those guards move.

The manually ignored `demand_declaration_reference_replays_local_variable40`
test reads original local OASIS stylesheet, source and expected bytes under
`FASTXSLT_OASIS_XSLT10_ROOT`. The archive SHA-256 was verified as
`66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`.
Both modern and strict effective-stylesheet runs pass the **existing unchanged
OASIS XML comparator**, and `TreeFrag` evaluates once. Initial dispatch models
the built-in document rule without injecting a stylesheet template. The shared
executor's semantic nodes go through the existing serializer.

An initial byte-string assertion failed solely on a newline after the XML
declaration. That is not a byte-exact pass: the replay now uses the existing
suite comparator's established prolog treatment, not a new normalization or
edited expected output. The normal compiler still returns `FXST1015` for the
unchanged stylesheet. This experimental result adds **no production corpus
credit** and does not amend the conserved numerator.

Reproduce the local replay with the explicit corpus root set, then run
`cargo test -p fastxslt --all-features demand_declaration_reference_replays_local_variable40 -- --ignored --nocapture`.

ADR-0004 review: the 3,110-line compiler composition unit gains a narrow
default-lowering callback and a test-only module declaration, not another
sequence compiler. The 109-line private reference module owns experiment
composition only; it consumes compiler-owned documents/plans and must not acquire
runtime state or resource authority. The compile directory now has 19 direct
files: retain named compiler responsibilities flat for this bounded checkpoint,
rather than mix test experiment orchestration into the production composition
loop. Revisit grouping with the next production global-plan extraction. The
runtime driver remains below 1,000 lines, and its semantic-node return seam uses
the same executor and existing result-to-temporary conversion. No duplicated
instruction semantics, new unsafe surface or public API is introduced.

Validation: 113 focused tests pass, with the ignored local `variable40` replay
executed separately and passing in both profiles. Full repository verification
passes with 1,850 core tests (61 ignored), 22 native tests (3 ignored), four
WASM adapter tests and 18 worker tests. Formatting, strict Clippy, documentation,
local links, unsafe-surface and pinned-corpus gates pass, as do the official WASM
target and diff checks. Fresh full OASIS replays retain modern 2,500 / 3,173
exact (14 mismatches) and strict 2,497 / 3,173 (17 mismatches), each with 423
expected-error credits. No performance claim or production corpus gain follows.

## Single-include constructor linking (2026-10-14)

The test-only declaration compiler now admits exactly one explicitly supplied
include. Both effective documents are supplied by the caller under one explicit
compilation-context token; neither URI resolution nor filesystem/network
acquisition is added. Nested includes, multiple dependencies and imports remain
guarded. The existing single-module entry retains its prior guard.

Module bodies are lowered before named-call validation, then the existing
`merge_included_program` combines definitions and rejects same-precedence
duplicates. Only after that merge does the reference validate ordinary templates
and constructor bodies against the combined named-template set. It does not
implement alternative include precedence or copy a second linker. Constructor
body maps merge only after the shared linker has rejected equal-name collisions.

Three tests prove cross-module named calls and global reads, namespace-qualified
binding identity, one evaluation per binding, original declaration ownership,
strict/modern whitespace behavior and COW/complete-clone parity. Missing named
calls retain the included instruction location; duplicate included globals retain
the existing `FXST0029` result. A cross-module cycle and an included terminating
message both unwind, memoize failure and preserve originating-module evidence.
Explicit nested/multiple/import guards remain covered.

The import path was inspected but not duplicated: its existing owner combines
precedence rebasing, root-template handling and definition merging after compiling
the principal module. The next import experiment should first expose that
already-compiled-program composition as a behavior-preserving checkpoint, then
attach constructor plans to the **winning resolved declaration**, not merely
overwrite a name-keyed map. Include success does not prove that import rule.

ADR-0004: the 1,811-line module compiler changes only the visibility of its
existing include merge to its parent; no runtime dependency or second merge
algorithm is added. The private declaration reference is 168 lines. The
791-line evaluation-schedule test family remains bounded, but declaration/link
tests now form a clear subject for extraction before further module-graph growth.
That extraction should be a separate passing checkpoint, not mixed with import
semantics. The existing 19-file compile-directory review still applies.

Validation: full repository verification passes with 1,853 core tests (61
ignored), 22 native tests (3 ignored), four WASM adapter tests and 18 worker
tests. Strict Clippy, formatting, documentation, Markdown links, unsafe-surface
and pinned-corpus gates pass, alongside the official WASM target and diff checks.
The unchanged manual `variable40` replay passes both profiles again. The full
OASIS denominator was not rerun for this test-only include tranche; the previous
modern 2,500 / 3,173 and strict 2,497 / 3,173 measurements remain the latest
accounting. No production support, corpus gain or performance claim is made.

## Import composition and winning constructor plans (2026-10-14)

First, a behavior-preserving checkpoint extracted the existing single-import
composition into `merge_single_imported_program`: the normal compiler still
supplies the same compiled principal, imported program and diagnostic locations.
Precedence rebasing, output/definition merging, root-template handling and
validation stay in that owner. Declaration/link tests moved from the schedule
file into the private `declaration_linking` subject. All 1,853 core tests passed
before the reference gained import behavior; no assertion or fixture changed
in that structural checkpoint.

The private reference then gained one explicitly supplied import. It validates
import order, lowers both modules, calls that same composition owner, and selects
each constructor body using the winning binding's name and resolved precedence.
This deliberately bounded graph has levels 0 and -1. It does not select bodies
by insertion order, URI spelling or equal source spans. Losing bodies are dropped;
neither source-derived state nor new acquisition authority is introduced.

Three tests cover qualified-name override, named-template override, a surviving
imported global reading the principal's winning value, strict/modern text and
COW/complete-clone parity. Overridden terminating constructors/templates never
execute. Imported failures retain their instruction resource; recursive cycles
unwind and repeated reads reuse the failure. Late imports retain `XTSE0200`.
Wrong-kind, multiple and nested dependencies remain explicitly guarded.

This is not a general graph linker or a decision about static errors in
overridden declarations. Both bodies are lowered; these tests use statically
valid losing bodies and prove only that their dynamic failures do not execute.
Transitive/mixed graphs, broader value kinds and public/production integration
remain separate gates. No resolver ADR is selected by these tests.

ADR-0004: the module-composition owner is 1,817 lines and retains one existing
linking responsibility; extraction adds no alternate precedence implementation.
The private reference is 214 lines. The schedule file shrank from 791 to 404
lines; declaration/link evidence now has its own 536-line module. The reference
directory retains 15 direct files and gains a named subdirectory rather than
another flat sibling. Dependency direction stays test driver -> compiler plans
-> shared runtime; neither compiler nor runtime imports the test driver.

Validation: full verification passes with 1,856 core tests (61 ignored), 22
native tests (3 ignored), four WASM adapter tests and 18 worker tests. Formatting,
strict Clippy, documentation, links, unsafe-surface and pinned-corpus gates pass;
the official WASM check and diff checks pass. The manual unchanged `variable40`
replay passes both profiles after extraction and import integration. Fresh full
OASIS measurements conserve modern 2,500 / 3,173 exact (14 mismatches) and strict
2,497 / 3,173 (17 mismatches), with 423 expected-error credits each. No new corpus
credit, production demand admission or performance claim is made.

## Edition obligations checkpoint (2026-10-14)

The scheduling probes do not select a standards edition. In particular,
ADR-0023's preprocessing profile is not an evaluation-strategy switch.

- Legacy: global constructors start at the source root with a singleton current
  node list. Referenced globals precede their consumers; circular definitions
  are errors. These are the obligations exercised by the reference's root and
  dependency tests. Section 11.4 alone does not establish a blanket permission
  to suppress every unused failing definition; do not import modern error
  latitude into a strict legacy claim without further review.
  [XSLT 1.0 section 11.4](https://www.w3.org/TR/1999/REC-xslt-19991116#top-level-variables).
- Modern: the global context item is distinct from initial match selection.
  Available global focus has position/size one; absent focus must not silently
  become the principal root. Library-package global focus is absent. The
  current legacy driver does not prove these broader invocation modes.
  [XSLT 3.0 initialization](https://www.w3.org/TR/xslt-30/#initiating).
- Modern dynamic-error reporting need not evaluate otherwise unused constructs;
  a false conditional must not evaluate its body. Therefore eager/demand
  equality on unused failures is not a universal correctness oracle. Errors
  initializing globals are not caught merely by wrapping a reference in
  `xsl:try`. The reference has no catch integration and earns no such claim.
  [XSLT 3.0 error handling](https://www.w3.org/TR/xslt-30/#error-handling),
  [initialization](https://www.w3.org/TR/xslt-30/#initiating).

Implementation consequence: one demand owner can serve different entry-context
and initialization policies, but this is a candidate, not an accepted decision.
Keep the current root-only experiment explicitly legacy-scoped. A production
proposal must separately choose unused-binding initialization, diagnostic codes,
message observations and control charging; the reference's `FXAR0030` is not a
selected standards circularity diagnostic. Static reachability remains an
unimplemented alternative, not something the eager schedule has benchmarked.

## Same-owner schedule cost probe (2026-10-14)

`declaration_linking/schedule_cost.rs` adds one ignored release-mode measurement,
not another conformance case. Run it explicitly:

```text
cargo test --release -p fastxslt --all-features demand_declaration_schedule_cost_probe -- --ignored --nocapture
```

Three actual-declaration fixtures use one constructor, all sixteen constructors,
or one of sixteen constructors. Every constructor makes `<box>payload</box>`;
the principal template reads the selected values. Both lanes use the same demand
owner and shared executor. Lane zero reads on demand; lane one first requests
every binding in declaration order. Each invocation checks output, exact total
evaluation count and empty evaluation stack.

Compilation/preparation are outside timing. Fresh control/resolver construction,
constructor-plan-map cloning, execution, string result checking and teardown are
inside both lanes. Each fixture warms each lane for 100 ms, then alternates lane
order over seven 100 ms windows. Report per-window mean time per invocation,
not request latency percentiles. This is one-machine exploratory attribution,
without steady-state publication qualification or allocator measurement.

Crucially, the private reference clones all constructor plans even when demand
skips their execution. These observations cannot isolate cell/read-hook overhead,
compare against the production completed-globals initializer, or establish a
host-visible speedup. The first exploratory run overlapped verification and is
excluded from timing evidence. Static reachability is not implemented here.

Two subsequent release runs, with repository verification finished, produced
these median window means in microseconds per invocation:

| Actual constructors used/declared | Demand run 1 / 2 | Eager run 1 / 2 |
| --- | --- | --- |
| 1 / 1 | 3.011 / 3.361 | 3.013 / 3.244 |
| 16 / 16 | 35.682 / 34.108 | 33.335 / 33.407 |
| 1 / 16 | 9.840 / 8.024 | 41.377 / 30.128 |

The all-used fixtures show no demand throughput advantage. Avoiding fifteen
unused constructors reduces time in both runs, but magnitude is unstable and
does not settle the permitted evaluation policy. Individual window means span
2.885–6.747 us for the one-constructor fixture across lanes/runs. No process
isolation, CPU attribution or statistical publication gate was used; do not
turn these observations into a production speed ratio. The 1/16 demand cost
also exposes the reference's all-plan cloning/setup overhead relative to 1/1.

Deterministic XPath-operation charges are demand/eager **1/2**, **16/32**, and
**1/17**, respectively. The extra eager requests are real reference read charges;
they are not a proposed production charging policy. Unused-work savings do not
authorize suppressing errors under a standards edition whose policy is unsettled.

ADR-0004: the measurement has a named private owner under declaration/linking
evidence; no new flat sibling is added to the 15-file reference directory.
No production implementation, public contract or corpus accounting changes.

Validation: full `scripts/verify.ps1` passes with 1,856 core tests and 62 ignored
tests, plus 22 native, four WASM adapter and 18 worker tests. Official WASM check
passes. Both final manual release runs pass every semantic/count/cleanup check.
The existing 119 regular reference tests and unchanged-corpus replay remain
separate from this new ignored measurement. OASIS was not rerun for this
test-only timing/documentation tranche; prior exact counts remain the checkpoint.

## Ordinary-global production control (2026-10-14)

The next private probe, `declaration_linking/production_cost.rs`, compares the
unchanged production initializer with demand and eager-through-demand using the
same normally compiled XSLT 1.0 program. Its globals select `/r/b`; no placeholder
declarations, constructor map, alternate compiler, or supplied parameters are
used. Fixtures cover zero globals, one read of one global, 64 reads of one
global, all sixteen globals, and one of sixteen globals.

All lanes return ordinary semantic result-node vectors. The demand lane invokes
the shared sequence executor directly, not the helper that constructs a
temporary result tree and then stringifies it. Preparation/compilation stay
outside timing; fresh invocation state, ordinary control, execution, semantic
result comparison and destruction stay inside. Production still performs its
normal entry dispatch/global initialization; the reference installs its private
hooks and resolver. These are whole-lane differences, not an isolated estimate
of an `Arc`, cell, lookup or callback.

The no-global fixture is a reference-plumbing control, not a reason to install
demand hooks in production programs that need none. This is a normally supported
source-node global workload, not coverage of all atomic/typed global kinds or a
production implementation of general constructors. No unsafe, acquisition or
compiled-state ownership change is involved.

```text
cargo test --release -p fastxslt --all-features ordinary_global_production_cost_probe -- --ignored --nocapture
```

Each lane warms for 100 ms. Six 100 ms measurement windows rotate all three
lanes: zero is production, one demand, two eager-through-demand. Output reports
window-mean time, median/min/max and XPath-operation charge. It does not report
latency percentiles or claim a host benchmark. The regular semantic-parity test
runs all five fixtures through all three lanes without asserting wall time.

Three manual release executions passed. Run two's captured console excerpt was
truncated for two cells, so the complete run-one/run-three summaries are retained
below rather than inventing those missing values. Run two's visible 0/0, 16/16
and 16/1 cells agree directionally. Values are median window means in us:

| Globals / reads | Production run 1 / 3 | Demand run 1 / 3 | Eager-demand run 1 / 3 |
| --- | --- | --- | --- |
| 0 / 0 | 0.570 / 0.502 | 0.753 / 0.673 | 0.740 / 0.683 |
| 1 / 1 | 1.632 / 1.130 | 1.999 / 1.432 | 1.976 / 1.482 |
| 1 / 64 | 8.852 / 6.175 | 11.308 / 8.819 | 11.058 / 8.731 |
| 16 / 16 | 12.788 / 9.072 | 14.348 / 10.286 | 15.346 / 10.403 |
| 16 / 1 | 10.140 / 7.695 | 2.744 / 1.809 | 12.319 / 8.317 |

This rejects treating the test reference as a drop-in performance improvement.
All-used demand is slower in these complete runs, while the no-global control
exposes reference setup cost. Wall time drifts materially between runs; there is
no publication-quality ratio or measured causal attribution to individual data
structures. Reference-only evaluation/stack assertions also remain inside timing.

Production reports zero XPath-operation charges for these simple paths/reads;
that is only this counter, not zero execution work. Demand reports 0, 1, 64, 16,
1 across the fixtures, and eager-demand 0, 2, 65, 32, 17. Node/text domains still
apply. Do not silently change existing budgets by substituting these counters.

Recommendation for a later ADR: select a semantic resolution owner, not the
test driver's physical representation. Completed-value reads and no-global
programs need a cheap path; general constructors must retain one authoritative
resolution meaning rather than adding a second evaluator. Stable binding slots
or pre-resolved reads are candidates for later measurement, not selected here.
Avoiding unused work is separately contingent on initialization/error policy.

Validation: full verification passes with 1,857 core tests (63 ignored), 22
native tests, four WASM adapter tests and 18 worker tests; official WASM check
passes. The regular reference count is now 120. This tranche changes only
test code and documentation; no production admission, OASIS replay or corpus
gain is claimed. The two small cost modules remain under the named private
declaration/linking evidence directory, without growing the flat reference root.

## Bounded static-admission comparison (2026-10-14)

`declaration_linking/static_admission.rs` supplies the previously missing static
candidate rather than calling eager-through-demand a static analysis. It walks
already compiled constructor instructions and selected named-template bodies,
collects resolved global-binding edges, and produces a dependency-first order.
It never evaluates XPath or implements template dispatch. Execution still uses
the same ordinary executor and reference owner. Each scheduled constructor is
checked to evaluate only its own cell: its dependencies must already be complete,
and no unplanned nested evaluation may occur.

This first candidate admits only text, direct legacy global string reads and
argument-free calls to parameter-free named templates. It caps globals and named
templates at 32 each, call-stack depth at 32 and instruction visits at the caller's
explicit experiment limit. These are experiment limits, not host defaults or
production compilation-budget accounting. Parameters, conditional instructions,
local bindings, dynamic dispatch and every unknown instruction shape fail closed.
An analysis refusal is not a standards invalid-input or circularity diagnostic.

Five regular tests establish:

- Forward dependencies through two named calls are ordered before execution;
  demand and the static schedule produce the same output using one executor.
- Qualified-name aliases retain the shared compiler's binding identity.
- Shared import winners determine both constructor and template dependencies;
  losing bodies cannot introduce false edges or cycles, under both preprocessing
  profiles.
- Conditional self-reads, unselected-mode self-reads and local shadowing are
  refused by this static subset while the demand reference executes correctly.
- Direct global cycles, recursive named calls and exhausted analysis limits have
  distinct private refusal reasons; exact/one-less visits and 32/33 definition
  caps are pinned. Supplied-parameter semantics are not guessed by the analyzer.

The manual unchanged `variable40` replay additionally requires this analyzer to
refuse the constructor while the demand reference produces the archived XML
comparison. No fixture bytes or expected outputs change.

Conclusion: static admission has demonstrated a useful bounded subset, but that
subset does not cover the motivating dispatch. This does **not** prove that a
more sophisticated mode/condition-aware static analysis is impossible. Expanding
it would add another analysis subsystem before establishing additional consumer
value. It is not a prerequisite to selecting the runtime ownership boundary.

### Accepted decision boundary

The proposal was accepted with modifications as
[ADR-0024](../ADR/ADR-0024-invocation-owned-global-resolution-with-shared-execution.md).
Static analysis is explicitly admission/scheduling only, not dependency semantics.
Specialized initializers are proven optimizations of the same binding semantics,
with shared-execution fallback when eligibility is unproved. If that fallback is
not admitted, the operation stays explicitly unsupported.

The selected direction is one invocation-owned
global resolver with shared-executor constructors, immutable completed values,
bounded active/failed state and no recursive mutable borrow. Static dependency
ordering may seed or schedule that same owner where proved; it must not become
an alternate global evaluator. Do not select the current callback/name-scan test
driver as the production representation, given the ordinary-global cost result.

For the first integration, preserve eager initialization of the
admitted global set: request each binding through
the same owner, letting recursive reads resolve actual dependencies. This avoids
requiring unused-error suppression merely to admit `variable40`. This does not
promise declaration-order side effects or prove all future legacy/modern
scheduling cases. Broader modern global-context support remains separate.

Acceptance authorizes implementation direction, not automatic support for
every global kind, graph, diagnostic attachment or modern context mode. Retain
the existing unsupported boundary until each production slice passes parity,
control/provenance, lifetime, corpus and performance gates. No runtime change or
coverage credit follows from acceptance alone.

Validation: five new static-admission tests pass; the full verification script
passes with 1,862 core tests (63 ignored), 22 native tests, four WASM adapter
tests and 18 worker tests. The official WASM check passes. Manual unchanged
`variable40` replay passes both profiles with the new static-refusal assertion.
The static candidate is a cohesive 263-line private test module under the
four-file declaration/linking directory; no production layer imports it.
The prior full OASIS counts remain the checkpoint; this test-only tranche does
not rerun the full corpus or claim a pass increase.

## Production initialization decomposition checkpoint (2026-10-14)

Before adding the ADR-0024 resolver, the existing global-initialization routines
were extracted unchanged from `runtime_context.rs` into the private
`runtime_context/global_initialization.rs`. The parent falls from 2,571 to 1,763
lines; the new owner is 836 lines. This addresses ADR-0004 responsibility pressure
before further semantic expansion, rather than mixing extraction with resolver
behavior changes.

The child owns existing initializer dispatch and publication. Lexical frames,
temporary-tree types and helpers shared with ordinary execution remain in the
parent. Existing callers retain the same private entry through a re-export;
the extraction introduces no public API, resolver state, alternate executor or
resource authority. A whitespace-normalized comparison confirms the moved
function bodies are unchanged apart from the entry visibility and one relative
module path. The runtime-context directory has five direct files; no additional
folder decomposition is warranted at this checkpoint.

Validation: the full `scripts/verify.ps1` gate passed, including formatting,
Clippy, workspace tests, documentation, Markdown links and pinned conformance
checks. The official `wasm32-unknown-unknown` adapter check passed. The manual
unchanged `variable40` reference replay still agrees under both profiles; this
remains experimental and earns no production credit. Fresh full OASIS replays
retain modern **2,500 / 3,173** exact comparisons and 14 mismatches, and strict
**2,497 / 3,173** exact comparisons and 17 mismatches, with 423 expected-error
credits in each profile. Original expectations remain unchanged.

This completes the structural checkpoint only. Invocation-owned production
resolution, shared-constructor admission and production `variable40` remain the
next implementation work; the test-only callback/name-scan reference is not
promoted wholesale.

## Owned initializer publication checkpoint (2026-10-14)

The production specialized initializers now return one private owned
`InitializedGlobal` value instead of mutating completed-global maps while
evaluating. Empty, atomic, principal-source selection, temporary-tree and literal
document results move into the existing kind-specific maps at a single
publication point. Supplied atomic parameters use that same publication point.
The transient return value is not a second cache and adds no value-map clone,
new reference-counted wrapper, authority or public representation.

This removes mutable completed-map access from initializer evaluation, a
prerequisite for recursive resolution. It does not yet remove all immutable
dependency-map access or introduce resolver cells. Existing eager order,
source-owner retention, parameter validation, alias eligibility, failure codes,
message timing and work charges are preserved. A failed initializer returns no
publishable value; messages already emitted before failure remain observable.
No new cancellation point or work unit is selected in this refactor.

Five production-handoff tests pin movement into each completed value kind,
temporary/document identity, immutable dependency reads without map cloning,
failure without partial publication, exact/one-less retention and cancellation,
supplied-parameter override, unused default failure and principal-owner retention.
The initializer owner is now 803 lines; its focused test sibling is 199 lines.
The runtime-context directory remains below the density-review threshold.

Full repository verification and the official WASM check pass: 1,867 core tests,
22 native tests, four WASM tests and 18 worker tests pass. Fresh OASIS replays
remain modern **2,500 / 3,173** with 14 mismatches and strict **2,497 / 3,173**
with 17 mismatches; each retains 423 expected-error credits. These are regression
results, not new corpus credit or performance evidence.

Next, connect authoritative binding state and recursive reads without retaining
map borrows across shared execution. Preserve completed-read fast paths and
parameter precedence. Compiler admission of general constructors, runtime
circularity/provenance integration and production `variable40` remain pending.

## Resolve-before-borrow initializer reads (2026-10-14)

Production initializer dependencies now pass through a narrow private
`GlobalInitializerReads::with_completed` boundary. A resolving implementation
can finish evaluation before lending the completed maps to a non-recursive
conversion callback. The callback returns an owned value, not a borrowed map
entry. Existing production calls still use `RuntimeGlobals` directly: this is
statically dispatched plumbing, with no new trait object, map clone, name index,
state allocation or work charge. It is not a public provider interface.

Alias, variable-rooted source paths, concatenation, temporary text parts and
conditional-text reads use the boundary at their actual read sites. In
particular, a source-path precondition still fails before requesting a dependency;
messages preceding a text-part read remain before recursive initialization.
Dependencies are not speculatively evaluated at initializer entry.

Five additional handoff tests use a bounded, acyclic test driver around the
actual production initializer evaluator. They prove recursive entry with no
completed-map borrow held, message ordering, repeated-read single evaluation,
source/alias/concat/temporary value parity, identical work charges against the
completed-map reference, failure reuse without partial publication, precondition
ordering and nested cancellation cleanup. The handoff suite now has ten tests.
This driver is not a production state machine, cycle implementation or unwind
contract. Those remain obligations of the authoritative production resolver.

The production initializer module is 835 lines and its focused tests 475 lines;
both remain cohesive below the source-size review threshold. Ordinary completed
reads and eager scheduling are unchanged. General shared-constructor admission
and production `variable40` are still unsupported; no corpus credit is earned.

Validation passes: full repository gates (1,872 core tests, 22 native, four WASM
and 18 worker), official WASM target check, Markdown links and diff checks.
Fresh OASIS replays retain modern 2,500 exact / 14 mismatches and strict 2,497
exact / 17 mismatches out of 3,173, with 423 expected-error credits each.

## Production specialized-initializer owner (2026-10-14)

`materialize_global_defaults` now runs the existing eager schedule through the
private `GlobalInitialization` owner. All existing specialized initializers use
that owner through the resolve-before-borrow boundary. Completed values remain
in one `RuntimeGlobals` store, moved out when initialization finishes; ordinary
principal execution still reads the completed store directly. No-global programs
retain an early return without constructing the resolver.

The owner tracks active linked binding ordinals in a fixed 128-entry stack,
avoiding a per-binding state-vector allocation or second value map. Completed
reads check the existing maps; only an unresolved read searches linked binding
declarations for its ordinal. This is not the reference driver's per-read name
scan, but unresolved lookup remains a cost/accounting review point before broad
constructor admission. A recursive unresolved entry charges one XPath operation
before entering; already ordered eager initialization and completed reads retain
their previous work charges. The depth ceiling is a private stack-safety bound,
not a host deployment quota or an XSLT semantic limit.

Active re-entry reports `XTDE0640`; depth exhaustion reports `FXRT0003` with the
limit category. The first initialization failure aborts the phase and is retained
once, rather than cloning a diagnostic into every dependent binding. Already
completed values are not partially overwritten; repeated failed initialization
does not replay messages. A drop guard clears active markers during success,
error, cancellation and unwinding. An unwound owner is poisoned and cannot resume;
this neither catches engine panics nor promises recovery from process abort.

Eight production-owner tests join the ten handoff tests. They cover recursive
read/message order, eager unused evaluation, repeated reuse, supplied-parameter
default suppression, runtime cycle re-entry, stable failure/no replay, 128/129
depth boundaries, exact/one-less recursive work, cancellation, unwind poisoning
and ordered source-value/work parity. Recursive ordering and cycles are exercised
with deliberately assembled/reordered private plans: these tests do not claim
that compiler admission has widened. Existing compilation still performs its
previous bounded static ordering/validation.

The state owner is a separate cohesive 235-line private module, rather than
growing initializer semantics or the runtime-context parent. Production general
constructor plans, shared-executor reads during initialization, declaration and
read-edge provenance, broader owned values and cost gates remain outstanding.
In particular, this checkpoint does not admit `variable40` or select new profile
semantics. The original reference remains available as a differential oracle.

Validation passes: formatting, Clippy, 1,880 core tests (63 ignored), 22 native
tests, four WASM tests, 18 worker tests, documentation, pinned corpus checks,
Markdown links, diff checks and the official WASM target. Fresh full OASIS
replays remain modern **2,500 / 3,173** with 14 mismatches and strict
**2,497 / 3,173** with 17 mismatches; each retains 423 expected-error credits.

One post-integration release rerun of `ordinary_global_production_cost_probe`
also passes semantic checks. Production-lane medians for global/read counts
0/0, 1/1, 1/64, 16/16 and 16/1 are respectively **587.1, 1,162.9, 6,054.8,
9,339.3 and 7,629.4 ns**. Production XPath-operation counts remain zero for
these already ordered source-path fixtures (not zero total work). The probe
uses the previously documented warm-up and six rotated 100 ms windows. This
single rerun is not a controlled before/after comparison, a convergence result
or host-performance evidence; the cost gate for broader constructor integration
remains open.

## Required follow-up

### Bounded production constructor checkpoint (2026-10-15)

The ordinary compiler now retains a private shared constructor for eligible
legacy global content. The invocation owner supplies a borrowed initialization
context to the ordinary sequence executor, with no completed-map borrow across
recursive calls and no second value cache. Legacy string reads return through
that owner; root focus and lexical state are freshly established per initializer.
The existing eager schedule, supplied-parameter override and resource authority
are unchanged. Caller multiple-match policy is carried into initialization.

Admission deliberately remains narrower than the reference. Module merges,
template parameters, unproved match patterns and namespace-alias/attribute-set
rewriting are rejected. Executed instructions and value consumers must be on the
connected slice; unconnected reads cannot silently consult the empty completed
map used by the constructor executor. New modern global contexts remain denied.
Static admission checks are conservative unsupported boundaries, not cycle
predictions. Actual re-entry is resolved by the invocation owner.

Nine production constructor tests cover forward named/matched reads, repeated
messages, singleton root focus, caller-local isolation, cycles, terminating
failure/reuse, cancellation, exact/one-less instruction and result node/text
charges, parameter override, host match policy and module/context/consumer
guards. Shared-cycle failures retain the constructor declaration location;
instruction failures retain their own locations. General bounded secondary
dependency context is still pending.

Secondary-document string reads during constructor initialization also remain
explicitly unsupported, rather than being mislabeled as unbound variables.

The ordinary-global release control was rerun after executor integration:
production median ns for `(globals, reads)` **(0,0), (1,1), (1,64), (16,16),
(16,1)** were **475.5, 1,166.9, 6,375.0, 9,591.1, 8,226.9**. The same semantic
checks and zero XPath-operation counts hold in that production lane. This is
one noisy six-window rerun, not a paired before/after, convergence result or
host regression conclusion; the broader performance gate stays open.

The manual unchanged `variable40` test now checks both production and the safe
reference. Full archive replays conserve all 3,173 cases, with **14 modern / 17
strict mismatches** and **423 expected-error credits** unchanged. The only exact
gain is `Lotus/variable_variable40#1`. No archived output, resource authority,
public facade or eager/lazy policy changed.

ADR-0004 review: compiler admission and executor eligibility live in separate
small private responsibility modules; the existing executor gains only an
initialization guard and borrowed context. Initialization remains its own owner.
This is not a new execution backend. The runtime-context directory currently has
ten files, still in ADR-0004's ordinary band. An initialization cluster is visible
for future directory review, without moving unrelated temporary/source owners.

Validation: all `scripts/verify.ps1` gates pass, including **1,889 core tests**
(63 manual probes ignored), native 22, WASM adapter 4 and worker 18 tests,
Clippy, formatting, documentation, pinned inventories and 1,991 Markdown files.
The official `wasm32-unknown-unknown` check and `git diff --check` also pass.
Cargo reports its existing incremental hard-link fallback to copying; no gate
was suppressed for that environmental warning.

Reproduction: `scripts/measure-oasis-xslt10.ps1`, the same command with
`-PreprocessingProfile strict-xslt10`, and the manual
`demand_declaration_reference_replays_local_variable40` test with the local
OASIS root supplied. Production admission does not close broader value-kind,
module-linking, provenance or host-performance gates below.

### Source-controlled production branches (2026-10-15)

The constructor guard now admits `if`/`choose` with constant, source-path
existence/string conditions and `not`/`and`/`or` combinations. This is an
eligibility check, not a second boolean evaluator. Conditions, short-circuiting,
template dispatch and branch bodies remain in the existing shared executor.
Global-valued conditions still fail as unsupported instead of reading the
constructor's empty completed-map placeholder. No global predicate conversion
semantics are inferred from the legacy string-value read bridge.

Seven production tests extend the previous nine:

- A source-controlled dependency that would form a cycle is not read on the
  untaken branch; taking it reports actual `XTDE0640` re-entry. Later eager
  initialization still evaluates the formerly unused binding after its
  dependency completes.
- Named and mode-selected matched dispatch preserve source conditions and
  avoid executing unsupported instructions in an untaken body.
- A global-valued condition remains visibly unsupported.
- Exact/one-less XPath visit budgets and cancellation prevent partial
  publication, clear active state and reuse the original failure.
- Eight concurrent invocations of one compiled program retain distinct source
  values and completed stores.
- Ordinary template execution and global-constructor execution agree under
  both modern and strict preprocessing views, including comment-separated text.
- Production effective whitespace views govern conditions, and the completed
  semantic result survives dropping the compiled program and source.

These are focused/golden regressions, not additional standards-suite credit.
Module linking, broader match/read consumers, full dependency provenance and
host performance remain separate gates. This slice adds no new owner, cache,
authority or public contract, and does not require another architecture decision.

Both complete OASIS replays remain **2,501 modern / 2,498 strict**, with 14/17
mismatches and 423 expected-error credits unchanged. Repository validation
passes with **1,896 core tests** (63 ignored), native 22, WASM adapter 4 and
worker 18, plus the official WASM target check and clean diff checks. No upstream
fixture or expected result was edited. The next read-integration slice should
address global-valued predicates explicitly, including node-set comparison and
effective-boolean distinctions, before relaxing the current guard.

### Production effective-boolean reads (2026-10-15)

Direct global variable conditions now request resolution before borrowing the
completed value. A single private completed-value helper serves both ordinary
execution and initialization, retaining typed atomic conversion, node-set
nonemptiness, true result-tree fragments and false empty sequences. There is no
conversion through string truthiness and no second evaluator or value cache.
Existing predicate work charging remains at the shared boolean evaluator;
unresolved recursive entries retain the resolver's additional charge.

Five focused tests cover seven value-shape cases, forward reads, actual versus
short-circuited cycles, local shadowing, unchanged eager completion, supplied
parameter overrides, exact/one-less operation budgets, cancellation cleanup and
stable failure reuse. Global string/numeric comparisons remain unsupported in
constructor conditions: a first-node string read cannot stand in for existential
node-set comparison. Secondary-document boolean reads also retain an explicit
unsupported boundary. No module or public-contract expansion is selected.

Validation passes: **1,901 core tests** (63 manual probes ignored), native 22,
WASM adapter 4 and worker 18; full formatting/Clippy/documentation/link and pinned
conformance gates; official WASM target and diff checks. Complete OASIS replays
remain **2,501 modern / 2,498 strict**, 14/17 mismatches and 423 expected-error
credits. The focused tests establish the new reader semantics, not a corpus
percentage increase or a host-performance claim.

### Production string-literal comparisons (2026-10-15)

The next bounded reader resolves a global before comparing it with a string
literal. Principal-source selections reuse the ordinary charged string-value
scan, without cloning the completed store or reducing a node set to its first
node. Equality and inequality are independently existential; a mixed set can
satisfy both and an empty set satisfies neither. Strings and result-tree
fragments retain the existing scalar conversion. No recursive evaluation occurs
while the completed-store read borrow is held.

Five new focused tests and the updated former refusal test prove forward reads,
later-node matches, both source orders, empty sets, ordinary-executor parity for
strings/source sets/empty and nonempty fragments, local shadowing, re-entry
cycles, stable failure reuse, exact/one-less node-visit limits and cancellation
without partial publication. Non-string typed scalars are explicitly refused
for both local and global bindings during initialization: this slice must not
turn typed equality into lexical equality. Numeric, variable-pair and secondary
document comparisons remain separate integration gates.

Validation: **1,906 core tests**, 63 ignored probes, native 22, WASM adapter 4,
worker 18; repository gates and official WASM target pass. Full OASIS profiles
remain **2,501 modern / 2,498 strict** with 14/17 mismatches and 423 expected-error
credits. This tranche adds reader parity, not a new conformance percentage.
The focused production test unit is now 897 lines; keep the next extraction
review on the comparison/control responsibility seam before further growth.

### Production numeric-literal relations (2026-10-15)

Relational conditions against numeric literals now request forward global
resolution before borrowing completed values. Ordinary and initialization reads
share a single controlled source string-value traversal; the existing numeric
predicate still owns conversion, relational operators and NaN behavior. There
is no new expression interpreter, completed-value cache or node-set-to-first-node
shortcut. Numeric-literal equality and variable-pair comparisons are not newly
admitted by this condition guard.

The admitted scalar forms are string/untyped, integer and decimal values, plus
temporary-tree strings. Boolean, float/double and other unproved typed operands
remain explicitly unsupported during initialization rather than inheriting
lexical conversions without parity evidence. Decimal parameter values are tested;
this does not admit decimal global-declaration syntax that the existing compiler
still rejects. Secondary documents remain guarded.

Seven production tests cover all four relation operators, reversed operands,
later-node matches, empty/all-NaN/mixed selections, ordinary-executor parity,
actual cycles and local shadowing, supplied parameters suppressing terminating
defaults, eight concurrent independent invocations, and exact/one-less
`XdmStringValueNode` limits plus cancellation after source selection is cached.
The budget oracle measures initialization separately from a subsequent read of
the completed result: read failure after publication must not be mislabeled as
partial constructor publication. Failed scan paths preserve the completed source
binding, clear the active marker and do not publish the dependent constructor.

ADR-0004 checkpoint: the 897-line constructor test unit now delegates comparison
reader/parity/control evidence to a private child. The parent is 704 lines and
the comparison child approximately 450 lines; runtime-context direct file count
stays ten. This is a test responsibility split only, not another runtime owner,
public module or execution path. Runtime resolution remains one invocation owner.

Validation passes with **1,913 core tests** and 63 ignored probes, native 22,
WASM adapter 4 and worker 18, repository verification and the official WASM target.
Unchanged OASIS replays conserve **2,501 modern / 2,498 strict**, 14/17 mismatches
and 423 expected-error credits. This is additional production-reader parity,
not a new corpus gain or a host-performance claim.

### Production typed pair equality (2026-10-15)

Legacy variable-pair equality conditions now resolve each global into the ordinary
equality evaluator's private owned operand. The resolver still owns binding state;
the comparison evaluator owns transient string/node-set/number/boolean/fragment
operands. The left operand survives recursive right-side initialization without
a completed-map borrow, a new global cache or a second equality implementation.
Source-string collection is shared with ordinary execution and retains its node,
string-value and retained-text charges; pair work keeps its existing operation
charges. Explicit `string()` retains first-node conversion, not existential
comparison semantics.

Nine tests cover ten ordinary-versus-constructor type combinations, node-set
nonemptiness versus boolean precedence, numeric versus string precedence, empty
fragments, explicit conversions, local shadowing, nested right-side resolution,
message-once behavior and actual-cycle cleanup. Supplied parameter checks include
decimal, boolean, infinity and NaN, with extended dates remaining unsupported.
Eight concurrent sources remain isolated. Exact/one-less and cancellation checks
cover XDM node buffers, source string-value traversal, retained text bytes and
XPath pair work, preserving completed operands without publishing a failed
dependent constructor.

The first test attempt also exposed a pre-existing compiler boundary: conditional
`$left != $right` is unsupported, although value expressions have a distinct
inequality path. That syntax remains explicitly refused. Tests use actual pair
equality and logical `not(...)`; no node-set inequality is implemented as negated
equality. Pair relational conditions and secondary-document operands remain
outside this tranche. No public representation or wider module admission follows.

The comparison evidence has a private 361-line pair-equality child; existing
constructor and literal-comparison units retain their named responsibilities.
Full verification passes with **1,922 core tests**, 63 ignored probes, native 22,
WASM adapter 4 and worker 18, plus the official WASM target. Both OASIS profiles
conserve **2,501 modern / 2,498 strict**, 14/17 mismatches and 423 expected-error
credits. This is production integration evidence, not a corpus-percentage gain.

### Production typed pair relations (2026-10-15)

Legacy variable-pair relational conditions now resolve global operands through
the same invocation owner. The existing numeric-pair evaluator retains operation
semantics; atomic conversion and controlled source-to-numeric-buffer construction
are shared between ordinary execution and completed-global reads. Each operand
is owned before the next can recursively initialize. No resolver borrow crosses
that recursion, no additional completed-value cache is created, and source
selection is not reduced to a first-node scalar.

Seven tests cover fourteen ordinary-versus-constructor combinations across all
four operators, empty/all-NaN selections, later-node witnesses and boolean/node-
set conversion in both operand orders. Further evidence pins recursive right-
side construction with message-once behavior, local shadowing versus actual
cycles, supplied parameter suppression of defaults, typed infinity/NaN/decimal
values versus untyped numeric strings, and explicit refusal of extended date
operands. Exact/one-less and cancellation checks preserve completed inputs while
preventing dependent publication across numeric buffers, string-value traversal
and pair-operation work. Eight concurrent sources remain isolated. Effective
whitespace views determine numeric source values, and the completed semantic
result survives release of source and stylesheet owners.

This does not admit conditional pair `!=`, secondary-document operands, modern
comparison contexts or the separately guarded scalar forms of numeric-literal
conditions. The private relational test child groups this evidence alongside
pair equality without expanding the flat runtime-context directory.

Validation passes with **1,929 core tests**, 63 ignored probes, native 22,
WASM adapter 4 and worker 18; full repository verification and the official WASM
target. Both full OASIS replays retain **2,501 modern / 2,498 strict**, 14/17
mismatches and 423 expected-error credits. No upstream fixture, public boundary,
initialization policy or performance claim changes in this tranche.

## Production comparison output and specialization fallback (2026-10-15)

Shared constructors now admit boolean/comparison `value-of` output through the
already-connected ordinary demand readers. This includes effective booleans,
boolean/string/numeric literal comparisons and typed variable-pair equality and
inequality. Inequality remains existential and is not implemented as negated
equality. No second evaluator or completed-global store is introduced.

Eight focused tests cover ordinary-execution parity, mixed/empty node sets,
empty result trees, typed refusal, local shadowing, cycles, supplied parameter
precedence, concurrent isolation, and exact/one-less budgets plus cancellation
before publication across five charged domains. Compiler tests found that
initializer specializations treated whole expressions beginning with `$` as
bare names, or rejected non-path selections before shared compilation. Those
optimizations now decline ineligible selections; shared compilation still
rejects malformed references. Bare-variable specialization remains intact.

The full verification passes with **1,937 core tests**, 63 ignored probes,
native 22, WASM adapter 4 and worker 18; the official WASM target also checks.
Both complete OASIS profiles remain **2,501 modern / 2,498 strict**, with 14/17
mismatches and 423 expected-error credits. No fixture or coverage credit changed.
Broader instruction readers, secondary-document comparisons, unproved scalar
kinds and module-graph admission remain guarded; this is not AR completion.

## Production string consumers, source iteration and constant attributes (2026-10-15)

Three further bounded admission slices reuse existing execution, without adding
another evaluator or changing the global initialization schedule:

- String normalization, character length, substring/before/after/starts-with and
  literal/variable `contains` use the connected legacy string reader. Eight tests
  cover ordinary parity, Unicode, first-node conversion, empty/typed/fragment
  values, recursive right-operand initialization, once-only messages, cycles,
  shadowing, parameter overrides, concurrent isolation, effective whitespace,
  result lifetime and exact/one-less controls across four charged domains.
- Unsorted principal-source location-path `for-each` uses the ordinary selector
  and frame/focus construction. Seven tests cover empty/nested selections,
  normalized duplicate paths, position/size, fresh root entry for recursive
  globals, lexical COW isolation, concurrent sources, effective whitespace,
  result lifetime and partial-result cleanup. Result-node/text-byte, traversal
  and instruction budgets are tested at exact and one-less limits with real
  charge-point cancellation. Empty iteration does not invent a dependency cycle.
- Literal result elements admit constant text-valued attributes. Three tests
  cover ordinary output parity, qualified names, Unicode, escaped braces,
  result lifetime and result-node/text-body/temporary-XDM charge boundaries.
  These preserve the existing accounting units; text-byte charges are not
  advertised as total attribute-memory accounting.

The tests assert shared-constructor admission where a static initializer could
otherwise bypass the path under test. Dynamic/computed attributes, attribute
sets, sorted or variable-selected iteration, secondary documents, numeric
conversion and broader expression consumers remain guarded. The ordinary
compiler still rejects the tested string-length multiplication form; it is not
silently enabled by runtime eligibility. Tests are grouped in private named
children of `global_constructor_tests`; the runtime owner is unchanged and no
flat runtime-context directory growth is needed (ADR-0004).

Validation: full repository gates and official WASM check; **1,955 core tests**,
63 ignored probes, native 22, WASM adapter 4 and worker 18. Complete OASIS
profiles retain **2,501 modern / 2,498 strict**, 14/17 mismatches and 423
expected-error credits. No new corpus credit, fixture edit, public surface or
performance claim accompanies these integration slices.
The release-mode `shared_constructor_` selection also passes all **76 tests**;
this is correctness validation, not a timing measurement.

## Production variable-valued literal attributes (2026-10-15)

Exact `{$variable}` literal-result attributes now use the already-connected
ordinary legacy string reader during shared-constructor execution. This changes
eligibility, not conversion or ownership: no second evaluator, completed store,
map borrow across recursion or new materialization path is introduced.

Seven additional tests prove ordinary-versus-constructor parity for typed
atomics, first/empty source-node selections and temporary fragments; result
lifetime; recursive forward reads and once-only messages; caller-local isolation;
local shadowing and cycles; concurrent supplied-parameter overrides; and effective
whitespace-view strings. Diagnostic spans distinguish an attribute read failure
from a terminating dependency's original message. Repeated failure reads do not
replay messages or publish a partial constructor. Exact/one-less result-node,
temporary-XDM, string-traversal and XPath-operation charges plus deterministic
cancellation retain ordinary control semantics.

Focus-based and computed attributes remain runtime-guarded, attribute sets remain
outside constructor admission, and composed forms are not implied by this slice.
The tests remain in the cohesive private literal-attribute child; no runtime
owner or cross-layer dependency changes (ADR-0004 / ADR-0024).

Full verification passes with **1,962 core tests**, 63 ignored probes, native 22,
WASM adapter 4 and worker 18, plus the official WASM target. Full OASIS profiles
remain **2,501 modern / 2,498 strict**, 14/17 mismatches and 423 expected-error
credits. No fixture, public contract, corpus credit or performance claim changes.

## Production focus-based literal attributes (2026-10-15)

Shared constructors now admit literal-attribute projections of position, size,
lexical name, local name, context string value and already-admitted source
attributes. These execute through the existing result builder, with no new
focus owner, global reader or initializer path. A recursively initialized global
starts at its own principal effective root/singleton, not the reading loop's
focus. Computed attributes, attribute sets and other AVT forms remain guarded.
The existing compiler refusal of qualified `{@p:a}` is preserved rather than
being treated as implicit permission to broaden attribute syntax.

Five focused tests compare ordinary and constructor output, qualified element
names versus local names, missing attributes, Unicode payloads, iteration focus,
root focus inside a recursive dependency, once-only evaluation, whitespace views,
concurrent source isolation and results outliving compiled/source owners. Exact
and one-less result-node, temporary-XDM, traversal, string-value and instruction
budgets plus charge-point cancellation prevent partial publication. Tests live
in a named private attribute-focus child; the guard remains eligibility only.

Full verification passes with **1,967 core tests**, 63 ignored probes, native 22,
WASM adapter 4 and worker 18, plus the official WASM target. Complete OASIS
profiles retain **2,501 modern / 2,498 strict**, 14/17 mismatches and 423
expected-error credits. No new fixture, public surface or performance claim.

## Production comments and static-target processing instructions (2026-10-15)

The shared constructor guard now admits comments and static-target PIs,
including legacy bodies executed through the ordinary sequence executor.
Nested instructions retain their own admission checks; dynamic PI targets and
unproved body operations remain unsupported. No alternate constructor evaluator
or public surface is introduced.

Five focused tests establish ordinary/global output parity (including Unicode,
comment double/trailing hyphens and PI terminator recovery), result serialization
after compiled/source owners are dropped, forward dependency evaluation once,
comment/PI exclusion from document string value, direct circularity without
publication or restart, and explicit unsupported target/body cleanup.
Result-node, result-text-byte, XDM-node and instruction domains each pass exact,
one-less and active cancellation checks before global publication.

Full verification passes with **1,972 core tests**, 63 ignored probes, native 22,
WASM adapter 4 and worker 18 tests. The official WASM target check passes.
Complete OASIS replay remains **2,501 modern / 2,498 strict exact**, with
14 / 17 mismatches and 423 expected-error credits respectively. This is shared
constructor integration evidence, not additional corpus credit. The manual
`variable40` reference replay now explicitly requires its existing `workbench`
comparator feature so default-feature test compilation does not refer to an
absent module.

## Production principal-source copying (2026-10-15)

Current-node, child-element and location-path copy instructions now pass the
shared constructor guard and use the ordinary effective-document copier.
No resolver reader, secondary resource access or alternative copy engine is
introduced. Variable/document/ancestor-specialized copies remain guarded;
existing unsupported tests now use those still-unadmitted forms.

Four focused tests compare ordinary and global construction across document,
element, text, comment, PI and attribute selections, missing selections and
namespace fixup. They also cover iteration focus, strip/preserve views and
inherited `xml:space`, nested global root focus with one-time dependency messages,
and serialization after source/compiled owners are dropped. Result-node,
result-text-byte, XDM-node, XPath-node-visit and instruction budgets pass exact,
one-less and active cancellation checks without partial publication.

All 97 shared-constructor tests pass. Full verification passes with **1,976 core
tests**, 63 ignored probes, native 22, WASM adapter 4 and worker 18 tests; the
official WASM target builds. Complete OASIS remains **2,501 modern / 2,498 strict
exact**, 14 / 17 mismatches and 423 expected-error credits. No new corpus credit
is claimed. The replay reports no `FXRT1004` execution frontier, so further guard
expansion should follow a demonstrated compiled consumer or independently
classified compilation blocker rather than assuming every admission earns cases.

Completed bounded evidence is not the same as production completion:

- [x] Safe demand owner, shared-executor constructor reference and bounded
  failure/control/lifetime experiments, including original declaration linking.
- [x] Unchanged `variable40` reference replay and conserved full OASIS profiles
  at the preceding semantic checkpoint.
- [x] Same-owner constructor scheduling cost and same-program ordinary
  source-node global production control; repeated runs expose reference overhead.
- [x] Repository and official WASM validation of the current reference.
- [x] Implement and compare a bounded static-admission subset through the shared
  executor, including dynamic-case refusal rather than invented dependency cycles.

Remaining decision/integration gates:

- [ ] Pin legacy and modern global-context/dependency obligations independently.
- [x] Review compilation/runtime responsibility pressure and name one private
  owner for evaluation state; no broad mutable executor context as an extraction.
- [x] Select ownership and initial eager initialization in ADR-0024, accepted
  with the static-analysis and specialized-path modifications. Broader static
  reachability remains optional deferred analysis.
- [ ] Prove root focus, position/size, built-in/matched/named templates, modes,
  lexical shadowing and absence of an inherited current matched rule.
- [ ] Prove forward dependencies through templates, conditional dependencies,
  direct/indirect cycles, namespace-qualified names and include/import precedence.
- [ ] Prove supplied parameter overrides, repeated-read reuse, value ownership,
  concurrent invocation isolation and generation lifetime behavior.
- [ ] Charge evaluation stack/work and result-to-temporary retention; test exact
  and one-less limits, cancellation, failure provenance and no partial publication.
- [ ] Finish global declaration/read-edge provenance and bounded cycle-context
  integration: test-only declaration metadata, captured failure context and
  selected read sites are represented and evidenced above; remaining readers
  and the production diagnostic attachment contract are still unresolved.
- [x] Replay unchanged `variable40` through production and both full OASIS
  profiles, plus the pinned modern regression tests; no archived output edits.
- [ ] Classify related constructor cases independently before extending the
  admitted value, instruction, match or module surface.
- [ ] Validate the chosen production integration against these cost controls,
  broader global value kinds and host-visible regression gates; the reference
  measurements do not complete that obligation.

## Reopening triggers

Expand scope only when an executable case requires another global value kind,
resource capability or execution strategy. Public facade, cross-document ranking,
DTD acquisition, parallel global evaluation and host scheduling remain separate.

## Review history

- 2026-10-14 -- Maintainer accepted with modifications; numbered ADR-0024.
  Static analysis cannot define dependency/circularity semantics; specialized
  initializers must prove parity and fall back to shared execution. Production
  integration is authorized but not completed; exact corpus counts unchanged.

- 2026-10-10 -- Opened after ordinary OASIS frontier inspection identified the
  completed-globals/shared-executor seam. No engine implementation or policy
  selected; current exact accounting conserved.
- 2026-10-10 -- Began the safe test-only demand reference; seven focused tests
  prove recursive borrowing cleanup and shared-executor callback feasibility.
  Production global initialization and motivating-case support remain unchanged.
- 2026-10-10 -- Connected a test-only legacy string-read bridge; fourteen further
  tests prove shared-executor dependency/context, module/name, control and lifetime
  behavior. Modern expression admission and production global integration remain
  separate gates; no corpus numerator change is claimed.
- 2026-10-10 -- Added the owned tree/source value reference and six lifetime,
  view, copy-boundary and control tests. Outstanding values survive recursive
  evaluation without a resolver borrow; foreign-focus copying remains explicitly
  unsupported at its existing consumer boundary.
- 2026-10-10 -- Compared eager and demand schedules through the same evaluator
  in five tests. Unused failure and work-budget differences are explicit;
  supplied defaults and untaken mode cycles remain controlled. Static admission
  analysis and typed production reads are not claimed complete.
- 2026-10-10 -- Connected typed temporary copies to the same test-only demand
  resolver; six tests pin recursive structure, mixed-read cycles, shadowing,
  lifetime, work parity and consumer-versus-constructor failure ownership.
- 2026-10-11 -- Added four failure/message observation tests. Exact originating
  instruction spans and control fields survive recursive reads; failure reuse
  does not replay messages. Recorded the missing global declaration and related-
  location representation as an explicit integration obligation.
- 2026-10-11 -- Added five bounded secondary-context tests. Explicit metadata
  preserves ordered declaration/read locations; secondary limits and cancellation
  mark omission rather than replacing the primary failure. Compiler metadata and
  resolver attachment remain separate integration work.
- 2026-10-12 -- Added three actual compiler-provenance tests and one capture-once
  test. Test-only binding locations survive qualification, ordering, module merge
  and owner release; per-failure context cannot grow or relabel on repeated reads.
  Recursive read-edge propagation and resolver-cell attachment remain pending.
- 2026-10-12 -- Attached capture-once context to the generic reference's failed
  cells. Four tests prove reuse, isolation, truncation and release without a
  second value map; typed shared-executor active paths now propagate in an opt-in
  reference, while complete cycle paths and string read locations remain pending.
