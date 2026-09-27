# OASIS XSLT 1.0 Empty HTML Output Version

- Date: 2026-09-25
- Status: Verified compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

The XSLT 1.0 corpus contains an HTML output declaration whose legacy `version`
attribute is empty. FastXSLT retained that value as serializer metadata, then
rejected it as an unsupported HTML version during execution. The corpus expects
the empty compatibility value to behave like an unspecified version.

This was the only remaining `SESU0013` execution frontier.

## Implemented slice

When and only when XSLT 1.0 compatibility is selected and the effective output
method is explicitly HTML, an all-whitespace `version` value now compiles as an
unspecified serialization version. Modern stylesheets continue to retain the
same lexical value, so the existing serializer validation and `SESU0013`
boundary remain unchanged for modern semantics.

The change neither broadens supported HTML versions nor changes method
selection, result construction, resource authority, or the public API.

## Verification

A focused compiler regression proves that the empty value becomes absent under
XSLT 1.0 compatibility while remaining present in a version 3.0 stylesheet.

The unchanged OASIS case `Microsoft/Output__84306#1` now initializes, executes,
and compares exactly. The full sweep remains at **2,317 initialized**, moves
from **2,265 to 2,266 successfully executed**, and moves from **2,103 to 2,104
/ 3,173 exact matches (66.31%)**. Execution failures fall from 52 to 51;
initialization failures and visible XML mismatches remain unchanged at 818 and
77 respectively. The `SESU0013` execution frontier is eliminated.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_empty_html_version_uses_the_unspecified_legacy_default
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Output__84306'
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier 'SESU0013'
```
