# OASIS XSLT 1.0 Apply-Templates Path Union

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can `xsl:apply-templates` consume a union of admitted source location paths
through the same node-identity, document-order, duplicate-removal, focus, and
work-control rules already proved for `xsl:copy-of` path unions?

## Change

The private apply-selection plan now owns a vector of typed location paths for
a top-level union. Every alternative executes through the existing controlled
path evaluator. The union operator is charged once, selected node identities
are normalized into source document order, duplicates are removed, and only
then are templates invoked with the resulting sequence's position and size.

The union tokenizer is shared with the existing `xsl:copy-of` path-union
compiler. It observes quotes, parentheses, and predicates, so only a top-level
`|` splits alternatives. It does not admit arbitrary XPath union operands,
variables, atomic values, or a general expression tree.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,343 | 1,376 | +33 |
| Executed successfully | 1,182 | 1,207 | +25 |
| Expected-result XML matches | 1,070 | 1,093 | +23 |
| XML comparison mismatches | 77 | 79 | +2 |
| Execution failures | 161 | 169 | +8 |

`Lotus/select_select43#1` is a representative unchanged exact pass: applying
templates to `ancestor::sub1|ancestor::sub2` produces the two selected
ancestors in document order.

The eight newly exposed execution failures remain structured and uncredited:
seven reach the existing unsupported source-node-kind boundary for `xsl:copy`
and one reaches the existing prefix-preserving `fn:name` boundary. The two
newly visible mismatches, `Microsoft/Number__84683#1` and
`Microsoft/Number__84694#1`, reach later numbering/result behavior and also
remain uncredited. The first carries upstream doubts metadata. No mismatch is
reclassified to obtain the pass increase.

The strict standard-operation lower bound is now
`1,093 / 2,742 = 39.86%`; the deliberately conservative all-catalog ratio is
`1,093 / 3,173 = 34.45%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- A focused runtime test reverses and duplicates attribute-path alternatives,
  then proves template application sees the deduplicated source document order
  and correct sequence focus.
- A focused tokenizer test proves nested and quoted `|` characters do not
  become top-level union separators.
- The pre-existing `xsl:copy-of` path-union oracle continues through the same
  tokenizer and shared controlled evaluation rules.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
