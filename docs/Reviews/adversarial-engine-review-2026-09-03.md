# FastXSLT Adversarial Engineering Review

| Field | Value |
| --- | --- |
| Date | 2026-09-03 |
| Status | Remediation in progress; Findings 1, 2, 3, 4, and 6 resolved |
| Scope | Repository state at `a9d495e`, with emphasis on changes after the resolved 2026-08-30 review |
| Method | Read-only source, decision-record, evidence, corpus-overlay, runtime, worker, native-ABI, and managed-host inspection; full all-feature workspace test execution; static counterexamples |

The earlier twelve findings remain closed. This review does not reopen one merely
because nearby code changed. The findings below are new mechanisms or newly
admitted behavior that the earlier review did not cover.

## Remediation status

| Finding | Status | Resolution or next evidence |
| --- | --- | --- |
| 1. Atomic range retention | Resolved 2026-09-03 | Range dispatch is iterator-owned, focus size is checked, and XPath control is charged before each item is dispatched. A billion-item zero-budget stylesheet fails before span-proportional retention. |
| 2. Cross-kind lexical shadowing | Resolved 2026-09-03 | Every local binding clears competing value kinds and suppresses same-name global fallback. The source-node-over-global-atomic counterexample passes through shared and complete-clone frames. |
| 3. Untaken creation outcome | Resolved 2026-09-03 | Releasing the outcome reclaims its engine and capacity; concurrent take/release has one linearized owner. |
| 4. Character-map scaling | Resolved 2026-09-03 | Release-mode measurements confirmed both adverse curves. Compilation now composes through an ordered keyed map and retains a sorted compact vector; serialization uses binary lookup. |
| 5. Test-only QT3 semantics | Open; expansion paused | Promote one complete family into the production XPath path and install a parity sentinel before adding another test-only semantic family. |
| 6. Worker command queue | Resolved 2026-09-03 | The shared event channel has capacity one and a focused backpressure test proves a second decoded event cannot be queued. |
| 7. Source-unit reopening | Open | Perform the required current-state cohesion reviews after the correctness tranche identifies stable seams. |

### Finding 1: Atomic `apply-templates` ranges allocate before control can intervene

Disposition: **Resolved 2026-09-03.** The executor no longer collects the
range. It calculates a checked focus size, iterates the inclusive range, and
charges `XPathOperation` before dispatching each atomic item. The unchanged
production compiler/runtime path now exercises a billion-item range with a zero
budget and returns the structured limit before proportional allocation.

Severity:
- High

Confidence:
- High

Area:
- Resource exhaustion, cancellation, XSLT execution

Evidence:

`parse_apply_selection` accepts any two `i64` endpoints in a lexical
`start to end` expression and lowers them to `ApplySelection::AtomicIntegerRange`
(`crates/fastxslt/src/compile/instruction_compiler/template_invocation_compiler.rs:137-151`).
Execution immediately collects the complete inclusive range into a `Vec<i64>`
before performing a work charge
(`crates/fastxslt/src/runtime/atomic_template_executor.rs:13-28`). The first
charge is reached only later, while selecting a template for the first already
materialized item. This differs from the adjacent static `for-each` and integer
range variable paths, which calculate a checked span or charge each item before
retention.

A tiny stylesheet can therefore request `1 to 1000000000`, or an endpoint near
`i64::MAX`, and force a multi-gigabyte allocation or capacity failure before
`max_xpath_operations`, `max_xslt_instructions`, cancellation, result-node
limits, or result-byte limits can stop it.

Why this may be wrong:

The range lane is private and deliberately narrow. An isolated host can impose
an operating-system memory limit and replace a failed worker. Those facts limit
the blast radius but do not make the in-process workbench's documented
cooperative limits effective at this allocation point.

Reproduction or falsification:

Compile a stylesheet containing
`<xsl:apply-templates select="1 to 1000000000"/>`, initialize an engine with
ordinary limits, and transform with an XPath-operation limit of one. Run the
probe only in a sacrificial memory-limited worker. The expected safe behavior
is a structured limit or cancellation outcome before allocation proportional
to the range. Current control flow allocates the vector first. A non-allocating
iterator implementation with checked focus size would falsify this finding.

