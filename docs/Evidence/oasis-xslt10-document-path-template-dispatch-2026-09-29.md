# OASIS XSLT 1.0 Document-Path Template Dispatch

Date: 2026-09-29  
Status: Verified implementation and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

Unchanged `Lotus/mdocs_mdocs05#1` applies templates through two external
document paths:

```xpath
document(pointer/urlref/@urlstr)/market.participant/business.identity.group/business.name
document('../mdocs/compu.xml')/market.participant/address.set/*
```

The first reference is selected from a principal-source attribute. The second
is a literal relative reference. Both calls continue through typed child and
wildcard steps before template dispatch.

## Implemented slice

The private XSLT 1.0 apply-selection plan can retain an admitted typed trailing
location path after either a one-argument source-selected `document()` call or
a literal `document()` call. Compilation keeps external acquisition separate
from the ordinary XPath grammar: the `document()` prefix becomes a typed
external-selection plan and only the suffix is compiled as a location path.

Execution resolves references only through the sealed invocation snapshot,
prepares each selected document under the existing work and cancellation
controls, and evaluates the trailing path from that document's document node.
Every selected node retains its owning-document provenance. Source-selected
documents are deduplicated by opaque invocation-local identity, then combined
into one aggregate template focus. No filesystem access, network access, live
resolver, or cross-invocation cache is introduced.

The local harness admits only archival `source/compu.xml` under its reviewed
source-relative logical identity for this case.

## Corpus result

Unchanged `Lotus/mdocs_mdocs05#1` initializes, executes, and compares exactly.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,456 | 2,457 | +1 |
| Executed successfully | 2,406 | 2,407 | +1 |
| Initialization failures | 714 | 713 | -1 |
| Execution failures | 50 | 50 | 0 |
| Exact expected-result matches | 2,275 | 2,276 | +1 |
| Visible mismatches | 0 | 0 | 0 |
| Comparator gaps | 0 | 0 | 0 |

The conservative all-catalog exact-match ratio is now
`2,276 / 3,173 = 71.73%`. Expected-error credit remains 423 / 431.

## Boundaries

- The source-selected reference and both trailing selections use the existing
  typed location-path subset.
- External preparation remains invocation-owned and sealed-snapshot-only.
- The slice does not admit computed reference strings, live acquisition,
  arbitrary function composition around `document()`, or source-node runtime
  values without owning-document provenance.
- This is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_apply_templates_navigates_typed_paths_in_source_selected_and_literal_documents
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/mdocs_mdocs05#1'
./scripts/verify.ps1
```
