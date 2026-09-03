# XSLT30 Schema-Aware Expression Profile Denominators

Date: 2026-09-03

## Inputs

- W3C XSLT 3.0 test-suite revision
  `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b`.
- Complete four-case `tests/expr/treat-as/_treat-as-test-set.xml`.
- Complete four-case `tests/expr/type-expr/_type-expr-test-set.xml`.

## Method

Each native test set declares `feature="schema_aware"` as a set-level
dependency. A typed first-party overlay records the immutable suite revision,
test-set path, exact case count, excluded profile feature, nonexecution status,
and rationale. The verifier parses both overlays with unknown-field denial,
parses both pinned native catalogs, proves each case name is unique, and proves
the exclusion rule is present at the enclosing test-set level.

The rule therefore covers every catalog case through native inherited metadata;
it is not inferred from stylesheet filenames, keywords, or expected results.

## Result

| Test set | Cases | Passed | Engine unsupported | Profile excluded | Default not run |
| --- | ---: | ---: | ---: | ---: | ---: |
| `expr/treat-as` | 4 | 0 | 0 | 4 | 0 |
| `expr/type-expr` | 4 | 0 | 0 | 4 | 0 |
| **Total** | **8** | **0** | **0** | **8** | **0** |

The conserved XSLT30 denominator total is now 683 cases: 488 passed
comparisons, 12 engine-unsupported cases, 63 profile exclusions, and 120
visible default not-run cases across 19 complete test sets.

## Claim boundary

These are profile dispositions, not executions or passing results. FastXSLT
does not claim schema import, source validation, user-defined schema types,
schema-aware casting, `treat as`, or the assertion families used by these
cases. An individual case may be promoted only if the selected profile changes
or native metadata establishes that it does not require schema-aware behavior;
that promotion must override the inherited rule explicitly.
