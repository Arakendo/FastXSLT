# OASIS XSLT 1.0 Unusable Attribute-Whitespace References

Date: 2026-09-26  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Do two Microsoft attribute-construction mismatches expose FastXSLT character
normalization defects, or do their immutable reference files contradict the
characters constructed by their stylesheets?

## Findings

### `Microsoft/Attributes__78365`

The stylesheet's namespace attribute value template contains the character
reference `&#13;`. XML end-of-line normalization does not replace a carriage
return introduced through a character reference. FastXSLT retains that
carriage return and the XML serializer emits it as `&#xD;`.

The expected result instead contains `&#xA;` at the corresponding position.
Matching it would require changing an explicit carriage return into a line
feed after stylesheet parsing.

### `Microsoft/Attributes__78372`

The `xsl:attribute` sequence constructor contains tabs around non-whitespace
text. FastXSLT preserves the constructed tabs and escapes them as `&#x9;` in the
serialized attribute so that reparsing retains their identity.

The expected result writes literal tab bytes in the XML attribute. XML
attribute-value normalization necessarily turns those literal tabs into
spaces when the expected document is parsed. The expected artifact therefore
cannot represent the constructed tab-valued attribute under its declared XML
comparison mode.

## Disposition

Both exact case identities are classified as
`unusable-reference-result-excluded`. They remain in the conserved 3,173-case
denominator and receive no pass credit. The immutable upstream artifacts are
not edited, and FastXSLT's parser or serializer behavior is not weakened to
match them.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,342 | 2,342 | 0 |
| Executed successfully | 2,291 | 2,291 | 0 |
| Exact XML-semantic matches | 2,125 | 2,125 | 0 |
| XML comparison mismatches | 66 | 64 | -2 |
| Unusable reference-result exclusions | 6 | 8 | +2 |

The strict exact lower bound remains 2,125 / 3,173 (66.97%).

## Verification

- The exclusion helper is exact and bounded to named case identities.
- Focused traces retain `&#xD;` for the explicit carriage return and `&#x9;` for
  constructed tabs.
- The complete unchanged catalog was rerun with zero panics.
