# XSLT30 Streaming Profile Denominators

Date: 2026-09-03

## Inputs

- W3C XSLT 3.0 test-suite revision
  `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`.
- Root `catalog.xml` and every referenced test-set catalog.
- ADR-0007's explicit exclusion of XSLT streaming implementation and
  conformance from the current FastXSLT profile.

## Method

A typed first-party aggregate overlay records the immutable suite revision,
root catalog, native feature name, exact test-set and case totals, nonexecution
status, and rationale. Its executable verifier rejects unknown overlay fields,
parses the pinned root catalog and each referenced test-set catalog, and selects
only test sets whose own dependency metadata declares
`feature="streaming"`.

The verifier proves that the catalog references 91 unique qualifying test-set
files, that case names are unique within each set, and that those sets contain
exactly 2,746 cases. Selection is therefore derived from inherited native
metadata. It is not inferred from directories, filenames, case names,
stylesheet text, or expected results.

## Result

| Scope | Test sets | Cases | Passed | Engine unsupported | Profile excluded | Default not run |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Native test sets declaring `feature="streaming"` | 91 | 2,746 | 0 | 0 | 2,746 | 0 |

The conserved XSLT30 denominator total is now 3,441 cases: 488 passed
comparisons, 12 engine-unsupported cases, 2,821 profile exclusions, and 120
visible default not-run cases across 111 complete test sets.

## Claim boundary

These are profile dispositions, not executions or passing results. FastXSLT
does not claim XSLT streaming semantics, streamability analysis, streamed
source processing, or streaming conformance. Case-level streaming metadata in
otherwise selected test sets remains the responsibility of those test-set
overlays; this aggregate rule covers only a dependency inherited from the test
set itself.

An individual case may be promoted only if the selected profile changes or
native metadata establishes that it does not require streaming behavior. Such
a promotion must explicitly override the inherited rule and carry executable
semantic evidence.
