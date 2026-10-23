# OASIS XSLT 1.0 Source-Copy Attribute Sets -- 2026-09-16

Date: 2026-09-16  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can `xsl:copy use-attribute-sets` reuse the compile-time static attribute-set
graph without adding runtime lookup state or changing non-element copy identity?

## Implemented slice

Yes. The source-copy compiler converts the already-validated static set values
into its ordinary immutable attribute plan. Explicit child `xsl:attribute`
instructions replace same-named set attributes before execution. Element copies
retain their source name and namespaces; text, comment, processing-instruction,
and attribute copies retain their existing behavior.

For a document-node context, XSLT 1.0 `xsl:copy` remains a no-op wrapper, but
its attribute constructors are now emitted as pending result attributes before
the body. This lets a containing result element receive them and repairs the
Microsoft comment-in-attribute-set case exposed by the first sweep.

## Corpus result

Six unchanged cases become exact XML-semantic expected-result passes: Lotus
`attribset04`, `attribset09`, `attribset21`, `attribset26`, and `attribset28`,
plus Microsoft `Comment_Comment_Test3`.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,803 | 1,809 | +6 |
| Initialization failures | 1,332 | 1,326 | -6 |
| Executed successfully | 1,613 | 1,619 | +6 |
| Execution failures | 190 | 190 | 0 |
| Expected-result XML matches | 1,468 | 1,474 | +6 |
| XML comparison mismatches | 115 | 115 | 0 |
| Execution panics | 0 | 0 | 0 |

Cases requiring same-name attribute-set declaration merging remain explicitly
unsupported; this slice does not infer include/import precedence semantics.

The exact compatibility lower bound rises to
`1,474 / 2,742 = 53.76%` for standard-operation cases and
`1,474 / 3,173 = 46.45%` for the complete catalog.

## Verification

Focused production-lifecycle tests cover element-copy set application, explicit
attribute override, and root-copy pending-attribute behavior. The unchanged
complete sweep conserves all 3,173 identities and adds no XML mismatch,
execution failure, or panic.
