# OASIS XSLT 1.0 Path-Union Count and Position

Date: 2026-09-21  
Status: Local compatibility evidence

## Question

Can XSLT 1.0 count a union of typed location paths and apply `last()` to the
same normalized union without introducing a separate union evaluator?

## Method

- Compile at most eight location-path alternatives inside `count()` through
  the existing XSLT 1.0 path parser.
- Compile a parenthesized path union followed by a literal, `last()`, or
  `last() - N` position into one typed apply selection.
- Evaluate every alternative through the charged path evaluator, then reuse
  the shared union normalization that restores document order and removes
  duplicate node identities.
- Apply count or checked positional selection only after normalization.
- Add a complete transform covering both operations, then execute unchanged
  Lotus `position_position80` and the conserved measurement.

## Result

The golden transform counts three distinct ancestors and selects the last
normalized ancestor. Unchanged Lotus `position_position80` initializes,
executes, and exactly matches its expected XML semantics.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,127 | 2,128 | +1 |
| Initialization failures | 1,008 | 1,007 | -1 |
| Executed successfully | 2,038 | 2,039 | +1 |
| Execution failures | 89 | 89 | 0 |
| Exact XML-semantic matches | 1,907 | 1,908 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The strict compatibility lower bound is now
`1,908 / 3,173 = 60.13%`. This remains local compatibility evidence against a
non-redistributed archival suite, not a broad conformance claim.

## Boundaries

- Both union forms are limited to eight already admitted location paths.
- General expressions, function calls, and variables inside these path unions
  remain separate typed families.
- Positional selection is limited to a positive literal, `last()`, or
  `last()` minus a positive literal.
- All selected nodes remain invocation-owned and no normalized union is cached
  across invocations.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_path_unions_support_count_and_last_position
./scripts/measure-oasis-xslt10.ps1 -TraceCase position_position80
./scripts/verify.ps1
```
