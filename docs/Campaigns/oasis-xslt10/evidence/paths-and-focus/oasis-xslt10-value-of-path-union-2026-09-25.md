# OASIS XSLT 1.0 `value-of` Path Union

- Date: 2026-09-25
- Status: Verified compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

FastXSLT already owned charged location-path evaluation, namespace-resolved
qualified child paths, union document-order normalization and deduplication,
and XSLT 1.0 first-node string conversion. The `xsl:value-of` compiler did not
compose those capabilities when its selection was a path union. Unchanged
Microsoft `BVTs_bvt001#1` therefore rejected
`title/text() | my:title/text()` as malformed path syntax.

## Implemented slice

Under XSLT 1.0 static context, a top-level union whose alternatives are all
admitted location paths now compiles to a typed first-node path-union value
plan. Qualified alternatives resolve prefixes from the instruction's static
namespace context. The qualified-child path parser also recognizes ordinary
child node-kind tests after a qualified step.

Runtime evaluates every alternative through the shared controlled path
evaluator, restores document order, removes duplicate node identities, and
converts only the first selected node to its string value. This preserves the
XSLT 1.0 `xsl:value-of` rule while reusing the same union normalization as other
typed consumers.

The slice does not admit function or variable union alternatives, general
sequence expressions, namespace nodes, a public node-set type, or modern
first-item coercion. Compiled-retention accounting includes every retained
path.

A focused runtime test places the namespaced alternative first in document
order but second in the lexical union and verifies that document order, not
union spelling, selects the emitted value.

## Corpus result

The unchanged Microsoft BVT initializes, executes, and exactly matches its
reference output. The complete sweep advances from **2,332 to 2,333
initialized**, from **2,285 to 2,286 successfully executed**, and from **2,120
to 2,121 / 3,173 exact matches (66.85%)**. Initialization failures fall from
838 to 837; execution failures, mismatches, exclusions, and comparator gaps are
unchanged.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_value_of_path_union_uses_first_node_in_document_order
./scripts/measure-oasis-xslt10.ps1 -TraceCase BVTs_bvt001
./scripts/verify.ps1
```
