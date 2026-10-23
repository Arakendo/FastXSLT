# OASIS XSLT 1.0 Source-Copy Path Attribute

Date: 2026-09-20

## Question

Can an `xsl:attribute` inside source `xsl:copy` obtain its value from an
ordinary source-node path without adding a source-copy-specific expression
evaluator?

## Finding

Yes. Lotus `copy40` copies one source attribute and then constructs another
whose `xsl:value-of select="text()"` uses the first selected text node's string
value. The preceding attribute-only `xsl:copy-of` does not begin child content.

## Implementation

Under compile-selected XSLT 1.0 compatibility, the bounded source-copy
attribute compiler may lower an admitted location path into the existing typed
path/string attribute-value plan. Runtime uses the shared charged path
evaluator and first selected node's string value. Existing specializations for
`.` and unqualified source attributes retain their prior plans, and modern
static contexts are not widened by this compatibility slice.

A focused regression uses two child text nodes separated by an element and
proves that `xsl:value-of select="text()"` takes only the first selected node's
string value. It also composes the value with an attribute copied by the
preceding `xsl:copy-of`.

## Corpus movement

The unchanged Lotus `copy40` case moves from initialization failure to exact
XML-semantic comparison.

| Counter | Before | After | Change |
| --- | ---: | ---: | ---: |
| Initialized | 2,089 | 2,090 | +1 |
| Initialization failures | 1,046 | 1,045 | -1 |
| Executed successfully | 2,003 | 2,004 | +1 |
| Execution failures | 86 | 86 | 0 |
| Exact XML-semantic matches | 1,872 | 1,873 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The measured exact compatibility lower bound is now
`1,873 / 3,173 = 59.03%`. This is local compatibility evidence against the
hash-verified, non-redistributed OASIS CD04 archive; it is not a broad
conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_source_copy_attribute_uses_first_selected_node_string_value
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier FXXP1012
./scripts/verify.ps1
```
