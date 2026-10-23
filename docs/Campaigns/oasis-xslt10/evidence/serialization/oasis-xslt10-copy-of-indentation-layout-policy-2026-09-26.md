# OASIS XSLT 1.0 Copy-of Indentation Layout Policy

- Date: 2026-09-26
- Status: Verified non-pass corpus classification
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Lotus `whitespace_whitespace17` copies `<doc><foo>a</foo></doc>` and requests
`xsl:output indent="yes"`. FastXSLT and the catalog reference produce the same
document, element names, hierarchy, and text. The reference inserts a newline
without horizontal indentation before `foo`; FastXSLT inserts a newline plus
two spaces.

XSLT 1.0 makes indentation method-dependent and requires processors to avoid
changing the result tree's meaning. It does not prescribe one horizontal
indentation width.

## Disposition

The identity receives the exact `serialization-layout-policy-excluded`
disposition. FastXSLT keeps its deterministic serializer layout and does not
weaken semantic XML comparison for arbitrary whitespace-sensitive content. The
case remains in the conserved denominator and receives no pass credit.

## Measurement effect

The unchanged sweep remains at 2,342 initialized cases, 2,291 successful
executions, and 2,126 exact XML comparisons out of 3,173 (67.00%). Visible XML
mismatches fall from seven to six, and serialization-layout-policy exclusions
rise from five to six.

## Verification

```powershell
cargo test -p fastxslt --all-features oasis_serialization_layout_policy_exclusion_is_exact_and_bounded
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/whitespace_whitespace17#1'
./scripts/verify.ps1
```
