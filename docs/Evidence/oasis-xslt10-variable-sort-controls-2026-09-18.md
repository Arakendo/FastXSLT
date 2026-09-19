# OASIS XSLT 1.0 Variable Sort Controls

Date: 2026-09-18  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can `xsl:sort` resolve `data-type` and `order` from XSLT 1.0 variables without
adding a general attribute-value-template evaluator to the sorting hot path?

## Method

- Admit only the exact one-expression AVT whose expression is one unqualified
  variable reference.
- Retain the variable name as part of the typed sort-control plan and include
  its owned capacity in compiled-program accounting.
- Convert each variable through the existing charged XSLT 1.0 string-value
  owner once per sort invocation, before extracting candidate keys.
- Resolve only `text` or `number` for `data-type`, and only `ascending` or
  `descending` for `order`. Preserve structured runtime failure for values
  outside those sets.
- Keep path-valued, composed, and mixed-text AVTs explicitly unsupported.
- Run a focused variable-controlled numeric descending sort and the complete
  unchanged, hash-verified 3,173-case OASIS catalog.

## Result

The focused case resolves both controls from local variables and produces
`10|2|1|` from source values `2`, `10`, and `1`.

The unchanged `Lotus/sort_sort32#1` and `Lotus/sort_sort33#1` cases now execute
and compare exactly. The exact-result lower bound rises from 1,614 to 1,616.
Initialized cases rise from 2,005 to 2,007, successfully executed cases rise
from 1,900 to 1,902, and initialization failures fall from 1,130 to 1,128.
Execution failures, comparison mismatches, and expected-error totals remain
unchanged.

## Boundary conclusion

This admits variable selection for two standard sort controls, not general
dynamic AVTs, language-sensitive collation, custom data-type QNames, or a
runtime version branch. Sort-key focus, stable ordering, work charging,
cancellation, and result semantics continue through the existing owners.

The result is bounded XSLT 1.0 compatibility evidence, not a general sorting
or conformance claim.
