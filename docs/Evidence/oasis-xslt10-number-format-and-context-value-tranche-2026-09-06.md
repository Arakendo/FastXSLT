# OASIS XSLT 1.0 Number Format and Context-Value Tranche

Date: 2026-09-06

## Question

Can the first `xsl:number` instruction support common decimal presentation and
context-item numeric conversion through the shared runtime without implying
complete numbering-format or XPath 1.0 node-set conversion support?

## Implemented slice

- one static decimal formatting token with optional leading zeroes;
- punctuation before and after that token, including `01`, `(001) `, and
  `[1]`;
- the default `1` format represented by the same compiled format structure;
- `value="."` converted from the charged context XDM string value;
- XML-whitespace trimming before numeric conversion;
- exact retained-capacity accounting for compiled format prefix and suffix.

Alphabetic and Roman tokens, multiple formatting tokens, grouping, language,
`letter-value`, dynamic AVTs, and multi-level numbering remain explicit
unsupported boundaries.

## Focused verification

A first-party runtime test verifies zero padding, surrounding punctuation,
exceptional `NaN` presentation, numeric context-item conversion, whitespace
trimming, and nonnumeric context-item conversion.

## Full local OASIS observation

Command:

```powershell
./scripts/measure-oasis-xslt10.ps1
```

The decimal-format slice moved seven cases directly to expected-result matches:

| Observation | Before | After | Change |
| --- | ---: | ---: | ---: |
| Initialized | 1,178 | 1,185 | +7 |
| Executed successfully | 970 | 977 | +7 |
| XML comparison pass | 863 | 870 | +7 |
| XML comparison mismatch | 73 | 73 | 0 |
| Execution failure | 208 | 208 | 0 |

The subsequent context-item slice exposed three more cases. One matched and two
upstream doubt-annotated cases remained visibly mismatched because their
expected files omit the `NaN` numbering text that the suite's own doubts say
should be present:

| Observation | Before | After | Change |
| --- | ---: | ---: | ---: |
| Initialized | 1,185 | 1,188 | +3 |
| Executed successfully | 977 | 980 | +3 |
| XML comparison pass | 870 | 871 | +1 |
| XML comparison mismatch | 73 | 75 | +2 |
| Doubt-annotated mismatch | 5 | 7 | +2 |
| Execution failure | 208 | 208 | 0 |

The strict unchanged-result lower bound is now 871 of 2,742
standard-operation cases (31.77%), or 871 of all 3,173 catalog cases (27.45%).
The two disputed expectations remain uncredited.

## Interpretation

This tranche adds a useful single-number decimal presentation primitive and
reuses the complete XDM string-value traversal for `.`. It does not select the
larger numbering tokenization, count/from pattern, multi-level, or legacy
node-set conversion systems.
