# OASIS XSLT 1.0 Descendant Match-Pattern Repair

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Trigger | Known mismatch `Lotus/conflictres_conflictres15#1` |
| Input baseline | 461 definite unchanged XML passes; 32 mismatches |
| Result | 462 definite unchanged XML passes; 31 mismatches |
| Disposition | Shared match-pattern defect repaired |

## Defect

The path parser correctly lowers `a//c` to the abbreviated descendant-or-self
shape, but relative template-pattern matching treated every step as one fixed
parent level. It therefore matched `a/*/c` but failed `a//c` when additional
ancestors occurred between `a` and `c`.

The selector now recognizes the admitted two-name descendant pattern and walks
the candidate's ancestors until the required ancestor name is found. Every
ancestor inspection charges `XPathNodeVisit` and observes normal cancellation.
Other path shapes retain the existing evaluator and no general pattern claim is
made from this focused repair.

## Evidence

A focused production test contains both an `a/*/c` grandchild and a deeper
`a//c` descendant. It proves the two candidates select different templates.
Unchanged OASIS `Lotus/conflictres_conflictres15#1` then moves from XML mismatch
to pass. No initialization, execution, expected-error, or comparator counts
change.

The strict lower bound is now **462 / 2,742 = 16.85%** of standard-operation
cases and **462 / 3,173 = 14.56%** of the full archival catalog. This remains
compatibility evidence rather than a conformance claim.

The repair is shared modern-engine behavior: it changes neither stylesheet
version handling nor the public/runtime model. Broader descendant patterns,
qualified names, predicates, and arbitrary reverse matching remain separately
bounded compiler/runtime work.
