# OASIS XSLT 1.0 Variable Node Position

Date: 2026-09-21  
Status: Local compatibility evidence

## Question

Can an XSLT 1.0 source-node variable be filtered by a positive literal
position, `last()`, or checked `last() - N` through both value and
sequence-selection consumers?

## Method

- Compile `$variable[N]`, `$variable[last()]`, and `$variable[last() - N]`
  into one private typed source-node selector when `N` is a positive `usize`.
- Reuse the invocation-owned source-node variable map for both `xsl:value-of`
  and sequence selection such as `xsl:for-each`.
- Charge the XPath operation against the retained node-set length before
  selecting a node.
- Preserve structured `XPTY0004` failure when the named variable is not a
  source-node sequence.
- Add one complete transform covering first, interior, and last selection,
  then run unchanged Lotus `position_position92` and the complete conserved
  measurement.

## Result

The golden transform selects `a`, `c`, `c`, and `d` through `$nodes[1]`,
`$nodes[3]`, `$nodes[last() - 1]`, and `$nodes[last()]`. Unchanged Lotus `position_position92`
initializes, executes, and exactly matches its expected XML result.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,125 | 2,126 | +1 |
| Initialization failures | 1,010 | 1,009 | -1 |
| Executed successfully | 2,036 | 2,037 | +1 |
| Execution failures | 89 | 89 | 0 |
| Exact XML-semantic matches | 1,905 | 1,906 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The strict compatibility lower bound is now
`1,906 / 3,173 = 60.07%`. This is local compatibility evidence against a
non-redistributed archival suite, not a broad conformance claim.

## Boundaries

- This admits a positive literal integer, exact `last()`, or `last()` minus a
  positive literal only; it does not add general predicate expressions over
  variable sequences.
- The selected nodes must belong to the current source document. Temporary
  trees and atomic sequences retain their existing separate behavior.
- No compiled artifact retains invocation nodes or mutable variable state.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_source_node_variables_support_literal_and_last_positions
./scripts/measure-oasis-xslt10.ps1 -TraceCase position_position92
./scripts/verify.ps1
```
