# Temporary path for each dispatch repair

Date: 2026-10-03. Scope: a separate runtime correctness tranche following the
AR-0027 capacity-parity checkpoint committed at `1e04cc7f`.

Follow-up status: the [attribute-value repair](ar-0027-temporary-attribute-value-parity-2026-10-03.md)
closes the remaining `@key` blocker and re-enables the unchanged regression.
The failures and ignored-test status below describe this earlier checkpoint.

Temporary-path for-each no longer falls into the source-only selector's panic.
The unchanged broader capacity regression remains failing: its `@key` expression
exposes a second ordinary value-path limitation under temporary focus. This is
partial repair, not completed temporary-tree parity or a conformance gain.

## Shared execution boundary

First, the existing for-each dispatch was moved mechanically into a
[private owner](../../crates/fastxslt/src/runtime/golden_runtime_experiment/for_each/mod.rs).
All 16 focused for-each tests and the full verification script passed before
semantic repair, with 1,526 core passes and 48 ignored probes.

The existing temporary exact-child path selector was then factored from
`apply_temporary_path` within its existing owner. Both apply-templates and
for-each reuse its charged visits. For-each constructs `TemporaryFocus::Node`
with the owning temporary tree, resets position/size and unrelated focus kinds,
and delegates the body to the existing sequence executor. Source-variable paths
retain their source dispatch. No arena ID is fabricated, prepared state is not
mutated, and runtime values do not acquire cross-invocation ownership.

Sorted temporary paths now return `FXRT0007 / unsupported` with the sort's
stylesheet provenance rather than silently ignoring sort keys. This does not
admit general temporary-axis evaluation or sorting.

The new owner contains 334 lines and its tests are a private child. It does not
own compilation, XML admission, runtime variable storage, host policy or result
serialization. The parent remains oversized at 7,437 lines; this extraction
addresses one cohesive seam, not the entire decomposition debt. The existing
temporary executor remains the owner of tree navigation. No new backend,
public API, unsafe code, authority or architectural contract was introduced.

## Executable evidence

The [hand-authored golden fixture](../../corpus/golden/temporary-path-for-each/stylesheet.xsl)
uses a global two-item temporary path, nested local three-item paths and empty
selections. Its exact result proves independent focus sizing; the source owner
is dropped before serialization. This is regression evidence, not an imported
standards test.

Three focused tests verify:

- exact nested positions and sizes, empty-path behavior, and result retirement;
- explicit sorted-path rejection with request identity and stylesheet location;
- visit-budget exhaustion at zero and one less than the successful total,
  success at the exact total, pre-cancellation, and clean subsequent execution.

The original capacity regression retains its stylesheet and expected output.
Explicit execution now returns `XPDY0002 / invalid` from ordinary attribute-value
path evaluation instead of reaching the source selector's `unreachable!` arm.
The test's unwrap makes that run fail; the diagnostic is not being credited as
correct attribute semantics. Its ignore reason names the remaining limitation.
Candidate output, charge and retirement assertions after that failure remain
unexecuted.

```text
cargo test -p fastxslt --all-features temporary_path_for_each -- --nocapture
cargo test -p fastxslt --all-features capacity_execution_preserves_temporary_focus_paths_and_result_retirement -- --ignored --nocapture
```

The second command is intentionally failing until the remaining value-path seam
is repaired. Production capacity construction stays unchanged; OASIS coverage
stays 2,485 / 3,173 (78.32%). AR-0027 remains Incubating.

## Remaining work

Full verification passes after repair with 1,529 core tests and 48 ignored
probes. Formatting, strict Clippy, workspace tests, documentation, links,
conformance sources/inventory and unsafe-surface checks pass. The official
WASM build check passes with filesystem hard-link fallback warnings. These
golden tests were not executed inside a WASM runtime.

Resolve ordinary location-path scalar evaluation under temporary focus through
the shared semantic machinery, with source/temporary differential controls and
explicit unsupported boundaries where parity is absent. Then re-enable the
unchanged capacity regression and complete broader candidate comparisons.
Capacity adoption, host ingestion measurements and the checkpoint publication
rule remain independent unresolved questions.
