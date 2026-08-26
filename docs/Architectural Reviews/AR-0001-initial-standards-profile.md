# AR-0001: Initial Standards Profile and Conformance Baseline

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-08-25 |
| Last reviewed | 2026-08-25 |
| Scope | Product standards and semantic baseline |
| Trigger | The first transform slice needs a precise XPath/XSLT target |
| Related ADRs | None |
| Related evidence | `corpus/golden/hello`, peer review in `docs/Evidence/`, `docs/Evidence/w3c-suite-catalog-inventory-2026-08-25.md`, `docs/Evidence/oasis-xslt10-suite-candidate-review-2026-08-25.md`, `docs/Evidence/private-golden-transform-slice-2026-08-25.md`, and `docs/Evidence/xslt30-template-006-private-execution-2026-08-25.md` |

## Architectural question

Which XSLT, XPath, XML, XDM, and serialization editions or profiles should
FastXSLT treat as its first normative target, and which versioned conformance
suites will measure progress?

## Trigger and evidence

The project cannot correctly design names, values, sequences, pattern matching,
backwards-compatibility behavior, errors, or its public scope without a declared
standards baseline. The seed golden case uses only a small syntax intersection
and therefore does not answer the question.

The reviewed Weaver XSLT peer targets XSLT 3.0 basic conformance and XPath 3.1,
using golden, QT3, XSLT 3.0 suite, and cross-backend tiers. This proves that a
modern target can be organized incrementally; it does not establish FastXSLT's
user requirements, Rust ecosystem constraints, or acceptable implementation
schedule.

A concrete consumer class is now known: performance-sensitive applications,
including ASP.NET services, will embed FastXSLT. Their representative transform
families, compatibility requirements, priority features, deployment
constraints, and workload envelopes remain missing product evidence. They are
needed before application-fit and host-performance claims, but they do not block
a deliberately staged standards-driven preview.

The pinned-suite inventory now establishes 31,821 QT3 cases across 428 catalog
test sets and 14,600 XSLT30 cases across 234 catalog test sets at the revisions
recorded in the corpus policy. It also proves every root-catalog test-set
reference resolves. These counts describe available upstream pressure; they do
not choose a profile, identify supported dependencies, or establish executable
coverage.

## Ownership and constraints

FastXSLT owns the declared semantics. An XML parser, reference processor, or
test harness may provide mechanics and evidence but cannot define the product's
contract accidentally. Suite licenses and acquisition requirements must remain
separate from the core crate.

ADR-0001 permits implementation in one crate and does not constrain the chosen
standards profile.

## Alternatives

### A. XSLT 1.0 and XPath 1.0 first

This offers a smaller language and a fast path to useful legacy transforms.
Risks include shaping value and node-set APIs around compatibility semantics
that do not generalize cleanly to XDM sequences and later standards.

The completed OASIS TC provides a 2005 Committee Draft 04 collection with 3,173
catalog entries and useful specification citations, scenarios, discretionary
metadata, and a doubts overlay. It is no longer maintained, contains duplicate
case identities and unresolved annotations, assumes older filesystem/network
conditions, and includes contributor-specific redistribution terms that are not
safe to treat as MIT-compatible vendored corpus without separate legal review.
It remains useful as a local legacy reference but is weaker as FastXSLT's
primary public denominator.

### B. XSLT 3.0 basic conformance and XPath 3.1 from the start

This provides a modern semantic foundation and clearer alignment with current
standards suites. It substantially increases the type system, function library,
error, package, and feature surface before a complete release is possible.

### C. Modern internal model with a named staged compatibility slice

Use modern XDM/XPath concepts internally while the first shipped slice supports
an explicitly enumerated intersection or compatibility profile. This may ease
growth but risks an ambiguous product claim unless unsupported behavior and
suite selection are reported precisely.

## Findings and uncertainties

- A standards decision is required before public API or conformance claims.
- The first golden case can exercise all three alternatives and is not decision
  evidence by itself.
- The test harness should be designed to catalog unsupported and excluded cases
  explicitly regardless of target.
- The pinned suites provide enough standards evidence to select a staged,
  explicitly incomplete preview without waiting for consumer artifacts. They do
  not establish which optional features or workload shapes matter most to
  embedded applications.
- Current suite and data-model evidence favors Alternative C over a 1.0-only
  internal model: modern suites are immutable Git inputs with richer dependency
  metadata, while the strongest legacy candidate is archival and redistribution
  constrained. Representative consumer transforms remain parallel evidence for
  prioritization, compatibility, and host design rather than a prerequisite for
  a testable standards-profile decision.