Expected impact:

Untrusted or accidentally large stylesheets can cause extreme transient memory,
allocator abort, process termination, or a long cancellation blind spot using
an input measured in tens of bytes.

Suggested next experiment:

Add a pre-allocation one-less-budget test and a sacrificial-process RSS probe for
small, medium, and hostile spans. Then make the range iterator-owned, calculate
focus size with checked arithmetic, and charge before each item is retained or
dispatched.

Decision interaction:

This conflicts with AR-0010's bounded-observation requirement and with the
project rule that resource limits and fallback behavior be explicit and
testable. ADR-0016 correctly says hard peak containment belongs to an isolated
process, but it does not waive ordinary in-process charge points.

### Finding 2: Node-valued template parameters do not shadow same-named global atomics

Disposition: **Resolved 2026-09-03.** Runtime frames now record local binding
identity independently of value kind. Installing an atomic, atomic sequence,
source-node sequence, or temporary tree clears competing local/inherited atomic
storage and prevents fallback to a same-named global of another kind. The exact
counterexample produces `node` through both the shared COW frame and the
complete-clone oracle.

Severity:
- High

Confidence:
- High

Area:
- XSLT variable binding, correctness, invocation isolation

Evidence:

`RuntimeVariables::from_atomics` begins every template frame with the complete
global atomic map (`crates/fastxslt/src/runtime/runtime_context.rs:162-178`).
When a supplied template argument is `SourceNodes`, `bind_template_parameters`
adds the nodes to `frame.source_nodes` but does not remove a same-named entry
from `frame.atomics` (`runtime_context.rs:658-704`). Consumers such as
`append_variable_value` resolve the atomic map before the node map
(`crates/fastxslt/src/runtime/value_evaluator.rs:653-681`). Other consumers fall
back independently to global node or temporary-tree maps, so the split maps do
not represent one lexical binding identity with one active value kind.

The result is a semantic shadow leak. A local template parameter named `value`
supplied with source nodes can still read a global atomic `$value` instead.
The complete-clone oracle preserves the same split-map behavior and therefore
cannot detect this language-level error.

Why this may be wrong:

The admitted node-sequence argument lane is narrow, and the selected
`call-template-0402` corpus case does not appear to collide with a global of the
same name. If compilation intentionally rejects cross-scope name reuse, the
counterexample would be unreachable; current compilation checks duplicate
parameters within a template but no such global/local prohibition was found.

Reproduction or falsification:

Use a global `<xsl:variable name="value">global</xsl:variable>`, call a named
template with `<xsl:with-param name="value" select="doc/a"/>`, and emit
`$value` in the callee. With source `<doc><a>node</a></doc>`, XSLT lexical
scoping requires `node`; current lookup order predicts `global`. Repeat with
the complete-clone oracle enabled. A test producing `node` through both paths
would falsify the finding.

Expected impact:

Valid stylesheets can silently produce the wrong result. The bug is
data-independent, can survive differential COW testing, and affects the newly
claimed node-sequence parameter boundary.

Suggested next experiment:

Add a cross-product test in which atomic, atomic-sequence, source-node,
temporary-tree, and empty-sequence locals each shadow every same-named global
kind. Model one binding as a tagged value, or explicitly clear all other value
stores whenever a local binding is installed.

Decision interaction:

ADR-0014 proves copy-on-write isolation, not variable lookup correctness. The
SDD now promises that relative-path arguments retain source-node sequences in
the invocation-local frame; lexical shadowing must be part of that same
contract.

### Finding 3: Releasing an untaken creation outcome orphans its engine

Disposition: **Resolved 2026-09-03.** Generic release of an untaken engine
creation outcome now removes the associated engine and its known-capacity
charge. A concurrent take-versus-release test proves exactly one operation owns
the engine afterward, direct engine release is rejected while the creation
outcome is pending, and either terminal path returns both registries to
baseline.

