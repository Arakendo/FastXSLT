# OASIS XSLT 1.0 Nested Template-Argument Constructor

Date: 2026-09-20  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an XSLT 1.0 `xsl:with-param` sequence constructor call another named
template and pass the resulting nodes as one invocation-owned temporary value?

## Implemented slice

After the existing literal, static constructed-content, local-binding/value,
and bounded `for-each` argument plans have been considered, an XSLT 1.0
content argument may retain a bounded compiled instruction sequence. The
sequence is limited to 64 meaningful stylesheet children and contains only
instructions already admitted by the ordinary compiler.

At invocation time the sequence executes through the normal instruction engine
with the caller's complete dynamic context, including source or temporary
focus, position, size, current mode, current matched-template identity, and
named-template call depth. The semantic result nodes are then charged and
materialized as one invocation-owned temporary value. Cancellation, work
budgets, diagnostics, recursion limits, namespace construction, and variable
frame isolation therefore remain those of the reference execution path.

The compiled instruction sequence participates in known-capacity accounting
and decimal-format binding traversal. No alternate evaluator, runtime parser,
new resource authority, or cross-invocation retention was introduced.

## Corpus result

The unchanged `Lotus/variable_variable47#1` case now executes an imported
library template inside a parameter constructor and becomes an exact
XML-semantic match.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,077 | 2,078 | +1 |
| Initialization failures | 1,058 | 1,057 | -1 |
| Executed successfully | 1,975 | 1,976 | +1 |
| Exact XML-semantic matches | 1,846 | 1,847 | +1 |
| XML comparison mismatches | 54 | 54 | 0 |
| Execution failures | 102 | 102 | 0 |

The exact compatibility lower bound becomes `1,847 / 3,173 = 58.21%` of the
complete catalog. This is compatibility evidence, not an XSLT 1.0 conformance
claim.

## Verification

- A focused lifecycle regression builds an argument by calling a nested named
  template, passes another parameter to that call, and copies the materialized
  element through the receiving template.
- The complete conserved sweep moved exactly one case from initialization
  failure to exact output without changing mismatch or execution-failure
  totals.
- No upstream corpus byte was edited.
