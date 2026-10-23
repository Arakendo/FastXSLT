# OASIS XSLT 1.0 Context Normalize-Space Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 521 definite unchanged XML passes; 784 initialized cases |
| Result | 522 definite unchanged XML passes; 785 initialized cases |
| Disposition | Shared XPath context-function expansion; not a conformance claim |

## Change

The context forms `normalize-space()` and `normalize-space(.)` now lower to one
typed value operation. Runtime evaluation visits the complete XDM string value
of the context node, preserves whitespace-collapse state across descendant text
boundaries, charges every examined character to XPath work, observes
cancellation during the scan, and appends the bounded normalized value through
the shared result-text path.

The implementation deliberately does not normalize each text fragment
independently: doing so could invent or lose spaces where adjacent descendant
text contributions meet. The focused regression exercises leading, trailing,
repeated, and cross-descendant XML whitespace with both admitted spellings.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 784 | 785 | +1 |
| Execution succeeded | 599 | 600 | +1 |
| XML comparison passes | 521 | 522 | +1 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 185 | 185 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **522 / 2,742 = 19.04%** of standard-operation
cases and **522 / 3,173 = 16.45%** of the complete catalog. The newly
initialized case reaches a definite unchanged XML comparison pass. Another
stylesheet containing a context normalization spelling remains behind an
independent unsupported feature and is not credited.

## Boundaries

This tranche does not add arbitrary `normalize-space` argument expressions,
general atomization/conversion, temporary-tree context evaluation beyond the
existing runtime, or a second normalization backend. Static/source-free
normalization remains on its existing QT3-backed typed evaluator.
