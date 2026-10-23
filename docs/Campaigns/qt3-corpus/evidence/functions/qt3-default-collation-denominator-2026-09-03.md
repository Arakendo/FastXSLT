# QT3 Default-Collation Denominator

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- The complete unchanged seven-case `fn/default-collation.xml` test set.
- A first-party typed denominator overlay and explicit selected-case ledger.

## Method

The original denominator admission used a private test-only source-free
evaluator. Following Finding 5 of the second adversarial review, all seven cases
now use a compiled typed `DefaultCollationExpression` in the production XSLT
path. The QT3 adapter performs only catalog selection, wrapper construction, and
result/error comparison.

Each unchanged QT3 expression is placed in an `xsl:value-of` `select` attribute,
then passes through the ordinary XML parser, stylesheet compiler, value
evaluator, result tree, and text serializer. The same compiled expression also
executes through `ExperimentalEngine`, the engine used by the ASP.NET workbench.
Static arity failures occur during production stylesheet compilation with
`XPST0017`; successful evaluation charges the XPath-operation work domain.

The production evaluator supplies the standard Unicode codepoint collation URI
as its explicit static-context default. The value does not depend on process
locale or host environment. Equality, count, and effective-boolean-value forms
retain distinct compiled and evaluated value kinds.

Every case executes the unchanged QT3 expression and its native assertion
shape.

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
It establishes a production-parity migration pattern for one complete family;
it does not retroactively place other test-only QT3 families on the production
path.
