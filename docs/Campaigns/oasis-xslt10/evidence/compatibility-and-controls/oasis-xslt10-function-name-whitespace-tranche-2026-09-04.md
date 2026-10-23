# OASIS XSLT 1.0 Function-Name Whitespace Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 613 definite unchanged XML passes; 879 initialized cases |
| Result | 614 definite unchanged XML passes; 880 initialized cases |
| Disposition | Shared XPath lexical recognition; not a conformance claim |

## Change

The instruction value compiler now recognizes XML whitespace between the
`string-length` function QName and its empty argument list. Both
`string-length()` and `string-length ()` lower to the same context-dependent
typed operation. The `fn:` spelling retains its existing source-free
compilation and missing-context diagnostic path.

This is lexical recognition only. It neither changes function semantics nor
adds a legacy evaluator.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 879 | 880 | +1 |
| Execution succeeded | 691 | 692 | +1 |
| XML comparison passes | 613 | 614 | +1 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 188 | 188 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **614 / 2,742 = 22.39%** of
standard-operation cases and **614 / 3,173 = 19.35%** of the complete catalog.
Unchanged OASIS `Lotus/select_select20#1` reaches a definite XML comparison
pass.

## Boundaries

This tranche deliberately normalizes only the function-name-to-opening-
parenthesis gap for an already admitted zero-argument operation. It does not
claim general XPath tokenization or broaden string-length argument conversion.
