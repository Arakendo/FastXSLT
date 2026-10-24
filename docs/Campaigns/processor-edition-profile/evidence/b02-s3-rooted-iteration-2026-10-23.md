# B02-S3: rooted named-child iteration

Date: 2026-10-23. Scope was declared in the [live register](../breadth-workflow.md)
before implementation. Direct source inspection and full replay establish that
`Microsoft/AVTs__77574#1` refused `/bookstore` at 197..231 in `XSLT17007.xsl`.

Selected `for-each` now admits one leading slash followed by nonempty ASCII
named-child steps, using shared selection parsing/evaluation and selected body
lowering. `/` alone, qualified/wildcard/predicate steps, mixed descendant paths,
sorting, resource functions and new body expressions remain guarded. This is a
local eligibility change, not a new parser, evaluator, owner or policy.

## Conserved measurement

Unchanged pinned OASIS archive, original expectations and 3,173-case denominator:

- Selected legacy / strict exact: **486 → 487 (+1), 15.35%**.
- Successful executions: **515 → 516**; initialization failures **2,644 → 2,643**.
- Execution failures remain 11, separate expected-error credits 118, mismatches 2.
  Exclusions, comparator limits and mismatch identities are unchanged.
- Ordinary declaration-driven / modern: **2,504/3,173 (78.92%), unchanged**.
- The named witness reaches its separate literal attribute-value expression
  guard at 245..269. Its expected output is not credited; that adjacent expression
  admission is outside this slice. The full replay, not the witness alone,
  establishes the one exact-case gain.

[Selected report](post-r0-root-iteration-selected-report-2026-10-23.txt),
[ordinary report](post-r0-root-iteration-baseline-report-2026-10-23.txt),
[source/toolchain identity](post-r0-root-iteration-run-identity-2026-10-23.json).

## Verification

Focused selected-entry tests pass for versions 1/2/3, absolute versus relative
selection from non-root focus, ordered siblings and deeper child paths, empty
selection, whitespace-view composition, restored caller focus and excluded
syntax/provenance. Controls now cover rooted paths with exact/one-less XPath
node visits, cancellation during charged work and reuse.

Full repository verification passes: **2,317 workspace tests** (2,273 core,
22 native, 4 WASM, 18 worker), fmt, Clippy, docs/links, unsafe and pinned corpus
checks. The 230 policy probes, 24 selected replay/control tests and official
WASM build pass. No unexplained regression or provisional blocker. The change
remains within the existing iteration owner and its private focused tests.

Reproduce with `cargo test -p fastxslt --all-features selected_rooted_iteration`,
`cargo test -p fastxslt --all-features selected_descendant_iteration_controls`,
standard repository/policy/WASM gates and both full measurement profiles; use
`-TraceCase AVTs__77574` for the selected witness. **B02 3/3: stop for review.**
