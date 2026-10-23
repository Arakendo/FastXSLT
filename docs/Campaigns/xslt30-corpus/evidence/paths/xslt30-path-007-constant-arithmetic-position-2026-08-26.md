# XSLT30 `path-007` Constant-Arithmetic Position Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Suite | W3C XSLT 3.0 test suite |
| Revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Native identity | test set `path`, case `path-007` |
| Dependency | `XSLT10+` |
| Result assertion | native `assert-xml` |
| Outcome | Passed through the private reference path |

## Executed behavior

The unmodified stylesheet expression is:

```xpath
element1[(((((2*10)-4)+9) div 5) mod 3 )]
```

The catalog's inline principal source and file-backed stylesheet now pass
through the same bounded resource-admission helper used by the file-backed path
cases. Compilation and execution consume only the sealed snapshot.

A private checked constant-integer parser evaluates parentheses and XPath-style
multiplicative/additive precedence for integer literals, `*`, `+`, `-`, exact
`div`, and non-negative `mod`. The native expression evaluates to `2`; the
numeric predicate therefore selects the second `element1` in the step's
name-matched sequence, not the second raw child node.

The parser rejects overflow, zero division, fractional division, negative-mod
semantics, functions, and other unadmitted tokens rather than silently using
host arithmetic with different semantics. It lives in a separate private source
unit because numeric expression parsing is a distinct responsibility from node
navigation and predicate application.

## Denominator effect

| Disposition | Count |
| --- | ---: |
| Selected and passed | 7 |
| Engine unsupported | 3 |
| Total | 10 |

No membership or exclusion changed.

## Claim boundary

This is constant checked-integer arithmetic sufficient for one positional
predicate. It does not establish XPath decimal, double, unary, function,
variable, context-position, general effective-boolean-value, or numeric-error
semantics, and it is not an XPath conformance claim.

