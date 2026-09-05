# OASIS XSLT 1.0 Boolean Number-Equality Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 756 definite unchanged XML passes; 1,040 initialized cases |
| Result | 758 definite unchanged XML passes; 1,042 initialized cases |
| Disposition | Shared static conversion semantics; not a conformance claim |

The compiler now folds the exact source-free equality forms comparing
`number(true())` or `number(false())` with `1` or `0`. It retains a typed
constant boolean in the plan; no runtime conversion or version branch is added.
Inequality, paths, variables, and general numeric expressions remain outside
this recognizer.

The unchanged `Lotus/math_math19#1` and `Lotus/math_math20#1` cases pass. The
measurement moves initialization from 1,040 to 1,042, execution from 839 to
841, and XML passes from 756 to 758. Mismatches remain 50, comparator gaps 19,
execution failures 201, unexpected successes 14, and panics 0.

The strict lower bound is now **758 / 2,742 = 27.64%** of standard-operation
cases and **758 / 3,173 = 23.89%** of the complete catalog. Focused tests cover
true/false, operand order, false results, and rejection of path and inequality
forms; the complete sweep confirms no later-failure category changed.
