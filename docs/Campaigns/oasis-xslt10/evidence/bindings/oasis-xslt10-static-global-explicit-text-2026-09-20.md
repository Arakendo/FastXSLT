# OASIS XSLT 1.0 Static Global Explicit Text

Date: 2026-09-20  
Status: Focused semantic evidence

## Question

Can a static global temporary tree preserve explicit `xsl:text` in document
order without admitting a general global instruction executor or weakening the
existing escaping boundary?

## Method

- Route an `xsl:text` child of the existing static constructed-node path through
  the ordinary text-instruction validator.
- Lower validated content to the existing immutable `ConstructedNode::Text`
  representation.
- Add a focused runtime regression with explicit text before and after a
  literal child element.
- Re-run all 3,173 cases in the hash-verified local OASIS XSLT 1.0 CD04 archive.

## Result

The focused regression preserves the exact order `text / element / text` and
materializes the tree per invocation. The OASIS aggregate remains unchanged at
2,102 initialized, 2,015 successful executions, 87 execution failures, 1,884
exact XML-semantic matches, and 55 mismatches. No previously blocked unchanged
case depends solely on this shape.

The absence of a corpus delta is useful negative evidence: this small semantic
gap is now covered without claiming broader compatibility progress.

## Boundaries

- Only static character content is admitted.
- Invalid child content retains the ordinary structured compile failure.
- `disable-output-escaping="yes"` remains unsupported and is not folded into
  ordinary text.
- Dynamic global instructions and source-dependent construction remain outside
  this path.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_static_global_tree_preserves_explicit_text_in_document_order
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
