# OASIS XSLT 1.0 `copy-of` Attribute Content

Date: 2026-09-19  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the remaining single-`xsl:copy-of` computed-attribute cases reuse the
existing typed location-path evaluator without admitting a general attribute
sequence constructor or a second execution path?

## Implemented slice

Yes. Under XSLT 1.0 static context only, the computed-attribute compiler accepts
content containing exactly one empty `xsl:copy-of` whose `select` is already an
admitted location path. The private value plan retains that typed path and its
owned-capacity accounting.

Execution evaluates the path with the ordinary charged evaluator and preserves
selection order. Selected text and attribute nodes contribute their string
values to the attribute value. Selected document, element, comment, and
processing-instruction nodes contribute no characters under this bounded XSLT
1.0 recovery behavior. Result-node construction and the copy instruction are
charged before retention. The modern static context continues to reject this
constructor shape as `FXST1033`.

This does not admit arbitrary computed-attribute sequence constructors, copy
node structure into an attribute, alter modern XSLT semantics, or introduce a
new path evaluator.

## Corpus result

The unchanged Lotus `copy56`, `copy57`, and `copy58` cases move from
`FXST1033` initialization failures to exact XML-semantic expected-result
matches. Against the conserved 3,173-case catalog:

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,042 | 2,045 | +3 |
| Initialization failures | 1,093 | 1,090 | -3 |
| Executed successfully | 1,940 | 1,943 | +3 |
| Execution failures | 102 | 102 | 0 |
| Exact XML-semantic matches | 1,811 | 1,814 | +3 |
| XML comparison mismatches | 54 | 54 | 0 |
| `FXST1033` frontier | 16 | 13 | -3 |

No expected result, corpus input, or upstream submodule was changed.

## Verification

A focused runtime test selects mixed text, element, and source-attribute nodes
and verifies the exact lexical attribute values. A cross-version regression
proves that the same constructor remains unsupported under XSLT 3.0 static
context. The complete corpus measurement conserves every catalog identity and
adds no mismatch, execution failure, or panic.
