# OASIS XSLT 1.0 Value Focus Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 522 definite unchanged XML passes; 785 initialized cases |
| Result | 525 definite unchanged XML passes; 790 initialized cases |
| Disposition | Shared dynamic-focus exposure; not a conformance claim |

## Change

`xsl:value-of` now admits the typed `position()` and `last()` expressions. The
value evaluator receives the runtime's existing sequence-focus pair rather than
recomputing it from a node list. This preserves the same focus already carried
through source-node, temporary-tree, and atomic template dispatch while keeping
the compiled operations independent of any one representation.

Both operations charge XPath work and append their integer result through the
bounded result-text path. Their compiled form retains the expression source
location. A focusless initial-template invocation reports located `XPDY0002`
rather than fabricating the default-looking value `1`.

Focused regressions prove a three-item source dispatch produces positions
`1`, `2`, and `3` with size `3`, and prove the missing-dynamic-focus failure for
an initial template invoked without a context item.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 785 | 790 | +5 |
| Execution succeeded | 600 | 603 | +3 |
| XML comparison passes | 522 | 525 | +3 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 185 | 187 | +2 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **525 / 2,742 = 19.15%** of standard-operation
cases and **525 / 3,173 = 16.55%** of the complete catalog. The two newly
initialized cases that stop at later execution boundaries remain explicitly
uncredited.

## Boundaries

This tranche does not add positional predicates, sorting, numbering, arbitrary
focus-dependent function composition, or a new focus representation. It only
passes the existing invocation-local sequence focus into typed value
evaluation. Static focus analysis and broader XPath expression composition
remain separate work.