- The private golden slice now executes through XML, owned XDM, XSLT/XPath
  compilation, runtime, semantic result, and serialization. Its syntax remains
  common to all three alternatives and therefore confirms architecture without
  selecting a standards profile.
- Pinned XSLT30 case `template-006` now executes from its unmodified upstream
  test-set metadata through a first-party overlay. Its `XSLT20+` dependency and
  very small root-template behavior show that honest suite-linked evidence can
  start before broad conformance, but one intersection case still cannot choose
  the product profile.
- A complete aggregate pass over all 14,600 XSLT30 cases finds 9,663 case-local
  stylesheet references, 7,646 distinct referenced stylesheet files, 22
  dependency kinds, 15 top-level assertion kinds, three environment-binding
  shapes, and 564 combined metadata shapes. This is sufficient to drive staged
  preview selection now, but it also proves that filename or superficial
  stylesheet syntax is not an honest denominator.
- A review of the local TS XSLT peer at commit `9c48142` identifies a candidate
  progression from literal/value extraction through apply-template dispatch,
  parameters/variables/conditionals, and later explicit multi-resource
  resolution. Exact element-name dispatch through `xsl:apply-templates` is the
  strongest next private pressure because it appears in the peer's first
  non-trivial golden, workbench, curated suite strategy, and large stylesheet
  workload. The peer worktree was modified and is not the intended consumer;
  this observation cannot support consumer-fit or performance claims.

## Disposition

**Under Review with Alternative C as the working recommendation.** A private
standards-driven preview may use complete W3C case metadata to select coherent
feature families and test ownership and resource boundaries. It must not imply
broad version support. Do not label FastXSLT as conformant until the initial
target, suites, exclusions, and reporting policy are accepted in an ADR.

## Required follow-up

- [x] Record the first intended consumer class: embedded applications including
  performance-sensitive ASP.NET services.
- [ ] Record representative transform families from an intended consumer before
  claiming application fit or selecting host/performance defaults; do not use
  this as a gate on standards-driven preview testing.
- [x] Pin and structurally inventory the QT3 and XSLT30 suites used by the
  modern-profile alternatives.
- [x] Inventory the official candidate suites, versions, licenses, acquisition,
  and harness requirements for the modern and XSLT 1.0 alternatives; retain the
  OASIS archive as local-only evidence rather than admitted corpus.
- [x] Prototype the `hello` case only as a throwaway/private vertical slice if
  needed to test architecture before disposition.
- [x] Inventory XSLT30 dependency, environment, stylesheet, and assertion
  families across all 14,600 pinned cases.
- [ ] Select a coherent preview denominator from complete case metadata and
  executable engine outcomes.
- [ ] Propose an ADR naming the target, deliberate exclusions, suites, and
  criteria for widening scope.

## Reopening triggers

After disposition, reopen or supersede this review when a required transform
falls outside the profile, upstream standards or suites change materially, or
the selected internal model blocks a planned compatibility level.

## Review history

- 2026-08-25 -- Opened as Under Review during project scaffolding.
- 2026-08-25 -- Recorded the pinned QT3/XSLT30 catalog inventory; profile and
  executable selection remain unresolved.
- 2026-08-25 -- Reviewed the OASIS XSLT/XPath 1.0 Committee Draft 04 archive;
  Alternative C became the working recommendation, pending consumer evidence.
- 2026-08-25 -- Completed the private `hello` transform across the intended
  semantic owners; its version-intersection syntax did not resolve the profile.
- 2026-08-25 -- Executed pinned XSLT30 `template-006` through an explicit local
  overlay while retaining upstream environment, stylesheet, assertion, and case
  identity; broader dependency-aware selection remains open.
- 2026-08-25 -- Inventoried TS XSLT peer transform families and selected exact
  element-name apply-template dispatch as the next private candidate; retained
  the first-consumer follow-up because peer implementation scope is not product
  demand.
- 2026-08-25 -- Clarified that the admitted W3C suites can drive a testable
  staged standards preview before consumer artifacts arrive. Consumer evidence
  remains required for application-fit, priority, host-lifecycle, and
  performance decisions rather than profile testability itself.
- 2026-08-25 -- Inventoried all XSLT30 case metadata and retained 564 distinct
  dependency/assertion/environment/stylesheet shapes. Preview denominator
  selection remains open and must use complete metadata plus engine outcomes.
