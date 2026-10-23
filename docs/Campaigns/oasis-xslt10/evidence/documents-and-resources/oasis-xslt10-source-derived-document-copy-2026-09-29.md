# OASIS XSLT 1.0 Source-Derived `document()` Copy

Date: 2026-09-29  
Status: Verified implementation and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

Three unchanged standard-operation cases use a source node-set as the sole
argument to `document()` inside `xsl:copy-of`. The selected source nodes contain
relative resource references, repeated references must not duplicate document
nodes, and two cases then select `//body` across the resulting documents.

FastXSLT already resolved literal `document()` references against a sealed
snapshot, but rejected these source-derived references during compilation.

## Implemented slice

The XSLT 1.0 `xsl:copy-of` compiler now retains one typed source location path
as a `document()` argument, plus an optional admitted path tail. During an
invocation the runtime:

- evaluates the reference path against the effective source view;
- obtains each selected node's controlled string value;
- resolves that value relative to the selected node's logical source identity;
- acquires bytes only from the host-sealed resource snapshot;
- prepares each resolved document under the existing invocation work budget;
- deduplicates repeated resolved documents by invocation-local document
  identity; and
- copies either each document node or its selected path in stable reference
  order through the ordinary charged source-copy path.

No filesystem or network lookup occurs. The compiled stylesheet retains no
source value, resolved resource, prepared external tree, or invocation state.

The archival catalog omits the four sibling data files named by these source
documents. The local measurement harness therefore has an exact three-case
overlay admitting only `mdocs04a.xml`, `mdocs04b.xml`, `mdocs06a.xml`, and
`mdocs06b.xml` under case-qualified logical identities. This is explicit
corpus-adapter authority; it is not general source-driven file discovery.

## Corpus result

The following unchanged cases initialize, execute, and compare exactly:

- `Lotus/mdocs_mdocs04#1`;
- `Lotus/mdocs_mdocs07#1`; and
- `Microsoft/XSLTFunctions_DocumentInUnionWithDuplicateNodes#1`.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,444 | 2,447 | +3 |
| Executed successfully | 2,394 | 2,397 | +3 |
| Initialization failures | 726 | 723 | -3 |
| Execution failures | 50 | 50 | 0 |
| Exact expected-result matches | 2,263 | 2,266 | +3 |
| Visible mismatches | 0 | 0 | 0 |
| Comparator gaps | 0 | 0 | 0 |

The conservative all-catalog exact-match ratio is now
`2,266 / 3,173 = 71.42%`. Expected-error credit remains 423 / 431.

## Boundaries

- Only a single source location-path argument is admitted; a second
  `document()` argument and nested `document()` calls remain unsupported.
- Resolution is invocation-owned and snapshot-only. Missing and denied
  resources retain structured outcomes.
- Deduplication is local to one invocation and uses document identity, not
  lexical URI equality or content fingerprints.
- Cross-document node sequences are not stored in globals, prepared inputs, or
  compiled state.
- The overlay admits exact reviewed corpus files only and does not infer host
  authority from arbitrary source text.
- This is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt xslt10_copy_of_source_document_references_deduplicates_and_uses_sealed_resources
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/mdocs_mdocs04#1'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/mdocs_mdocs07#1'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/XSLTFunctions_DocumentInUnionWithDuplicateNodes#1'
./scripts/verify.ps1
```
