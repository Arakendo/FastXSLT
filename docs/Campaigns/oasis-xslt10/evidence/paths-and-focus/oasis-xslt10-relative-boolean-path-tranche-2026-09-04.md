# OASIS XSLT 1.0 Relative Boolean-Path Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 760 definite unchanged XML passes; 1,044 initialized cases |
| Result | 762 definite unchanged XML passes; 1,046 initialized cases |
| Disposition | Shared dynamic-focus semantics; not a conformance claim |

The document-aware effective-boolean-value plan now recognizes a conservative
unqualified relative name-path argument and evaluates every retained path from
the invocation's actual dynamic context. The previous runtime entry point
discarded that context and always started from the document node. Direct
document-level evaluation still supplies the document node explicitly, so no
second evaluator or compatibility branch was introduced.

The unchanged `Lotus/boolean_boolean40#1` and
`Lotus/boolean_boolean41#1` cases pass. Initialization moves from 1,044 to
1,046, execution from 843 to 845, and XML passes from 760 to 762. Mismatches
remain 50, comparator gaps 19, execution failures 201, unexpected successes
14, and panics 0.

The strict lower bound is now **762 / 2,742 = 27.79%** of standard-operation
cases and **762 / 3,173 = 24.02%** of the complete catalog. A focused runtime
test proves present and absent relative children under a non-document template
focus; the complete sweep confirms no later-failure category changed.
