# OASIS XSLT 1.0 Output Whitespace and Malformed UTF-16 References

Date: 2026-09-26  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Do four remaining Microsoft output comparisons expose serializer defects, or
do their references depend on undeclared host parsing and malformed archival
encoding?

## Findings

### `Output__77927` and `Output__77928`

Both stylesheets copy the principal `xslTutorial` element. One relies on the
default XML output method and the other declares it explicitly. Their source
contains whitespace-only text nodes between the root and its `HR` children,
plus a single space inside the first `HR`. Neither stylesheet declares
`xsl:strip-space` and the source has no DTD element-content declarations.

FastXSLT copies those source nodes. The shared archival result removes or
replaces them and chooses three different lexical empty-element forms. The
suite's doubt records already call out possible newline/normalization problems.
The semantic difference is historical host-parser whitespace policy, not an
XML-method distinction, so both cases receive exact
`host-parser-policy-excluded` dispositions and no pass credit.

### `Output__77939`

This stylesheet recursively applies templates and emits each element name.
The source whitespace-only text nodes are processed by the built-in text rule
because no stripping declaration exists. FastXSLT consequently emits those
characters around `HRHRHR`; the archival result contains only `HRHRHR`.
This is the same undeclared historical host-parser policy and receives the same
bounded non-pass disposition.

### `Output__77936`

The expected file begins with a UTF-16LE BOM and correctly encodes its XML
declaration. Starting at the first physical newline, however, it stores byte
pairs such as `0D 0A`, followed by isolated `00` bytes, instead of valid
UTF-16LE CR/LF code units (`0D 00 0A 00`). Decoding therefore produces unrelated
characters such as U+0A0D rather than XML line endings. The immutable bytes are
not a usable UTF-16 representation of the intended result, so the case receives
an exact `unusable-reference-result-excluded` disposition and no pass credit.

## Measurement

| Measurement | Before | After |
| --- | ---: | ---: |
| Catalog cases | 3,173 | 3,173 |
| Exact expected-result matches | 2,125 | 2,125 |
| Visible XML mismatches | 48 | 44 |
| Host-parser-policy exclusions | 9 | 12 |
| Unusable-reference-result exclusions | 23 | 24 |

No engine or corpus byte changed. The cases remain conserved in the complete
denominator with explicit non-pass dispositions.
