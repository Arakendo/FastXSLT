# QT3 Encode-For-URI Denominator

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- The complete unchanged 29-case `fn/encode-for-uri.xml` test set.
- A first-party typed denominator overlay and explicit selected-case ledger.

## Method

The denominator was initially admitted through a private source-free evaluator.
It now runs every selected unchanged expression through generated XSLT and the
ordinary XML parser, stylesheet compiler, typed expression plan, runtime,
result tree, and text serializer. A separate workbench sentinel executes the
same typed expression through the ASP.NET-facing experimental engine.

The production expression retains the URI unreserved ASCII characters `A-Z`,
`a-z`, `0-9`, `-`, `_`, `.`, and `~`. Every other byte of the UTF-8 encoding is
percent-encoded using uppercase hexadecimal digits. It accepts the selected
outer-parentheses and two-argument `concat` forms and insignificant whitespace
between a function name and its argument list. The QT3 adapter now owns only
catalog selection, wrapper construction, and assertion comparison; it does not
evaluate the function or manufacture the reported error.

The selected tranche covers:

- empty string and empty-sequence conversion;
- all URI punctuation isolated by the native cases;
- spaces, percent signs, and non-ASCII UTF-8 input;
- two-argument `concat` and equality composition;
- native `XPST0017` invalid-arity and `XPTY0004` invalid-argument diagnostics.

Every selected case executes the unchanged QT3 expression and its native
assertion shape. Invalid arity and argument type are reported by production
compilation as `XPST0017` and `XPTY0004`. Evaluation charges the XPath-operation
work domain.

## Conserved result

| Disposition | Cases |
| --- | ---: |
| Selected and passed | 28 |
| Profile excluded | 0 |
| Visible default not run | 1 |
| **Total** | **29** |

The sole visible default, `K-EncodeURIfunc-6`, exercises `current-time()`,
sequence indexing, `treat as`, and `normalize-space()` around the URI call. It
remains explicitly unexecuted instead of receiving an inferred result.

Across all complete QT3 overlays, the conserved subtotal is now 1,099 cases:
851 passes, 194 profile exclusions, and 54 visible default not-run cases.

## Boundary

This evidence does not claim general function dispatch, arbitrary expression
composition, invocation clocks, sequence indexing, `treat as`, or whitespace
normalization. The production expression is a private typed semantic slice for
the selected forms and does not select a public URI encoding API or widen the
one visible not-run case.
