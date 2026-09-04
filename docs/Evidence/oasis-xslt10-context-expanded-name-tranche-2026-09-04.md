# OASIS XSLT 1.0 Context Expanded-Name Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 514 definite unchanged XML passes; 777 initialized cases |
| Result | 519 definite unchanged XML passes; 782 initialized cases |
| Disposition | Shared XPath context-function expansion; not a conformance claim |

## Change

No-argument and context-item spellings of `local-name()` and `namespace-uri()`
now lower to two typed value operations. Both read the current node's owned
expanded name, charge the existing XPath node-visit domain, and emit bounded
result text through the shared result builder.

Unlike `name()`, neither operation needs to reconstruct a source lexical QName.
`local-name()` returns the retained local part and `namespace-uri()` returns the
retained namespace URI or the empty string. Unnamed nodes likewise produce the
empty string required by the function semantics.

The focused end-to-end regression evaluates all four admitted spellings against
a namespaced source element and proves that the operations preserve the local
name and namespace URI independently of the source prefix.

## Measurement

The complete hash-verified local sweep moves as follows:

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 777 | 782 | +5 |
| Execution succeeded | 592 | 597 | +5 |
| XML comparison passes | 514 | 519 | +5 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 185 | 185 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **519 / 2,742 = 18.93%** of standard-operation
cases and **519 / 3,173 = 16.36%** of the complete catalog. Every case newly
initialized by this tranche reaches a definite unchanged XML comparison pass.

## Boundaries

This tranche does not add argument expressions beyond the context-item
spelling, lexical-QName prefix reconstruction, namespace-node support, or
general function-call parsing. It reuses the current owned source-tree
representation and does not establish a public node-provider abstraction.
