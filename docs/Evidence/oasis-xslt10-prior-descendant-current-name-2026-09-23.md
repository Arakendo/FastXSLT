# OASIS XSLT 1.0 Prior-Descendant Current Name

Date: 2026-09-23  
Status: Verified semantic and compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an XSLT 1.0 predicate determine whether an earlier document descendant
has the same lexical QName as the outer current node without adding a general
dynamic predicate evaluator?

## Implemented slice

Yes. XSLT 1.0 compilation recognizes the bounded predicate
`/descendant::*[position() < $variable and name() = name(current())]` and its
relative spelling. It lowers the expression to a typed boolean plan retaining
only the position-variable identity.

Execution resolves that ordinary invocation variable through the existing
XSLT 1.0 numeric conversion, traverses document descendants in document order,
charges every node visit, and compares retained lexical QNames against the
outer current node. The plan is selected only during XSLT 1.0 compilation;
modern XPath retains its existing unsupported boundary.

## Corpus result

Unchanged `Microsoft/Miscellaneous__84423#1` becomes an exact
expected-result XML match.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,198 | 2,199 | +1 |
| Executed successfully | 2,145 | 2,146 | +1 |
| Expected-result XML matches | 1,998 | 1,999 | +1 |
| XML comparison mismatches | 63 | 63 | 0 |
| Execution failures | 53 | 53 | 0 |
| Comparator unsupported | 75 | 75 | 0 |

The strict complete-catalog lower bound is now
`1,999 / 3,173 = 63.00%`. Generic `FXXP1001` initialization failures fall from
16 to 15. The 3,173-case denominator and every other disposition are
conserved.

## Boundaries

- This is not a general descendant-predicate implementation.
- It does not admit arbitrary position comparisons, name operands, boolean
  compositions, or axes.
- The outer `current()` focus is retained explicitly by the typed plan rather
  than inferred from the predicate candidate focus.
- No namespace-node identity or new resource authority is introduced.
- The archive remains locally acquired and is not redistributed.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_prior_descendant_name_test_uses_the_outer_focus_and_position_variable
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Microsoft/Miscellaneous__84423#1'
./scripts/verify.ps1
```
