# OASIS XSLT 1.0 Static `string()` Global Defaults

- Date: 2026-09-24
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

The host-bounded dependency-depth tranche exposed two unchanged deep import
chains whose leaf modules declared global parameters using
`select="string('literal')"`. Static string folding already implemented the
expression exactly, but global-default compilation fell through to the
location-path parser and reported it as unsupported syntax.

## Implemented slice

Untyped global variables and parameters now reuse the existing bounded static
`string()` folder before attempting numeric, boolean, or location-path
compilation. Both single- and double-quoted literal arguments compile to the
existing immutable text-global representation. No runtime evaluation path,
conversion rule, value kind, or XSLT-version special case was added.

A focused compiler test proves both quote forms retain their exact text values.

## Corpus result

The unchanged cases `Lotus/impincl_impincl03#1` and
`Lotus/impincl_impincl11#1` now traverse their sealed four-module dependency
chains and compare exactly. The complete sweep moves from:

- 2,286 to 2,288 initialized cases;
- 2,234 to 2,236 successfully executed cases; and
- 2,074 to 2,076 exact expected-result matches.

Initialization failures fall from 849 to 847. Execution failures and
comparison mismatches remain 52 and 77. The exact compatibility lower bound is
therefore **2,076 / 3,173 (65.43%)**.

## Non-claims

This does not admit dynamic `string()` evaluation for arbitrary global values,
general global expressions, or a second compatibility evaluator. It reuses a
context-free XPath operation whose result is known during compilation. The
result remains compatibility evidence, not an XSLT 1.0 conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features folds_static_string_function_global_defaults
./scripts/measure-oasis-xslt10.ps1 -TraceCase Lotus/impincl_impincl03#1
./scripts/measure-oasis-xslt10.ps1 -TraceCase Lotus/impincl_impincl11#1
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
