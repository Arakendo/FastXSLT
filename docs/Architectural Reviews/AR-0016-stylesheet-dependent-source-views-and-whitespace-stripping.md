# AR-0016: Stylesheet-Dependent Source Views and Whitespace Stripping

| Field | Value |
| --- | --- |
| Status | Incubating |
| Opened | 2026-08-30 |
| Last reviewed | 2026-08-30 |
| Scope | XSLT whitespace declarations, XDM navigation, prepared-input reuse, node identity, and execution accounting |
| Trigger | XSLT30 `mode-1301` requires `xsl:strip-space` over a reusable prepared source |
| Related ADRs | ADR-0001, ADR-0002, ADR-0004, ADR-0007 |
| Related reviews | AR-0007, AR-0008, AR-0009, AR-0013 |
| Related evidence | `../Evidence/xslt30-mode-denominator-and-qname-identity-2026-08-29.md` and the pinned XSLT30 `mode-1301` case |

## Architectural question

How should FastXSLT apply stylesheet-dependent source-tree semantics such as
`xsl:strip-space` while prepared XDM remains immutable, source-derived, and
reusable across stylesheets, and while XPath, template dispatch, built-in rules,
string values, diagnostics, budgets, and node identity all observe one
consistent semantic document?

## Trigger and evidence

The pinned XSLT30 `mode-1301` case is otherwise within the current mode and
template-dispatch surface. Its source contains indentation-only text children,
and its stylesheet declares `<xsl:strip-space elements="*"/>`. The expected
result depends on those text nodes being absent while built-in template rules
descend from the document node to the explicitly selected element children.

The current XML and XDM path correctly retains the source text nodes. AR-0008
assigns whitespace stripping to XSLT rather than parser mechanics. AR-0009
requires reusable prepared input to remain source-derived and excludes
stylesheet-dependent state. Consequently, parsing a stripped tree or mutating a
prepared document would make the same admitted source mean different things
according to which stylesheet prepared it first.

Filtering children only in the built-in template path would make this one case
appear to pass while XPath axes, explicit selections, string-value operations,
copying, and other consumers could still observe the stripped nodes. That is a
semantic split, not an admissible incremental implementation.

No current measurement compares a derived tree, a visibility overlay, or
stylesheet-specialized preparation. No current review selects a navigation
interface for carrying such a view through all semantic consumers.

## Ownership and constraints

- XSLT compilation owns the expanded-name rules and precedence semantics
  derived from `xsl:strip-space` and `xsl:preserve-space` declarations.
- XML parsing retains lexical XML behavior and must not infer stylesheet policy.
- XDM owns source node identity, document order, relationships, provenance, and
  the physical prepared representation. A semantic view may hide nodes but may
  not silently renumber, merge, or change the identity of visible source nodes.
- Prepared input remains immutable and source-derived under AR-0009. One
  prepared document must remain reusable by stripping and non-stripping
  stylesheets, including concurrent executions and overlapping generations.
- Runtime owns the composition of a compiled stylesheet's whitespace policy
  with one invocation's source document. Invocation state must not be written
  back into the prepared input or compiled stylesheet.
- XPath navigation, template selection, built-in rules, node tests, string
  values, copying, comparison, serialization-facing result construction, and
  inspection must observe the same effective source semantics wherever the
  standard makes whitespace stripping relevant.
- Work and retained memory introduced by view construction or filtering must be
  charged and attributable to the owning compilation, prepared input, worker,
  or invocation. It may not become an unbounded global or cross-generation
  cache.
- Cancellation and structural budgets must remain enforceable while a view is
  constructed and consumed. No fallback may spill to disk under ADR-0002.
- Source locations and structured diagnostics must continue to refer to the
  admitted source and stylesheet declarations rather than a fabricated file or
  host path.
- AR-0007 permits the concrete tree evaluator but does not admit a speculative
  public or universal provider trait. Any private indirection must be justified
  by this concrete semantic pressure and measured against the direct tree path.
- Safe Rust remains the reference. This review cannot admit an unsafe
  optimization under ADR-0003.

## Alternatives

### A. Apply whitespace policy during XML parsing or XDM construction

This is mechanically simple but assigns stylesheet semantics to the parser and
couples one prepared source to one stylesheet. It violates AR-0008 and AR-0009
and prevents correct reuse across stylesheets.

### B. Clone and derive a complete effective document for each invocation

