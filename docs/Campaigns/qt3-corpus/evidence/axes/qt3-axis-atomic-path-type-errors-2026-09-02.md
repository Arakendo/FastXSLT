# QT3 atomic path type errors -- 2026-09-02

## Result

FastXSLT now executes ten unchanged QT3 AxisStep negative cases whose path or
axis context is statically known to be atomic. A private recognition seam above
the node-only location-path parser reports `XPTY0019` when an atomic sequence is
the left operand of `/`, and `XPTY0020` when an axis step directly receives an
atomic context item. It retains structured source location and nonempty detail.

The admitted cases are `K2-Axes-38`, `K2-Axes-39`, `K2-Axes-50`,
`K2-Axes-53`, and `statictypingaxis-1` through `statictypingaxis-6`. For the two
predicate-context cases, the adapter verifies that the native `any-of`
assertion permits the selected `XPTY0020` result. Every other case requires the
exact upstream `XPTY0019` assertion.

## Conservation

All ten identities are explicit `selected/passed` records and are verified
against the immutable parent set. AxisStep advances from 206 to 216 passes and
its visible defaults fall from 31 to 21. The combined 612-case subtotal remains
conserved as 399 passes, 179 profile exclusions, and 34 visible defaults.

## Boundary

Recognition is deliberately restricted to integer literals, parenthesized
integer sequences, and the positional filters required by these cases. It does
not claim a general static type system, runtime mixed node/atomic sequence
checking, arbitrary predicate evaluation, or general XPath parsing. Expressions
that are not provably within the bounded grammar remain unclassified rather
than being mislabeled as type errors.
