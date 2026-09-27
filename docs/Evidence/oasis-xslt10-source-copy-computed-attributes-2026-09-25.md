# OASIS XSLT 1.0 Source-Copy Computed Attributes

- Date: 2026-09-25
- Status: Verified compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Three unchanged Lotus namespace cases construct numbered namespaced attributes
inside `xsl:copy`. They cover an explicit namespace URI paired with a QName, an
in-scope QName prefix without an explicit `namespace` attribute, and an
unnamespaced lexical name paired with a newly introduced namespace URI.
FastXSLT's private source-copy compiler previously used a separate literal-only
attribute path and rejected these constructors at generic `FXST1031`.

## Implemented slice

`xsl:copy` now reuses the existing computed-attribute compiler, linker,
materializer, duplicate recovery, and namespace-fixup path rather than owning a
second restricted attribute implementation. This admits:

- qualified static attribute names resolved against stylesheet namespaces;
- explicit computed-attribute namespace URIs;
- the existing bounded `xsl:number` sequence-constructor value; and
- the existing XSLT 1.0 source-path value fallback.

The compiled copy instruction retains computed attributes directly. Runtime
materialization still occurs in the current invocation, charges the existing
work controls, applies duplicate recovery, and retains namespace bindings on
the semantic result. No new public type, authority, or alternate execution
backend is introduced.

## Verification and corpus disposition

A focused regression shallow-copies source elements while constructing a
qualified numbered attribute and proves the shared computed-attribute path is
used for each source focus.

The unchanged cases below now initialize, execute, and compare exactly:

- `Lotus/namespace_namespace44#1`;
- `Lotus/namespace_namespace45#1`; and
- `Lotus/namespace_namespace46#1`.

The complete sweep reaches **2,325 initialized**, **2,277 successfully
executed**, and **2,111 / 3,173 exact matches (66.53%)**. Initialization
failures fall to 810; execution failures remain 48, XML mismatches remain 81,
and comparator exclusions remain 77. Prefix spelling is not treated as XML
expanded-name identity.

## Reproduction

```powershell
cargo test -p fastxslt --all-features source_copy_reuses_computed_namespaced_number_attributes
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/namespace_namespace44#1'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/namespace_namespace45#1'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/namespace_namespace46#1'
```
