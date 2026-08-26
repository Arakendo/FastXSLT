# M1 Standards Evidence and Private Vertical Slice

| Field | Value |
| --- | --- |
| Status | In Progress |
| Opened | 2026-08-25 |
| Last updated | 2026-08-25 |
| Owner | FastXSLT maintainers |
| Target | M1 standards decision and first private transform |
| Related ADRs | ADR-0001, ADR-0002 |
| Related reviews | AR-0001, AR-0003, AR-0004, AR-0007, AR-0008, AR-0009, AR-0011 |
| Related change requests | None |
| Depends on | Pinned W3C suites and complete case metadata; consumer evidence remains parallel product-fit input |

## Purpose

Produce the evidence needed to decide FastXSLT's initial standards profile and
then implement the smallest private end-to-end transform without stabilizing a
public API, parser representation, diagnostic catalog, or streaming claim.

## Trigger and evidence

The workspace and W3C suite submodules are ready, but the engine performs no
transforms. AR-0001 blocks version and conformance claims until candidate
profiles, suites, and consumer needs are explicit. The seed `hello` golden case
provides a narrow syntax intersection suitable for architecture work after its
semantics are assigned to an accepted profile.

## Goals

- Make candidate-suite size, revision, licensing, and harness pressure
  reproducible.
- Close AR-0001 through a deliberate standards-profile ADR.
- Execute the seed transform through private XML, XDM, XPath, XSLT,
  compilation, runtime, resource, result, and diagnostic ownership.

## Non-goals

- A stable public transform API or ABI.
- An unqualified XSLT/XPath conformance percentage.
- Broad QT3 or XSLT30 execution before dependency-aware selection.
- Streaming implementation, persistent compiled artifacts, or ASP.NET binding.

## Ownership and dependency boundary

Suite inventory and selection are test-harness concerns. They may describe
standards metadata but do not own engine semantics. AR-0001 and its resulting
ADR own the admitted profile. Engine XML mechanics remain replaceable; XDM,
XPath, and XSLT meaning remain FastXSLT-owned.

## Slice 0: Reproducible suite baseline

**Status:** Complete.

- [x] Pin the same QT3 and XSLT30 revisions used by the TS XSLT peer.
- [x] Preserve licensing and acquisition provenance.
- [x] Verify initialized, clean submodule worktrees and exact revisions.
- [x] Walk both catalogs without DTD or external-resource resolution.
- [x] Retain structural counts and limitations as evidence.

Exit state: 31,821 QT3 and 14,600 XSLT30 cases are reproducibly discoverable;
none are represented as supported or executable.

## Slice 1: Standards profile disposition

**Status:** In Progress.

- [ ] Obtain representative transform families and compatibility needs from an
  intended consumer before application-fit, host-default, or performance
  claims; this does not block standards-driven preview selection.
- [x] Complete suite/harness evidence for the XSLT 1.0 alternative without
  admitting redistribution-constrained legacy material to the repository.
- [x] Compare staged-modern and version-specific alternatives against time to
  first useful release, data-model growth, diagnostics, and migration risk.
- [x] Define dependency-aware selection and reporting categories.
- [x] Prove one overlay-selected XSLT30 case can retain upstream case,
  environment, stylesheet, assertion, and revision identity without copying
  fixture content.
- [x] Inventory dependency, environment, stylesheet, assertion, and combined
  metadata shapes across all 14,600 pinned XSLT30 cases.
- [x] Retain the complete six-case XSLT30 `template` test set as the first
  conserved preview denominator, including unsupported outcomes.
- [x] Close AR-0001 through accepted ADR-0007.

Exit state: source code, documentation, and test selection can name one initial
profile without ambiguity.

## Slice 2: Private golden vertical behavior

**Status:** Complete as a private experiment; ADR-0007 now supplies its modern
reference direction while public API and broad conformance remain unstabilized.

- [x] Select a leading XML parser for private evaluation without delegating engine
  semantics.
- [x] Admit source and stylesheet bytes through a test-only bounded resource
  builder, release import handles, and seal an immutable snapshot; retain the
  public contract as unresolved.
- [x] Compile one root template and one path/value expression.
- [x] Execute `corpus/golden/hello` through batch-capable private machinery.
- [x] Compare the semantic result separately from serialization.
- [x] Add invalid, unsupported, missing-resource, and transform-set budget cases
  with private structured diagnostics.
- [x] Add denied-authority and serialization-output budget cases.
- [x] Add cancellation and non-output runtime budget cases through a real
  host/control boundary.
- [x] Record navigation capabilities actually required by the slice for AR-0007.

Exit state: one exact golden result passes through named semantic owners with no
ambient I/O or public stability claim.

## Validation matrix

