# OASIS XSLT 1.0 Chained Relational Predicates

Date: 2026-09-20  
Status: Local compatibility evidence

## Question

Can FastXSLT preserve XPath 1.0 left-associative chained relational semantics
for source-free literal predicates without admitting that legacy grammar into
the modern XPath path?

## Method

- Add a compile-time XSLT 1.0 compatibility fold for exactly two chained
  ordered comparisons over literal numeric, string, or boolean operands.
- Evaluate the first comparison, convert its boolean result to XPath 1.0 number
  `1` or `0`, and apply the second comparison. More than two comparisons and
  non-literal operands remain unsupported.
- Select the compatibility fold only through `parse_xslt10_location_path`; the
  ordinary modern path parser remains unchanged.
- Add focused left-association, false-result, mixed-operator, overlong-chain,
  and dynamic-operand tests, then run unchanged Lotus `predicate10`, Lotus
  `predicate36`, and Microsoft `Miscellaneous_Bug74174` from the hash-verified
  local OASIS XSLT 1.0 CD04 archive.

## Result

All three unchanged corpus cases initialize, execute, and exactly match their
expected XML results.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,115 | 2,118 | +3 |
| Initialization failures | 1,020 | 1,017 | -3 |
| Executed successfully | 2,026 | 2,029 | +3 |
| Execution failures | 89 | 89 | 0 |
| Exact XML-semantic matches | 1,895 | 1,898 | +3 |
| XML comparison mismatches | 55 | 55 | 0 |

The strict compatibility lower bound is now `1,898 / 3,173 = 59.82%`.
The generic `FXXP1001` initialization frontier falls from 31 to 28 cases.
This is local compatibility evidence against a non-redistributed archival
suite, not a broad conformance claim.

## Boundaries

- Exactly two ordered comparison operators and three source-free literal
  operands are admitted.
- Folding occurs only under the XSLT 1.0 static compatibility path.
- Dynamic operands, node-set comparisons, longer chains, and a general legacy
  expression grammar remain unsupported.
- The compiled result is an ordinary selected path or empty sequence; no
  runtime compatibility branch is added to node traversal.

## Reproduction

```powershell
cargo test -p fastxslt --all-features folds_xpath10_chained_ordered_literals_left_associatively
./scripts/measure-oasis-xslt10.ps1 -TraceCase predicate_predicate10
./scripts/measure-oasis-xslt10.ps1 -TraceCase predicate_predicate36
./scripts/measure-oasis-xslt10.ps1 -TraceCase Miscellaneous_Bug74174
./scripts/verify.ps1
```
