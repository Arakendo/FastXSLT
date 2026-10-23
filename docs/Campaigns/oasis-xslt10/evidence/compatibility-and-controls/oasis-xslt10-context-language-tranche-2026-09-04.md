# OASIS XSLT 1.0 Context Language Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 606 definite unchanged XML passes; 871 initialized cases |
| Result | 607 definite unchanged XML passes; 872 initialized cases |
| Disposition | Shared context-dependent XPath semantics; not a conformance claim |

## Change

Literal-argument `lang()` and `fn:lang()` now compile to one private semantic
operation used by both value production and instruction test expressions. The
evaluator walks the context node and its ancestors, uses the nearest `xml:lang`
attribute, compares ASCII case-insensitively, and admits the required
language-subtag prefix followed by `-`.

Ancestor and attribute inspection is charged in the XPath node-visit work
domain, so inherited-language discovery remains observable by cancellation and
work budgets. No ambient locale or host language setting participates.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 871 | 872 | +1 |
| Execution succeeded | 684 | 685 | +1 |
| XML comparison passes | 606 | 607 | +1 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 187 | 187 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **607 / 2,742 = 22.14%** of standard-operation
cases and **607 / 3,173 = 19.13%** of the complete catalog. Unchanged OASIS
`Lotus/boolean_boolean08#1` reaches a definite XML comparison pass.

## Boundaries

This tranche admits one literal language argument. Dynamic arguments and the
optional-node modern function form remain explicit. It does not admit the
neighboring XPath 1.0 mixed-type comparison cases, whose coercion rules remain
behind AR-0019's unresolved version-mode boundary.
