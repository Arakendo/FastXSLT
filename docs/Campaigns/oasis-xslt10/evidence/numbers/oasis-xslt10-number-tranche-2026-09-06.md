# OASIS XSLT 1.0 Number Tranche

Date: 2026-09-06

## Question

Can a first `xsl:number` slice reuse the shared compiled-instruction, dynamic
focus, XDM navigation, work-budget, and result-text paths while keeping the
larger numbering and formatting systems explicit?

## Implemented slice

- default and explicit `level="single"` numbering when `value` is absent;
- default count-pattern behavior based on the context node's kind and expanded
  name;
- charged preceding-sibling inspection, including attribute sibling order;
- literal numeric/string conversion for the `value` attribute;
- `value="position()"` through the existing invocation-local focus;
- XSLT 1.0 default decimal rounding and `NaN`, infinity, negative, and
  sub-0.5 conversion behavior;
- bounded result construction through the existing text-output charge points.

Explicit `count`/`from`, `level="multiple|any"`, number-format tokens,
grouping, language/alphabetic numbering, variables, paths, and general value
expressions remain unsupported rather than producing approximate output.

## Focused verification

A first-party runtime test verifies that unrelated siblings do not affect the
default count, repeated same-name siblings number as 1/2/3, `position()` uses
the selected sequence focus, and representative literal values produce
`1999`, `2000`, `NaN`, and `0.42`.

## Full local OASIS observation

Command:

```powershell
./scripts/measure-oasis-xslt10.ps1
```

Relative to the preceding 857-pass sorting checkpoint:

| Observation | Before | After | Change |
| --- | ---: | ---: | ---: |
| Initialized | 1,171 | 1,178 | +7 |
| Executed successfully | 963 | 970 | +7 |
| XML comparison pass | 857 | 863 | +6 |
| XML comparison mismatch | 72 | 73 | +1 |
| Execution failure | 208 | 208 | 0 |

The strict unchanged-result lower bound is now 863 of 2,742
standard-operation cases (31.47%), or 863 of all 3,173 catalog cases (27.20%).
The newly exposed mismatch remains uncredited.

## Interpretation

The generic unsupported-instruction wall has been replaced by typed,
explainable numbering boundaries. Six cases reach their expected result without
a legacy backend or runtime version branch. This is useful shared foundation,
not a claim of complete XSLT numbering support.

