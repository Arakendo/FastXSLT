# OASIS XSLT 1.0 Static Comment and PI Text -- 2026-09-16

Date: 2026-09-16  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can comment and processing-instruction constructors reuse explicit static
`xsl:text` content without admitting a second dynamic sequence-constructor
engine or weakening the result-tree policy for disable-output-escaping?

## Implemented slice

Yes. The existing static comment and processing-instruction compilers now fold
an explicit `xsl:text` child into their owned string value. Whitespace-only
stylesheet text surrounding the explicit instruction remains excluded through
the ordinary meaningful-child rule.

Within comment and processing-instruction construction only,
`disable-output-escaping="yes"` is accepted and ignored because it has no effect
on those constructed node values. Invalid lexical values still report
`XTSE0020`, while an ordinary result `xsl:text` with `yes` remains outside the
semantic result-tree slice. Dynamic `xsl:value-of`, `xsl:for-each`, and other
constructor content remain explicit boundaries.

## Corpus result

Unchanged Microsoft
`Comment_DisableOutputEscaping_XslTextInXslComment` now produces its exact
XML-semantic expected result. Simplified Microsoft `BVTs_bvt098` advances from
initialization to the existing bounded HTML-serialization boundary and is not
counted as a pass.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,835 | 1,837 | +2 |
| Initialization failures | 1,300 | 1,298 | -2 |
| Executed successfully | 1,638 | 1,639 | +1 |
| Execution failures | 197 | 198 | +1 |
| Expected-result XML matches | 1,492 | 1,493 | +1 |
| XML comparison mismatches | 116 | 116 | 0 |
| Expected errors unexpectedly succeeding | 6 | 6 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound rises to
`1,493 / 2,742 = 54.45%` for standard-operation cases and
`1,493 / 3,173 = 47.05%` for the complete catalog.

## Verification

A focused compiler test requires both comment and processing-instruction nodes
to retain the decoded static `xsl:text` value while accepting the semantically
inert `disable-output-escaping="yes"`. Existing tests retain the ordinary text
constructor rejection. The complete sweep conserves all 3,173 identities and
adds no mismatch, unexpected success, or panic.
