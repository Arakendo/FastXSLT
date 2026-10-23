# OASIS XSLT 1.0 Named Processing-Instruction Path Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 481 definite unchanged XML passes; 715 initialized cases |
| Result | 485 definite unchanged XML passes; 719 initialized cases |
| Disposition | Shared typed XPath node-test refinement; not a conformance claim |

## Change

The location-path parser and evaluator now retain the optional NCName target in
`processing-instruction('target')` and its double-quoted equivalent. The typed
step filters processing-instruction nodes by their target on ordinary child,
relative descendant, and leading descendant paths while continuing to use the
existing charged traversal, document-order normalization, and cancellation
points.

The focused XPath regression distinguishes an immediate `work` processing
instruction from one nested below an element and from a sibling with another
target. It proves both child and descendant selection with single- and
double-quoted target literals. Retained-capacity accounting includes the owned
target string.

This is shared path behavior available to every existing consumer of the typed
location-path evaluator. It was not implemented as an `xsl:copy-of`-specific
function-shaped exception.

## Measurement

The complete hash-verified local sweep moves as follows:

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 715 | 719 | +4 |
| Execution succeeded | 540 | 544 | +4 |
| XML comparison passes | 481 | 485 | +4 |
| XML comparison mismatches | 30 | 30 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **485 / 2,742 = 17.69%** of standard-operation
cases and **485 / 3,173 = 15.29%** of the complete catalog. Every newly
initialized case reaches a definite XML comparison pass.

## Boundaries

This tranche does not admit arbitrary function calls in path steps, computed PI
targets, wildcard string arguments, targeted PI tests on unimplemented axes, or
the corresponding template-pattern syntax. Those remain separate typed grammar
and semantic obligations.
