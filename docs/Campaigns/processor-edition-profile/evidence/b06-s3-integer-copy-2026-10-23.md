# B06-S3: bounded integer constant copying

Scope recorded before implementation in the [workflow](../breadth-workflow.md).
Unchanged `copy06` copies `32`. Reuse selected argument admission's signed i32
ASCII integer domain, then shared static atomic text copying. All admitted values
are exactly representable in the legacy numeric domain; this does not admit
arbitrary i64 literals, decimals, exponent notation, leading plus, arithmetic,
variables or general numeric calls.

Tests pin leading zeroes, negative zero, signed endpoints, surrounding whitespace,
normal/forward-compatible declared versions and rejection beyond the domain.
Result-node budgets, cancellation and reuse use the existing control oracle.
An old numeric-rejection test now guards a decimal instead. An erroneous new
negative test for `NaN` was removed: it is a legal child-name path, not a numeric
literal. No runtime change was made to accommodate that test.

Final replay: selected **512 → 513/3,173 (+1)**; `copy06` exact. Executions
545 → 546; initialization failures 2,614 → 2,613. Existing mismatches 2,
execution failures 11 and expected-error credits 118 unchanged. Ordinary baseline
independently replayed at **2,504/3,173**. All repository gates, 2,331 workspace
tests, 230 policy probes, 24 selected replay/control tests and WASM pass.
[Selected report](post-r0-numeric-copy-selected-report-2026-10-23.txt),
[ordinary report](post-r0-numeric-copy-baseline-report-2026-10-23.txt),
[identity](post-r0-numeric-copy-run-identity-2026-10-23.json).
