# OASIS XSLT 1.0 Stylesheet `xml:space` Admission -- 2026-09-17

Date: 2026-09-17  
Status: Verified bounded semantic and corpus evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Finding

The sequence compiler already computed inherited `xml:space` state and used it
to decide whether whitespace-only stylesheet text nodes become result text.
Most XSLT instruction attribute validators nevertheless rejected `xml:space`
before that semantic path could run. Only specialized `xsl:text` and
`xsl:output` validation admitted the attribute.

## Repair

The common XSLT instruction-attribute validator now admits `xml:space` with the
standard `default` and `preserve` values and rejects any other lexical with
static `XTSE0020`. The existing sequence compiler remains the owner of inherited
stylesheet-text preservation; `xml:space="default"` overrides an inherited
`preserve` value.

Focused tests prove preservation around an ordinary result element, inherited
state reset, and invalid-value rejection. This slice does not claim that every
structural XSLT content model with preserved whitespace is complete.

## Corpus result

Two unchanged Microsoft text cases leave attribute-level `FXST1009`, initialize,
and execute. Both expose visible output mismatches, so the exact-result lower
bound remains 1,517. This is recorded as frontier movement rather than a
conformance pass.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,863 | 1,865 | +2 |
| Initialization failures | 1,272 | 1,270 | -2 |
| Executed successfully | 1,664 | 1,666 | +2 |
| Execution failures | 199 | 199 | 0 |
| Expected-result XML matches | 1,517 | 1,517 | 0 |
| XML comparison mismatches | 118 | 120 | +2 |
| XML comparator unsupported | 23 | 23 | 0 |
| Execution panics | 0 | 0 | 0 |

## Verification

The complete sweep conserves all 3,173 identities. It reports 1,865 initialized
cases, 1,666 successful executions, 1,517 exact XML comparisons, 120 visible
XML mismatches, 23 comparator-unsupported outcomes, 403 expected errors during
initialization, 21 expected errors during execution, and four doubt-annotated
expected-error cases that execute successfully.
