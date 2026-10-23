# OASIS XSLT 1.0 Counted Key Selection

## Question

Can the XSLT 1.0 `count()` value-expression consumer reuse the private charged
`key()` node selection without adding another lookup implementation?

## Method

- Add a typed count-key value expression in the existing XSLT 1.0 `count()`
  compiler branch.
- Delegate node selection to the same complete charged key scan used by value
  conversion, `xsl:for-each`, `xsl:apply-templates`, and `xsl:copy-of`.
- Count only after key values, positional predicates, and optional path tails
  have selected the effective ordered node sequence.
- Retain exact prepared-program capacity accounting for the shared lookup plan.
- Test static, source-node-variable, positional, and changing-context lookup
  values, then run the unchanged, hash-verified 3,173-case OASIS CD04 catalog.

## Result

The focused case proves that all four admitted lookup shapes produce their
expected cardinalities through the shared selector.

The unchanged `Lotus/idkey_idkey15#1` case now executes and compares exactly as
`<out>3</out>`.

The strict exact-result lower bound rises from 1,607 to 1,608. Initialized cases
rise from 1,997 to 1,998 and successfully executed cases rise from 1,892 to
1,893. Initialization failures fall from 1,138 to 1,137. Execution failures,
comparison mismatches, and expected-error totals remain unchanged.

## Boundary conclusion

This is a typed XSLT 1.0 `count(key(...))` consumer over the already admitted
literal-name key lookup. It does not admit key expressions in attribute value
templates, boolean conditions, sort keys, unions, match patterns, nested key
calls, dynamic key names, cross-document context switching, or a retained
index.

The result is bounded XSLT 1.0 compatibility evidence, not a general `key()` or
conformance claim.
