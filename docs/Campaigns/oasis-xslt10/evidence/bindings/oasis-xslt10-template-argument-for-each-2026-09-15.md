# OASIS XSLT 1.0 Template-Argument `for-each` -- 2026-09-15

Date: 2026-09-15  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the bounded `xsl:for-each` plus `xsl:value-of select="."` constructor also
supply XSLT 1.0 `xsl:with-param` content while preserving result-tree-fragment
semantics?

## Implemented slice

Yes. Content-valued `xsl:with-param` on the private `xsl:call-template` path may
now retain the same compile-validated controlled location path used by the
computed-attribute slice. The admitted shape remains exactly one XSLT 1.0
`xsl:for-each` whose body is exactly one empty `xsl:value-of select="."`.

Runtime evaluation concatenates selected source-node string values under the
existing path, XDM traversal, instruction, budget, and cancellation controls,
then materializes one parentless temporary text node. The callee therefore
observes the XSLT 1.0 content-constructor value rather than an incorrectly
substituted atomic string. The path contributes to prepared-engine retention
accounting. No general nested argument constructor was admitted.

## Corpus result

Unchanged Lotus `namedtemplate_namedtemplate11` moves from `FXST1033`
initialization failure to an exact XML-semantic expected-result pass. Its named
template receives the temporary value `XYZ` produced by iterating three source
elements.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,776 | 1,777 | +1 |
| Initialization failures | 1,359 | 1,358 | -1 |
| Executed successfully | 1,592 | 1,593 | +1 |
| Execution failures | 184 | 184 | 0 |
| Expected-result XML matches | 1,448 | 1,449 | +1 |
| XML comparison mismatches | 115 | 115 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound rises to
`1,449 / 2,742 = 52.84%` for standard-operation cases and
`1,449 / 3,173 = 45.67%` for the complete catalog.

## Verification

A focused production-lifecycle test passes concatenated source values through a
named template and verifies the callee's serialized result. The unchanged full
sweep conserves all 3,173 identities and adds no failure, mismatch, or panic.
