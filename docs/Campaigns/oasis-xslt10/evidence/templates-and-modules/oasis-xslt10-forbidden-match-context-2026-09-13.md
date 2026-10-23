# OASIS XSLT 1.0 forbidden match context -- 2026-09-13

Date: 2026-09-13  
Status: Verified standards/corpus diagnostic evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Should FastXSLT execute the archival Xalan `match14` expectation, which uses a
global variable in a template match pattern, or preserve the normative XSLT
1.0 restriction?

## Standards authority

The [XSLT 1.0 Recommendation, section 5.3](https://www.w3.org/TR/1999/REC-xslt-19991116.html#defining-template-rules)
states that a `match` attribute containing a variable reference is an error.
Section 12.4 separately states that using `current()` in a pattern is an error.
The archival `match14` description and expected successful output therefore
conflict with the selected normative standard.

## Implemented slice

For stylesheet elements processed with XSLT 1.0 compatibility, the private
match compiler now rejects variable references and `current()` calls found
outside string literals as invalid `FXST1005` input. The check is explicitly
version-sensitive: the existing later-edition variable-enabled pattern slice
remains admitted for a version 3.0 stylesheet.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,678 | 1,678 | 0 |
| Executed successfully | 1,507 | 1,507 | 0 |
| Expected-result XML matches | 1,376 | 1,376 | 0 |
| Initialization failures | 1,457 | 1,457 | 0 |
| Generic unsupported match-pattern frontier | 8 | 6 | -2 |

Both cases were already initialization failures under the narrower engine, so
the aggregate counters do not change. Their classification is now
standards-correct rather than generic unsupported capability. `match14` is not
credited as a pass merely because its archival expected result asks the engine
to recover contrary to the normative error rule.

The strict standard-operation lower bound remains
`1,376 / 2,742 = 50.18%`; the conservative all-catalog ratio remains
`1,376 / 3,173 = 43.37%`.

## Boundaries

This does not remove variable-enabled patterns from later standards editions,
select expressions, or other locations where variables are legal. It does not
select a general recovery policy for erroneous XSLT 1.0 stylesheets.

## Verification

- Focused compiler coverage proves XSLT 1.0 rejection for nested variable and
  `current()` use.
- The same coverage proves a version 3.0 variable-enabled pattern remains
  admitted.
- The complete 3,173-case local measurement produced the counters above.
