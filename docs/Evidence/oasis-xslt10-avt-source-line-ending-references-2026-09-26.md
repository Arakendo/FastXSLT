# OASIS XSLT 1.0 AVT Source-Line-Ending References

Date: 2026-09-26  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Should attribute value templates that convert a source element or document to
its string value preserve the source file's physical CRLF byte pairs?

## Findings

Microsoft `AVTs__77574` and `AVTs__77591` both read source XML files whose
physical lines use CRLF. Their AVTs convert source nodes containing that text
to strings and place the strings in result attributes.

XML parsing normalizes physical CRLF to LF before the XDM text nodes and their
string values exist. FastXSLT therefore retains LF and escapes it as `&#xA;` in
the serialized attributes.

The expected files instead reconstruct the original physical CRLF pairs with
`&#xD;&#xA;`. This asserts a pre-XML byte representation that is no longer present
in the parsed source tree. Matching it would require FastXSLT to bypass XML
line-end normalization or preserve ambient parser/source-byte behavior in XDM.

## Disposition

The two exact case identities are classified as
`unusable-reference-result-excluded`. They remain in the conserved denominator
and receive no pass credit. The upstream bytes remain unchanged, and the
engine's XML parser semantics are not weakened.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,342 | 2,342 | 0 |
| Executed successfully | 2,291 | 2,291 | 0 |
| Exact XML-semantic matches | 2,125 | 2,125 | 0 |
| XML comparison mismatches | 59 | 57 | -2 |
| Unusable reference-result exclusions | 13 | 15 | +2 |

The strict exact lower bound remains 2,125 / 3,173 (66.97%).

## Verification

- Byte inspection confirms 160 CRLF pairs in `books.xml` and 2,069 CRLF pairs
  in `64K.xml`.
- Focused comparison traces show FastXSLT's normalized `&#xA;` values against
  the references' reconstructed `&#xD;&#xA;` values.
- The classification helper is exact and bounded to the two named identities.
- The complete unchanged catalog was rerun with zero panics.
