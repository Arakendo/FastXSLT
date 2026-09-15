# OASIS XSLT 1.0 Computed-Attribute `for-each` -- 2026-09-15

Date: 2026-09-15  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a computed attribute execute the common XSLT 1.0 constructor shape
`xsl:for-each` plus `xsl:value-of select="."` without admitting a second,
general sequence-constructor evaluator?

## Implemented slice

Yes. The compiler now retains one private typed plan containing the existing
controlled location path when all of these conditions hold:

- the stylesheet uses XSLT 1.0 compatibility;
- the computed attribute contains exactly one `xsl:for-each`;
- the loop has a required path-valued `select` and no other attributes; and
- its body contains exactly one empty `xsl:value-of select="."`.

Execution evaluates the retained path from the current source-node focus and
concatenates each selected node's controlled string value in selection order.
The path traversal, string-value traversal, loop instruction, and each
value-of instruction retain invocation-local work accounting and cancellation.
Other nested computed-attribute constructors remain explicitly unsupported.
The representation contributes its path capacity to prepared-engine retention
accounting.

## Corpus result

Unchanged Lotus `attribset_attribset25` moves from `FXST1033` initialization
failure to an exact XML-semantic expected-result pass. It constructs
`test1="XYZ"` by iterating three selected `a` elements inside `xsl:attribute`.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,775 | 1,776 | +1 |
| Initialization failures | 1,360 | 1,359 | -1 |
| Executed successfully | 1,591 | 1,592 | +1 |
| Execution failures | 184 | 184 | 0 |
| Expected-result XML matches | 1,447 | 1,448 | +1 |
| XML comparison mismatches | 115 | 115 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound rises to
`1,448 / 2,742 = 52.81%` for standard-operation cases and
`1,448 / 3,173 = 45.64%` for the complete catalog.

## Verification

A focused production-lifecycle test constructs one attribute from two selected
source elements and verifies their concatenated string value. The unchanged
corpus sweep conserves all 3,173 identities, adds one exact pass, and does not
add an execution failure, mismatch, or panic.
