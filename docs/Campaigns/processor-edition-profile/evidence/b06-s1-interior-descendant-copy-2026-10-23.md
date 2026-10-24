# B06-S1: interior descendant copy paths

Scope declared before implementation in the [workflow](../breadth-workflow.md).
Unchanged `Lotus/copy_copy10#1` refuses `docs//d` at 504..535. Admit one interior
descendant separator between unprefixed named child paths, optionally rooted.
Repeated separators, wildcards, predicates, functions and other axes remain
guarded. Shared location-path compilation and copy execution remain authoritative;
selected legacy unattached-attribute recovery is explicitly retained.

Focused tests pin root/relative focus, nested descendants, source order,
unprefixed name matching, empty selection, strip/preserve views, result-node
exact/one-less budgets, cancellation and reuse. No fixture changes.

Selected exact **509 → 511/3,173 (+2)**; successful executions 542 → 544;
initialization failures 2,617 → 2,615. Mismatches 2, execution failures 11 and
expected-error credits 118 unchanged. Ordinary remains **2,504/3,173**.
All repository gates (2,330 workspace tests) and WASM pass. An obsolete rejection
witness for the newly admitted shape was changed to a repeated descendant path.
[Selected report](post-r0-interior-descendant-copy-selected-report-2026-10-23.txt),
[ordinary report](post-r0-interior-descendant-copy-baseline-report-2026-10-23.txt),
[identity](post-r0-interior-descendant-copy-run-identity-2026-10-23.json).
