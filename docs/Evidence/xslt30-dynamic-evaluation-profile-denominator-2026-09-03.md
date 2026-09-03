# XSLT30 Dynamic-Evaluation Profile Denominator

Date: 2026-09-03

## Inputs

- W3C XSLT 3.0 test-suite revision
  `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`.
- Complete 57-case `tests/insn/evaluate/_evaluate-test-set.xml`.
- ADR-0007's explicit exclusion of dynamic evaluation from the current
  FastXSLT profile.

## Method

A typed first-party aggregate overlay records the immutable suite revision,
root catalog, native feature name, exact test-set and case totals, nonexecution
status, and rationale. The same executable verifier used for inherited
streaming exclusions rejects unknown overlay fields, parses the pinned root
catalog and every referenced test-set catalog, and selects only test sets whose
own dependency metadata declares `feature="dynamic_evaluation"`.

The verifier proves that exactly one unique test-set file qualifies, that its
57 case names are unique, and that the dependency is inherited by the whole
set. The rule does not infer capability from the `evaluate` name or inspect
stylesheet syntax.

## Result

| Test set | Total | Passed | Engine unsupported | Profile excluded | Default not run |
| --- | ---: | ---: | ---: | ---: | ---: |
| `insn/evaluate` | 57 | 0 | 0 | 57 | 0 |

The conserved XSLT30 denominator total is now 3,498 cases: 488 passed
comparisons, 12 engine-unsupported cases, 2,878 profile exclusions, and 120
visible default not-run cases across 112 complete test sets.

## Claim boundary

This is a profile disposition, not execution or passing evidence. FastXSLT
does not claim `xsl:evaluate`, dynamic compilation, dynamic static-context
construction, or its associated error behavior. An individual case may be
promoted only after a profile decision and executable semantic evidence
explicitly override the inherited exclusion.
