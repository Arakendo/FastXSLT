# QT3 Codepoint-Equal Denominator

Date: 2026-09-03

## Inputs

- QT3 revision `83993587711dbd5c18ed846385ec37d079d6e492`.
- The complete unchanged 36-case `fn/codepoint-equal.xml` test set.
- A first-party typed denominator overlay and explicit selected-case ledger.

## Method

The private source-free case-conversion evaluator now retains an explicit empty
sequence and compares two optional strings by Unicode scalar sequence. If
either operand is empty, `fn:codepoint-equal` returns the empty sequence. An
atomic value other than a string produces the native `XPTY0004` boundary.

The selected tranche covers:

- equal and unequal empty, singleton, and multi-codepoint strings;
- empty-sequence propagation from either argument position;
- strict rejection of integer arguments;
- `fn:string`, `xs:integer`, `xs:boolean`, lower/upper case, `not`, `and`, and
  `or` composition;
- native `XPST0017` invalid-arity diagnostics.

Every selected case executes the unchanged QT3 expression and its native
assertion shape. Evaluation charges the XPath-operation work domain.

## Conserved result

| Disposition | Cases |
| --- | ---: |
| Selected and passed | 23 |
| Profile excluded | 6 |
| Visible default not run | 7 |
| **Total** | **36** |

The six `cbcl-codepoint-equal-*` cases declare local XQuery functions and remain
outside the XPath-in-XSLT profile. `fn-codepoint-equal-22` requires Unicode
normalization, while the six `K2-CodepointEqual-*` cases compose an invocation
clock into their arguments; all seven remain visibly unexecuted.

Across all complete QT3 overlays, the conserved subtotal is now 1,239 cases:
974 passes, 203 profile exclusions, and 62 visible default not-run cases.

## Boundary

This evidence does not claim Unicode normalization, invocation clocks, local
XQuery functions, or general XPath function composition. Comparison uses the
already produced string values and does not select a public collation API.
