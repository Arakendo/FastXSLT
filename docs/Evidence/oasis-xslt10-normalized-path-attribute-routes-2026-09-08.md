# OASIS XSLT 1.0 normalized-path attribute routes -- 2026-09-08

Date: 2026-09-08  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can `normalize-space(path)` produce identical values through `xsl:value-of`, a
computed `xsl:attribute`, and a literal-result AVT without duplicating path or
normalization semantics?

## Changes

- One XSLT 1.0 parser recognizes `normalize-space(path)` only when its argument
  fully parses through the typed location-path grammar.
- Computed attributes and literal-result AVTs retain the same typed normalized
  path representation. AVTs may retain bounded literal text around the dynamic
  expression.
- Runtime applies XSLT 1.0 node-set conversion by selecting the first node in
  document order, then delegates source string normalization to the same
  controlled implementation used by `xsl:value-of`.
- Empty node sets produce the empty string. Source traversal, character work,
  and result construction remain charged.
- The computed-attribute compiler's value selection was extracted into a
  private helper when this tranche triggered the ADR-0004 size review threshold.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,502 | 1,503 | +1 |
| Executed successfully | 1,337 | 1,338 | +1 |
| Expected-result XML matches | 1,229 | 1,230 | +1 |
| XML comparison mismatches | 82 | 82 | 0 |
| Execution failures | 165 | 165 | 0 |

The unchanged Lotus `whitespace23` case now produces the same normalized URL
values through all three construction routes, including embedded newlines and
indentation. Its archival CRLF versus generated LF spelling is correctly handled
by the XML infoset comparator.

The strict standard-operation lower bound becomes
`1,230 / 2,742 = 44.86%`; the conservative all-catalog ratio becomes
`1,230 / 3,173 = 38.76%`. These are local compatibility measurements, not an
XSLT 1.0 conformance claim.

## Boundaries

This tranche does not admit arbitrary AVT expression composition, modern
sequence normalization, temporary-tree paths, qualified paths, or a general
computed-attribute evaluator. No corpus bytes or expected results were changed.

## Verification

- A focused runtime test covers both attribute routes, descendant string-value
  normalization, empty path selection, and surrounding result construction.
- The unchanged `whitespace23` case passes its expected XML comparison.
- The complete 3,173-case local measurement produced the counters above.
- The ordinary workspace verification gate passes.
