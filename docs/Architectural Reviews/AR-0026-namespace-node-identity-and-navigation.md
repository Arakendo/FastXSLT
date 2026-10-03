# AR-0026: Namespace Node Identity and Navigation

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-10-01 |
| Last reviewed | 2026-10-01 |
| Scope | Engine-owned namespace nodes, source navigation, focus, and result copying |
| Trigger | Fifteen standard-operation OASIS CD04 cases first fail on namespace-node selection or identity |
| Related ADRs | ADR-0004, ADR-0007, ADR-0012, ADR-0017, ADR-0018, ADR-0021 |
| Related reviews | AR-0007, AR-0009, AR-0013, AR-0019 |

## Architectural question

Can the shared XDM and XPath engine expose genuine namespace nodes with stable
element-relative identity, bounded navigation, and correct focus/copy behavior,
without treating declarations as attributes or changing immutable prepared
source ownership?

## Trigger and evidence

The current hash-verified OASIS CD04 sweep remains 2,471 / 3,173 exact matches
(77.88%). Namespace selection is the first failure in these standard cases:

- Lotus `axes59`, `axes62`, `axes68`, `axes120`, and `copy51`;
- Lotus `namespace28`, `namespace32`, `namespace33`, `namespace34`, and
  `namespace142`;
- Lotus `node17`, `position76`, and `position111`;
- Microsoft `Namespace_CheckXmlnsResetOnResultTree` and
  `XSLTFunctions_GenereateIdAppliedToNamespaceNodesOnDifferentElements`.

These abbreviated names identify cases in the catalog, not a selected pass
denominator. Full identities retain submitter, native ID, and scenario index.
Opening the first blocker does not promise fifteen exact matches: implementation
dependent order, serialization, or later language gaps may remain.

`Document` currently owns six node kinds, element namespace declarations,
ancestor-based binding lookup, and arena-local `NodeId`s. Bindings suffice for
namespace fixup and the existing narrow count operation, but are not selectable
nodes. Adding another node kind affects XPath axes, node tests, identity/order,
template dispatch, variable frames, copying, temporary trees, and accounting.

