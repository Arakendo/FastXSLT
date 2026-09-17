# OASIS XSLT 1.0 Literal-Result Prefix Exclusion -- 2026-09-16

Date: 2026-09-16  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an `xsl:exclude-result-prefixes` control attribute on a literal result
element use the same compiled namespace-selection machinery as exclusions on
stylesheet instructions?

## Implemented slice

Yes. The literal-result validator now admits the namespaced control attribute,
while ordinary result-attribute compilation continues to omit every attribute
in the XSLT namespace. Namespace collection reads the namespaced spelling on a
literal result element as well as the unnamespaced spelling used by XSLT
instructions, and applies its prefix list over the element's inherited
namespace scope.

The existing rules remain intact: the namespace required by the result
element's own expanded name cannot be excluded, the XML and XSLT namespaces do
not become ordinary result bindings, and the compiled result owns only its
selected immutable namespace bindings. This adds no runtime namespace-policy
branch.

## Corpus result

Fourteen cases leave the blanket `FXST1007` control-attribute frontier. Ten
initialize and execute: eight produce exact XML-semantic results, while Lotus
`lre_lre15` and `lre_lre18` expose later discretionary indentation mismatches
after producing the required namespace nodes. Four
namespace-alias cases reach their already explicit `xsl:namespace-alias`
declaration boundary. The mismatches are newly visible dispositions, not
regressions from former passes.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,850 | 1,860 | +10 |
| Initialization failures | 1,285 | 1,275 | -10 |
| Executed successfully | 1,651 | 1,661 | +10 |
| Execution failures | 199 | 199 | 0 |
| Expected-result XML matches | 1,504 | 1,512 | +8 |
| XML comparison mismatches | 116 | 118 | +2 |
| Expected errors unexpectedly succeeding | 6 | 6 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound rises to
`1,512 / 2,742 = 55.14%` for standard-operation cases and
`1,512 / 3,173 = 47.65%` for the complete catalog.

## Verification

A focused compiler test proves that the excluded binding is absent, an
unexcluded sibling binding remains, and the XSLT control attribute does not
become a result attribute. The complete sweep conserves all 3,173 identities
and adds no unexpected success or panic.
