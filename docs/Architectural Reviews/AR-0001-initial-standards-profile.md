# AR-0001: Initial Standards Profile and Conformance Baseline

| Field | Value |
| --- | --- |
| Status | Under Review |
| Opened | 2026-08-25 |
| Last reviewed | 2026-08-25 |
| Scope | Product standards and semantic baseline |
| Trigger | The first transform slice needs a precise XPath/XSLT target |
| Related ADRs | None |
| Related evidence | `corpus/golden/hello`, peer review in `docs/Evidence/` |

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

## Disposition

**Under Review.** Do not label FastXSLT as conformant with a version or expose a
stable transformation API until the initial target, suites, and reporting policy
are accepted in an ADR.

## Required follow-up

- [x] Record the first intended consumer class: embedded applications including
  performance-sensitive ASP.NET services.
- [ ] Record representative transform families from the first consumer.
- [ ] Inventory the official candidate suites, versions, licenses, acquisition,
  and harness requirements for each viable target.
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
