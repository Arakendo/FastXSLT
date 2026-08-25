# AR-0001: Initial Standards Profile and Conformance Baseline

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-08-25 |
| Last reviewed | 2026-08-25 |
| Scope | Product standards and semantic baseline |
| Trigger | The first transform slice needs a precise XPath/XSLT target |
| Related ADRs | None |
| Related evidence | `corpus/golden/hello`, peer review in `docs/Evidence/`, `docs/Evidence/w3c-suite-catalog-inventory-2026-08-25.md`, and `docs/Evidence/oasis-xslt10-suite-candidate-review-2026-08-25.md` |

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
including ASP.NET services, will embed FastXSLT. Missing evidence still includes
their representative transform families, compatibility requirements, priority
features, deployment constraints, acceptable time to first useful release, and
any need to compare with 1.0-only processors.

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
- There is not yet enough product evidence to choose among the alternatives.
- Current suite and data-model evidence favors Alternative C over a 1.0-only
  internal model: modern suites are immutable Git inputs with richer dependency
  metadata, while the strongest legacy candidate is archival and redistribution
  constrained. Representative consumer transforms are still required before
  this recommendation becomes an accepted product decision.

## Disposition

**Under Review with Alternative C as the working recommendation.** A private
`hello` experiment may use the syntax intersection to test ownership and
resource boundaries, but must not stabilize a public API or imply a version.
Do not label FastXSLT as conformant until the initial target, suites, exclusions,
and reporting policy are accepted in an ADR.

## Required follow-up

- [x] Record the first intended consumer class: embedded applications including
  performance-sensitive ASP.NET services.
- [ ] Record representative transform families from the first consumer.
- [x] Pin and structurally inventory the QT3 and XSLT30 suites used by the
  modern-profile alternatives.
- [x] Inventory the official candidate suites, versions, licenses, acquisition,
  and harness requirements for the modern and XSLT 1.0 alternatives; retain the
  OASIS archive as local-only evidence rather than admitted corpus.
- [ ] Prototype the `hello` case only as a throwaway/private vertical slice if
  needed to test architecture before disposition.
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
