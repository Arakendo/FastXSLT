# OASIS XSLT 1.0 Boolean-Predicate Focus Position

Date: 2026-09-20  
Status: Local compatibility evidence

## Question

Can `position()` and `last()` participate in a typed boolean predicate
conjunction while retaining the original path-step focus rather than being
reapplied after another operand filters the candidate sequence?

## Method

- Pass the path-step context position and size into the existing private typed
  boolean-predicate evaluator.
- Compile admitted positional expressions as a typed boolean operand and
  propagate the same focus through `not`, `and`, and `or` recursion.
- Keep ordinary sequential XPath predicates on their existing separate path;
  this change applies only to positional operands inside one boolean predicate.
- Add a focused three-author regression proving first/last selection, then run
  unchanged Lotus `select_select48` from the hash-verified local OASIS XSLT 1.0
  CD04 archive.

## Result

The focused predicates select only the first author whose nested attribute is
`no` and the last author whose nested attribute is `yes`. The unchanged corpus
case initializes, executes, and exactly matches its expected XML result.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,123 | 2,124 | +1 |
| Initialization failures | 1,012 | 1,011 | -1 |
| Executed successfully | 2,034 | 2,035 | +1 |
| Execution failures | 89 | 89 | 0 |
| Exact XML-semantic matches | 1,903 | 1,904 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The strict compatibility lower bound is now `1,904 / 3,173 = 60.01%`.
The generic `FXXP1001` initialization frontier falls from 23 to 22 cases.
This is local compatibility evidence against a non-redistributed archival
suite, not a broad conformance claim.

## Boundaries

- Positional focus is supplied only by the owning path-step evaluation; no
  focus or invocation state enters the compiled predicate.
- This does not merge sequential predicates, reorder boolean operands, or
  admit arbitrary numeric expressions.
- Existing short-circuit order, child/attribute traversal charging, and
  cancellation observation remain unchanged.

## Reproduction

```powershell
cargo test -p fastxslt --all-features path_boolean_conjunction_preserves_original_position_focus
./scripts/measure-oasis-xslt10.ps1 -TraceCase select_select48
./scripts/verify.ps1
```
