# B02-S1: standalone wildcard iteration

Date: 2026-10-23. Maintainer approved continuation after B01 under the same
[bounded workflow](../breadth-workflow.md). No ADR or promotion change.

## Scope and witness correction

The predeclared semantic scope admits only standalone `*` in selected legacy
`for-each`, through the existing shared selection evaluator and selected body
lowering. Wildcard path steps, predicates, sorting, attributes, namespace axes
and other instructions remain guarded. No runtime changes were necessary.

The initial ledger incorrectly associated `copy07`'s first-refusal span 601..626
with its wildcard loop. Byte-span inspection shows that span is its attribute
template. This attribution error is corrected, not silently rewritten as a gain.
The saved preimplementation full frontier independently contains actual wildcard
iteration failures, including `Lotus/node_node07#1` at 604..629 and
`Microsoft/Elements__89065#1` at 188..214. The latter now compares `true` to the
unchanged expected `true`; `node07` reaches its separate variable-copy guard
at 1160..1188. No adjacent feature was admitted to finish those remaining cases.
An exploratory bulk mapping emitted invalid-span errors; its aggregate output
is not used as a yield estimate. Direct case/source checks establish these witnesses.

## Full replay and gates

Same pinned OASIS archive, profiles, original expectations and conserved 3,173
cases as [B01](b01-review-2026-10-23.md):

- Selected legacy / strict: **482 → 483 exact (+1), 15.22%**.
- Successful executions: **511 → 512**; initialization failures **2,648 → 2,647**.
- Execution failures remain 11; expected-error credits remain 118 (separate).
- Mismatch identities/count (2), exclusions and comparator limits are unchanged.
- Ordinary declaration-driven / modern: **2,504/3,173 (78.92%), unchanged**.

[Selected report](post-r0-wildcard-iteration-selected-report-2026-10-23.txt),
[ordinary report](post-r0-wildcard-iteration-baseline-report-2026-10-23.txt),
[source/toolchain identity](post-r0-wildcard-iteration-run-identity-2026-10-23.json).

The focused selected-entry test covers versions 1/2/3, namespaced element
children, exclusion of text/comments/PIs, document order, empty groups,
position/size restoration and effective whitespace string values. The existing
control test now also exercises wildcard exact/one-less XPath visit limits,
charged cancellation and reuse. Adjacent selector families remain explicitly
unsupported with principal-resource provenance.

All repository gates pass: **2,315 workspace tests** (2,271 core, 22 native,
4 WASM, 18 worker), fmt, Clippy, docs/links, unsafe-surface and pinned corpus
checks. **230** policy probes, **24** selected replay/control tests and the
official WASM build pass. No unexplained regression or provisional blocker.
This small eligibility change stays with the existing iteration owner and
cohesive iteration tests; no additional decomposition is warranted.

Reproduce with the standard full gates, `cargo test -p fastxslt --all-features
selected_wildcard_iteration`, `cargo test -p fastxslt --all-features
selected_descendant_iteration_controls`, and full selected measurement using
`-TraceCase Elements__89065` or `-TraceCase node_node07`. S1 complete: **B02 1/3**.
Reassess another concrete blocker before S2; no per-slice approval is needed.
