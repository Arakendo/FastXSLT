# B03-S1: bounded ISO-8859-1 output admission

Date: 2026-10-23. B03 was approved after the B02 review. The
[live register](../breadth-workflow.md) records the preimplementation boundary.

## Reassessment and scope

A fresh full selected-profile frontier mapped repeated output declarations to
their original byte spans. ISO-8859-1 declarations are first blockers for
`Lotus/copy_copy19#1`, `copy22` and `numberformat06`, among others. The mapping
is a leverage clue, not a conserved pass estimate; only complete corpus replay
earns exact credit. Other repeated blockers include template patterns, shallow
copying and output options, which remain separate work.

Selected output validation now admits the exact case-insensitive `ISO-8859-1`
label alongside existing UTF-8. Shared output compilation, semantic execution
and bounded byte serialization are unchanged. No alias (`latin1`, `unicode`),
other encoding, public API, default or string-result behavior is admitted.
The existing string serialization lane still refuses this encoding rather than
mislabeling encoded bytes as a UTF-8 string.

## Conserved full replay

Same pinned OASIS archive and original 3,173-case denominator and comparisons:

- Selected legacy/strict exact: **487 → 491 (+4), 15.47%**.
- Successful executions: **516 → 520**; initialization failures **2,643 → 2,639**.
- Execution failures remain 11, separate expected-error credits 118, mismatches 2.
  Exclusions and mismatch identities remain unchanged.
- Ordinary declaration-driven/modern: **2,504/3,173 (78.92%), unchanged**.
- `copy19` now reaches and passes the original comparison. Most apparent
  encoding-frontier candidates retain independent downstream guards; no broad
  yield claim follows from counting output declarations.

[Selected report](post-r0-latin1-selected-report-2026-10-23.txt),
[ordinary report](post-r0-latin1-baseline-report-2026-10-23.txt),
[source/toolchain manifest](post-r0-latin1-run-identity-2026-10-23.json).

## Verification and limits

Three direct selected-entry tests pass across declared versions 1/2/3: actual
Latin-1 bytes for representable text, XML character-reference fallback, guarded
encodings and preserved diagnostic resource identity, unrepresentable text and
comment failure/reuse, output ceiling, charged cancellation and result reuse.
The existing serializer bounds intermediate UTF-8 as well as final bytes.
The exact/one-less output ceiling test therefore uses ASCII (equal lengths);
non-ASCII encoding is tested separately. This does not redefine byte accounting
or assert that the smallest successful ceiling always equals final byte length.

Repository gates pass: **2,320 workspace tests** (2,276 core, 22 native, 4 WASM,
18 worker), fmt, Clippy, docs/links, unsafe-surface and pinned corpus checks.
The 230 policy probes, 24 selected replay/control tests and official WASM build
pass. No unexplained regression or provisional blocker. The eligibility change
remains inside selected output validation; a cohesive private encoding-test
file takes the resource-compiler directory to nine direct files, with no new
runtime ownership or decomposition needed.

Reproduce with `cargo test -p fastxslt --all-features selected_latin1`, standard
repository/policy/WASM gates, and both complete measurement profiles. Use
`-TraceCase copy_copy19` for the selected witness. **B03-S1 complete; B03 1/3.**
Reassess and declare the next bounded slice; no individual approval is required.
