# FastXSLT Roadmap

The roadmap is ordered by executable semantic evidence, not by a desire to fill
every conceptual module. Dates are intentionally absent until requirements and
standards scope are decided.

## Current critical path

ADR-0004's first mandatory 2,000-line review has fired. The retained
[runtime and compiler decomposition review](../Evidence/runtime-and-compiler-decomposition-review-2026-08-27.md)
moves general runtime contract tests out as a preparatory navigation checkpoint
and then separates private transform-set admission/result correlation from the
invocation engine. A second semantic extraction gives dynamic `xsl:value-of`
and its XPath adapters a one-way private owner. The runtime composition owner
fell from 2,431 to 1,310 lines; its 304-line transform-set and 267-line value
children call, but do not own, the remaining invocation semantics.
The stylesheet compiler is now divided between a 1,019-line top-level assembly
and validation owner and a 775-line private instruction compiler. The remaining
runtime must be reassessed before another semantic family grows template
dispatch, temporary-tree, or sequence-evaluation responsibilities; the compiler
units retain the reopening triggers recorded by the review.

FastXSLT has accepted its staged-modern semantic direction and passes the
complete XSLT30 `template`, `path`, and `expr/for` test-set denominators. It also
executes the complete four-case QT3 `Axes002` group through a
stylesheet-independent XPath seam and the complete two-case XSLT30
`fn/deep-equal` denominator through attribute/comment node comparison.
The complete five-case QT3 `fn-deep-equalint2args` group also executes through
source-free checked `xs:int` comparison, and the adjacent five-case
`fn-deep-equalintg2args` group extends that evidence to the checked `i128`
subset of arbitrary-precision `xs:integer`. The complete five-case
`fn-deep-equaldec2args` group adds exact normalized decimal comparison within a
checked `i128` coefficient boundary, without using binary floating point. The
complete five-case `fn-deep-equallng2args` group now also passes through a
range-checked signed 64-bit `xs:long` constructor. The adjacent five-case
`fn-deep-equalusht2args` group adds a range-checked `xs:unsignedShort`
constructor with explicit lower- and upper-bound rejection controls.
`for-004` closes its family through
bound-variable attribute paths, checked exact-decimal multiplication and
aggregation, and the single required two-decimal formatting picture. The
complete 28-case `expr/data-manipulation` denominator now passes through native
execution. The current order of work is:

1. obtain representative consumer transforms, input/result distributions,
   concurrency, deployment targets, trust model, and latency/throughput budgets;
2. execute those workloads through both candidates and add dedicated cold-load,
   native-retention, transport-attribution, and sustained-load evidence;
3. decide whether the measured low-latency and containment candidates become
   supported profiles, then stabilize only their shared lifecycle plus any
   deliberately distinct guarantee surfaces.

Representative consumer transforms are not a prerequisite for a testable
standards-driven preview. The pinned W3C suites provide executable stylesheets,
sources, dependency metadata, environments, assertions, and expected errors and
can drive incremental implementation now. Consumer examples remain necessary
to prioritize optional compatibility, validate useful workload coverage, choose
host-facing lifecycle details, and make ASP.NET or application-performance
claims.

CR-0001 records Tokimu's Web3D X3D-to-VRML workflow as the first concrete
Rust-native consumer pressure and opens AR-0012 for the supported Rust facade.
The request is deferred while Tokimu likely uses Saxon in the near term, so it
does not displace standards-driven engine work or select a facade. It also does
not yet close the representative-workload item: the authoritative Web3D
invocation, pinned/licensed stylesheet and resources, parameters, trusted
sentinels/output, input distributions, trust model, and performance budgets are
still missing.

ADR-0006 now makes AR-0011's essential ledger invariants binding. Its remaining
reporting, storage, CI, and comparison-family work proceeds when executable
standards slices need it; those deferred choices are not the current release
gate. AR-0005 and AR-0010 remain seam-preservation reviews. Compatibility AR-0006
and streaming AR-0007 are deferred unless a representative case activates one
of their reopening pressures.

## M0 -- Project scaffold

- [x] Buildable Rust workspace with formatting, lint, test, and docs gates.
- [x] Documentation authority and lifecycle established.
- [x] Initial SDD, ADR process, AR process, and testing strategy established.
- [x] Golden corpus layout seeded without claiming executable support.

