# OASIS XSLT 1.0 Global Variable QNames

Date: 2026-09-20  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can global XSLT 1.0 variables and parameters use QName names and be referenced
through a different prefix bound to the same namespace?

## Implemented slice

Global variable and parameter declarations now normalize valid QName names to
the compiler's existing expanded-name spelling. Direct variable value
expressions perform the same static namespace resolution. Prefix spelling is
therefore not runtime identity: `decl:value` and `use:value` address the same
binding when both prefixes resolve to the same namespace URI.

The compiled program retains one canonical string key and the runtime variable
maps remain unchanged. Invalid lexical QNames and unbound prefixes still fail
during compilation; no namespace lookup or prefix comparison was added to the
execution hot path. More complex expressions containing qualified variable
references remain outside this slice until their typed compilers adopt the
same normalization deliberately.

## Corpus result

Four unchanged cases become exact XML-semantic matches:

- `Lotus/variable_variable25#1`
- `Lotus/variable_variable55#1`
- `Microsoft/Variables__78097#1`
- `Microsoft/Variables__78316#1`

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,061 | 2,065 | +4 |
| Initialization failures | 1,074 | 1,070 | -4 |
| Executed successfully | 1,959 | 1,963 | +4 |
| Exact XML-semantic matches | 1,830 | 1,834 | +4 |
| XML comparison mismatches | 54 | 54 | 0 |
| Execution failures | 102 | 102 | 0 |

The exact compatibility lower bound becomes `1,834 / 3,173 = 57.80%` of the
complete catalog. This is compatibility evidence, not an XSLT 1.0 conformance
claim.

## Verification

- A focused runtime regression declares and references one global through two
  different prefixes bound to the same namespace.
- The complete conserved sweep moved exactly four cases from initialization
  failure to exact output without changing mismatch or execution-failure
  totals.
- Two adjacent cases with a qualified variable embedded in a broader path
  remain visibly rejected; this slice does not overstate general QName-variable
  expression support.
- No upstream corpus byte was edited.
