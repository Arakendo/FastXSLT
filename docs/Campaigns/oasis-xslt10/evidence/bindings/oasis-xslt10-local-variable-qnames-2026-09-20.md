# OASIS XSLT 1.0 Local Variable QNames

Date: 2026-09-20  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a local XSLT 1.0 variable use a QName name and be referenced through a
prefix alias without changing runtime frame ownership?

## Implemented slice

Local `xsl:variable` declarations now normalize valid QName names through the
same static expanded-name owner used by globals, parameters, and direct value
references. The canonical key enters the existing invocation-local variable
frame; copy-on-write isolation, lexical scope, shadowing, and value-kind storage
are unchanged. No namespace resolution was added to runtime lookup.

This does not yet admit QName local parameters or qualified names in every
specialized expression production.

## Corpus result

The unchanged `Microsoft/Variables__78162#1` case becomes an exact XML-semantic
match.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,072 | 2,073 | +1 |
| Initialization failures | 1,063 | 1,062 | -1 |
| Executed successfully | 1,970 | 1,971 | +1 |
| Exact XML-semantic matches | 1,841 | 1,842 | +1 |
| XML comparison mismatches | 54 | 54 | 0 |
| Execution failures | 102 | 102 | 0 |

The exact compatibility lower bound becomes `1,842 / 3,173 = 58.05%` of the
complete catalog. This is compatibility evidence, not an XSLT 1.0 conformance
claim.

## Verification

- The focused QName-variable regression declares a local content variable and
  reads it through a different prefix bound to the same namespace.
- The complete conserved sweep moved exactly one case from initialization
  failure to exact output without changing mismatch or execution-failure
  totals.
- No upstream corpus byte was edited.
