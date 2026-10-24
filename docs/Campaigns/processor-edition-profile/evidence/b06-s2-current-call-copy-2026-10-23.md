# B06-S2: direct current() copying

Scope declared before implementation in the [workflow](../breadth-workflow.md).
Unchanged `copy29` copies both `.` and `current()`; the latter remains its blocker
after B05. Admit only the complete zero-argument call via shared `CopyOfCurrent`.
No path/predicate composition, arguments or broader current-focus semantics.

The selected test matrix proves equivalence to direct `.` at nested iteration
entry, contrast with document-root copying, effective whitespace and namespace
preservation, result-node exact/one-less controls, cancellation and reuse. Both
normal and forward-compatible declared versions use the selected processor.

Selected **511 → 512/3,173 (+1)**: `copy29` now exact. Executions 544 → 545;
initialization failures 2,615 → 2,614. Mismatches 2, execution failures 11 and
expected-error credits 118 unchanged. Ordinary remains **2,504/3,173**.
All repository gates (2,330 workspace tests) and official WASM build pass.
[Selected report](post-r0-current-call-copy-selected-report-2026-10-23.txt),
[ordinary report](post-r0-current-call-copy-baseline-report-2026-10-23.txt),
[identity](post-r0-current-call-copy-run-identity-2026-10-23.json).