Exit criterion: a clean checkout can run the local verification script and the
next architectural questions are visible rather than encoded accidentally.

## M1 -- Standards decision and first vertical slice

- [ ] Record representative transform families, input/output shapes, and
  compatibility needs from the first intended consumer before claiming product
  fit or selecting host/performance defaults; this does not block the
  standards-driven preview.
- [x] Close AR-0001 through accepted ADR-0007.
- [x] Select and document the leading private XML parser boundary for the slice.
- [x] Compile one root template, evaluate one path/value expression, and produce
  one result through a private end-to-end engine path.
- [x] Run `corpus/golden/hello` with private structured failure identities.
- [x] Load the golden source and stylesheet through a bounded resource set,
  seal it, and execute the case without engine-owned filesystem access.
- [x] Release import handles before sealing, then replace or remove the original
  fixture files and prove the snapshot still executes identically.
- [x] Add negative cases that distinguish invalid input from unsupported syntax.
- [x] Establish the first private structured boundary failures and reportable semantic
  outcomes from emitted behavior rather than an aspirational error catalog,
  providing evidence for AR-0004.
- [x] Preserve native QT3/XSLT30 case identity, separate selection from
  execution disposition, reject unknown metadata visibly, and conserve report
  denominators through a private AR-0011 experiment.

Exit criterion: the seed transform passes through the intended layers and every
implemented behavior belongs to a named standards slice.

## M2 -- Data model and XPath foundation

- [ ] Define node identity, document order, names, strings, and sequence/value
  behavior needed by the accepted profile.
- [x] Record the navigation and retention capabilities the implemented XPath
  and XSLT slice actually requires; keep tree-specific random-access assumptions
  inside their physical owner, providing evidence for AR-0007.
- [ ] Expand XPath lex/parse/evaluate tests before growing XSLT instructions.
- [x] Admit all ten cases in the complete XSLT30 `path` test set and execute
  `path-001` through `path-010`, including charged axis predicates, per-step
  positions, `last()`, checked constant-integer arithmetic, integer-domain
  `floor()`, and the native complex relative match pattern.
- [x] Execute the complete QT3 `Axes002` named-child-axis group through native
  environments, direct XPath, `fn:count`, charged navigation, and `assert-eq`
  comparison without an XSLT wrapper.
- [x] Admit and execute the complete two-case XSLT30 `fn/deep-equal` denominator
  through positioned descendant attribute/comment selection and charged,
  pairwise node comparison. Preserve distinct XDM identity while comparing
  attribute expanded names and values or comment values; broader node kinds,
  sequences, typed values, and collations remain outside this slice.
- [x] Execute the complete five-case QT3 `fn-deep-equalint2args` group through
  checked `xs:int` constructors and source-free numeric value comparison,
  including both argument orders and the type's lower and upper bounds. Keep
  cross-type promotion, floating-point/NaN rules, arbitrary sequences, and the
  remainder of the 263-case QT3 function test set explicitly unclaimed.
- [x] Execute the complete five-case QT3 `fn-deep-equalintg2args` group through
  checked `i128` values, including its 18-digit lower, middle, and upper
  operands in both orders. Treat this as a bounded `xs:integer` subset rather
  than an arbitrary-precision implementation claim.
- [x] Execute the complete five-case QT3 `fn-deep-equaldec2args` group through
  normalized coefficient-and-scale values with checked `i128` coefficients.
  Preserve exact decimal equality without binary floating point and leave
  arbitrary precision, cross-type promotion, floats, doubles, and NaN outside
  this slice.
- [x] Execute the complete five-case QT3 `fn-deep-equallng2args` group through
  checked signed 64-bit values, with a focused upper-bound control that rejects
  an out-of-range constructor value. Do not infer numeric promotion or general
  constructor support from this group.
- [x] Execute the complete five-case QT3 `fn-deep-equalusht2args` group through
  checked unsigned 16-bit values. Prove the derived type boundary by accepting
  `65535` and rejecting both `-1` and `65536`, without claiming the other
  derived-integer families.
- [x] Admit all four XSLT30 `expr/for` cases with their native environments,
  stylesheets, entry metadata, XML assertions, and explicit unsupported
  dispositions before implementing sequence semantics.