Severity:
- Medium

Confidence:
- High

Area:
- Native ABI lifecycle, registry admission, availability

Evidence:

Successful native creation inserts an engine and an `Outcome::Engine` together
(`crates/fastxslt-dotnet-workbench/src/lib.rs:202-258`).
`State::release_outcome` removes only the outcome and subtracts its payload
charge (`lib.rs:260-275`). An engine outcome has a zero payload and the
associated engine is not removed. If a caller releases that outcome without
first calling `fastxslt_workbench_v0_outcome_take_engine`, the numeric engine
handle is never delivered but the engine and its known-capacity charge remain
in the process-wide registry.

Why this may be wrong:

The managed happy path checks the outcome kind and immediately takes the engine,
so ordinary `NativeFastXsltClient` construction does not exercise the leak.
Direct ABI callers are expected to follow the documented take operation.
However, ADR-0016 explicitly treats buggy direct callers and abandonment as
reasons for registry admission, and `outcome_release` is otherwise a valid
generic cleanup operation.

Reproduction or falsification:

Configure an engine-count limit of one, create an engine, verify an engine
creation outcome, release that outcome without taking it, then query registry
counts and create again. Current code predicts one unreachable engine and an
engine-count exhaustion status. Releasing the creation outcome and observing
both counts return to zero would falsify the finding.

Expected impact:

A cancellation, exception, foreign-wrapper bug, or deliberate direct-ABI call
can permanently consume engine-count and known-capacity quota until the process
is recycled. This converts a bounded leak into deterministic denial of service.

Suggested next experiment:

Add lifecycle tests for release-before-take, take-versus-release races, and
engine-release attempts around a pending creation outcome. Give
`Outcome::Engine` ownership of the unpublished engine until take transfers it,
so dropping either side has one unambiguous reclamation rule.

Decision interaction:

ADR-0016 requires atomic engine/outcome publication and immediate capacity
recovery after release. It does not currently spell out ownership transfer for
an abandoned creation outcome; this case should be made explicit there or in
ADR-0008's handle lifecycle.

### Finding 4: Character maps create uncharged quadratic compile and serialization work

Disposition: **Resolved 2026-09-03.** Release-mode measurements confirmed
quadratic distinct-entry composition and map-size-proportional serialization
lookup. Resolution now applies precedence through a compilation-local ordered
map, retains a compact scalar-sorted vector, and performs binary lookup during
serialization. The measurement and conservation evidence is recorded in
[Character-Map Scaling Remediation](../Evidence/character-map-scaling-remediation-2026-09-03.md).

Severity:
- Medium

Confidence:
- High

Area:
- Performance, resource accounting, serialization

Evidence:

Character-map composition stores entries in a `Vec` and merges every entry by
linearly searching the accumulated vector
(`crates/fastxslt/src/compile/golden_stylesheet_experiment.rs:391-447`). This is
quadratic for a large map and repeats across referenced/output maps. During
serialization, every input character again linearly scans the map in
`write_character_expansion`
(`crates/fastxslt/src/runtime/golden_runtime_experiment/serialization.rs:1892-1927`).

The workbench's resource-byte limit bounds stylesheet size, and the serialized
byte limit charges bytes eventually written, but neither bounds the product of
map entries and input characters. Compilation has no invocation work control;
serialization can perform thousands of comparisons for one charged output
character.

Why this may be wrong:

The one-megabyte resource ceiling limits the maximum practical map, and current
corpus character maps are small. For those workloads a vector may be faster
than hashing. A measured crossover could show that the risk is below accepted
host budgets, but no such bound or measurement was found.

Reproduction or falsification:

Generate valid stylesheets containing increasing numbers of distinct
`xsl:output-character` entries, then serialize long text containing a character
absent from the map. Plot compile and serialize time for 100, 1,000, 5,000, and
10,000 entries while holding byte output constant. Linear or explicitly capped
behavior would falsify the finding; the current algorithms predict quadratic
compile growth and `O(text length * map size)` serialization.

Expected impact:

