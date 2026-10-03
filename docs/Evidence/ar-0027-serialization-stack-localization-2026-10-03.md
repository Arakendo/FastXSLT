# Deep result serialization stack failure

- Date: 2026-10-03
- Status: Historical extraction checkpoint; observed blocker subsequently resolved
- Related review: [AR-0027](../Architectural%20Reviews/AR-0027-execution-oriented-prepared-document-layout.md)
- Related decision: [ADR-0004](../ADR/ADR-0004-source-unit-cohesion-size-pressure-and-decomposition.md)

The 256-deep ordinary growth-tree copy reference failed during serialization,
not during copying or source retirement. Test-only phase markers establish
that execution completes and the source is dropped before the debug harness
overflows its default thread stack inside the serialization call. This narrows
the repair without attributing the failure to a capacity candidate. The
[subsequent stack-safe repair and allocation replay](ar-0027-stack-safe-serialization-and-allocation-replay-2026-10-03.md)
resolve this blocker. The reproduction and gate observations below describe
the unchanged extraction checkpoint, not the current repaired writer.

## Reproduction

```text
cargo test -p fastxslt --all-features capacity_full_transform_deep_copy_reference -- --ignored --nocapture
```

This is a sacrificial process, not a verification gate. Before and after the
extraction below it prints, in order:

```text
deep-copy reference: prepared; entering execution
deep-copy reference: executed; retiring source
deep-copy reference: source retired; entering serialization
```

It then exits with `0xc00000fd / STATUS_STACK_OVERFLOW`, before the marker
following serialization. These markers localize the failing phase; they do
not identify an individual stack frame or prove that every serializer mode
shares the failure. The earlier optimized release reproducer passed. No
stack-size override or production tracing was introduced.

## Node writer extraction checkpoint

The serializer parent was 2,853 lines. Its recursive result-node traversal,
element emission, attributes and child-context handling move unchanged into
the private 309-line `serialization/node_writer.rs`; the parent becomes
2,560 lines. The moved implementation remains recursive at this checkpoint.

The child consumes semantic result nodes, serialization options, the existing
namespace scope and the existing budgeted string sink. It owns traversal and
node emission only. Its explicit imports point toward its serialization owner;
it does not acquire resources, compile stylesheets, execute transforms, select
host policy, or expose a public API. Namespace state and output accounting
remain with their existing owners. The parent still has decomposition pressure;
this extraction is not a claim that all serializer cohesion debt is resolved.

The unchanged extraction passes `scripts/verify.ps1`: formatting, strict
Clippy, 1,535 core tests with 49 ignored probes, all adapter tests, documentation,
local Markdown links, unsafe-surface checks and conformance integrity. The
official WASM target builds. Hard-link cache fallback warnings remain.
The separate crash reproducer still fails as described above.

## Repair follow up

Replace recursive node traversal with a bounded explicit traversal strategy,
preserving XML, HTML and XHTML byte output, namespace scope restoration,
indentation, CDATA handling, diagnostics, work charges and failure ordering.
Retain a shallow recursive differential oracle. Demonstrate the default-debug
256-deep reference without changing the harness stack, then reopen the excluded
deep-copy allocation cell and remeasure any changed peaks.

This checkpoint alone claimed no stack-safety repair, prepared-layout adoption,
timing improvement or corpus credit. AR-0027 remains Incubating and OASIS remains 2,485 / 3,173
(78.32%). The existing allocation tables remain observations of the earlier
implementation, not new measurements of this extraction.
