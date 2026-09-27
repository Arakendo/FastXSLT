# OASIS XSLT 1.0 Stylesheet Text-Boundary References

Date: 2026-09-26  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Do three remaining Microsoft whitespace mismatches reveal incorrect FastXSLT
constructor semantics, or do their expected files rely on boundaries absent
from the XSLT data model?

## Findings

### `BVTs_bvt029`

The global temporary-tree constructor places formatting whitespace immediately
before and after a CDATA section containing ` CDATA Text `. A CDATA boundary is
not a node boundary in the XSLT data model. The adjacent characters therefore
form one non-whitespace text node, so stylesheet whitespace stripping cannot
remove only the formatting characters surrounding the former CDATA section.

FastXSLT preserves that complete text node. The expected file preserves the
CDATA characters but removes adjacent characters as though the lexical CDATA
markers created an observable tree boundary.

### `Text__78272` and `Text__78275`

The two catalog cases point to materially different stylesheets but their
expected files are byte-identical. `Text__78272` necessarily constructs a
literal `<test>` element containing `<ws>` and `<case>` descendants; the shared
expected file contains none of that markup. `Text__78275` uses inherited
`xml:space="preserve"` around its sequence constructor, while the reference
removes stylesheet whitespace and source whitespace that the constructor and
built-in template processing retain.

## Disposition

The three exact identities are classified as
`unusable-reference-result-excluded`. They remain in the denominator and
receive no pass credit. FastXSLT does not expose lexical CDATA boundaries in
XDM and does not discard text protected by `xml:space="preserve"`.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,342 | 2,342 | 0 |
| Executed successfully | 2,291 | 2,291 | 0 |
| Exact XML-semantic matches | 2,125 | 2,125 | 0 |
| XML comparison mismatches | 54 | 51 | -3 |
| Unusable reference-result exclusions | 18 | 21 | +3 |

The strict exact lower bound remains 2,125 / 3,173 (66.97%).

## Verification

- Focused traces expose each exact actual/reference difference.
- Direct inspection confirms that `78272.txt` and `78275.txt` have the same
  SHA-256 digest despite the different stylesheets.
- The classification helper is exact and bounded to the three named cases.
- The unchanged full catalog was rerun with zero panics.
