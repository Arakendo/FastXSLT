# OASIS chained child-axis match predicates

Date: 2026-10-01

## Result

The unchanged OASIS CD04 `Microsoft/BVTs_bvt099#1` now initializes, executes,
and compares exactly. Its five modes distinguish predicate order, repeated
`position()`/`last()` filtering, context-name tests, and child-value boolean
composition. No upstream input, stylesheet, expected output, or comparator was
modified.

Shared typed path predicates now recognize whitespace-tolerant
`position() = last()`, equality to `last() - constant`, and relational
comparisons to `last()`. Existing boolean composition handles the last/penultimate
disjunction. Sequential filters recompute focus from the surviving nodes.

The compiler admits these match predicates only on child-axis paths for which
ancestor-context reconstruction is valid. Non-child chained patterns remain
unsupported. Absolute paths retain ADR-0013's invocation-owned charged
membership/fallback behavior; relative patterns do not acquire a cache.

## Temporary-node boundary

Temporary-node dispatch has no location-path pattern evaluator. Previously its
catch-all arm silently returned false. It now reports `FXRT1007 / unsupported`
with the pattern's stylesheet location rather than selecting a plausible
fallback template. Source/temporary path-pattern parity remains future work;
no second evaluator is introduced.

## Conserved measurement

The complete hash-verified local OASIS CD04 sweep reports:

- 3,173 catalog cases;
- 2,469 exact XML comparisons (77.81%), up one;
- 2,666 initialized and 2,619 successfully executed, each up one;
- 504 initialization failures and 47 execution failures;
- 12 visible XML mismatches and four comparator gaps, unchanged;
- 423 / 431 expected-error credits, unchanged.

Direct DTD attribution remains 133 cases: 30 internal parses, 94 sealed-external
parses, and nine unsupported. Production DTD denial is unchanged.

## Verification and cohesion

Focused regressions cover source selection (`23|234|2`), relational `last()`
tests, sequential refiltering, rejection of non-child chained patterns, and
structured temporary-pattern rejection with provenance. Three former compiler
unsupported assertions now verify the newly admitted typed child-axis paths.

ADR-0004 review: path owner approximately 2,919 lines, template pattern compiler
1,375, temporary executor 1,685; direct directory counts 49, 16, and 54.
Retain the bounded additions in existing named owners: scalar focus grammar,
pattern eligibility, and temporary dispatch. No new responsibility, mutable
context, cycle, public API, authority, unsafe surface, or host dependency is
introduced. Boolean composition already has a private predicate owner. A broader
focus-expression grammar warrants extraction before further expansion; directory
regrouping remains a separate conservation checkpoint.

Reproduce with `scripts/measure-oasis-xslt10.ps1`, optionally
`-TraceCase 'Microsoft/BVTs_bvt099#1'`, and `scripts/verify.ps1`.

All verification gates passed: formatting, strict workspace Clippy, workspace
tests (1,419 core tests passed; 34 manual probes ignored), documentation,
Markdown links, unsafe-surface checks, and conformance source/inventory checks.