- [x] Execute native `for-001` through ordered distinct-value binding,
  comparison/path selection, source-node identity preservation, and
  `xsl:sequence` result construction against its complete upstream assertion.
- [x] Execute source-free native `for-002` through an invocation-local
  initial-template entry, ordered integer bindings/addition, an independently
  bounded XPath-operation domain, and `xsl:value-of` separator semantics.
- [x] Execute native `for-003` with the outer focus preserved across its
  binding, empty-sequence multiplication, and the integer zero result of
  `sum(())`, while refusing non-empty numeric multiplication.
- [x] Execute native `for-004` with bound-variable attribute navigation,
  checked exact-decimal multiplication and aggregation, and only the required
  `'0.00'` formatting picture. The complete four-case denominator now passes.
- [x] Admit the complete nine-case XSLT30 `expr/castable` denominator: seven
  selected cases, two explicit schema-aware profile exclusions, four
  admission-time engine gaps, and three harness gaps.
- [x] Execute native `castable-001` through controlled atomization and owned
  built-in lexical castability, retaining inherited prefixed namespaces on its
  literal result. The selected denominator is one pass, three engine gaps, and
  three harness gaps.
- [x] Execute native `castable-002` through explicit built-in casts and typed
  invocation-local variables. The selected denominator is two passes, two
  engine gaps, and three harness gaps.
- [x] Execute native `castable-003` through an explicit value-aware conversion
  matrix for boolean, integer, decimal, float, and double. The selected
  denominator is three passes, one engine gap, and three harness gaps.
- [x] Execute native `castable-004` through explicit duration-family
  castability and its inline XML assertion. The selected denominator is four
  passes, no engine-classified gaps, and three harness gaps.
- [x] Resolve the source-free standard initial-template entry for
  `castable-007` through `castable-009` to a namespace-canonical compiled
  identity, inventory both compound assertion predicates, and classify all
  three at their actual `xsl:function` engine boundary. The selected denominator
  is four passes, three engine gaps, and no harness gaps.
- [x] Admit all 28 XSLT30 `expr/data-manipulation` cases with their referenced
  inline/file-backed environments and XML assertions, then execute `001`
  through `028` using ordered conditional instructions, checked exact-rational
  predicates, nonnegative `round()`, exact-decimal formatting, and
  invocation-local materialization of top-level variable/parameter text
  defaults and source-derived node sequences. The complete denominator passes;
  host parameter overrides, arbitrary global expressions, forward references,
  and general dependency ordering are not claimed.
- [ ] Establish diagnostic codes and source spans across XML and XPath phases.
- [x] Provide a read-only semantic inspection snapshot for the implemented
  compilation slice without exposing private parser, arena, or IR types,
  providing evidence for AR-0005.
- [x] Import the first licensed, versioned, integrity-checked suite selection.

Exit criterion: a published test report identifies supported, unsupported,
failed, and harness-error cases without an unqualified conformance claim.

## M3 -- Reusable stylesheet engine

- [x] Separate reusable compiled stylesheet state from dynamic transform state
  in the private reference path; its public representation remains unstabilized.
- [ ] Add template selection, built-in rules, parameters, variables, and output
  behavior required by the accepted profile.
- [x] Execute XSLT30 `template-001` through built-in document dispatch,
  comment-node selection, and an isolated named mode while retaining the other
  four unsupported cases in the six-case denominator.
- [x] Execute XSLT30 `template-002/003` through processing-instruction and
  general child-node tests, retaining mode isolation and exact-pattern
  precedence. Four of the six denominator cases now pass.
- [x] Execute XSLT30 `template-004` through attribute-axis selection and an
  exact attribute pattern without adding attributes to the child axis. Five of
  the six denominator cases now pass.
- [x] Execute XSLT30 `template-005` through statically resolved named templates,
  invocation-local parameters, conditional equality, calls, and bounded
  recursion. The complete six-case denominator now passes.
- [x] Admit the complete five-case XSLT30 `misc/initial-mode` denominator,
  preserving each mode identity and expected error or XML assertion through
  bounded snapshots. A focused host-neutral initial-mode entry executes with an
  admitted principal source and rejects unknown compiled mode identity.
