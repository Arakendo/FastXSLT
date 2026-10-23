# OASIS XSLT 1.0 HTML Attribute Line and Layout Policy

- Date: 2026-09-26
- Status: Verified non-pass corpus classification
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Microsoft `Output_EntityRefInAttribHtml` constructs one HTML `out` element
whose attribute contains a literal line break. FastXSLT emits the required
attribute value without manufacturing a numeric character reference. The
catalog reference uses CRLF where the stylesheet's XML value has already been
normalized to LF and inserts an additional formatting newline between the
element's start and end tags.

The suite-owned doubts metadata explicitly warns of newline/normalization
problems. After XML attribute normalization the attribute values are
semantically equal; the remaining difference is the historical serializer's
optional element layout.

## Disposition

The identity receives the exact `serialization-layout-policy-excluded`
disposition. The comparator is not weakened to discard arbitrary mixed-content
whitespace, and FastXSLT does not imitate one serializer's discretionary
formatting. The case remains visible in the conserved denominator and receives
no pass credit.

## Measurement effect

The unchanged sweep remains at 2,342 initialized cases, 2,291 successful
executions, and 2,126 exact XML comparisons out of 3,173 (67.00%). Visible XML
mismatches fall from eight to seven, serialization-layout-policy exclusions
rise from four to five, and no doubt-annotated XML mismatch remains.

## Verification

```powershell
cargo test -p fastxslt --all-features oasis_serialization_layout_policy_exclusion_is_exact_and_bounded
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Output_EntityRefInAttribHtml#1'
./scripts/verify.ps1
```
