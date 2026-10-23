# OASIS XSLT 1.0 Key Number Pattern

Date: 2026-09-23  
Status: Local implementation and compatibility evidence

## Question

Can `xsl:number` use a static XSLT 1.0 `key()` count pattern without adding a
second key evaluator, weakening modern semantics, or hiding its work from the
invocation budget?

## Method

- Reuse the existing typed `Xslt10KeyLookup` produced from literal key name and
  value arguments.
- Evaluate the declared key definitions through the existing charged static-key
  selector once per number-instruction invocation.
- Retain the selected source-node identities only for that instruction
  invocation, then use ordinary charged count/from traversal against the
  evaluated membership.
- Preserve the existing unsupported boundary for the same pattern in an XSLT
  3.0 stylesheet.
- Add focused execution and cross-version compilation regressions.
- Rerun the unchanged 3,173-case OASIS catalog.

## Result

Unchanged `Lotus/numbering_numbering90#1` moves from initialization failure at
`FXST1050` to an exact XML-semantic expected-result match.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Catalog cases | 3,173 | 3,173 | 0 |
| Initialized | 2,208 | 2,209 | +1 |
| Executed successfully | 2,155 | 2,156 | +1 |
| Exact expected-result matches | 2,007 | 2,008 | +1 |
| Initialization failures | 927 | 926 | -1 |
| Execution failures | 53 | 53 | 0 |
| XML comparison mismatches | 64 | 64 | 0 |
| Comparator-unsupported results | 75 | 75 | 0 |

The strict complete-catalog lower bound is now `2,008 / 3,173 = 63.32%`.

## Boundaries

- The admitted number-pattern form requires static XSLT 1.0 `key()` name and
  value arguments; predicates, tails, and dynamic arguments remain explicit.
- Key-definition matching, `use` evaluation, node visits, and number traversal
  retain their existing work charges and cancellation observation.
- Evaluated membership is invocation-local and short-lived. This change does
  not widen ADR-0013's compiled-template cache, create cross-invocation state,
  or select a global key index.
- Modern XSLT retains the prior `FXST1050` boundary for this compatibility-only
  pattern.
- The local OASIS archive remains non-redistributed. This is compatibility
  evidence, not an XSLT 1.0 conformance claim.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_single_number_counts_static_key_membership
cargo test -p fastxslt --all-features keeps_key_number_patterns_inside_xslt10_compatibility
./scripts/measure-oasis-xslt10.ps1 -TraceCase 'Lotus/numbering_numbering90#1'
./scripts/verify.ps1
```
