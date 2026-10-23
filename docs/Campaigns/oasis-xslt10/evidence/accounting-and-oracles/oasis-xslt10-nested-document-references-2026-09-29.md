# OASIS XSLT 1.0 Nested `document()` References

Date: 2026-09-29  
Status: Verified implementation and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed  
Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`

## Pressure

Two unchanged cases require the result of one `document()` call to supply URI
reference nodes to another:

```xpath
document(document(places))
document(document('level1/level2/xreluri11b.xml')/*/filename)/*/body
```

The inner reference can therefore come from the principal source or be a
literal reference with an admitted location-path tail. The outer relative URI
must use the logical base identity of the actual inner result node.

## Implemented slice

The private XSLT 1.0 `xsl:copy-of` plan now retains one bounded nested-reference
shape. Its inner `document()` argument is either a literal string or one typed
source location path; its optional inner and outer tails are typed location
paths. Execution prepares each inner document through the existing sealed
snapshot, evaluates its selected reference nodes, and resolves every outer URI
relative to the node that supplied its string value.

Both inner and outer prepared documents remain invocation-owned, budgeted, and
deduplicated by opaque document identity. The local harness adds only
`mdocs02a.xml` and `mdocs02b.xml` under source-relative logical identities for
the archival case whose catalog identities do not represent that base. The
`reluri11` resources are already catalog-declared exactly.

## Corpus result

Unchanged `Lotus/mdocs_mdocs02#1` and `Lotus/reluri_reluri11#1` initialize,
execute, and compare exactly.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,451 | 2,453 | +2 |
| Executed successfully | 2,401 | 2,403 | +2 |
| Initialization failures | 719 | 717 | -2 |
| Execution failures | 50 | 50 | 0 |
| Exact expected-result matches | 2,270 | 2,272 | +2 |
| Visible mismatches | 0 | 0 | 0 |
| Comparator gaps | 0 | 0 | 0 |

The conservative all-catalog exact-match ratio is now
`2,272 / 3,173 = 71.60%`. Expected-error credit remains 423 / 431. No case
remains in the former `unsupported/FXXP1003/unsupported xsl:copy-of selection`
cluster for these complex `document()` expressions.

## Boundaries

- Exactly two `document()` levels are admitted by this private instruction.
- The inner first argument is one literal string or typed source location path;
  general computed expressions and deeper nesting remain unsupported.
- Optional inner and outer path tails use the existing typed location-path
  evaluator.
- Resource acquisition remains sealed-snapshot-only and invocation-owned.
- This is compatibility evidence from a locally acquired archive, not a
  conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt xslt10_nested_
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/mdocs_mdocs02#1'
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/reluri_reluri11#1'
./scripts/verify.ps1
```
