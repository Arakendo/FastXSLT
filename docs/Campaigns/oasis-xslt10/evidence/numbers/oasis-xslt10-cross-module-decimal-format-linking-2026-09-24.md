# OASIS XSLT 1.0 Cross-Module Decimal-Format Linking

- Date: 2026-09-24
- Status: Verified semantic and compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related reviews: AR-0014, AR-0019

## Pressure

The stylesheet compiler parsed and statically bound each module's
`format-number()` calls before include/import composition. A principal module
therefore could not reference a named `xsl:decimal-format` declared in an
included or imported module, even though the declaration belongs to the
assembled stylesheet package. The unchanged OASIS cases stopped at explicit
`FXST1092` rather than silently selecting the default format.

## Implemented slice

Decimal-format declarations and unresolved references are now retained while
individual stylesheet modules compile. Static binding and undeclared-format
validation occur after the bounded include/import graph has been composed.
A standalone stylesheet still finalizes immediately, so an undeclared named
format remains the same explicit `FXST1092` failure.

This changes link timing, not acquisition authority or runtime lookup.
Declarations remain immutable compiled state, references remain statically
resolved, and invocation execution performs no filesystem, network, catalog,
or live-resolver work. Existing import-precedence merging and same-precedence
conflict boundaries remain intact.

## Verification

A focused sealed-resource regression has a principal template reference the
named `periodgroup` format declared only by its included module. It compiles
through the normal resource graph and produces `-26.931,4`. The complete crate
suite also retains the standalone undeclared-format failure and existing
include/import decimal-format precedence checks.

The unchanged OASIS sweep moves five cases beyond `FXST1092`; all five execute
and compare exactly:

- `Lotus/numberformat_numberformat21#1`
- `Lotus/numberformat_numberformat22#1`
- `Lotus/numberformat_numberformat23#1`
- `Lotus/numberformat_numberformat24#1`
- `Lotus/numberformat_numberformat44#1`

Initialization rises from 2,306 to **2,311**, successful execution from 2,254
to **2,259**, and exact expected-result matches from 2,091 to
**2,096 / 3,173 (66.06%)**. Initialization failures fall from 829 to 824;
execution failures remain 52 and visible XML mismatches remain 78. The full
sweep has no initialization or execution panics.

## Reproduction

```powershell
cargo test -p fastxslt --all-features principal_named_decimal_format_reference_binds_to_included_declaration
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/numberformat_numberformat21#1'
cargo test -p fastxslt --all-features
```