Bounded-size stylesheets can consume disproportionate CPU, delay cancellation,
and reduce ASP.NET throughput without exhausting any configured semantic or
byte budget.

Suggested next experiment:

Measure the crossover, add a character-map entry ceiling or compilation work
budget, and use a scalar-keyed indexed representation for resolved maps. Charge
lookup work or prove it is constant-bounded.

Decision interaction:

AR-0010 requires hostile valid input to be controlled by the owning layer and
checks to occur at bounded intervals. AR-0013 permits representation changes
when a measured seam demonstrates pressure; this is a focused candidate, not a
request for a generalized cache.

### Finding 5: QT3 pass growth is implemented by test-only XPath evaluators

Severity:
- Medium

Confidence:
- High

Area:
- Conformance evidence, architecture, semantic parity

Evidence:

Several newly completed QT3 denominators call family-specific evaluators that
are compiled only under `#[cfg(test)]`: case conversion, duration components,
and string length are declared test-only in
`crates/fastxslt/src/xpath/mod.rs:3-5,26-28,74-76`; most of the URI evaluators
are likewise individually test-gated in
`xpath/escape_html_uri_experiment.rs`. The private ledger records these cases
as `selected/passed`, while the workbench compiler/runtime does not route the
same general expressions through those evaluators. For example, production
instruction compilation recognizes only the special `upper-case(.)` shape,
whereas the QT3 case-conversion evaluator owns a separate parser and value
model.

The separate implementations already have observable boundary divergence.
The test-only duration parser uses one permissive parser for
`xs:yearMonthDuration`, `xs:duration`, and `xs:dayTimeDuration`, ignores the
remainder after the date `D` component, and accepts an otherwise empty `P`
lexical (`xpath/duration_component_experiment.rs:218-294`). Consequently
`xs:yearMonthDuration("P1D")`, `xs:dayTimeDuration("P1Y")`, and
`xs:duration("P1Dgarbage")` are accepted by that semantic slice even though
they are not valid values of the named constructor.

Why this may be wrong:

The evidence documents repeatedly and correctly call these private semantic
slices, disclaim public APIs and broad conformance, and in the duration case
explicitly disclaim complete lexical validation. AR-0011 allows a harness to
consume honestly private engine boundaries. If maintainers define these
test-gated modules as staged engine code that must later be promoted unchanged,
the claim wording may be considered sufficiently narrow.

Reproduction or falsification:

For each ledger pass, run the unchanged expression through the same compiled
XPath/XSLT path exposed to the workbench, and require identical typed values,
errors, work charges, and Unicode/version behavior. Separately add constructor
negative controls for subtype contamination, empty duration lexicals, trailing
garbage, and malformed fractional seconds. Full parity would falsify the
finding.

Expected impact:

Pass counts can grow faster than reusable engine capability, and fixes made in
one mini-evaluator need not repair the runtime path. Future denominator changes
can become green through harness semantics that no embedding can execute.

Suggested next experiment:

Choose one of the newly complete families and promote its parser/evaluator into
the single runtime semantic path. Make the QT3 adapter supply metadata and
comparisons only. Record a parity sentinel that fails if any selected case uses
a test-only evaluation function.

Decision interaction:

ADR-0006 and AR-0011 say the harness must not duplicate XPath/XSLT behavior to
decide that the engine passed, and the general architecture forbids a second
execution backend without a parity strategy. The current disclaimers reduce
claim severity but do not provide that parity strategy.

### Finding 6: The isolated worker has an unbounded eager command queue

Disposition: **Resolved 2026-09-03.** The reader, supervisor, and completion
producers now share a capacity-one synchronous event channel. One decoded event
may wait while the reader holds at most the command it is trying to submit;
further framing is backpressured. A focused queue test proves the second event
is rejected by nonblocking admission while the first occupies the slot.

Severity:
- Medium

Confidence:
- High

Area:
- Worker transport, resource exhaustion, supervision

Evidence:

