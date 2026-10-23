# OASIS XSLT 1.0 Parent-Relative Document Admission

- Date: 2026-09-26
- Status: Verified corpus-adapter evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Implemented slice

The local OASIS acquisition adapter now discovers the narrow principal-
stylesheet form:

```xpath
document('../literal-relative-reference')
```

Discovery applies only to standard-operation cases and only to a single-
argument literal reference in the principal stylesheet. The physical path is
canonicalized and must remain beneath the owning suite-family directory. At
most 16 documents and 8 MiB of newly discovered bytes are admitted. Logical
identity is still resolved from the stylesheet base, and the bytes enter the
immutable resource snapshot before compilation or execution.

This is corpus acquisition policy, not engine authority. The engine continues
to resolve `document()` only through the sealed snapshot and never opens the
archive, filesystem, or network during compilation or transformation.

## Breadth control

An initial probe that admitted every literal local `document()` reference was
rejected: it changed eight unrelated initialization paths and reduced exact
credit. Restricting discovery to parent-relative references preserves every
prior disposition while addressing the archive's undeclared sibling-directory
dependencies. Same-directory literals, dynamic arguments, two-argument
`document()`, fragments, queries, absolute URIs, and paths outside the canonical
suite-family root remain unadmitted by this slice.

## Evidence

- Unchanged Lotus `mdocs_mdocs10` advances from missing-resource failure to an
  exact XML-semantic comparison.
- Unchanged Lotus `mdocs_mdocs11` advances from missing-resource failure to an
  exact comparison while exercising descendant dispatch, focus position/size,
  and attribute access over the supplemental document.
- Lotus `mdocs_mdocs05` retains its prior unsupported dynamic-expression
  frontier; acquisition does not disguise the remaining XPath boundary.

## Conserved result

The sweep moves from 2,298 to **2,300 successful executions** and from 2,141
to **2,143 exact matches (67.54%)**. Initialization remains 2,350, initialization
failures remain 820, and execution failures fall from 52 to 50. Expected-error
credit remains 423 / 431, comparator gaps remain 67, and visible XML mismatches
remain zero.

## Verification

```powershell
cargo test -p fastxslt --all-features discovers_only_single_argument_literal_document_references
./scripts/measure-oasis-xslt10.ps1 -TraceCase mdocs_mdocs10
./scripts/measure-oasis-xslt10.ps1 -TraceCase mdocs_mdocs11
./scripts/verify.ps1
```
