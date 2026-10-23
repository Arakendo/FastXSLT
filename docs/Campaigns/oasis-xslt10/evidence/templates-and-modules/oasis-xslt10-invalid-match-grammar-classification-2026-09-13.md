# OASIS XSLT 1.0 invalid match grammar classification -- 2026-09-13

Date: 2026-09-13  
Status: Verified local compatibility-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can FastXSLT distinguish invalid XSLT 1.0 match-pattern grammar from valid
`id()` and `key()` patterns whose semantics are not implemented yet?

## Implemented slice

The private match-pattern compiler now classifies a whole-pattern variable
reference and `key()` arguments that are not string literals as invalid
`FXST1005` input. Literal `id('value')` and `key('name', 'value')` patterns
remain valid but unsupported capability. This prevents a generic unsupported
diagnostic from concealing malformed stylesheet input without falsely claiming
DTD-derived ID typing or key-index support.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,671 | 1,671 | 0 |
| Executed successfully | 1,500 | 1,500 | 0 |
| Expected-result XML matches | 1,369 | 1,369 | 0 |
| Initialization failures | 1,464 | 1,464 | 0 |
| Generic unsupported match-pattern frontier | 17 | 15 | -2 |

The two affected OASIS cases already expected initialization errors, so their
observed-error disposition and every compatibility denominator remain
unchanged. The improvement is diagnostic: their failures now identify invalid
grammar instead of unimplemented valid semantics. The strict
standard-operation lower bound remains `1,369 / 2,742 = 49.93%`; the
conservative all-catalog ratio remains `1,369 / 3,173 = 43.15%`.

## Boundaries

This tranche does not implement DTD ID typing, `id()` selection, `xsl:key`,
`key()` indexes, variable-valued match predicates, or general match-pattern
XPath evaluation. In particular, the valid literal `id()` and `key()` cases
remain visibly unsupported.

## Verification

- Focused compiler coverage distinguishes invalid grammar from valid but
  unsupported `id()`/`key()` capability.
- The complete 3,173-case local measurement produced the counters above.
