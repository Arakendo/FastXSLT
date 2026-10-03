# AR 0027 Sealed Subset Parity and Temporary Path Blocker

| Field | Value |
| --- | --- |
| Date | 2026-10-03 |
| Review | [AR-0027](../Architectural%20Reviews/AR-0027-execution-oriented-prepared-document-layout.md) |
| Checkpoint | `74daf64b` commits prior namespace integration and capacity experiments |
| Scope | Existing sealed external-subset semantics and broader temporary-tree execution |
| Disposition | Sealed-subset differential control passes; temporary-path reference panics and remains an open regression |

Capacity candidates preserve the tested sealed external-DTD behavior, including
typed IDs, defaults, entity text, whitespace views and path deduplication. A
separate temporary-tree probe fails in the ordinary growth reference before
candidate comparison. It is not evidence against pre-sizing, and cannot count
as successful temporary-tree parity or a conformance pass.

## Sealed external subset control

The [execution parity child](../../crates/fastxslt/src/runtime/prepared_input_experiment/capacity_execution_parity_tests.rs)
admits source and DTD bytes before sealing. The existing one-attempt snapshot
resolver maps `types.dtd` relative to the source logical identity; parsing sees
only those supplied bytes. No acquisition callback or ambient authority is added.
The ordinary parser retains its explicit DTD-denial outcome on the same input.
AR-0025 remains deferred; no supported DTD profile is selected.

Two authored fixtures exercise an external entity, typed ID, external attribute
default, authored override and, in the second fixture, internal-subset precedence
over the external default. Each fixture runs through growth, pre-sizing, freezing,
polled pre-sizing and polled freezing. Their XML/XDM work charges agree across
all ten construction runs. Structural comparisons check node kind, names,
relationships, order, spans, values, namespaces, typed-ID lookup and independent
origin qualification against the reference.

Twenty full transform/serialize controls cover preserving and stripping programs.
Exact outputs independently check default selection (`outer` or `inner`),
expanded entity text, one normalized parent from `/r/item/..`, and visible root
whitespace counts (three preserving, zero stripping). All ten work-domain
consumptions agree with the respective reference output. These are hand-authored
controls, not additions to the OASIS denominator.

## Open temporary path regression

The second probe constructs an XSLT 2.0 temporary document:

```xml
<xsl:variable name="t">
  <box><xsl:copy-of select="/r/item"/></box>
</xsl:variable>
<xsl:for-each select="$t/box/item">
  <!-- observe position(), last() and @key -->
</xsl:for-each>
```

Compilation succeeds. Executing the ordinary growth reference panics with:

```text
temporary-tree selection is dispatched before source selection
```

The backtrace reaches `execute_for_each_nodes` and then `select_apply_nodes` in
the [runtime owner](../../crates/fastxslt/src/runtime/golden_runtime_experiment.rs).
The selection is `ApplySelection::TemporaryPath`. Source-variable lookup cannot
serve a temporary-tree variable and returns no selection; the dispatcher falls
through to the source-only selector's `unreachable!` arm. This is a pre-existing
baseline dispatch defect, not allocation failure, transport loss or a candidate
representation mismatch.

The intended exact output is retained in the test alongside the planned
capacity/charge/result-retirement comparisons. Those later assertions have not
executed. The regression is explicitly ignored with an open-blocker reason so
ordinary verification does not silently claim that this operation passes.
Reproduce the still-failing regression directly:

```text
cargo test -p fastxslt --all-features capacity_execution_preserves_temporary_focus_paths_and_result_retirement -- --ignored --nocapture
```

Do not normalize the panic into unsupported behavior or weaken the expected
result merely to obtain a green gate. Repair or deliberately reject this
compiled combination in a separate correctness slice, review its shared
dispatch boundary under ADR-0004, then re-enable the regression and resume
temporary-tree capacity parity. No runtime repair is included here.

## Validation and next work

The sealed-subset control passes. Full verification passes with 1,526 core tests
and 48 ignored probes, including the newly explicit failing regression. Formatting,
strict Clippy, documentation, local links, conformance and unsafe checks pass.
The official WASM build passes with filesystem hard-link fallback warnings;
these test-only candidates were not run inside WASM.

The small execution child depends on its semantic-parity parent for existing
fixture/program/comparison routines and on the existing XML/resource/runtime
owners. It does not access sibling-test internals, expose parser-private failure
types, or create another builder/evaluator. Nine direct preparation-test files
remain below the directory-density trigger. Production construction and DTD
denial remain unchanged. Host ingestion, full transform peaks, the checkpoint
publication rule and broader temporary-tree parity remain prerequisites to
adoption. Prior timing reruns remain exploratory, not an acceptance decision.
