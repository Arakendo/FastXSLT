# OASIS XSLT 1.0 Variable Child-Name Sort

Date: 2026-09-23  
Status: Verified implementation and compatibility evidence  
Corpus: OASIS XSLT/XPath Conformance Test Suite CD04, locally acquired and not redistributed

## Question

Can an XSLT 1.0 stylesheet select a sort key dynamically with the established
`*[name(.) = $variable]` technique without adding a general dynamic XPath
evaluator or changing modern XPath behavior?

## Implemented slice

Yes. XSLT 1.0 compilation recognizes only the bounded child-element forms
`./*[name(.) = $variable]`, `*[name(.) = $variable]`, and their symmetric
comparison. It lowers them to a typed sort-key plan containing the variable
identity. At execution, the plan resolves the ordinary invocation variable,
visits charged child nodes in document order, compares each element's lexical
QName, and uses the first matching child's string value.

The plan is selected during XSLT 1.0 compilation. XSLT 3.0 retains the existing
unsupported boundary for this still-unimplemented general XPath predicate, and
there is no stylesheet-version branch in the runtime loop.

## Corpus result

Unchanged `Lotus/sort_sort35#1` becomes an exact expected-result pass.

| Measurement | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,197 | 2,198 | +1 |
| Executed successfully | 2,144 | 2,145 | +1 |
| Expected-result XML matches | 1,997 | 1,998 | +1 |
| XML comparison mismatches | 63 | 63 | 0 |
| Execution failures | 53 | 53 | 0 |
| Comparator unsupported | 75 | 75 | 0 |

The strict complete-catalog lower bound is now
`1,998 / 3,173 = 62.97%`. Generic `FXXP1001` initialization failures fall from
17 to 16. The 3,173-case denominator and every other disposition are
conserved.

## Boundaries

- This is not a general `name()` predicate implementation.
- Namespace-node sorting, computed function calls, arbitrary boolean
  predicates, and non-child axes remain outside this slice.
- Variable values and child traversal remain invocation-owned and use existing
  work-budget and cancellation charge points.
- The archive remains locally acquired and is not redistributed.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_sort_selects_a_child_key_by_variable_lexical_name
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/sort_sort35#1'
./scripts/verify.ps1
```
