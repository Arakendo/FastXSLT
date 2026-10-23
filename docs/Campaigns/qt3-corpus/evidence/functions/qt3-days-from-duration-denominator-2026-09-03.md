# QT3 Days-From-Duration Denominator

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- The complete unchanged 31-case `fn/days-from-duration.xml` test set.
- A first-party typed denominator overlay and explicit selected-case ledger.

## Method

The private production duration representation retains signed total months and signed
whole seconds. Construction normalizes oversized hour fields before the day
component is derived by division by 86,400. Fractional lexical seconds are
ignored after their whole-second contribution because they cannot affect this
integer component; this narrow representation does not claim to retain them.
The shared adapter compiles every unchanged expression inside `xsl:value-of`,
executes the ordinary runtime, serializes its result, and checks true, false,
integer, string, empty-sequence, composition, and arity-error assertions.

Runtime evaluation charges the XPath-operation work domain. The shared
workbench sentinel covers the same production duration-component path.

## Conserved result

| Disposition | Cases |
| --- | ---: |
| Selected and passed | 31 |
| Profile excluded | 0 |
| Visible default not run | 0 |
| **Total** | **31** |

Across all complete QT3 overlays, the conserved subtotal is now 1,378 cases:
1,107 passes, 207 profile exclusions, and 64 visible default not-run cases.

## Boundary

This evidence admits only the component extraction and lexical shapes exercised
by this denominator. It does not establish complete duration lexical
validation, arbitrary-precision duration storage, a public duration type, or
general duration arithmetic.
