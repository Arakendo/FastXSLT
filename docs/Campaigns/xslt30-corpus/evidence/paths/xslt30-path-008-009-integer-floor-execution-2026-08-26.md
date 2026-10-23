# XSLT30 `path-008/009` Integer-Floor Execution Evidence

| Field | Value |
| --- | --- |
| Date | 2026-08-26 |
| Suite | W3C XSLT 3.0 test suite |
| Revision | `6f8fd9e966ae74a251a2604abef9d904c7bc5c9b` |
| Native identities | test set `path`, cases `path-008` and `path-009` |
| Dependency | `XSLT10+` |
| Result assertions | native `assert-xml` |
| Outcome | Both passed through the private reference path |

## Executed behavior

The unmodified positional expressions are:

```xpath
element1[(((((2*10)-4)+9) div 5) mod floor(3))]
element1[floor(2)]
```

Both cases use the catalog's inline principal source and the shared bounded
resource-admission path. The private constant-integer parser now recognizes
`floor(expression)`. Because every admitted operand and intermediate value is
an integer, `floor` returns that integer unchanged. The resulting value `2` is
applied as a position over the name-matched `element1` sequence.

Focused parser tests cover direct, nested-arithmetic, and parenthesized uses.
Other function names remain unsupported. Decimal literals and fractional
division also remain unsupported, so this addition does not silently claim the
rounding behavior of the broader XPath numeric type system.

## Denominator effect

| Disposition | Count |
| --- | ---: |
| Selected and passed | 9 |
| Engine unsupported | 1 |
| Total | 10 |

No membership or exclusion changed.

## Claim boundary

This establishes `floor()` only as the identity operation within the admitted
constant-integer subset. It does not establish decimal or double values,
NaN/infinity behavior, function resolution, general function calls, or XPath
numeric conformance.

