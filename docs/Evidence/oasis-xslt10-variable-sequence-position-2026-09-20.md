# OASIS XSLT 1.0 Variable-Sequence Position Filter

Date: 2026-09-20  
Status: Verified semantic and corpus-frontier evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an invocation-owned source-node sequence be filtered by a position stored
in another variable without cloning, mutating, or reinterpreting the sequence?

## Implemented slice

XSLT 1.0 apply/for-each selection now admits the exact form:

```xpath
$path[$pos]
```

Both names must be unqualified variable names. Runtime requires `$path` to own
a source-node sequence, converts `$pos` through the existing XSLT 1.0 variable
string/number rules, charges the predicate work, and selects one node only for
a finite positive integral position within the sequence. The original variable
binding remains immutable and invocation-local.

This does not admit predicates over temporary result trees, general variable
predicates, variable-rooted location paths, or modern sequence-filter syntax.
An incompatible variable kind reports the existing typed runtime failure rather
than producing a partial selection.

## Corpus result

The unchanged `Lotus/variable_variable52#1` case becomes an exact
XML-semantic match, including source-node parameters passed both by direct path
and by another variable.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,059 | 2,060 | +1 |
| Initialization failures | 1,076 | 1,075 | -1 |
| Executed successfully | 1,957 | 1,958 | +1 |
| Exact XML-semantic matches | 1,828 | 1,829 | +1 |
| XML comparison mismatches | 54 | 54 | 0 |
| Execution failures | 102 | 102 | 0 |

The exact compatibility lower bound becomes `1,829 / 3,173 = 57.64%` of the
complete catalog. This is compatibility evidence, not an XSLT 1.0 conformance
claim.

## Verification

- A focused runtime regression passes a source-node sequence through a named
  template parameter and filters it with a position captured from an outer
  `xsl:for-each` focus.
- The complete conserved sweep moved exactly one case from initialization
  failure to exact output without changing mismatch or execution-failure
  totals.
- No upstream corpus byte was edited.
