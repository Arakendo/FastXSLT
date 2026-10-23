# OASIS XSLT 1.0 Catalog Resource-Identity Normalization

Date: 2026-09-22  
Status: Local harness evidence

## Question

Does the local OASIS adapter give supplemental stylesheets the same normalized
logical identities that XSLT include/import resolution produces?

## Method

- Keep the locally acquired OASIS archive and its catalog immutable.
- Interpret catalog stylesheet fields as archive-relative filenames rather than
  already parsed URI references.
- Escape literal `#` and `?` filename characters, then resolve each filename
  against the case's synthetic logical base through FastXSLT's ordinary
  reference resolver.
- Add a focused regression for both parent-segment normalization and a literal
  hash character in a catalog filename.
- Trace the unchanged `Lotus/impincl_impincl04#1` case and rerun the complete
  3,173-case catalog.

## Result

The supplemental `../impincl-test/impincl04.xsl` resource is now admitted as
`https://oasis.invalid/impincl-test/impincl04.xsl`, matching the identity that
the included stylesheet reference resolves to. The case leaves its false
`FXRS0002` missing-resource initialization failure and reaches XML comparison;
its remaining difference is visible whitespace, so it is not counted as an
exact match.

The complete sweep initializes 2,153 cases, reports 982 initialization
failures, executes 2,097 cases successfully, and reports 56 execution
failures. Exact XML-semantic matches reach 1,946, with 61 mismatches. The strict
compatibility lower bound is `1,946 / 3,173 = 61.33%`.

## Boundaries

- This is a local corpus-adapter repair, not a change to engine resource
  authority or resolution semantics.
- Catalog fields remain filesystem/archive names only at acquisition time;
  engine compilation still consumes normalized logical identities and sealed
  in-memory bytes.
- No filesystem or network access is added during compilation or execution.
- The result is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features oasis_supplemental_stylesheet_identity_normalizes_parent_segments
./scripts/measure-oasis-xslt10.ps1 -TraceCase Lotus/impincl_impincl04#1
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
