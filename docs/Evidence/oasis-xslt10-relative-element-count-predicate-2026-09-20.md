# OASIS XSLT 1.0 Relative Element-Count Predicate

Date: 2026-09-20  
Status: Local compatibility evidence

## Question

Can a location-path predicate compare the number of elements selected by a
bounded relative child path without introducing a general XPath expression
evaluator?

## Method

- Compile `count(relative-element-path) comparison nonnegative-integer` to a
  private typed predicate with at most eight child-element steps.
- Admit only unqualified NCName and wildcard element tests, with an optional
  leading `./`, and the six ordinary numeric comparison operators.
- Traverse from each predicate candidate through the shared immutable XDM,
  charging every inspected child and the final integer comparison while
  retaining cancellation observation.
- Add a focused nested-child regression and run unchanged Lotus `axes_axes85`
  from the hash-verified local OASIS XSLT 1.0 CD04 archive.

## Result

The focused expression `*[count(./*/*) > 0]` selects only the element having a
grandchild. The unchanged corpus case initializes, executes, and exactly
matches its expected XML result.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,120 | 2,121 | +1 |
| Initialization failures | 1,015 | 1,014 | -1 |
| Executed successfully | 2,031 | 2,032 | +1 |
| Execution failures | 89 | 89 | 0 |
| Exact XML-semantic matches | 1,900 | 1,901 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The strict compatibility lower bound is now `1,901 / 3,173 = 59.91%`.
The generic `FXXP1001` initialization frontier falls from 26 to 25 cases.
This is local compatibility evidence against a non-redistributed archival
suite, not a broad conformance claim.

## Boundaries

- The retained path is child-element-only, has at most eight steps, and admits
  only wildcard or unqualified NCName tests.
- Predicates, axes, attributes, qualified names, dynamic comparison operands,
  arithmetic, and general aggregate expressions remain unsupported here.
- The plan is immutable stylesheet-derived state; traversal and work
  accounting remain invocation-owned.

## Reproduction

```powershell
cargo test -p fastxslt --all-features path_boolean_predicates_compare_relative_element_path_counts
./scripts/measure-oasis-xslt10.ps1 -TraceCase axes_axes85
./scripts/verify.ps1
```
