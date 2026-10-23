# OASIS XSLT 1.0 Concatenated Key Name

## Question

Can the runtime key-name plan reuse the existing typed XSLT 1.0 `concat()`
evaluator without admitting arbitrary context-dependent name expressions?

## Method

- Admit a key name expressed as `concat()` only when every typed part is a
  literal or unqualified variable reference.
- Retain the typed concatenation plan and the immutable call-site namespace
  slice in exact prepared-program capacity accounting.
- Evaluate the concatenation through the existing charged XSLT 1.0 string
  conversion owner, then use the same lexical-QName validation and namespace
  resolution as a single variable name.
- Test literal/variable composition, then run and trace the unchanged, hash-
  verified 3,173-case OASIS CD04 catalog.

## Result

The focused case composes `keyspace`, resolves the declared key, and selects the
expected source node.

The unchanged case `Microsoft/Keys__91727#1` leaves initialization failure and
executes successfully with the expected selected source value and variable
values. It then exposes an independent HTML-indentation whitespace mismatch.
Initialized cases rise from 2,004 to 2,005, successful executions rise from
1,899 to 1,900, initialization failures fall from 1,131 to 1,130, and visible
comparison mismatches rise from 214 to 215. The strict exact-result lower bound
correctly remains 1,614.

## Boundary conclusion

The admitted concatenation contains only literals and variable references. It
does not admit paths, sums, arbitrary function calls, source-derived namespace
context, new authority, or a retained key index. The later serialization
mismatch is not hidden or credited to this semantic slice.

The result is bounded XSLT 1.0 compatibility evidence, not a general dynamic
QName, `key()`, serialization, or conformance claim.
