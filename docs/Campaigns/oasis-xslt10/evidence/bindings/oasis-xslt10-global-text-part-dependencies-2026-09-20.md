# OASIS XSLT 1.0 Global Text-Part Dependencies

Date: 2026-09-20

## Question

Can a bounded XSLT 1.0 global temporary text tree compose literal text and
`xsl:value-of` references to other globals while preserving forward dependency
ordering and cycle detection?

## Finding

Yes, for string literals and direct variable references. Compilation retains
an ordered typed list of literal and variable parts. Variable names use the
existing expanded-QName key, and every referenced global participates in the
same topological ordering and cycle detection as direct global aliases.

At invocation time, each part reads only already-materialized invocation
globals. Atomic values use their lexical value, source node sets use the first
node's charged string value, temporary trees use the shared charged document
string-value traversal, and empty sequences contribute an empty string. The
concatenated value becomes one invocation-owned temporary text node under the
existing XDM-node budget.

Focused tests prove a multi-hop forward dependency is ordered before
materialization and that a cycle made only of content-built references is
rejected as `XTDE0640`.

## Corpus movement

Three unchanged Microsoft cases move beyond the former global-constructor
frontier:

- `BVTs_bvt035` (`fwdref-nocycle`) becomes an exact result;
- `Variables__84632` reaches its expected unbound-variable error during
  execution instead of initialization;
- `BVTs_bvt036` reaches a visible local-binding shadowing defect and remains
  uncredited.

| Counter | Before | After | Change |
| --- | ---: | ---: | ---: |
| Initialized | 2,093 | 2,096 | +3 |
| Initialization failures | 1,042 | 1,039 | -3 |
| Executed successfully | 2,007 | 2,008 | +1 |
| Execution failures | 86 | 88 | +2 |
| Expected errors observed during initialization | 395 | 394 | -1 |
| Expected errors observed during execution | 28 | 29 | +1 |
| Exact XML-semantic matches | 1,876 | 1,877 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The measured exact compatibility lower bound is now
`1,877 / 3,173 = 59.15%`. The generic `FXST1015` frontier falls from 18 to 15
cases. This is local compatibility evidence against the hash-verified,
non-redistributed OASIS CD04 archive; it is not a broad conformance claim.

## Boundaries

- Only literal text plus `xsl:value-of` over a string literal or direct global
  variable reference is admitted.
- General XPath, template calls, apply-templates, choices, literal result
  elements mixed with instructions, and messages remain explicit.
- The plan does not introduce a general global instruction executor or put
  invocation values in compiled state.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_content_global_dependencies_are_ordered_and_cycles_rejected
cargo test -p fastxslt --all-features xslt10_global_text_parts_resolve_forward_dependencies_before_materialization
./scripts/measure-oasis-xslt10.ps1 -TraceFrontier FXST1015
./scripts/verify.ps1
```
