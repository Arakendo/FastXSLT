# QT3 IRI-To-URI Denominator

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- The complete unchanged 47-case `fn/iri-to-uri.xml` test set.
- A first-party typed denominator overlay and explicit selected-case ledger.

## Method

A private source-free evaluator preserves the URI-permitted printable ASCII
characters and percent-encodes spaces, controls, non-ASCII UTF-8 bytes, and the
excluded ASCII characters identified by the function specification. It retains
existing percent escapes rather than double-encoding them.

The selected tranche covers:

- empty string and empty-sequence conversion;
- each permitted or excluded ASCII punctuation class represented upstream;
- non-ASCII literals and bounded codepoint ranges `32 to 294` and
  `15000 to 16000`;
- XPath doubled-quote string escaping and embedded newlines;
- `xs:anyURI` and `xs:untypedAtomic` promotion;
- equality composition plus native `XPST0017` and `XPTY0004` diagnostics.

Every selected case executes the unchanged QT3 expression and its native
assertion shape. Evaluation charges the XPath-operation work domain.

## Conserved result

| Disposition | Cases |
| --- | ---: |
| Selected and passed | 45 |
| Profile excluded | 1 |
| Visible default not run | 1 |
| **Total** | **47** |

`fn-iri-to-uri-18` carries an explicit `XQ10+` dependency and remains outside
the XPath-in-XSLT profile. `K-IRIToURIfunc-4` exercises `current-time()`,
sequence indexing, and `normalize-space()` around the URI call, so it remains
visibly unexecuted rather than receiving an inferred result.

Across all complete QT3 overlays, the conserved subtotal is now 1,146 cases:
896 passes, 195 profile exclusions, and 55 visible default not-run cases.

## Boundary

This evidence does not claim general constructors, function dispatch,
arbitrary expression composition, invocation clocks, sequence indexing, or
whitespace normalization. Codepoint construction is bounded by the literal
ranges in the selected cases. The evaluator remains a private semantic slice
and does not select a public URI API.
