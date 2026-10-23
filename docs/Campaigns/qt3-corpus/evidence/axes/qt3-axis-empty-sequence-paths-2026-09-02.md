# QT3 empty-sequence paths -- 2026-09-02

## Result

FastXSLT now executes unchanged QT3 `K2-Axes-55` and `K2-Axes-56` from the
pinned `prod/AxisStep.xml` test set. The private path representation recognizes
an explicit empty-sequence origin before lowering the requested attribute or
child step. Evaluation begins with no context nodes and therefore produces an
empty sequence without inspecting the document supplied to the private
adapter. The native permitted `assert-true` alternative is retained and
executed through `empty(...)`.

## Verification

- A focused control evaluates both attribute and child paths over `()` against
  a deliberately nonempty document and records zero XPath node visits.
- The QT3 adapter reads each expression and native assertion from the immutable
  upstream set before execution.
- Both case identities are explicit `selected/passed` private-ledger records.
- The AxisStep denominator advances from 204 to 206 passes while its visible
  defaults fall from 33 to 31. The combined 612-case subtotal is conserved as
  389 passes, 179 profile exclusions, and 44 visible defaults.

## Boundary

This slice does not admit general sequence expressions, optional absent dynamic
contexts, static typing, atomic path operands, namespace nodes, or a general
`fn:empty` implementation. It only establishes that the two selected path
forms have an explicit empty input and cannot acquire nodes from an unrelated
document context.
