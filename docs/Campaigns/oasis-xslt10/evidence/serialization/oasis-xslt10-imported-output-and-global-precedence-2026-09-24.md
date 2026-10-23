# OASIS XSLT 1.0 Imported Output and Global Precedence

- Date: 2026-09-24
- Status: Verified compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

The module compiler already composed imported output method, encoding, and
indent declarations, but rejected an imported `omit-xml-declaration` property.
That was an artificial composition gap rather than a serialization limitation.

A second boundary appeared where an included module had itself imported a
lower-precedence global binding. The include merge treated that inherited
binding as though it had the included module's precedence and consequently
reported a duplicate against the principal module's higher-precedence binding.

## Implemented slice

Imported output declarations now compose `omit-xml-declaration` with the same
precedence and same-precedence shadowing behavior as the already admitted
scalar output properties.

Compiled global bindings now retain a private scalar import precedence. Module
rebasing shifts that precedence with checked arithmetic. Include composition
still rejects two same-name bindings at the same precedence, preserves a
higher-precedence binding over an inherited lower-precedence binding, and
ignores the lower binding once it is shadowed.

This is private compiled-state metadata. It adds no runtime lookup, cache,
resource authority, or public representation.

## Verification

Focused regressions prove that:

- an unshadowed imported `omit-xml-declaration` property is inherited;
- a principal output declaration shadows the same property imported through
  an include;
- a same-precedence include retains the principal binding over a binding that
  the included module inherited from an import; and
- a genuine same-precedence duplicate remains `FXST0029`.

The unchanged OASIS case
`Microsoft/Variables_VarScopeInImportedStylesheet#1` now initializes, executes,
and compares exactly. The full sweep moves from **2,316 to 2,317 initialized**,
**2,264 to 2,265 successfully executed**, and **2,102 to 2,103 / 3,173 exact
matches (66.28%)**. Initialization failures fall from 819 to 818; execution
failures and visible XML mismatches remain unchanged at 52 and 77 respectively.

`Microsoft/BVTs_bvt044#1` now passes imported output-property composition and
reaches a later unsupported global-expression boundary involving a
higher-precedence binding that references its shadowed lower-precedence
binding. It remains uncredited. This slice does not retain complete shadow
chains or admit that self-reference behavior.

## Reproduction

```powershell
cargo test -p fastxslt --all-features imported_omit_xml_declaration_is_inherited_when_unshadowed
cargo test -p fastxslt --all-features principal_output_shadows_output_imported_through_an_include
cargo test -p fastxslt --all-features same_precedence_include_keeps_higher_binding_over_an_inherited_import
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Variables_VarScopeInImportedStylesheet'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'BVTs_bvt044'
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier 'FXST0029'
```
