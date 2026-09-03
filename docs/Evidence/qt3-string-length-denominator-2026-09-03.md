# QT3 String-Length Denominator

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- The complete unchanged 36-case `fn/string-length.xml` test set.
- A first-party typed denominator overlay and explicit selected-case ledger.

## Method

A private source-free evaluator retains empty sequence, string, integer, and
boolean values as distinct types. This avoids the incorrect shortcut of
reducing a string to its effective boolean value before measuring it.
`fn:string-length` counts Rust `char` values, which correspond to Unicode scalar
values for the admitted literals rather than UTF-8 bytes or UTF-16 code units.

The selected tranche covers:

- literal and explicitly constructed strings, including empty and non-BMP
  strings;
- the empty sequence converted to the optional-string default;
- integer addition, equality, and `xs:integer` type assertions;
- `fn:string`, `fn:concat`, `fn:boolean`, `fn:not`, and boolean `and`
  composition;
- lazy conditional evaluation whose unselected branch requires an absent
  context item;
- native `XPST0017` invalid-arity and `XPDY0002` absent-context diagnostics.

Every selected case executes the unchanged QT3 expression and its native
assertion shape. Evaluation charges the XPath-operation work domain.

## Conserved result

| Disposition | Cases |
| --- | ---: |
| Selected and passed | 30 |
| Profile excluded | 3 |
| Visible default not run | 3 |
| **Total** | **36** |

The exclusions are the two schema-validation cases and the higher-order
function-item case. The visible defaults are `fn-string-length-19`,
`fn-string-length-24`, and `fn-string-length-25`, which require document
atomization/cardinality or range-predicate focus semantics not admitted by this
source-free evaluator.

Across all complete QT3 overlays, the conserved subtotal is now 1,036 cases:
787 passes, 194 profile exclusions, and 55 visible default not-run cases.

## Boundary

This evidence does not claim the zero-argument context form generally,
document atomization, schema-aware typed values, function-item conversion, or
general predicate focus. The evaluator is a private semantic slice, not a
public XPath API or a second execution backend.
