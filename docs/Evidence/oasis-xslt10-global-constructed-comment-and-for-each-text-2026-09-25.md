# OASIS XSLT 1.0 Global Constructed Comment and `for-each` Text

- Date: 2026-09-25
- Status: Verified compatibility evidence
- Corpus: OASIS XSLT/XPath Conformance Test Suite, Committee Draft 04
- Archive SHA-256: `66750E63994C3A07252D8A6555E82E5CE5127D1942005BCC8314C7D736BD7BD5`
- Related review: AR-0019

## Pressure

Unchanged Microsoft case `BVTs_bvt029` constructs a global XSLT 1.0 result
tree fragment containing a literal element, attributes, a static
`xsl:comment`, character content, and a nested `xsl:for-each` that repeats
static text once for every selected source node. FastXSLT's existing global
temporary-tree slice already preserved literal elements and source-derived
`xsl:value-of`, but rejected the two remaining constructor forms at
`FXST1015`.

## Implemented slice

The private immutable constructed-node representation now admits:

- a statically compiled comment node using the existing XSLT 1.0 comment
  validation and recovery rules; and
- an XSLT 1.0 `xsl:for-each` with a charged location-path selection and an
  entirely static text or `xsl:text` body.

The compiled program retains only the source-independent path and static
content. The principal source remains invocation-owned. Runtime temporary-tree
materialization evaluates the path through the ordinary controlled evaluator,
constructs the comment/text nodes under the existing XDM budget, and preserves
the same immutable global-tree lifecycle as the earlier source-valued form.
No general sequence-constructor executor, invocation state in compiled data,
or public representation was introduced.

## Verification and honest disposition

A focused two-layer regression proves that a source-dependent global tree
preserves a comment and repeats `Chunk` once for each of three selected source
elements before `xsl:copy-of` copies the temporary tree.

The unchanged OASIS case now initializes and executes, moving one case out of
the generic `FXST1015` frontier. It remains an uncredited XML mismatch. The
archived stylesheet's CDATA-adjacent indentation is one parsed non-whitespace
text node in FastXSLT, so the actual result preserves its line breaks and
spaces; the Microsoft expected file contains only two spaces on each side of
`CDATA Text`. Neither the engine nor the XML comparator discards that visible
text difference.

The full sweep therefore moves from **2,318 to 2,319 initialized** and from
**2,270 to 2,271 successfully executed**. Initialization failures fall from
817 to 816, visible XML mismatches rise from 80 to 81, and exact matches remain
**2,105 / 3,173 (66.34%)**. Comparator-unsupported cases remain 77. The
later-frontier movement is recorded, but is not counted as a compatibility
pass.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_global_constructed_tree_preserves_comments_and_repeats_static_for_each_text
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/BVTs_bvt029#1'
```
