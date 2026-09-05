# OASIS XSLT 1.0 Context Number Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 753 definite unchanged XML passes; 1,037 initialized cases |
| Result | 756 definite unchanged XML passes; 1,040 initialized cases |
| Disposition | Shared implicit-context function semantics; not a conformance claim |

## Change

Zero-argument `number()` now compiles to the same typed number-path operation
as `number(.)`. The implicit context item is therefore made explicit in the
compiled plan, and execution reuses the existing context checks, work-charged
XDM string-value derivation, numeric conversion, and result construction. No
runtime stylesheet-version branch or second numeric evaluator is introduced.

## Unchanged cases

Three standard-operation cases move directly to definite XML comparison passes:

- `Lotus/math_math102#1`;
- `Microsoft/XSLTFunctions_NumberFunctWithNoArg1#1`; and
- `Microsoft/XSLTFunctions_NumberFunctWithNoArg2#1`.

The two Microsoft identities intentionally reuse one stylesheet against two
distinct principal sources and expected results; each retains its independent
catalog identity and disposition.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 1,037 | 1,040 | +3 |
| Execution succeeded | 836 | 839 | +3 |
| XML comparison passes | 753 | 756 | +3 |
| XML comparison mismatches | 50 | 50 | 0 |
| XML comparator gaps | 19 | 19 | 0 |
| Execution failures | 201 | 201 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **756 / 2,742 = 27.57%** of standard-operation
cases and **756 / 3,173 = 23.83%** of the complete catalog.

## Verification

The focused numeric parser test distinguishes an empty `number()` argument
list from a non-call, and the production runtime test executes the zero-argument
form against an actual context node. The complete local OASIS sweep confirms
all three affected identities pass without a new mismatch, comparator gap,
runtime failure, unexpected success, or panic.
