# QT3 Years-From-Duration Denominator

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- The complete unchanged 31-case `fn/years-from-duration.xml` test set.
- A first-party typed denominator overlay and explicit selected-case ledger.

## Method

A bounded, source-free production expression parses the year and month
components of `xs:yearMonthDuration`, `xs:duration`, and
`xs:dayTimeDuration` lexical values into signed total months. It derives the
normalized year component using integer division by twelve. Existing private
composition handles empty sequences, count/empty/average, arithmetic, integer
comparisons, optional-integer instance tests, and the native `XPST0017` arity
diagnostic.

Every unchanged QT3 expression is compiled inside a generated `xsl:value-of`,
executed by the ordinary XSLT runtime, serialized as text, and compared with
its native assertion shape. Runtime evaluation charges the XPath-operation
work domain. A workbench sentinel proves the same compiled expression reaches
the host-facing engine path.

## Conserved result

| Disposition | Cases |
| --- | ---: |
| Selected and passed | 31 |
| Profile excluded | 0 |
| Visible default not run | 0 |
| **Total** | **31** |

Across all complete QT3 overlays, the conserved subtotal is now 1,316 cases:
1,045 passes, 207 profile exclusions, and 64 visible default not-run cases.

## Boundary

This evidence admits only the private typed duration construction and
year-component
expressions exercised by this denominator. It does not establish a public
duration representation, general duration arithmetic, complete lexical
validation beyond the admitted forms, implicit timezone semantics, or the
seconds component function.
