# OASIS XSLT 1.0 Output Source and Stylesheet Whitespace References

Date: 2026-09-26  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Do five additional Microsoft output mismatches expose output-property defects,
or do their references discard source or stylesheet characters independently
of the declared serialization method?

## Findings

### Historical source-parser whitespace

`Output__78175` and `Output__78177` recursively apply templates to the same
source used by the earlier output cases. `Output__78182` and `Output__78183`
copy that source subtree into constructed output. In every case the source has
whitespace-only text nodes, no DTD element-content declarations, and no
`xsl:strip-space` authority.

FastXSLT preserves the source characters through built-in text processing or
copying. The archival results omit them. Changing output declarations,
namespace prefixes, or the lexical form of empty elements cannot semantically
remove those source nodes. These four cases therefore receive exact, bounded
`host-parser-policy-excluded` dispositions and no pass credit.

### `Output__78180` stylesheet text

The included template contains text nodes such as the label ending in
`method=` followed by indentation, then `xsl:value-of`, then another text node
containing indentation and the next non-whitespace label. Because each such
text node contains non-whitespace characters, XSLT stylesheet stripping does
not discard its indentation characters.

FastXSLT retains the complete text-node values. The archival reference removes
selected whitespace from inside those non-whitespace text nodes. Reproducing
that reference would corrupt sequence-constructor character preservation, so
the case receives an exact `unusable-reference-result-excluded` disposition
and no pass credit.

## Measurement

| Measurement | Before | After |
| --- | ---: | ---: |
| Catalog cases | 3,173 | 3,173 |
| Exact expected-result matches | 2,125 | 2,125 |
| Visible XML mismatches | 44 | 39 |
| Host-parser-policy exclusions | 12 | 16 |
| Unusable-reference-result exclusions | 24 | 25 |

No engine or corpus byte changed. All five identities remain conserved under
named non-pass dispositions.
