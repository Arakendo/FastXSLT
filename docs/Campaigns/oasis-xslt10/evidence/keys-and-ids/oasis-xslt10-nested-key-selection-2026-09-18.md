# OASIS XSLT 1.0 Nested Key Selection

## Question

Can one typed XSLT 1.0 `key()` lookup use the node strings selected by another
typed lookup without introducing unbounded expression recursion or an index?

## Method

- Add a recursive key-value plan whose recursive edge is boxed and whose
  compile-time nesting depth is limited to four.
- Compile the complete inner lookup, including its existing predicate and path-
  tail semantics, through the same typed compiler.
- Execute the inner lookup through the complete charged reference selector and
  convert every selected node to its controlled string value.
- Use those strings as the outer lookup values; charge the outer complete scan
  independently.
- Include the nested plan in exact prepared-program capacity accounting.
- Test multiple inner values selecting multiple outer nodes, then run the
  unchanged, hash-verified 3,173-case OASIS CD04 catalog.

## Result

The focused case selects inner marker nodes `a` and `c`, then uses both values
to select the outer items in document order as `AC`.

The unchanged case `Lotus/idkey_idkey21#1` now executes and compares exactly.
The strict exact-result lower bound rises from 1,612 to 1,613. Initialized cases
rise from 2,002 to 2,003 and successfully executed cases rise from 1,897 to
1,898. Initialization failures fall from 1,133 to 1,132. Execution failures,
comparison mismatches, and expected-error totals remain unchanged.

## Boundary conclusion

Nested lookup is a bounded composition of the existing safe charged reference
selector. It does not add a retained index, cross-document key context, dynamic
key names, arbitrary lookup-value expressions, or unbounded recursive plans.

The result is bounded XSLT 1.0 compatibility evidence, not a general `key()` or
conformance claim.