| Concern | Evidence | Required result |
| --- | --- | --- |
| Suite integrity | `check-conformance-sources.ps1` | Exact clean revisions |
| Catalog structure | `inventory-conformance-sources.ps1` | Retained counts; no missing/duplicates |
| Standards ownership | AR-0001 and resulting ADR | One named initial profile |
| Golden behavior | `corpus/golden/hello` | Exact classified result |
| Resource authority | Missing/denied and handle-release cases | No ambient fallback or retained handle |
| Diagnostics | Negative slice cases | Invalid and unsupported remain distinct |
| Repository gates | `scripts/verify.ps1` | Pass |

## Risks and mitigations

| Risk | Impact | Mitigation or evidence |
| --- | --- | --- |
| Catalog size is mistaken for implementation scope | Unbounded first milestone | Select dependencies only after AR-0001 |
| Private prototype leaks into public API | Premature compatibility burden | Keep facade empty until vertical evidence is reviewed |
| Harness defines semantics | Incorrect product ownership | Standards and FastXSLT tests remain authoritative over harness convenience |
| Tree representation spreads beyond XDM | Streaming/backend rewrite pressure | Record real navigation needs under AR-0007 |

## Progress log

### 2026-08-25

- Work completed: pushed scaffold checkpoint `f5e6064`; admitted and inventoried
  the pinned W3C suites.
- Validation: exact revisions, clean worktrees, 428/234 catalog references, no
  missing or duplicate sets, and 31,821/14,600 discovered cases.
- Findings: suite availability is adequate for modern-profile pressure but does
  not decide AR-0001 or cover the XSLT 1.0 alternative.
- Next slice: gather consumer transform families and complete candidate-profile
  suite evidence.

### 2026-08-25: Legacy-suite review and private-slice opening

- Work completed: reviewed the OASIS XSLT/XPath 1.0 Committee Draft 04 archive
  and recorded its catalog, doubts, acquisition, platform, and licensing facts.
- Findings: the archival suite is useful local evidence but unsuitable for
  automatic vendoring; current evidence favors a staged modern model.
- Plan change: opened Slice 2 only for private version-neutral ownership and
  resource-boundary work. Public standards semantics remain blocked.
- Next slice: implement bounded in-memory admission and immutable sealing for
  the existing golden source and stylesheet, including duplicate/size/budget
  failures and file-handle release evidence.

### 2026-08-25: Private bounded-resource experiment

- Work completed: added a test-only resource builder and immutable snapshot
  with opaque identity and explicit entry, per-entry-byte, and aggregate-byte
  limits.
- Validation: duplicate and empty identities, entry and byte limits, aggregate
  limits, equal bytes under distinct identities, and golden source/stylesheet
  rename/removal immediately after import.
- Findings: owned memory and handle release work without selecting public type
  names, URI interpretation, cache semantics, or batch behavior.
- Next slice: select the replaceable XML parser boundary for the private golden
  transform and record the exact semantic operations it must supply.

### 2026-08-25: Private XML parser experiment

- Work completed: opened AR-0008, compared three Rust parser candidates, and
  pinned `quick-xml` 0.40.1 only as a development dependency for the leading
  private experiment.
- Validation: in-memory parsing, expanded namespaces, resource identity and byte
  spans, malformed structure, duplicate expanded attributes, DTD and unknown
  entity denial, comment/PI retention pressure, and event/depth limits.
- Findings: pull events fit the owned-XDM seam, but FastXSLT must own document and
  namespace validation, external-authority policy, diagnostics, and limits.
  XML name/declaration/encoding conformance and production dependency admission
  remain open.
- Next slice: construct the first owned XDM document from the private adapter
  without retaining parser events or dependency-owned nodes.

### 2026-08-25: First owned XDM document

- Work completed: extended the private parser adapter with owned semantic events
  and consumed them into a separate engine-owned document arena.
- Validation: the golden input remains queryable after its input allocation is
  dropped; node identity, parent/child and attribute relationships, semantic
  order, string value, text coalescing, expanded names, and provenance pass
  focused tests.
- Findings: no parser event, dependency node, path, resolver, or source lifetime
  needs to enter the XDM model. The first navigation inventory is now recorded
  in AR-0007 without introducing a provider abstraction.
- Next slice: parse the golden stylesheet into the owned model and privately
  recognize only the syntax needed for its root template and value expression.

### 2026-08-25: Private golden transform completed

- Work completed: compiled the golden stylesheet once, parsed and evaluated its
  relative child path, executed two identified requests in reverse submission
  order, built a semantic result, and serialized it separately to the expected
  XML.
- Validation: 24 focused tests cover the full private path, batch-of-one parity,
  stable result correlation, invalid versus unsupported compile failures,
  missing versus denied resources, duplicate request/result identities, request
  and output budgets, and sibling-result invisibility.
- Findings: the layer boundaries support the first exact transform without
  leaking parser or XDM representation into compiled semantics. Parse per
  invocation remains AR-0009's reference behavior. The syntax intersection does
  not decide AR-0001 or establish conformance.
- Next slice: obtain representative consumer transforms to close the
  standards-profile decision; defer cancellation and broader runtime budgets
  until a real host/control boundary can exercise them.

### 2026-08-25: First pinned XSLT30 case executed

