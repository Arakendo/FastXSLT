# OASIS XSLT 1.0 Literal `document()` Copy

- Date: 2026-09-26
- Status: Verified corpus and focused-regression evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related decisions and reviews: ADR-0002, AR-0014, and AR-0019

## Pressure

The local OASIS runner admitted catalog-declared supplemental documents into
each case's bounded sealed snapshot, but an unchanged `xsl:copy-of` using a
literal `document()` reference stopped at the generic location-path compiler
frontier. The runtime already owned invocation-local preparation for literal
sealed-snapshot references used by document-identity expressions.

## Implemented slice

The compiler now retains a literal, non-empty `document('reference')` or
`document("reference")` used directly by `xsl:copy-of`, optionally followed by
one unprefixed `//name` descendant selection, as a typed document reference
with the stylesheet instruction's base identity. During invocation, the
existing snapshot resolver:

- resolves only against immutable bytes explicitly admitted by the host;
- applies denial before membership disclosure;
- rejects fragments in this initial slice;
- parses and constructs the supplemental XDM under existing work budgets;
- retains it only in the invocation-local dynamic-document map; and
- copies either the document node's children or selected no-namespace
  descendants in document order through the ordinary charged source-copy
  implementation.

The prepared workbench state now retains its sealed snapshot so runtime
references use the same admitted generation as compilation and source
preparation. It does not reopen files, access the network, introduce a live
resolver, or establish a cross-invocation document cache. The workbench's
prepared-engine retention estimate now charges the snapshot map, logical
identity capacities, and admitted byte lengths explicitly; retaining runtime
resource authority therefore does not create unaccounted native admission
pressure.

Focused regressions prove exact copied output and prove that a sealed resource
which the host marks denied remains denied during execution with structured
`FXRS0003` classification.

## Measurement result

Unchanged Microsoft case `XSLTFunctions_Document#1` now initializes, executes,
and compares exactly. The complete conserved sweep moves from 2,333 to
**2,334 initialized**, from 2,286 to **2,287 successfully executed**, and from
2,121 to **2,122 / 3,173 exact matches (66.88%)**.

The generic unsupported `xsl:copy-of` frontier falls from 12 to 11. Dynamic
arguments, a second `document()` base argument, `document('')`, and path tails
remain visible and are not inferred from this literal whole-document slice.

A second bounded tranche admits the unprefixed `//name` tail. Unchanged Lotus
`mdocs_mdocs10` leaves the compiler frontier, raising initialization to
**2,335** and lowering initialization failures to **835**. Its archive catalog
does not declare the referenced sibling document, so execution reports that
identity as missing from the sealed snapshot. Successful execution remains
2,287 and exact matches remain **2,122 / 3,173 (66.88%)**; the execution-failure
count rises to 48 and the generic unsupported `xsl:copy-of` frontier falls from
11 to 10. The harness does not infer acquisition authority from a stylesheet
URI literal merely to turn the case green.

This is compatibility evidence from a locally acquired archival corpus, not an
XSLT 1.0 conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_copy_of_literal_document_uses_only_the_sealed_snapshot
cargo test -p fastxslt --all-features xslt10_copy_of_literal_document_descendants_preserves_order_and_no_namespace_test
cargo test -p fastxslt --all-features denied_sealed_document_resource_remains_denied_during_execution
./scripts/measure-oasis-xslt10.ps1 -TraceCase XSLTFunctions_Document#1
./scripts/measure-oasis-xslt10.ps1 -TraceCase Lotus/mdocs_mdocs10#1
```
