# OASIS XSLT 1.0 Local Static Attribute Sets -- 2026-09-15

Date: 2026-09-15  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the compiler admit a first XSLT 1.0 attribute-set slice without adding a
runtime registry, weakening attribute override rules, or claiming import/include
composition that has not been implemented?

## Implemented slice

Yes. One standard stylesheet document may declare uniquely named
`xsl:attribute-set` elements containing only statically named `xsl:attribute`
children with static text values. Literal result elements may reference one or
more such local sets through `xsl:use-attribute-sets`.

Resolution happens during stylesheet compilation. The resulting attributes are
stored in the ordinary compiled element plan and use the existing result-node,
budget, and cancellation controls at execution. There is no runtime lookup,
cache, resource acquisition, or cross-generation state.

Override order is preserved for the admitted slice: later referenced sets
replace earlier set attributes, literal result attributes replace set
attributes, and explicit child `xsl:attribute` instructions replace set
attributes. Dynamic attribute-set values are rejected because their global-only
variable visibility has not yet been represented. Multiple declarations of one
set and same-name set composition across include/import boundaries remain
explicitly unsupported.

## Corpus result

Seven unchanged Lotus cases become exact XML-semantic expected-result passes:
`attribset_attribset01`, `02`, `11`, `39`, `47`, `48`, and `49`.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,779 | 1,789 | +10 |
| Initialization failures | 1,356 | 1,346 | -10 |
| Executed successfully | 1,595 | 1,603 | +8 |
| Execution failures | 184 | 186 | +2 |
| Expected-result XML matches | 1,451 | 1,458 | +7 |
| XML comparison mismatches | 115 | 115 | 0 |
| Execution panics | 0 | 0 | 0 |

The two additional execution failures expose the already-known unsupported
UTF-16 serialization boundary after otherwise valid static attribute-set
compilation; they do not represent attribute-set semantic failures. Three
previously observed partial-result mismatches were eliminated before admission
by rejecting dynamic values and same-name cross-module composition.

The exact compatibility lower bound rises to
`1,458 / 2,742 = 53.17%` for standard-operation cases and
`1,458 / 3,173 = 45.95%` for the complete catalog.

## Verification

A focused production-lifecycle test verifies set, literal, and explicit
computed-attribute override order. A compiler test verifies that same-name
attribute sets across an include boundary remain unsupported rather than being
partially merged. The unchanged full sweep conserves all 3,173 identities and
adds no XML mismatch or panic.