The worker connects its stdin reader and supervisor with
`std::sync::mpsc::channel()`, which is unbounded
(`crates/fastxslt-worker/src/main.rs:31-41`). The reader thread continuously
parses complete commands and allocates their bounded byte fields before sending
them (`main.rs:269-289,364-385`). The supervisor consumes one event at a time;
it can be occupied by synchronous transformation, a first-charge spin barrier,
or the deliberate non-cooperating probe. Per-frame one-megabyte bounds therefore
do not bound aggregate queued bytes.

Why this may be wrong:

The current managed host appears to serialize protocol use and owns a bounded
worker pool. The worker is unpublished, and hard-isolation deployments are
expected to apply process memory limits. A trusted host that never pipelines
commands will not trigger the queue.

Reproduction or falsification:

Start one non-cooperating or long-running operation, then write many valid
maximum-sized initialization frames without reading responses. Observe worker
RSS and queued allocations. A bounded/synchronous channel that stops reading
when its single command slot is occupied would falsify the finding.

Expected impact:

A buggy or adversarial parent can consume worker memory without violating any
individual frame bound, potentially reaching the operating-system kill path for
transport behavior rather than transformation work.

Suggested next experiment:

Replace the channel with an explicitly sized `sync_channel`, or keep framing on
the supervisor thread and read only when admission is available. Add a flood
probe that proves bounded RSS/backpressure while an invocation is active.

Decision interaction:

AR-0010 explicitly assigns bounded admission, queueing, and in-flight policy to
the dispatcher/supervisor layer. This finding does not weaken the decision that
only process isolation can terminate non-cooperating execution; it identifies
an avoidable transport allocation outside that execution.

### Finding 7: Mandatory source-unit reopening triggers have fired again

Severity:
- Medium

Confidence:
- High

Area:
- Architecture drift, reviewability, ownership

Evidence:

At this checkpoint, the hand-maintained units are approximately 2,958 lines for
`compile/golden_stylesheet_experiment.rs`, 2,819 lines for
`runtime/golden_runtime_experiment.rs`, 2,013 lines for the semantic serializer,
and 1,818 lines for `compile/instruction_compiler.rs`. ADR-0004 requires an
explicit retained decomposition review from 2,001 lines. More specifically,
the 2026-08-27 runtime/compiler review required reopening the runtime and
top-level compiler at 1,200 lines and the instruction compiler at 1,000 lines.
All three named triggers have been exceeded substantially.

The UTF-16 evidence records that `serialization.rs` was reduced from the 2,000
trigger to 1,973 lines, but subsequent feature additions have already carried it
back over the mandatory threshold. Focused extraction records exist for several
children, but no later retained review covering the current parent sizes and
responsibility mix was found.

Why this may be wrong:

Line count is only a trigger, not a defect, and existing decomposition records
may be intended as continuing authority. Some of the physical size is cohesive
tests or narrow bounded recognizers. A current retained decision, if it exists
outside the searched documentation, would reduce this to a navigation concern.

Reproduction or falsification:

Run the repository's physical-line inventory, then locate a post-trigger review
that inventories current responsibilities and explicitly retains or extracts
each unit. Such records would falsify the governance part. Independent edit
histories for template dispatch, temporary trees, output semantics, expression
parsing, and corpus-shaped special cases would continue to test cohesion.

Expected impact:

Semantic changes become harder to review locally, charge and shadowing gaps are
easier to miss, and supposedly separate feature campaigns repeatedly modify the
same composition owners. This is defect-amplifying architecture debt rather
than a direct standards failure.

Suggested next experiment:

Perform the required current-state review before the next semantic expansion.
The strongest demonstrated seams are atomic-sequence dispatch, variable binding
and lookup, result-method serializers, character-map compilation, and
test/corpus adapters. Extract only seams whose dependency direction remains
one-way.

Decision interaction:

ADR-0004 sets the exact triggers and says responsibility boundaries, not line
buckets, justify decomposition. The prior retained review's explicit reopening
criteria are already stronger evidence than a generic style preference.

## Top 5 correctness or security risks

1. Eager atomic-range collection can terminate a process before any configured
   control observes the work (Finding 1).
