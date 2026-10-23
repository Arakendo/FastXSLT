# QT3 Months-From-Duration Denominator

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- The complete unchanged 31-case `fn/months-from-duration.xml` test set.
- A first-party typed denominator overlay and explicit selected-case ledger.

## Method

The private bounded production expression reuses the signed total-month
representation established by the adjacent years-from-duration denominator and
derives the normalized month component using signed integer remainder by
twelve. The shared QT3 adapter compiles every unchanged expression inside
`xsl:value-of`, executes the ordinary runtime, serializes its result, and checks
native true, false, integer, string, and arity-error assertions without
substituting fixture-specific results.

Runtime evaluation charges the XPath-operation work domain. The shared
workbench sentinel covers the same production duration-component path.

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

This result has the same private production representation boundary as the adjacent
years-from-duration evidence. It does not establish a public duration type,
complete duration lexical validation beyond the admitted forms, general
duration arithmetic, or support for the seconds component function.
