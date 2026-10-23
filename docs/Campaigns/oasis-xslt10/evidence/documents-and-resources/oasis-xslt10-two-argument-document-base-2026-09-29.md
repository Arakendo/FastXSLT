# OASIS XSLT 1.0 Two-Argument `document()` Base Identity

Date: 2026-09-29  
Status: Verified implementation and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

XSLT 1.0 permits the second argument of `document()` to select the base URI
used to resolve a relative string first argument. The unchanged
`Lotus/mdocs_mdocs03#1` supplies a literal relative reference and a source
node-set base. `Lotus/reluri_reluri09#1` instead supplies another literal
`document()` result as the base node-set. `Lotus/reluri_reluri10#1` combines a
source-node-set first argument with that literal-document base. FastXSLT
previously rejected every two-argument form.

## Implemented slice

The private XSLT 1.0 `xsl:copy-of` compiler now admits:

```xpath
document('relative-reference', source-location-path)
document(source-location-path, document('base-reference'))
```

The first argument is retained as either one literal string or one typed source
location path. The base selection is retained as one source location path or
one literal `document()` reference. At execution, FastXSLT evaluates or
prepares the base selection and uses its first node's logical resource identity
for every first-argument reference. An empty source base node-set produces an
empty result. Resolution and preparation then use the existing invocation-owned,
budgeted, sealed-snapshot path.

The local harness admits only `mdocs03a.xml` for the exact OASIS case under a
case-qualified logical identity. The `reluri09` resources were already exact
catalog-declared supplemental data and require no overlay. The `reluri10`
overlay admits only the final target omitted by its catalog entry. No filesystem
discovery, network lookup, live resolver, or ambient authority is introduced.

## Corpus result

Unchanged `Lotus/mdocs_mdocs03#1`, `Lotus/reluri_reluri09#1`, and
`Lotus/reluri_reluri10#1` initialize, execute, and compare exactly.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,447 | 2,450 | +3 |
| Executed successfully | 2,397 | 2,400 | +3 |
| Initialization failures | 723 | 720 | -3 |
| Execution failures | 50 | 50 | 0 |
| Exact expected-result matches | 2,266 | 2,269 | +3 |
| Visible mismatches | 0 | 0 | 0 |
| Comparator gaps | 0 | 0 | 0 |

The conservative all-catalog exact-match ratio is now
`2,269 / 3,173 = 71.51%`. Expected-error credit remains 423 / 431.

## Boundaries

- The first argument is a compile-time string literal or one typed source
  location path.
- The second argument is one admitted source location path or one literal
  `document()` call. Other nested calls, variables, functions, unions, and
  computed strings remain outside this slice.
- The base identity comes from the selected source node, not a host path or
  content fingerprint.
- Resolution is invocation-owned and snapshot-only. Missing and denied
  resources retain structured outcomes.
- This is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt xslt10_copy_of_literal_document_uses_source_node_set_base_identity
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/mdocs_mdocs03#1'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/reluri_reluri09#1'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/reluri_reluri10#1'
./scripts/verify.ps1
```
