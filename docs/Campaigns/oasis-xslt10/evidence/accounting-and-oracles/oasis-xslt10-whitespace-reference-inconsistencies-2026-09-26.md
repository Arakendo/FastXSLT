# OASIS XSLT 1.0 Whitespace Reference Inconsistencies

- Date: 2026-09-26
- Status: Verified non-pass corpus classification
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Five Microsoft whitespace cases execute successfully but their selected
references do not form a coherent XSLT/XDM oracle.

Cases `Whitespaces__91422`, `91423`, `91425`, and `91428` require whitespace
that originated in a source CDATA section to remain visible even though the
stylesheet strips whitespace-only text nodes from `span`. CDATA boundaries do
not survive into the XPath/XSLT data model; the whitespace is an ordinary text
node at that boundary. The references also change unrelated source-document
string values between otherwise identical stylesheets whose relevant change is
only `xml:space` on a temporary variable or output indentation.

Case `Whitespaces__91453` reverses the same expectation and, more importantly,
applies imported whitespace declarations over higher-precedence declarations
in the principal stylesheet. Its expected result strips `span` according to an
imported `xsl:strip-space` and preserves `p` according to an imported
`xsl:preserve-space`. The principal stylesheet explicitly says and declares
the opposite: principal `xsl:preserve-space elements="*"` overrides imported
strip rules, while principal `xsl:strip-space elements="p"` overrides the
imported preserve rule.

## Disposition

All five identities receive exact `unusable-reference-result-excluded`
dispositions. FastXSLT retains XDM text-node semantics and import-precedence
composition rather than reproducing mutually incompatible archival behavior.
The cases remain visible in the conserved denominator and receive no pass
credit.

## Measurement effect

The unchanged sweep remains at 2,342 initialized cases, 2,291 successful
executions, and 2,126 exact XML comparisons out of 3,173 (67.00%). Visible XML
mismatches fall from six to one, and unusable-reference-result exclusions rise
from 40 to 45.

## Verification

```powershell
cargo test -p fastxslt --all-features oasis_unusable_reference_result_exclusion_is_exact_and_bounded
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
