# OASIS XSLT 1.0 Extension Fallback

Date: 2026-09-22  
Status: Local compatibility evidence

## Question

Can an unavailable extension element execute its standard `xsl:fallback`
children without granting extension authority, treating extension content as a
literal result, or adding an extension execution API?

## Method

- Detect elements whose namespace is named by the effective
  `extension-element-prefixes` declaration.
- Compile only direct `xsl:fallback` child sequence constructors, in document
  order.
- Discard all non-fallback extension content rather than compiling it as
  ordinary XSLT or literal result content.
- Retain explicit `FXST1059` when no fallback exists.
- Validate the fallback instruction's admitted attributes and compile its body
  through the ordinary sequence compiler.
- Rerun the unchanged 3,173-case OASIS catalog.

## Result

Eight cases leave the `FXST1059` frontier. Six initialize, execute, and compare
exactly; the remaining two expose distinct later static boundaries rather than
being credited:

- one uses an unsupported control attribute on `xsl:fallback`;
- one declares an unbound default extension prefix.

Three unavailable-extension cases without fallback retain explicit
`FXST1059`. The conserved totals are 2,149 initialized cases, 986 initialization
failures, 2,089 successful executions, 60 execution failures, 1,938 exact
XML-semantic matches, and 61 mismatches. The strict compatibility lower bound
is `1,938 / 3,173 = 61.08%`.

## Boundaries

- FastXSLT still executes no extension element or extension function.
- This does not create a host callback, extension registration surface, or new
  resource authority.
- An unavailable extension without fallback remains explicitly unsupported.
- General forwards-compatible processing remains outside this tranche.

## Reproduction

```powershell
cargo test -p fastxslt --all-features declared_extension_elements_compile_only_standard_fallback_content
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier FXST1059
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
