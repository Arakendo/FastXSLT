# OASIS XSLT 1.0 Number-Pattern Union Tranche

| Field | Value |
| --- | --- |
| Date | 2026-09-06 |
| Corpus | OASIS XSLT/XPath 1.0 CD04, locally acquired and hash-verified |
| Scope | Static unions of already admitted `xsl:number` `count` and `from` pattern atoms |
| Disposition | 13 new expected-result matches; one upstream doubt-annotated mismatch remains visible |

## Change

The private `xsl:number` compiler now represents a `|`-separated `count` or
`from` pattern as an owned typed list of the already admitted atom forms:
document root, any element, unprefixed exact element name, or
namespace-qualified exact element name. Empty alternatives and all richer
pattern syntax remain structured `FXST1050` unsupported outcomes.

The numbering executor tests a candidate against those alternatives without
changing traversal, document order, sibling position, `from` reset, or work
charging. Known compiled retention includes both the alternative vector and
each retained expanded name. This is not a general template-match-pattern
expansion.

A focused production test covers spaced and unspaced unions in both `count`
and `from`, across `level="any"` and `level="multiple"`.

## Measurement

`scripts/measure-oasis-xslt10.ps1` changed the conserved measurement as follows:

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Catalog cases | 3,173 | 3,173 | 0 |
| Initialized | 1,213 | 1,227 | +14 |
| Executed successfully | 1,002 | 1,016 | +14 |
| Expected-result XML matches | 893 | 906 | +13 |
| XML comparison mismatches | 75 | 76 | +1 |
| Execution failures | 211 | 211 | 0 |

The one newly visible mismatch is `Lotus/numbering_numbering63#1`; the corpus
marks it with doubts metadata, so it remains visible and uncredited. The strict
lower bound over the 2,742 standard-operation cases is now 33.04%. The complete
catalog ratio is 28.55%. Neither is an XSLT 1.0 conformance claim.

At this tranche checkpoint, predicate-bearing number patterns remained
explicit. The subsequent
[node-kind and attribute-predicate tranche](oasis-xslt10-number-node-kind-and-attribute-predicate-tranche-2026-09-06.md)
admits one bounded predicate form without changing these recorded union
counts.
