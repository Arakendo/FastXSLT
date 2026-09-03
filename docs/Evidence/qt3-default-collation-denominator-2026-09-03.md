# QT3 Default-Collation Denominator

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- The complete unchanged seven-case `fn/default-collation.xml` test set.
- A first-party typed denominator overlay and explicit selected-case ledger.

## Method

The private source-free evaluator supplies the standard Unicode codepoint
collation URI as its explicit static-context default. The value does not depend
on process locale or host environment. Existing bounded equality, count, and
effective-boolean-value composition evaluates the native surrounding
expressions.

Every case executes the unchanged QT3 expression and its native assertion
shape. Evaluation charges the XPath-operation work domain.

## Conserved result

| Disposition | Cases |
| --- | ---: |
| Selected and passed | 7 |
| Profile excluded | 0 |
| Visible default not run | 0 |
| **Total** | **7** |

Across all complete QT3 overlays, the conserved subtotal is now 1,285 cases:
1,014 passes, 207 profile exclusions, and 64 visible default not-run cases.

## Boundary

This evidence selects the codepoint collation only for the admitted private
static-context slice. It does not define a public static-context API, admit
host-defined collations, or settle future default-collation configurability.
