# B01-S2: standalone child-element wildcard copying

Ordinary bounded breadth, 2026-10-23, under the [live register](../breadth-workflow.md).
The unchanged `Lotus/copy_copy04#1` refused `copy-of select="*"` with FXAR0031.
The selected guard now admits exactly standalone `*` (surrounding whitespace
allowed), lowers through the existing shared copy compiler and requires
`CopyOfChildElements` or the previously admitted named `CopyOfLocationPath`.
Legacy recovery remains explicitly selected. No runtime or policy owner changed.
Wildcard path steps, attributes, predicates, descendant, namespace, context and
parent selections remain refused; no adjacent instruction was widened.

## Conserved results

Same pinned OASIS archive and unchanged 3,173-case denominator as
[S1](b01-s1-selected-processing-instructions-2026-10-23.md):

- Selected legacy / strict exact: **466 → 469 (+3)**.
- Successful executions: **495 → 498**; initialization failures **2,665 → 2,662**.
- Execution failures remain 10; mismatches remain 2 with unchanged identities.
- Expected-error credits remain 117, separately counted; exclusions unchanged.
- Ordinary declaration-driven / modern: **2,504 exact, unchanged**.
- `copy04` now passes the original comparator. No fixtures/classifiers changed.

[Selected report](post-r0-wildcard-selected-report-2026-10-23.txt),
[ordinary report](post-r0-wildcard-baseline-report-2026-10-23.txt),
[source/toolchain manifest](post-r0-wildcard-run-identity-2026-10-23.json).

## Validation and closure

Five selected copy-entry tests pass, including the new wildcard test across
versions 1/2/3, non-root focus, ordered children, empty selection, exclusion of
non-element children, retained subtree namespaces/attributes/comments/PIs and
effective whitespace views. Controls also exercise wildcard exact/one-less
result budgets, cancellation and reuse. Existing refusal tests now reject
wildcard path steps rather than the newly admitted standalone wildcard.

Full verification passes: **2,311 workspace tests** (2,267 core, 22 native,
4 WASM, 18 worker), fmt, Clippy, docs/links, unsafe and pinned conformance checks.
The 230 policy probes, 24 selected replay/control tests and official WASM build
pass. No new mismatch, execution failure or provisional blocker was observed.
The small guard change and existing copy-focused test owner require no new
decomposition under ADR-0004. S2 is complete; B01 is **2/3**.

Reproduce with `cargo test -p fastxslt --all-features selected_copy`, the S1 gate
commands, and the full selected measurement with `-TraceCase copy_copy04`.
No default, API or conformance claim changed.
