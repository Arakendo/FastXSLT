# OASIS XSLT 1.0 Qualified Variable-Rooted Paths

Date: 2026-09-20  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can a QName-named source-node variable act as the root of a relative XSLT 1.0
selection path while retaining expanded-name identity and ordinary charged path
semantics?

## Implemented slice

The existing `ApplySelection::SourceVariablePath` plan now accepts ordinary
XSLT 1.0 child paths rooted at a variable, including:

```xpath
$use:nodes/book
```

The variable QName is resolved to the same canonical expanded-name key used by
its declaration. Runtime obtains the invocation-owned source-node sequence and
evaluates the already typed relative location path from each node, then restores
document order and removes duplicate identities. Namespace resolution remains
a compile-time operation.

This does not make arbitrary XPath expressions variable-rooted. Modern-mode
admission is unchanged, and unsupported relative syntax still fails through the
ordinary typed path compiler.

## Corpus result

Two unchanged cases become exact XML-semantic matches:

- `Microsoft/Variables_xslt_variable_qname_test1#1`
- `Microsoft/Variables_xslt_variable_qname_test2#1`

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,065 | 2,067 | +2 |
| Initialization failures | 1,070 | 1,068 | -2 |
| Executed successfully | 1,963 | 1,965 | +2 |
| Exact XML-semantic matches | 1,834 | 1,836 | +2 |
| XML comparison mismatches | 54 | 54 | 0 |
| Execution failures | 102 | 102 | 0 |

The exact compatibility lower bound becomes `1,836 / 3,173 = 57.86%` of the
complete catalog. This is compatibility evidence, not an XSLT 1.0 conformance
claim.

## Verification

- The focused QName-variable regression now also selects children through a
  differently prefixed alias of the global source-node variable.
- The complete conserved sweep moved exactly two cases from initialization
  failure to exact output without changing mismatch or execution-failure
  totals.
- No upstream corpus byte was edited.
