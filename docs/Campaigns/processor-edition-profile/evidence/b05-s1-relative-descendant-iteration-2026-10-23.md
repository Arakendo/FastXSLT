# B05-S1: relative descendant iteration

Date: 2026-10-23. Scope declared before implementation in the
[workflow](../breadth-workflow.md). Recorded unchanged first refusals:
`conditional19` `.//title` (384..416), `position37` `.//a` (410..438).

Admit only `.//` followed by one unprefixed name in selected `xsl:for-each`.
The shared selector/runtime owns navigation and focus; selected body admission
is unchanged. Chains, wildcards, predicates, sorting and other axes remain
excluded. Tests distinguish root `//a` from context-relative `.//a`, verify that
the current node itself is not selected, and cover order, position/size, caller
focus restoration, empty selection, whitespace views, exact/one-less visit
budgets, cancellation and reuse under selected declared versions 1/2/3.

Complete: full conserved reports remain **501/3,173 selected**, **2,504/3,173
ordinary**, with unchanged counters. `position37` advances to a later value-of
guard at 542..572; no adjacent expression expansion. This closes the named
selection blocker but earns no exact credit.

[Selected report](post-r0-relative-descendant-iteration-selected-report-2026-10-23.txt),
[ordinary report](post-r0-relative-descendant-iteration-baseline-report-2026-10-23.txt),
[identity](post-r0-relative-descendant-iteration-run-identity-2026-10-23.json).
All repository gates (2,327 workspace tests) and official WASM build pass.
No regression or architectural blocker. B05 1/3; reassess another family.
