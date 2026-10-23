# OASIS XSLT 1.0 Global Source-Path Text Parts

Date: 2026-09-20  
Status: Local compatibility evidence

## Question

Can the bounded XSLT 1.0 global temporary-text plan compose a source location
path with literal text while keeping source state invocation-owned and reusing
the charged location-path evaluator?

## Method

- Add a compiled source-location-path part to the existing private global
  temporary-text plan.
- Evaluate each path from the current invocation's principal document and use
  the first selected node's controlled string value, or the empty string when
  the selection is empty.
- Retain only the typed path in compiled state and materialize one charged
  temporary text node per invocation.
- Add a focused regression composing `//Test/@a`, `//Test/@b`, and literal
  suffixes.
- Run the hash-verified 3,173-case OASIS CD04 measurement and trace unchanged
  Lotus `variable64` and `variable65` cases.

## Result

The focused regression produces `<out>Ax|By</out>`. Unchanged Lotus
`variable64` becomes exact. Both `variable65` variants pass global
construction and stop later at the existing unsupported `string($by)` template
argument, so they are not credited as passes.

| Counter | Before | After | Delta |
| --- | ---: | ---: | ---: |
| Initialized | 2,096 | 2,097 | +1 |
| Initialization failures | 1,039 | 1,038 | -1 |
| Executed successfully | 2,009 | 2,010 | +1 |
| Execution failures | 87 | 87 | 0 |
| Exact XML-semantic matches | 1,878 | 1,879 | +1 |
| XML comparison mismatches | 55 | 55 | 0 |

The exact compatibility lower bound is now `1,879 / 3,173 = 59.22%`. The
generic `FXST1015` initialization frontier falls from 15 to 12 cases. This is
local compatibility evidence against the hash-verified, non-redistributed
OASIS CD04 archive; it is not a broad conformance claim.

## Boundaries

- Only location paths already admitted by the shared parser/evaluator are
  accepted as source parts.
- Evaluation uses the invocation's principal source; no source node or dynamic
  value enters compiled state.
- Each part uses first-node string semantics rather than concatenating every
  selected node.
- Arbitrary expressions and general global instruction sequences remain
  explicit unsupported boundaries.

## Reproduction

```powershell
cargo test -p fastxslt --all-features xslt10_global_text_parts_compose_source_paths_and_literal_text
./scripts/measure-oasis-xslt10.ps1 -TraceCase variable_variable64
./scripts/measure-oasis-xslt10.ps1 -TraceCase variable_variable65
./scripts/verify.ps1
```
