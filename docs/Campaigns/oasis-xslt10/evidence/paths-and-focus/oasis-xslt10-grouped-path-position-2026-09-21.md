# OASIS XSLT 1.0 Grouped Path Position

Date: 2026-09-21  
Status: Local compatibility evidence

## Question

Can the bounded outer-position plan used for normalized path unions also
represent a single parenthesized location path, preserving the XPath 1.0
difference between a step predicate and a predicate over the completed path?

## Method

- Generalize the existing XSLT 1.0 parenthesized path-union position parser to
  retain one typed location path when no top-level union separator exists.
- Continue to evaluate the complete path through the charged location-path
  evaluator before applying a positive literal, `last()`, or `last() - N`
  position.
- Add a complete transform proving that `(chapter//footnote)[2]` selects the
  second node from the completed descendant path rather than the second
  `footnote` child per parent.
- Execute unchanged Lotus `position_position88`, `position_position91`, and the
  conserved 3,173-case measurement.

## Result

The focused transform selects `ahoy`. Both unchanged Lotus cases initialize,
execute, and exactly match their expected XML semantics.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,128 | 2,130 | +2 |
| Initialization failures | 1,007 | 1,005 | -2 |
| Executed successfully | 2,039 | 2,041 | +2 |
| Execution failures | 89 | 89 | 0 |
| Exact XML-semantic matches | 1,908 | 1,910 | +2 |
| XML comparison mismatches | 55 | 55 | 0 |

The strict compatibility lower bound is now
`1,910 / 3,173 = 60.20%`. This remains local compatibility evidence against a
non-redistributed archival suite, not a broad conformance claim.

## Boundaries

- The grouped expression must be one already admitted typed location path or
  an at-most-eight-path union.
- General expressions and trailing navigation after the outer predicate remain
  separate expression families.
- Positional selection remains limited to a positive literal, `last()`, or
  `last()` minus a positive literal.
- Evaluation remains invocation-owned and retains the existing cancellation
  and node-visit charge points.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_parenthesized_descendant_paths_apply_outer_positions
./scripts/measure-oasis-xslt10.ps1 -TraceCase Lotus/position_position88#1
./scripts/measure-oasis-xslt10.ps1 -TraceCase Lotus/position_position91#1
./scripts/verify.ps1
```
