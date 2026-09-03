# QT3 Escape-HTML-URI Denominator

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- The complete unchanged 34-case `fn/escape-html-uri.xml` test set.
- A first-party typed denominator overlay and explicit selected-case ledger.

## Method

A private source-free evaluator recognizes the standard one-argument
`fn:escape-html-uri` call, preserves printable ASCII characters, and
percent-encodes every other Unicode scalar value as its UTF-8 bytes using
uppercase hexadecimal digits. The same escaping implementation remains used by
the existing compile-time literal fold; the QT3 adapter adds bounded expression
and assertion handling without creating a public XPath API.

The selected tranche covers:

- empty string and empty-sequence conversion;
- printable ASCII preservation, including URI punctuation and spaces;
- non-ASCII BMP characters and a codepoint-constructed control/non-ASCII value;
- equality composition used by the native boolean assertions;
- native `XPST0017` invalid-arity and `XPTY0004` invalid-argument diagnostics.

Every selected case executes the unchanged QT3 expression and its native
assertion shape. Evaluation charges the XPath-operation work domain.

## Conserved result

| Disposition | Cases |
| --- | ---: |
| Selected and passed | 33 |
| Profile excluded | 0 |
| Visible default not run | 1 |
| **Total** | **34** |

The sole visible default, `K-EscapeHTMLURIFunc-6`, exercises
`current-time()`, `iri-to-uri()`, sequence indexing, `treat as`, and
`normalize-space()` rather than the escape operation. It remains explicitly
unexecuted instead of receiving an inferred result or an unrelated exclusion.

Across all complete QT3 overlays, the conserved subtotal is now 1,070 cases:
823 passes, 194 profile exclusions, and 53 visible default not-run cases.

## Boundary

This evidence does not claim general function dispatch, arbitrary expression
composition, invocation clocks, `iri-to-uri`, sequence indexing, `treat as`, or
whitespace normalization. The evaluator is a private semantic slice and the
production literal fold remains deliberately narrower than the test adapter.
