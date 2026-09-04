# QT3 Hour and Minute Duration Denominators

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- The complete unchanged 31-case `fn/hours-from-duration.xml` test set.
- The complete unchanged 32-case `fn/minutes-from-duration.xml` test set.
- First-party typed denominator overlays and explicit selected-case ledger
  records.

## Method

The private production duration-component expression derives normalized hour and minute
components from its signed whole-second representation. Division and remainder
normalize oversized lexical fields such as `PT123H` and `P21DT10H65M`; signed
general durations preserve the sign on each extracted component. The shared
adapter compiles each unchanged expression inside `xsl:value-of`, executes the
ordinary runtime, serializes its result, and checks the unchanged boolean,
integer, string, empty-sequence, arithmetic, comparison, optional-type, and
arity-error assertions. A negative-duration workbench sentinel reaches the
same expression through the host-facing engine path.

Every case executes its unchanged QT3 expression and native assertion shape.
Evaluation charges the XPath-operation work domain.

## Conserved result

| Test set | Selected and passed | Profile excluded | Visible default not run | Total |
| --- | ---: | ---: | ---: | ---: |
| `fn/hours-from-duration.xml` | 31 | 0 | 0 | 31 |
| `fn/minutes-from-duration.xml` | 32 | 0 | 0 | 32 |
| **Total** | **63** | **0** | **0** | **63** |

Across all complete QT3 overlays, the conserved subtotal is now 1,441 cases:
1,170 passes, 207 profile exclusions, and 64 visible default not-run cases.

## Boundary

This evidence does not admit the seconds component, which requires exact
fractional decimal retention rather than this integer-component slice. It does
not establish complete duration lexical validation, a public duration type, or
general duration arithmetic.
