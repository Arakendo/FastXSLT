# OASIS XSLT 1.0 Context String-Length Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 527 definite unchanged XML passes; 792 initialized cases |
| Result | 529 definite unchanged XML passes; 794 initialized cases |
| Disposition | Shared XPath context-function expansion; not a conformance claim |

## Change

No-argument `string-length()` now lowers to a typed context operation. Runtime
evaluation visits the complete XDM string value, counts Unicode scalar values
rather than UTF-8 bytes, charges every character to XPath work, observes
cancellation throughout the traversal, and emits the result through the bounded
result-text path.

The compiled operation retains its source location so a focusless invocation
reports located `XPDY0002`. The focused end-to-end regression uses non-ASCII
characters split across direct and descendant text nodes, proving both Unicode
counting and complete XDM string-value traversal.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 792 | 794 | +2 |
| Execution succeeded | 605 | 607 | +2 |
| XML comparison passes | 527 | 529 | +2 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 187 | 187 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **529 / 2,742 = 19.29%** of standard-operation
cases and **529 / 3,173 = 16.67%** of the complete catalog. Both newly
initialized cases reach definite unchanged XML comparison passes.

## Boundaries

This tranche does not add arbitrary context-dependent `string-length`
arguments, general atomization/conversion, or a second source-free evaluator.
The existing QT3-backed typed implementation remains responsible for static
and admitted document-path expressions.
