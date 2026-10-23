# OASIS XSLT 1.0 Superseded Extension Output Method

- Date: 2026-09-29
- Status: Verified private compiler and corpus evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Two Microsoft BVT stylesheets contain several same-precedence `xsl:output`
declarations. An intermediate declaration selects a namespace-qualified
extension method, but a later declaration selects the effective standard
`text` or `html` method. FastXSLT previously rejected the intermediate method
before the XSLT 1.0 declaration-order recovery path could compose the complete
effective output settings.

## Bounded implementation

For an exact-version-1.0 stylesheet only, compilation may defer rejection of a
namespace-qualified extension method when a later unnamed `xsl:output` in the
same module explicitly selects `xml`, `html`, `text`, or `xhtml`. Normal
declaration merging then replaces the superseded method. A null-namespace
unknown method remains invalid, an effective extension method remains
unsupported, named output declarations do not participate, and modern
stylesheets retain strict behavior.

This is a bounded compatibility recovery rule. It does not admit extension
serialization, infer behavior from an extension method's local name, or select
a public serializer-extension interface.

## Corpus result

- `Microsoft/BVTs_bvt070#1` now initializes, executes, and compares exactly
  using the final standard `text` method.
- `Microsoft/BVTs_bvt063#1` now executes using the final standard `html`
  method, but remains an explicit serialization-layout-policy exclusion. The
  archival Microsoft reference omits the charset parameter that XSLT 1.0
  requires in the generated HTML content-type `meta`; FastXSLT retains its
  standards-shaped serializer result.
- `Microsoft/Output__77931#1`, whose effective method is `my:xml`, remains
  explicitly unsupported as `FXST1004`.

The conserved sweep changes as follows:

- exact matches rise from 2,285 to **2,286 / 3,173 (72.05%)**;
- initialized cases rise from 2,461 to **2,463**;
- successful executions rise from 2,417 to **2,419**;
- initialization failures fall from 709 to **707**;
- execution failures remain **44**;
- serialization-layout-policy exclusions rise from 17 to **18**;
- visible mismatches and comparator gaps remain **zero**; and
- expected-error credit remains **423 / 431**.

## Verification

```powershell
cargo test -p fastxslt xslt10_output_recovery_defers_a_superseded_extension_method --all-features
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier 'unsupported/FXST1004/FXST1004'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/BVTs_bvt063#1'
./scripts/verify.ps1
```
