# OASIS XSLT 1.0 Path Concat Tranche

Date: 2026-09-06  
Status: Verified local compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can XPath 1.0 `concat()` combine namespace-aware location paths with bounded
static string, integer, and variable operands while preserving first-node
node-set string conversion and the shared result-construction controls?

## Change

The XSLT 1.0 compiler now retains a bounded typed `concat()` plan with 2 through
64 operands. Each operand is either a static string, an integer lexical value,
a variable reference, or a namespace-aware location path. Runtime evaluation
converts variables through the existing XSLT 1.0 atomic/node-set/temporary-tree
string owner, converts the first node selected by each path in document order
to its controlled string value (or the empty string for an empty node set), and
appends operands through the existing charged result-text owner.

The plan is selected only in XSLT 1.0 compatibility mode. Nested function
calls, booleans, general numerics, and modern sequence conversion remain
outside this bounded slice. The implementation streams operands into the
result rather than allocating one additional concatenated intermediate string.

## Corpus result

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 1,326 | 1,330 | +4 |
| Executed successfully | 1,153 | 1,156 | +3 |
| Expected-result XML matches | 1,041 | 1,044 | +3 |
| XML comparison mismatches | 77 | 77 | 0 |
| Execution failures | 173 | 174 | +1 |

The three newly executed string cases agree with their unchanged archival
expected results:

- `Lotus/string_string98#1`
- `Lotus/string_string99#1`
- `Lotus/string_string103#1`

`Lotus/copy_copy38#1` now compiles through its path-based `concat()` use and
reaches the existing bounded HTML-serialization frontier (`FXSR1001`). It
remains visibly uncredited; this is a later exposed boundary rather than a new
semantic mismatch. No upstream corpus byte was edited.

Variable operands are covered by the focused engine oracle but add no further
corpus credit in this tranche: current variable-`concat()` candidates encounter
earlier import, AVT, nested-function, or other unsupported boundaries. The
complete measurement remains at the counters above.

The strict standard-operation lower bound is now
`1,044 / 2,742 = 38.07%`; the deliberately conservative all-catalog ratio is
`1,044 / 3,173 = 32.90%`. These remain compatibility measurements, not an
XSLT 1.0 conformance claim.

## Verification

- A focused runtime sentinel combines two paths, a string literal, an integer,
  an empty node set, and a global string variable.
- The complete local OASIS measurement completed with the counters above.
- Full workspace verification is recorded after this tranche is closed.