- Work completed: added a first-party overlay selecting XSLT30 `template-006`,
  loaded its environment, stylesheet, and XML assertion from the immutable
  pinned suite, and executed it through the existing in-memory transform-set
  path.
- Validation: 27 focused tests pass. The result is compared as XML, so the
  suite's `<o/>` assertion correctly matches the serializer's declaration plus
  expanded empty-element form.
- Findings: absence of `xsl:output` must remain semantically distinct from an
  explicit XML method so runtime method inference can be correct. This case has
  an `XSLT20+` dependency and is useful suite-linked evidence, not a profile or
  conformance claim.
- Next slice: define general dependency/classification reporting only after
  representative consumer transforms supply the denominator for AR-0001.

### 2026-08-25: Private invocation control and charge points

- Work completed: added an invocation-local atomic cancellation token and
  independent XML-event, XDM-node, XDM string-value, XPath node-visit, XSLT
  instruction, and serialized-byte work counters.
- Validation: 30 tests pass. Cancellation is distinct from budget exhaustion;
  every implemented work domain can be exhausted with preserved request/domain
  identity; the separate output-size limit retains its own failure shape.
- Findings: XPath charges candidate nodes inspected rather than expressions
  entered. Cancellation is observed only at charge points and cannot interrupt
  one dependency call already in progress.
- Next slice: measure/check maximum observation gaps and accounting overhead
  before proposing work-unit composition or defaults. Phase-specific
  cancellation, deadlines, panic containment, and process isolation remain
  AR-0010 follow-up.

### 2026-08-25: Explicit prepared-input reuse

- Work completed: explicitly prepared selected resource identities into sealed
  immutable XDM documents tied to their originating snapshot generation.
- Validation: 36 tests pass. Two stylesheets reuse one prepared document; one
  stylesheet executes over two separately prepared equal-byte identities; the
  prepared path matches parse-per-invocation semantics and serialization.
- Findings: equal content cannot merge logical document allocation or
  provenance. Preparation has its own cancellation and XML/XDM work budgets.
  The golden source is 87 bytes, constructs six nodes, and reports 1,932 bytes
  of owned representation capacity under the current build. Eight threads share
  the same document/program allocations with isolated invocation controls.
- Next slice: measure XDM retained/peak memory, preparation time, and concurrent
  preparation/contended reuse before choosing eager, lazy, transform-set,
  eviction, or public handle policy in AR-0009.

### 2026-08-25: Phase-specific cancellation fault injection

- Work completed: added a deterministic test-only fault that signals the host
  token at a selected real charge point after optional earlier phase work.
- Validation: 38 tests pass. XML, XDM, XSLT instruction, XPath visit, XDM
  string-value, and serialization cancellation retain request/domain identity.
  A sibling may complete first, but the private reference operation exposes no
  partial result set when a later request cancels.
- Findings: cancellation behavior at an observed charge point is deterministic;
  wall-clock signal latency and work inside one dependency call remain
  unbounded. Public batch failure collection is still undecided.
- Next slice: account for result construction and add adversarial growth cases
  before measuring observation gaps and hot-path accounting overhead.

### 2026-08-25: Result-construction accounting

- Work completed: extracted private serialization at the 38-test checkpoint,
  then added independent semantic result-node and retained UTF-8 text-byte work
  domains before serialization.
- Validation: 39 tests pass. Every one of eight domains exhausts independently
  and supports phase-specific cancellation. An exact four-byte budget admits a
  single `🚀` result text node and rejects the next byte without consulting the
  separately exhausted serialization budget.
- Findings: semantic result retention and serialized output are distinct bounded
  responsibilities. Dynamic string-value construction still creates a
  temporary string before the result meter, so its peak is not yet bounded by
  this change.
- Cohesion: serialization moved from the 988-line composition unit into a
  private 139-line child responsibility before semantic changes. The runtime
  composition unit is 885 lines after this slice; no public types or calls were
  added.
- Next slice: bound or avoid the temporary dynamic string-value allocation, then
  measure observation gaps and accounting overhead before selecting defaults.

### 2026-08-25: Direct bounded string-value construction

- Work completed: added an XDM-owned controlled fragment sink and changed
  `xsl:value-of` execution to append borrowed fragments directly through the
  semantic result meter.
- Validation: 40 tests pass. A nested `one`, `two`, `three` fixture preserves
  fragment order and concatenates to the same convenience string value. Golden,
  prepared-input, work-limit, and phase-cancellation behavior remain unchanged.
- Findings: execution no longer allocates an aggregate dynamic string value and
  then copies it into the result. Source fragments still reside in the fully
  materialized owned XDM; this is neither source streaming nor a generalized
  provider contract.
- Next slice: measure charge-point observation gaps and accounting overhead,
  then add diagnostic/message bounds when those facilities become real.

### 2026-08-25: Charge profile, gaps, and local cost

- Work completed: exposed private consumed-unit observations, asserted the exact
  golden charge profile across all eight domains, inventoried the maximum gap in
  each current semantic unit, and added an ignored release-mode cost probe.
