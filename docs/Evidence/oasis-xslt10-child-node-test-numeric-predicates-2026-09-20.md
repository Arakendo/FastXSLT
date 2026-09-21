# OASIS XSLT 1.0 Child Node-Test Numeric Predicates

Date: 2026-09-20  
Status: Local compatibility evidence

## Question

Can FastXSLT distinguish operator-shaped tokens from child node tests inside
the bounded predicates `child[div=integer]` and `child[*=integer]` without
admitting general predicate arithmetic?

## Method

- Parse an unqualified NCName or `*` to the left of `=` as a child element
  node test when the right operand is one signed integer literal.
- Retain the node test and integer in a typed location-path predicate rather
  than reinterpreting `div` or `*` as arithmetic operators.
- At runtime, scan direct element children, convert each controlled string
  value to an XPath number, and apply existential node-set equality.
- Charge every child visit and numeric conversion through the existing
  invocation control.
- Add a focused named/wildcard regression and run unchanged Lotus `select31`
  and `select32` from the hash-verified local OASIS XSLT 1.0 CD04 archive.

## Result

Both unchanged cases exactly match their expected XML results.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,105 | 2,107 | +2 |
| Initialization failures | 1,030 | 1,028 | -2 |
| Executed successfully | 2,018 | 2,020 | +2 |
| Execution failures | 87 | 87 | 0 |
| Exact XML-semantic matches | 1,887 | 1,889 | +2 |
| XML comparison mismatches | 55 | 55 | 0 |

The strict compatibility lower bound is now `1,889 / 3,173 = 59.53%`.
The generic `FXXP1001` initialization frontier falls from 41 to 39 cases.
This is compatibility evidence against a hash-verified, non-redistributed
archive; it is not a broad conformance claim.

## Boundaries

- The child node test is one unqualified NCName or `*`.
- The comparison operator is exactly `=` and the right operand is one signed
  integer literal.
- The plan does not admit arithmetic `div`/`*`, namespace-qualified tests,
  deeper relative paths, alternate comparisons, or general predicate
  composition.
- Compiled state retains only the node test and integer; source nodes and
  computed values remain invocation-owned.

## Reproduction

```powershell
cargo test -p fastxslt --all-features path_boolean_predicate_compares_named_and_wildcard_children_with_integer
./scripts/measure-oasis-xslt10.ps1 -TraceCase Lotus/select_select31
./scripts/measure-oasis-xslt10.ps1 -TraceCase Lotus/select_select32
./scripts/verify.ps1
```
