# OASIS XSLT 1.0 Global Source-Copy Temporary Tree

Date: 2026-09-20

## Question

Can an XSLT 1.0 global variable construct a temporary tree with
`xsl:copy-of select="/"` without placing source-derived state in the reusable
compiled stylesheet?

## Finding

Yes, for the bounded document-root selection. Compilation retains only the
typed location path. Each transformation evaluates that path against its own
principal source and materializes a new invocation-owned temporary tree.

The safe recursive copy preserves expanded names, in-scope namespace slices,
attributes, child order, text, comments, and processing instructions. A
selected document node contributes its children as temporary-tree roots rather
than becoming a nested document node. Every retained node is charged to the
XDM-node budget before retention, and the shared path evaluator retains its
existing work and cancellation checks.

One focused regression reuses one compiled stylesheet concurrently against two
different source documents. Each result contains only its invocation's source
copy, proving that the compiled generation remains stylesheet-derived.

## Corpus movement

The unchanged Lotus `copy59` and `copy60` cases move from the generic global
constructor frontier to exact XML-semantic comparison.

| Counter | Before | After | Change |
| --- | ---: | ---: | ---: |
| Initialized | 2,090 | 2,092 | +2 |
| Initialization failures | 1,045 | 1,043 | -2 |
| Executed successfully | 2,004 | 2,006 | +2 |
| Execution failures | 86 | 86 | 0 |
| Exact XML-semantic matches | 1,873 | 1,875 | +2 |
| XML comparison mismatches | 55 | 55 | 0 |

The measured exact compatibility lower bound is now
`1,875 / 3,173 = 59.09%`. The generic `FXST1015` frontier falls from 21 to 19
cases. This is local compatibility evidence against the hash-verified,
non-redistributed OASIS CD04 archive; it is not a broad conformance claim.

## Boundaries

- The admitted global constructor has one `xsl:copy-of`, no declared type, and
  the exact typed document-root selection `/`.
- Other source-copy selections and mixed global sequence constructors remain
  explicit frontiers.
- Temporary node identity is invocation-local and newly allocated; source node
  identity is not retained by copy semantics.
- No global cache, cross-invocation tree sharing, public temporary-tree type,
  or source-dependent compiled state is introduced.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_global_copy_of_document_materializes_an_invocation_temporary_tree
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier FXST1015
./scripts/verify.ps1
```