- Validation: 41 ordinary tests pass; one ignored measurement test passed in
  three manual optimized runs. The golden path consumes 10 XML events, 6 XDM nodes, 4
  instructions, 4 XPath visits, 2 string-value nodes, 2 result nodes, 16 result
  text bytes, and 35 serialized bytes.
- Measurement: three seven-sample runs observed 1.249, 1.241, and 1.215 ns median
  per successful charge versus 0.205, 0.205, and 0.207 ns for their black-box
  baselines on the recorded Windows/Rust environment.
- Findings: semantic-unit gaps are attributable, but dependency work and append
  chunks prevent a wall-clock cancellation guarantee. The microprobe does not
  establish complete-transform or ASP.NET overhead.
- Next slice: add a representative end-to-end accounting comparison only after
  broader consumer transforms exist; meanwhile continue adversarial structural
  limits that do not depend on the unresolved standards profile.

### 2026-08-25: Roadmap reconciliation after corpus-ledger evidence

- Work completed: private suite-specific records now retain QT3 `assert-eq` and
  a second XSLT30 compound/message assertion family; unknown metadata fails
  visibly and synthetic filtered/sharded/interrupted/retried reports conserve
  both denominators independent of merge order.
- Findings: AR-0011 has enough evidence to preserve its ledger seam but its
  remaining report format, CI, and comparison-family work is downstream of
  executable semantics. The roadmap understated completed handle-release,
  batch, reusable-compilation, navigation, and suite-admission evidence.
- Plan change: keep AR-0001 as the M1 exit gate. First inventory representative
  transform families and compatibility needs, then accept a standards-profile
  ADR, implement the next standards-directed slice, measure reuse, and exercise
  that lifecycle through ASP.NET.
- Next slice: inspect the TS XSLT peer for candidate transform families and
  testing shapes. Treat them as peer-derived questions until an intended
  consumer confirms which families and workload envelopes are representative.

### 2026-08-25: Peer-family inventory and exact template dispatch

- Work completed: inventoried the modified local TS XSLT peer at `9c48142` and
  retained its goldens, 73-case curated XSLT30 subset, .NET resolver fixtures,
  and large S1000D graph only as candidate pressure. Extended the private
  reference backend with exact unprefixed element-name templates and explicit
  `xsl:apply-templates` child-path selection.
- Validation: the new `template-dispatch` golden selects two source items and
  invokes one compiled rule in source order. Duplicate patterns and modes fail
  visibly rather than acquiring accidental priority semantics. The workspace
  has 47 passing tests and one ignored manual probe.
- Findings: compiled rule state remains stylesheet-derived while selected nodes
  and control remain invocation-local. Peer recurrence justifies the private
  experiment but does not satisfy AR-0001's first-consumer requirement.
- Next slice: identify a native XSLT30 case whose full dependency, environment,
  and assertion shape fits exact-name dispatch, while requesting representative
  stylesheets and workload envelopes from the intended consumer.

### 2026-08-25: Built-in rules and honest suite-fit boundary

- Work completed: screened syntax-light XSLT30 apply-template stylesheets,
  retained the closest candidate's `id()`/DTD and attribute-path dependencies,
  and admitted no misleading overlay case. Added default child application,
  built-in document/element/text behavior, and context-item selection to the
  same private reference backend.
- Validation: the independently authored `built-in-template-rules` golden
  transforms two invoice items through an unmatched document element and one
  exact item rule. The workspace has 49 passing tests and one ignored probe.
- Findings: standards suites appropriately couple small instructions to deeper
  semantic dependencies. FastXSLT should widen semantics from consumer and
  architectural evidence, then select native cases whose full dependency shape
  fits—not reverse-engineer scope from the desire for another green case.
- Next slice: obtain intended-consumer transforms/workload envelopes. If those
  confirm this family, select the next required pattern/XPath feature and its
  native cases; otherwise redirect before accepting AR-0001.

### 2026-08-25: Prepared-input lifecycle measurements

- Work completed: retained a release-mode direct-versus-prepared timing probe,
  separated raw-byte/XDM retention observations, and established the current
  explicit-construction concurrency and retry baseline.
- Validation: the tiny built-in-rule source measured 3.23–3.62 times slower for
  parse/XDM construction per invocation than prepared reuse in three local
  runs. The 87-byte hello source reports 938 bytes of parsed-phase capacity, 6
  XDM nodes, and 1,932 bytes of XDM capacity; a generated 2,109-byte source
  reports 46,862 parsed-phase bytes, 202 nodes, and 63,755 XDM bytes.
  Independent concurrent builders produce distinct documents, while cancelled
  and budget-exhausted attempts permit clean retries.
- Measurement: three separate-phase release runs over the 55-byte source
  observed 1,123.9–1,133.3 ns XML-parse medians and 863.4–921.5 ns XDM
  construct-and-drop medians. Loop/allocation differences prevent adding these
  into an exact decomposition of the complete direct-path measurement.
