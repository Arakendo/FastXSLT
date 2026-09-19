# OASIS XSLT 1.0 Generated Key Identity AVT

## Question

Can a literal-result attribute compose stable source-node identity with the
first node selected by an admitted `key()` call without adding a general AVT
expression evaluator?

## Method

- Add one typed AVT value for `generate-id(key(...))`, with optional literal
  text before and after the expression.
- Compile the inner lookup through the existing literal-name XSLT 1.0 key plan.
- Execute the complete charged key selector, take the first node in document
  order, and format it through the existing stable principal-source identity
  owner.
- Preserve empty-string behavior for an empty selected sequence and exact
  prepared-program capacity accounting.
- Test a matching and absent context-derived lookup, then run the unchanged,
  hash-verified 3,173-case OASIS CD04 catalog and trace the two motivating
  Microsoft cases.

## Result

The focused case produces a stable identity for the first selected key node and
an empty identity for an absent key value.

The conserved OASIS totals remain unchanged at 1,608 exact expected-result
matches, 1,998 initialized cases, and 1,893 successful executions. The two
motivating cases, `Microsoft/Keys__91832#1` and
`Microsoft/Keys__91833#1`, now compile their `generate-id(key('func', .))`
AVTs but remain in initialization failure at a separate zero-argument
`generate-id()` AVT. That form is not admitted by this slice.

## Boundary conclusion

This is one statically recognized composition of two existing semantic owners.
It does not admit general function calls in AVTs, zero-argument `generate-id()`,
dynamic key names, nested key calls, cross-document key context, or a retained
index.

The result is bounded XSLT 1.0 compatibility evidence, not a conformance claim.
