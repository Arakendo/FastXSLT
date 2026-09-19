# OASIS XSLT 1.0 Variable-Selection Sort

Date: 2026-09-18  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an `xsl:for-each` over an invocation-owned source-node variable use the
ordinary sort machinery without creating a second sorting implementation or
moving source state into the compiled program?

## Method

- Retain sort keys on the existing typed `ForEachVariable` instruction.
- At execution, copy only the variable's node-ID slice into invocation-local
  ordering storage, then call the same charged stable sort used by direct
  source selections.
- Preserve the complete selected node set as focus after sorting, including
  `position()` and `last()` behavior.
- Include the retained sort plans in exact compiled-program capacity
  accounting.
- Exercise a focused local source-node variable with numeric descending order,
  then run the complete unchanged, hash-verified 3,173-case OASIS catalog.

## Result

The focused case sorts source values `2`, `10`, and `1` as `10|2|1|` while
retaining their source-node identities.

The unchanged `Lotus/sort_sort40#1` case now executes and compares exactly. The
exact-result lower bound rises from 1,616 to 1,617. Initialized cases rise from
2,007 to 2,008, successfully executed cases rise from 1,902 to 1,903, and
initialization failures fall from 1,128 to 1,127. Execution failures,
comparison mismatches, and expected-error totals remain unchanged.

## Boundary conclusion

This composes two existing private capabilities: invocation-owned source-node
variables and ordinary source-node sorting. It does not add a retained index,
cross-invocation sharing, sorting for atomic sequences, or a second evaluator.
The remaining `FXST1044` frontier concerns global dependency ordering and
unsupported custom sort data-type values rather than this ownership seam.

The result is bounded XSLT 1.0 compatibility evidence, not a general variable
sequence or conformance claim.
