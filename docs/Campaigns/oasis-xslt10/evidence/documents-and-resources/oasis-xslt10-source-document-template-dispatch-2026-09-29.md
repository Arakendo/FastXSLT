# OASIS XSLT 1.0 Source-Document Template Dispatch

Date: 2026-09-29  
Status: Verified implementation and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

Unchanged `Lotus/mdocs_mdocs08#1` applies templates to descendants drawn from
two documents whose relative references are supplied by nodes in the principal
source. Unchanged `Lotus/mdocs_mdocs18#1` counts the same kind of selection and
sorts its aggregate template focus:

```xpath
document(places)//body
count(document(a)//body)
substring-after(., '-')
```

The selected nodes must retain their owning document while sharing one
aggregate template-application focus. Template parameters and `position()` /
`last()` must therefore behave as one selection rather than one focus per
resource.

## Implemented slice

The private XSLT 1.0 apply-selection plan now admits one typed principal-source
location path as the first `document()` argument followed by one exact
descendant element name. Each reference resolves relative to the source node
that supplied its string value and only through the invocation's sealed
snapshot. Prepared documents are invocation-owned and deduplicated by opaque
document identity.

Execution collects the selected descendants with their owning prepared
documents, establishes one aggregate focus, and dispatches the existing
template runtime with the external document as the active source. The existing
multi-document safety guard still rejects principal-source node globals or
parameters whose unqualified node IDs could otherwise be interpreted against
the wrong document. The same typed selection can be counted, and cross-document
template dispatch admits stable text sorting by the bounded
`substring-after(path, literal)` form. Sort keys are evaluated against each
node's owning document, while one aggregate focus supplies position and size.
Other cross-document sort expressions remain explicitly unsupported.

The local harness admits only the archival `mdocs04a.xml`, `mdocs04b.xml`,
`mdwords-a.xml`, and `mdwords-b.xml` siblings under source-relative logical
identities for these cases.

## Corpus result

Unchanged `Lotus/mdocs_mdocs08#1` and `Lotus/mdocs_mdocs18#1` initialize,
execute, and compare exactly.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,453 | 2,455 | +2 |
| Executed successfully | 2,403 | 2,405 | +2 |
| Initialization failures | 717 | 715 | -2 |
| Execution failures | 50 | 50 | 0 |
| Exact expected-result matches | 2,272 | 2,274 | +2 |
| Visible mismatches | 0 | 0 | 0 |
| Comparator gaps | 0 | 0 | 0 |

The conservative all-catalog exact-match ratio is now
`2,274 / 3,173 = 71.67%`. Expected-error credit remains 423 / 431.

## Boundaries

- The first argument is one typed principal-source location path.
- The result selection is one exact descendant element name.
- Cross-document sorting is limited to stable text ordering by
  `substring-after(path, literal)`; other sort expressions, computed reference
  expressions, live acquisition, and ambient filesystem/network access remain
  unsupported.
- Source-node globals and parameters remain rejected at the external-document
  execution boundary until node values carry document provenance.
- This is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_apply_templates_to_source_selected_documents
cargo test -p fastxslt --all-features xslt10_apply_templates_sorts_descendants_across_source_selected_documents
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/mdocs_mdocs08#1'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/mdocs_mdocs18#1'
./scripts/verify.ps1
```
