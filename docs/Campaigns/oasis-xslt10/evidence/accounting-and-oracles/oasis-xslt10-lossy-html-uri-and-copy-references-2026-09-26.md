# OASIS XSLT 1.0 Lossy HTML URI and Copy References

Date: 2026-09-26  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the remaining Microsoft HTML URI-attribute references distinguish correct
UTF-8 serialization, and does the duplicated namespace/copy case expose a new
engine behavior?

## Findings

### `Output__84455` through `Output__84462`

Seven executing cases exercise HTML output around `classid`, `codebase`,
`data`, `cite`, `action`, `for`, and `datasrc`. Their principal sources contain
Japanese, Cyrillic, Latin Extended-A, and Tamil characters, and the catalog
describes them as UTF-8 cases.

FastXSLT preserves those text characters. Every affected archival reference
instead stores literal ASCII question marks, and even changes `ĀāĂă` to
`AaAa`. The reference bytes therefore cannot distinguish correct Unicode
serialization from lossy legacy conversion. They also discard source
whitespace independently of the URI-attribute behavior under test.

The standard XSLT 1.0 HTML URI-attribute set covers the standard attributes in
this family; Microsoft `datasrc` and the treatment of `script/@for` do not
create a new standard-profile obligation. FastXSLT does not add a proprietary
`datasrc` rule from this evidence.

All seven identities receive exact, bounded
`unusable-reference-result-excluded` dispositions and no pass credit.

### `Output__84480`

The catalog describes this as a copy of `Output__78183`, and it carries the
same material comparison difference: copied source whitespace is omitted by
the archival result despite there being no stylesheet stripping authority.
Namespace-prefix spelling is not the issue because comparison is by expanded
name. The case receives the same bounded `host-parser-policy-excluded`
disposition and no pass credit.

## Measurement

| Measurement | Before | After |
| --- | ---: | ---: |
| Catalog cases | 3,173 | 3,173 |
| Exact expected-result matches | 2,125 | 2,125 |
| Visible XML mismatches | 39 | 31 |
| Host-parser-policy exclusions | 16 | 17 |
| Unusable-reference-result exclusions | 25 | 32 |

No engine or corpus byte changed. The cases remain conserved under explicit
non-pass dispositions.
