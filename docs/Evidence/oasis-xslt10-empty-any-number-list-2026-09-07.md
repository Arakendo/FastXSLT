# OASIS XSLT 1.0 Empty Level-Any Number List

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Does `xsl:number level="any"` distinguish an empty number list from the
number zero when no node up to the context node matches its `count` pattern?

## Change

The level-any traversal now retains an empty string when its matching count is
zero. The existing formatter consequently emits any format prefix or suffix
without inventing a numeric token. This is distinct from an explicit
`value="0"`, which remains the number zero.

The focused case uses `count="section"` while numbering a `chapter` with
`format="i."`. It produces only the punctuation suffix (`.`), not `0.`.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,445 | 1,445 | 0 |
| Executed successfully | 1,284 | 1,284 | 0 |
| Expected-result XML matches | 1,169 | 1,170 | +1 |
| XML comparison mismatches | 80 | 79 | -1 |
| Execution failures | 161 | 161 | 0 |

The unchanged exact case is `Microsoft/Number__84694#1`. Its first `chapter`
does not match `count="section"`; the expected leading value is therefore the
format suffix `.`, after which later matching `section` nodes retain their
Roman numbering.

The strict standard-operation lower bound becomes
`1,170 / 2,742 = 42.67%`; the conservative all-catalog ratio becomes
`1,170 / 3,173 = 36.87%`. These are compatibility measurements, not an XSLT
1.0 conformance claim.

## Verification

- A focused runtime test distinguishes the empty level-any number list from
  numeric zero while preserving format punctuation.
- The complete local OASIS measurement completed with the counters above and
  no upstream corpus byte was edited.
