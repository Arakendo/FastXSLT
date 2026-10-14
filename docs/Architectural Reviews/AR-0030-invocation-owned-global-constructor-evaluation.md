# AR-0030: Invocation Owned Global Constructor Evaluation

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-10-10 |
| Last reviewed | 2026-10-13 |
| Scope | Global initialization, shared sequence execution and dependency resolution |
| Trigger | OASIS `Lotus/variable_variable40#1` requires template dispatch while constructing a global value |
| Related ADRs | ADR-0004, ADR-0007, ADR-0012, ADR-0014, ADR-0017, ADR-0022, ADR-0023 |
| Related evidence | [Current OASIS frontier](../Evidence/oasis-xslt10-xml-indentation-frontier-2026-10-10.md) |

## Architectural question

How should invocation-owned global values be resolved when their constructors
execute the shared instruction/template machinery and discover dependencies
through dispatched templates, without a second interpreter or source-dependent
compiled state?

## Trigger and evidence

The unchanged `Lotus/variable_variable40#1` constructs `$TreeFrag` with
`xsl:apply-templates select="//b"`. It currently fails compilation with
`FXST1015`. Its matched templates include recursive application and text
construction; the principal transformation later reads the completed value.
No new resource authority is required by this case.

Current initialization runs `materialize_global_defaults` before constructing
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

No alternative is selected. The first unanswered questions are whether static
admission can cover a useful shared slice, and whether a safe demand reference can
integrate with existing lexical frames without cloning the entire invocation or
holding recursive borrows. Evaluation strategy also affects unused failures and
message observations; compare permitted behavior by edition rather than assume
eager and demand execution are interchangeable in every observable circumstance.

## Disposition

Under Review. Keep the production unsupported boundary until a bounded experiment
establishes context, dependencies and cleanup. Do not promote a resolver or public
global-value representation solely to gain the motivating archival case.

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

## Required follow-up

- [ ] Pin legacy and modern global-context/dependency obligations independently.
- [ ] Review compilation/runtime responsibility pressure and name one private
  owner for evaluation state; no broad mutable executor context as an extraction.
- [ ] Compare static admission with a safe invocation demand reference, both
  invoking the existing sequence/template executor.
- [ ] Prove root focus, position/size, built-in/matched/named templates, modes,
  lexical shadowing and absence of an inherited current matched rule.
- [ ] Prove forward dependencies through templates, conditional dependencies,
  direct/indirect cycles, namespace-qualified names and include/import precedence.
- [ ] Prove supplied parameter overrides, repeated-read reuse, value ownership,
  concurrent invocation isolation and generation lifetime behavior.
- [ ] Charge evaluation stack/work and result-to-temporary retention; test exact
  and one-less limits, cancellation, failure provenance and no partial publication.
- [ ] Preserve global declaration/read-edge provenance and define bounded cycle
  context; current instruction failures are conserved, but global declaration
  locations and structured related locations are not yet represented.
- [ ] Replay unchanged `variable40` and related cases independently, then both
  full OASIS profiles and representative modern cases; no archived output edits.
- [ ] Run full repository and official WASM gates; measure ordinary-global and
  constructor-heavy regression cost before recommending an ADR.

## Reopening triggers

Expand scope only when an executable case requires another global value kind,
resource capability or execution strategy. Public facade, cross-document ranking,
DTD acquisition, parallel global evaluation and host scheduling remain separate.

## Review history

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
