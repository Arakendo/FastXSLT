# B03-S3: selected boolean constant copy

Date: 2026-10-23. Complete; B03 reaches 3/3 and stops for maintainer review.

Fresh full frontier replay identified unchanged `Lotus/copy_copy12#1`,
`copy12.xsl` 373..403: `<xsl:copy-of select="true()"/>` refused by selected
admission. Scope declared before implementation in the
[workflow](../breadth-workflow.md): exact zero-argument `true()`/`false()` with
outer whitespace, through existing shared static atomic copying only.

The selected guard now admits those two calls and requires the existing shared
compiler to produce `CopyOfStaticAtomicText`. No second expression evaluator,
new function implementation or introspection change. Numeric literals, variables,
arguments, general boolean expressions and other function calls remain guarded.

Focused tests pin true/false text under selected declared versions 1/2/3,
adjacent unsupported shapes and provenance, exact/one-less result-node budgets,
instruction cancellation and clean compiled-artifact reuse.

## Full replay

Selected exact **491 → 492 / 3,173**. `copy12` now compares exactly. Successful
executions 520 → 521, initialization failures 2,639 → 2,638. Execution failures
11, expected-error credits 118, mismatches 2 and other dispositions are unchanged.
The ordinary report is byte-for-byte unchanged at **2,504/3,173**.

[Selected report](post-r0-boolean-copy-selected-report-2026-10-23.txt),
[ordinary report](post-r0-boolean-copy-baseline-report-2026-10-23.txt),
[source/toolchain identity](post-r0-boolean-copy-run-identity-2026-10-23.json).

Verification: 2,324 workspace tests (2,280 core, 22 native, 4 WASM, 18 worker),
230 policy probes, 24 selected replay/control tests, fmt, Clippy, docs/links,
unsafe-surface and pinned conformance checks pass. Official WASM build passes.
No regression or provisional blocker remains. No fourth slice started.