The references are [XDM 3.1 namespace nodes](https://www.w3.org/TR/xpath-datamodel-31/#NamespaceNode),
[XDM document order](https://www.w3.org/TR/xpath-datamodel-31/#document-order),
and [XPath 1.0 namespace nodes](https://www.w3.org/TR/1999/REC-xpath-19991116/#namespace-nodes).
Namespace nodes belong to their parent element, not its child or attribute
sequence. Distinct elements do not share their namespace-node identities even
when bindings match. Empty default bindings remove an inherited default;
the implicit `xml` binding participates. Modern default-namespace nodes have
no node-name, while legacy name functions still return the empty prefix.
Relative order among one element's namespace nodes is implementation dependent;
it must not be inferred from an archival processor's preferred prefix order.

## Ownership and constraints

- XDM owns namespace-node meaning and identity; XML supplies declarations and
  provenance rather than a parser-native node type.
- Prepared source remains immutable and source-derived. No invocation state,
  stylesheet policy, cross-document cache, or cross-generation interning enters it.
- Namespace identity must include owning document and element, not URI alone.
  Existing arena IDs must not silently become globally qualified identities.
- XPath owns axis selection, normalization, predicates, and focus; the complete
  charged path remains the oracle where applicable.
- Result construction owns namespace attachment/fixup. Copying a namespace
  node is not copying an attribute or parsing its URI as XML.
- Effective source views preserve visible identity. The full derived-document
  oracle and temporary-tree/source parity require explicit controls.
- Charge traversal, retained occurrences, and string/copy work before allocation
  or retention. Observe cancellation on real work and discard partial products
  on failure. Namespace URIs grant no acquisition authority.
- Public source-provider traits, host APIs, node representations, unsafe code,
  and support claims are not selected by this review.

## Alternatives

### Materialize namespace nodes in prepared XDM

This could reuse `NodeId` across semantic consumers, but increases prepared
retention for inherited bindings on every element and requires a broad node-kind
audit. Preserve existing visible IDs/order and measure source inflation first.

### Derive bounded private occurrences over immutable XDM

This avoids eager expansion and can qualify each occurrence by document,
element, and prefix. It requires an honest mixed-node navigation/identity seam
and bounded lifetime/accounting; a parallel special-case executor is not acceptable.

### Continue explicit unsupported selection

Retain the current safe profile while ownership or measured costs remain unclear.
Existing binding/count operations do not imply namespace-axis support.

## Findings and uncertainties

Begin with a test-only, safe occurrence reference over one borrowed immutable
document. Study shadowing, default undeclaration, implicit `xml`, element-relative
identity, name/value/parent behavior, provenance, and limits before wiring XPath.
Borrowed-document identity in that experiment is deliberately provisional:
shared visibility views, complete clones, temporary trees, and cross-document
sequences must be reviewed before a runtime representation is selected.

The parser currently retains the containing start-tag span for namespace
declarations, not an exact declaration span. Inherited occurrences should keep
both their parent occurrence and the declaring element's provenance; do not
claim a more precise diagnostic location than the XML boundary supplies.

The temporary-tree experiment exposes a separate qualification requirement:
temporary tree identities are allocated within an invocation and restart in
another invocation. Deep lexical-frame clones retain the tree identity, so
storage addresses cannot be the identity oracle either. The test-only reference
qualifies tree/element/prefix by an explicitly borrowed invocation scope. This
is a feasibility model, not an accepted runtime token or public node handle.
Temporary nodes currently have no source/declaration location field; do not
invent source provenance when selecting their namespace occurrences.

Copying an element already retains namespace bindings, but copying a selected
namespace occurrence needs a distinct result-attachment rule. `ResultNode`
currently has pending attribute items, not pending namespace items. Neither
encoding a namespace as an attribute nor claiming element-copy support proves
namespace-node copy semantics. Attachment timing, conflicts, fixup, accounting,
and diagnostics remain part of this review's unresolved integration seam.

### Runtime consumer inventory

The first integration audit found these concrete ownership seams. This inventory
does not admit new node variants or claim that a local selector proves runtime
parity.

| Consumer | Current boundary | Required conservation before admission |
| --- | --- | --- |
| XML and prepared XDM | `Document` stores six arena kinds plus declarations | Preserve existing IDs/provenance; do not eagerly multiply inherited bindings or mutate prepared state |
| XPath paths and normalization | Existing tree paths return `Vec<NodeId>`; a bounded terminal namespace plan returns qualified nodes | Extend qualified occurrences through predicates, mixed order/deduplication, and navigation without inventing arena IDs |
| Dynamic focus | `SequenceContext` also carries an actual derived source namespace occurrence for bounded for-each scalar bodies | Extend that qualified focus through remaining consumers without substituting the owning element |
| Runtime variable frames | Separate source-ID vectors and temporary-tree maps | Preserve mixed values and invocation-local identity through binding, shadowing, and complete-clone/COW controls |
| Name/value/identity functions | Shared scalar value-of handles namespace prefix/name, URI string value, and bounded source-parent scalars; broader identity/navigation remain unmigrated | Broader navigation, identity, base behavior, and temporary parity must be deliberate and profile-aware |
| Template dispatch | Source match patterns and temporary-kind matching are distinct consumers | Preserve profile-specific built-in behavior; do not silently make namespace bindings match element/attribute patterns |
| Result construction and copying | Literal/current-name result elements, bounded ordinary computed attributes, and empty source-namespace `xsl:copy` execute; broader copy/temporary integration remains open | Preserve result-owned names/values and focus; copy bindings without aliasing source lifetimes and test attachment conflicts, timing, fixup, and diagnostics |
| Serialization and host delivery | Consume semantic result elements, not source occurrences | Resolve attachment in result construction rather than teaching transports or serializers an alternate source evaluator |

The [XSLT 1.0 built-in rules](https://www.w3.org/TR/1999/REC-xslt-19991116#built-in-rule)
specify a no-output namespace-node built-in rule and no namespace-node matching
pattern. That legacy rule must not silently settle the broader modern profile.
The [copying rules](https://www.w3.org/TR/1999/REC-xslt-19991116#copying)
and [result-tree fragment rules](https://www.w3.org/TR/1999/REC-xslt-19991116#section-Result-Tree-Fragments)
also distinguish element namespace copying from placing namespace nodes under
a root. Recovery and modern attachment errors still need separate review.
Executable namespace focus and copying cannot be credited by passing the parent
element into the existing executor. The next runtime slice needs an explicit
qualified-occurrence seam, not another namespace-specific executor.

### Accepted integration direction and remaining implementation

ADR-0021 selects derived, element-relative occurrences carried by one
private qualified semantic-node representation. Keep the existing six prepared
arena kinds and their IDs unchanged. A namespace occurrence is not an arena
attribute, an owning-element focus substitute, or a URI-interned node.

Source qualification must preserve prepared origin across effective whitespace
views while navigation uses the effective relationships. Temporary qualification
must include invocation, tree, element, and prefix, preserving identity through
lexical-frame clones without equating storage addresses with identity. The
borrowed test references demonstrate these requirements; they do not select the
runtime token, physical layout, or retained-owner strategy. No borrowed binding
may escape its owner, and cross-document order remains an explicit open item.

Paths, unions, predicates, dynamic focus, variables, and node functions must
consume that same qualified representation through the shared semantic engine.
Retain existing tree-only paths where their eligibility is proven, rather than
forcing every ordinary path to allocate namespace wrappers. XDM binding meaning
must not depend upward on runtime temporary storage. Source and temporary owners
may supply declaration relationships to the common bounded resolver; this does
not justify a public provider trait or a namespace-specific executor.

Namespace string value is the binding URI; its node name describes the prefix,
not that URI as a QName namespace. Preserve the profile distinction for absent
default-node names. Source declaration locations remain no more precise than
the retained start-tag span; temporary declarations currently have no location.
Execution failures retain the available construction/expression call site
without fabricating declaration provenance.

Variable integration must conserve ADR-0017's invocation-local COW and complete
deep-clone oracle, including cross-kind shadowing. A proposed distinct pending
namespace result item would be consumed by result construction before
serialization, with result-owned dynamic payloads and pre-retention charges.
It must not be encoded as an attribute. The tested modern attachment subset is
evidence for that seam, not admission of all constructor conflicts or selection
of XSLT 1.0 recovery rules. ADR-0018's immutable static namespace slices remain
separate from dynamically copied source bindings.

The proposed implementation sequence is:

1. Review the qualified identity/lifetime and mixed-order contract, including
   effective source views and real temporary trees, before production admission.
2. Integrate named/wildcard namespace selection, parent navigation, current-item
   name/value, and focus through existing semantic consumers. Keep unsupported
   combinations explicit until their parity and accounting are exercised.
3. Extend predicates, mixed unions, variables, identity, dispatch, and copying
   through the same representation, preserving the complete reference controls.
4. Replay unchanged OASIS cases and pinned modern cases, recording exact,
   order-sensitive, unsupported, and later-blocker dispositions separately.
   Measure ordinary tree-path regression and retained/peak capacity before
   retaining the integrated private layout.

The accepted direction does not establish full runtime parity, cross-document
order, general production namespace-axis support, or a performance benefit.
Those implementation gates remain open.

### Unchanged fifteen-case integration baseline

A fresh full replay traces all fifteen native scenario identities below. Every
case remains **initialization / unsupported / `FXXP1001`**; the requirement column
describes stylesheet pressure, not an implemented feature or promised pass.

| Native identity (scenario `#1`) | Integration requirement beyond the first blocker |
| --- | --- |
| `Lotus/axes_axes59` | Wildcard selection, sorted prefix names, current-item string value |
| `Lotus/axes_axes62` | Named namespace selection and current-item name/value |
| `Lotus/axes_axes68` | Namespace focus during recursive element traversal |
| `Lotus/axes_axes120` | Namespace iteration inside mixed element selection |
| `Lotus/copy_copy51` | Copy selected namespace occurrences into a result element |
| `Lotus/namespace_namespace28` | First namespace occurrence and legacy `local-name()` |
| `Lotus/namespace_namespace32` | Predicates over binding URI string value |
| `Lotus/namespace_namespace33` | `namespace-uri()` of namespace-node names, not binding values |
| `Lotus/namespace_namespace34` | URI-value predicates followed by node-name namespace lookup |
| `Lotus/namespace_namespace142` | Prefix filtering, count, name, local name, and namespace URI |
| `Lotus/node_node17` | `namespace::node()` and computed ordinary attributes from prefix/value |
| `Lotus/position_position76` | First namespace occurrence and legacy `name()` |
| `Lotus/position_position111` | Namespace/attribute union order and selected-sequence focus |
| `Microsoft/Namespace_CheckXmlnsResetOnResultTree` | Default undeclaration and namespace string values |
| `Microsoft/XSLTFunctions_GenereateIdAppliedToNamespaceNodesOnDifferentElements` | Distinct owner-element identity despite equal prefix/URI bindings |

The unchanged expected outputs for `namespace28` and `position76` select `xml`
as the first namespace. The lexical-prefix reference need not reproduce that
historical order. Do not change upstream outputs, adopt that order solely to
raise the numerator, or call a different permitted ordering a semantic failure.
Keep exact-byte corpus disposition separate from standards interpretation.

Reproduce the baseline with
`pwsh -NoProfile -File scripts/measure-oasis-xslt10.ps1 -TraceCase '/'`, filtering
`failure-trace` rows by the full identities above. The full replay retains 2,471
exact matches, 2,668 initialized, 2,621 successfully executed, twelve mismatches,
four comparator gaps, and 423 expected-error credits. No fixture, expected
output, production evaluator, or corpus credit changed.

## Disposition

**Incubating implementation follow-up; architectural direction accepted.**
[ADR-0021](../ADR/ADR-0021-derived-qualified-namespace-node-occurrences.md) now
selects derived qualified occurrences and their shared integration boundary.
It does not close parity, accounting, corpus, or measurement obligations or
claim general namespace-axis support. No public API, wire format, or physical
layout is stabilized. Continue incremental runtime integration under its gates.

## Required follow-up

- [x] Prove a bounded occurrence reference and cancellation/failure cleanup.
- [ ] Compare materialized versus derived identity/order and retained capacity.
- [ ] Audit every affected semantic consumer before admitting a new node kind.
- [x] Prove test-only immutable reuse and identity across shared whitespace views
  and complete derived documents, with bounded mixed-node ordering.
- [x] Compare binding navigation over real temporary-tree materialization with
  source bindings; qualify identity across deep clones and separate invocations.
- [x] Share bounded test-only binding resolution and owner qualification between
  source and temporary references; exercise the existing scalar focus evaluator.
- [x] Prove a bounded modern namespace-attachment subset into existing semantic
  results, including ownership after source disposal and serializer parity.
- [ ] Prove source/temporary-tree, focus, navigation, and copying parity before
  choosing the runtime representation.
- [ ] Execute the unchanged fifteen-case frontier and retain every disposition.
- [x] Replay and record all fifteen pre-integration first failures, including
  historical first-namespace ordering assumptions in two expected outputs.
- [x] Exercise existing charged tree-path evaluation before namespace selection
  and after explicit parent handoff across effective view/complete derivation.
- [ ] Add pinned modern-suite parity evidence for the admitted behavior.
- [x] Decide the derived representation direction and lifecycle through ADR-0021;
  executable feature admission and private layout validation remain open.

## Reopening triggers

Revisit alternatives when mixed-node navigation requires a public provider,
namespace expansion amplifies prepared retention, views lose identity, a case
requires parentless namespace construction, or a real consumer needs a broader
namespace-node contract.

## Review history

- 2026-10-02: The subsequent ordinary-source compatibility slice admits
  composed `contains()` conditions through existing typed string conversion,
  not namespace-focus emulation. Full OASIS replay remains 2,485 / 3,173
  (78.32%); `idkey31` now first fails on `generate-id(/)` within concatenation.
  Namespace-valued variables, generated-ID strings and temporary carriers
  remain separate work. Three focused string/control tests live in the
  ordinary runtime test family rather than adding an unrelated owner to the
  twelve-file namespace experiment directory.


- 2026-10-02: Template arguments now admit namespace-path counts under both
  profiles from ordinary source focus. Scalar output and argument evaluation
  share one controlled qualified selector/count helper. The callee retains an
  atomic integer, never borrowed namespace occurrences; compiled retention
  includes the selection plan. Two focused controls cover named-template
  execution, scalar parity, owner-relative counts, absent bindings, effective
  view/reference parity, cancellation, exact/one-less budgets and reuse.
  This does not admit namespace-valued variables/parameters or named-template
  calls from namespace focus.

  Full replay remains 2,485 / 3,173 (78.32%), 2,684 initialized, 2,637 executed,
  486 initialization failures, 47 execution failures, fourteen mismatches,
  four comparator gaps and 423 expected-error credits. `idkey31` now first
  stops at `contains($accumulated, concat('+', generate-id(/), '+'))`, not
  count-argument compilation. General namespace generated-ID strings, variable
  selections and composed string conditions remain a separate migration.
  Reproduce with `cargo test -p fastxslt --all-features namespace_count_argument`
  and `scripts/measure-oasis-xslt10.ps1 -TraceCase idkey31`.

  All workspace gates pass with 1,502 core tests (35 manual probes ignored),
  and the official WASM target check passes. Count argument dispatch now lives
  with existing context-derived scalar arguments; ordinary union-count behavior
  and charges are conserved. The new 77-line test child owns scalar argument
  invariants. The experiment directory has twelve direct files: retain the
  existing named semantic test/adapter families at this checkpoint, and review
  a namespace subdirectory when variable/temporary carrier integration forms
  a stable subject boundary. No public or unsafe surface changes.

- 2026-10-02: Conditional runtime integration now preserves qualified namespace
  focus through the existing boolean evaluator and its short-circuit recursive
  composition. `if`/`choose` admit codepoint `contains(., literal)`, constants,
  and already-supported focus-number tests only when their bodies are admitted.
  The private `boolean_focus.rs` owns the borrowed node-focus carrier and shared
  string search; it is not a namespace-specific executor. Namespace URI values
  are borrowed rather than substituted with parent text or copied into prepared
  state. Ordinary tree values reuse controlled string-value construction.
  Search charges one XPath operation, namespace value access, and abstract scan
  work equal to both UTF-8 lengths. Compiled retention includes the literal and
  expression site. Temporary/atomic contains focus and other namespace boolean
  consumers remain explicitly unsupported.

  Three focused controls cover both profiles and whitespace-representation parity,
  actual URI versus owner text, mixed focus positions, nested choose/if,
  ordinary tree parity, Unicode, empty literals, short-circuit charging,
  exact/one-less work limits, deterministic cancellation and clean reuse.
  Unchanged `Lotus/position_position111#1` now compares exactly. Full replay is
  2,485 / 3,173 (78.32%), 2,684 initialized, 2,637 executed, 486 initialization
  failures, 47 execution failures, fourteen mismatches, four comparator gaps,
  and unchanged 423 / 431 expected-error credits. The original fifteen-case
  namespace frontier is thirteen exact and two archival mismatches, with no
  unsupported cases remaining in that original selection. This is not complete
  namespace support or a performance claim. Reproduce with
  `cargo test -p fastxslt --all-features namespace_conditional` and
  `scripts/measure-oasis-xslt10.ps1 -TraceCase position111`.

  All workspace verification gates pass with 1,500 core tests (35 manual probes
  ignored); the official WASM target check and diff check pass as well.

- 2026-10-02: Source `for-each` now admits bounded mixed unions of ordinary
  paths and terminal namespace selections through the shared focus executor.
  Same-origin normalization preserves element/namespace/attribute/descendant
  order and deduplicates qualified identity before focus position/size. No
  cross-document ranking, variable carrier, temporary namespace focus, mixed
  sorting, or conditional body support is inferred. Intermediate products are
  bounded before retention, including duplicates, and normalization/deduplication
  work is charged. Six focused controls cover both stylesheet profiles,
  visibility/complete-reference parity, default/implicit namespace order,
  unrelated origins, quoted separators, malformed empty arms, exact/one-less
  work budgets, cancellation, capacity failure, and clean reuse.

  All workspace verification gates pass with 1,497 core tests (35 manual
  probes ignored); the official WASM target check and diff check also pass.

  The unchanged full replay remains 2,484 / 3,173 (78.29%), with 2,683
  initialized, 2,636 executed, 487 initialization failures, 47 execution
  failures, fourteen mismatches, four comparator gaps, and 423 expected-error
  credits. `position111` now first fails on unsupported `contains(.,'http')`
  compilation; conditional focus handling remains a separate next slice.
  These handcrafted controls receive no corpus credit. Reproduce with
  `cargo test -p fastxslt --all-features qualified_union`,
  `cargo test -p fastxslt --all-features mixed_focus`, and
  `scripts/measure-oasis-xslt10.ps1 -TraceCase position111`.

- 2026-10-01: Empty `xsl:copy` now executes with actual source namespace focus.
  It produces a distinct owned pending namespace item, not an attribute or
  fabricated child. The existing result-element content loop delegates
  attachment to the private 151-line `result_tree/namespace_attachment.rs`;
  ordinary content incurs no extra attachment traversal. Identical bindings
  collapse, mandatory `xml` stays implicit, and conflicts, late/default-invalid
  attachment, and escaped top-level items retain deterministic construction
  errors and the original copy site. Both profiles report these errors; broader
  legacy recovery is not selected. Text/HTML/XML output cannot silently hide
  an unattached namespace. Temporary focus, variables, `copy-of`, mixed unions,
  and nonempty copy bodies remain unsupported.

  Five new controls cover compiled execution under both profiles,
  visibility/complete-reference parity, result survival after source/compiled
  disposal, duplicates, failure provenance, cancellation, exhaustion and reuse.
  The compiled test family is a private 181-line child of namespace-focus tests;
  the original seven complete-copy reference controls remain intact. Copying
  charges one result item and prefix/URI bytes before allocation. Attachment
  charges one item, each visited preceding child/conflict binding, and the
  `B + 1` retained binding slots plus existing payload bytes before rebuilding
  a slice with `B` bindings. A one-binding append requires exactly four
  attachment result-node units and six copied bytes in its pinned fixture;
  one-less limits fail before replacing the original shared slice. A duplicate
  needs two units and zero copied bytes. No global cache, parser, authority,
  public API, or unsafe surface changes.

  Unchanged `Lotus/copy_copy51#1` compares exactly. Full replay reaches
  2,484 / 3,173 (78.29%), 2,683 initialized, 2,636 executed, 487 initialization
  failures, 47 execution failures, fourteen mismatches, four comparator gaps,
  and 423 / 431 expected-error credits. The original fifteen-case frontier is
  now twelve exact, two mismatch, one unsupported (`position111`). The immediate
  user target is 80%, not an already-achieved checkpoint.
  All workspace gates pass with 1,491 core tests (35 manual probes ignored);
  the official WASM target check also passes. Namespace-heavy and
  host-boundary performance characterization remain outstanding; no performance
  improvement is claimed. Reproduce with `cargo test -p fastxslt --all-features
  namespace_copy`, `scripts/verify.ps1`, and
  `scripts/measure-oasis-xslt10.ps1 -TraceCase copy51`.

- 2026-10-01: Inline equality/inequality of two `generate-id(path)` calls now
  uses the shared qualified source selector and XDM identity relation when a
  terminal namespace path is involved. Document origin, owning element, and
  prefix determine identity, not binding URI, arena-ID fabrication, or generated
  strings. Tree operands use the same qualified carrier; empty/empty compares
  equal and empty/nonempty differs. No cross-document ranking is introduced.

  Two compiled controls cover both profiles/views, repeated selections,
  distinct prefixes with equal URI, inherited bindings on different elements,
  namespace-versus-element distinction, empty selections, and a quoted predicate
  literal containing `=`. Legacy first-node conversion remains separate from
  modern zero-or-one cardinality (`XPTY0004`) with stylesheet provenance.
  Operand evaluation and comparison each charge one XPath operation; comparison
  also charges one node visit. Cancellation, operation exhaustion after operand
  selection, and fresh-invocation recovery are exercised. Known compiled
  retention includes both qualified paths. Standalone namespace generated-ID
  strings, namespace-focus identity bodies, variables, and mixed unions remain
  unsupported.

  The unchanged Microsoft
  `XSLTFunctions_GenereateIdAppliedToNamespaceNodesOnDifferentElements#1` now
  compares exactly. Full replay reaches **2,482 / 3,173 (78.22%)**, with 2,681
  initialized, 2,634 executed, 489 initialization failures, 47 execution
  failures, 14 mismatches, and four comparator gaps. Expected-error credit
  remains 423 / 431. The original namespace frontier is eleven exact, two
  mismatch, two unsupported (`copy51` and `position111`). No fixture, serializer,
  or comparator policy changed.

  All workspace gates pass, including 1,484 core tests (35 manual probes
  ignored), adapter tests, strict Clippy, formatting, documentation, links,
  unsafe-surface and inventory checks. The official WASM target check passes.
  Reproduce with `scripts/verify.ps1` and
  `scripts/measure-oasis-xslt10.ps1 -TraceCase GenereateIdApplied`.

- 2026-10-01: Literal result attributes now admit static values and narrow
  current-item name/local-name, string-value, position, and size AVTs under
  namespace focus. The shared attribute materializer receives the actual
  qualified occurrence; computed attribute values use the same eligibility and
  name handoff. Prefix copies charge a node visit and payload bytes before
  allocation. Ordinary tree/temporary name branches are unchanged.

  Two compiled controls cover both stylesheet profiles, effective-view/
  complete-reference parity, default empty prefixes, inherited and shadowed
  bindings, implicit `xml`, positions/sizes, and result ownership after source
  and compiled owners drop. Cancellation, a one-byte budget exhausted by two
  prefix copies, and fresh-invocation recovery are exercised. Broader path AVTs,
  variables, nested iteration, and namespace copying remain unsupported.

  The full unchanged OASIS replay remains 2,481 / 3,173 exact comparisons
  (78.19%), with 2,680 initialized, 2,633 executed, 490 initialization failures,
  47 execution failures, 14 mismatches, four comparator gaps, and 423 / 431
  expected-error credits. No new corpus pass is credited for these controls.
  All workspace gates pass, including 1,482 core tests (35 manual probes
  ignored), adapter tests, strict Clippy, formatting, documentation, links,
  unsafe-surface and inventory checks. The official WASM target check passes.

- 2026-10-01: Bounded ordinary computed attributes now consume qualified
  namespace focus through the shared QName resolver, scalar-value evaluator,
  and result attribute builder. Static/current-item names and absent/static
  namespace overrides are admitted; broader AVTs and namespace copy attachment
  remain unsupported. The modern compiler admits only the narrow `{name()}` /
  `{name(.)}` context-name form, not general dynamic names.

  Two focused controls cover both stylesheet profiles and whitespace-view/
  complete-reference parity, result ownership after source/program release,
  cancellation, budgets, failure cleanup, late attributes (`XTDE0410`), and
  empty prefix names (`XTDE0850`) with stylesheet provenance. Prefix and URI
  copies charge bytes before allocation: the `p` / `urn:p` fixture charges six
  result-text bytes and rejects a five-byte budget. This constructs ordinary
  attributes, not namespace attachment items, and introduces no alternate
  executor or prepared-state mutation.

  Unchanged `Lotus/node_node17#1` now compares exactly through the existing XML
  comparator. Full replay reaches **2,481 / 3,173 (78.19%)**, with 2,680
  initialized, 2,633 executed, 490 initialization failures, 47 execution
  failures, 14 mismatches, and four comparator gaps. Expected-error credit
  remains 423 / 431. The original namespace frontier is ten exact, two
  mismatch, three unsupported. This is XML-comparison credit, not a claim of
  identical serialization bytes.

  Final workspace verification passes: 1,480 core tests (35 manual probes
  ignored), strict Clippy, formatting, adapter tests, documentation, local links,
  unsafe-surface checks, and conserved inventories. The official
  `wasm32-unknown-unknown` adapter check also passes. Reproduce with
  `scripts/verify.ps1` and `scripts/measure-oasis-xslt10.ps1 -TraceCase node17`.

- 2026-10-01 -- Admitted bounded result-element construction and scalar sorting
  in namespace focus. Literal wrappers have no attributes; current-name computed
  elements have no attribute sets/computed attributes and only absent or static
  namespace overrides. Nested bodies retain the namespace item, not its owner.
  The common lexical-QName resolver handles the derived prefix, including the
  existing `XTDE0820` failure for an empty name. Names and text remain usable
  after source and compiled owners are dropped. No namespace attachment, AVT,
  variable-frame, temporary-focus, or new result kind was admitted.

  Sort keys admit name/local-name, bare string value, original position/size,
  and literals. The private adapter reuses existing control evaluation, typed
  key conversion, stable multi-key comparison, and comparison charges. It
  charges `s` XPath operations before retaining `s` controls and, for `n >= 2`,
  `n * (s + 1)` before retaining item/key collections, using saturating arithmetic.
  Every key charges one node visit before scalar retention; URI-value keys also
  charge one XDM string-value visit. Comparison retains the shared conservative
  `n * (floor(log2(n)) + 1) * s` operation charge. Body position/size are recomputed
  after sorting. No general path or variable sort key is inferred.

  Three new compiled controls exercise both profiles and source representations,
  stable ties, multiple keys, descending names/positions, numeric URI values,
  post-sort focus, result ownership after owner disposal, empty-name provenance,
  cancellation in sort/construction/string work, result exhaustion, and recovery.
  Full replay gains unchanged `Lotus/axes_axes59#1`, `Lotus/axes_axes62#1`,
  `Lotus/axes_axes68#1`, `Lotus/axes_axes120#1`, and
  `Microsoft/Namespace_CheckXmlnsResetOnResultTree#1`. Coverage is 2,480 / 3,173
  exact (78.16%), with 2,679 initialized, 2,632 executed, 491 initialization
  failures, and 47 execution failures. Fourteen mismatches, four comparator gaps,
  and 423 / 431 expected-error credits are unchanged. The original fifteen-case
  frontier is nine exact, two mismatch, four unsupported. Workspace gates and
  the official WASM check passed; focused namespace tests pass fourteen controls.

  ADR-0004 cohesion check: the 7,644-line runtime composition owner remains
  decomposition debt. This slice adds only qualified-sort dispatch and the
  namespace handoff in its existing context-name resolver; sorting lives in a
  private 68-line child that depends one-way on the common sort contracts.
  The 145-line compiler child owns eligibility, not execution. Its directory has
  eleven files and the runtime child directory seven; keep the named private
  seams rather than add count-based nesting. No public/ABI/unsafe change or
  ordinary tree-path allocation was introduced. General sort extraction is
  deferred to a behavior-preserving checkpoint, not mixed into this repair.

- 2026-10-01 -- Added one nonnegative integer terminal namespace predicate.
  Position restarts for each owning element after the namespace name test,
  rather than filtering the concatenated path result. Prefix order remains
  ADR-0021's lexical convention. Zero and out-of-range positions select nothing;
  unsupported general position expressions and repeated predicates remain closed.
  The compiled plan retains a scalar position, not an invocation selection.
  Each candidate charges one XPath operation before comparison, and existing
  selection capacity/work checks still discard partial results on failure.

  Three new controls compare positional results against complete per-owner
  selections, cover named axes and current sequence focus under both profiles
  and whitespace representations, and exercise partial-selection capacity,
  cancellation, exhaustion, and fresh-control recovery. Reproduce with
  `cargo test -p fastxslt --all-features namespace_focus` and
  `cargo test -p fastxslt --all-features integer_namespace_positions`.
  The unchanged `Lotus/namespace_namespace28#1` and
  `Lotus/position_position76#1` now compare exactly. Their actual sources expose
  only implicit `xml`, so this does not change the private ordering convention.
  Full OASIS replay reaches 2,475 / 3,173 exact (78.00%), 2,674 initialized,
  2,627 executed, 496 initialization failures, and 47 execution failures.
  Fourteen mismatches, four comparator gaps, and 423 / 431 expected-error
  credits are unchanged. The original fifteen-case frontier is now four exact,
  two mismatch, and nine unsupported.
  Workspace verification, official WASM-target checks, and local Markdown links
  passed; the core suite has 1,475 passed tests and 35 ignored manual probes.

- 2026-10-01 -- Admitted bare namespace-parent scalar queries in compiled
  for-each bodies: `..`, `parent::node()`, and parent string/name/local-name/
  namespace-URI functions. The qualified XPath handoff charges before returning
  the actual owning element. Runtime checks that it belongs to the same
  effective source owner before reusing existing element scalar consumers;
  neither namespace identity nor the enclosing position/size is replaced.
  No prepared representation or public boundary changed.

  Three new controls cover distinct qualified parents and source provenance,
  both stylesheet profiles, QName/namespace/string behavior, visibility-view
  versus complete-reference parity with `xml:space="preserve"`, and recovery
  after cancellation or exhausted string-value budgets. Longer paths, parent
  predicates, nested execution, variables, copying, and temporary namespace
  navigation remain unsupported. Reproduce with `cargo test -p fastxslt
  --all-features namespace_focus` and `cargo test -p fastxslt --all-features
  qualified_nodes`.

  All local verification gates and the WASM target check passed. Complete OASIS
  replay remains 2,473 / 3,173 exact (77.94%), with 2,672 initialized, 2,625
  executed, fourteen XML mismatches, and 423 / 431 expected-error credits.
  These hand-authored controls do not increase the corpus numerator.

- 2026-10-01 -- Admitted one terminal scalar/literal namespace predicate and
  qualified namespace-path scalar queries through the shared engine. The small
  private XPath predicate owner accepts `=`/`!=` for name/local-name, string
  value, or node-name namespace URI, retaining only stylesheet-derived literal
  text. It charges XPath operation and scalar visits before comparison; control
  failures propagate and discard partial selection. URI slashes inside quoted
  predicates are not owner-path separators, and quoted `namespace::` text does
  not activate this compiler path. Positional/repeated/compound predicates,
  variables, intermediate namespace steps, and broader navigation stay closed.

  Typed `count`, `name`, `local-name`, and `namespace-uri` selections reuse
  qualified path selection and the existing current-node scalar evaluator.
  Legacy first-node conversion and modern zero-or-one cardinality are distinct;
  modern violations return located `XPTY0004`. Compiled capacity accounting now
  includes predicate literals. Four added controls cover real 1.0/3.0 execution,
  filtered focus position/size, empty selections, source view/reference parity,
  URI/name distinction, cardinality, predicate cancellation/exhaustion cleanup,
  distinct owner identities, output ceilings, and unsupported grammar.

  Unchanged `Lotus/namespace_namespace142#1` becomes exact. Cases `namespace32`
  and `namespace34` now execute but remain visible mismatches: both stylesheets
  match an element in `http://test`, while their principal inputs bind the
  element to `test`. The built-in rules consequently emit only whitespace;
  no fixture, namespace meaning, comparator, or exclusion is altered to credit
  the archival reference. The full replay reports 2,473 / 3,173 exact (77.94%),
  2,672 initialized, 2,625 executed, fourteen mismatches, four comparator gaps,
  and unchanged 423 / 431 expected-error credits. Of the original fifteen cases,
  two are exact, two now mismatch, and eleven remain unsupported.

  Workspace gates passed with 1,469 core tests and 35 ignored probes; official
  WASM checking passed. Reproduce with `cargo test -p fastxslt --all-features namespace_focus`,
  `cargo test -p fastxslt --all-features qualified_nodes`, and
  `scripts/measure-oasis-xslt10.ps1 -TraceCase namespace_namespace3`.
  Parent navigation, constructor/AVT bodies, sorting, frames, identity, copying,
  temporary namespace parity, and host performance measurements remain pending.
- 2026-10-01 -- Added bounded compiled source namespace focus. A private
  compiler eligibility owner admits terminal namespace selection in
  `xsl:for-each` only when its body contains text and proven scalar value-of
  operations. A private runtime selection adapter passes the actual occurrence
  into the existing sequence executor, never substituting the parent element
  or interpreting a second instruction language. Shared value-of consumers
  distinguish prefix names, binding string values, and empty node-name
  namespace URIs; existing position/size evaluation recognizes namespace focus.
  Selection, scalar visits, string-value work, and result bytes use existing
  invocation charge points, with no prepared mutation or new authority.

  Four real compile/execute controls cover 1.0 and 3.0, default/implicit/named
  bindings, non-element empty axes, restored outer focus, view/complete-reference
  parity, cancellation during current-item value extraction, budget failure,
  concurrent prepared reuse, and explicit rejection of unmigrated consumers.
  Sorting, constructors/AVTs, variable bodies, nested execution, parent
  navigation, and temporary focus remain unsupported. Reproduce with
  `cargo test -p fastxslt --all-features namespace_focus`. Ordinary-path and
  namespace-heavy host regression measurements remain pending; no performance
  or general namespace support claim follows from these controls.
  The unchanged `Lotus/namespace_namespace33#1` now compares exactly: its
  `namespace-uri(.)` values remain empty while all effective bindings are
  counted through real iteration. The full replay conserves 3,173 cases with
  2,472 exact (77.91%), 2,669 initialized, 2,622 executed, twelve mismatches,
  four comparator gaps, and unchanged 423 / 431 expected-error credits. Fourteen
  original frontier cases remain uncredited. All workspace gates passed with
  1,465 core tests and 35 ignored probes; the official WASM check also passed.
  Compiler eligibility and runtime handoff are separate small private owners,
  not additions to a second executor. Folder review at eleven direct compiler
  children retains the existing instruction-family grouping for now; this
  narrow eligibility helper does not justify an arbitrary new folder split.
  The runtime experiment folder has six direct files, including these controls.
- 2026-10-01 -- Added the private 399-line XPath path child owner
  `xpath/path_experiment/qualified_nodes.rs`. Its typed plan delegates ordinary
  owner selection to the existing charged tree evaluator, then derives a named,
  wildcard, or `node()` terminal namespace step through XDM. Returned values are
  genuine qualified occurrences, ordered by normalized owner and lexical prefix;
  equal inherited bindings on separate elements retain distinct identity.
  Ordinary tree plans can return qualified tree values without changing the
  existing tree-only callers. Namespace predicates, intermediate namespace
  steps, descendant-separator composition, and compound name tests remain
  explicitly unsupported by this slice.

  The existing namespace-count AVT now compiles this plan rather than retaining
  only a marker. Runtime count consumes qualified XPath results; compiled
  retention accounting includes the owned path/location and optional prefix.
  No other stylesheet selection entry point admits namespace syntax yet.
  Owner evaluation, candidate visits, and qualified output retention share the
  invocation control. Binding and final sequence ceilings are independent;
  failure discards the partial path product. Four controls cover named/wildcard
  selection, tree identity, unsupported grammar, capacity/work/cancellation
  failure and recovery, and view/complete-reference parity without hidden text.
  The compiled count/attribute-focus golden remains unchanged. No alternate
  executor, new prepared storage, public interface, authority, or unsafe code is
  added. Current-item namespace focus, variables, unions, and copying remain
  the next consumer migrations. Reproduce with
  `cargo test -p fastxslt --all-features qualified_nodes` and
  `cargo test -p fastxslt --all-features xslt10_namespace_axis_count`.
- 2026-10-01 -- Qualified-path checkpoint validation passed all workspace gates:
  1,461 core tests, 35 ignored manual probes, strict Clippy, formatting,
  documentation, local links, unsafe-surface checks, and conserved inventories.
  The official WASM target check passed. Full OASIS replay remains 2,471 exact,
  2,668 initialized, 2,621 executed, twelve mismatches, and 423 expected-error
  credits. Namespace current-item focus and general stylesheet selectors remain
  uncredited; no upstream fixtures or expected outputs changed.
- 2026-10-01 -- Maintainer accepted the integration direction as ADR-0021.
  Added the private production `xdm/qualified_nodes.rs` source carrier and
  nearest-binding selector. Prepared documents now retain a private immutable
  origin token shared only with their effective views and complete derivation;
  separately parsed documents receive distinct origins. The token contains no
  invocation state or cache, and its allocation does not retain the original
  node arena in a complete clone. Existing tree IDs and namespace declarations
  remain unchanged. Borrowed occurrences cannot outlive their effective owner.

  The already-admitted `count(namespace::*)` AVT now consumes this selector
  instead of cloning the complete binding strings. Ancestor/declaration visits,
  implicit binding retention, and retained output occurrences charge XDM work
  before retention; the runtime maps exhaustion/cancellation to existing
  structured failures. This deliberately adds previously absent lookup charges
  rather than preserving an unmetered scan. Source-derived finite declarations
  and invocation work limits bound that caller; the selector also supports an
  independent binding ceiling, including undeclaration tombstones.

  Five focused controls pin `str::cmp` prefix order (empty, uppercase/lowercase,
  implicit `xml`, and Unicode), owner identity and provenance across view/clone
  references, shadowing/undeclaration, failure cleanup, and exact seven-charge
  selection boundaries. A complete root-first binding lookup independently
  checks production nearest-first selection. The existing compiled namespace-
  count/attribute-focus golden also passes. No general namespace path is
  compiled, no temporary carrier or public/unsafe surface is admitted, and
  namespace current-item focus remains the next integration step. Reproduce
  with `cargo test -p fastxslt --all-features qualified_nodes` and
  `cargo test -p fastxslt --all-features xslt10_namespace_axis_count`.
  The cohesive new source module is 333 lines; XDM has seven direct files.
  The tree builder only acquires immutable origin metadata and origin comparison;
  it does not absorb axis parsing, runtime focus, temporary storage, or copying.
  Existing pressured owners receive only a narrow caller/origin update, not a
  new semantic executor or arbitrary decomposition. The original test-only
  binding and occurrence references remain intact. All workspace gates passed,
  including 1,457 core tests (35 manual probes ignored), strict Clippy,
  formatting, documentation, links, unsafe-surface and inventory checks.
  `cargo check -p fastxslt-wasm-workbench --target wasm32-unknown-unknown`
  passed. Fresh full OASIS replay conserves 2,471 exact matches, 2,668 initialized,
  2,621 executed, twelve mismatches, and 423 expected-error credits. No new
  namespace case is credited by accepting the direction or replacing count.
- 2026-10-01 -- Added two test-only path-composition controls. The existing
  charged XPath evaluator selects an owner element, the bounded occurrence
  reference selects its inherited/shadowed binding, and an explicit parent
  handoff resumes ordinary child navigation through that same evaluator.
  Shared whitespace views and complete derivation preserve occurrence identity
  and declaration location while hiding stripped text; prepared relationships
  remain unchanged. Cancellation signalled after namespace selection is observed
  by the subsequent real XPath charge using the same invocation control.
  This test-only differential dependency does not add an XDM production
  dependency on XPath, compile a namespace axis, or execute a namespace as its
  parent. Qualified current-item focus, frames, mixed paths, and runtime
  representation remain unresolved. No production behavior or corpus credit
  changed. Reproduce with
  `cargo test -p fastxslt --all-features namespace_parent_handoff`.
  All workspace verification gates passed, including 1,452 core tests (35
  manual probes ignored), strict Clippy, formatting, documentation, local links,
  unsafe-surface checks, and conserved inventories. The last full OASIS replay
  remains 2,471 / 3,173 exact matches (77.88%); this test-only tranche was not
  credited or replayed as new corpus behavior.
- 2026-10-01 -- Recorded a candidate shared-runtime integration and the unchanged
  fifteen-case first-failure baseline. The proposal favors derived qualified
  occurrences without selecting runtime layout, changing prepared arena IDs,
  bypassing invocation-local frame ownership, or admitting a second evaluator.
  Exact-byte ordering expectations remain separate from standards meaning.
  Full OASIS replay is unchanged at 2,471 / 3,173 (77.88%). Production admission
  and an accepting ADR still require executable consumer parity and accounting.
- 2026-10-01 -- Opened as Incubating from the conserved OASIS frontier.
- 2026-10-01 -- Added `xdm/namespace_nodes_reference.rs`, a test-only safe
  reference with eight focused tests. It borrows namespace payloads from one
  immutable document, bounds unique bindings (including undeclaration
  tombstones), and charges ancestor/declaration traversal and occurrence
  construction through XDM control before retention. Tests cover inheritance,
  shadowing, default undeclaration, implicit `xml`, modern default-name absence,
  owning element, declaring-element provenance, repeated identity, different
  elements/documents/prefixes, non-element selection, concurrent reuse, capacity
  exhaustion, budget exhaustion, and mid-traversal cancellation. A root-first
  complete binding lookup independently checks the nearest-binding reference.
  Identity uses the borrowed document object only in this experiment; whitespace
  views, complete clones, node order, temporary trees, and runtime integration
  remain unchecked. No parser event, prepared arena, XPath plan, result model,
  adapter, authority, or public API changed. The new cohesive 313-line private
  module keeps the ownership experiment separate from the existing tree builder;
  the XDM directory has five direct files and requires no density extraction.
  All verification gates passed, including 1,432 core tests (34 manual probes
  ignored), strict Clippy, documentation, local links, unsafe-surface checks,
  and conserved conformance inventories. The unchanged full corpus replay
  retains 2,471 exact matches, 2,668 initialized cases, 2,621 successful
  executions, twelve mismatches, four comparator gaps, and 423 expected-error
  credits. No namespace-axis case is credited by these hand-authored tests.
  Reproduce with
  `cargo test -p fastxslt --all-features namespace_nodes_reference`,
  `scripts/verify.ps1`, and `scripts/measure-oasis-xslt10.ps1`.
- 2026-10-01 -- Extended the safe reference to twelve focused tests. An owned
  composition establishes the borrowed prepared origin when constructing either
  the shared whitespace view or complete derived-document oracle; namespace
  identity uses that origin, owning element, and prefix. It does not use the
  effective document's address, namespace URI, resource identity, or content hash.
  Both effective forms preserve namespace identity, values, and declaration
  provenance while rejecting hidden whitespace from mixed-node sequences.
  Complete charged insertion normalizes and deduplicates a bounded sequence
  containing document, element, namespace, attribute, text, comment, and PI
  occurrences. Namespace occurrences follow their owning element and precede
  its attributes and children. Lexical prefix ordering is a stable experimental
  choice only, not a normative or accepted runtime ordering convention.
  Cross-document sequences and mixtures of different effective views are
  explicitly rejected; this is not a general cross-document ordering solution.
  Sequence capacity, comparison-work exhaustion, and cancellation discard the
  partial product without changing prepared source. Source origin remains scoped
  to the borrowed prepared lifetime, and arena IDs remain document-local.
  The factory validates effective relationships for existing local tree IDs;
  it is not a public qualified-node admission API. Temporary/result-tree
  ownership, runtime axes, predicates/focus, copying, and representation-cost
  comparison remain open. No production semantics or corpus credit changed.
  All verification gates passed: 1,436 core tests (34 manual probes ignored),
  strict Clippy, formatting, documentation, links, unsafe-surface checks, and
  conserved inventories. Full OASIS replay retains 2,471 / 3,173 exact matches,
  2,668 initialized, 2,621 successfully executed, twelve mismatches, four
  comparator gaps, and 423 expected-error credits. The cohesive test-only
  module is 696 lines; no production source unit or ownership boundary expanded.
  Reproduce with the same three commands recorded above.
- 2026-10-01 -- Added four focused temporary-owner tests in
  `runtime/golden_runtime_experiment/namespace_nodes_reference_tests.rs`, bringing
  the namespace reference family to sixteen tests. They exercise the existing
  semantic-result-to-temporary-tree materializer, not reparsed XML standing in
  for temporary storage. A complete source binding lookup independently checks
  inheritance, shadowing, default undeclaration, implicit `xml`, and parent
  relationships. An explicit borrowed invocation scope disambiguates colliding
  tree IDs across invocations, preserves identity through complete tree clones,
  and distinguishes newly allocated trees within one invocation. Non-elements
  select nothing; occurrence selection does not add namespace nodes to child
  sequences. A local enumeration checks selection positions and size, but does
  not claim executable XPath predicate or `position()`/`last()` parity. Capacity,
  work exhaustion, and mid-traversal cancellation discard partial occurrences
  and leave temporary storage unchanged. Temporary declaration provenance and
  namespace-only result attachment remain unsupported integration seams.
  The test module depends inward on the existing temporary materializer and
  result types; it adds no production navigation interface, alternate executor,
  public representation, source authority, or runtime sequence variant. This
  named test family lives under the existing runtime test directory rather than
  adding another direct runtime file. The pressured `runtime_context.rs` only
  adds a test-module declaration; its semantic responsibilities do not expand.
  No OASIS exact match is credited by these controls.
  All workspace verification gates passed, including 1,440 core tests (34
  manual probes ignored), strict Clippy, formatting, documentation, local links,
  unsafe-surface checks, and conserved inventories. The new test module is 258
  lines. Reproduce the sixteen focused controls with
  `cargo test -p fastxslt --all-features namespace_nodes_reference`.
- 2026-10-01 -- Audited path results, dynamic focus, frames, node functions,
  dispatch, copying/attachment, and serialization before attempting executable
  namespace focus. Current source-ID and temporary-node types cannot carry the
  occurrence without a representation change; no parent-element substitution
  or alternate evaluator was added. Added an identity-parity regression and an
  ignored known-capacity probe comparing eager retention of borrowed occurrences
  with one-selection-at-a-time derivation. The probe does not model a seventh
  arena kind, clone namespace strings, or measure allocator-exact peak memory.
  On this 64-bit build, one borrowed occurrence is 64 bytes:

  | Elements | Bindings per element, including `xml` | Occurrences | Eager vector capacity bytes | One derived selection vector capacity bytes |
  | --- | --- | --- | --- | --- |
  | 9 | 5 | 45 | 4,824 | 512 |
  | 65 | 9 | 585 | 68,120 | 1,024 |
  | 513 | 17 | 8,721 | 1,062,936 | 2,048 |

  Both references preserve owner-relative identity and binding values. Counts
  include vector capacity slack; eager totals include the outer vector. They
  exclude prepared storage, transient binding maps, allocator overhead, and
  process/runtime memory. Derivation repeats traversal and is not shown faster
  by this capacity probe. These generated wide fixtures justify studying lazy
  occurrences further, not selecting a runtime representation. Reproduce with
  `cargo test -p fastxslt --all-features measure_eager_vs_derived_namespace_occurrence_capacity -- --ignored --nocapture`.
  All workspace verification gates passed: 1,441 core tests, 35 ignored manual
  probes, strict Clippy, formatting, documentation, links, unsafe-surface checks,
  and conserved inventories. The focused family has seventeen passing controls
  plus one manual probe. Production semantics and the last full OASIS replay
  remain unchanged at 2,471 / 3,173 exact matches (77.88%); this tranche adds no
  corpus credit. The cohesive XDM reference remains below 1,000 lines.
- 2026-10-01 -- Extracted nearest-binding resolution and source/temporary owner
  qualification into `xdm/namespace_binding_reference.rs`, a 136-line test-only
  owner. Both reference callers supply nearest-first declaration rows and their
  honest provenance (source locations or explicit absence). The shared resolver
  preserves tombstones, capacity admission, implicit `xml`, and the original
  ancestor/declaration work-charge sequence. Occurrence construction remains
  charged by each caller. A typed owner discriminant prevents source and
  temporary occurrences from aliasing even when local element numbers match.
  The runtime test depends inward on XDM; XDM does not depend on runtime storage,
  navigation, parameters, result state, or host policy. No provider trait,
  production variant, external ABI, or unsafe surface is introduced. The XDM
  directory now has six direct files and the reference remains below 1,000 lines.
  This removes duplicated binding semantics rather than splitting by line count.
  The known-capacity probe reproduces all three prior rows unchanged.

  One additional control sends selected namespace sequence positions/sizes
  through the existing charged `evaluate_context_focus_equality` implementation.
  First/last comparisons use the selected sequence, not the parent's child count.
  It verifies exact XPath-operation charges, cooperative cancellation on the
  charge following the signal (`FXCT0001`), and missing focus (`XPDY0002`) with
  supplied call-site diagnostic location. No temporary declaration location is
  manufactured. This proves scalar focus-evaluator reuse, not compiled namespace
  axis/predicate admission or namespace current-item name/value/copy execution.
  Those still require the reviewed runtime representation and attachment seam.
  The reference family now has nineteen passing controls and one ignored manual
  probe. Reproduce it with `cargo test -p fastxslt --all-features namespace_` (the
  filter also runs existing unrelated namespace controls) and the capacity
  command recorded above. No production semantics or corpus credit changed.
  All workspace verification gates passed: 1,443 core tests, 35 ignored manual
  probes, strict Clippy, formatting, documentation, local links, unsafe-surface
  checks, and conserved inventories. OASIS remains at the last full replay's
  2,471 / 3,173 exact matches (77.88%); it was not rerun for this test-only
  extraction and scalar-control tranche.
- 2026-10-01 -- Added seven namespace-copy attachment controls in the private
  419-line `runtime/golden_runtime_experiment/namespace_copy_reference_tests.rs`.
  The prototype consumes valid selected namespace bindings and already-built
  child results; it is a result-construction experiment, not an XPath/XSLT
  interpreter. Source and real temporary-tree selections use the same attachment
  path and copy namespace strings into result-owned storage. Results serialize
  after the source document/tree and selected borrowed occurrences are dropped.
  No declaration becomes an ordinary attribute or child. Copied bindings also
  participate in the existing serializer's descendant namespace fixup.

  The narrow modern rules are checked against
  [XSLT 3.0 complex content](https://www.w3.org/TR/xslt-30/#constructing-complex-content):
  identical prefix/URI duplicates collapse; differing URI values for one prefix
  fail with `XTDE0430`; namespace attachment after text/comment children fails
  with `XTDE0410`; document attachment fails with `XTDE0420`; and attaching a
  default binding to a null-namespace destination fails with `XTDE0440`.
  An empty text item does not create a late-attachment barrier. These experiments
  do not select XSLT 1.0 recovery or backward-compatible error policy.

  Member count and retained binding count are separately bounded. Result-node
  work is charged during candidate visits and conflict scans; namespace payload
  bytes are charged before copying strings. Exhaustion and mid-attachment
  cancellation return a structured failure without a partial result. Children
  are supplied already constructed and moved, not cloned by this prototype;
  these controls do not establish accounting for general child construction.
  Attributes are explicitly outside this attachment subset, as are general
  namespace construction, atomization, document flattening, adjacent-text
  merging, initial destination bindings, and full namespace fixup conflicts.
  Construction-site and declaration provenance remain integration obligations.

  Every successful serialization is compared through the existing scoped-stack
  serializer and complete namespace-copy oracle. Tests preserve current explicit
  end tags and, when selected, the redundant valid `xml` binding; no serializer
  output rule or upstream expected result was changed. The namespace reference
  family now has 26 passing controls and one ignored capacity probe. The copy
  module depends inward on existing result types and serializers; the production
  result enum, compiler, path evaluator, focus representation, variable frames,
  adapters, resource authority, and unsafe surface are unchanged. The existing
  runtime test directory has four direct files, below the density trigger.
  Reproduce with `cargo test -p fastxslt --all-features namespace_copy_reference`
  and `scripts/verify.ps1`. All gates passed, including 1,450 core tests (35
  manual probes ignored), strict Clippy, formatting, documentation, links,
  unsafe-surface checks, and conserved inventories. No corpus pass is credited;
  the last full OASIS replay remains 2,471 / 3,173 exact matches (77.88%).
