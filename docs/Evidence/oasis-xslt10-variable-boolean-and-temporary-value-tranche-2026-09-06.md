# OASIS XSLT 1.0 Variable Boolean and Temporary-Value Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-06 |
| Corpus | OASIS XSLT/XPath 1.0 CD04, locally acquired and hash-verified |
| Scope | Variable effective-boolean-value conversion, XPath 1.0 variable comparisons, and source-derived temporary result trees |
| Disposition | Ten additional expected-result XML matches; two newly executing doubt-annotated cases remain visible mismatches |

## Changes

`xsl:value-of select="boolean($variable)"` now reuses the same private,
budgeted effective-boolean-value evaluator as instruction conditions. Atomic
values, source-node sequences, empty sequences, and temporary trees therefore
do not acquire a second conversion implementation. Global atomic fallback was
also repaired so local and global variables observe the same typed rule.

XSLT 1.0 compilation now recognizes equality and inequality between one
variable and `true()` or `false()`, in either operand order and under one
outer `not(...)`. Execution converts the variable through the shared EBV path
before applying the comparison. This compatibility-only plan does not alter
modern general-comparison behavior. The unchanged `boolean85` and `boolean86`
node-set cases become exact expected-result matches.

A global XSLT 1.0 variable containing exactly one `xsl:value-of` over a
location path can now materialize a source-derived temporary tree. The
location path is evaluated from the principal source document node, only the
first selected node contributes its string value, and path/string/tree work is
charged at its owning layer. The unchanged `boolean42` and Microsoft
`XSLTFunctions_BooleanFunction` cases become exact. `boolean87` also executes,
but its doubt-annotated archival expectation disagrees with the implemented
temporary-tree boolean semantics and remains uncredited. The already executing
`boolean43` disagreement remains equally visible.

The compatibility plan also covers variable comparison with string and integer
literals. Temporary trees and scalar values compare through their XPath string
or number conversion; source node-sets retain XPath 1.0 existential semantics.
Equality and inequality are evaluated independently across every node, so a
multi-node set may satisfy both, and an outer `not(...)` is applied only after
that existential result is known. This corrects the unchanged `boolean58`,
`boolean59`, `boolean84`, `boolean88`, and `boolean89` families without teaching
the modern evaluator XPath 1.0 general-comparison rules.

Focused tests cover typed global atomic EBV, empty/nonempty/source-derived
temporary trees, nonempty node-set comparisons, both operand orders,
equality/inequality, and outer negation.

## Measurement

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Catalog cases | 3,173 | 3,173 | 0 |
| Initialized | 1,269 | 1,281 | +12 |
| Executed successfully | 1,096 | 1,108 | +12 |
| Expected-result XML matches | 986 | 996 | +10 |
| XML comparison mismatches | 75 | 77 | +2 |
| Doubt-annotated comparison mismatches | 8 | 8 | 0 |
| Execution failures | 173 | 173 | 0 |
| Expected execution errors observed | 15 | 15 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |

The strict standard-operation lower bound is now `996 / 2,742 = 36.32%`;
the deliberately conservative all-catalog ratio is `996 / 3,173 = 31.39%`.
This is compatibility evidence, not an XSLT 1.0 conformance claim.
