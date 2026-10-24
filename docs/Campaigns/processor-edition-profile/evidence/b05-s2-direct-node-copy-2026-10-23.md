# B05-S2: direct current/root node copying

Date: 2026-10-23. Scope declared in the [workflow](../breadth-workflow.md).
Recorded unchanged witnesses: `copy55` selects `.` (709..734); `copy43` selects
`/`. Admit these complete selectors only, through shared current-node/location-
path copying with selected legacy recovery explicitly retained.

`current()`, extended self/parent paths, namespace/attribute selectors, variables
and general expressions remain guarded. No new node identity, owner, result
construction or navigation semantics. Tests distinguish current element from
document root under nested focus, verify document comments/PIs and child copying,
namespace ownership and strip/preserve views; existing exact/one-less result-node
budget and cancellation/reuse checks now include both selectors.

Selected exact **501 → 508/3,173 (+7)**; successful executions 534 → 541,
initialization failures 2,625 → 2,618. Mismatches 2, execution failures 11 and
expected-error credits 118 remain unchanged. `copy55` compares exactly, including
namespace rebinding. Ordinary remains **2,504/3,173**.

[Selected report](post-r0-direct-node-copy-selected-report-2026-10-23.txt),
[ordinary report](post-r0-direct-node-copy-baseline-report-2026-10-23.txt),
[identity](post-r0-direct-node-copy-run-identity-2026-10-23.json).
Repository gates pass (2,328 workspace tests); official WASM build passes.
No regression or architectural blocker identified.