- [x] Add invocation-owned atomic parameter values to the private transform
  request and use them to override matching global `xsl:param` defaults without
  mutating reusable compiled state or leaking values between sibling requests.
- [x] Execute pinned `initial-mode-004` with leading template-local parameters,
  expanded QName identity, tunnel/non-tunnel matching, inherited
  `exclude-result-prefixes`, and its ordered child-node/atomic sequence. The
  complete denominator reached one native pass and four explicit engine gaps;
  general parameter defaults/types and tunnel propagation remain open.
- [x] Execute pinned `initial-mode-003` as its expected `XTDE0050` outcome by
  preserving `xsl:output/@indent`, required global-parameter identity, and mode
  identity from matched templates. Indented serialization remains explicitly
  unsupported rather than being silently ignored. The denominator reached two
  native passes and three explicit engine gaps.
- [x] Execute pinned `initial-mode-002` as its expected `XTDE0045` outcome by
  preserving `mode="#all"` declaration metadata without treating it as a
  wildcard that makes every initial mode available. The denominator is now
  three native passes and two explicit engine gaps.
- [x] Execute pinned `initial-mode-001` through a bounded typed local integer
  sequence over `1 to 10`, preserving ten invocation-local atomic values and
  separator semantics without collapsing the sequence into a preformatted
  string. The denominator is now four native passes and one explicit engine
  gap; general `xsl:for-each` and typed sequence conversion remain open.
- [x] Complete pinned `initial-mode-005` by preserving multiple explicit mode
  names, materializing an attribute-free literal global temporary tree per
  invocation under XDM budgets, selecting `$temp/*` without conflating it with
  the principal source, and copying the selected element through the unnamed
  wildcard template. The full five-case initial-mode denominator now passes;
  general temporary-tree navigation and `xsl:copy` construction remain open.
- [ ] Establish explicit URI/resource resolution and execution limits.
- [x] Execute a private batch of independent requests with shared compiled stylesheets
  and isolated dynamic contexts; randomize scheduling, correlate results by
  identity, and prove a batch of one matches the convenience API.
- [x] Compare parse-per-invocation with private snapshot/work-generation prepared
  input reuse, reporting retained XDM and peak construction memory separately.
- [x] Measure parse, XDM construction, compilation, compiled/direct execution,
  compiled/prepared execution, compile-per-invocation execution, retained XDM,
  and preparation peak memory over native XSLT30 `for-004` and `castable-004`.
- [x] Execute a two-stage host-owned workflow and prove stage-one results remain
  invisible until explicitly admitted into a stage-two snapshot.
- [ ] Compare file-per-call, preloaded snapshot, warmed filesystem cache, and
  compile-once paths with correctness held constant.
- [ ] Add differential and integration tests against named processors.
  - [x] Establish a small ASP.NET comparison against Microsoft's built-in XSLT
    1.0 processor and a locally acquired, non-distributed SaxonCS-HE 13.0.0
    adapter, preserving exact-stylesheet versus equivalent-workload distinctions.
