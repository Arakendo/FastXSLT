# OASIS XSLT 1.0 Context String Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 519 definite unchanged XML passes; 782 initialized cases |
| Result | 521 definite unchanged XML passes; 784 initialized cases |
| Disposition | Shared XPath context-function spelling; not a conformance claim |

## Change

The `string()` and `string(.)` spellings now compile to the existing typed `.`
location path. This deliberately adds no second string-value implementation:
the shared path evaluator visits the context node, and the existing XDM
string-value traversal supplies descendant text with its normal work charging,
cancellation points, and result-text bounds.

The focused end-to-end regression proves that both spellings return the same
string value for an element with mixed direct and descendant text.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 782 | 784 | +2 |
| Execution succeeded | 597 | 599 | +2 |
| XML comparison passes | 519 | 521 | +2 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 185 | 185 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **521 / 2,742 = 19.00%** of standard-operation
cases and **521 / 3,173 = 16.42%** of the complete catalog. Both newly
initialized cases reach definite unchanged XML comparison passes. Other
stylesheets containing these spellings remain behind independent features such
as `xsl:key` and are not credited.

## Boundaries

This tranche does not add general `fn:string` argument evaluation, atomic
conversion rules, temporary-tree context support beyond the existing runtime,
or a new XPath function-call parser. It only normalizes the two context-item
spellings to an already admitted semantic operation.
