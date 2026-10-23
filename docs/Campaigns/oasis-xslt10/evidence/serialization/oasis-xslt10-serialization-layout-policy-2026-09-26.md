# OASIS XSLT 1.0 Serialization Layout Policy

- Date: 2026-09-26
- Status: Verified non-pass corpus classification
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Four successfully executed cases produce the intended elements, expanded
names, attributes, values, and HTML attribute escaping but differ from their
archival expected files only in serializer-added whitespace:

- `Attributes__78386`;
- `AttributeSets_AttributeSets_WithPI`;
- `Messages__91758`; and
- `Output_HtmlOutputWithLessThanInAttribute`.

The expected files require one historical processor's indentation shape,
including line feeds inside otherwise empty elements and a specific nesting
layout. FastXSLT's XML/HTML serialization does not add those exact text nodes.
The semantic purpose of each case is nevertheless reached: namespace-aware
computed attributes, a processing instruction inside an attribute-set
declaration, non-terminating `xsl:message`, and a literal less-than character in
an HTML attribute respectively.

## Disposition

XSLT 1.0 `indent="yes"` permits a processor to add whitespace and does not
standardize a byte- or tree-exact pretty-printing algorithm. HTML output also
does not make one historical indentation layout portable. These four cases now
receive exact `serialization-layout-policy-excluded` dispositions.

This is not pass credit and does not cause the XML comparator to ignore authored
whitespace generally. The cases remain named in the conserved 3,173-case
denominator, while the engine's semantic result remains distinct from this
serializer policy.

## Measurement effect

The unchanged sweep remains at 2,342 initialized cases, 2,291 successful
executions, and 2,125 exact XML comparisons out of 3,173 (66.97%). Visible XML
mismatches fall from 18 to 14, and four serialization-layout-policy exclusions
are added.

## Verification

The adapter has an exact bounded-list control for these four identities and a
negative control for the remaining HTML entity-reference mismatch. The normal
serializer suites continue to verify the currently supported XML and HTML
output behavior.

```powershell
cargo test -p fastxslt --all-features oasis_serialization_layout_policy_exclusion_is_exact_and_bounded
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
