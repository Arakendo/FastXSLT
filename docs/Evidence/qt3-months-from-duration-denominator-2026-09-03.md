# QT3 Months-From-Duration Denominator

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- The complete unchanged 31-case `fn/months-from-duration.xml` test set.
- A first-party typed denominator overlay and explicit selected-case ledger.

## Method

The private bounded duration-component evaluator reuses the signed total-month
representation established by the adjacent years-from-duration denominator and
derives the normalized month component using signed integer remainder by
twelve. The shared QT3 adapter executes native true, false, integer, string,
and arity-error assertions without substituting fixture-specific results.

Every case executes its unchanged QT3 expression and native assertion shape.
Evaluation charges the XPath-operation work domain.

## Conserved result

| Disposition | Cases |
| --- | ---: |
| Selected and passed | 31 |
| Profile excluded | 0 |
| Visible default not run | 0 |
| **Total** | **31** |

Across all complete QT3 overlays, the conserved subtotal is now 1,347 cases:
1,076 passes, 207 profile exclusions, and 64 visible default not-run cases.

## Boundary

This result has the same private representation boundary as the adjacent
years-from-duration evidence. It does not establish a public duration type,
complete duration lexical validation, general duration arithmetic, or support
for day, hour, minute, and second component functions.
