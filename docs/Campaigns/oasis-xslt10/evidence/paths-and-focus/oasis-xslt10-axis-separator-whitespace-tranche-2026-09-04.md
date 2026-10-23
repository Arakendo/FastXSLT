# OASIS XSLT 1.0 Axis-Separator Whitespace Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-04 |
| Suite | OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04 |
| Input baseline | 729 definite unchanged XML passes; 1,010 initialized cases |
| Result | 731 definite unchanged XML passes; 1,013 initialized cases |
| Disposition | Shared XPath lexical recognition; not a conformance claim |

## Change

The typed location-path parser now accepts XPath whitespace immediately before
or after the `::` token separating an axis name from its node test. Compilation
canonicalizes only XML whitespace at that token boundary before ordinary axis
validation and typed step lowering. The retained path and runtime evaluator are
unchanged.

This does not ignore whitespace inside an axis name or node test, loosen QName
validation, or add a compatibility-only parser.

## Unchanged cases

Two cases move directly to definite XML comparison passes:

- `Lotus/select_select27#1`, using `attribute :: div`; and
- `Lotus/select_select28#1`, using `attribute :: *`.

`Lotus/select_select16#1`, using `child :: sub`, also crosses initialization but
selects multiple nodes for `xsl:value-of`. It retains the shared modern
`FXRT1001` multi-node conversion boundary and receives no compatibility credit.

## Measurement

| Observation | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Engine initialization succeeded | 1,010 | 1,013 | +3 |
| Execution succeeded | 810 | 812 | +2 |
| XML comparison passes | 729 | 731 | +2 |
| XML comparison mismatches | 48 | 48 | 0 |
| XML comparator gaps | 19 | 19 | 0 |
| Execution failures | 200 | 201 | +1 |
| Expected-error unexpected successes | 14 | 14 | 0 |
| Engine panics | 0 | 0 | 0 |

The strict lower bound is now **731 / 2,742 = 26.66%** of standard-operation
cases and **731 / 3,173 = 23.04%** of the complete catalog.

## Verification

A focused parser test covers tab, carriage-return, line-feed, and space tokens
around child- and attribute-axis separators. The complete local OASIS sweep
confirms both intended cases pass without a new mismatch, comparator gap,
unexpected success, or panic. The newly exposed cardinality failure remains
separately visible.
