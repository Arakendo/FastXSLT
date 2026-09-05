# OASIS XSLT 1.0 Static Short-Circuit Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 758 definite unchanged XML passes; 1,042 initialized cases |
| Result | 760 definite unchanged XML passes; 1,044 initialized cases |
| Disposition | Shared static boolean semantics; not a conformance claim |

The compiler now folds the exact valid constant short-circuit expressions
`false() and 1 div 0` and `true() or 1 div 0`, including the corresponding
`fn:` boolean spellings. The compiled plan retains a typed constant boolean,
so the unreachable right operand introduces no runtime evaluation or version
branch. Arbitrary right operands and the non-short-circuiting variants remain
outside this recognizer; this is not a general partial-expression evaluator.

The unchanged `Lotus/boolean_boolean56#1` and
`Lotus/boolean_boolean57#1` cases pass. Initialization moves from 1,042 to
1,044, execution from 841 to 843, and XML passes from 758 to 760. Mismatches
remain 50, comparator gaps 19, execution failures 201, unexpected successes
14, and panics 0.

The strict lower bound is now **760 / 2,742 = 27.72%** of standard-operation
cases and **760 / 3,173 = 23.95%** of the complete catalog. Focused tests cover
both results, namespace-prefixed boolean functions, rejection of the opposite
operators, and rejection of an invalid arbitrary right operand.
