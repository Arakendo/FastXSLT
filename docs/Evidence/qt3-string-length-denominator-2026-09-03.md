# QT3 String-Length Denominator

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- The complete unchanged 36-case `fn/string-length.xml` test set.
- A first-party typed denominator overlay and explicit selected-case ledger.

## Method

All 33 selected unchanged expressions now compile through generated XSLT and
execute through the ordinary runtime and serializer. The compiled form keeps
source-free evaluation distinct from its one document-path expression; the
latter uses the real QT3 context document and reports its runtime cardinality
error. A workbench sentinel reaches the same production expression path.

The shared evaluator retains empty sequence, string, integer, and boolean values
as distinct types. This avoids the incorrect shortcut of
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
| Selected and passed | 33 |
| Profile excluded | 3 |
| Visible default not run | 0 |
| **Total** | **36** |

The exclusions are the two schema-validation cases and the higher-order
function-item case. A second tranche closes the original three visible
defaults: `fn-string-length-19` evaluates its unchanged document path and
reports `XPTY0004` for the multi-node optional-string argument;
`fn-string-length-24` evaluates zero-argument string length against each
integer focus item; and `fn-string-length-25` reports `XPTY0004` when that
integer is supplied explicitly to the optional-string argument.

Across all complete QT3 overlays, the conserved subtotal is now 1,036 cases:
790 passes, 194 profile exclusions, and 52 visible default not-run cases.

## Boundary

This evidence does not claim the zero-argument context form generally,
general document atomization, schema-aware typed values, function-item
conversion, or general predicate focus. The document and integer-range forms
are deliberately bounded to the exercised semantics. The evaluator is a
production semantic slice, not a public XPath API or a second execution backend.
