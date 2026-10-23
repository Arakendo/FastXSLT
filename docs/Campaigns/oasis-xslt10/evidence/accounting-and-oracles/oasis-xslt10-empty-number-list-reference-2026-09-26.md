# OASIS XSLT 1.0 Empty Number-List Reference

Date: 2026-09-26  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Should `Microsoft/Number__84683` cause FastXSLT to emit the punctuation from
`format="A."` when the current node produces an empty number list?

## Finding

The stylesheet applies one template to both `chapter` and `section`, but its
`xsl:number` counts only `section`. For a `chapter` context there is no matching
ancestor-or-self node, so the number list is empty and the numbering instruction
contributes an empty string. FastXSLT does that.

The archival reference instead prefixes each chapter label with a lone period.
The suite-owned `doubts.xml` entry is unusually direct:

> Spurious . and new-line before each occurrence of "chapter"

Emitting a formatting-token suffix for an empty number list would reproduce the
acknowledged reference defect and would regress the shared numbering semantics.
The case therefore receives an exact, bounded
`unusable-reference-result-excluded` disposition and no pass credit.

## Measurement

| Measurement | Before | After |
| --- | ---: | ---: |
| Catalog cases | 3,173 | 3,173 |
| Exact expected-result matches | 2,125 | 2,125 |
| Visible XML mismatches | 49 | 48 |
| Unusable-reference-result exclusions | 22 | 23 |

No engine or corpus byte changed. This is a denominator-preserving reference
classification, not a new conformance pass.
