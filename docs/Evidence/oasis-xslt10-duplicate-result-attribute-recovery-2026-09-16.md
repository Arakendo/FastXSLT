# OASIS XSLT 1.0 Duplicate Result-Attribute Recovery -- 2026-09-16

Date: 2026-09-16  
Status: Verified standards and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can FastXSLT preserve XSLT 1.0's same-expanded-name attribute replacement
behavior without weakening the later-edition duplicate-attribute error?

## Implemented slice

Yes. In XSLT 1.0 static context, repeated leading `xsl:attribute`
instructions with the same expanded name are normalized during compilation so
the last value wins. A computed attribute similarly replaces a same-named
literal result attribute. Every source instruction is still excluded from the
ordinary result-child body, including an earlier constructor removed by this
normalization.

The rule is selected at compilation. Later stylesheet versions continue to
report `XTDE0410`; runtime does not branch on stylesheet version, and dynamic
late-attribute errors remain unchanged.

## Corpus result

Three unchanged standard-operation cases become exact XML-semantic
expected-result passes: Lotus `attribset12`, Lotus `attribset18`, and Lotus
`namespace03`.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,818 | 1,821 | +3 |
| Initialization failures | 1,317 | 1,314 | -3 |
| Executed successfully | 1,628 | 1,631 | +3 |
| Execution failures | 190 | 190 | 0 |
| Expected-result XML matches | 1,483 | 1,486 | +3 |
| XML comparison mismatches | 115 | 115 | 0 |
| Execution panics | 0 | 0 | 0 |

The exact compatibility lower bound rises to
`1,486 / 2,742 = 54.19%` for standard-operation cases and
`1,486 / 3,173 = 46.83%` for the complete catalog.

## Verification

A focused production-lifecycle test covers both computed-over-literal and
computed-over-computed replacement. The existing XSLT 3.0 compiler test still
requires a structured duplicate-result-attribute failure. The unchanged
complete sweep conserves all 3,173 identities and adds no XML mismatch,
execution failure, or panic.
