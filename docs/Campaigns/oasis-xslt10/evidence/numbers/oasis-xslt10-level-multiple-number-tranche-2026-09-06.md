# OASIS XSLT 1.0 Level-Multiple Number Tranche

Date: 2026-09-06

## Question

Can the shared numbering plan produce a multiple-level number list from a
source-node ancestor lineage without admitting the complete multi-token
formatting or pattern languages?

## Implemented slice

- charged ancestor-or-self lineage traversal up to an admitted `from` boundary;
- selection by the context-derived default pattern or an admitted static
  `count` pattern;
- charged preceding-sibling position calculation for every selected ancestor;
- document-order reversal of the selected ancestor lineage;
- repeated single decimal tokens separated by `.` with per-token zero padding;
- existing prefix/suffix punctuation around the complete formatted list.

Multiple explicit formatting tokens, alphabetic/Roman numbering, pattern
unions/predicates, grouping, and AVTs remain unsupported.

## Focused verification

A first-party recursive-template test numbers a nested `a` hierarchy as `1`,
`1.1`, `1.2`, and `1.2.1`, proving lineage ordering and per-level sibling
positions.

## Full local OASIS observation

Command:

```powershell
./scripts/measure-oasis-xslt10.ps1
```

Relative to the 887-pass level-any checkpoint:

| Observation | Before | After | Change |
| --- | ---: | ---: | ---: |
| Initialized | 1,206 | 1,213 | +7 |
| Executed successfully | 996 | 1,002 | +6 |
| XML comparison pass | 887 | 893 | +6 |
| XML comparison mismatch | 75 | 75 | 0 |
| Execution failure | 210 | 211 | +1 |

The newly exposed later failure remains uncredited. The strict unchanged-result
lower bound is now 893 of 2,742 standard-operation cases (32.57%), or 893 of
all 3,173 catalog cases (28.14%).

## Interpretation

All three XSLT 1.0 numbering levels now have a bounded executable foundation.
Remaining cases fail at named pattern, format, expression, serialization, or
other feature boundaries rather than a generic unsupported-level wall. This is
still a partial numbering system, not a complete `xsl:number` claim.
