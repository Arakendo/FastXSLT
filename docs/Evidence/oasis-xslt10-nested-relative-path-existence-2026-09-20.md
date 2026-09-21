# OASIS XSLT 1.0 Nested Relative-Path Existence

Date: 2026-09-20  
Status: Local compatibility evidence

## Question

Can a location-path predicate apply a second relative child path to nodes from
a first relative child path and use the resulting node sequence's effective
boolean value?

## Method

- Compile exact `(relative-child-path)[relative-child-path]` predicates to two
  private typed child-path plans.
- Evaluate the outer path from the predicate candidate, then evaluate the
  inner path from each selected outer node until one produces a node.
- Reuse the shared charged child traversal and cancellation observation rather
  than introducing a second path evaluator.
- Add a focused positive/negative sibling regression and run unchanged
  Microsoft `Template_MatchPatternVariation8` from the hash-verified local
  OASIS XSLT 1.0 CD04 archive.

## Result

The focused expression `doc[(element1/foo)[bar]]` selects only the `doc` whose
`foo` child contains `bar`. The unchanged corpus case initializes, executes,
and exactly matches its expected XML result.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,122 | 2,123 | +1 |
| Initialization failures | 1,013 | 1,012 | -1 |
| Executed successfully | 2,033 | 2,034 | +1 |
| Execution failures | 89 | 89 | 0 |
| Exact XML-semantic matches | 1,902 | 1,903 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The strict compatibility lower bound is now `1,903 / 3,173 = 59.97%`.
The generic `FXXP1001` initialization frontier falls from 24 to 23 cases.
This is local compatibility evidence against a non-redistributed archival
suite, not a broad conformance claim.

## Boundaries

- Both paths contain only direct unqualified element or `text()` child steps
  already owned by the relative-path evaluator.
- Axes, attributes, qualified names, positional filtering, nested boolean
  expressions, and dynamic operands remain unsupported in this form.
- The compiled plan retains only stylesheet-derived path names; selected nodes
  and traversal state remain invocation-owned.

## Reproduction

```powershell
cargo test -p fastxslt --all-features path_boolean_predicates_select_nested_relative_path_existence
./scripts/measure-oasis-xslt10.ps1 -TraceCase Template_MatchPatternVariation8
./scripts/verify.ps1
```
