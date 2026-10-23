# OASIS XSLT 1.0 Qualified Variable Sequence Selection

Date: 2026-09-20  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a QName-named global source-node variable or parameter be selected directly
by `xsl:for-each` through the same invocation-owned sequence semantics as an
unqualified binding?

## Implemented slice

XSLT 1.0 apply/for-each selection now normalizes a direct qualified variable
reference such as `$use:nodes` to the compiler's canonical expanded-name key.
The existing `ApplySelection::VariableSequence` plan and runtime value-kind
dispatch are unchanged. Prefix resolution occurs once during compilation; the
hot path neither resolves namespaces nor compares lexical prefixes.

This slice does not admit qualified local declarations or arbitrary expressions
containing qualified variables. Modern-mode variable admission remains
unchanged.

## Corpus result

Five unchanged cases become exact XML-semantic matches:

- `Microsoft/Variables__78112#1`
- `Microsoft/Variables__78115#1`
- `Microsoft/Variables__78353#1`
- `Microsoft/Variables__78355#1`
- `Microsoft/Variables__78400#1`

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,067 | 2,072 | +5 |
| Initialization failures | 1,068 | 1,063 | -5 |
| Executed successfully | 1,965 | 1,970 | +5 |
| Exact XML-semantic matches | 1,836 | 1,841 | +5 |
| XML comparison mismatches | 54 | 54 | 0 |
| Execution failures | 102 | 102 | 0 |

The exact compatibility lower bound becomes `1,841 / 3,173 = 58.02%` of the
complete catalog. This is compatibility evidence, not an XSLT 1.0 conformance
claim.

## Verification

- The focused QName-variable regression selects the same source-node binding
  both directly and through a relative child path using a prefix alias.
- The complete conserved sweep moved exactly five cases from initialization
  failure to exact output without changing mismatch or execution-failure
  totals.
- No upstream corpus byte was edited.
