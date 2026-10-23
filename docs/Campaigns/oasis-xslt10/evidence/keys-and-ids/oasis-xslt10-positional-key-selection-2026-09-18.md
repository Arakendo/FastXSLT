# OASIS XSLT 1.0 Positional Key Selection

## Question

Can the private `key()` node-selection plan preserve XSLT 1.0 positional
predicate semantics before an optional path tail without admitting a general
predicate evaluator?

## Method

- Add a typed optional predicate to the private key lookup plan.
- Admit exact integer positions, `position()=N`, `last()`, and
  `last()=position()`.
- Apply the predicate to the document-ordered key result before evaluating an
  optional tail, matching XPath filter-expression ordering.
- Reuse the same plan from value conversion, `xsl:for-each`,
  `xsl:apply-templates`, and `xsl:copy-of`.
- Extend the focused cross-consumer oracle with first, middle, and last
  selection and run the unchanged, hash-verified 3,173-case OASIS CD04 catalog.

## Result

The focused case preserves the filtered focus and first/last node identity.
Thirteen unchanged OASIS cases initialize, execute, and compare exactly.

The strict exact-result lower bound rises from 1,592 to 1,605. Initialized
cases rise from 1,980 to 1,993 and successfully executed cases rise from 1,876
to 1,889. Initialization failures fall from 1,155 to 1,142. Execution failures,
comparison mismatches, and expected-error totals remain unchanged.

## Boundary conclusion

This is a typed positional filter over one already ordered key result. It does
not add general predicates, dynamic key arguments, unions, descendant tails,
match-pattern `key()` semantics, or a retained index. Unsupported forms remain
explicit rather than being approximated.

The result is bounded XSLT 1.0 compatibility evidence, not a general `key()` or
conformance claim.