This supplies a simple semantic reference because every existing navigation
operation can consume the derived tree. It preserves the original prepared
document, but it walks and allocates the source again on every transform. That
may erase the demonstrated value of prepared reuse and increase peak memory.
Node identity, provenance, budget charges, cancellation, and the relationship
between original and derived nodes also require explicit rules.

### C. Compose an immutable visibility view at execution time

Compilation retains the whitespace rule table and execution composes it with
the prepared document. Navigation skips stripped text nodes while preserving
the identity and order of visible source nodes. A compact mask or lazy policy
could avoid cloning node payloads, but every semantic access path must consume
the view consistently. Repeated predicate checks, view construction cost, and
private API shape require measurement.

### D. Retain stylesheet-specific prepared variants

A generation could derive and retain a document form keyed by stylesheet
whitespace policy. Reuse may amortize construction but makes retention
stylesheet-dependent, adds cache-key and eviction policy, and can multiply
memory across compiled stylesheets. This reopens AR-0009 and AR-0013 and is not
admitted by the present review.

### E. Filter only the paths exercised by `mode-1301`

Filtering built-in child traversal would be small and would satisfy the visible
fixture. Other already-supported source operations would still observe the
same stripped nodes. This creates plausible partial output and is rejected as a
conformance shortcut.

## Findings and uncertainties

- Whitespace stripping is stylesheet-derived source semantics, not parser
  normalization and not source-only prepared state.
- The same prepared XDM document must support different effective whitespace
  policies without mutation or identity collapse.
- A complete derived document is the clearest safe reference candidate, while
  an immutable execution view is the leading optimization candidate. Neither is
  selected without a semantic inventory and measurements.
- The pressure is broader than `mode-1301`: explicit XPath axes, template
  selection, string values, copying, and future inspection surfaces must agree.
- General `xsl:strip-space` and `xsl:preserve-space` matching, import
  precedence, conflicts, schema-aware whitespace, and interaction with
  `xml:space` remain outside the first exact `elements="*"` experiment unless
  required by the pinned case.
- The required private seam may also inform future physical source strategies,
  but this evidence does not reopen XSLT streaming or justify a universal
  navigation provider.

## Disposition

**Incubating.** Keep `mode-1301` visibly not run until FastXSLT can apply one
stylesheet-owned whitespace policy consistently across every source-semantic
consumer used by the case. Do not mutate prepared XDM, move the rule into XML
parsing, retain an implicit stylesheet-specific cache, or add a built-in-rule-
only filter.

The first experiment may use a complete safe derived document as a semantic
reference and compare it with a private immutable visibility view. No public
source-view API, cache policy, physical representation, or performance
guarantee follows from this disposition.

## Required follow-up

- [ ] Inventory every current source-document navigation and string-value entry
  point used by XPath, XSLT template dispatch, built-in rules, copying,
  comparison, diagnostics, and result construction.
- [ ] Compile the exact `elements="*"` declaration into stylesheet-owned static
  policy without changing parser or prepared-input behavior.
- [ ] Implement a safe complete-derived-document reference for the exact
  `mode-1301` semantics, including cancellation, work charges, provenance, and
  visible-node identity mapping.
- [ ] Prototype a private immutable visibility view only after the semantic
  inventory identifies the smallest complete access seam.
- [ ] Differentially verify derived-document and view behavior for stripping
  and non-stripping stylesheets sharing the same prepared source, including
  concurrent execution and generation replacement.
- [ ] Add focused tests proving XPath, built-in traversal, explicit template
  selection, string values, and copying cannot disagree about stripped nodes.
- [ ] Execute pinned `mode-1301` without modifying its source, stylesheet, or
  expected result, then update the conserved mode ledger.
- [ ] Measure preparation/execution latency, retained and peak memory, warm
  throughput, and break-even reuse for the reference and view candidates before
  retaining an optimized representation or cache.
- [ ] Revisit general declaration matching, precedence, `xml:space`, and typed
  whitespace only when exact corpus cases enter selection.

## Reopening triggers

Revisit the disposition when the navigation inventory is complete, a safe
reference executes `mode-1301`, another corpus case requires broader whitespace
semantics, the reference path materially harms prepared reuse, or a consumer
needs an explicit effective-document inspection contract.

## Review history

- 2026-08-30 -- Opened as Incubating after `mode-1301` demonstrated that
  stylesheet-dependent whitespace semantics cannot be assigned to the parser,
  reusable prepared XDM, or one narrow built-in traversal without violating
  existing ownership and semantic-parity constraints.
