# OASIS XSLT 1.0 Lossy Single-Byte `format-number()` References

Date: 2026-09-26  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

After correcting XPath 1.0 double conversion, do the five remaining Microsoft
`format-number()` mismatches expose formatter defects or unusable physical
reference output?

## Findings

All five stylesheets request XML output encoded as ISO-8859-1 while their
source data deliberately contains characters outside that encoding.

`defaultPattern`, `EuropeanPattern`, and `Non_DigitPattern` place U+2030
PER MILLE SIGN in result text. Their expected files contain raw byte `0x89`.
Under ISO-8859-1 that byte denotes U+0089, which is not a permitted XML 1.0
character; it is not U+2030. The artifacts appear to rely on a Windows code
page while declaring ISO-8859-1.

`Pattern-separator` and `percentPattern` exercise U+FFFD and U+1234 in picture
prefixes and suffixes. Their expected files replace both distinct characters
with literal `?`, losing the result characters and making semantic comparison
impossible.

FastXSLT's bounded ISO-8859-1 serializer emits numeric character references
for characters the physical encoding cannot represent. That preserves the
result-tree characters when the XML is reparsed.

## Disposition

The five exact identities are classified as
`unusable-reference-result-excluded`. They remain in the conserved denominator
and receive no pass credit. The upstream files are unchanged, and the engine
does not adopt Windows-1252 substitution under an ISO-8859-1 declaration.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,342 | 2,342 | 0 |
| Executed successfully | 2,291 | 2,291 | 0 |
| Exact XML-semantic matches | 2,125 | 2,125 | 0 |
| XML comparison mismatches | 64 | 59 | -5 |
| Unusable reference-result exclusions | 8 | 13 | +5 |

The strict exact lower bound remains 2,125 / 3,173 (66.97%).

## Verification

- The classification helper is exact and bounded to the five named cases.
- Byte inspection confirms eight raw `0x89` bytes in each of the first three
  reference files and no representable U+2030 value.
- Text inspection confirms lossy `?` replacement in the remaining two files.
- Focused formatter tests preserve the independently repaired XPath 1.0 double
  conversion and modern exact-decimal behavior.
- The complete unchanged catalog was rerun with zero panics.