- Allocation measurement: three identical optimized runs observed 2,744
  retained and 3,424 peak allocator-requested bytes while preparing hello. The
  generated 100-item source retained 64,577 and peaked at 130,357 bytes. The
  exact-pinned optional tool is current-thread-only and excludes allocator
  metadata, snapshot admission, process memory, and host overhead. Its explicit
  feature remains disabled for timing probes.
- Relationship-shape measurement: three local runs over eight distinct source
  identities and eight separately compiled stylesheet identities observed
  3.19–3.33 times prepared benefit for the multi-source shape and 3.22–3.43
  times for the multi-stylesheet shape. All bytes remain equal and the workload
  remains tiny, warm, single-threaded, and consumer-unconfirmed.
- Findings: the seam has measurable private value and meaningful retention
  cost. The current builder performs explicit owned construction, so it has no
  shared first-access, single-flight, or waiter semantics to stabilize.
- Next slice: obtain consumer workload envelopes before choosing lifecycle
  defaults; evaluate single-flight only if duplicate construction is observed
  under a representative workload.

### 2026-08-25: Host-owned two-stage workflow

- Work completed: added a reviewed two-stage golden and executed each stage as
  a separate batch of one over a separately sealed snapshot.
- Validation: stage one produces an identified `<message>` result. An earlier
  stage-two snapshot still rejects that identity with `FXRS0001`; after the host
  explicitly admits the result bytes under that identity into a new snapshot,
  stage two produces the expected `<stage-two>` result.
- Findings: result identity supplies correlation, not authority. Production
  does not mutate a snapshot, publish a file, or promote a sibling result. The
  host owns stage order, selected-result retention/copy, admission, and sealing.
- Next slice: retain this behavior when a real host adapter exists. Do not add a
  graph, implicit promotion, or zero-copy public representation without
  consumer and performance evidence.

### 2026-08-25: Private compiled-semantic inspection

- Work completed: projected the implemented compiled stylesheet into an owned,
  read-only report containing logical identity, declared version, output
  semantics, template/instruction counts, and implemented feature counts.
- Validation: the hello stylesheet produces the exact expected projection;
  inspection leaves the compiled program equal, the report survives program
  drop, and text/feature-kind limits fail structurally without partial output.
- Findings: semantic inspection can answer useful static questions without
  exposing source text, matches, paths, locations, parser/XDM nodes,
  instruction bodies, IR, addresses, or caches. The current caller supplies the
  stylesheet identity because compiled ownership is not yet public.
- Next slice: obtain ASP.NET and conformance-harness questions before choosing
  public fields, redaction, compatibility/versioning, serialization, dynamic
  summaries, or tracing.

### 2026-08-25: Private XPath boundary expansion

- Work completed: expanded the private relative child-path parser and evaluator
  tests across supported ASCII NCName punctuation, invalid syntax,
  valid-but-unimplemented syntax, context selection, repeated children,
  namespace exclusion, empty selection, document order, and logical failure
  provenance.
- Correction: `item.name` no longer fails merely because a supported NCName
  contains a dot. Non-ASCII names remain conservatively unsupported until the
  accepted profile selects editions and name rules; the ASCII-only helper does
  not label them malformed.
- Findings: the implemented evaluator selects unnamespaced expanded names in
  document order and returns an empty sequence when no child matches. This
  remains boundary evidence, not a general XPath parser or version claim.
- Next slice: obtain intended-consumer transforms before selecting another
  expression family. Keep general tokenization, Unicode name classification,
  axes, predicates, functions, operators, and sequences standards-directed.

### 2026-08-25: W3C-driven preview replanning and metadata inventory

- Plan change: representative consumer transforms no longer block a testable
  standards-driven preview. They remain necessary for product priority,
  compatibility, ASP.NET lifecycle, and representative performance evidence.
- Work completed: reproducibly inventoried all 14,600 pinned XSLT30 cases,
  including 9,663 stylesheet references, 7,646 distinct referenced stylesheet
  files, 22 dependency kinds, 15 top-level assertion kinds, three environment
  binding shapes, and 564 combined metadata shapes.
- Findings: the suite provides enough complete inputs to drive implementation
  now, but stylesheet filenames or syntax-only screening cannot define an
  honest denominator. Selection must retain dependency, environment, assertion,
  revision, and engine outcome.
- Next slice: define a first coherent preview overlay around complete upstream
  cases and `assert-xml` comparison, using actual compiler/executor outcomes to
  distinguish applicable, unsupported, and harness-gap cases.

### 2026-08-26: First conserved XSLT30 preview denominator

- Work completed: expanded the first-party overlay from one green template case
  to all six cases in the pinned XSLT30 `template` test set. The harness retains
  every case's standards dependency, environment, stylesheet, and `assert-xml`
  shape and imports each stylesheet through a bounded snapshot.
- Validation: `template-006` compiles and executes; `template-001` through
  `template-004` fail compilation as unsupported attribute/semantic shapes, and
  `template-005` fails as unsupported named-template behavior. Valid named
  templates are no longer misclassified as invalid missing-match input.
