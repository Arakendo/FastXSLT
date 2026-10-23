# OASIS XSLT 1.0 Unicode Alias and Boolean Sort Key

- Date: 2026-09-25
- Status: Verified compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

The remaining OASIS execution frontier included three Microsoft cases using
the historical output-encoding label `unicode`. FastXSLT already had a bounded
UTF-16 byte serializer, but treated the compatibility spelling as an unrelated
unsupported encoding.

The generic unsupported XPath frontier also included an otherwise executable
case whose secondary `xsl:sort` key is the XPath 1.0 node-set/string comparison
`last-name = 'Bob'`. The ordinary location-path sort evaluator could not
represent the boolean result of that expression.

## Implemented slices

Only under XSLT 1.0 compatibility, the case-insensitive output-encoding label
`unicode` now compiles to the existing `UTF-16` byte lane. Modern stylesheets
retain the original lexical label and therefore retain the existing unsupported
encoding boundary. No new encoder or general encoding alias registry was
introduced.

XSLT 1.0 sort compilation now admits a bounded child location path compared
with a string literal by `=` or `!=`. Runtime evaluation preserves XPath 1.0's
existential node-set comparison, charges every candidate comparison and
controlled string-value traversal, and emits the ordinary boolean lexical sort
key. The complete location-path sort machinery remains shared.

## Verification

Focused tests prove that:

- `unicode` normalizes to `UTF-16` in an XSLT 1.0 static context while a version
  3.0 stylesheet retains `unicode`;
- a multi-node child path uses existential comparison rather than first-node
  comparison when producing the boolean sort key.

The three unchanged Microsoft encoding cases now initialize and execute, so
the `SESU0007` frontier falls from ten to seven. They expose already visible
source/result whitespace differences and remain uncredited as three additional
XML mismatches.

Unchanged `Microsoft/BVTs_bvt021#1` now initializes, executes, and compares
exactly. The full sweep moves from **2,317 to 2,318 initialized**, from **2,266
to 2,270 successfully executed**, and from **2,104 to 2,105 / 3,173 exact
matches (66.34%)**. Initialization failures fall from 818 to 817, execution
failures fall from 51 to 48, visible XML mismatches rise from 77 to 80, and
comparator-unsupported cases remain 77. No later-frontier movement is counted
as an exact pass.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_normalizes_the_historical_unicode_output_encoding_alias_only
cargo test -p fastxslt --all-features xslt10_sort_uses_an_existential_child_path_string_comparison
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/BVTs_bvt021'
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier 'unsupported/SESU0007/SESU0007'
```
