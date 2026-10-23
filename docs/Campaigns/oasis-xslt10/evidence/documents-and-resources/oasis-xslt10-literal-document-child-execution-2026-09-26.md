# OASIS XSLT 1.0 Literal `document()` Child Execution

- Date: 2026-09-26
- Status: Verified corpus and focused-regression evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related decisions and reviews: ADR-0002, AR-0014, and AR-0019

## Pressure

The direct literal `document()` root slice could execute an external document
root, but it could not represent the common XPath 1.0 selections
`document('resource')/*` or `document('resource')/name`. Three unchanged OASIS
cases stopped at that compiler boundary, including one normal template-dispatch
case and one expected missing-resource case.

## Implemented slice

`xsl:apply-templates` and `xsl:for-each` now retain a private typed selection for
the element children of a literal, non-empty sealed-snapshot `document()`
reference. The tail may be `*` or one unprefixed NCName. Execution:

- resolves and prepares the supplemental document through the existing
  invocation-owned, sealed-resource path;
- charges every inspected document child as XPath work;
- selects only element children, preserving document order and XPath 1.0's
  no-namespace interpretation of an unprefixed name test;
- establishes the selected external node as source focus before ordinary
  template dispatch or sequence-constructor execution; and
- derives the existing ADR-0012 invocation-owned whitespace visibility view
  over the immutable supplemental document whenever the stylesheet requires
  stripping; and
- retains the existing guard against unqualified cross-document source-node
  values.

Dynamic references, a second `document()` base argument, prefixed names, deeper
paths, predicates, unions, and arbitrary mixed-document node sequences remain
outside this slice. URI spelling grants no filesystem or network authority.

Focused regressions cover wildcard-child template dispatch and named-child
`for-each`, both proving that relative XPath evaluation reads the external
source rather than the principal input.

## Measurement result

Three unchanged cases leave the initialization frontier:

- Microsoft `Template_MatchFirstElementBelowDocRootWithExpression#1` compares
  exactly;
- Microsoft `XSLTFunctions_DocumentFunctionWithNonExistingFilename#1` reaches
  the expected structured missing-resource execution outcome; and
- Lotus `whitespace_whitespace35#1` compares exactly through the shared
  invocation-owned whitespace visibility view.

The complete conserved sweep moves from 2,339 to **2,342 initialized**, from
2,288 to **2,290 successfully executed**, and from 2,123 to **2,125 / 3,173
exact matches (66.97%)**. Initialization failures fall from 831 to 828;
execution failures rise from 51 to 52 because the expected missing-resource
case now reaches its honest runtime disposition. Visible mismatches remain 69
and execution panics remain zero.

This is compatibility evidence from a locally acquired archival corpus, not an
XSLT 1.0 conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features literal_document_
./scripts/measure-oasis-xslt10.ps1
```
