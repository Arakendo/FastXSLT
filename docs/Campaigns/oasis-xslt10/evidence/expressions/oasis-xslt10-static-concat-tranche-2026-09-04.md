# OASIS XSLT 1.0 Static Concat Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 560 definite unchanged XML passes; 825 initialized cases |
| Result | 567 definite unchanged XML passes; 832 initialized cases |
| Disposition | Shared bounded XPath constant folding; not a conformance claim |

## Change

`concat()` and `fn:concat()` now fold when every argument is a statically known
string, boolean, integer, or an admitted static `string()` conversion. Argument
splitting respects nested parentheses, quoted commas, and both XPath quote
styles. The folded value enters the existing bounded literal-result path, so
execution does not carry a general-purpose concat interpreter or runtime
feature branch.

The compiler admits at most 4,096 arguments. A focused unit test proves the
ceiling, and a normal end-to-end transform proves multi-argument conversion and
result construction. The unchanged OASIS 1,000-argument stress case remains
inside the ceiling and passes.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 825 | 832 | +7 |
| Execution succeeded | 638 | 645 | +7 |
| XML comparison passes | 560 | 567 | +7 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 16 | 16 | 0 |
| Execution failures | 187 | 187 | 0 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **567 / 2,742 = 20.68%** of standard-operation
cases and **567 / 3,173 = 17.87%** of the complete catalog. Every newly
initialized case reaches a definite unchanged XML comparison pass.

## Boundaries

This tranche does not add runtime concat over variables or source paths,
general atomization, arbitrary numeric formatting, nested dynamic functions,
or compatibility-only conversion. Those expressions remain explicit rather
than being partially evaluated.