- [ ] Run an ASP.NET consumer workbench through the selected host boundary,
  reusing compiled stylesheets across requests with explicit cancellation and
  resource policy.
  - [x] Establish the first ASP.NET 8 persistent isolated-worker baseline with
    one-time bounded resource transfer, compile-once/prepared reuse, correlated
    results, structured failures, and one explicit in-flight slot.
  - [x] Exercise deterministic 5-, 50-, and 500-item tiers through a bounded
    four-worker pool, recording throughput, p50/p95/p99 latency, CPU,
    allocation, working-set scope, result size, and comparison-engine caveats.
  - [x] Terminate an acknowledged non-cooperating isolated request without
    poisoning a sibling, decline ambiguous retry, replace only its worker slot,
    and promote/drain explicitly identified snapshot generations.
  - [x] Import and close host files, replace them while an old generation lease
    remains active, and prove old/new requests retain their sealed source
    semantics without engine-owned filesystem access.
  - [x] Carry already-signalled cooperative cancellation into an isolated
    invocation, preserve its exact direct-path diagnostic, and reuse the same
    worker generation; natural active-signal measurement remains open.
  - [x] Route a correlated cancellation while execution is paused at a real
    charge point, ignore an unrelated identity, preserve structured failure and
    worker reuse, and keep the artificial barrier out of latency claims.
  - [x] Sample 25 unpaused 20,000-item cancellation races, conserve cancellation
    and completion outcomes, retain same-worker recovery, and distinguish local
    latency observations from deadline guarantees.
  - [x] Adapt a managed `CancellationToken` without converting cooperative
    cancellation into a hard-stop claim, and preserve a four-case direct versus
    isolated diagnostic matrix for invalid, unsupported, and cancelled work.
  - [x] Exhaust an invocation-local XSLT-instruction budget through the isolated
    host boundary, retain `FXCT0002 / limit`, decline retry or replacement, and
    prove the same compiled/prepared worker remains reusable.
  - [x] Design the proposed workbench-only native ABI safety contract, exact
    unsafe surface, panic quarantine, verification matrix, and removal criteria;
    ADR-0008 now accepts that narrow exception.
  - [x] Execute the first in-process native compile/prepare/reuse path with
    byte-exact output, structured invalid/XML diagnostics, independent-handle
    concurrency, `SafeHandle` disposal, and a three-run warm comparison.
  - [x] Exercise the same deterministic 5-, 50-, and 500-item sources through
    four independent native handles, recording tiered throughput, latency,
    managed-allocation scope, isolated working set, and the limits of whole-host
    native memory observations.
  - [x] Carry pre-dispatch cooperative cancellation and a deterministic
    XSLT-instruction budget through ADR-0009 scalar native controls, preserving
    exact diagnostics and ordinary reuse of the same engine handle without
    claiming active cancellation or hard termination.
  - [x] Fully initialize and atomically promote a changed native engine
    generation, retain old prepared semantics under a lease, drain its retired
    pool on release, and preserve the unsupported-stylesheet diagnostic fields
    asserted by direct and isolated execution.
  - [x] Signal active native cancellation after a real charge through ADR-0010
    Rust-owned control handles, ignore an unrelated handle, conserve two
    unpaused 25-trial managed-token samples, and recover the same engine without
    describing cooperative control as hard termination.
- [ ] Exercise AR-0010's private invocation controls under adversarial work;
  distinguish deterministic budgets, cooperative cancellation, best-effort
  deadlines, panic handling, and process-level hard termination claims.

Exit criterion: representative stylesheets compile once, transform multiple
documents without leaked state, fail through structured diagnostics, and expose
measured end-to-end behavior to at least one non-Rust consumer.

## Later candidates

CLI, WASM, streaming implementation or conformance, schema awareness, extension
functions, packages, alternate execution backends, transformation graphs, and
specific parallel executor strategies require their own product evidence and
architectural review. Their presence in this list is not a commitment.

### Deferred Tokimu/Web3D consumer workload

CR-0001 remains a future real-world compiler, resource, parameter, fidelity,
Rust-facade, and performance workload. Tokimu's likely near-term use of Saxon
does not make Saxon behavior normative and does not authorize FastXSLT-specific
Web3D semantics.

- [ ] Reopen CR-0001 when Tokimu needs to replace or supplement Saxon and has
  supplied the authoritative Web3D invocation to FastXSLT.
- [ ] Independently acquire and verify known-good immutable Web3D stylesheet
  revision `35289`; record its redistribution terms, complete logical resource
  graph, catalog/base-URI behavior, and required parameters. Revision `40046`
  is a known reproducible fidelity failure and must not become expected data.
- [ ] Admit only licensed representative inputs and independently trusted
  outputs or semantic sentinels. Reuse Tokimu-owned checks for translations,
  indexed topology/coordinates, texture URLs, material colours, and
  interpolator keys/values where licensing permits; keep incomplete revision-
  `40046` output out of expected corpus data.
- [ ] Inventory required standards features and compile to the first explicit
  unsupported frontier, then feed independently justified features into normal
  standards-driven slices.
- [ ] Exercise the AR-0012 Rust facade, bounded execution, structured
  diagnostics, compiled reuse, and in-memory result handling before claiming
  Tokimu compatibility.
- [ ] Benchmark cold compilation, warm execution, preparation, result transfer,
  allocation, and retained memory only after semantic fidelity passes.
