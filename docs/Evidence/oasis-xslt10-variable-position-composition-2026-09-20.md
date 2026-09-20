# OASIS XSLT 1.0 Variable-Position Composition

Date: 2026-09-20  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can the existing invocation-owned source-node sequence and XSLT 1.0 numeric
variable semantics compose inside value and conditional expressions without a
second sequence model or an untyped general-expression fallback?

## Implemented slice

The typed XSLT 1.0 value-expression plan now admits a variable-position source
selection inside `concat()` in either exact form:

```xpath
$series[$pos]
$series[number($pos)]
```

Both names must be unqualified variable names. Runtime requires `$series` to
own source nodes, converts `$pos` through the existing XSLT 1.0 variable
number rules, charges selection work, and obtains the selected node's string
value through controlled source traversal. The shared positional selector is
also used by the apply/for-each compatibility path; neither consumer mutates or
clones the bound sequence.

The typed conditional plan additionally admits the exact recursion guard:

```xpath
$pos < count($series)
```

The left variable uses the existing XSLT 1.0 numeric conversion. The right
variable must own source nodes; incompatible kinds report the existing typed
runtime failure. Evaluation charges XPath work and does not generalize either
production into an unrestricted legacy expression evaluator.

## Corpus result

The unchanged `Lotus/variable_variable53#1` case becomes an exact XML-semantic
match. It recursively emits every item from a source-node variable using
`$series[number($pos)]`, `$pos < count($series)`, and the previously admitted
numeric increment expression.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,060 | 2,061 | +1 |
| Initialization failures | 1,075 | 1,074 | -1 |
| Executed successfully | 1,958 | 1,959 | +1 |
| Exact XML-semantic matches | 1,829 | 1,830 | +1 |
| XML comparison mismatches | 54 | 54 | 0 |
| Execution failures | 102 | 102 | 0 |

The exact compatibility lower bound becomes `1,830 / 3,173 = 57.67%` of the
complete catalog. This is compatibility evidence, not an XSLT 1.0 conformance
claim.

## Verification

- A focused runtime regression composes the shared positional selector in both
  apply/for-each and `concat()` contexts and evaluates the variable-versus-count
  condition for true and false results.
- The complete conserved sweep moved exactly one case from initialization
  failure to exact output without changing mismatch or execution-failure
  totals.
- No upstream corpus byte was edited.
