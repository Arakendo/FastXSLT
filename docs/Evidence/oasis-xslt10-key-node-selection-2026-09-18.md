# OASIS XSLT 1.0 Key Node Selection

## Question

Can the charged literal `key()` reference scan become a reusable node selection
for `xsl:for-each` and `xsl:apply-templates` without duplicating lookup
semantics, selecting an index, or widening dynamic key arguments?

## Method

- Extract the complete charged lookup from value conversion into a private
  runtime owner that returns source `NodeId` values.
- Retain the existing `xsl:value-of` behavior as XSLT 1.0 first-node string
  conversion over that node sequence.
- Add the typed lookup to the existing private apply-selection plan shared by
  `xsl:for-each` and `xsl:apply-templates`.
- Preserve additive same-name declarations, full source traversal, work and
  cancellation charge points, optional-tail document-order normalization,
  source node identity, runtime `XTDE1260`, and exact retained-capacity
  accounting.
- Exercise both consumers in one focused case with three matches from two key
  declarations, checking document order and `position()` / `last()` focus.
- Run the unchanged, hash-verified 3,173-case OASIS CD04 catalog.

## Result

The focused `for-each` and `apply-templates` paths produce the same ordered
three-node focus and serialize the expected result.

The strict exact-result lower bound rises from 1,587 to 1,591. Initialized
cases rise from 1,971 to 1,979 and successfully executed cases rise from 1,868
to 1,875. Initialization failures fall from 1,164 to 1,156. One expected-error
case moves from initialization to execution because its lookup now reaches the
runtime undeclared-key check; the combined expected-error total remains 423.

## Boundary conclusion

This is a shared node-selection reference path, not an index. It does not retain
source-derived lookup state, add cross-invocation sharing, admit dynamic key
names or values, define a public node-set API, or add `key()` to every
expression consumer. `xsl:copy-of` and other consumers remain separate typed
admission work.

The result is bounded XSLT 1.0 compatibility evidence, not a general `key()` or
conformance claim.
