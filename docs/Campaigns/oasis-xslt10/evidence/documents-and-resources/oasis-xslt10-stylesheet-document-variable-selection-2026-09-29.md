# OASIS XSLT 1.0 Stylesheet-Document Variable Selection

Date: 2026-09-29  
Status: Verified implementation and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

Unchanged `Lotus/select_select67#1` and `Lotus/select_select68#1` select
template elements from the stylesheet document through this XSLT 1.0 shape:

```xpath
document('')/*/xsl:template[@name=$whichtmplt]
```

The first case binds `$whichtmplt` to a string. The second binds it to a node
in the principal source and later dispatches into the principal bytes through
another sealed `document()` reference.

## Implemented slice

Compilation retains a private typed plan containing the literal document
reference, predicate-free location path, expanded attribute name, and normalized
variable name. Execution converts the comparison variable while the principal
source still owns its node identities, prepares the stylesheet document only
through the sealed invocation snapshot, evaluates the path in that document,
and compares candidate attributes there. Raw `NodeId` values never cross
documents.

The local corpus adapter admits the already reviewed `select68.xml` principal
bytes under the additional stylesheet-relative logical identity required by
`document('select68.xml')`. This is an exact case overlay: no directory scan,
ambient filesystem authority, or engine-side alias is introduced.

Focused tests cover both an atomic local variable and a principal-source node
variable, including nested sealed document dispatch.

## Corpus result

Both unchanged cases initialize, execute, and compare exactly.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,457 | 2,459 | +2 |
| Executed successfully | 2,407 | 2,409 | +2 |
| Initialization failures | 713 | 711 | -2 |
| Execution failures | 50 | 50 | 0 |
| Exact expected-result matches | 2,276 | 2,278 | +2 |
| Visible mismatches | 0 | 0 | 0 |
| Comparator gaps | 0 | 0 | 0 |

The conservative all-catalog exact-match ratio is now
`2,278 / 3,173 = 71.79%`. Expected-error credit remains 423 / 431.

## Boundaries

- The admitted predicate is one trailing attribute-equals-variable filter on a
  typed literal-document path.
- Variable conversion occurs against the principal invocation context before
  candidate comparison in the stylesheet document.
- Resource authority remains sealed, bounded, and host-supplied.
- General cross-document expressions, arbitrary predicates, live acquisition,
  and cross-invocation caches remain unselected.
- This is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_apply_templates_filters_stylesheet_document_nodes_with_a_local_string_variable
cargo test -p fastxslt --all-features xslt10_stylesheet_document_filter_scalarizes_a_principal_source_variable
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'select67'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'select68'
./scripts/verify.ps1
```