- Conservation: six discovered cases equal one selected/pass plus five
  engine-unsupported/not-run cases. No filename or completion outcome controls
  membership.
- Next slice: propose the staged standards-profile ADR around this selection
  and ADR-0006, explicitly denying any broad XSLT version or conformance claim.

### 2026-08-26: First post-profile semantic widening

- Work completed: implemented built-in document dispatch, comment-node
  selection and pattern matching, and unprefixed mode isolation through native
  XSLT30 `template-001`.
- Conservation: the six-case denominator advanced to two selected passes and
  four explicit engine-unsupported cases without changing membership.
- Next slice: extend the same node-kind selection seam under
  `template-002/003` processing-instruction and `node()` pressure.

### 2026-08-26: Processing-instruction and general node tests

- Work completed: executed native XSLT30 `template-002/003` through
  processing-instruction and general child-node selection and pattern matching.
- Boundary correction: normalized the syntactic PI target/data separator at the
  XML adapter and kept the root document node outside the child `node()` pattern.
- Conservation: the six-case denominator now contains four passes and two
  explicit engine-unsupported cases.
- Next slice: implement attribute-axis selection and attribute patterns under
  `template-004` without treating attributes as children.

### 2026-08-26: Attribute selection and patterns

- Work completed: executed native XSLT30 `template-004` through unprefixed
  attribute-axis selection, exact attribute matching, and named-mode isolation.
- XDM boundary: attributes remain separately owned and are not exposed through
  the child axis.
- Conservation: the six-case denominator now contains five passes and one
  explicit engine-unsupported case.
- Next slice: decompose `template-005` into named-template lookup, parameters,
  conditional evaluation, calls, and bounded recursion before implementation.

### 2026-08-26: Complete XSLT30 template denominator

- Work completed: executed native XSLT30 `template-005` through statically
  resolved named templates, invocation-local string parameters, variable access,
  integer equality conditionals, calls, and recursion.
- Limit evidence: an independent infinite recursive call returns structured
  limit failure at the private depth boundary; calls also consume the existing
  XSLT instruction work budget.
- Conservation: all six cases in the complete `template` denominator are now
  selected and passing, with zero exclusions or hidden unsupported outcomes.
- Next slice: select the next complete XSLT30 family together with the QT3
  expression cases needed to implement it honestly.

### 2026-08-26: XSLT30 path denominator opened

- Work completed: admitted all ten cases in the complete pinned XSLT30 `path`
  test set and the complete four-case QT3 `Axes002` named-child-axis pressure
  group before broadening implementation.
- Execution: native `path-001` now passes through a narrow final
  `child::name` existence predicate over a relative child path.
- Work accounting: both candidate children and predicate children examined are
  charged to the invocation's XPath node-visit domain.
- Conservation: the XSLT30 denominator contains one pass and nine explicit
  engine-unsupported outcomes; QT3 retains four explicit unsupported outcomes.
- Next slice: map the file-backed `path-002` environment into sealed resources
  and add only the ancestor/descendant XPath behavior its native case requires.

### 2026-08-26: XSLT30 `path-002` file-backed execution

- Work completed: resolved the native file-backed principal source, imported
  its bytes into a bounded sealed snapshot, and executed the unmodified
  `path-002` stylesheet without engine-owned filesystem access.
- XPath behavior: added leading descendant navigation and a final named
  ancestor existence predicate, preserving document order.
- Work accounting: descendant candidates and examined ancestors consume the
  invocation's XPath node-visit budget.
- Conservation: the complete `path` denominator now contains two passes and
  eight explicit engine-unsupported outcomes.
- Next slice: execute `path-003` by extending the same predicate seam to the
  narrowly required ancestor-or-self behavior.

### 2026-08-26: XSLT30 `path-003` ancestor-or-self execution

- Work completed: executed the unmodified native `path-003` stylesheet through
  the same bounded, sealed file-backed environment path used by `path-002`.
- XPath behavior: represented ancestor-or-self distinctly, testing the
  candidate before walking its parent chain and charging every inspection.
- Verification: an independent self-match test prevents ancestor-or-self from
  being implemented as an alias for ancestor-only behavior.
- Conservation: the complete `path` denominator now contains three passes and
  seven explicit engine-unsupported outcomes.
- Cohesion: the corpus integration test now shares one file-backed path-case
  executor instead of duplicating catalog resolution, resource admission, and
  transform construction.
- Next slice: execute `path-004` through an explicit named attribute existence
  predicate without treating attributes as children.

### 2026-08-26: XSLT30 `path-004` attribute predicate

- Work completed: executed the unmodified native `path-004` stylesheet and its
  file-backed environment through the shared bounded snapshot harness.
- XPath/XDM behavior: added a named attribute existence predicate over the
  candidate's separately owned attribute collection; attributes remain absent
  from the child axis.
- Work accounting: every inspected attribute consumes the invocation's XPath
  node-visit budget, with an exact small-case charge assertion.
