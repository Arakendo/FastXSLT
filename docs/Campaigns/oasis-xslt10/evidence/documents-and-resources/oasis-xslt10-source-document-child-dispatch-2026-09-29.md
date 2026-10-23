# OASIS XSLT 1.0 Source-Document Child Dispatch

Date: 2026-09-29  
Status: Verified implementation and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

Unchanged `Lotus/mdocs_mdocs06#1` applies templates to the document-element
children of two resources selected through the two-argument form:

```xpath
document(places, second)/*
```

The first argument is a node-set of relative references. The first node in the
second argument supplies their base URI. The selected document elements form
one template-application focus and receive the same explicit template
parameter.

## Implemented slice

The private XSLT 1.0 apply-selection plan admits two typed principal-source
location paths followed by `/*`. The first path supplies reference strings; the
first node selected by the second supplies the logical base identity. Resource
resolution remains invocation-owned and sealed-snapshot-only. Prepared
documents are deduplicated by opaque identity.

Execution selects each prepared document's element children, retains the
owning document with every node, and applies templates over one aggregate
position/size focus. The existing cross-document guard continues to reject
source-node globals or parameters without document provenance. No live
resolver, filesystem access, or network access is introduced.

The local harness admits only archival `mdocs06a.xml` and `mdocs06b.xml` under
their reviewed source-relative logical identities.

## Corpus result

Unchanged `Lotus/mdocs_mdocs06#1` initializes, executes, and compares exactly.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,455 | 2,456 | +1 |
| Executed successfully | 2,405 | 2,406 | +1 |
| Initialization failures | 715 | 714 | -1 |
| Execution failures | 50 | 50 | 0 |
| Exact expected-result matches | 2,274 | 2,275 | +1 |
| Visible mismatches | 0 | 0 | 0 |
| Comparator gaps | 0 | 0 | 0 |

The conservative all-catalog exact-match ratio is now
`2,275 / 3,173 = 71.70%`. Expected-error credit remains 423 / 431.

## Boundaries

- Both arguments are typed principal-source location paths.
- Result selection is limited to document element children via `/*`.
- Sorting, arbitrary trailing paths, computed arguments, and live acquisition
  remain unsupported for this form.
- This is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_apply_templates_selects_document_children_with_an_explicit_source_base
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/mdocs_mdocs06#1'
./scripts/verify.ps1
```
