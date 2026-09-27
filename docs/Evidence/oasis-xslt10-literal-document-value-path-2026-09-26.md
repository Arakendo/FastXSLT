# OASIS XSLT 1.0 Literal `document()` Value Path

- Date: 2026-09-26
- Status: Verified shared prerequisite; no corpus credit claimed
- Related review: AR-0019

## Implemented slice

An XSLT 1.0 `xsl:value-of` expression shaped as a literal
`document('relative.xml')` reference followed by a typed location path now
compiles to immutable stylesheet-derived state. At execution it:

1. resolves only through the invocation's sealed resource snapshot;
2. prepares the external XML in invocation-owned state;
3. applies the compiled stylesheet whitespace policy through the same private
   visibility-view machinery used for other source documents;
4. evaluates the precompiled path under normal work control; and
5. emits the XSLT 1.0 first selected node's string-value.

The focused regression proves the supplemental document wins over a conflicting
principal-source value and that whitespace stripping applies without mutating
the prepared document. Compiled-state retained-capacity accounting includes the
resource reference and typed path.

## Corpus result and boundary

The unchanged OASIS sweep gains no pass from this prerequisite. The remaining
literal `document()` identities bind external node-sets in global variables,
compose them into path unions, or use them as execution focus. Those forms need
an explicit cross-document node ownership representation; they must not be
faked by placing external `NodeId` values in the principal document's runtime
frame.

No live resolver, filesystem access, network access, cross-invocation cache, or
public resource contract was introduced.

## Verification

```powershell
cargo test -p fastxslt --all-features xslt10_value_of_literal_document_path_uses_the_sealed_snapshot
./scripts/measure-oasis-xslt10.ps1
./scripts/verify.ps1
```
