# OASIS XSLT 1.0 Key Attribute Predicate

## Question

Can the shared XSLT 1.0 key node selector apply an exact literal
attribute-equality predicate while preserving ordered selection, accounting,
and composed key declarations?

## Method

- Add one typed key predicate for `[@QName='literal']`, resolving the attribute
  QName against the stylesheet's static namespace context.
- Apply the predicate to the ordered key result before any optional path tail.
- Charge every inspected source attribute as an XPath node visit.
- Retain the predicate name and literal in exact prepared-program capacity
  accounting.
- Test the same filtered selection through `xsl:for-each` and
  `count(key(...)[...])`, then run the unchanged, hash-verified 3,173-case OASIS
  CD04 catalog.

## Result

The focused case preserves source order and produces the same two-node filtered
sequence through both consumers.

The unchanged composed-module case
`Microsoft/Keys_MultipltKeysInclude#1` now executes and compares exactly as
`<out>4040</out>`.

The strict exact-result lower bound rises from 1,608 to 1,609. Initialized cases
rise from 1,998 to 1,999 and successfully executed cases rise from 1,893 to
1,894. Initialization failures fall from 1,137 to 1,136. Execution failures,
comparison mismatches, and expected-error totals remain unchanged.

## Boundary conclusion

The admitted predicate is one literal attribute equality over an already
selected key node sequence. It does not admit arbitrary predicates, variable
comparisons, boolean composition, nested filters, dynamic key names,
cross-document key context, or a retained index.

The result is bounded XSLT 1.0 compatibility evidence, not a general `key()` or
conformance claim.
