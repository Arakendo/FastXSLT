# OASIS XSLT 1.0 Document-Function Introspection

Date: 2026-09-22  
Status: Local compatibility evidence

## Question

Should XSLT 1.0 `function-available('document')` report the existing
sealed-snapshot `document()` capability as available?

## Method

- Keep XSLT 1.0 static introspection compile-time folded and namespace-aware.
- Add the unqualified `document` name to the existing supported-function set.
- Preserve the runtime `document()` contract: literal references resolve only
  through an explicitly supplied sealed resource snapshot, with the existing
  authority checks and invocation accounting.
- Add compiler and end-to-end execution assertions, then rerun the unchanged
  3,173-case OASIS catalog.

## Result

The unchanged Lotus `extend_extend03` case now reports both
`element-available('xsl:value-of')` and `function-available('document')` as
true and compares XML-semantically. Conserved totals remain 2,152 initialized
cases, 983 initialization failures, 2,096 successful executions, and 56
execution failures. Exact XML-semantic matches rise to 1,945 and mismatches
fall to 61. The strict compatibility lower bound is
`1,945 / 3,173 = 61.30%`.

## Boundaries

- Static availability does not grant filesystem or network authority.
- `document()` remains limited to its admitted literal-reference and sealed-
  snapshot profile; this change adds no dynamic resolver or ambient I/O.
- Qualified extension-function names remain unavailable unless separately
  implemented and reviewed.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features folds_namespace_aware_xslt10_introspection
cargo test -p fastxslt --all-features xslt10_static_introspection_is_namespace_aware_and_compiled_once
./scripts/measure-oasis-xslt10.ps1 -TraceCase Lotus/extend_extend03#1
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
