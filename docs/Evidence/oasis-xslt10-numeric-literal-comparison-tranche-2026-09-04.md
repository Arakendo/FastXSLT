# OASIS XSLT 1.0 Numeric Literal Comparison Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 529 definite unchanged XML passes; 794 initialized cases |
| Result | 553 definite unchanged XML passes; 818 initialized cases |
| Disposition | Shared XPath numeric semantics; not a conformance claim |

## Change

Source-free comparisons whose two operands are XPath numeric literals now use
a typed numeric comparison path. The parser accepts the XPath 1.0 decimal
lexical forms needed by the measured cases, including optional unary minus and
decimal forms such as `1.00` and `.5`. It evaluates equality and ordered
comparisons as numbers and retains the result in the existing bounded,
work-charged scalar path.

This is deliberately separate from the existing boolean constant comparator.
Numeric operands are never converted to effective boolean values before the
comparison: `2 > 1` is true because of numeric ordering, not because both
operands happen to have the same truth value. A normal end-to-end runtime test
locks equality, inequality, negative zero, and decimal ordering through an
XSLT 1.0 stylesheet.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 794 | 818 | +24 |
| Execution succeeded | 607 | 631 | +24 |
| XML comparison passes | 529 | 553 | +24 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 187 | 187 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **553 / 2,742 = 20.17%** of standard-operation
cases and **553 / 3,173 = 17.43%** of the complete catalog. Every newly
initialized case reaches a definite unchanged XML comparison pass.

## Boundaries

This tranche does not admit comparisons between source nodes, variables,
strings, booleans, or mixed types; arithmetic expressions; XPath 1.0 general
comparison over node-sets; or a general numeric AST. Those remain explicit
frontiers until their typed conversion and cardinality rules are implemented.