2. Split variable stores violate lexical shadowing and can silently return the
   wrong value kind (Finding 2).
3. Releasing an untaken native creation outcome can permanently exhaust engine
   admission capacity (Finding 3).
4. An unbounded worker command queue turns bounded frames into unbounded
   aggregate memory (Finding 6).
5. Test-only XPath implementations can validate behavior that no embedding path
   executes (Finding 5).

## Top 5 performance risks

1. Full `Vec<i64>` materialization for atomic `apply-templates` ranges.
2. Quadratic character-map composition and linear scan per serialized character.
3. Repeated full-string normalization intermediates for CDATA and some
   serialization paths before the final budgeted append.
4. Full matched-template candidate scans for each dispatched item; these are now
   charged and bounded, but remain a dominant scaling term where no index applies.
5. Namespace-scope vector cloning and linear prefix lookup per result element
   under namespace-heavy stylesheets; this is a measurement target, not a
   confirmed regression.

## Top 5 places where architecture is stronger than evidence

1. One lexical binding abstraction is needed before separate atomic/node/tree
   maps can safely claim general variable semantics.
2. QT3 should execute through the same production XPath semantic path used by
   XSLT and hosts, with adapters restricted to suite metadata and comparison.
3. Atomic sequence execution should be iterator-owned and budgeted before
   retention, following the already safer static `for-each` pattern.
4. Resolved character maps need an indexed representation or an explicit small
   cardinality contract backed by measurement.
5. Worker framing needs protocol-level admission/backpressure, not only
   per-field lengths and external process containment.

## Likely false positives

- No regression was found in the earlier document-order, namespace-copy,
  whitespace-boundary, template-candidate charging, rooted-match cache, or COW
  isolation fixes. Finding 2 is a semantic lookup problem that both COW and the
  clone oracle share; it is not evidence that COW isolation itself failed.
- Filesystem reads found in the engine crate remain test/import mechanics; no
  new ambient execution-time filesystem or network acquisition path was found.
- The native unsafe surface remains limited to the reviewed export/copy boundary;
  this review found a safe-Rust lifecycle issue, not a new memory-unsafe block.
- The QT3 evidence documents use unusually careful boundary language. Finding 5
  challenges architectural parity and the meaning of `passed`, not the stated
  absence of a broad public conformance claim.
- The large source units may still be cohesive enough to retain. Finding 7 is
  definite about fired review triggers, not a predetermined demand to split by
  line count.

## Missing evidence at review time

- A sacrificial-process range test proving control precedes allocation.
- Cross-kind local/global variable shadowing, including the complete-clone oracle.
- Release-before-take and take-versus-release native creation lifecycle tests.
- Character-map compile/serialize scaling and cancellation-observation curves.
- A bounded worker-input flood/backpressure test.
- QT3-to-workbench parity for one complete newly admitted function family.
- Current retained ADR-0004 reviews for the compiler, runtime, instruction
  compiler, and serializer parents.

## Original recommended experiment and remediation order

1. Stop eager atomic-range allocation and add the hostile-span sacrificial test.
2. Repair cross-kind binding shadowing and add the value-kind cross-product.
3. Define and test ownership transfer for native creation outcomes.
4. Bound the worker command queue.
5. Measure and index or cap character maps.
6. Route one complete QT3 family through the production semantic path.
7. Reopen the fired ADR-0004 source-unit reviews and extract only the seams
   demonstrated by the fixes above.

## Validation performed

- `cargo test --workspace --all-features`: passed; 594 engine tests passed,
  15 ignored, 16 native tests passed with 2 ignored, and 3 worker tests passed.
- Typed QT3 denominator and ledger code was inspected against ADR-0006 and
  AR-0011.
- Current code was traced through compilation, runtime parameter binding,
  serialization, worker framing, native registry ownership, and managed
  creation handling.
- At the initial read-only review checkpoint, no engine or host implementation
  code had yet been changed.

The validation list above records the original read-only review. Remediation
validation is recorded separately and does not preserve the original
“implementation unchanged” state as a claim about the current repository.
