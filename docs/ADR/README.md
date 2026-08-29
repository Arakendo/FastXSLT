# Architecture Decision Records

ADRs record accepted, binding architectural decisions. Implementation work must
follow them unless a later accepted ADR explicitly supersedes the decision.

Proposed, unnumbered drafts live in [`Proposed/`](Proposed/). A draft has no
architectural authority until maintainers accept, number, and move it here.

## Statuses

- **Accepted** -- current binding decision.
- **Superseded** -- replaced by another named ADR.
- **Deprecated** -- retained for compatibility while being removed.

ADRs are append-oriented historical records. Correct typos and links in place,
but supersede a materially changed decision instead of rewriting its history.

## Index

- [ADR-0001: Evidence-Led Modular Monolith](ADR-0001-evidence-led-modular-monolith.md)
  -- Accepted; keep logical engine layers in one crate until concrete pressure
  justifies a structural split.
- [ADR-0002: Memory-Resident Execution](ADR-0002-memory-resident-execution.md)
  -- Accepted; host adapters import owned bytes and close handles before sealed
  snapshots, while core compilation and transformation avoid implicit disk I/O.
- [ADR-0003: Unsafe Rust Exception Policy](ADR-0003-unsafe-rust-exception-policy.md)
  -- Accepted; first-party unsafe code remains forbidden unless a narrow later
  ADR establishes necessity, invariants, containment, evidence, and removal
  criteria.
- [ADR-0004: Source Unit Cohesion, Size Pressure, and Decomposition](ADR-0004-source-unit-cohesion-size-pressure-and-decomposition.md)
  -- Accepted; source size triggers proportional review, while demonstrated
  ownership and responsibility seams justify behavior-preserving decomposition.
- [ADR-0005: Unordered Transform Sets and Host-Owned Workflow](ADR-0005-unordered-transform-sets-and-host-owned-workflow.md)
  -- Accepted; transform sets contain independent unordered requests, while the
  host sequences dependent stages and explicitly admits prior results.
- [ADR-0006: Verification Ledger Invariants](ADR-0006-verification-ledger-invariants.md)
  -- Accepted; every discovered standards case retains native identity and an
  explainable disposition, with separate selection/execution axes and conserved
  denominators across filtering, sharding, interruption, retry, and merging.
- [ADR-0007: Staged Modern Standards Profile](ADR-0007-staged-modern-standards-profile.md)
  -- Accepted; use XSLT 3.0, XPath/XDM 3.1, Serialization 3.1, XML 1.0 Fifth
  Edition, and Namespaces 1.0 Third Edition as reference semantics while
  widening an explicitly incomplete, ledger-accounted preview by feature.
- [ADR-0008: Unsafe Native .NET Workbench Boundary](ADR-0008-unsafe-native-dotnet-workbench-boundary.md)
  -- Accepted; admit a narrowly bounded unsafe buffer-copy surface only in the
  unpublished native .NET workbench, with numeric handles, panic quarantine,
  differential verification, and explicit removal criteria.
- [ADR-0009: Scalar Native Invocation Controls](ADR-0009-scalar-native-invocation-controls.md)
  -- Accepted; carry pre-dispatch cooperative cancellation and an invocation
  XSLT-instruction budget as validated scalar values without callbacks, retained
  foreign state, or another unsafe pointer operation.
- [ADR-0010: Native Active Control Handles](ADR-0010-native-active-control-handles.md)
  -- Accepted; use Rust-owned numeric control handles for active cooperative
  native cancellation, with explicit cancel/release races and no callback,
  foreign borrow, or additional unsafe block.
- [ADR-0011: Bounded Stylesheet Dependency Host Framing](ADR-0011-bounded-stylesheet-dependency-host-framing.md)
  -- Accepted; admit one explicit sealed stylesheet dependency and independent
  denial policy through each unpublished .NET workbench initialization boundary.
