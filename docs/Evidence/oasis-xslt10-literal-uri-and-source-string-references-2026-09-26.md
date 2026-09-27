# OASIS XSLT 1.0 Literal URI and Source String References

Date: 2026-09-26  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Do three remaining Microsoft mismatches identify FastXSLT semantic defects, or
do their immutable expected results disagree with the stylesheet/source data?

## Findings

### `Include_RelUriTest5`

The included stylesheet constructs the literal text `../import1.xsl`. FastXSLT
preserves that literal. The expected result instead contains
`..\import1.xsl`, replacing the URI path separator with a Windows filesystem
separator. Matching the reference would require host-platform path rewriting
of result text and would contradict the stylesheet bytes.

### `Keys__91726` and `Keys__91727`

Both cases select the same source `p` element through `key()` and copy its
XPath string-value. The source ends the paragraph with:

```xml
web applications.
<!-- more paragraphs -->
</p>
```

The comment contributes no string-value, but the text nodes on both sides of
it contribute their normalized line feeds. FastXSLT therefore ends the copied
value with two line feeds. The expected files instead retain only the first
line feed and invent one space before `</LI>`. That space is absent from the
source tree.

## Disposition

The three exact case identities are classified as
`unusable-reference-result-excluded`. They remain in the conserved denominator
and receive no pass credit. The upstream files remain unchanged, URI literals
remain host-neutral, and source-node string-value semantics are not weakened.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,342 | 2,342 | 0 |
| Executed successfully | 2,291 | 2,291 | 0 |
| Exact XML-semantic matches | 2,125 | 2,125 | 0 |
| XML comparison mismatches | 57 | 54 | -3 |
| Unusable reference-result exclusions | 15 | 18 | +3 |

The strict exact lower bound remains 2,125 / 3,173 (66.97%).

## Verification

- Focused comparison traces isolate the forward-slash/backslash difference and
  the two source-string tail differences.
- Direct inspection of the immutable stylesheets, source, and expected files
  confirms the contradictions.
- The classification helper is exact and bounded to the three named identities.
- The complete unchanged catalog was rerun with zero panics.
