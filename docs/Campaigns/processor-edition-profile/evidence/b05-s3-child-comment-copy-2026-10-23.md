# B05-S3: child-comment copy paths

Date: 2026-10-23. Scope declared before implementation in the
[workflow](../breadth-workflow.md). Unchanged `Lotus/copy_copy41#1` first refuses
`doc/comment()` at 435..472.

Selected copying now admits `comment()` alone or following named child steps,
optionally rooted. Shared location-path compilation, navigation, result copying
and selected legacy recovery remain authoritative. Descendant comments,
predicates, PI/text/node tests, unions and general functions remain guarded.

Focused tests cover current/root focus, direct-only selection, source order,
empty selection, strip/preserve views and result-node budgets/cancellation/reuse.
The shared authored test source now includes a direct child comment so control
tests exercise actual comment copying rather than an empty selection. No upstream
fixture or expectation was changed.

Selected legacy/strict exact **508 → 509/3,173 (+1)**: `copy41` now compares
exactly. Successful executions 541 → 542; initialization failures 2,618 → 2,617.
Mismatches 2, execution failures 11 and expected-error credits 118 are unchanged.
Ordinary declaration-driven/modern remains **2,504/3,173**.

[Selected report](post-r0-child-comment-copy-selected-report-2026-10-23.txt),
[ordinary report](post-r0-child-comment-copy-baseline-report-2026-10-23.txt),
[source identity](post-r0-child-comment-copy-run-identity-2026-10-23.json).
All 2,329 workspace tests, 230 policy probes and 24 selected replay/control tests
pass. Repository gates and official WASM build also pass.
This is B05's final slice; no fourth slice is authorized before review.
