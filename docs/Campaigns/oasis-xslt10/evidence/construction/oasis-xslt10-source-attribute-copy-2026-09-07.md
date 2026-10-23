# OASIS XSLT 1.0 Source-Attribute `xsl:copy`

Date: 2026-09-07  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can `xsl:copy` over a source attribute reuse the existing pending-attribute
owner, including its duplicate-name and late-attribute checks, rather than
remaining an unsupported source-node kind?

## Change

The source-node copy operation now turns an attribute into the same private
`PendingAttribute` representation used by computed attributes and source
`xsl:copy-of`. The operation retains the source expanded name and string value,
charges the source-node visit, and leaves attachment to the enclosing result
element owner.

No attribute is appended directly to an already constructed result tree. The
existing result-element merge therefore continues to enforce duplicate-name
and child-before-attribute ordering as `XTDE0410`.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,376 | 1,376 | 0 |
| Executed successfully | 1,207 | 1,208 | +1 |
| Expected-result XML matches | 1,093 | 1,094 | +1 |
| XML comparison mismatches | 79 | 79 | 0 |
| Execution failures | 169 | 168 | -1 |
| `FXRT1007` observations | 13 | 5 | -8 |
| `XTDE0410` observations | 3 | 10 | +7 |

`Lotus/copy_copy25#1` is the newly exact unchanged case. Seven other cases now
reach their real result-attribute placement/ordering errors instead of stopping
at the former unsupported-node-kind boundary. They remain visible and
uncredited.

## Verification

- A focused runtime test copies two selected source attributes into one literal
  result element and checks their names and values.
- Existing pending-attribute tests continue to cover duplicate and late
  attributes.
- The complete local OASIS measurement completed with the counters above and no
  upstream corpus byte was edited.

