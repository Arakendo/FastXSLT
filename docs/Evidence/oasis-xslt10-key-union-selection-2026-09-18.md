# OASIS XSLT 1.0 Key Union Selection

## Question

Can the shared XSLT 1.0 apply-selection path combine several `key()` node
selections while preserving XPath union order and identity semantics?

## Method

- Admit a top-level union only when every alternative is an already supported
  typed `key()` lookup.
- Bound the private plan to at most eight alternatives.
- Evaluate every alternative through the complete charged key-selection
  reference path.
- Concatenate the selected nodes, restore principal-source document order, and
  remove duplicate node identities before establishing template focus.
- Retain every lookup in exact prepared-program capacity accounting.
- Test reversed alternatives with an overlapping node, then run the unchanged,
  hash-verified 3,173-case OASIS CD04 catalog.

## Result

The focused case produces `abc` in document order even though the union lists
the later-selecting key first and both keys select the middle node.

The unchanged case `Lotus/select_select55#1` now executes and compares exactly.
The strict exact-result lower bound rises from 1,609 to 1,610. Initialized cases
rise from 1,999 to 2,000 and successfully executed cases rise from 1,894 to
1,895. Initialization failures fall from 1,136 to 1,135. Execution failures,
comparison mismatches, and expected-error totals remain unchanged.

## Boundary conclusion

This is a bounded union of independently evaluated key selections in the
existing `xsl:apply-templates` / `xsl:for-each` selection owner. It does not
admit mixed key/path unions, arbitrary union expressions, retained indexes,
cross-document key context, or a public node-set representation.

The result is bounded XSLT 1.0 compatibility evidence, not a general union,
`key()`, or conformance claim.
