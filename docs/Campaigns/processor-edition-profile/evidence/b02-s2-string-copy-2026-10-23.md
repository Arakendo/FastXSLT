# B02-S2: quoted string copying

Ordinary bounded breadth, 2026-10-23, under the [live workflow](../breadth-workflow.md).
Unchanged `Lotus/copy_copy05#1` refused `copy-of select="'test'"` at 366..396.
Selected copying now admits complete single/double-quoted XPath 1.0 literals
without an embedded matching delimiter. Shared lowering must return
`CopyOfStaticAtomicText`; shared execution owns text construction and charging.
Numeric/boolean/arithmetic/variable values, malformed syntax and modern doubled
quotes remain outside this slice. No evaluator, authority or policy changed.

## Replay

Unchanged OASIS archive and 3,173-case profiles from the prior slice:

- Selected legacy/strict exact: **483 → 486 (+3), 15.32%**.
- Successes: **512 → 515**; initialization failures: **2,647 → 2,644**.
- Execution failures remain 11, expected-error credits 118, mismatches 2.
  Exclusion counts and mismatch identities remain unchanged.
- Ordinary declaration-driven/modern: **2,504 exact, unchanged**.
- `copy05` passes unchanged. No fixture or comparison changes.

[Selected report](post-r0-string-copy-selected-report-2026-10-23.txt),
[ordinary report](post-r0-string-copy-baseline-report-2026-10-23.txt),
[source/toolchain manifest](post-r0-string-copy-run-identity-2026-10-23.json).

## Gates and closure

Six focused copy tests pass, including both quote styles, empty text, Unicode,
XML escaping, opposite quotation marks, versions 1/2/3 and refused adjacent
syntax with provenance. The control test additionally covers literal copy with
exact/one-less result-node limits, cancellation and reuse. Existing path and
wildcard behavior remains covered. The helper remains within the copy owner.

Repository verification passes: **2,316 workspace tests** (2,272 core, 22 native,
4 WASM, 18 worker), fmt, Clippy, docs/links, unsafe and pinned conformance checks;
230 policy probes, 24 selected replay/control tests and official WASM build pass.
No provisional blocker or unexplained regression. S2 complete: **B02 2/3**.

Reproduce with `cargo test -p fastxslt --all-features selected_copy`, the standard
repository/policy/WASM gates and both complete measurement profiles; use
`-TraceCase copy_copy05` for the selected witness. Original reports remain intact.
