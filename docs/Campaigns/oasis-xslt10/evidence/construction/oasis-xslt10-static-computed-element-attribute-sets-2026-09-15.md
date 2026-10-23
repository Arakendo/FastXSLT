# OASIS XSLT 1.0 Static Computed-Element Attribute Sets -- 2026-09-15

Date: 2026-09-15  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the compile-time local static attribute-set mechanism also serve statically
named `xsl:element` without creating a second resolution path or weakening
explicit child-attribute precedence?

## Implemented slice

Yes. A statically named `xsl:element` may now use the same uniquely declared,
single-document, static-text attribute sets admitted for literal result
elements. The shared compile-time resolver expands each referenced set into the
ordinary compiled element plan. Explicit child `xsl:attribute` instructions
replace same-named set attributes before execution.

The existing restrictions remain: no dynamic set values, duplicate local set
declarations, same-name include/import composition, dynamic element names, or
runtime attribute-set lookup. Namespace bindings required by admitted set
attributes continue through the existing computed-element namespace fixup.

## Corpus result

Three unchanged cases become exact XML-semantic expected-result passes:
Lotus `attribset_attribset03`, Lotus `attribset_attribset30`, and Lotus
`impincl_impincl09`.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,789 | 1,794 | +5 |
| Initialization failures | 1,346 | 1,341 | -5 |
| Executed successfully | 1,603 | 1,606 | +3 |
| Execution failures | 186 | 188 | +2 |
| Expected-result XML matches | 1,458 | 1,461 | +3 |
| XML comparison mismatches | 115 | 115 | 0 |
| Execution panics | 0 | 0 | 0 |

The two additional execution failures reach the already-known unsupported
UTF-16 serialization boundary after valid static computed-element construction.
No partial XML result was admitted.

The exact compatibility lower bound rises to
`1,461 / 2,742 = 53.28%` for standard-operation cases and
`1,461 / 3,173 = 46.04%` for the complete catalog.

## Verification

A focused production-lifecycle test verifies a static computed element receiving
set attributes while an explicit child attribute overrides one set value. The
unchanged full sweep conserves all 3,173 identities and adds no XML mismatch or
panic.