- Conservation: the complete `path` denominator now contains four passes and
  six explicit engine-unsupported outcomes.
- Next slice: execute `path-005` through the narrowly required
  descendant-or-self existence predicate.

### 2026-08-26: XSLT30 `path-005` descendant-or-self predicate

- Work completed: executed the unmodified native `path-005` stylesheet and its
  file-backed environment through the bounded snapshot harness.
- XPath behavior: added a self-first, document-order descendant-or-self named
  existence predicate.
- Work accounting: every self or descendant inspection consumes the XPath
  node-visit budget; a focused test fixes the small-case charge total.
- Conservation: the complete `path` denominator now contains five passes and
  five explicit engine-unsupported outcomes.
- Next slice: execute `path-006` through the narrowly required named parent
  existence predicate.

### 2026-08-26: XSLT30 `path-006` parent predicate

- Work completed: executed the unmodified native `path-006` stylesheet and its
  file-backed environment through the bounded snapshot harness.
- XPath/XDM behavior: added a named parent existence predicate over the XDM
  parent link; it inspects only the immediate parent.
- Work accounting: each present parent inspection consumes one XPath node visit,
  and a focused case fixes the full evaluation at eight visits.
- Conservation: the complete `path` denominator now contains six passes and
  four explicit engine-unsupported outcomes.
- Next slice: decompose `path-007`'s arithmetic positional predicate into the
  minimum grammar and numeric semantics needed by the native case before
  implementation.

### 2026-08-26: XSLT30 `path-007` constant arithmetic position

- Work completed: executed the unmodified native `path-007` stylesheet and its
  inline environment through a resource-admission helper shared with the
  file-backed path cases.
- XPath behavior: added checked constant-integer parsing with parentheses,
  multiplicative/additive precedence, exact `div`, and non-negative `mod`, then
  applied the result as a position over name-matched step nodes.
- Claim control: fractional division, zero division, overflow, functions, and
  other unimplemented numeric semantics remain explicit failures rather than
  host-language approximations.
- Cohesion: numeric expression parsing is isolated in a private XPath source
  unit instead of expanding the navigation evaluator's responsibility.
- Conservation: the complete `path` denominator now contains seven passes and
  three explicit engine-unsupported outcomes.
- Next slice: pressure the numeric seam with `floor()` in native `path-008`
  without implying general XPath function support.

### 2026-08-26: XSLT30 `path-008/009` integer floor

- Work completed: executed both unmodified native floor cases through their
  inline catalog environment and the shared bounded snapshot harness.
- XPath behavior: admitted `floor()` only inside the checked constant-integer
  parser, where it is the identity operation; nested arithmetic and direct
  positional uses both select position two.
- Claim control: decimal/double rounding, fractional division, other functions,
  and general function resolution remain unsupported.
- Conservation: the complete `path` denominator now contains nine passes and
  one explicit engine-unsupported outcome.
- Next slice: decompose `path-010`'s intermediate positional predicate, final
  `last()` predicate, and complex match pattern before implementing the final
  denominator case.

### 2026-08-26: Complete XSLT30 path denominator

- Work completed: executed native `path-010` with an intermediate arithmetic
  position, final `last()`, and the same relative path as a template match
  pattern.
- XPath behavior: positional predicates now belong to individual path steps and
  operate over each step's name-matched sequence.
- XSLT behavior: a private relative path pattern walks the candidate lineage to
  the first-step context, then reuses the charged path evaluator and node
  identity for matching.
- Conservation: all ten cases in the complete `path` denominator are selected
  and passing, with zero exclusions or hidden unsupported outcomes.
- Claim control: general XSLT pattern grammar and priority, general XPath
  predicates/functions, and standards conformance remain outside this evidence.
- Next slice: execute the paired QT3 `Axes002-1` through `Axes002-4` group by
  adding native environment resolution, explicit named child-axis steps,
  `fn:count`, and `assert-eq` comparison without routing it through XSLT.

### 2026-08-26: Native QT3 `Axes002` execution

- Work completed: executed all four selected `Axes002` cases from the pinned
  QT3 `AxisStep` test set through native environment, source, expression, and
  `assert-eq` resolution.
- Resource boundary: each upstream source is imported into a bounded sealed
  snapshot and parsed from retained bytes after its import handle closes.
- XPath behavior: leading descendant navigation and explicit named `child::`
  steps share one path representation; a narrow `fn:count` seam reports the
  selected sequence size without routing the case through XSLT.
- Work accounting: every examined navigation node consumes the invocation's
  XPath node-visit budget.
- Conservation: the complete group reports four selected passes with expected
  values `0`, `0`, `1`, and `2`; the adjacent `Axes001-1` wildcard case remains
  explicitly unsupported.
- Claim control: this is direct evidence for one expression and assertion
  family, not a general function library or broad XPath/QT3 conformance claim.
- Next slice: admit all four cases in XSLT30 `tests/expr/for/_for-test-set.xml`
  before implementation, then let their native metadata expose the first
  harness and semantic gaps without shrinking the denominator.

