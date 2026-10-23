# OASIS XSLT 1.0 Variable `document()` Reference

Date: 2026-09-29  
Status: Verified implementation and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

Unchanged `Lotus/mdocs_mdocs15#1` binds the relative reference
`mdocs15a.xml` to a stylesheet-global string variable and evaluates:

```xpath
document($typefile,/)
```

FastXSLT already supported the source-root second argument and sealed-snapshot
resolution, but rejected a variable-valued first argument during compilation.

## Implemented slice

The private XSLT 1.0 `xsl:copy-of` compiler now retains a scalar variable name,
the stylesheet-static base identity, an optional typed document base, and an
optional admitted path tail. At execution, the ordinary invocation variable
frame supplies the atomic lexical reference. The existing invocation-owned
`document()` preparation path then resolves that value against the first node
selected by the explicit base expression.

This keeps global parameter/default materialization, lexical variable shadowing,
resource resolution, parsing, caching, work charging, and diagnostics in their
existing owners. The local harness admits only `mdocs15a.xml` under the exact
case-qualified logical identity; no ambient file discovery is introduced.

## Corpus result

Unchanged `Lotus/mdocs_mdocs15#1` initializes, executes, and compares exactly.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,450 | 2,451 | +1 |
| Executed successfully | 2,400 | 2,401 | +1 |
| Initialization failures | 720 | 719 | -1 |
| Execution failures | 50 | 50 | 0 |
| Exact expected-result matches | 2,269 | 2,270 | +1 |
| Visible mismatches | 0 | 0 | 0 |
| Comparator gaps | 0 | 0 | 0 |

The conservative all-catalog exact-match ratio is now
`2,270 / 3,173 = 71.54%`. Expected-error credit remains 423 / 431.

## Boundaries

- The first argument is exactly one atomic variable reference in the private
  XSLT 1.0 `xsl:copy-of` slice.
- Non-atomic sequences, source node-sets, and temporary trees supplied through
  this variable form remain explicitly unsupported.
- The second argument and path tail retain the previously admitted typed forms.
- Resolution remains invocation-owned and sealed-snapshot-only; no live
  acquisition or cross-invocation cache is admitted.
- This is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt xslt10_copy_of_variable_document_uses_source_root_base_identity
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/mdocs_mdocs15#1'
./scripts/verify.ps1
```
