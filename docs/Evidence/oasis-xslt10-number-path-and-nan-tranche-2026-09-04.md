# OASIS XSLT 1.0 Number Path and NaN Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 749 definite unchanged XML passes; 1,033 initialized cases |
| Result | 753 definite unchanged XML passes; 1,037 initialized cases |
| Disposition | Shared typed `number()` semantics; not a conformance claim |

## Change

The compiled value-expression model now has a typed `number(location-path)`
operation. Execution evaluates the shared work-charged path and string-value
machinery, enforces the modern zero-or-one cardinality contract, charges the
numeric conversion, and emits a canonical finite decimal or `NaN`. An empty
selection and an ordinary non-convertible lexical both produce `NaN`, matching
the shared modern `fn:number` behavior.

The same conversion now folds quoted non-convertible static strings to `NaN`.
Exponent and infinity lexicals remain unsupported because their admitted forms
and compatibility consequences need separate review. The path operation does
not take the first of several nodes under XSLT 1.0 compatibility rules; it
retains the modern core's `XPTY0004` boundary.

## Unchanged cases

Four standard-operation cases move directly to definite XML comparison passes:

- `Lotus/math_math01#1` (`number(n1)`);
- `Lotus/math_math11#1` (`number(foo)` with an empty selection);
- `Lotus/math_math14#1` (`number('')`); and
- `Lotus/math_math15#1` (`number('abc')`).

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 1,033 | 1,037 | +4 |
| Execution succeeded | 832 | 836 | +4 |
| XML comparison passes | 749 | 753 | +4 |
| XML comparison mismatches | 50 | 50 | 0 |
| XML comparator gaps | 19 | 19 | 0 |
| Execution failures | 201 | 201 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **753 / 2,742 = 27.46%** of standard-operation
cases and **753 / 3,173 = 23.73%** of the complete catalog.

## Verification

Focused conversion tests cover finite canonicalization, negative zero, empty
and invalid `NaN` results, and retained special-lexical boundaries. Production
runtime tests exercise static, singleton-path, empty-path, invalid-path, and
multi-node cardinality behavior. The complete local OASIS sweep confirms all
four newly initialized identities pass without a new mismatch, comparator gap,
runtime failure, unexpected success, or panic.