### 2026-08-26: XSLT30 `expr/for` denominator admitted

- Work completed: admitted all four cases in the complete pinned `expr/for`
  test set, resolving every XSLT20+ dependency, stylesheet, environment,
  principal source, initial-template declaration, and XML assertion.
- Resource boundary: four stylesheets and three case-specific sources are
  imported under qualified identities into one bounded sealed snapshot after
  their file handles close; the file-backed `for-001` assertion is also
  acquired and checked.
- Conservation: the denominator reports zero passes, three
  engine-unsupported cases, and one harness-unsupported case. `for-002` retains
  its source-free initial-template entry instead of receiving a synthetic
  principal source.
- Diagnostic correction: valid expression syntax outside the private
  child-path grammar now reports unsupported rather than invalid, while truly
  malformed ASCII child names retain the invalid category.
- Pressure map: `for-001` requires ordered node/atomic sequences,
  `distinct-values`, binding, comparisons, path filtering, and `xsl:sequence`;
  the later cases add initial-template entry, multiple clauses, focus-sensitive
  evaluation, numeric aggregation, decimal behavior, and formatting.
- Next slice: decompose and implement only the first `for-001` semantic layer
  that can preserve sequence order and node identity; do not shrink the
  four-case denominator or imply general XPath sequence support.

### 2026-08-26: Native XSLT30 `for-001` ordered sequence

- Work completed: compiled and executed unmodified native `for-001`, including
  `xsl:sequence`, ordered `distinct-values`, one `for` binding, value-based
  predicates, first-item selection, and related title selection.
- Ownership: the compiled expression owns only stylesheet-derived variable,
  path, and name structure. Bound values and selected source `NodeId` values
  remain invocation-local until selected elements are copied into the semantic
  result.
- Result boundary: the private copy seam handles the unnamespaced element/text
  subtrees required by the case and rejects unsupported attributes or node
  kinds instead of dropping them.
- Work accounting: path navigation, XDM string atomization, XSLT instruction
  execution, result construction, and serialization use their existing
  separate charge domains.
- Conservation: the complete `expr/for` denominator advances to one pass, two
  engine-unsupported cases, and one harness-unsupported case. The upstream
  file-backed XML assertion matches exactly.
- Claim control: arbitrary FLWOR grammar, general sequences/functions,
  collations, namespaces, atomic sequence results, and generalized node copying
  remain outside this evidence.
- Next slice: enable source-free initial-template entry for native `for-002`,
  then implement only its multiple integer bindings, addition, ordered atomic
  return sequence, and `xsl:value-of` separator behavior.

### 2026-08-26: Source-free native XSLT30 `for-002`

- Work completed: executed native `for-002` by adding explicit principal-source
  and initial-template invocation-entry variants. The case invokes `main`
  without a fabricated source document or context item.
- Ownership: the named template remains immutable compiled state; the selected
  entry name is request-local. Unknown names fail request admission with
  structured request identity, and initial-template parameters remain outside
  the private seam.
- XPath behavior: two literal integer bindings iterate in clause order, their
  addition return produces `11`, `12`, `21`, `22`, and the native
  `xsl:value-of separator=", "` constructs the expected text.
- Work accounting: atomic expression work now has a distinct
  `xpath-operation` domain. Four additions consume four units, and a focused
  three-unit limit stops before the fourth result without borrowing the node
  navigation meter.
- Conservation: the complete `expr/for` denominator advances to two passes and
  two engine-unsupported cases; no harness-unsupported case remains.
- Claim control: arbitrary initial-template parameters/dynamic context,
  generalized FLWOR clauses, numeric types, overflow, operators, and atomic
  sequence conversion remain unsupported.
- Next slice: execute native `for-003` while preserving XPath focus across the
  binding, so its unqualified attribute paths remain relative to `order` and
  its empty multiplication results make `sum()` return zero.

### 2026-08-26: Native XSLT30 `for-003` focus preservation

- Work completed: executed native `for-003` through its `for` binding,
  unqualified attribute multiplication, empty return sequences, and the zero
  result of `sum(())`.
- Focus rule: binding each `order-item` to `$i` does not replace the matched
  `order` context item. A focused case with attributes only on bound children
  fixes this behavior independently of the upstream assertion.
- Claim control: when both multiplication operands exist on the outer focus,
  evaluation fails as unsupported rather than introducing partial numeric
  semantics under the empty-sequence case.
- Work accounting: tuple iterations and final aggregation consume
  `xpath-operation`; navigation and attribute inspection retain
  `xpath-node-visit` charges.
- Cohesion: the shared principal-source corpus executor now owns repeated
  environment/source/stylesheet/assertion resolution for `for-001` and
  `for-003`, reducing duplicated harness responsibility.
- Conservation: the complete denominator advances to three passes and one
  engine-unsupported case.
- Next slice: execute native `for-004` using `$i/@price` and `$i/@qty`, exact
  decimal multiplication/aggregation, and the narrow two-decimal
  `format-number` picture required by its assertion.
