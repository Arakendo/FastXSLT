# B04-S2: final child-wildcard iteration

Date: 2026-10-23. Scope declared in the [workflow](../breadth-workflow.md).
Unchanged `Microsoft/Output__84616#1` refuses
`for-each select="SALAD/SALAD_INGREDIENTS/*"` at 232..281.

Selected iteration now admits a final child-element wildcard after one or more
unprefixed named child steps, optionally rooted. Shared selection and selected
body lowering remain unchanged. No intermediate wildcard, bare `/*`, sorting,
predicates, other axes, variable or secondary-document selection is admitted.

Tests cover selected versions 1/2/3, relative/rooted focus, position/size and
caller-focus restoration, namespaced child elements, text/comment/PI exclusion,
strip/preserve views, empty selections, exact/one-less node-visit budgets,
cancellation and reuse. Previously negative final-wildcard witnesses are replaced
by still-excluded intermediate-wildcard paths.

Complete: selected exact **493 → 500/3,173 (+7)**; successful executions
523 → 530, initialization failures 2,636 → 2,629. Execution failures 11,
expected-error credits 118, mismatches 2 and other dispositions unchanged.
`Output__84616` compares exactly. Ordinary remains **2,504/3,173**.

[Selected report](post-r0-final-wildcard-iteration-selected-report-2026-10-23.txt),
[ordinary report](post-r0-final-wildcard-iteration-baseline-report-2026-10-23.txt),
[source identity](post-r0-final-wildcard-iteration-run-identity-2026-10-23.json).

Repository gates pass: fmt, Clippy, 2,326 workspace tests (2,282 core, 22 native,
4 WASM, 18 worker), docs/links, unsafe-surface and pinned conformance. Official
WASM build passes. No regression/blocker; B04 advances to 2/3.
