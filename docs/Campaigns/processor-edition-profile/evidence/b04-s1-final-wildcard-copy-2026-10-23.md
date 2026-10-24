# B04-S1: final child-wildcard copy paths

Date: 2026-10-23. Scope declared in the [workflow](../breadth-workflow.md)
before implementation. Witnesses: `Lotus/namespace_namespace138#1`, `doc/*`
at 641..670; `Microsoft/Output__84374#1`, `/xslTutorial/*` at 164..202.

The selected guard admits a final child-element wildcard after one or more
unprefixed named child steps, with optional leading root slash. Shared copy
compilation, source navigation, namespace fixup, effective whitespace and selected
legacy recovery remain the owners. No new evaluator or source representation.
Intermediate wildcards, bare `/*`, predicates, descendants, variable/attribute/
namespace selectors and unions remain guarded.

Focused regressions cover versions 1/2/3, rooted versus relative focus, document
order, empty selection, exclusion of non-element children, copied namespace
bindings, strip/preserve views, exact/one-less result-node budgets, cancellation
and reuse. The former `num/*` negative witness is now a positive case; the negative
test uses still-excluded `num/*/child`. Its initial failure was the intentionally
changed admission boundary, not an unexplained semantic regression.

## Conserved corpus results

Selected legacy / strict: **492 → 493 / 3,173 exact**.
`namespace138` now compares exactly, including the copied namespace override.
`Output__84374` executes but its archived reference is unusable under the unchanged
harness; it earns no exact credit. Successful executions 521 → 523, initialization
failures 2,638 → 2,636, unusable-reference dispositions 6 → 7. Execution failures
(11), expected-error credits (118) and mismatches (2) are unchanged.

Ordinary declaration-driven / modern: **2,504 / 3,173**, unchanged.
Reports retain the full denominator and unchanged upstream fixture bytes:
[selected](post-r0-final-wildcard-copy-selected-report-2026-10-23.txt),
[ordinary](post-r0-final-wildcard-copy-baseline-report-2026-10-23.txt),
[source/toolchain manifest](post-r0-final-wildcard-copy-run-identity-2026-10-23.json).

Repository verification passed: fmt, Clippy, 2,325 workspace tests (2,281 core,
22 native, 4 WASM, 18 worker), docs/links, unsafe-surface and pinned conformance
checks. Additionally **8 focused copy tests**, **230 policy probes**, **24 selected
replay/control tests** and the official WASM build pass. No architectural or
correctness blocker identified. S1 is complete; carry **B04 1/3** across resumption.
The next eligible slice needs no new approval, but must declare its own scope.
