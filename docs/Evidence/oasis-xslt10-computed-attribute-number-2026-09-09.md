# OASIS XSLT 1.0 computed-attribute numbering -- 2026-09-09

Date: 2026-09-09  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a computed attribute reuse FastXSLT's existing bounded `xsl:number`
semantics without introducing a general constructor executor or a second
numbering implementation?

## Changes

- A computed attribute may contain exactly one already-admitted `xsl:number`
  instruction. The ordinary number compiler produces the same typed
  instruction used by result-sequence numbering.
- Number evaluation is separated from result-text appending. Ordinary
  `xsl:number` retains its previous behavior; computed attributes consume the
  evaluated string as their value.
- Attribute evaluation preserves the current source node, focus position, and
  focus size. Result-node and XPath work remain charged through their existing
  owners, and cancellation remains observable at those charge points.
- Retained-capacity accounting includes the nested compiled instruction, and
  semantic inspection reports both the computed attribute and its activated
  numbering operation.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,566 | 1,570 | +4 |
| Executed successfully | 1,395 | 1,399 | +4 |
| Expected-result XML matches | 1,275 | 1,279 | +4 |
| XML comparison mismatches | 93 | 93 | 0 |
| XML comparator unsupported | 22 | 22 | 0 |
| Execution failures | 171 | 171 | 0 |

Six cases move beyond the `FXST1033` computed-attribute content boundary,
reducing that frontier from 21 to 15. Four produce exact expected results; two
reach later compilation boundaries and remain uncredited.

The strict standard-operation lower bound is now
`1,279 / 2,742 = 46.64%`; the conservative all-catalog ratio is
`1,279 / 3,173 = 40.31%`. These are local compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit general sequence constructors inside attributes,
multiple child instructions, arbitrary instruction composition, or a separate
XSLT 1.0 numbering runtime. No corpus bytes or expected results changed.

## Verification

- A focused runtime test proves sibling source focus produces attribute values
  `1` and `2` through the shared number evaluator.
- A focused inspection test proves nested numbering remains visible in the
  compiled semantic report.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
